//! [`DataGrid`], [`DataSource`], [`CellValue`], [`FnSource`] and [`GridColumn`]. Specs:
//! `docs/specs/widgets/data-grid.md`, `data-grid-sort.md` and `data-grid-columns.md`.

use std::cell::Cell;
use std::cmp::Ordering;
use std::fmt;
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

use tantu_view::core::{Affine, Color, EdgeInsets, Point, Rect, Size, Vec2};
use tantu_view::layout::{
    Alignment, BoxConstraints, CrossAxisAlignment, GridSlot, ListExtents, RenderConstrainedBox,
    RenderGridBody, ScrollAxis,
};
use tantu_view::reactive::{Memo, Signal, memo, signal};
use tantu_view::scene::{BorderRadius, Clip, RoundedRect};
use tantu_view::{
    AnyView, BuildCx, Dyn, ElementId, FocusOptions, For, Handled, LogicalKey, NamedKey, Paint,
    PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics, View,
};

use crate::scroll::{KEY_STEP, Scrollbar, THUMB_ALPHA, THUMB_WIDTH};
use crate::theme::{ThemeHandle, at};
use crate::{Align, Column, Expanded, Padding, ScrollController, Text};

/// A cell's horizontal padding.
const CELL_PADDING: f32 = 8.0;
/// The default column width.
const COLUMN_WIDTH: f32 = 120.0;
/// The narrowest a column can be dragged by default (WIDGETS-GRID-13).
const MIN_COLUMN_WIDTH: f32 = 40.0;
/// The header's bold weight.
const HEADER_WEIGHT: f32 = 700.0;
/// The resize handle: this many pixels left of a column's right boundary (WIDGETS-GRID-13).
const HANDLE_WIDTH: f32 = 6.0;
/// How far a header press moves before it is a reorder drag, not a click (WIDGETS-GRID-14).
const DRAG_SLOP: f32 = 4.0;
/// The drop line's thickness (WIDGETS-GRID-14).
const DROP_LINE: f32 = 2.0;
/// The sort chevron's width and bar thickness (WIDGETS-GRID-10).
const CHEVRON_WIDTH: f32 = 10.0;
const CHEVRON_STROKE: f32 = 2.0;

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
        match self {
            CellValue::Empty => Ok(()),
            CellValue::Text(text) => f.write_str(text),
            CellValue::Int(n) => write!(f, "{n}"),
            CellValue::Float(x) => write!(f, "{x}"),
            CellValue::Bool(b) => write!(f, "{b}"),
        }
    }
}

impl CellValue {
    /// A total order over values for sorting: `Empty`, then `Bool` (false first), then
    /// numbers (`Int` and `Float` compared as `f64`, NaN lowest), then `Text` (byte order)
    /// (WIDGETS-GRID-11).
    pub fn compare(&self, other: &CellValue) -> Ordering {
        fn rank(value: &CellValue) -> u8 {
            match value {
                CellValue::Empty => 0,
                CellValue::Bool(_) => 1,
                CellValue::Int(_) | CellValue::Float(_) => 2,
                CellValue::Text(_) => 3,
            }
        }
        fn number(value: &CellValue) -> Option<f64> {
            match value {
                CellValue::Int(n) => Some(*n as f64),
                CellValue::Float(x) => Some(*x),
                _ => None,
            }
        }
        match (self, other) {
            (CellValue::Bool(a), CellValue::Bool(b)) => a.cmp(b),
            (CellValue::Text(a), CellValue::Text(b)) => a.cmp(b),
            _ => match (number(self), number(other)) {
                (Some(a), Some(b)) => match (a.is_nan(), b.is_nan()) {
                    (true, true) => Ordering::Equal,
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    (false, false) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
                },
                _ => rank(self).cmp(&rank(other)),
            },
        }
    }
}

/// Which column the grid is sorted by, and how (Flutter's `sortColumnIndex` and
/// `sortAscending`) (WIDGETS-GRID-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortState {
    /// The sorted column.
    pub column: usize,
    /// Ascending (true) or descending.
    pub ascending: bool,
}

/// A predicate over a source's rows (WIDGETS-GRID-12).
pub type RowFilter = Rc<dyn Fn(&dyn DataSource, usize) -> bool>;

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
        self.rows
    }

    fn cell(&self, row: usize, column: usize) -> CellValue {
        (self.cell)(row, column)
    }
}

/// Builds a cell's content from the row and its value.
type CellBuilder = Rc<dyn Fn(usize, CellValue) -> AnyView>;

/// A column of a [`DataGrid`].
pub struct GridColumn {
    title: String,
    width: f32,
    builder: Option<CellBuilder>,
    /// Whether a header click sorts by this column; `None` follows `client_sort`.
    sortable: Option<bool>,
    /// Whether the user can drag the column's right edge (WIDGETS-GRID-13).
    resizable: bool,
    /// The narrowest the user can drag the column (WIDGETS-GRID-13).
    min_width: f32,
}

impl GridColumn {
    /// A column titled `title`, 120 px wide, showing values as text.
    pub fn new(title: impl Into<String>) -> Self {
        GridColumn {
            title: title.into(),
            width: COLUMN_WIDTH,
            builder: None,
            sortable: None,
            resizable: true,
            min_width: MIN_COLUMN_WIDTH,
        }
    }

    /// Whether the user can drag the column's right edge (default true) (WIDGETS-GRID-13).
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// The narrowest the user can drag the column (default 40 px; negative or NaN counts
    /// as 0) (WIDGETS-GRID-13).
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Whether a header click sorts by this column (default: only with `client_sort`)
    /// (WIDGETS-GRID-10).
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = Some(sortable);
        self
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

/// A table over a [`DataSource`] that builds only its visible cells and scrolls on both
/// axes, with frozen columns, sorting and filtering, and columns the user can resize and
/// reorder (the enterprise data grid; selection and cell editors are still to come).
pub struct DataGrid {
    columns: Vec<GridColumn>,
    source: Rc<dyn DataSource>,
    row_height: f32,
    header_height: f32,
    revision: Option<Signal<u64>>,
    controllers: Option<(ScrollController, ScrollController)>,
    wheel: Duration,
    frozen: usize,
    sort: Option<Signal<Option<SortState>>>,
    client_sort: bool,
    filter: Option<Signal<Option<RowFilter>>>,
    widths: Option<Signal<Vec<f32>>>,
    reorderable: bool,
    order: Option<Signal<Vec<usize>>>,
}

impl DataGrid {
    /// Enables row, cell or rectangular selection (default disabled).
    pub fn selection_mode(self, _mode: crate::GridSelectionMode) -> Self {
        todo!("WIDGETS-GRID-16")
    }

    /// Shares selection and source-cell mapping with the application.
    pub fn selection_controller(self, _controller: crate::GridSelectionController) -> Self {
        todo!("WIDGETS-GRID-16")
    }

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
            frozen: 0,
            sort: None,
            client_sort: false,
            filter: None,
            widths: None,
            reorderable: false,
            order: None,
        }
    }

    /// Shares the column widths, indexed by source column (the grid owns one, seeded from
    /// each `GridColumn::width`, otherwise) (WIDGETS-GRID-13).
    pub fn column_widths(mut self, widths: Signal<Vec<f32>>) -> Self {
        self.widths = Some(widths);
        self
    }

    /// Lets the user drag headers to reorder columns (default false) (WIDGETS-GRID-14).
    pub fn reorderable(mut self, enabled: bool) -> Self {
        self.reorderable = enabled;
        self
    }

    /// Shares the display order: source column indices in display order (the grid owns one,
    /// the identity, otherwise) (WIDGETS-GRID-14).
    pub fn column_order(mut self, order: Signal<Vec<usize>>) -> Self {
        self.order = Some(order);
        self
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

    /// The first `count` columns stay at the left while the others scroll (clamped to the
    /// column count; default 0) (WIDGETS-GRID-09).
    pub fn frozen_columns(mut self, count: usize) -> Self {
        self.frozen = count;
        self
    }

    /// Shares the sort state with the app (the grid owns one otherwise) (WIDGETS-GRID-10).
    pub fn sort_state(mut self, state: Signal<Option<SortState>>) -> Self {
        self.sort = Some(state);
        self
    }

    /// The grid orders the rows itself by the sorted column's values, and every column is
    /// sortable unless it says otherwise (default false: the app reorders its source)
    /// (WIDGETS-GRID-11).
    pub fn client_sort(mut self, enabled: bool) -> Self {
        self.client_sort = enabled;
        self
    }

    /// Shows only the source rows the predicate accepts (`None`: every row) (WIDGETS-GRID-12).
    pub fn filter(mut self, filter: Signal<Option<RowFilter>>) -> Self {
        self.filter = Some(filter);
        self
    }
}

/// A non-negative finite length, else 0.
fn clean(length: f32) -> f32 {
    if length.is_finite() && length >= 0.0 {
        length
    } else {
        0.0
    }
}

/// A display order over `count` columns from the app's: duplicates and out-of-range entries
/// dropped, missing columns following in source order (WIDGETS-GRID-14).
fn normalize_order(order: &[usize], count: usize) -> Vec<usize> {
    let mut seen = vec![false; count];
    let mut out = Vec::with_capacity(count);
    for &column in order {
        if column < count && !seen[column] {
            seen[column] = true;
            out.push(column);
        }
    }
    out.extend((0..count).filter(|c| !seen[*c]));
    out
}

/// A column's sizing: its `GridColumn::width`, its minimum and whether it is resizable
/// (WIDGETS-GRID-13).
#[derive(Clone, Copy)]
struct ColumnSizing {
    width: f32,
    min_width: f32,
    resizable: bool,
}

/// The columns as displayed: source indices in display order and their extents in that
/// order (WIDGETS-GRID-13, -14).
#[derive(Clone, PartialEq)]
struct ColumnLayout {
    order: Rc<[usize]>,
    extents: ListExtents,
}

/// A cell to build: its row, the display position of its column and the source column.
/// The `For` key, so a reordered cell is rebuilt for its new position.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct CellKey {
    row: usize,
    display: usize,
    source: usize,
}

impl CellKey {
    fn slot(self) -> GridSlot {
        GridSlot {
            row: self.row,
            column: self.display,
        }
    }
}

/// A header held for reordering (WIDGETS-GRID-14): the column, the boundary it would drop
/// at (a display position, `column_count` for after the last) and that boundary's x in the
/// header strip.
#[derive(Clone, Copy, PartialEq)]
struct Reorder {
    source: usize,
    boundary: usize,
    x: f32,
}

/// The grid's geometry, shared by the strips, the handlers and the paint.
#[derive(Clone)]
struct Geometry {
    row_height: f32,
    header_height: f32,
    /// The row count, re-read when the revision changes (WIDGETS-GRID-06).
    row_count: Memo<usize>,
    column_count: usize,
    /// The display order and extents, following `widths` and `order` (WIDGETS-GRID-13, -14).
    layout: Memo<ColumnLayout>,
    sizing: Rc<[ColumnSizing]>,
    /// The widths by source column, the grid's or the app's (WIDGETS-GRID-13).
    widths: Signal<Vec<f32>>,
    /// The display order, the grid's or the app's (WIDGETS-GRID-14).
    order: Signal<Vec<usize>>,
    reorderable: bool,
    /// The header being dragged, for the drop line and the held cell (WIDGETS-GRID-14).
    drag: Signal<Option<Reorder>>,
    /// The leading display positions pinned at the left (WIDGETS-GRID-09), at most
    /// `column_count`.
    frozen_columns: usize,
}

impl Geometry {
    /// The body's content size, from the current row count (an untracked read).
    fn content_untracked(&self) -> Size {
        Size::new(
            self.layout
                .with_untracked(|l| l.extents.total(self.column_count)),
            self.row_height * self.row_count.get_untracked() as f32,
        )
    }

    /// The columns to build, as (display position, source column): the scrolling ones
    /// between the frozen width and `viewport` (one of buffer each side, WIDGETS-GRID-02),
    /// then the frozen ones, which paint last and so on top (WIDGETS-GRID-09). A tracked
    /// read of the layout.
    fn visible_columns(&self, offset: f32, viewport: f32) -> Vec<(usize, usize)> {
        let frozen = self.frozen_columns;
        self.layout.with(|layout| {
            let frozen_width = layout.extents.start(frozen);
            let scrolling = if viewport > frozen_width {
                range(
                    &layout.extents,
                    self.column_count,
                    offset + frozen_width,
                    viewport - frozen_width,
                )
            } else {
                0..0
            };
            scrolling
                .filter(|column| *column >= frozen)
                .chain(0..frozen)
                .map(|display| (display, layout.order[display]))
                .collect()
        })
    }

    /// The grid body for the strip: `rows` rows with the grid's columns and frozen columns.
    fn body(&self, rows: ListExtents, row_count: usize) -> RenderGridBody {
        let mut body = RenderGridBody::new(
            rows,
            row_count,
            self.layout.with_untracked(|l| l.extents.clone()),
            self.column_count,
        );
        body.frozen_columns = self.frozen_columns;
        body
    }

    /// Keeps a strip's column extents in step with the layout (WIDGETS-GRID-13, -14).
    fn bind_extents(&self, cx: &mut BuildCx<'_>, id: ElementId) {
        let layout = self.layout;
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || layout.with(|l| l.extents.clone()))),
            |e, extents| {
                e.update_render::<RenderGridBody>(|b| b.columns = extents);
            },
        );
    }

    /// The horizontal offset as the strips apply it.
    fn offset_untracked(&self, horizontal: ScrollController) -> f32 {
        let max = horizontal.max_scroll_extent_untracked().max(0.0);
        horizontal.offset_untracked().clamp(0.0, max)
    }

    /// The x of display boundary `k` in a strip, given the horizontal offset: frozen
    /// boundaries don't scroll.
    fn boundary_x(&self, extents: &ListExtents, k: usize, offset: f32) -> f32 {
        let start = extents.start(k);
        if k <= self.frozen_columns {
            start
        } else {
            start - offset
        }
    }

    /// The display boundary nearest `x` in the strip (WIDGETS-GRID-14).
    fn nearest_boundary(&self, x: f32, offset: f32) -> (usize, f32) {
        self.layout.with_untracked(|layout| {
            (0..=self.column_count)
                .map(|k| (k, self.boundary_x(&layout.extents, k, offset)))
                .min_by(|a, b| {
                    (a.1 - x)
                        .abs()
                        .partial_cmp(&(b.1 - x).abs())
                        .unwrap_or(Ordering::Equal)
                })
                .unwrap_or((0, 0.0))
        })
    }

    /// Whether `column` is frozen, and whether it is the last frozen one (the edge).
    fn frozen(&self, column: usize) -> (bool, bool) {
        (
            column < self.frozen_columns,
            self.frozen_columns > 0 && column + 1 == self.frozen_columns,
        )
    }

    /// The rows likewise, for `count` rows.
    fn row_range(&self, count: usize, offset: f32, viewport: f32) -> Range<usize> {
        range(
            &ListExtents::Fixed(self.row_height),
            count,
            offset,
            viewport,
        )
    }
}

/// The items of `extents` from `offset` to `offset + viewport` (the one starting exactly at
/// the end excluded), with one of buffer each side.
fn range(extents: &ListExtents, count: usize, offset: f32, viewport: f32) -> Range<usize> {
    let end = (offset + viewport).next_down().max(offset);
    let (Some(first), Some(last)) = (
        extents.index_at(offset, count),
        extents.index_at(end, count),
    ) else {
        return 0..0;
    };
    let first = first.saturating_sub(1);
    let last = (last + 2).min(count);
    first..last.max(first)
}

/// What the handlers share with the paint: the grid's size and its content's size from the
/// last paint.
type Shared = Rc<Cell<(Size, Size)>>;

impl View for DataGrid {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (vertical, horizontal) = self.controllers.unwrap_or_default();
        let theme = ThemeHandle::of(cx);
        let wheel = self.wheel;
        let revision = self.revision;
        let source = self.source;
        // The sort state (WIDGETS-GRID-10) and the row map behind filtering and client
        // sorting (WIDGETS-GRID-11, -12): the filtered rows, then their order.
        let sort = self.sort.unwrap_or_else(|| signal(None));
        let (client_sort, filter) = (self.client_sort, self.filter);
        let filtered: Memo<Option<Rc<[usize]>>> = {
            let source = Rc::clone(&source);
            memo(move || {
                if let Some(revision) = revision {
                    revision.get();
                }
                let accept = filter.and_then(|f| f.get())?;
                let count = source.row_count();
                let rows: Vec<usize> = (0..count).filter(|row| accept(&*source, *row)).collect();
                Some(Rc::from(rows))
            })
        };
        let map: Memo<Option<Rc<[usize]>>> = {
            let source = Rc::clone(&source);
            memo(move || {
                let filtered = filtered.get();
                let sort = if client_sort { sort.get() } else { None };
                let Some(SortState { column, ascending }) = sort else {
                    return filtered;
                };
                let mut rows: Vec<usize> = match filtered {
                    Some(rows) => rows.to_vec(),
                    None => {
                        if let Some(revision) = revision {
                            revision.get();
                        }
                        (0..source.row_count()).collect()
                    }
                };
                rows.sort_by(|a, b| {
                    let order = source.cell(*a, column).compare(&source.cell(*b, column));
                    if ascending { order } else { order.reverse() }
                });
                Some(Rc::from(rows))
            })
        };
        let row_count = {
            let source = Rc::clone(&source);
            memo(move || match map.get() {
                Some(rows) => rows.len(),
                None => {
                    if let Some(revision) = revision {
                        revision.get();
                    }
                    source.row_count()
                }
            })
        };
        // The widths and the display order (WIDGETS-GRID-13, -14): the app's signals or the
        // grid's own, combined into the display layout.
        let column_count = self.columns.len();
        let sizing: Rc<[ColumnSizing]> = self
            .columns
            .iter()
            .map(|c| ColumnSizing {
                width: clean(c.width),
                min_width: clean(c.min_width),
                resizable: c.resizable,
            })
            .collect();
        let widths = self
            .widths
            .unwrap_or_else(|| signal(sizing.iter().map(|s| s.width).collect()));
        let order = self
            .order
            .unwrap_or_else(|| signal((0..column_count).collect()));
        let layout = {
            let sizing = Rc::clone(&sizing);
            memo(move || {
                let widths: Vec<f32> = widths.with(|w| {
                    sizing
                        .iter()
                        .enumerate()
                        .map(|(c, s)| w.get(c).copied().map_or(s.width, clean).max(s.min_width))
                        .collect()
                });
                let order = order.with(|o| normalize_order(o, column_count));
                let extents = ListExtents::table(order.iter().map(|&c| widths[c]));
                ColumnLayout {
                    order: Rc::from(order),
                    extents,
                }
            })
        };
        let geometry = Geometry {
            row_height: clean(self.row_height),
            header_height: clean(self.header_height),
            row_count,
            column_count,
            layout,
            sizing,
            widths,
            order,
            reorderable: self.reorderable,
            drag: signal(None),
            frozen_columns: self.frozen.min(column_count),
        };
        let columns: Rc<[ColumnSpec]> = self
            .columns
            .into_iter()
            .map(|c| ColumnSpec {
                title: c.title,
                builder: c.builder,
                sortable: c.sortable.unwrap_or(client_sort),
            })
            .collect();
        let header = HeaderStrip {
            geometry: geometry.clone(),
            columns: Rc::clone(&columns),
            horizontal,
            theme,
            sort,
        };
        let body = Body {
            geometry: geometry.clone(),
            columns,
            source,
            revision,
            map,
            vertical,
            horizontal,
            theme,
        };
        let id = Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .child(header)
            .child(Expanded::new(body))
            .build(cx);
        // The grid's role (WIDGETS-GRID-07).
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Grid,
                ..Default::default()
            },
        );
        let shared: Shared = Rc::default();
        cx.set_paint(
            id,
            GridPaint {
                geometry: geometry.clone(),
                vertical,
                horizontal,
                shared: Rc::clone(&shared),
                thumb: Color::TRANSPARENT,
            },
        );
        cx.bind(
            id,
            theme.color_prop(|c| at(c.on_surface, THUMB_ALPHA)),
            |e, color| {
                e.update_paint::<GridPaint>(|p| p.thumb = color);
            },
        );
        // Keys (WIDGETS-GRID-05): a press focuses the grid, Tab doesn't stop at it.
        cx.focusable(
            id,
            FocusOptions {
                traversable: false,
                focus_on_press: true,
            },
        );
        {
            let (shared, geometry) = (Rc::clone(&shared), geometry.clone());
            cx.on_key(id, Phase::Bubble, move |k| {
                if !k.event.pressed {
                    return Handled::Continue;
                }
                let (size, _) = shared.get();
                let body = (size.height - geometry.header_height).max(0.0);
                let (current, max) = (
                    vertical.offset_untracked(),
                    vertical.max_scroll_extent_untracked(),
                );
                let moved = match &k.event.key {
                    LogicalKey::Named(NamedKey::ArrowDown) => {
                        vertical.scroll_by(geometry.row_height)
                    }
                    LogicalKey::Named(NamedKey::ArrowUp) => {
                        vertical.scroll_by(-geometry.row_height)
                    }
                    LogicalKey::Named(NamedKey::ArrowRight) => horizontal.scroll_by(KEY_STEP),
                    LogicalKey::Named(NamedKey::ArrowLeft) => horizontal.scroll_by(-KEY_STEP),
                    LogicalKey::Named(NamedKey::PageDown) => vertical.scroll_by(body),
                    LogicalKey::Named(NamedKey::PageUp) => vertical.scroll_by(-body),
                    LogicalKey::Named(NamedKey::End) => vertical.scroll_by(max - current),
                    LogicalKey::Named(NamedKey::Home) => vertical.scroll_by(-current),
                    _ => return Handled::Continue,
                };
                if moved {
                    Handled::Stop
                } else {
                    Handled::Continue
                }
            });
        }
        // Wheel and thumbs (WIDGETS-GRID-04).
        let drag: Rc<Cell<Option<(ScrollAxis, f32, f32)>>> = Rc::default();
        cx.on_pointer(id, Phase::Bubble, move |p| {
            let (size, content) = shared.get();
            let bars = Bars::new(&geometry, size, content);
            match p.event.kind {
                PointerKind::Scroll(delta) => {
                    let moved_y = vertical.wheel(delta.y, wheel);
                    let moved_x = horizontal.wheel(delta.x, wheel);
                    if moved_x || moved_y {
                        Handled::Stop
                    } else {
                        Handled::Continue
                    }
                }
                PointerKind::Down(PointerButton::Primary) => {
                    let hit = [
                        (ScrollAxis::Vertical, vertical),
                        (ScrollAxis::Horizontal, horizontal),
                    ]
                    .into_iter()
                    .find(|(axis, controller)| {
                        bars.thumb(*axis, controller.offset_untracked())
                            .is_some_and(|thumb| thumb.contains(p.local))
                    });
                    match hit {
                        Some((axis, controller)) => {
                            drag.set(Some((
                                axis,
                                along(axis, p.local),
                                controller.offset_untracked(),
                            )));
                            Handled::Stop
                        }
                        None => Handled::Continue,
                    }
                }
                PointerKind::Move => match drag.get() {
                    Some((axis, start, offset)) => {
                        let bar = bars.bar(axis);
                        let moved = (along(axis, p.local) - start) * bar.ratio();
                        let target = (offset + moved).clamp(0.0, bar.max());
                        let controller = match axis {
                            ScrollAxis::Vertical => vertical,
                            ScrollAxis::Horizontal => horizontal,
                        };
                        if target != controller.offset_untracked() {
                            controller.jump_to(target);
                        }
                        Handled::Stop
                    }
                    None => Handled::Continue,
                },
                PointerKind::Up(PointerButton::Primary) if drag.get().is_some() => {
                    drag.set(None);
                    Handled::Stop
                }
                _ => Handled::Continue,
            }
        });
        id
    }
}

/// A point's coordinate along an axis.
fn along(axis: ScrollAxis, point: Point) -> f32 {
    match axis {
        ScrollAxis::Vertical => point.y,
        ScrollAxis::Horizontal => point.x,
    }
}

/// The two scrollbars' geometry: the vertical one in the body (below the header), the
/// horizontal one along the grid's bottom edge.
struct Bars {
    header_height: f32,
    vertical: Scrollbar,
    horizontal: Scrollbar,
}

impl Bars {
    fn new(geometry: &Geometry, size: Size, content: Size) -> Self {
        let body = Size::new(size.width, (size.height - geometry.header_height).max(0.0));
        Bars {
            header_height: geometry.header_height,
            vertical: Scrollbar::from(ScrollAxis::Vertical, body, content),
            horizontal: Scrollbar::from(ScrollAxis::Horizontal, size, content),
        }
    }

    fn bar(&self, axis: ScrollAxis) -> Scrollbar {
        match axis {
            ScrollAxis::Vertical => self.vertical,
            ScrollAxis::Horizontal => self.horizontal,
        }
    }

    /// The thumb's rect in the grid's coordinates, when the axis overflows.
    fn thumb(&self, axis: ScrollAxis, offset: f32) -> Option<Rect> {
        let thumb = self.bar(axis).thumb_rect(offset)?;
        Some(match axis {
            ScrollAxis::Vertical => thumb.translate(Vec2::new(0.0, self.header_height)),
            ScrollAxis::Horizontal => thumb,
        })
    }
}

/// Paints the strips, then the scrollbars, and records the extents (WIDGETS-GRID-04).
struct GridPaint {
    geometry: Geometry,
    vertical: ScrollController,
    horizontal: ScrollController,
    shared: Shared,
    /// The thumbs' color (the theme's `on_surface` at 40 %).
    thumb: Color,
}

impl Paint for GridPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let content = self.geometry.content_untracked();
        self.shared.set((size, content));
        let bars = Bars::new(&self.geometry, size, content);
        self.vertical
            .set_extents(bars.vertical.viewport, bars.vertical.max());
        self.horizontal
            .set_extents(bars.horizontal.viewport, bars.horizontal.max());
        cx.paint_children();
        for (axis, controller) in [
            (ScrollAxis::Vertical, self.vertical),
            (ScrollAxis::Horizontal, self.horizontal),
        ] {
            if let Some(thumb) = bars.thumb(axis, controller.offset_untracked()) {
                cx.scene().fill(
                    RoundedRect::new(thumb, BorderRadius::circular(THUMB_WIDTH / 2.0)),
                    self.thumb,
                );
            }
        }
    }
}

/// A column's title and cell builder.
struct ColumnSpec {
    title: String,
    builder: Option<CellBuilder>,
    /// Whether a header press sorts by the column (WIDGETS-GRID-10).
    sortable: bool,
}

/// The header: one row of column titles, following the horizontal offset only.
struct HeaderStrip {
    geometry: Geometry,
    columns: Rc<[ColumnSpec]>,
    horizontal: ScrollController,
    theme: ThemeHandle,
    sort: Signal<Option<SortState>>,
}

impl View for HeaderStrip {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (geometry, horizontal, theme, sort) =
            (self.geometry, self.horizontal, self.theme, self.sort);
        let mut style = theme.snapshot().text.label;
        style.weight = HEADER_WEIGHT;
        let cells = {
            let (geometry, geometry_cells, columns) =
                (geometry.clone(), geometry.clone(), self.columns);
            For::new(
                move || {
                    if geometry.header_height <= 0.0 {
                        return Vec::new();
                    }
                    geometry
                        .visible_columns(horizontal.offset(), horizontal.viewport_extent())
                        .into_iter()
                        .map(|(display, source)| CellKey {
                            row: 0,
                            display,
                            source,
                        })
                        .collect()
                },
                |key: &CellKey| *key,
                {
                    let geometry = geometry_cells;
                    move |key: CellKey| {
                        let (frozen, edge) = geometry.frozen(key.display);
                        let column = key.source;
                        CellBox {
                            key,
                            role: Role::ColumnHeader,
                            name: Some(columns[column].title.clone()),
                            theme,
                            frozen,
                            edge,
                            header: Some(HeaderCx {
                                column,
                                display: key.display,
                                sortable: columns[column].sortable,
                                frozen,
                                geometry: geometry.clone(),
                                horizontal,
                                sort,
                            }),
                            child: AnyView::new(cell_content(
                                Text::new(columns[column].title.clone()).style(style.clone()),
                            )),
                        }
                    }
                },
            )
        };
        let rows = if geometry.header_height > 0.0 { 1 } else { 0 };
        let id = cx.render(
            geometry.body(ListExtents::Fixed(geometry.header_height), rows),
            [AnyView::new(cells)],
        );
        geometry.bind_extents(cx, id);
        cx.set_paint(
            id,
            StripPaint {
                background: Color::TRANSPARENT,
                line: Color::TRANSPARENT,
                header: true,
                drop_line: None,
                accent: Color::TRANSPARENT,
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<StripPaint>(|p| {
                p.background = colors.surface;
                p.line = colors.outline_variant;
                p.accent = colors.primary;
            });
        });
        // The drop line follows the held header (WIDGETS-GRID-14).
        let drag = geometry.drag;
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || drag.get().map(|d| d.x))),
            |e, x| {
                e.update_paint::<StripPaint>(|p| p.drop_line = x);
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                let max = horizontal.max_scroll_extent().max(0.0);
                horizontal.offset().clamp(0.0, max)
            })),
            |e, x| {
                e.update_render::<RenderGridBody>(|b| b.offset = Vec2::new(x, 0.0));
            },
        );
        id
    }
}

/// The body: every row, following both offsets, holding only the visible cells.
struct Body {
    geometry: Geometry,
    columns: Rc<[ColumnSpec]>,
    source: Rc<dyn DataSource>,
    revision: Option<Signal<u64>>,
    /// Display row to source row, when filtered or client-sorted (WIDGETS-GRID-11, -12).
    map: Memo<Option<Rc<[usize]>>>,
    vertical: ScrollController,
    horizontal: ScrollController,
    theme: ThemeHandle,
}

impl View for Body {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (geometry, vertical, horizontal, theme) =
            (self.geometry, self.vertical, self.horizontal, self.theme);
        let (source, revision, columns, map) = (self.source, self.revision, self.columns, self.map);
        // The source row shown at a display row (a tracked read of the map).
        let source_row = move |row: usize| {
            map.with(|m| m.as_ref().and_then(|m| m.get(row).copied()).unwrap_or(row))
        };
        let cells = {
            let (geometry, geometry_cells) = (geometry.clone(), geometry.clone());
            For::new(
                move || {
                    let rows = geometry.row_range(
                        geometry.row_count.get(),
                        vertical.offset(),
                        vertical.viewport_extent(),
                    );
                    let cols =
                        geometry.visible_columns(horizontal.offset(), horizontal.viewport_extent());
                    rows.flat_map(|row| {
                        cols.iter().map(move |&(display, source)| CellKey {
                            row,
                            display,
                            source,
                        })
                    })
                    .collect()
                },
                |key: &CellKey| *key,
                move |key| {
                    let CellKey {
                        row,
                        display,
                        source: column,
                    } = key;
                    let (frozen, edge) = geometry_cells.frozen(display);
                    let source = Rc::clone(&source);
                    let child: AnyView = match &columns[column].builder {
                        // A built cell is rebuilt when the revision changes (WIDGETS-GRID-06).
                        Some(builder) => {
                            let builder = Rc::clone(builder);
                            AnyView::new(Dyn::new(move || {
                                if let Some(revision) = revision {
                                    revision.get();
                                }
                                let row = source_row(row);
                                cell_content(builder(row, source.cell(row, column)))
                            }))
                        }
                        // The default text re-reads its value in place.
                        None => AnyView::new(cell_content(
                            Text::new(move || {
                                if let Some(revision) = revision {
                                    revision.get();
                                }
                                source.cell(source_row(row), column).to_string()
                            })
                            .max_lines(1)
                            .soft_wrap(false),
                        )),
                    };
                    CellBox {
                        key,
                        role: Role::Cell,
                        name: None,
                        theme,
                        frozen,
                        edge,
                        header: None,
                        child,
                    }
                },
            )
        };
        let id = cx.render(
            geometry.body(ListExtents::Fixed(geometry.row_height), 0),
            [AnyView::new(cells)],
        );
        geometry.bind_extents(cx, id);
        cx.set_paint(
            id,
            StripPaint {
                background: Color::TRANSPARENT,
                line: Color::TRANSPARENT,
                header: false,
                drop_line: None,
                accent: Color::TRANSPARENT,
            },
        );
        // The row count and the offset follow the memo and the controllers.
        let row_count = geometry.row_count;
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                let (vmax, hmax) = (
                    vertical.max_scroll_extent().max(0.0),
                    horizontal.max_scroll_extent().max(0.0),
                );
                (
                    row_count.get(),
                    Vec2::new(
                        horizontal.offset().clamp(0.0, hmax),
                        vertical.offset().clamp(0.0, vmax),
                    ),
                )
            })),
            |e, (rows, offset)| {
                e.update_render::<RenderGridBody>(|b| {
                    b.row_count = rows;
                    b.offset = offset;
                });
            },
        );
        id
    }
}

/// A cell's content: padded and vertically centered at the left.
fn cell_content(child: impl View) -> impl View {
    Padding::new(EdgeInsets::symmetric(CELL_PADDING, 0.0))
        .child(Align::new(Alignment::CENTER_LEFT).child(child))
}

/// Clips a strip to its bounds; the header also paints its background, its bottom line and,
/// while a header is held, the drop line (WIDGETS-GRID-14).
struct StripPaint {
    background: Color,
    line: Color,
    header: bool,
    /// The drop line's x in the strip, while a header is held.
    drop_line: Option<f32>,
    accent: Color,
}

impl Paint for StripPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let bounds = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        if self.header {
            cx.scene().fill_rect(bounds, self.background);
        }
        cx.scene().push_clip(Clip::Rect(bounds));
        cx.paint_children();
        cx.scene().pop();
        if self.header {
            cx.scene().fill_rect(
                Rect::from_ltwh(0.0, size.height - 1.0, size.width, 1.0),
                self.line,
            );
            if let Some(x) = self.drop_line {
                cx.scene().fill_rect(
                    Rect::from_ltwh(x - DROP_LINE / 2.0, 0.0, DROP_LINE, size.height),
                    self.accent,
                );
            }
        }
    }

    fn clips_children(&self) -> bool {
        true
    }
}

/// What a header cell's pointer handler needs: sorting (WIDGETS-GRID-10, -15), resizing
/// (WIDGETS-GRID-13) and reordering (WIDGETS-GRID-14).
struct HeaderCx {
    /// The source column.
    column: usize,
    display: usize,
    sortable: bool,
    frozen: bool,
    geometry: Geometry,
    horizontal: ScrollController,
    sort: Signal<Option<SortState>>,
}

/// A header press in progress.
#[derive(Clone, Copy)]
enum Press {
    /// A resize from the handle: the pointer's x and the column's width at the press.
    Resize { x: f32, width: f32 },
    /// A press that may become a click or a reorder drag: the pointer's x at the press and
    /// whether it moved past the slop without reordering (then it is neither).
    Pending { x: f32, moved: bool },
    /// A reorder drag.
    Reorder,
}

impl HeaderCx {
    /// Cycles the sort state (WIDGETS-GRID-10): ascending, descending, none.
    fn click(&self) {
        if !self.sortable {
            return;
        }
        let column = self.column;
        let next = match self.sort.get_untracked() {
            Some(state) if state.column == column => state.ascending.then_some(SortState {
                column,
                ascending: false,
            }),
            _ => Some(SortState {
                column,
                ascending: true,
            }),
        };
        self.sort.set(next);
    }

    /// The pointer's x in the header strip from its x in this cell.
    fn strip_x(&self, local_x: f32) -> f32 {
        let geometry = &self.geometry;
        let offset = geometry.offset_untracked(self.horizontal);
        let start = geometry
            .layout
            .with_untracked(|l| l.extents.start(self.display));
        let left = if self.frozen { start } else { start - offset };
        left + local_x
    }

    /// Records the boundary nearest the pointer as the drop target (WIDGETS-GRID-14).
    fn hold(&self, local_x: f32) {
        let geometry = &self.geometry;
        let offset = geometry.offset_untracked(self.horizontal);
        let (boundary, x) = geometry.nearest_boundary(self.strip_x(local_x), offset);
        let next = Some(Reorder {
            source: self.column,
            boundary,
            x,
        });
        if geometry.drag.get_untracked() != next {
            geometry.drag.set(next);
        }
    }

    /// Moves the column to the recorded boundary and ends the drag (WIDGETS-GRID-14).
    fn drop(&self) {
        let geometry = &self.geometry;
        let Some(Reorder { boundary, .. }) = geometry.drag.get_untracked() else {
            return;
        };
        geometry.drag.set(None);
        let display = self.display;
        if boundary == display || boundary == display + 1 {
            return;
        }
        let mut order = geometry.layout.with_untracked(|l| l.order.to_vec());
        let column = order.remove(display);
        let at = if boundary > display {
            boundary - 1
        } else {
            boundary
        };
        order.insert(at, column);
        geometry.order.set(order);
    }

    /// Sets the column's width from a resize drag, floored at its minimum
    /// (WIDGETS-GRID-13).
    fn resize(&self, width: f32) {
        let geometry = &self.geometry;
        let sizing = &geometry.sizing;
        let target = width.max(sizing[self.column].min_width);
        let column = self.column;
        let current = geometry
            .widths
            .with_untracked(|w| w.get(column).copied().map(clean));
        if current == Some(target) {
            return;
        }
        geometry.widths.update(|w| {
            if w.len() < sizing.len() {
                let missing = sizing[w.len()..].iter().map(|s| s.width);
                w.extend(missing);
            }
            w[column] = target;
        });
    }
}

/// One built cell: a box carrying its `GridSlot`, its role, a clip and the grid lines
/// (WIDGETS-GRID-03, -07).
struct CellBox {
    key: CellKey,
    role: Role,
    name: Option<String>,
    theme: ThemeHandle,
    /// A frozen column's cell: opaque, painted over the scrolling ones (WIDGETS-GRID-09).
    frozen: bool,
    /// The last frozen column's cell: its right line is the theme's `outline`.
    edge: bool,
    /// A header's sorting, resizing and reordering (WIDGETS-GRID-10, -13, -14).
    header: Option<HeaderCx>,
    child: AnyView,
}

impl View for CellBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [self.child],
        );
        cx.set_parent_data(id, self.key.slot());
        cx.set_semantics(
            id,
            Semantics {
                role: self.role,
                name: self.name,
                ..Default::default()
            },
        );
        cx.set_paint(
            id,
            CellPaint {
                line: Color::TRANSPARENT,
                background: None,
                edge: None,
                chevron: None,
                chevron_color: Color::TRANSPARENT,
                held: false,
                held_color: Color::TRANSPARENT,
            },
        );
        let (frozen, edge) = (self.frozen, self.edge);
        cx.bind(id, self.theme.colors_prop(), move |e, colors| {
            e.update_paint::<CellPaint>(|p| {
                p.line = colors.outline_variant;
                p.background = frozen.then_some(colors.surface);
                p.edge = edge.then_some(colors.outline);
                p.chevron_color = colors.on_surface_variant;
                p.held_color = colors.surface_container_highest;
            });
        });
        if let Some(header) = self.header {
            let (sort, column, drag) = (header.sort, header.column, header.geometry.drag);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || {
                    sort.get()
                        .filter(|state| state.column == column)
                        .map(|state| state.ascending)
                })),
                |e, chevron| {
                    e.update_paint::<CellPaint>(|p| p.chevron = chevron);
                },
            );
            // The held header's background (WIDGETS-GRID-14).
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || {
                    drag.get().is_some_and(|d| d.source == column)
                })),
                |e, held| {
                    e.update_paint::<CellPaint>(|p| p.held = held);
                },
            );
            // One press at a time: a resize from the handle (WIDGETS-GRID-13), a reorder
            // drag (WIDGETS-GRID-14) or a click (WIDGETS-GRID-15).
            let press: Rc<Cell<Option<Press>>> = Rc::default();
            let resizable = header.geometry.sizing[column].resizable;
            let reorderable = header.geometry.reorderable;
            cx.on_pointer(id, Phase::Bubble, move |p| {
                let x = p.event.position.x;
                match (p.event.kind, press.get()) {
                    (PointerKind::Down(PointerButton::Primary), _) => {
                        let on_handle = p.local.x >= p.size.width - HANDLE_WIDTH;
                        if resizable && on_handle {
                            press.set(Some(Press::Resize {
                                x,
                                width: p.size.width,
                            }));
                        } else if reorderable || header.sortable {
                            press.set(Some(Press::Pending { x, moved: false }));
                        } else {
                            return Handled::Continue;
                        }
                        Handled::Stop
                    }
                    (PointerKind::Move, Some(Press::Resize { x: start, width })) => {
                        header.resize(width + (x - start));
                        Handled::Stop
                    }
                    (PointerKind::Move, Some(Press::Pending { x: start, moved })) => {
                        if (x - start).abs() > DRAG_SLOP {
                            if reorderable {
                                press.set(Some(Press::Reorder));
                                header.hold(p.local.x);
                            } else if !moved {
                                press.set(Some(Press::Pending {
                                    x: start,
                                    moved: true,
                                }));
                            }
                        }
                        Handled::Stop
                    }
                    (PointerKind::Move, Some(Press::Reorder)) => {
                        header.hold(p.local.x);
                        Handled::Stop
                    }
                    (PointerKind::Up(PointerButton::Primary), Some(current)) => {
                        press.set(None);
                        match current {
                            Press::Pending { moved: false, .. } => header.click(),
                            Press::Reorder => header.drop(),
                            Press::Pending { .. } | Press::Resize { .. } => {}
                        }
                        Handled::Stop
                    }
                    _ => Handled::Continue,
                }
            });
        }
        id
    }
}

/// Clips the content to the cell and draws its bottom and right lines; a frozen cell fills
/// its background first and a frozen edge uses the outline color (WIDGETS-GRID-09).
struct CellPaint {
    line: Color,
    background: Option<Color>,
    edge: Option<Color>,
    /// A header's sort chevron: `Some(true)` points up (ascending) (WIDGETS-GRID-10).
    chevron: Option<bool>,
    chevron_color: Color,
    /// A header held for reordering is highlighted (WIDGETS-GRID-14).
    held: bool,
    held_color: Color,
}

/// One bar of the chevron: a rect of `CHEVRON_STROKE` from `from` to `to` in a transform
/// scope that rotates it into place (as the checkbox's check mark).
fn chevron_bar(cx: &mut PaintCx<'_, '_>, from: Point, to: Point, color: Color) {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let mid = Vec2::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
    let half = dx.hypot(dy) / 2.0;
    let scene = cx.scene();
    scene.push_transform(Affine::translate(mid) * Affine::rotate(dy.atan2(dx)));
    scene.fill_rect(
        Rect::from_ltrb(-half, -CHEVRON_STROKE / 2.0, half, CHEVRON_STROKE / 2.0),
        color,
    );
    scene.pop();
}

impl Paint for CellPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let bounds = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        if let Some(background) = self.background {
            cx.scene().fill_rect(bounds, background);
        }
        if self.held {
            cx.scene().fill_rect(bounds, self.held_color);
        }
        cx.scene().push_clip(Clip::Rect(bounds));
        cx.paint_children();
        cx.scene().pop();
        cx.scene().fill_rect(
            Rect::from_ltwh(0.0, size.height - 1.0, size.width, 1.0),
            self.line,
        );
        cx.scene().fill_rect(
            Rect::from_ltwh(size.width - 1.0, 0.0, 1.0, size.height),
            self.edge.unwrap_or(self.line),
        );
        if let Some(ascending) = self.chevron {
            let (x, y) = (
                size.width - CELL_PADDING - CHEVRON_WIDTH / 2.0,
                size.height / 2.0,
            );
            let (tip, base) = if ascending { (-2.5, 2.5) } else { (2.5, -2.5) };
            let tip = Point::new(x, y + tip);
            let half = CHEVRON_WIDTH / 2.0;
            chevron_bar(cx, Point::new(x - half, y + base), tip, self.chevron_color);
            chevron_bar(cx, tip, Point::new(x + half, y + base), self.chevron_color);
        }
    }

    fn clips_children(&self) -> bool {
        true
    }
}
