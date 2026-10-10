//! [`App`], [`Window`], [`AppHandler`] and [`Error`]: the app runner. Spec:
//! `docs/specs/facade/app.md`.

use std::fmt;

use tantu_core::{Color, Point, Vec2};
use tantu_layout::BoxConstraints;
use tantu_layout::RenderConstrainedBox;
use tantu_platform::{
    ButtonState, Platform, PlatformContext, PlatformError, PlatformHandler, PointerButton,
    ScrollDelta, WindowAttributes, WindowEvent, WindowId,
};
use tantu_scene::{RenderError, Renderer, Resources, Scene};
use tantu_text::{FontFamily, TextStyles, TextSystem};
use tantu_view::{
    AnyView, BuildCx, ElementId, Paint, PaintCx, PointerEvent, PointerKind, SystemText, View,
    ViewTree,
};

/// Logical pixels scrolled per wheel line (FACADE-APP-05; Chromium's value).
const PIXELS_PER_LINE: f32 = 40.0;

/// Errors from running an app.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The platform failed (event loop or window creation).
    Platform(PlatformError),
    /// Creating a renderer failed.
    Renderer(Box<dyn std::error::Error + Send + Sync>),
    /// Rendering a frame failed (other than a lost target, which is retried).
    Render(RenderError),
    /// `run` was called without a platform or GPU renderer compiled in (features `winit`,
    /// `wgpu`).
    Unsupported(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Platform(e) => write!(f, "platform error: {e}"),
            Error::Renderer(e) => write!(f, "creating the renderer failed: {e}"),
            Error::Render(e) => write!(f, "rendering failed: {e}"),
            Error::Unsupported(what) => write!(f, "not supported in this build: {what}"),
        }
    }
}

impl std::error::Error for Error {}

/// `std::result::Result<T, tantu::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Creates the renderer for a new window.
type RendererFactory =
    Box<dyn FnMut(&mut dyn PlatformContext, WindowId) -> Result<Box<dyn Renderer>>>;

/// A window to open: title and initial logical size (default 800 × 600).
#[derive(Clone, Debug)]
pub struct Window {
    attributes: WindowAttributes,
    background: Color,
}

impl Window {
    /// A window titled `title`, 800 × 600, resizable.
    pub fn new(title: impl Into<String>) -> Self {
        Window {
            attributes: WindowAttributes::new(title),
            background: Color::WHITE,
        }
    }

    /// The initial inner size in logical pixels.
    pub fn size(self, width: f32, height: f32) -> Self {
        Window {
            attributes: self.attributes.size(width, height),
            ..self
        }
    }

    /// The smallest inner size in logical pixels.
    pub fn min_size(self, width: f32, height: f32) -> Self {
        Window {
            attributes: self.attributes.min_size(width, height),
            ..self
        }
    }

    /// The color painted behind the window's content (default `Color::WHITE`).
    pub fn background(self, color: Color) -> Self {
        Window {
            background: color,
            ..self
        }
    }

    /// Whether the user can resize the window.
    pub fn resizable(self, resizable: bool) -> Self {
        Window {
            attributes: self.attributes.resizable(resizable),
            ..self
        }
    }
}

/// Builds a window's view tree with the app's style table.
type Content = Box<dyn FnOnce(TextStyles) -> ViewTree>;

/// An app: fonts and windows, then [`run`](App::run).
pub struct App {
    windows: Vec<(Window, Content)>,
    fonts: Vec<Vec<u8>>,
    system_fonts: bool,
}

impl App {
    /// An app with no windows; the text system uses system fonts.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        App {
            windows: Vec::new(),
            fonts: Vec::new(),
            system_fonts: true,
        }
    }

    /// Opens `window` showing the view `content` builds. `content` runs once, when the window
    /// opens, with the window's reactive runtime current.
    pub fn window<V: View>(self, window: Window, content: impl FnOnce() -> V + 'static) -> Self {
        let background = window.background;
        let content: Content = Box::new(move |styles| {
            ViewTree::with_text_styles(styles, move || Background {
                color: background,
                content: AnyView::new(content()),
            })
        });
        let mut app = self;
        app.windows.push((window, content));
        app
    }

    /// Registers a font file (e.g. bundled with the app).
    pub fn font(mut self, data: Vec<u8>) -> Self {
        self.fonts.push(data);
        self
    }

    /// Uses only registered fonts, not the system's (tests, reproducible output).
    pub fn without_system_fonts(mut self) -> Self {
        self.system_fonts = false;
        self
    }

    /// Runs on winit with the wgpu renderer until the last window closes.
    pub fn run(self) -> Result<()> {
        #[cfg(all(feature = "winit", feature = "wgpu"))]
        {
            let platform = tantu_platform_winit::WinitPlatform::new().map_err(Error::Platform)?;
            self.run_with(platform, |cx, window| {
                let target = cx
                    .surface_target(window)
                    .ok_or(Error::Unsupported("a window without a surface"))?;
                let size = cx
                    .inner_size(window)
                    .unwrap_or(tantu_platform::PhysicalSize::new(1, 1));
                let renderer =
                    tantu_render_wgpu::WgpuRenderer::for_window(target, size.width, size.height)
                        .map_err(|e| Error::Renderer(Box::new(e)))?;
                Ok(Box::new(renderer) as Box<dyn Renderer>)
            })
        }
        #[cfg(not(all(feature = "winit", feature = "wgpu")))]
        {
            let _ = self;
            Err(Error::Unsupported(
                "App::run needs the `winit` and `wgpu` features",
            ))
        }
    }

    /// Runs on `platform`, with `renderer` called once per window to create its renderer.
    pub fn run_with(
        self,
        platform: impl Platform,
        renderer: impl FnMut(&mut dyn PlatformContext, WindowId) -> Result<Box<dyn Renderer>> + 'static,
    ) -> Result<()> {
        let mut handler = self.handler(renderer);
        platform.run(&mut handler).map_err(Error::Platform)?;
        handler.finish()
    }

    /// The runner as a platform handler, for shells that drive handlers themselves (and for
    /// `FakePlatform::run_logged` in tests).
    pub fn handler(
        self,
        renderer: impl FnMut(&mut dyn PlatformContext, WindowId) -> Result<Box<dyn Renderer>> + 'static,
    ) -> AppHandler {
        let mut text = if self.system_fonts {
            TextSystem::new()
        } else {
            TextSystem::without_system_fonts()
        };
        let mut default_family = None;
        for font in self.fonts {
            let families = text.register_font(font);
            if default_family.is_none() {
                default_family = families.into_iter().next();
            }
        }
        // The first registered font is the default family (FACADE-APP-09).
        if let Some(family) = default_family {
            text.set_default_family(FontFamily::Named(family.into()));
        }
        AppHandler {
            text,
            resources: Resources::new(),
            pending: self.windows,
            windows: Vec::new(),
            factory: Box::new(renderer),
            error: None,
            modifiers: tantu_view::Modifiers::default(),
        }
    }
}

/// The window's background under its content (FACADE-APP-10): a box that passes the window's
/// constraints to the content and fills its bounds with `color`.
struct Background {
    color: Color,
    content: AnyView,
}

impl View for Background {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::default()),
            [self.content],
        );
        cx.set_paint(id, BackgroundPaint(self.color));
        id
    }
}

/// Fills the element's bounds.
struct BackgroundPaint(Color);

impl Paint for BackgroundPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let rect = tantu_core::Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        cx.scene().fill_rect(rect, self.0);
    }
}

/// One open window.
struct WindowState {
    id: WindowId,
    tree: ViewTree,
    renderer: Box<dyn Renderer>,
    scene: Scene,
}

/// The app runner (a [`PlatformHandler`]).
pub struct AppHandler {
    text: TextSystem,
    resources: Resources,
    /// Windows not opened yet (until `started`).
    pending: Vec<(Window, Content)>,
    windows: Vec<WindowState>,
    factory: RendererFactory,
    error: Option<Error>,
    /// The modifiers held, from `ModifiersChanged` (FACADE-APP-11).
    modifiers: tantu_view::Modifiers,
}

impl AppHandler {
    /// How the run ended: the first error, or `Ok(())`.
    pub fn finish(self) -> Result<()> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Records the first error and ends the run.
    fn fail(&mut self, cx: &mut dyn PlatformContext, error: Error) {
        if self.error.is_none() {
            self.error = Some(error);
        }
        cx.exit();
    }

    /// Opens one window: platform window, view tree, renderer (FACADE-APP-01).
    fn open(&mut self, cx: &mut dyn PlatformContext, window: Window, content: Content) {
        let id = match cx.create_window(&window.attributes) {
            Ok(id) => id,
            Err(error) => return self.fail(cx, Error::Platform(error)),
        };
        let tree = content(self.text.styles());
        let mut renderer = match (self.factory)(cx, id) {
            Ok(renderer) => renderer,
            Err(error) => return self.fail(cx, error),
        };
        fit(renderer.as_mut(), cx, id);
        cx.request_redraw(id);
        self.windows.push(WindowState {
            id,
            tree,
            renderer,
            scene: Scene::new(),
        });
    }

    /// Runs a frame and renders it (FACADE-APP-02, -07).
    fn redraw(&mut self, cx: &mut dyn PlatformContext, index: usize) {
        let state = &mut self.windows[index];
        let (Some(size), Some(scale)) = (cx.inner_size(state.id), cx.scale_factor(state.id)) else {
            return;
        };
        let constraints = BoxConstraints::tight(size.to_logical(scale));
        let mut text = SystemText {
            system: &mut self.text,
            resources: &mut self.resources,
        };
        state.tree.frame(constraints, &mut text, &mut state.scene);
        match state.renderer.render(&state.scene, &self.resources) {
            Ok(report) => {
                if !report.is_clean() {
                    tracing::debug!(?report, "frame rendered with problems");
                }
            }
            Err(RenderError::TargetLost) => cx.request_redraw(state.id),
            Err(error) => self.fail(cx, Error::Render(error)),
        }
    }

    /// Asks for a redraw of every window whose tree changed (FACADE-APP-03).
    fn request_frames(&self, cx: &mut dyn PlatformContext) {
        for state in &self.windows {
            if state.tree.needs_frame() {
                cx.request_redraw(state.id);
            }
        }
    }
}

/// Sizes `renderer` to the window's physical size and scale factor.
fn fit(renderer: &mut dyn Renderer, cx: &mut dyn PlatformContext, window: WindowId) {
    if let (Some(size), Some(scale)) = (cx.inner_size(window), cx.scale_factor(window)) {
        renderer.resize(size.width, size.height, scale);
    }
}

/// The view layer's button for a platform button (FACADE-APP-05).
fn button(button: PointerButton) -> tantu_view::PointerButton {
    match button {
        PointerButton::Primary => tantu_view::PointerButton::Primary,
        PointerButton::Secondary => tantu_view::PointerButton::Secondary,
        PointerButton::Middle => tantu_view::PointerButton::Middle,
        PointerButton::Back => tantu_view::PointerButton::Other(3),
        PointerButton::Forward => tantu_view::PointerButton::Other(4),
        PointerButton::Other(n) => tantu_view::PointerButton::Other(n),
    }
}

/// The view-layer pointer event for a platform event, if it is one (FACADE-APP-05).
fn pointer_event(event: &WindowEvent) -> Option<PointerEvent> {
    let (kind, position) = match *event {
        WindowEvent::PointerMoved { position, .. } => (PointerKind::Move, position),
        WindowEvent::PointerButton {
            button: b,
            state,
            position,
            ..
        } => {
            let kind = match state {
                ButtonState::Pressed => PointerKind::Down(button(b)),
                ButtonState::Released => PointerKind::Up(button(b)),
            };
            (kind, position)
        }
        WindowEvent::Wheel {
            delta, position, ..
        } => {
            let delta = match delta {
                ScrollDelta::Lines { x, y } => Vec2::new(x * PIXELS_PER_LINE, y * PIXELS_PER_LINE),
                ScrollDelta::Pixels(delta) => delta,
            };
            (PointerKind::Scroll(delta), position)
        }
        WindowEvent::PointerLeft { .. } => (PointerKind::Leave, Point::ZERO),
        _ => return None,
    };
    Some(PointerEvent { kind, position })
}

/// The view layer's key for a platform key; `None` for modifier, lock and unidentified keys,
/// which aren't dispatched (FACADE-APP-11).
fn logical_key(key: &tantu_platform::Key) -> Option<tantu_view::LogicalKey> {
    use tantu_platform::{Key, NamedKey as P};
    use tantu_view::{LogicalKey, NamedKey as V};
    let named = match key {
        Key::Character(text) => return Some(LogicalKey::Character(text.as_str().into())),
        Key::Unidentified => return None,
        Key::Named(named) => named,
    };
    let view = match named {
        P::Enter => V::Enter,
        P::Tab => V::Tab,
        P::Escape => V::Escape,
        P::Backspace => V::Backspace,
        P::Delete => V::Delete,
        P::Insert => V::Insert,
        P::ArrowLeft => V::ArrowLeft,
        P::ArrowRight => V::ArrowRight,
        P::ArrowUp => V::ArrowUp,
        P::ArrowDown => V::ArrowDown,
        P::Home => V::Home,
        P::End => V::End,
        P::PageUp => V::PageUp,
        P::PageDown => V::PageDown,
        P::F1 => V::F(1),
        P::F2 => V::F(2),
        P::F3 => V::F(3),
        P::F4 => V::F(4),
        P::F5 => V::F(5),
        P::F6 => V::F(6),
        P::F7 => V::F(7),
        P::F8 => V::F(8),
        P::F9 => V::F(9),
        P::F10 => V::F(10),
        P::F11 => V::F(11),
        P::F12 => V::F(12),
        P::F13 => V::F(13),
        P::F14 => V::F(14),
        P::F15 => V::F(15),
        P::F16 => V::F(16),
        P::F17 => V::F(17),
        P::F18 => V::F(18),
        P::F19 => V::F(19),
        P::F20 => V::F(20),
        P::F21 => V::F(21),
        P::F22 => V::F(22),
        P::F23 => V::F(23),
        P::F24 => V::F(24),
        P::Shift | P::Control | P::Alt | P::Super | P::CapsLock | P::NumLock | P::ScrollLock => {
            return None;
        }
        _ => V::Other,
    };
    Some(LogicalKey::Named(view))
}

/// The view-layer key event for a platform key event (FACADE-APP-11).
fn key_event(
    event: &tantu_platform::KeyEvent,
    modifiers: tantu_view::Modifiers,
) -> Option<tantu_view::KeyEvent> {
    Some(tantu_view::KeyEvent {
        key: logical_key(&event.key)?,
        pressed: event.state == ButtonState::Pressed,
        repeat: event.repeat,
        modifiers,
        text: event.text.as_deref().map(Into::into),
    })
}

impl PlatformHandler for AppHandler {
    fn started(&mut self, cx: &mut dyn PlatformContext) {
        if self.pending.is_empty() {
            cx.exit();
            return;
        }
        for (window, content) in std::mem::take(&mut self.pending) {
            self.open(cx, window, content);
            if self.error.is_some() {
                return;
            }
        }
    }

    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent) {
        let Some(index) = self.windows.iter().position(|w| w.id == window) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                // Dropping the tree disposes its scope (FACADE-APP-06).
                cx.close_window(window);
                self.windows.remove(index);
                if self.windows.is_empty() {
                    cx.exit();
                }
                return;
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged(_) => {
                let state = &mut self.windows[index];
                fit(state.renderer.as_mut(), cx, window);
                cx.request_redraw(window);
            }
            WindowEvent::RedrawRequested => self.redraw(cx, index),
            WindowEvent::ModifiersChanged(m) => {
                self.modifiers = tantu_view::Modifiers {
                    shift: m.shift,
                    control: m.control,
                    alt: m.alt,
                    super_key: m.super_key,
                };
            }
            WindowEvent::Keyboard(ref key) => {
                if let Some(event) = key_event(key, self.modifiers) {
                    self.windows[index].tree.dispatch_key(event);
                }
            }
            ref other => {
                if let Some(pointer) = pointer_event(other) {
                    self.windows[index].tree.dispatch_pointer(pointer);
                }
            }
        }
        self.request_frames(cx);
    }

    fn idle(&mut self, cx: &mut dyn PlatformContext) {
        self.request_frames(cx);
    }
}
