//! [`RenderListContent`] and [`ListIndex`]: a virtualized list's content. Spec:
//! `docs/specs/layout/list.md`.

use std::sync::Arc;

use tantu_core::{Size, Vec2};

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox, ScrollAxis};

/// How long each item of a [`RenderListContent`] is along the axis (LAYOUT-LIST-03).
#[derive(Clone, Debug, PartialEq)]
pub enum ListExtents {
    /// Every item the same; negative or NaN counts as 0.
    Fixed(f32),
    /// Item `i` ends at `ends[i]` (prefix sums, non-decreasing; `ends.len()` is the count).
    Table(Arc<[f32]>),
}

impl ListExtents {
    /// Prefix sums of `extents` (negative or NaN entries count as 0).
    pub fn table(extents: impl IntoIterator<Item = f32>) -> ListExtents {
        let mut end = 0.0f32;
        let ends: Vec<f32> = extents
            .into_iter()
            .map(|extent| {
                end += clean(extent);
                end
            })
            .collect();
        ListExtents::Table(Arc::from(ends))
    }

    /// Where item `index` starts.
    pub fn start(&self, index: usize) -> f32 {
        match self {
            ListExtents::Fixed(extent) => index as f32 * clean(*extent),
            ListExtents::Table(ends) => match index {
                0 => 0.0,
                _ => ends
                    .get(index - 1)
                    .copied()
                    .unwrap_or_else(|| ends.last().copied().unwrap_or(0.0)),
            },
        }
    }

    /// Item `index`'s extent.
    pub fn extent(&self, index: usize) -> f32 {
        match self {
            ListExtents::Fixed(extent) => clean(*extent),
            ListExtents::Table(ends) => match ends.get(index) {
                Some(end) => end - self.start(index),
                None => 0.0,
            },
        }
    }

    /// The total length of `count` items.
    pub fn total(&self, count: usize) -> f32 {
        match self {
            ListExtents::Fixed(extent) => count as f32 * clean(*extent),
            ListExtents::Table(ends) => match count.min(ends.len()) {
                0 => 0.0,
                n => ends[n - 1],
            },
        }
    }

    /// The index of the item containing `position` (the last item at or past the end, 0
    /// before the start), among `count` items; `None` when `count` is 0.
    pub fn index_at(&self, position: f32, count: usize) -> Option<usize> {
        if count == 0 {
            return None;
        }
        let last = count - 1;
        let index = match self {
            ListExtents::Fixed(extent) => {
                let extent = clean(*extent);
                if extent > 0.0 && position.is_finite() && position > 0.0 {
                    (position / extent).floor().min(last as f32) as usize
                } else if position.is_finite() && position <= 0.0 {
                    0
                } else {
                    last
                }
            }
            ListExtents::Table(ends) => {
                let n = count.min(ends.len());
                ends[..n].partition_point(|end| *end <= position)
            }
        };
        Some(index.min(last))
    }
}

/// A non-negative finite extent, else 0.
fn clean(extent: f32) -> f32 {
    if extent.is_finite() && extent >= 0.0 {
        extent
    } else {
        0.0
    }
}

/// Which item a built child of a [`RenderListContent`] is (parent data).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListIndex(pub usize);

/// Sizes to `count` items of `item_extent` along `axis` and places its children by their
/// [`ListIndex`] (Flutter's `RenderSliverFixedExtentList`, as a box).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderListContent {
    /// The list's axis.
    pub axis: ScrollAxis,
    /// How many items the list has (built or not).
    pub count: usize,
    /// Each item's extent along the axis (LAYOUT-LIST-03).
    pub extents: ListExtents,
}

impl RenderListContent {
    /// `count` items of `item_extent` along `axis` (`ListExtents::Fixed`).
    pub const fn new(axis: ScrollAxis, count: usize, item_extent: f32) -> Self {
        RenderListContent {
            axis,
            count,
            extents: ListExtents::Fixed(item_extent),
        }
    }
}

impl RenderListContent {
    /// The total length along the axis.
    fn main_size(&self) -> f32 {
        self.extents.total(self.count)
    }
}

impl RenderBox for RenderListContent {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
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
            let index = children
                .parent_data::<ListIndex>(i)
                .map_or(i, |data| data.0);
            let extent = self.extents.extent(index);
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
            let along = self.extents.start(index);
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
