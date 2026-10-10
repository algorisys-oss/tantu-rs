//! # Tantu: Widgets
//!
//! The standard Tantu widget set. Layout widgets follow Flutter's names, defaults and
//! semantics in `snake_case` (AGENTS.md); every widget is a [`View`](tantu_view::View) built
//! once, with reactive props that update their element at the next frame. Specs:
//! `docs/specs/widgets/`.
//!
//! ```
//! use tantu_view::ViewTree;
//! use tantu_view::layout::{BoxConstraints, NoTextMeasure};
//! use tantu_view::core::Size;
//! use tantu_widgets::{Column, Padding, SizedBox};
//!
//! let mut tree = ViewTree::new(|| {
//!     Padding::all(16.0).child(
//!         Column::new()
//!             .spacing(8.0)
//!             .child(SizedBox::new(50.0, 50.0))
//!             .child(SizedBox::new(50.0, 50.0)),
//!     )
//! });
//! let size = tree.layout(BoxConstraints::loose(Size::new(200.0, 200.0)), &mut NoTextMeasure);
//! assert_eq!(size, Size::new(200.0, 200.0));
//! ```

#![forbid(unsafe_code)]

mod layout;

pub use layout::{
    Align, Center, Column, ConstrainedBox, Expanded, Flexible, Padding, Positioned, Row, SizedBox,
    Spacer, Stack,
};
