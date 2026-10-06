//! [`WinitPlatform`]: the event loop, windows and the `PlatformContext` on top of winit.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use tantu_core::Point;
use tantu_platform::{
    PhysicalSize, Platform, PlatformContext, PlatformError, PlatformHandler, SurfaceTarget,
    WindowAttributes, WindowId,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

/// The winit platform shell. Create it on the main thread.
pub struct WinitPlatform {
    event_loop: EventLoop<()>,
}

impl WinitPlatform {
    /// Creates the event loop. Must be called on the main thread (a winit requirement on macOS,
    /// and on Linux). Fails, without panicking, when there is no display or an event loop was
    /// already created in this process.
    pub fn new() -> Result<WinitPlatform, PlatformError> {
        todo!()
    }
}

impl Platform for WinitPlatform {
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError> {
        todo!()
    }
}

impl fmt::Debug for WinitPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WinitPlatform").finish_non_exhaustive()
    }
}

/// A Tantu-created window.
struct Entry {
    window: Arc<Window>,
    /// Last cursor position, in logical pixels.
    pointer: Point,
}

/// The windows Tantu created, by Tantu id and by winit id.
#[derive(Default)]
struct Windows {
    entries: HashMap<WindowId, Entry>,
    by_winit: HashMap<winit::window::WindowId, WindowId>,
    next: u64,
}

/// `PlatformContext` for one callback.
struct Context<'a> {
    event_loop: &'a ActiveEventLoop,
    windows: &'a mut Windows,
}

impl PlatformContext for Context<'_> {
    fn create_window(&mut self, attributes: &WindowAttributes) -> Result<WindowId, PlatformError> {
        todo!()
    }

    fn close_window(&mut self, window: WindowId) {
        todo!()
    }

    fn request_redraw(&mut self, window: WindowId) {
        todo!()
    }

    fn inner_size(&self, window: WindowId) -> Option<PhysicalSize> {
        todo!()
    }

    fn scale_factor(&self, window: WindowId) -> Option<f32> {
        todo!()
    }

    fn set_title(&mut self, window: WindowId, title: &str) {
        todo!()
    }

    fn surface_target(&self, window: WindowId) -> Option<SurfaceTarget> {
        todo!()
    }

    fn exit(&mut self) {
        todo!()
    }
}
