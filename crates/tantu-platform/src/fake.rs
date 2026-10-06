//! [`FakePlatform`]: a scripted platform for tests without a display.

use crate::platform::{Platform, PlatformError, PlatformHandler};
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
#[derive(Clone, Debug)]
pub struct FakePlatform {
    scale_factor: f32,
    steps: Vec<FakeStep>,
}

impl FakePlatform {
    /// An empty script; new windows get scale factor 1.
    pub fn new() -> FakePlatform {
        todo!()
    }

    /// The scale factor new windows get.
    pub fn scale_factor(self, scale_factor: f32) -> Self {
        todo!()
    }

    /// Appends a step.
    pub fn step(self, step: FakeStep) -> Self {
        todo!()
    }

    /// Appends `FakeStep::Event { window, event }`.
    pub fn event(self, window: usize, event: WindowEvent) -> Self {
        todo!()
    }

    /// Runs like [`Platform::run`] and returns the log.
    pub fn run_logged(self, handler: &mut dyn PlatformHandler) -> FakeLog {
        todo!()
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
