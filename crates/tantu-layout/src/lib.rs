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
//!
//! Layouts run in a [`LayoutTree`]. A row with a fixed-width box and an `Expanded` box that
//! takes the rest:
//!
//! ```
//! use tantu_core::{Size, Vec2};
//! use tantu_layout::{BoxConstraints, FlexParentData, LayoutTree, RenderConstrainedBox, RenderFlex};
//!
//! let mut tree = LayoutTree::new();
//! let row = tree.insert(RenderFlex::row());
//! let fixed = tree.insert(RenderConstrainedBox::sized(Some(80.0), Some(20.0)));
//! let rest = tree.insert(RenderConstrainedBox::expand());
//! tree.set_parent_data(rest, Some(FlexParentData::expanded(1)));
//! tree.set_children(row, &[fixed, rest]).expect("fresh nodes");
//!
//! let size = tree.layout(row, BoxConstraints::loose(Size::new(300.0, 40.0)));
//! assert_eq!(size, Size::new(300.0, 40.0));
//! assert_eq!(tree.size(rest), Some(Size::new(220.0, 40.0)));
//! assert_eq!(tree.offset(rest), Some(Vec2::new(80.0, 0.0)));
//! ```

#![forbid(unsafe_code)]

mod alignment;
mod constraints;
mod flex;
mod single_child;
mod stack;
mod text;
mod tree;
mod wrap;

pub use alignment::Alignment;
pub use constraints::BoxConstraints;
pub use flex::{
    Axis, CrossAxisAlignment, FlexFit, FlexParentData, MainAxisAlignment, MainAxisSize, RenderFlex,
};
pub use single_child::{
    RenderAspectRatio, RenderConstrainedBox, RenderFractionallySizedBox, RenderPadding,
    RenderPositionedBox,
};
pub use stack::{RenderStack, StackFit, StackParentData};
pub use text::{
    MeasureCache, NoTextMeasure, RenderParagraph, TextMeasure, TextMetrics, TextStyleKey,
    TextWidthBasis,
};
pub use tree::{
    IntrinsicChildren, LayoutChildren, LayoutId, LayoutSession, LayoutTree, RenderBox, TreeError,
};
pub use wrap::{RenderWrap, WrapAlignment, WrapCrossAlignment};
