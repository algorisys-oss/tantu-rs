//! [`WinitPlatform`]: the event loop, windows and the `PlatformContext` on top of winit.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use tantu_core::{Point, Size};
use tantu_platform::accesskit::TreeUpdate;
use tantu_platform::{
    AccessibilityEvent, PhysicalSize, Platform, PlatformContext, PlatformError, PlatformHandler,
    SurfaceTarget, WindowAttributes, WindowEvent, WindowId,
};
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::Window;

/// The event loop's user event: what the AccessKit adapters send (PLATFORM-WINIT-15).
struct AccessEvent(accesskit_winit::Event);

impl From<accesskit_winit::Event> for AccessEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        AccessEvent(event)
    }
}

use crate::conv;

/// The winit platform shell. Create it on the main thread.
pub struct WinitPlatform {
    event_loop: EventLoop<AccessEvent>,
}

impl WinitPlatform {
    /// Creates the event loop. Must be called on the main thread (a winit requirement on macOS,
    /// and on Linux). Fails, without panicking, when there is no display or an event loop was
    /// already created in this process.
    pub fn new() -> Result<WinitPlatform, PlatformError> {
        let event_loop = EventLoop::with_user_event()
            .build()
            .map_err(|e| PlatformError::Os(Box::new(e)))?;
        Ok(WinitPlatform { event_loop })
    }
}

impl Platform for WinitPlatform {
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError> {
        let proxy = self.event_loop.create_proxy();
        let mut shell = Shell {
            handler,
            windows: Windows::default(),
            started: false,
            proxy,
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
    /// For the AccessKit adapters, which send their events through the loop.
    proxy: EventLoopProxy<AccessEvent>,
}

impl ApplicationHandler<AccessEvent> for Shell<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Desktop platforms resume once; later resumes (mobile) don't start the app again.
        if self.started {
            return;
        }
        self.started = true;
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
            proxy: &self.proxy,
        };
        self.handler.started(&mut cx);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AccessEvent) {
        // The adapters' events become accessibility window events (PLATFORM-WINIT-15).
        let Some(&id) = self.windows.by_winit.get(&event.0.window_id) else {
            return;
        };
        let event = match event.0.window_event {
            accesskit_winit::WindowEvent::InitialTreeRequested => AccessibilityEvent::Activated,
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                AccessibilityEvent::Action(request)
            }
            accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                AccessibilityEvent::Deactivated
            }
        };
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
            proxy: &self.proxy,
        };
        self.handler
            .window_event(&mut cx, id, WindowEvent::Accessibility(event));
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
        // The adapter sees every event first (PLATFORM-WINIT-15).
        entry.adapter.process_event(&entry.window, &event);
        let scale = entry.window.scale_factor() as f32;
        let Some(event) = conv::window_event(event, scale, &mut entry.pointer) else {
            return;
        };
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
            proxy: &self.proxy,
        };
        self.handler.window_event(&mut cx, id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
            proxy: &self.proxy,
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
    /// The window's AccessKit adapter (PLATFORM-WINIT-15).
    adapter: accesskit_winit::Adapter,
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
    proxy: &'a EventLoopProxy<AccessEvent>,
}

impl PlatformContext for Context<'_> {
    fn create_window(&mut self, attributes: &WindowAttributes) -> Result<WindowId, PlatformError> {
        // The adapter must exist before the window is shown (PLATFORM-WINIT-15).
        let window = self
            .event_loop
            .create_window(conv::attributes(attributes).with_visible(false))
            .map_err(|e| PlatformError::Os(Box::new(e)))?;
        let adapter = accesskit_winit::Adapter::with_event_loop_proxy(
            self.event_loop,
            &window,
            self.proxy.clone(),
        );
        window.set_visible(attributes.visible);
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
                adapter,
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

    fn set_ime_allowed(&mut self, window: WindowId, allowed: bool) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.set_ime_allowed(allowed);
        }
    }

    fn set_ime_cursor_area(&mut self, window: WindowId, origin: Point, size: Size) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.set_ime_cursor_area(
                winit::dpi::LogicalPosition::new(origin.x, origin.y),
                winit::dpi::LogicalSize::new(size.width, size.height),
            );
        }
    }

    fn set_cursor(&mut self, window: WindowId, cursor: tantu_platform::CursorIcon) {
        let _ = (window, cursor);
        todo!("PLATFORM-WINIT-16")
    }

    fn exit(&mut self) {
        self.event_loop.exit();
    }

    fn update_accessibility(&mut self, window: WindowId, update: TreeUpdate) {
        if let Some(entry) = self.windows.entries.get_mut(&window) {
            entry.adapter.update_if_active(|| update);
        }
    }

    fn now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }

    fn wake_at(&mut self, deadline: Option<std::time::Instant>) {
        // PLATFORM-WINIT-14: `about_to_wait` (the handler's `idle`) runs when the deadline
        // passes. A passed deadline would fire again at once, so the runner clears it.
        self.event_loop.set_control_flow(match deadline {
            Some(deadline) => winit::event_loop::ControlFlow::WaitUntil(deadline),
            None => winit::event_loop::ControlFlow::Wait,
        });
    }
}
