//! [`DataGrid`], [`DataSource`], [`CellValue`], [`FnSource`] and [`GridColumn`]. Spec:
//! `docs/specs/widgets/data-grid.md`.

use std::fmt;
use std::rc::Rc;
use std::time::Duration;

use tantu_view::reactive::Signal;
use tantu_view::{AnyView, BuildCx, ElementId, View};

use crate::ScrollController;

/// A cell's value.
#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    /// No value.
    Empty,
    /// Text.
    Text(String),
    /// An integer.
    Int(i64),
    /// A floating-point number.
    Float(f64),
    /// A boolean.
    Bool(bool),
}

/// `Empty` is "", `Text` its text, `Int` and `Float` the number (`{}`), `Bool` "true" or
/// "false" (WIDGETS-GRID-01).
impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!()
    }
}

/// Where a grid's rows come from. Indices are 0-based; the grid asks only for the cells it
/// shows, each time it builds or refreshes them, so `cell` should be cheap or cached.
pub trait DataSource: 'static {
    /// How many rows there are.
    fn row_count(&self) -> usize;
    /// The value of a cell; `Empty` for one that doesn't exist (yet).
    fn cell(&self, row: usize, column: usize) -> CellValue;
}

/// A source computed from a closure, for generated or lazily loaded data.
pub struct FnSource {
    rows: usize,
    cell: Box<dyn Fn(usize, usize) -> CellValue>,
}

impl FnSource {
    /// `rows` rows whose cells `cell(row, column)` computes.
    pub fn new(rows: usize, cell: impl Fn(usize, usize) -> CellValue + 'static) -> Self {
        FnSource {
            rows,
            cell: Box::new(cell),
        }
    }
}

impl DataSource for FnSource {
    fn row_count(&self) -> usize {
        let _ = self.rows;
        todo!()
    }

    fn cell(&self, row: usize, column: usize) -> CellValue {
        let _ = (&self.cell, row, column);
        todo!()
    }
}

/// Builds a cell's content from the row and its value.
type CellBuilder = Rc<dyn Fn(usize, CellValue) -> AnyView>;

/// A column of a [`DataGrid`].
pub struct GridColumn {
    title: String,
    width: f32,
    builder: Option<CellBuilder>,
}

impl GridColumn {
    /// A column titled `title`, 120 px wide, showing values as text.
    pub fn new(title: impl Into<String>) -> Self {
        GridColumn {
            title: title.into(),
            width: 120.0,
            builder: None,
        }
    }

    /// The column's width in logical pixels (negative or NaN counts as 0).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Builds the cell's content from the row and its value instead of the default text
    /// (WIDGETS-GRID-03).
    pub fn builder<V: View>(mut self, build: impl Fn(usize, CellValue) -> V + 'static) -> Self {
        self.builder = Some(Rc::new(move |row, value| AnyView::new(build(row, value))));
        self
    }
}

/// A table over a [`DataSource`] that builds only its visible cells and scrolls on both axes
/// (the first part of the enterprise data grid).
pub struct DataGrid {
    columns: Vec<GridColumn>,
    source: Rc<dyn DataSource>,
    row_height: f32,
    header_height: f32,
    revision: Option<Signal<u64>>,
    controllers: Option<(ScrollController, ScrollController)>,
    wheel: Duration,
}

impl DataGrid {
    /// A grid with `columns` over `source`.
    pub fn new(columns: impl IntoIterator<Item = GridColumn>, source: Rc<dyn DataSource>) -> Self {
        DataGrid {
            columns: columns.into_iter().collect(),
            source,
            row_height: 32.0,
            header_height: 36.0,
            revision: None,
            controllers: None,
            wheel: Duration::from_millis(150),
        }
    }

    /// Every row's height (default 32; negative or NaN counts as 0).
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// The header row's height (default 36; 0 hides the header).
    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height;
        self
    }

    /// A signal the app bumps when the source's data changed: the row count and the shown
    /// cells are re-read (WIDGETS-GRID-06).
    pub fn revision(mut self, revision: Signal<u64>) -> Self {
        self.revision = Some(revision);
        self
    }

    /// Shares the vertical and horizontal scroll controllers with the app.
    pub fn controllers(mut self, vertical: ScrollController, horizontal: ScrollController) -> Self {
        self.controllers = Some((vertical, horizontal));
        self
    }

    /// How long a wheel step takes (default 150 ms; zero applies deltas at once), as
    /// `ScrollView::wheel_duration`.
    pub fn wheel_duration(mut self, duration: Duration) -> Self {
        self.wheel = duration;
        self
    }
}

impl View for DataGrid {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.columns.iter().map(|c| (&c.title, c.width, &c.builder)),
            &self.source,
            self.row_height,
            self.header_height,
            self.revision,
            self.controllers,
            self.wheel,
            cx,
        );
        todo!()
    }
}
