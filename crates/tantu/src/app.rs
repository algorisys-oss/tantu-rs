//! [`App`], [`Window`], [`AppHandler`] and [`Error`]: the app runner. Spec:
//! `docs/specs/facade/app.md`.

use std::fmt;

use tantu_platform::{
    Platform, PlatformContext, PlatformError, PlatformHandler, WindowAttributes, WindowEvent,
    WindowId,
};
use tantu_scene::{RenderError, Renderer};
use tantu_view::View;

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
}

impl Window {
    /// A window titled `title`, 800 × 600, resizable.
    pub fn new(title: impl Into<String>) -> Self {
        Window {
            attributes: WindowAttributes::new(title),
        }
    }

    /// The initial inner size in logical pixels.
    pub fn size(self, width: f32, height: f32) -> Self {
        Window {
            attributes: self.attributes.size(width, height),
        }
    }

    /// The smallest inner size in logical pixels.
    pub fn min_size(self, width: f32, height: f32) -> Self {
        Window {
            attributes: self.attributes.min_size(width, height),
        }
    }

    /// Whether the user can resize the window.
    pub fn resizable(self, resizable: bool) -> Self {
        Window {
            attributes: self.attributes.resizable(resizable),
        }
    }
}

/// An app: fonts and windows, then [`run`](App::run).
pub struct App {}

impl App {
    /// An app with no windows; the text system uses system fonts.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        todo!()
    }

    /// Opens `window` showing the view `content` builds. `content` runs once, when the window
    /// opens, with the window's reactive runtime current.
    pub fn window<V: View>(self, window: Window, content: impl FnOnce() -> V + 'static) -> Self {
        let _ = (window, content);
        todo!()
    }

    /// Registers a font file (e.g. bundled with the app).
    pub fn font(self, data: Vec<u8>) -> Self {
        let _ = data;
        todo!()
    }

    /// Uses only registered fonts, not the system's (tests, reproducible output).
    pub fn without_system_fonts(self) -> Self {
        todo!()
    }

    /// Runs on winit with the wgpu renderer until the last window closes.
    pub fn run(self) -> Result<()> {
        todo!()
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
        let _ = Box::new(renderer) as RendererFactory;
        todo!()
    }
}

/// The app runner (a [`PlatformHandler`]).
pub struct AppHandler {}

impl AppHandler {
    /// How the run ended: the first error, or `Ok(())`.
    pub fn finish(self) -> Result<()> {
        todo!()
    }
}

impl PlatformHandler for AppHandler {
    fn started(&mut self, cx: &mut dyn PlatformContext) {
        let _ = cx;
        todo!()
    }

    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent) {
        let _ = (cx, window, event);
        todo!()
    }

    fn idle(&mut self, cx: &mut dyn PlatformContext) {
        let _ = cx;
        todo!()
    }
}
