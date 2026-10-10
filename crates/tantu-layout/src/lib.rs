//! # Tantu: Layout
//!
//! Flutter's layout protocol: a parent passes [`BoxConstraints`] down, the child picks a size
//! within them and returns it, and the parent positions the child. The specs are in
//! `docs/specs/layout/`; ADR 0009 explains why the layout tree lives in this crate.
//!
//! ```
//! use tantu_core::{EdgeInsets, Size};
//! use tantu_layout::BoxConstraints;
//!
//! // A parent offering up to 200 × 100, and a child padded by 10 on each side.
//! let parent = BoxConstraints::loose(Size::new(200.0, 100.0));
//! let child = parent.deflate(EdgeInsets::all(10.0));
//! assert_eq!(child, BoxConstraints::new(0.0, 180.0, 0.0, 80.0));
//! // The child wants 300 × 50; it gets the closest allowed size.
//! assert_eq!(child.constrain(Size::new(300.0, 50.0)), Size::new(180.0, 50.0));
//! ```

#![forbid(unsafe_code)]

mod alignment;
mod constraints;
mod single_child;
mod tree;

pub use alignment::Alignment;
pub use constraints::BoxConstraints;
pub use single_child::{
    RenderAspectRatio, RenderConstrainedBox, RenderFractionallySizedBox, RenderPadding,
    RenderPositionedBox,
};
pub use tree::{IntrinsicChildren, LayoutChildren, LayoutId, LayoutTree, RenderBox, TreeError};
