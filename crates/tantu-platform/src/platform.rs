//! The [`Platform`], [`PlatformHandler`] and [`PlatformContext`] traits.

use std::fmt;
use std::sync::Arc;

use crate::types::{PhysicalSize, WindowAttributes, WindowEvent, WindowId};

/// Why a platform operation failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum PlatformError {
    /// The platform can't do this (e.g. no display, unsupported feature).
    Unsupported(String),
    /// The OS or windowing library reported an error.
    Os(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::Unsupported(what) => write!(f, "not supported by this platform: {what}"),
            PlatformError::Os(e) => write!(f, "platform error: {e}"),
        }
    }
}

impl std::error::Error for PlatformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PlatformError::Os(e) => Some(e.as_ref()),
            PlatformError::Unsupported(_) => None,
        }
    }
}

/// Window and display handles a GPU renderer can create a surface from.
pub trait WindowHandles:
    raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync
{
}

impl<T> WindowHandles for T where
    T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync
{
}

/// A shared handle to a window's handles. Keeps the native window alive while held.
pub type SurfaceTarget = Arc<dyn WindowHandles>;

/// What a handler can ask the platform to do while it is called.
pub trait PlatformContext {
    /// Creates a window. A `RedrawRequested` for it follows; read its initial size and scale
    /// factor with [`inner_size`](Self::inner_size) and [`scale_factor`](Self::scale_factor)
    /// (`Resized` and `ScaleFactorChanged` only report changes).
    fn create_window(&mut self, attributes: &WindowAttributes) -> Result<WindowId, PlatformError>;

    /// Closes a window. Unknown or already closed ids are ignored.
    fn close_window(&mut self, window: WindowId);

    /// Asks for a `RedrawRequested` event for `window`, delivered later. Several requests before
    /// it arrives produce one event.
    fn request_redraw(&mut self, window: WindowId);

    /// Current inner size, or `None` for an unknown window.
    fn inner_size(&self, window: WindowId) -> Option<PhysicalSize>;

    /// Current scale factor, or `None` for an unknown window.
    fn scale_factor(&self, window: WindowId) -> Option<f32>;

    /// Sets the window title. Unknown windows are ignored.
    fn set_title(&mut self, window: WindowId, title: &str);

    /// Handles a renderer can draw into, or `None` (unknown window, or a platform without native
    /// windows such as [`FakePlatform`](crate::FakePlatform)).
    fn surface_target(&self, window: WindowId) -> Option<SurfaceTarget>;

    /// Ends the event loop after the current callback returns. [`Platform::run`] then returns.
    fn exit(&mut self);
}

/// The app side of the event loop.
pub trait PlatformHandler {
    /// Called once, first, when windows can be created.
    fn started(&mut self, cx: &mut dyn PlatformContext);

    /// Called for each event of each window.
    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent);

    /// Called when pending events have been handled and the platform is about to wait.
    fn idle(&mut self, cx: &mut dyn PlatformContext) {
        let _ = cx;
    }
}

/// A platform shell: owns the event loop and calls the handler.
pub trait Platform {
    /// Runs the event loop until [`PlatformContext::exit`], calling `handler`. Blocks.
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError>
    where
        Self: Sized;
}
