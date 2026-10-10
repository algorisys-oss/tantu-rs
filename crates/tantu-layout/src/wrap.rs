//! [`RenderWrap`]: Flutter's wrap layout, which starts a new run when the next child doesn't
//! fit. Spec: `docs/specs/layout/wrap.md`.

use tantu_core::Size;

use crate::{Axis, BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// How children are distributed within a run, or runs within the wrap (Flutter's
/// `WrapAlignment`; the same meanings as `MainAxisAlignment`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WrapAlignment {
    /// Packed at the start.
    Start,
    /// Packed at the end.
    End,
    /// Packed in the middle.
    Center,
    /// Free space evenly between, none at the ends.
    SpaceBetween,
    /// Free space evenly between, half as much at the ends.
    SpaceAround,
    /// Free space evenly between and at the ends.
    SpaceEvenly,
}

/// How children are placed on the cross axis within their run (Flutter's
/// `WrapCrossAlignment`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WrapCrossAlignment {
    /// At the run's cross-axis start.
    Start,
    /// At the run's cross-axis end.
    End,
    /// In the middle of the run.
    Center,
}

/// Lays out children in runs along `direction`, starting a new run when the next child
/// doesn't fit (Flutter's `RenderWrap`, behind `Wrap`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderWrap {
    /// The main axis of each run.
    pub direction: Axis,
    /// Distribution of children within each run.
    pub alignment: WrapAlignment,
    /// Space between children in a run (negative or NaN counts as 0).
    pub spacing: f32,
    /// Distribution of the runs on the cross axis.
    pub run_alignment: WrapAlignment,
    /// Space between runs (negative or NaN counts as 0).
    pub run_spacing: f32,
    /// Placement of children on the cross axis within their run.
    pub cross_axis_alignment: WrapCrossAlignment,
}

impl RenderWrap {
    /// Flutter's defaults: `Start` everywhere, spacing 0.
    pub const fn new(direction: Axis) -> Self {
        let _ = direction;
        todo!()
    }

    /// `new(Axis::Horizontal)`.
    pub const fn horizontal() -> Self {
        todo!()
    }

    /// `new(Axis::Vertical)`.
    pub const fn vertical() -> Self {
        todo!()
    }
}

impl RenderBox for RenderWrap {
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
