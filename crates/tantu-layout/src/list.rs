//! [`RenderListContent`] and [`ListIndex`]: a virtualized list's content. Spec:
//! `docs/specs/layout/list.md`.

use tantu_core::{Size, Vec2};

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

impl RenderListContent {
    /// The item extent, with negative or NaN counting as 0.
    fn extent(&self) -> f32 {
        if self.item_extent.is_finite() && self.item_extent >= 0.0 {
            self.item_extent
        } else {
            0.0
        }
    }

    /// `count · extent`.
    fn main_size(&self) -> f32 {
        self.count as f32 * self.extent()
    }
}

impl RenderBox for RenderListContent {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let extent = self.extent();
        let (bounded, cross_max) = match self.axis {
            ScrollAxis::Vertical => (constraints.has_bounded_width(), constraints.max_width),
            ScrollAxis::Horizontal => (constraints.has_bounded_height(), constraints.max_height),
        };
        let (cross_min, cross_limit) = if bounded {
            (cross_max, cross_max)
        } else {
            (0.0, f32::INFINITY)
        };
        let mut cross = if bounded { cross_max } else { 0.0 };
        for i in 0..children.len() {
            let child_constraints = match self.axis {
                ScrollAxis::Vertical => BoxConstraints::new(cross_min, cross_limit, extent, extent),
                ScrollAxis::Horizontal => {
                    BoxConstraints::new(extent, extent, cross_min, cross_limit)
                }
            };
            let size = children.layout(i, child_constraints);
            if !bounded {
                cross = cross.max(match self.axis {
                    ScrollAxis::Vertical => size.width,
                    ScrollAxis::Horizontal => size.height,
                });
            }
            let index = children
                .parent_data::<ListIndex>(i)
                .map_or(i, |data| data.0);
            let along = index as f32 * extent;
            children.set_offset(
                i,
                match self.axis {
                    ScrollAxis::Vertical => Vec2::new(0.0, along),
                    ScrollAxis::Horizontal => Vec2::new(along, 0.0),
                },
            );
        }
        let size = match self.axis {
            ScrollAxis::Vertical => Size::new(cross, self.main_size()),
            ScrollAxis::Horizontal => Size::new(self.main_size(), cross),
        };
        constraints.constrain(size)
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.axis {
            ScrollAxis::Horizontal => self.main_size(),
            ScrollAxis::Vertical => (0..children.len())
                .map(|i| children.min_intrinsic_width(i, height))
                .fold(0.0, f32::max),
        }
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.axis {
            ScrollAxis::Horizontal => self.main_size(),
            ScrollAxis::Vertical => (0..children.len())
                .map(|i| children.max_intrinsic_width(i, height))
                .fold(0.0, f32::max),
        }
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.axis {
            ScrollAxis::Vertical => self.main_size(),
            ScrollAxis::Horizontal => (0..children.len())
                .map(|i| children.min_intrinsic_height(i, width))
                .fold(0.0, f32::max),
        }
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.axis {
            ScrollAxis::Vertical => self.main_size(),
            ScrollAxis::Horizontal => (0..children.len())
                .map(|i| children.max_intrinsic_height(i, width))
                .fold(0.0, f32::max),
        }
    }
}
