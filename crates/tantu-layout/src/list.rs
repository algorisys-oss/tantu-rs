//! [`RenderListContent`] and [`ListIndex`]: a virtualized list's content. Spec:
//! `docs/specs/layout/list.md`.

use tantu_core::Size;

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox, ScrollAxis};

/// Which item a built child of a [`RenderListContent`] is (parent data).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListIndex(pub usize);

/// Sizes to `count` items of `item_extent` along `axis` and places its children by their
/// [`ListIndex`] (Flutter's `RenderSliverFixedExtentList`, as a box).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderListContent {
    /// The list's axis.
    pub axis: ScrollAxis,
    /// How many items the list has (built or not).
    pub count: usize,
    /// Each item's extent along the axis; negative or NaN counts as 0.
    pub item_extent: f32,
}

impl RenderListContent {
    /// `count` items of `item_extent` along `axis`.
    pub const fn new(axis: ScrollAxis, count: usize, item_extent: f32) -> Self {
        RenderListContent {
            axis,
            count,
            item_extent,
        }
    }
}

impl RenderBox for RenderListContent {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!("LAYOUT-LIST-01")
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!("LAYOUT-LIST-02")
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!("LAYOUT-LIST-02")
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!("LAYOUT-LIST-02")
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!("LAYOUT-LIST-02")
    }
}
