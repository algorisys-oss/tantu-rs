//! Single-child layouts: [`RenderPadding`], [`RenderPositionedBox`], [`RenderConstrainedBox`],
//! [`RenderFractionallySizedBox`] and [`RenderAspectRatio`]. Each lays out at most its first
//! child. Spec: `docs/specs/layout/single-child.md`.

use tantu_core::{EdgeInsets, Size};

use crate::{Alignment, BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};
/// Insets its child by `padding` (Flutter's `RenderPadding`, behind `Padding`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPadding {
    /// Space around the child.
    pub padding: EdgeInsets,
}

impl RenderPadding {
    /// Padding of `padding`.
    pub const fn new(padding: EdgeInsets) -> Self {
        let _ = padding;
        todo!()
    }
}

impl RenderBox for RenderPadding {
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
/// Positions its child within itself by `alignment`. On an axis with a factor, or with
/// unbounded constraints, it sizes itself to the child's extent times the factor (1 without
/// one); on the other axes it is as large as allowed (Flutter's `RenderPositionedBox`, behind
/// `Align` and `Center`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPositionedBox {
    /// Where the child goes.
    pub alignment: Alignment,
    /// Width as a multiple of the child's width; `None` fills the width when bounded.
    pub width_factor: Option<f32>,
    /// Height as a multiple of the child's height; `None` fills the height when bounded.
    pub height_factor: Option<f32>,
}

impl RenderPositionedBox {
    /// `alignment`, no factors.
    pub const fn new(alignment: Alignment) -> Self {
        let _ = alignment;
        todo!()
    }

    /// Centered, no factors (`Center`).
    pub const fn center() -> Self {
        todo!()
    }
}

impl Default for RenderPositionedBox {
    /// [`RenderPositionedBox::center`].
    fn default() -> Self {
        todo!()
    }
}

impl RenderBox for RenderPositionedBox {
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
/// Imposes `additional` constraints on its child, within the incoming ones (Flutter's
/// `RenderConstrainedBox`, behind `ConstrainedBox` and `SizedBox`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderConstrainedBox {
    /// The constraints added to the incoming ones (through `BoxConstraints::enforce`).
    pub additional: BoxConstraints,
}

impl RenderConstrainedBox {
    /// Adds `additional`.
    pub const fn new(additional: BoxConstraints) -> Self {
        let _ = additional;
        todo!()
    }

    /// `SizedBox(width, height)`: `BoxConstraints::tight_for(width, height)`.
    pub const fn sized(width: Option<f32>, height: Option<f32>) -> Self {
        let _ = (width, height);
        todo!()
    }

    /// `SizedBox.expand()`: `BoxConstraints::expand(None, None)`, filling bounded constraints.
    pub const fn expand() -> Self {
        todo!()
    }

    /// `SizedBox.shrink()`: tight at 0 × 0, as small as allowed.
    pub const fn shrink() -> Self {
        todo!()
    }
}

impl RenderBox for RenderConstrainedBox {
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
/// Sizes its child to a fraction of the incoming maximum on an axis with a factor, and
/// positions it by `alignment`; the child may overflow (Flutter's
/// `RenderFractionallySizedOverflowBox`, behind `FractionallySizedBox`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFractionallySizedBox {
    /// Where the child goes (it may be larger than this box).
    pub alignment: Alignment,
    /// The child's width as a fraction of the incoming maximum width; `None` passes the width
    /// constraints through.
    pub width_factor: Option<f32>,
    /// As `width_factor`, for heights.
    pub height_factor: Option<f32>,
}

impl RenderFractionallySizedBox {
    /// Centered, with the given factors.
    pub const fn new(width_factor: Option<f32>, height_factor: Option<f32>) -> Self {
        let _ = (width_factor, height_factor);
        todo!()
    }
}

impl RenderBox for RenderFractionallySizedBox {
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
/// Sizes itself (and its child, tightly) to `aspect_ratio` = width / height, as large as the
/// constraints allow (Flutter's `RenderAspectRatio`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderAspectRatio {
    /// Width divided by height; must be finite and positive to have an effect.
    pub aspect_ratio: f32,
}

impl RenderAspectRatio {
    /// A box with `aspect_ratio` = width / height.
    pub const fn new(aspect_ratio: f32) -> Self {
        let _ = aspect_ratio;
        todo!()
    }
}

impl RenderBox for RenderAspectRatio {
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
