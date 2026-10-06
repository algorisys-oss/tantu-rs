//! # Tantu: Platform
//!
//! The boundary between Tantu and the operating system. A platform shell (winit by default, in
//! `tantu-platform-winit`) implements [`Platform`]: it owns the event loop, creates windows and
//! turns OS input into Tantu's own [`WindowEvent`]s, which it hands to a [`PlatformHandler`]
//! (the app runner). Nothing above this crate sees a windowing library's types.
//! [`FakePlatform`] runs a handler against a script of events, for tests without a display.
//! The spec is `docs/specs/platform/platform.md`.
//!
//! ```
//! use tantu_platform::{
//!     FakePlatform, PlatformContext, PlatformHandler, WindowAttributes, WindowEvent, WindowId,
//! };
//!
//! /// Opens one window and closes it when asked.
//! struct App;
//!
//! impl PlatformHandler for App {
//!     fn started(&mut self, cx: &mut dyn PlatformContext) {
//!         cx.create_window(&WindowAttributes::new("Hello").size(400.0, 300.0))
//!             .expect("the fake platform always creates windows");
//!     }
//!
//!     fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent) {
//!         if event == WindowEvent::CloseRequested {
//!             cx.close_window(window);
//!             cx.exit();
//!         }
//!     }
//! }
//!
//! let log = FakePlatform::new()
//!     .scale_factor(2.0)
//!     .event(0, WindowEvent::CloseRequested)
//!     .run_logged(&mut App);
//! assert!(log.exited);
//! assert_eq!(log.windows[0].size.width, 800); // 400 logical pixels at scale 2
//! assert_eq!(log.windows[0].redraws, 1); // a new window gets one redraw
//! assert!(log.windows[0].closed);
//! ```

#![forbid(unsafe_code)]

mod fake;
mod platform;
mod types;

pub use fake::{FakeLog, FakePlatform, FakeStep, FakeWindow};
pub use platform::{
    Platform, PlatformContext, PlatformError, PlatformHandler, SurfaceTarget, WindowHandles,
};
/// The `raw-window-handle` version surface targets implement.
pub use raw_window_handle;
pub use types::{
    ButtonState, Key, KeyEvent, KeyLocation, Modifiers, NamedKey, PhysicalSize, PointerButton,
    PointerId, ScrollDelta, WindowAttributes, WindowEvent, WindowId,
};
