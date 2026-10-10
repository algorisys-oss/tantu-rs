//! # Tantu
//!
//! A cross-platform desktop UI framework with Flutter-style authoring and pluggable renderers.
//! *Compose once. Render your way.*
//!
//! This is the facade crate apps depend on: [`App`] opens windows and runs one view tree per
//! window, connecting the platform shell, the text system and a renderer, and
//! [`prelude`] holds what app code imports. Spec: `docs/specs/facade/app.md`.
//!
//! ```no_run
//! use tantu::prelude::*;
//!
//! fn counter() -> impl View {
//!     let count = signal(0);
//!     Padding::all(16.0).child(
//!         Column::new()
//!             .spacing(8.0)
//!             .child(Text::new(move || format!("Count: {}", count.get())))
//!             .child(Button::new("Increment").on_press(move || count.update(|c| *c += 1))),
//!     )
//! }
//!
//! fn main() -> tantu::Result<()> {
//!     App::new().window(Window::new("Counter").size(400.0, 300.0), counter).run()
//! }
//! ```
//!
//! ## Features
//!
//! - `wgpu` (default): GPU renderer
//! - `winit` (default): winit platform shell
//! - `default-theme` (default): the default themes
//! - `soft`: CPU renderer built on tiny-skia

#![forbid(unsafe_code)]

mod app;

pub use app::{App, AppHandler, Error, Result, Window};

/// `tantu-core`: geometry, color, ids.
pub use tantu_core as core;
/// `tantu-layout`: the layout protocol and render objects.
pub use tantu_layout as layout;
/// `tantu-platform`: the platform trait and event types.
pub use tantu_platform as platform;
/// `tantu-reactive`: signals, memos and effects.
pub use tantu_reactive as reactive;
/// `tantu-scene`: the Scene display list and the renderer trait.
pub use tantu_scene as scene;
/// `tantu-text`: fonts, styles and shaping.
pub use tantu_text as text;
/// `tantu-view`: views, the view tree and event dispatch.
pub use tantu_view as view;
/// `tantu-widgets`: the standard widget set.
pub use tantu_widgets as widgets;

/// What app code imports: `use tantu::prelude::*;`
pub mod prelude {
    pub use crate::{App, Result, Window};
    pub use tantu_core::{Color, EdgeInsets, Point, Size};
    pub use tantu_layout::{
        Alignment, CrossAxisAlignment, MainAxisAlignment, MainAxisSize, StackFit,
    };
    pub use tantu_reactive::{Memo, Signal, batch, effect, memo, signal};
    pub use tantu_text::TextStyle;
    pub use tantu_view::{AnyView, Dyn, For, IntoProp, Show, View};
    pub use tantu_widgets::*;
}
