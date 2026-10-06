//! [`WinitPlatform`]: the event loop, windows and the `PlatformContext` on top of winit.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use tantu_core::Point;
use tantu_platform::{
    PhysicalSize, Platform, PlatformContext, PlatformError, PlatformHandler, SurfaceTarget,
    WindowAttributes, WindowId,
};
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

use crate::conv;

/// The winit platform shell. Create it on the main thread.
pub struct WinitPlatform {
    event_loop: EventLoop<()>,
}

impl WinitPlatform {
    /// Creates the event loop. Must be called on the main thread (a winit requirement on macOS,
    /// and on Linux). Fails, without panicking, when there is no display or an event loop was
    /// already created in this process.
    pub fn new() -> Result<WinitPlatform, PlatformError> {
        let event_loop = EventLoop::new().map_err(|e| PlatformError::Os(Box::new(e)))?;
        Ok(WinitPlatform { event_loop })
    }
}

impl Platform for WinitPlatform {
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError> {
        let mut shell = Shell {
            handler,
            windows: Windows::default(),
            started: false,
        };
        self.event_loop
            .run_app(&mut shell)
            .map_err(|e| PlatformError::Os(Box::new(e)))
    }
}

/// winit's application handler: forwards to the Tantu handler.
struct Shell<'h> {
    handler: &'h mut dyn PlatformHandler,
    windows: Windows,
    started: bool,
}

impl ApplicationHandler for Shell<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Desktop platforms resume once; later resumes (mobile) don't start the app again.
        if self.started {
            return;
        }
        self.started = true;
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
        };
        self.handler.started(&mut cx);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(&id) = self.windows.by_winit.get(&window_id) else {
            return;
        };
        let Some(entry) = self.windows.entries.get_mut(&id) else {
            return;
        };
        let scale = entry.window.scale_factor() as f32;
        let Some(event) = conv::window_event(event, scale, &mut entry.pointer) else {
            return;
        };
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
        };
        self.handler.window_event(&mut cx, id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
        };
        self.handler.idle(&mut cx);
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
        let window = self
            .event_loop
            .create_window(conv::attributes(attributes))
            .map_err(|e| PlatformError::Os(Box::new(e)))?;
        self.windows.next += 1;
        let id = WindowId::from_raw(self.windows.next);
        self.windows.by_winit.insert(window.id(), id);
        // Every new window gets a first redraw (spec: platform.md, `create_window`).
        window.request_redraw();
        self.windows.entries.insert(
            id,
            Entry {
                window: Arc::new(window),
                pointer: Point::ZERO,
            },
        );
        Ok(id)
    }

    fn close_window(&mut self, window: WindowId) {
        if let Some(entry) = self.windows.entries.remove(&window) {
            self.windows.by_winit.remove(&entry.window.id());
            // The native window goes away when the last `SurfaceTarget` clone is dropped.
            entry.window.set_visible(false);
        }
    }

    fn request_redraw(&mut self, window: WindowId) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.request_redraw();
        }
    }

    fn inner_size(&self, window: WindowId) -> Option<PhysicalSize> {
        let entry = self.windows.entries.get(&window)?;
        Some(conv::size(entry.window.inner_size()))
    }

    fn scale_factor(&self, window: WindowId) -> Option<f32> {
        let entry = self.windows.entries.get(&window)?;
        Some(entry.window.scale_factor() as f32)
    }

    fn set_title(&mut self, window: WindowId, title: &str) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.set_title(title);
        }
    }

    fn surface_target(&self, window: WindowId) -> Option<SurfaceTarget> {
        let entry = self.windows.entries.get(&window)?;
        Some(entry.window.clone())
    }

    fn exit(&mut self) {
        self.event_loop.exit();
    }
}
