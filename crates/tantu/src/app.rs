//! [`App`], [`Window`], [`AppHandler`] and [`Error`]: the app runner. Spec:
//! `docs/specs/facade/app.md`.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use tantu_a11y::AccessibilityTree;
use tantu_core::{Color, Point, Rect, Size, Vec2};
use tantu_layout::BoxConstraints;
use tantu_layout::RenderConstrainedBox;
use tantu_platform::{
    AccessibilityEvent, ButtonState, Platform, PlatformContext, PlatformError, PlatformHandler,
    PointerButton, ScrollDelta, WindowAttributes, WindowEvent, WindowId,
};
use tantu_reactive::{Runtime, Scope, Signal, Wake};
use tantu_scene::{RenderError, Renderer, Resources, Scene};
use tantu_text::{FontFamily, TextSystem};
use tantu_theme::{Theme, ThemeProvider};
use tantu_view::{
    AnyView, BuildCx, ElementId, Paint, PaintCx, PointerEvent, PointerKind, Prop, SystemText, View,
    ViewTree,
};

use crate::desktop::{Desktop, DesktopQueue, DesktopRequest, FileDone, MessageDone};
use crate::windows::{Content, Queue, Request, Windows};

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
    /// The background color; `None` follows the theme's `surface` (FACADE-APP-13).
    background: Option<Color>,
    /// Asked when the user wants to close the window (FACADE-WIN-04).
    on_close: Option<CloseHook>,
}

/// A window's close veto.
#[derive(Clone)]
struct CloseHook(Rc<RefCell<dyn FnMut() -> bool>>);

impl fmt::Debug for CloseHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CloseHook")
    }
}

impl Window {
    /// A window titled `title`, 800 × 600, resizable.
    pub fn new(title: impl Into<String>) -> Self {
        Window {
            attributes: WindowAttributes::new(title),
            background: None,
            on_close: None,
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
            background: Some(color),
            ..self
        }
    }

    /// Called when the user asks to close the window; returning `false` keeps it open
    /// (FACADE-WIN-04). The default allows closing.
    pub fn on_close_request(self, f: impl FnMut() -> bool + 'static) -> Self {
        Window {
            on_close: Some(CloseHook(Rc::new(RefCell::new(f)))),
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

/// Provides one app-owned value at a window's root (FACADE-WIN-02).
type Provider = Rc<dyn Fn(&mut BuildCx<'_>)>;

/// Builds an app-owned value, in the app scope, and its provider.
type Init = Box<dyn FnOnce() -> Provider>;

/// An app: fonts and windows, then [`run`](App::run).
pub struct App {
    windows: Vec<(Window, Content)>,
    fonts: Vec<Vec<u8>>,
    system_fonts: bool,
    inits: Vec<Init>,
}

impl App {
    /// An app with no windows; the text system uses system fonts.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        App {
            windows: Vec::new(),
            fonts: Vec::new(),
            system_fonts: true,
            inits: Vec::new(),
        }
    }

    /// Opens `window` showing the view `content` builds. `content` runs once, when the window
    /// opens, with the app's reactive runtime current and the window's root scope as owner.
    pub fn window<V: View>(self, window: Window, content: impl FnOnce() -> V + 'static) -> Self {
        let content: Content = Box::new(move || AnyView::new(content()));
        let mut app = self;
        app.windows.push((window, content));
        app
    }

    /// Builds a value once, in the app scope, before the first window opens, and provides it
    /// as an inherited value at the root of every window (`cx.inherit::<T>()`). Signals
    /// created in `init` live as long as the app (FACADE-WIN-02).
    pub fn provide<T: Clone + 'static>(self, init: impl FnOnce() -> T + 'static) -> Self {
        let init: Init = Box::new(move || {
            let value = init();
            Rc::new(move |cx: &mut BuildCx<'_>| cx.provide(value.clone())) as Provider
        });
        let mut app = self;
        app.inits.push(init);
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
        let app_name = self
            .windows
            .first()
            .map(|(window, _)| window.attributes.title.clone())
            .unwrap_or_default();
        // One runtime for every window, and the app scope (FACADE-WIN-01, -02; ADR 0016).
        let runtime = Rc::new(Runtime::new());
        let (app_scope, queue, open, desktop) = runtime.enter(|| {
            let scope = Scope::new();
            let (queue, open, desktop) = scope.run(|| {
                (
                    tantu_reactive::signal(Rc::new(Queue::default())),
                    tantu_reactive::signal(Vec::<u64>::new()),
                    tantu_reactive::signal(Rc::new(DesktopQueue::default())),
                )
            });
            (scope, queue, open, desktop)
        });
        AppHandler {
            app_name,
            text,
            resources: Resources::new(),
            pending: self.windows,
            inits: self.inits,
            providers: Rc::from(Vec::new()),
            windows: Vec::new(),
            factory: Box::new(renderer),
            error: None,
            wake: None,
            runtime,
            app_scope,
            queue,
            open,
            ticked: false,
            desktop,
            dialogs: std::collections::HashMap::new(),
            menus: Rc::default(),
            tray: crate::tray::TrayHandle(Rc::default()),
        }
    }
}

/// The window's background under its content (FACADE-APP-10): a box that passes the window's
/// constraints to the content and fills its bounds with `color`, or with the theme's
/// `surface` when no color was set. With the `default-theme` feature it also provides the
/// window's `Signal<Theme>`, holding the light theme (FACADE-APP-13).
struct Background {
    color: Option<Color>,
    content: AnyView,
    /// The app-provided values, the app's windows and the desktop services (FACADE-WIN-02,
    /// -03, FACADE-DESK-01).
    providers: Rc<[Provider]>,
    windows: Windows,
    desktop: Desktop,
    /// The native menu host, where the platform has native menus (FACADE-APP-21).
    menu_host: Option<tantu_widgets::NativeMenus>,
    /// The app's tray, where the platform has one (FACADE-TRAY-01).
    tray: Option<crate::tray::TrayHandle>,
}

impl View for Background {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        // On the root, so the whole window inherits them.
        for provide in self.providers.iter() {
            provide(cx);
        }
        cx.provide(self.windows);
        cx.provide(self.desktop);
        if let Some(host) = self.menu_host {
            cx.provide(host);
        }
        if let Some(tray) = self.tray {
            cx.provide(tray);
        }
        #[cfg(feature = "default-theme")]
        let theme: Option<Signal<Theme>> = Some(tantu_reactive::signal(Theme::light()));
        #[cfg(not(feature = "default-theme"))]
        let theme: Option<Signal<Theme>> = None;
        let content = match theme {
            Some(theme) => AnyView::new(ThemeProvider::new(theme, self.content)),
            None => self.content,
        };
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::default()),
            [content],
        );
        cx.set_paint(
            id,
            BackgroundPaint(self.color.unwrap_or(Theme::light().colors.surface)),
        );
        if let (None, Some(theme)) = (self.color, theme) {
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || theme.with(|t| t.colors.surface))),
                |e, color| {
                    e.update_paint::<BackgroundPaint>(|p| p.0 = color);
                },
            );
        }
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
    modifiers: tantu_view::Modifiers,
    id: WindowId,
    /// The app's name for it (FACADE-WIN-03).
    token: u64,
    /// The close veto (FACADE-WIN-04).
    on_close: Option<CloseHook>,
    tree: ViewTree,
    renderer: Box<dyn Renderer>,
    scene: Scene,
    /// The last rendered frame, to diff the next one against (FACADE-APP-22).
    previous: Scene,
    /// Whether the next frame must repaint everything (first frame, resize, scale change,
    /// lost target).
    repaint_all: bool,
    /// The IME area last told to the platform (FACADE-APP-12).
    ime_area: Option<Rect>,
    /// The accessibility tree builder while assistive technology is active (FACADE-APP-15).
    a11y: Option<AccessibilityTree>,
    /// The cursor last sent to the platform (FACADE-APP-17).
    cursor: tantu_platform::CursorIcon,
    /// Whether the pointer is over the window; nothing is sent while it is outside.
    pointer_inside: bool,
}

impl WindowState {
    /// Sends the tree's cursor when it differs from the last one sent (FACADE-APP-17).
    fn sync_cursor(&mut self, cx: &mut dyn PlatformContext) {
        if !self.pointer_inside {
            return;
        }
        let cursor = platform_cursor(self.tree.cursor());
        if cursor != self.cursor {
            cx.set_cursor(self.id, cursor);
            self.cursor = cursor;
        }
    }
}

/// The platform's clipboard as views see it (FACADE-APP-18).
struct PlatformClipboard(Rc<dyn tantu_platform::Clipboard>);

impl tantu_view::Clipboard for PlatformClipboard {
    fn read_text(&self) -> Option<String> {
        self.0.read_text()
    }

    fn write_text(&self, text: &str) -> bool {
        self.0.write_text(text)
    }

    // Images cross as the platform's `ClipboardImage` (FACADE-APP-20).
    fn read_image(&self) -> Option<tantu_scene::ImageData> {
        let image = self.0.read_image()?;
        tantu_scene::ImageData::rgba8(image.width, image.height, image.rgba).ok()
    }

    fn write_image(&self, image: &tantu_scene::ImageData) -> bool {
        self.0.write_image(&tantu_platform::ClipboardImage {
            width: image.width(),
            height: image.height(),
            rgba: image.pixels().to_vec(),
        })
    }

    fn write_html(&self, html: &str, alt_text: &str) -> bool {
        self.0.write_html(html, alt_text)
    }

    fn read_html(&self) -> Option<String> {
        self.0.read_html()
    }

    fn read_primary(&self) -> Option<String> {
        self.0.read_primary()
    }

    fn write_primary(&self, text: &str) -> bool {
        self.0.write_primary(text)
    }
}

/// The view's cursor as the platform's.
fn platform_cursor(cursor: tantu_view::CursorIcon) -> tantu_platform::CursorIcon {
    use tantu_platform::CursorIcon as P;
    use tantu_view::CursorIcon as V;
    match cursor {
        V::Default => P::Default,
        V::Pointer => P::Pointer,
        V::Text => P::Text,
        V::Grab => P::Grab,
        V::Grabbing => P::Grabbing,
        V::NotAllowed => P::NotAllowed,
        V::ResizeHorizontal => P::ResizeHorizontal,
        V::ResizeVertical => P::ResizeVertical,
    }
}

/// The platform's IME event as the view's (FACADE-APP-12).
fn ime_event(ime: &tantu_platform::ImeEvent) -> tantu_view::ImeEvent {
    use tantu_platform::ImeEvent as P;
    use tantu_view::ImeEvent as V;
    match ime {
        P::Enabled => V::Enabled,
        P::Preedit { text, cursor } => V::Preedit {
            text: text.clone(),
            cursor: *cursor,
        },
        P::Commit(text) => V::Commit(text.clone()),
        P::Disabled => V::Disabled,
    }
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
    /// The wake-up deadline last passed to the platform (FACADE-APP-14).
    wake: Option<std::time::Instant>,
    /// The accessibility tree's window name: the first window's title (FACADE-APP-15).
    app_name: String,
    /// App-state builders, run in the app scope at start (FACADE-WIN-02).
    inits: Vec<Init>,
    /// The app-provided values every window inherits.
    providers: Rc<[Provider]>,
    /// The runtime every window's tree shares (FACADE-WIN-01).
    runtime: Rc<Runtime>,
    /// Owns app state for the whole run (FACADE-WIN-02).
    app_scope: Scope,
    /// Requests from views (FACADE-WIN-03).
    queue: Signal<Rc<Queue>>,
    /// The tokens of the open windows.
    open: Signal<Vec<u64>>,
    /// Whether the runtime was ticked since the last `idle` (FACADE-WIN-06).
    ticked: bool,
    /// Desktop requests from views (FACADE-DESK-01).
    desktop: Signal<Rc<DesktopQueue>>,
    /// Open dialogs' continuations, by id, with their window's token.
    dialogs: std::collections::HashMap<tantu_platform::DialogId, (u64, DialogDone)>,
    /// Native menu bars, by window token (FACADE-APP-21).
    menus: Menus,
    /// The app's tray (FACADE-TRAY-01).
    tray: crate::tray::TrayHandle,
}

/// A window's native menu bar as the runner tracks it (FACADE-APP-21).
#[derive(Default)]
struct WindowMenu {
    /// Menus sent by the window's `MenuBar` and not installed yet.
    pending: Option<Vec<tantu_widgets::NativeMenuEntry>>,
    /// What is installed, to re-send when the window gains focus.
    installed: Option<tantu_platform::NativeMenu>,
    /// The installed items' actions, by id.
    actions: std::collections::HashMap<u64, Rc<dyn Fn()>>,
}

/// Every window's menu bar, by window token.
type Menus = Rc<RefCell<std::collections::HashMap<u64, WindowMenu>>>;

/// The runner as a window's `NativeMenuHost` (FACADE-APP-21): it keeps the menus until
/// the runner installs them after the event.
struct MenuHost {
    token: u64,
    menus: Menus,
}

impl tantu_widgets::NativeMenuHost for MenuHost {
    fn set_menus(&self, menus: Vec<tantu_widgets::NativeMenuEntry>) -> bool {
        self.menus
            .borrow_mut()
            .entry(self.token)
            .or_default()
            .pending = Some(menus);
        true
    }
}

/// The platform's description of `entries`, numbering items from `next` and keeping their
/// actions.
fn native_menu_items(
    entries: Vec<tantu_widgets::NativeMenuEntry>,
    next: &mut u64,
    actions: &mut std::collections::HashMap<u64, Rc<dyn Fn()>>,
) -> Vec<tantu_platform::NativeMenuItem> {
    use tantu_platform::NativeMenuItem;
    use tantu_widgets::NativeMenuEntryKind;
    entries
        .into_iter()
        .map(|entry| match entry.kind {
            NativeMenuEntryKind::Separator => NativeMenuItem::Separator,
            NativeMenuEntryKind::Submenu(children) => NativeMenuItem::Submenu {
                label: entry.label,
                enabled: entry.enabled,
                items: native_menu_items(children, next, actions),
            },
            NativeMenuEntryKind::Item => {
                *next += 1;
                if let Some(action) = entry.on_select {
                    actions.insert(*next, action);
                }
                NativeMenuItem::Item {
                    id: *next,
                    label: entry.label,
                    enabled: entry.enabled,
                    checked: entry.checked,
                    shortcut: entry.shortcut,
                }
            }
        })
        .collect()
}

/// A native dialog's continuation (FACADE-DESK-01).
enum DialogDone {
    File(FileDone),
    Message(MessageDone),
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

    /// Opens one window: platform window, view tree, renderer (FACADE-APP-01, FACADE-WIN-03).
    fn open(&mut self, cx: &mut dyn PlatformContext, window: Window, content: Content, token: u64) {
        let id = match cx.create_window(&window.attributes) {
            Ok(id) => id,
            Err(error) => return self.fail(cx, Error::Platform(error)),
        };
        // Open from the moment its content builds, so it counts itself.
        let open = self.open;
        self.runtime.enter(|| open.update(|o| o.push(token)));
        let windows = Windows {
            queue: self.queue,
            open,
            current: token,
        };
        let desktop = Desktop {
            queue: self.desktop,
            window: token,
            capabilities: cx.native_capabilities(),
        };
        let menu_host = desktop.capabilities.menus.then(|| {
            tantu_widgets::NativeMenus(Rc::new(MenuHost {
                token,
                menus: Rc::clone(&self.menus),
            }))
        });
        let tray = desktop.capabilities.tray.then(|| self.tray.clone());
        let (color, providers) = (window.background, self.providers.clone());
        let mut tree =
            ViewTree::with_runtime(self.runtime.clone(), self.text.styles(), move || {
                Background {
                    color,
                    content: content(),
                    providers,
                    windows,
                    desktop,
                    menu_host,
                    tray,
                }
            });
        // The platform's clipboard for the window's views (FACADE-APP-18).
        if let Some(clipboard) = cx.clipboard() {
            tree.set_clipboard(Rc::new(PlatformClipboard(clipboard)));
        }
        let mut renderer = match (self.factory)(cx, id) {
            Ok(renderer) => renderer,
            Err(error) => return self.fail(cx, error),
        };
        fit(renderer.as_mut(), cx, id);
        cx.request_redraw(id);
        self.windows.push(WindowState {
            modifiers: tantu_view::Modifiers::default(),
            id,
            token,
            on_close: window.on_close,
            tree,
            renderer,
            scene: Scene::new(),
            previous: Scene::new(),
            repaint_all: true,
            ime_area: None,
            a11y: None,
            cursor: tantu_platform::CursorIcon::Default,
            pointer_inside: false,
        });
    }

    /// Closes the window at `index`: platform window, tree (its root scope), the open list;
    /// the last one ends the run (FACADE-APP-06, FACADE-WIN-04).
    fn close(&mut self, cx: &mut dyn PlatformContext, index: usize) {
        let state = self.windows.remove(index);
        cx.close_window(state.id);
        let token = state.token;
        drop(state);
        // Its dialogs' answers and its menu bar go with it (FACADE-DESK-01, FACADE-APP-21).
        self.dialogs.retain(|_, (owner, _)| *owner != token);
        self.menus.borrow_mut().remove(&token);
        let open = self.open;
        self.runtime
            .enter(|| open.update(|o| o.retain(|t| *t != token)));
        if self.windows.is_empty() {
            cx.exit();
        }
    }

    /// Applies what views asked for since the last call (FACADE-WIN-03..05).
    fn apply_requests(&mut self, cx: &mut dyn PlatformContext) {
        loop {
            let queue = self.queue;
            let requests = self
                .runtime
                .enter(|| queue.try_with(|queue| queue.take()))
                .unwrap_or_default();
            let desktop = self.desktop;
            let desktop_requests = self
                .runtime
                .enter(|| desktop.try_with(|queue| queue.take()))
                .unwrap_or_default();
            if (requests.is_empty() && desktop_requests.is_empty()) || self.error.is_some() {
                self.install_menus(cx);
                self.install_tray(cx);
                return;
            }
            for request in desktop_requests {
                self.apply_desktop(cx, request);
            }
            for request in requests {
                match request {
                    Request::Open {
                        token,
                        window,
                        content,
                    } => self.open(cx, window, content, token),
                    Request::Close(token) => {
                        if let Some(index) = self.windows.iter().position(|w| w.token == token) {
                            self.close(cx, index);
                        }
                    }
                    Request::Title(token, title) => {
                        if let Some(state) = self.windows.iter().find(|w| w.token == token) {
                            cx.set_title(state.id, &title);
                        }
                    }
                }
                if self.error.is_some() {
                    return;
                }
            }
        }
    }

    /// Shows, replaces or removes the tray as the views asked (FACADE-TRAY-01, -02).
    fn install_tray(&mut self, cx: &mut dyn PlatformContext) {
        let mut state = self.tray.0.borrow_mut();
        let Some(change) = state.pending.take() else {
            return;
        };
        match change {
            Some(request) => {
                let mut actions = std::collections::HashMap::new();
                let tray = tantu_platform::Tray {
                    image: request.image,
                    tooltip: request.tooltip,
                    menu: native_menu_items(request.entries, &mut 0, &mut actions),
                };
                cx.set_tray(Some(tray));
                state.actions = actions;
                state.on_click = request.on_click;
            }
            None => {
                cx.set_tray(None);
                state.actions.clear();
                state.on_click = None;
            }
        }
    }

    /// Installs the menu bars views sent since the last call (FACADE-APP-21).
    fn install_menus(&mut self, cx: &mut dyn PlatformContext) {
        let mut menus = self.menus.borrow_mut();
        for state in &self.windows {
            let Some(menu) = menus.get_mut(&state.token) else {
                continue;
            };
            let Some(entries) = menu.pending.take() else {
                continue;
            };
            let mut actions = std::collections::HashMap::new();
            let native = tantu_platform::NativeMenu {
                menus: native_menu_items(entries, &mut 0, &mut actions),
            };
            cx.set_menu_bar(state.id, Some(native.clone()));
            menu.installed = Some(native);
            menu.actions = actions;
        }
    }

    /// Starts a dialog or shows a notification for a view; an unsupported dialog answers at
    /// once (FACADE-DESK-01, -02).
    fn apply_desktop(&mut self, cx: &mut dyn PlatformContext, request: DesktopRequest) {
        let window = |token: u64| self.windows.iter().find(|w| w.token == token).map(|w| w.id);
        match request {
            DesktopRequest::File {
                token,
                request,
                done,
            } => {
                let Some(id) = window(token) else { return };
                match cx.file_dialog(id, request) {
                    Some(dialog) => {
                        self.dialogs.insert(dialog, (token, DialogDone::File(done)));
                    }
                    None => self.runtime.enter(|| done(Vec::new())),
                }
            }
            DesktopRequest::Message {
                token,
                request,
                done,
            } => {
                let Some(id) = window(token) else { return };
                let buttons = request.buttons;
                match cx.message_dialog(id, request) {
                    Some(dialog) => {
                        self.dialogs
                            .insert(dialog, (token, DialogDone::Message(done)));
                    }
                    None => self
                        .runtime
                        .enter(|| done(crate::desktop::unanswered(buttons))),
                }
            }
            DesktopRequest::Notify(notification) => {
                if !cx.notify(&notification) {
                    tracing::debug!(?notification, "the notification wasn't shown");
                }
            }
        }
    }

    /// Ticks the app's runtime unless it was ticked since the last `idle` (FACADE-WIN-06).
    /// The caller asks for the frames that changed.
    fn tick(&mut self, cx: &mut dyn PlatformContext) {
        if !self.ticked {
            self.ticked = true;
            self.runtime.tick(cx.now());
        }
    }

    /// Runs a frame and renders it (FACADE-APP-02, -07).
    fn redraw(&mut self, cx: &mut dyn PlatformContext, index: usize) {
        // Timers and tickers run first, so what they change is in this frame (FACADE-APP-14,
        // FACADE-WIN-06).
        self.tick(cx);
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
        // Keep the platform's IME in step with the focused element's area (FACADE-APP-12).
        let area = state.tree.ime_area();
        if area != state.ime_area {
            match area {
                Some(rect) => {
                    if state.ime_area.is_none() {
                        cx.set_ime_allowed(state.id, true);
                    }
                    cx.set_ime_cursor_area(
                        state.id,
                        Point::new(rect.left, rect.top),
                        Size::new(rect.width(), rect.height()),
                    );
                }
                None => cx.set_ime_allowed(state.id, false),
            }
            state.ime_area = area;
        }
        // A frame can move content under a still pointer (FACADE-APP-17).
        state.sync_cursor(cx);
        // Damage against the last frame, unless everything must repaint (FACADE-APP-22).
        if !std::mem::take(&mut state.repaint_all) {
            state.scene.diff_damage(&state.previous);
        }
        match state.renderer.render(&state.scene, &self.resources) {
            Ok(report) => {
                if !report.is_clean() {
                    tracing::debug!(?report, "frame rendered with problems");
                }
                std::mem::swap(&mut state.scene, &mut state.previous);
            }
            Err(RenderError::TargetLost) => {
                state.repaint_all = true;
                cx.request_redraw(state.id);
            }
            Err(error) => return self.fail(cx, Error::Render(error)),
        }
        // Every frame of an active window sends its tree (FACADE-APP-15).
        if let Some(a11y) = &mut state.a11y {
            cx.update_accessibility(state.id, a11y.update(&state.tree));
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
    Some(PointerEvent {
        kind,
        position,
        modifiers: Default::default(),
    })
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
        // App state first, in the app scope (FACADE-WIN-02).
        let (runtime, scope) = (self.runtime.clone(), self.app_scope);
        let providers: Vec<Provider> = std::mem::take(&mut self.inits)
            .into_iter()
            .map(|init| runtime.enter(|| scope.run(init)))
            .collect();
        self.providers = Rc::from(providers);
        for (window, content) in std::mem::take(&mut self.pending) {
            let queue = self.queue;
            let token = self
                .runtime
                .enter(|| queue.with_untracked(|queue| queue.token()));
            self.open(cx, window, content, token);
            if self.error.is_some() {
                return;
            }
        }
        self.apply_requests(cx);
    }

    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent) {
        let Some(index) = self.windows.iter().position(|w| w.id == window) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                // The window's veto first (FACADE-WIN-04); dropping the tree disposes its
                // scope (FACADE-APP-06).
                let allowed = match self.windows[index].on_close.clone() {
                    Some(CloseHook(hook)) => self.runtime.enter(|| (hook.borrow_mut())()),
                    None => true,
                };
                if allowed {
                    self.close(cx, index);
                }
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged(_) => {
                let state = &mut self.windows[index];
                state.repaint_all = true;
                fit(state.renderer.as_mut(), cx, window);
                cx.request_redraw(window);
            }
            WindowEvent::RedrawRequested => {
                self.redraw(cx, index);
                // Only this window, if its own frame left it dirty: other windows may be due
                // in this round already, and `idle` asks for the rest (FACADE-WIN-06).
                self.apply_requests(cx);
                if let Some(state) = self.windows.get(index) {
                    if state.id == window && state.tree.needs_frame() {
                        cx.request_redraw(window);
                    }
                }
                return;
            }
            WindowEvent::ModifiersChanged(m) => {
                self.windows[index].modifiers = tantu_view::Modifiers {
                    shift: m.shift,
                    control: m.control,
                    alt: m.alt,
                    super_key: m.super_key,
                };
            }
            WindowEvent::Keyboard(ref key) => {
                if let Some(event) = key_event(key, self.windows[index].modifiers) {
                    self.windows[index].tree.dispatch_key(event);
                }
            }
            WindowEvent::Ime(ref ime) => {
                self.windows[index].tree.dispatch_ime(ime_event(ime));
            }
            WindowEvent::FilesHovered {
                ref paths,
                position,
            } => {
                // FACADE-APP-19: file drops go to the window's tree.
                self.windows[index]
                    .tree
                    .dispatch_file_drop(tantu_view::FileDrop::Hovered {
                        paths: paths.clone(),
                        position,
                    });
            }
            WindowEvent::FileDialogClosed { id, ref paths } => {
                // FACADE-DESK-01: the view's continuation, once.
                if let Some((_, DialogDone::File(done))) = self.dialogs.remove(&id) {
                    let paths = paths.clone();
                    self.runtime.enter(|| done(paths));
                }
            }
            WindowEvent::MenuItemActivated { id } => {
                // FACADE-APP-21: the item's action, with the runtime current.
                let token = self.windows[index].token;
                let action = self
                    .menus
                    .borrow()
                    .get(&token)
                    .and_then(|menu| menu.actions.get(&id).cloned());
                if let Some(action) = action {
                    self.runtime.enter(|| action());
                }
            }
            WindowEvent::Focused(true) => {
                // macOS has one bar for the app: the focused window's (FACADE-APP-21).
                let state = &self.windows[index];
                let installed = self
                    .menus
                    .borrow()
                    .get(&state.token)
                    .and_then(|menu| menu.installed.clone());
                if let Some(menu) = installed {
                    cx.set_menu_bar(state.id, Some(menu));
                }
            }
            WindowEvent::MessageDialogClosed { id, result } => {
                if let Some((_, DialogDone::Message(done))) = self.dialogs.remove(&id) {
                    self.runtime.enter(|| done(result));
                }
            }
            WindowEvent::FilesHoverCancelled => {
                self.windows[index]
                    .tree
                    .dispatch_file_drop(tantu_view::FileDrop::Cancelled);
            }
            WindowEvent::FilesDropped {
                ref paths,
                position,
            } => {
                self.windows[index]
                    .tree
                    .dispatch_file_drop(tantu_view::FileDrop::Dropped {
                        paths: paths.clone(),
                        position,
                    });
            }
            WindowEvent::Accessibility(ref event) => {
                // FACADE-APP-15: a tree on activation, actions to the view, silence after.
                let state = &mut self.windows[index];
                match event {
                    AccessibilityEvent::Activated => {
                        let mut a11y = AccessibilityTree::new(self.app_name.clone());
                        cx.update_accessibility(state.id, a11y.update(&state.tree));
                        state.a11y = Some(a11y);
                    }
                    AccessibilityEvent::Deactivated => state.a11y = None,
                    AccessibilityEvent::Action(request) => {
                        if let Some((element, action)) = AccessibilityTree::map_action(request) {
                            state.tree.perform_action(element, action);
                        }
                    }
                }
            }
            ref other => {
                if let Some(mut pointer) = pointer_event(other) {
                    let state = &mut self.windows[index];
                    pointer.modifiers = state.modifiers;
                    state.pointer_inside = !matches!(other, WindowEvent::PointerLeft { .. });
                    state.tree.dispatch_pointer(pointer);
                    state.sync_cursor(cx);
                }
            }
        }
        self.apply_requests(cx);
        self.request_frames(cx);
    }

    fn app_event(&mut self, cx: &mut dyn PlatformContext, event: tantu_platform::AppEvent) {
        // FACADE-TRAY-01: the tray's actions, with the runtime current.
        let action = {
            let state = self.tray.0.borrow();
            match event {
                tantu_platform::AppEvent::TrayMenuItemActivated { id } => {
                    state.actions.get(&id).cloned()
                }
                tantu_platform::AppEvent::TrayClicked => state.on_click.clone(),
                _ => None,
            }
        };
        if let Some(action) = action {
            self.runtime.enter(|| action());
        }
        self.apply_requests(cx);
        self.request_frames(cx);
    }

    fn idle(&mut self, cx: &mut dyn PlatformContext) {
        // FACADE-WIN-06: a due runtime is ticked here, once for the app, and only the trees
        // that changed are redrawn; otherwise the first redraw after this idle ticks.
        let now = cx.now();
        let due = match self.runtime.next_wake() {
            Some(Wake::Now) => true,
            Some(Wake::At(deadline)) => deadline <= now,
            None => false,
        };
        self.ticked = false;
        if due {
            self.tick(cx);
        }
        self.apply_requests(cx);
        // Wake-ups (FACADE-APP-14): an active ticker keeps frames coming; the next deadline
        // goes to the platform, and a changed or passed deadline clears it.
        let earliest = match self.runtime.next_wake() {
            Some(Wake::Now) => {
                if !self.windows.iter().any(|w| w.tree.needs_frame()) {
                    if let Some(first) = self.windows.first() {
                        cx.request_redraw(first.id);
                    }
                }
                None
            }
            // A deadline that passed was handled by the tick above.
            Some(Wake::At(deadline)) => (deadline > now).then_some(deadline),
            None => None,
        };
        if earliest != self.wake {
            cx.wake_at(earliest);
            self.wake = earliest;
        }
        self.request_frames(cx);
    }
}
