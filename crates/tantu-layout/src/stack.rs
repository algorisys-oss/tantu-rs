//! [`RenderStack`]: Flutter's stack layout, with [`StackParentData`] for `Positioned`
//! children. Spec: `docs/specs/layout/stack.md`.

use tantu_core::Size;

use crate::{Alignment, BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// How non-positioned children are constrained (Flutter's `StackFit`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StackFit {
    /// The incoming constraints, loosened (children may be smaller than the stack).
    Loose,
    /// Tight at the largest allowed size (children fill the stack).
    Expand,
    /// The incoming constraints unchanged.
    Passthrough,
}

/// Parent data read by [`RenderStack`]: distances from the stack's edges, and an optional
/// size. A child is *positioned* when it has this parent data with at least one field set (a
/// NaN field counts as not set).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StackParentData {
    /// Distance from the stack's left edge to the child's left edge.
    pub left: Option<f32>,
    /// Distance from the stack's top edge to the child's top edge.
    pub top: Option<f32>,
    /// Distance from the stack's right edge to the child's right edge.
    pub right: Option<f32>,
    /// Distance from the stack's bottom edge to the child's bottom edge.
    pub bottom: Option<f32>,
    /// The child's width (tight); with both `left` and `right` set, `left` wins for placement.
    pub width: Option<f32>,
    /// The child's height (tight); with both `top` and `bottom` set, `top` wins for placement.
    pub height: Option<f32>,
}

impl StackParentData {
    /// True if any field is set (and not NaN).
    pub fn is_positioned(&self) -> bool {
        todo!()
    }

    /// All four edges at 0: the child covers the stack (`Positioned.fill`).
    pub const fn fill() -> Self {
        todo!()
    }
}

/// Layers its children; the non-positioned ones size the stack (Flutter's `RenderStack`,
/// behind `Stack`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderStack {
    /// Where non-positioned children go, and positioned ones on an axis with no edge set.
    pub alignment: Alignment,
    /// How non-positioned children are constrained.
    pub fit: StackFit,
}

impl RenderStack {
    /// `alignment: TOP_LEFT`, `fit: Loose` (Flutter's defaults with left-to-right text).
    pub const fn new() -> Self {
        todo!()
    }
}

impl Default for RenderStack {
    /// [`RenderStack::new`].
    fn default() -> Self {
        todo!()
    }
}

impl RenderBox for RenderStack {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!()
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!()
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!()
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!()
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!()
    }
}
