//! [`DataGrid`], [`DataSource`], [`CellValue`], [`FnSource`] and [`GridColumn`]. Spec:
//! `docs/specs/widgets/data-grid.md`.

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
/// The header's bold weight.
const HEADER_WEIGHT: f32 = 700.0;
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
}

impl GridColumn {
    /// A column titled `title`, 120 px wide, showing values as text.
    pub fn new(title: impl Into<String>) -> Self {
        GridColumn {
            title: title.into(),
            width: COLUMN_WIDTH,
            builder: None,
            sortable: None,
        }
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
    frozen: usize,
    sort: Option<Signal<Option<SortState>>>,
    client_sort: bool,
    filter: Option<Signal<Option<RowFilter>>>,
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
            frozen: 0,
            sort: None,
            client_sort: false,
            filter: None,
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

/// The grid's geometry, shared by the strips, the handlers and the paint.
#[derive(Clone)]
struct Geometry {
    row_height: f32,
    header_height: f32,
    /// The row count, re-read when the revision changes (WIDGETS-GRID-06).
    row_count: Memo<usize>,
    column_count: usize,
    columns: ListExtents,
    /// The leading columns pinned at the left (WIDGETS-GRID-09), at most `column_count`.
    frozen_columns: usize,
}

impl Geometry {
    /// The body's content size, from the current row count (an untracked read).
    fn content_untracked(&self) -> Size {
        Size::new(
            self.columns.total(self.column_count),
            self.row_height * self.row_count.get_untracked() as f32,
        )
    }

    /// The columns to build: the scrolling ones between the frozen width and `viewport`
    /// (one of buffer each side, WIDGETS-GRID-02), then the frozen ones, which paint last
    /// and so on top (WIDGETS-GRID-09).
    fn visible_columns(&self, offset: f32, viewport: f32) -> Vec<usize> {
        let frozen = self.frozen_columns;
        let frozen_width = self.columns.start(frozen);
        let scrolling = if viewport > frozen_width {
            range(
                &self.columns,
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
            .collect()
    }

    /// The grid body for the strip: `rows` rows with the grid's columns and frozen columns.
    fn body(&self, rows: ListExtents, row_count: usize) -> RenderGridBody {
        let mut body =
            RenderGridBody::new(rows, row_count, self.columns.clone(), self.column_count);
        body.frozen_columns = self.frozen_columns;
        body
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
        let geometry = Geometry {
            row_height: clean(self.row_height),
            header_height: clean(self.header_height),
            row_count,
            column_count: self.columns.len(),
            columns: ListExtents::table(self.columns.iter().map(|c| c.width)),
            frozen_columns: self.frozen.min(self.columns.len()),
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
                        .map(|column| GridSlot { row: 0, column })
                        .collect()
                },
                |slot: &GridSlot| *slot,
                {
                    let geometry = geometry_cells;
                    move |slot: GridSlot| {
                        let (frozen, edge) = geometry.frozen(slot.column);
                        let column = slot.column;
                        // A press on a sortable header cycles the sort state
                        // (WIDGETS-GRID-10): ascending, descending, none.
                        let on_press: Option<Rc<dyn Fn()>> = columns[column].sortable.then(|| {
                            Rc::new(move || {
                                let next = match sort.get_untracked() {
                                    Some(state) if state.column == column => {
                                        state.ascending.then_some(SortState {
                                            column,
                                            ascending: false,
                                        })
                                    }
                                    _ => Some(SortState {
                                        column,
                                        ascending: true,
                                    }),
                                };
                                sort.set(next);
                            }) as Rc<dyn Fn()>
                        });
                        CellBox {
                            slot,
                            role: Role::ColumnHeader,
                            name: Some(columns[column].title.clone()),
                            theme,
                            frozen,
                            edge,
                            on_press,
                            chevron: Some((sort, column)),
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
        cx.set_paint(
            id,
            StripPaint {
                background: Color::TRANSPARENT,
                line: Color::TRANSPARENT,
                header: true,
            },
        );
        cx.bind(id, theme.colors_prop(), |e, colors| {
            e.update_paint::<StripPaint>(|p| {
                p.background = colors.surface;
                p.line = colors.outline_variant;
            });
        });
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
                    rows.flat_map(|row| cols.iter().map(move |&column| GridSlot { row, column }))
                        .collect()
                },
                |slot: &GridSlot| *slot,
                move |slot| {
                    let GridSlot { row, column } = slot;
                    let (frozen, edge) = geometry_cells.frozen(column);
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
                        slot,
                        role: Role::Cell,
                        name: None,
                        theme,
                        frozen,
                        edge,
                        on_press: None,
                        chevron: None,
                        child,
                    }
                },
            )
        };
        let id = cx.render(
            geometry.body(ListExtents::Fixed(geometry.row_height), 0),
            [AnyView::new(cells)],
        );
        cx.set_paint(
            id,
            StripPaint {
                background: Color::TRANSPARENT,
                line: Color::TRANSPARENT,
                header: false,
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

/// Clips a strip to its bounds; the header also paints its background and bottom line.
struct StripPaint {
    background: Color,
    line: Color,
    header: bool,
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
        }
    }

    fn clips_children(&self) -> bool {
        true
    }
}

/// One built cell: a box carrying its `GridSlot`, its role, a clip and the grid lines
/// (WIDGETS-GRID-03, -07).
struct CellBox {
    slot: GridSlot,
    role: Role,
    name: Option<String>,
    theme: ThemeHandle,
    /// A frozen column's cell: opaque, painted over the scrolling ones (WIDGETS-GRID-09).
    frozen: bool,
    /// The last frozen column's cell: its right line is the theme's `outline`.
    edge: bool,
    /// A header's press handler (sorting, WIDGETS-GRID-10).
    on_press: Option<Rc<dyn Fn()>>,
    /// A header's sort state and column, for its chevron.
    chevron: Option<(Signal<Option<SortState>>, usize)>,
    child: AnyView,
}

impl View for CellBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [self.child],
        );
        cx.set_parent_data(id, self.slot);
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
            },
        );
        let (frozen, edge) = (self.frozen, self.edge);
        cx.bind(id, self.theme.colors_prop(), move |e, colors| {
            e.update_paint::<CellPaint>(|p| {
                p.line = colors.outline_variant;
                p.background = frozen.then_some(colors.surface);
                p.edge = edge.then_some(colors.outline);
                p.chevron_color = colors.on_surface_variant;
            });
        });
        if let Some((sort, column)) = self.chevron {
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
        }
        if let Some(press) = self.on_press {
            cx.on_pointer(id, Phase::Bubble, move |p| {
                if p.event.kind == PointerKind::Down(PointerButton::Primary) {
                    press();
                    Handled::Stop
                } else {
                    Handled::Continue
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
