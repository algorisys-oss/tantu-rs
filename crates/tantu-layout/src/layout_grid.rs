//! [`RenderLayoutGrid`]: children on column and row tracks. Spec:
//! `docs/specs/widgets/layout-grid.md`.

use tantu_core::Size;

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox, TextDirection};

/// How a column or row is sized (LAYOUT-LGRID-01).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackSize {
    /// A fixed extent (negative or NaN counts as 0).
    Fixed(f32),
    /// A share of the free space, in proportion to the factor (negative or NaN counts as 0).
    Flex(f32),
    /// The largest natural extent of the items in it.
    Auto,
}

impl TrackSize {
    /// A fixed extent.
    pub const fn px(extent: f32) -> Self {
        TrackSize::Fixed(extent)
    }

    /// A flexible share.
    pub const fn fr(factor: f32) -> Self {
        TrackSize::Flex(factor)
    }

    /// Sized to its items.
    pub const AUTO: TrackSize = TrackSize::Auto;
}

/// Where an item sits (parent data of a [`RenderLayoutGrid`] child, LAYOUT-LGRID-02).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridPlacementData {
    /// The first column, or `None` for automatic placement.
    pub column: Option<usize>,
    /// The first row, or `None` for automatic placement.
    pub row: Option<usize>,
    /// Columns covered (0 counts as 1).
    pub column_span: usize,
    /// Rows covered (0 counts as 1).
    pub row_span: usize,
}

/// Lays children out on column and row tracks (CSS Grid's track sizing, Flutter's
/// `flutter_layout_grid`; LAYOUT-LGRID-01..06).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderLayoutGrid {
    /// The column tracks.
    pub columns: Vec<TrackSize>,
    /// The row tracks; rows past them are implicit auto rows.
    pub rows: Vec<TrackSize>,
    /// Space between columns (negative or NaN counts as 0).
    pub column_gap: f32,
    /// Space between rows (negative or NaN counts as 0).
    pub row_gap: f32,
    /// `Rtl` runs the columns from the right.
    pub text_direction: TextDirection,
}

impl RenderLayoutGrid {
    /// A grid with these tracks, no gaps, left to right.
    pub fn new(columns: Vec<TrackSize>, rows: Vec<TrackSize>) -> Self {
        RenderLayoutGrid {
            columns,
            rows,
            column_gap: 0.0,
            row_gap: 0.0,
            text_direction: TextDirection::Ltr,
        }
    }
}

impl RenderBox for RenderLayoutGrid {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!("LAYOUT-LGRID-02..05")
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!("LAYOUT-LGRID-06")
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!("LAYOUT-LGRID-06")
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!("LAYOUT-LGRID-06")
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!("LAYOUT-LGRID-06")
    }
}
