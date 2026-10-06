//! [`FakePlatform`]: a scripted platform for tests without a display.

use crate::platform::{Platform, PlatformContext, PlatformError, PlatformHandler, SurfaceTarget};
use crate::types::{PhysicalSize, WindowAttributes, WindowEvent, WindowId};

/// One step of a [`FakePlatform`] script.
#[derive(Clone, Debug, PartialEq)]
pub enum FakeStep {
    /// Deliver `event` to the `window`-th window created (0 = the first).
    Event {
        /// Index of the window in creation order.
        window: usize,
        /// The event to deliver.
        event: WindowEvent,
    },
    /// Deliver nothing; just let pending redraws and `idle` run.
    Idle,
}

/// What the handler asked a [`FakePlatform`] for.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FakeLog {
    /// Every window created, in creation order.
    pub windows: Vec<FakeWindow>,
    /// Whether the handler called `exit`.
    pub exited: bool,
    /// How many script steps were delivered (skipped steps not counted).
    pub steps_delivered: usize,
}

/// A window of a [`FakePlatform`], as last seen.
#[derive(Clone, Debug, PartialEq)]
pub struct FakeWindow {
    /// Its id.
    pub id: WindowId,
    /// The attributes, with the title as last set.
    pub attributes: WindowAttributes,
    /// Its inner size.
    pub size: PhysicalSize,
    /// Its scale factor.
    pub scale_factor: f32,
    /// Whether the handler closed it.
    pub closed: bool,
    /// `RedrawRequested` events delivered to it.
    pub redraws: u32,
}

/// A platform without a display: runs a handler against a script of events.
///
/// After `started` and after each delivered step, every window with a pending redraw gets one
/// `RedrawRequested` (requests made meanwhile wait for the next round), then `idle` is called
/// (not after `started`). One more redraw round runs after the last step.
#[derive(Clone, Debug)]
pub struct FakePlatform {
    scale_factor: f32,
    steps: Vec<FakeStep>,
}

impl FakePlatform {
    /// An empty script; new windows get scale factor 1.
    pub fn new() -> FakePlatform {
        FakePlatform {
            scale_factor: 1.0,
            steps: Vec::new(),
        }
    }

    /// The scale factor new windows get.
    pub fn scale_factor(self, scale_factor: f32) -> Self {
        FakePlatform {
            scale_factor,
            ..self
        }
    }

    /// Appends a step.
    pub fn step(mut self, step: FakeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Appends `FakeStep::Event { window, event }`.
    pub fn event(self, window: usize, event: WindowEvent) -> Self {
        self.step(FakeStep::Event { window, event })
    }

    /// Runs like [`Platform::run`] and returns the log.
    pub fn run_logged(self, handler: &mut dyn PlatformHandler) -> FakeLog {
        let mut cx = FakeContext {
            log: FakeLog::default(),
            scale_factor: self.scale_factor,
            pending: Vec::new(),
        };
        handler.started(&mut cx);
        if !cx.log.exited {
            cx.redraw_round(handler);
        }
        for step in self.steps {
            if cx.log.exited {
                break;
            }
            match step {
                FakeStep::Event { window, mut event } => {
                    let Some(w) = cx.log.windows.get_mut(window).filter(|w| !w.closed) else {
                        continue;
                    };
                    match &mut event {
                        WindowEvent::Resized(size) => w.size = *size,
                        WindowEvent::ScaleFactorChanged(scale) => w.scale_factor = *scale,
                        _ => {}
                    }
                    let id = w.id;
                    cx.log.steps_delivered += 1;
                    handler.window_event(&mut cx, id, event);
                }
                FakeStep::Idle => cx.log.steps_delivered += 1,
            }
            if cx.log.exited {
                break;
            }
            cx.redraw_round(handler);
            if cx.log.exited {
                break;
            }
            handler.idle(&mut cx);
        }
        if !cx.log.exited {
            cx.redraw_round(handler);
        }
        cx.log
    }
}

impl Default for FakePlatform {
    fn default() -> Self {
        FakePlatform::new()
    }
}

impl Platform for FakePlatform {
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError> {
        self.run_logged(handler);
        Ok(())
    }
}

/// The `PlatformContext` a `FakePlatform` hands its handler.
struct FakeContext {
    log: FakeLog,
    scale_factor: f32,
    /// Indices of windows with a redraw pending, in order of first request.
    pending: Vec<usize>,
}

impl FakeContext {
    /// The index of an open window.
    fn open_index(&self, window: WindowId) -> Option<usize> {
        self.log
            .windows
            .iter()
            .position(|w| w.id == window && !w.closed)
    }

    /// Delivers one `RedrawRequested` to each window with a redraw pending.
    fn redraw_round(&mut self, handler: &mut dyn PlatformHandler) {
        let due = std::mem::take(&mut self.pending);
        for index in due {
            if self.log.exited {
                return;
            }
            let Some(w) = self.log.windows.get_mut(index).filter(|w| !w.closed) else {
                continue;
            };
            w.redraws += 1;
            let id = w.id;
            handler.window_event(self, id, WindowEvent::RedrawRequested);
        }
    }

    fn mark_pending(&mut self, index: usize) {
        if !self.pending.contains(&index) {
            self.pending.push(index);
        }
    }
}

impl PlatformContext for FakeContext {
    fn create_window(&mut self, attributes: &WindowAttributes) -> Result<WindowId, PlatformError> {
        let index = self.log.windows.len();
        let id = WindowId::from_raw(index as u64 + 1);
        self.log.windows.push(FakeWindow {
            id,
            attributes: attributes.clone(),
            size: PhysicalSize::from_logical(attributes.size, self.scale_factor),
            scale_factor: self.scale_factor,
            closed: false,
            redraws: 0,
        });
        self.mark_pending(index);
        Ok(id)
    }

    fn close_window(&mut self, window: WindowId) {
        if let Some(index) = self.open_index(window) {
            self.log.windows[index].closed = true;
            self.pending.retain(|&i| i != index);
        }
    }

    fn request_redraw(&mut self, window: WindowId) {
        if let Some(index) = self.open_index(window) {
            self.mark_pending(index);
        }
    }

    fn inner_size(&self, window: WindowId) -> Option<PhysicalSize> {
        self.open_index(window).map(|i| self.log.windows[i].size)
    }

    fn scale_factor(&self, window: WindowId) -> Option<f32> {
        self.open_index(window)
            .map(|i| self.log.windows[i].scale_factor)
    }

    fn set_title(&mut self, window: WindowId, title: &str) {
        if let Some(index) = self.open_index(window) {
            self.log.windows[index].attributes.title = title.to_owned();
        }
    }

    fn surface_target(&self, _window: WindowId) -> Option<SurfaceTarget> {
        None
    }

    fn exit(&mut self) {
        self.log.exited = true;
    }
}
