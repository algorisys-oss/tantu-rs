//! # Tantu: Views
//!
//! The view layer (ADR 0011): views are one-shot builders, components are functions that run
//! once, and a [`ViewTree`] per window holds the retained elements, their reactive scopes and
//! the window's layout tree. Specs: `docs/specs/view/`.
//!
//! ```
//! use tantu_core::Size;
//! use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
//! use tantu_view::{AnyView, BuildCx, ElementId, View, ViewTree};
//!
//! /// A fixed-size box.
//! struct Boxed(f32, f32);
//!
//! impl View for Boxed {
//!     fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
//!         cx.render(RenderConstrainedBox::sized(Some(self.0), Some(self.1)), [])
//!     }
//! }
//!
//! /// A column of children.
//! struct Column(Vec<AnyView>);
//!
//! impl View for Column {
//!     fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
//!         cx.render(RenderFlex::column(), self.0)
//!     }
//! }
//!
//! let mut tree = ViewTree::new(|| {
//!     Column(vec![AnyView::new(Boxed(40.0, 10.0)), AnyView::new(Boxed(20.0, 30.0))])
//! });
//! let size = tree.layout(BoxConstraints::tight(Size::new(200.0, 100.0)), &mut NoTextMeasure);
//! assert_eq!(size, Size::new(200.0, 100.0));
//! assert_eq!(tree.len(), 4); // the root, the column and two boxes
//! ```

#![forbid(unsafe_code)]

mod dynamic;
mod events;
mod frame;
mod layout_builder;
mod paint;
mod prop;
mod text;
mod tree;

pub use dynamic::{Dyn, For, Show};
pub use events::{CursorIcon, Handled, Phase, PointerButton, PointerCx, PointerEvent, PointerKind};
pub use frame::{ElementMut, FrameReport};
pub use layout_builder::LayoutBuilder;
pub use paint::{NoPaint, Paint, PaintCx};
pub use prop::{IntoProp, Prop};
pub use tantu_scene::ElementId;
pub use text::{ParagraphPaint, SystemText, TextContext, TextPainter};
pub use tree::{AnyView, BuildCx, ElementKind, View, ViewTree};
