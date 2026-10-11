//! [`RenderGridBody`] and [`GridSlot`]: a two-dimensional virtualized content. Spec:
//! `docs/specs/layout/grid.md`.

use tantu_core::{Size, Vec2};

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, ListExtents, RenderBox};

/// Which cell a built child of a [`RenderGridBody`] is (parent data).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridSlot {
    /// The cell's row.
    pub row: usize,
    /// The cell's column.
    pub column: usize,
}

/// A two-dimensional virtualized content: `row_count` × `column_count` cells with the given
/// extents, scrolled by `offset`, placing the children it is given by their [`GridSlot`]
/// (the two-dimensional `RenderListContent`).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderGridBody {
    /// How many rows there are (built or not).
    pub row_count: usize,
    /// Each row's height.
    pub rows: ListExtents,
    /// How many columns there are.
    pub column_count: usize,
    /// Each column's width.
    pub columns: ListExtents,
    /// How far the content is scrolled (positive scrolls content up and left); a non-finite
    /// component counts as 0.
    pub offset: Vec2,
    /// The first `frozen_columns` columns ignore `offset.x` (pinned at the left)
    /// (LAYOUT-GRID-04).
    pub frozen_columns: usize,
}

impl RenderGridBody {
    /// `row_count` rows and `column_count` columns of the given extents, at offset zero with
    /// no frozen columns.
    pub fn new(
        rows: ListExtents,
        row_count: usize,
        columns: ListExtents,
        column_count: usize,
    ) -> Self {
        RenderGridBody {
            row_count,
            rows,
            column_count,
            columns,
            offset: Vec2::ZERO,
            frozen_columns: 0,
        }
    }

    /// The content's size: every column's width by every row's height (LAYOUT-GRID-03).
    pub fn content(&self) -> Size {
        todo!()
    }
}

impl RenderBox for RenderGridBody {
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
