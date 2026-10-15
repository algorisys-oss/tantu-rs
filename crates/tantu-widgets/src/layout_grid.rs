//! [`LayoutGrid`] and [`GridPlacement`]: children on column and row tracks. Spec:
//! `docs/specs/widgets/layout-grid.md`.

use tantu_view::layout::{GridPlacementData, RenderLayoutGrid, TextDirection, TrackSize};
use tantu_view::{AnyView, BuildCx, ElementId, View};

use crate::directionality::direction_prop;

/// Lays children out on column and row tracks (`flutter_layout_grid`'s `LayoutGrid`,
/// WIDGETS-LGRID-01).
pub struct LayoutGrid {
    columns: Vec<TrackSize>,
    rows: Vec<TrackSize>,
    column_gap: f32,
    row_gap: f32,
    children: Vec<AnyView>,
}

impl LayoutGrid {
    /// A grid with these column tracks.
    pub fn new(columns: impl IntoIterator<Item = TrackSize>) -> Self {
        LayoutGrid {
            columns: columns.into_iter().collect(),
            rows: Vec::new(),
            column_gap: 0.0,
            row_gap: 0.0,
            children: Vec::new(),
        }
    }

    /// Row tracks (default none: every row is an implicit auto row).
    pub fn rows(mut self, rows: impl IntoIterator<Item = TrackSize>) -> Self {
        self.rows = rows.into_iter().collect();
        self
    }

    /// Space between columns.
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self
    }

    /// Space between rows.
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap;
        self
    }

    /// Appends a child (placed automatically unless it is a [`GridPlacement`]).
    pub fn child(mut self, child: impl View) -> Self {
        self.children.push(AnyView::new(child));
        self
    }

    /// Appends children.
    pub fn children<V: View>(mut self, children: impl IntoIterator<Item = V>) -> Self {
        self.children.extend(children.into_iter().map(AnyView::new));
        self
    }
}

impl View for LayoutGrid {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let direction = direction_prop(cx);
        let mut render = RenderLayoutGrid::new(self.columns, self.rows);
        render.column_gap = self.column_gap;
        render.row_gap = self.row_gap;
        let id = cx.render(render, self.children);
        cx.bind(id, direction, |e, direction: TextDirection| {
            e.update_render::<RenderLayoutGrid>(|r| r.text_direction = direction);
        });
        id
    }
}

/// Places its child on a [`LayoutGrid`] (`flutter_layout_grid`'s `GridPlacement`).
pub struct GridPlacement {
    child: AnyView,
    column: Option<usize>,
    row: Option<usize>,
    column_span: usize,
    row_span: usize,
}

impl GridPlacement {
    /// `child`, placed automatically until a column or row is set.
    pub fn new(child: impl View) -> Self {
        GridPlacement {
            child: AnyView::new(child),
            column: None,
            row: None,
            column_span: 1,
            row_span: 1,
        }
    }

    /// The first column.
    pub fn column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// The first row.
    pub fn row(mut self, row: usize) -> Self {
        self.row = Some(row);
        self
    }

    /// Columns covered.
    pub fn column_span(mut self, span: usize) -> Self {
        self.column_span = span;
        self
    }

    /// Rows covered.
    pub fn row_span(mut self, span: usize) -> Self {
        self.row_span = span;
        self
    }
}

impl View for GridPlacement {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.child.build(cx);
        let data = GridPlacementData {
            column: self.column,
            row: self.row,
            column_span: self.column_span,
            row_span: self.row_span,
        };
        if !cx.set_parent_data(id, data) {
            tracing::warn!(
                "GridPlacement around a dynamic region has no element to hold its placement; \
                 the child is placed automatically"
            );
        }
        id
    }
}
