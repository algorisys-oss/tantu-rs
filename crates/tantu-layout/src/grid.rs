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
        Size::new(
            self.columns.total(self.column_count),
            self.rows.total(self.row_count),
        )
    }

    /// The offset with non-finite components counting as 0.
    fn finite_offset(&self) -> Vec2 {
        let finite = |v: f32| if v.is_finite() { v } else { 0.0 };
        Vec2::new(finite(self.offset.x), finite(self.offset.y))
    }
}

impl RenderBox for RenderGridBody {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let offset = self.finite_offset();
        for i in 0..children.len() {
            let slot = children
                .parent_data::<GridSlot>(i)
                .copied()
                .unwrap_or(GridSlot { row: 0, column: 0 });
            let size = Size::new(self.columns.extent(slot.column), self.rows.extent(slot.row));
            children.layout(i, BoxConstraints::tight(size));
            // Frozen columns ignore the horizontal offset (LAYOUT-GRID-04).
            let x = self.columns.start(slot.column)
                - if slot.column < self.frozen_columns {
                    0.0
                } else {
                    offset.x
                };
            let y = self.rows.start(slot.row) - offset.y;
            children.set_offset(i, Vec2::new(x, y));
        }
        // The incoming maximum on a bounded axis, the content's extent on an unbounded one.
        let content = self.content();
        let width = if constraints.has_bounded_width() {
            constraints.max_width
        } else {
            content.width
        };
        let height = if constraints.has_bounded_height() {
            constraints.max_height
        } else {
            content.height
        };
        constraints.constrain(Size::new(width, height))
    }

    fn min_intrinsic_width(&self, _height: f32, _children: &mut IntrinsicChildren<'_>) -> f32 {
        self.content().width
    }

    fn max_intrinsic_width(&self, _height: f32, _children: &mut IntrinsicChildren<'_>) -> f32 {
        self.content().width
    }

    fn min_intrinsic_height(&self, _width: f32, _children: &mut IntrinsicChildren<'_>) -> f32 {
        self.content().height
    }

    fn max_intrinsic_height(&self, _width: f32, _children: &mut IntrinsicChildren<'_>) -> f32 {
        self.content().height
    }
}
