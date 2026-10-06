//! # Tantu: winit platform shell
//!
//! [`WinitPlatform`] implements [`tantu_platform::Platform`] on
//! [winit](https://github.com/rust-windowing/winit) 0.30: windows and input on Windows, macOS
//! and Linux (Wayland and X11). It converts winit's events into Tantu's
//! [`WindowEvent`](tantu_platform::WindowEvent)s and hands renderers the native window handles.
//! This is the only crate that sees winit types. The spec is
//! `docs/specs/platform-winit/shell.md`.
//!
//! ```no_run
//! use tantu_platform::{Platform, PlatformContext, PlatformHandler, WindowAttributes, WindowEvent, WindowId};
//! use tantu_platform_winit::WinitPlatform;
//!
//! struct App;
//!
//! impl PlatformHandler for App {
//!     fn started(&mut self, cx: &mut dyn PlatformContext) {
//!         cx.create_window(&WindowAttributes::new("Hello")).expect("a window");
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
//! // On the main thread:
//! WinitPlatform::new()?.run(&mut App)?;
//! # Ok::<(), tantu_platform::PlatformError>(())
//! ```

#![forbid(unsafe_code)]

mod conv;
mod shell;

pub use shell::WinitPlatform;
