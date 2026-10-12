# DataGrid: data source, columns, header, virtualization and frozen columns

- **Status:** Implemented (the user said "continue" on the draft, 2026-10-12, taking its nine
  proposals); **part 2, frozen columns (WIDGETS-GRID-09), Implemented** on 2026-10-12 (decided by
  the agent on the user's "continue and then commit and push"; proposals 10 to 12 are marked
  for review); part 3, sorting and filtering, is [its own spec](data-grid-sort.md)
- **Crates:** `tantu-widgets` (`DataGrid`, `GridColumn`, `DataSource`, `CellValue`,
  `FnSource`), `tantu-view` and `tantu-a11y` (three roles), `examples/gallery`
- **Plan item:** Phase 4, "`DataGrid`" → "data source, columns, header, row and column
  virtualization, scrolling on both axes"
- **Related:** [grid body layout](../layout/grid.md), [ListView](list-view.md) and
  [its extents](list-view-advanced.md), [scroll](scroll.md) (controllers, wheel, scrollbars,
  keys), [semantics](../a11y/semantics.md), [gallery](../examples/gallery.md)

## Purpose

The enterprise widget: a table over a data source with a million rows and dozens of columns,
scrolling smoothly on both axes, building only the cells on screen. This first part delivers
the skeleton every later part extends: the data source contract, column definitions, a
pinned header, two-dimensional virtualization and scrolling (wheel, scrollbars, keys), a
change notification, roles for assistive technology, a gallery section and a scrolling
benchmark against the 0.1 target (60 fps on a 1M-row grid).

Later parts (PLAN.md): frozen columns; sorting and filtering; column resize and reorder;
selection models with keyboard navigation; cell editors; the stress example.

## Scope

In scope:

- `CellValue`, `DataSource`, `FnSource`.
- `GridColumn` (title, width, optional cell builder).
- `DataGrid`: structure, sizing, header, virtualization, default cells, wheel and scrollbar
  scrolling on both axes, provisional keyboard scrolling, `revision`, controllers.
- Roles `Grid`, `ColumnHeader`, `Cell` in `tantu-view` and their AccessKit mapping.
- The gallery's Data grid section and the `GridScroll` benchmark.

Out of scope (and where it goes):

- Frozen columns in the widget (the layout object already places them): the next part.
- Sorting, filtering, resize, reorder, selection, editing: their parts.
- Row and column indices, selection state and cell values in the accessibility tree: with
  the selection part.
- Variable row heights, auto-fit column widths, row and column spans: later, if needed.
- Asynchronous or paged sources: the app's job over `revision` (see Open questions).

## Public API

```rust
// tantu-widgets
/// A cell's value.
#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    Empty,
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// `Empty` is "", `Text` its text, `Int` and `Float` the number (`{}`), `Bool` "true"/"false".
impl fmt::Display for CellValue { /* ... */ }

/// Where a grid's rows come from. Indices are 0-based; the grid asks only for the cells it
/// shows, each time it builds or refreshes them.
pub trait DataSource: 'static {
    /// How many rows there are.
    fn row_count(&self) -> usize;
    /// The value of a cell; `Empty` for one that doesn't exist (yet).
    fn cell(&self, row: usize, column: usize) -> CellValue;
}

/// A source computed from a closure, for generated or lazily loaded data.
pub struct FnSource { /* ... */ }

impl FnSource {
    /// `rows` rows whose cells `cell(row, column)` computes.
    pub fn new(rows: usize, cell: impl Fn(usize, usize) -> CellValue + 'static) -> Self;
}

impl DataSource for FnSource { /* ... */ }

/// A column of a `DataGrid`.
pub struct GridColumn { /* title, width, builder */ }

impl GridColumn {
    /// A column titled `title`, 120 px wide, showing values as text.
    pub fn new(title: impl Into<String>) -> Self;
    /// The column's width in logical pixels (negative or NaN counts as 0).
    pub fn width(self, width: f32) -> Self;
    /// Builds the cell's content from the row and its value instead of the default text.
    pub fn builder<V: View>(self, build: impl Fn(usize, CellValue) -> V + 'static) -> Self;
}

/// A table over a `DataSource` that builds only its visible cells and scrolls on both axes
/// (the first part of the enterprise data grid).
pub struct DataGrid { /* ... */ }

impl DataGrid {
    /// A grid with `columns` over `source`.
    pub fn new(columns: impl IntoIterator<Item = GridColumn>, source: Rc<dyn DataSource>) -> Self;
    /// Every row's height (default 32; negative or NaN counts as 0).
    pub fn row_height(self, height: f32) -> Self;
    /// The header row's height (default 36; 0 hides the header).
    pub fn header_height(self, height: f32) -> Self;
    /// A signal the app bumps when the source's data changed: the row count and the shown
    /// cells are re-read (WIDGETS-GRID-06).
    pub fn revision(self, revision: Signal<u64>) -> Self;
    /// Shares the vertical and horizontal scroll controllers with the app.
    pub fn controllers(self, vertical: ScrollController, horizontal: ScrollController) -> Self;
    /// How long a wheel step takes (default 150 ms; zero applies deltas at once), as
    /// `ScrollView::wheel_duration`.
    pub fn wheel_duration(self, duration: Duration) -> Self;
    /// The first `count` columns stay at the left while the others scroll (clamped to the
    /// column count; default 0) (WIDGETS-GRID-09).
    pub fn frozen_columns(self, count: usize) -> Self;
}

// tantu-view
pub enum Role { /* ... */ Grid, ColumnHeader, Cell }
```

Structure: one outer render element (a column flex) holding the focus, the key, wheel and
scrollbar handlers, the `Grid` semantics and a paint that draws the two scrollbars after its
children; under it a header strip and a body, each a `RenderGridBody` (the header with one row
of `header_height`, offset `(x, 0)`; the body with every row, offset `(x, y)`, taking the rest
of the height) whose paint clips to its bounds; under each a `For` over the visible slots,
keyed by `(row, column)`, building one cell wrapper per slot (a box with `GridSlot` parent
data that clips its content) around the cell's content.

Default cell content: the value's `Display` text in the body text style, vertically centered,
with 8 px horizontal padding, on one line (no wrapping; overflow is clipped by the wrapper).
Header cells: the title in the label text style, bold, the same padding, over a `surface`
background with a 1 px `outline_variant` line below the strip. Body rows get a 1 px
`outline_variant` line below each row and between columns (drawn by the cell wrappers).

## Behavior

### `tantu-widgets`

- **WIDGETS-GRID-01:** Structure and sizing: a grid of three 120 px columns over 1 000 rows
  in a 600 × 400 window fills it; the header strip is 600 × 36 at the top and the body
  600 × 364 below it; `Finder::text("Name")` finds the header cell of the column titled
  "Name", at x = 120 for the second column; with `header_height(0)` there is no header and the
  body starts at y = 0. The default `Display` texts are as the API says.
- **WIDGETS-GRID-02:** Virtualization: the body builds the rows from the one containing
  `offset.y` to the one containing the body's bottom edge (the row starting exactly there
  excluded, as WIDGETS-LIST-02) with one row of buffer each side, and the columns likewise
  from `offset.x` to the right edge with one column of buffer each side; the header builds the
  same columns. A 1 000 000 × 20 grid (32 px rows, 120 px columns) in 600 × 400 therefore
  builds at most (12 + 2) × (5 + 2) body cells and 7 header cells, and the source's `cell` is
  called only for them. Cells keep their elements while they stay in the range (keyed by
  `(row, column)`); scrolling by one row removes one row of cells and builds one.
- **WIDGETS-GRID-03:** A column's `builder` replaces the default text for every body cell of
  that column (not the header); it is called with the row index and the value each time the
  cell is built. A cell's content wider than the column is clipped to the cell.
- **WIDGETS-GRID-04:** Scrolling: a wheel delta's `y` scrolls the rows and its `x` the
  columns (each clamped to its extents: rows total minus the body height, columns total minus
  the width; both at least 0), with the wheel animation of WIDGETS-SCROLL-07; the header
  follows the columns and never the rows; the vertical controller's extents are the body's
  (not the whole grid's); a vertical scrollbar at the body's right edge and a horizontal one
  at the grid's bottom edge are drawn and dragged as WIDGETS-SCROLL-04, each only when its
  axis overflows; the event is stopped when either axis moved, else continues.
- **WIDGETS-GRID-05:** Keyboard (provisional until the selection part): the grid is focusable
  as a scroll view (a press focuses it, Tab doesn't stop at it); while focused, ArrowDown/Up
  scroll by one row height, ArrowRight/Left by 40 px, PageDown/PageUp by the body height,
  End/Home to the last/first row, each clamped and handled only when it moved.
- **WIDGETS-GRID-06:** `revision(signal)`: when the signal changes, the row count is re-read
  (the vertical extents and scrollbar follow at the next frame) and every built default cell
  re-reads `cell()` and shows the new text without rebuilding its element; cells built by a
  column `builder` are rebuilt. Without a revision signal the source is read when a cell is
  built and never again.
- **WIDGETS-GRID-07:** Semantics (WIDGETS-SEM-02 extended): the grid's node has role `Grid`,
  each header cell `ColumnHeader` with the title as its label, each body cell `Cell`; the
  default text inside a cell stays a `Label` child. `tantu-view` gains the three roles and
  `tantu-a11y` maps them to AccessKit's `Grid`, `ColumnHeader` and `Cell`.
- **WIDGETS-GRID-08:** Benchmark: `benches/grid.rs` in `tantu-widgets` (criterion, like the
  reactive and layout benches) has a `GridScroll` scenario: a 1 000 000 × 20 `FnSource` grid
  in a 1280 × 720 `WidgetTester` with a zero wheel duration, one step being a wheel scroll of
  one row height (a `tester.scroll` and its frame). The scenario is also a test: after a step
  the first visible row advanced by one and the built cell count is within WIDGETS-GRID-02's
  bound for that window.
- **WIDGETS-GRID-09:** Frozen columns (part 2): with `frozen_columns(n)`, the first `n`
  columns (clamped to the column count) are placed at their own starts whatever the
  horizontal offset (LAYOUT-GRID-04), in the header and the body, and are always built; the
  scrolling columns visible are those between the frozen width and the right edge (one
  column of buffer each side, never a frozen one), so a 20-column grid with two frozen 120 px
  columns in a 600 px window scrolled by 240 px builds columns 0, 1 and 3 to 7 and shows
  column 4 at x = 240. Frozen cells are painted after the scrolling cells and over an opaque
  `surface` background, so scrolling content passes under them, and the last frozen column's
  right line is the theme's `outline` (the others stay `outline_variant`). The horizontal
  extents are unchanged (the content width minus the grid width). `frozen_columns(0)` is the
  part 1 behavior.
- **GALLERY-10:** A "Data grid" section (key `grid`): a 1 000 000-row `FnSource` with six
  columns (Id, Name, City, Amount, Active, Score), the Id column frozen, a "First row: N"
  text (key `first-row`) following the vertical controller, and a golden
  `tests/goldens/gallery-grid.png`; a wheel scroll of 320 px changes the text to "First row:
  10", and a horizontal wheel scroll of 200 px leaves the "Id" header where it was while
  "Name" moves left.

## Performance and allocation

Per frame while scrolling: one `For` reconciliation over at most (visible rows + 2) ×
(visible columns + 2) keys, with cells entering and leaving at the edges only; layout of the
built cells (LAYOUT-GRID-02, two lookups each); one text layout per new cell (cached by text
and style, so repeated values are hits). Target, from PLAN.md's 0.1 criteria: a `GridScroll`
step under 8 ms (criterion median, release build, the reference laptop), leaving the rest
of a 16.7 ms frame to the renderer. **Met.** Baseline (criterion median per `step()`,
`cargo bench -p tantu-widgets`, Rust 1.88.0, Linux, Intel Core i5-1235U laptop, 2026-10-12):

| Scenario | Built cells | Median per step |
|---|---|---|
| `grid_scroll_1m` (1 000 000 × 20, 1280 × 720, one-row wheel step) | 288 | 1.30 ms |

Of that, the frame (apply, layout, paint) is about 1.4 ms in a debug-instrumented run and the
dispatch about 20 µs; swapping 20 unique-text cells costs about 0.5 ms. Note for benchmark
authors: `WidgetTester::scroll` looks the element up by key on every call, and that walk over
every element took about 30 ms here, so the scenario dispatches the wheel at a position looked
up once (the finder's cost is an open question in HANDOFF.md). When a number regresses by
more than about 20 % on the same machine, find out why before moving on.

## Open questions

Proposals (decided by the agent for the draft; say "continue" to accept, or change them):

1. **A synchronous, pull-based `DataSource`** (`row_count`, `cell(row, column)`) with a
   `revision` signal for changes, rather than a push or async API. A paged or remote source
   returns `Empty` (or a placeholder) for rows it hasn't loaded, loads them in the
   background, and bumps the revision; the grid never blocks. The `cell` call is per visible
   cell per build, so sources should be cheap or cache (the trait doc says so).
2. **Cells are built per cell, keyed by `(row, column)`,** in one `For`, rather than a `For`
   of rows each holding a `For` of columns: one reconciliation per frame, and a horizontal
   scroll keeps a row's other cells. The cost is a wrapper element per cell (a clipping box
   with parent data), which the bound in WIDGETS-GRID-02 keeps at about a hundred.
3. **The grid composes two `RenderGridBody` instances and paints its own scrollbars** rather
   than nesting `ScrollView`s: the header must follow one axis only, frozen columns the
   other, and two nested viewports can't express that. The scrollbar geometry and dragging
   reuse the scroll view's code (moved to a shared helper in the crate).
4. **Fixed row height and fixed column widths** for this part. Enterprise grids use fixed
   row heights for exactly this virtualization; auto-fit widths can come with resize.
5. **Wheel `x` scrolls columns directly;** no Shift+wheel remapping. Trackpads and tilt
   wheels deliver `x` deltas through the platform already; a Shift convention can be added
   in the runner later for mouse-only users.
6. **Keyboard scrolling is provisional:** arrows scroll until the selection part makes them
   move a cell cursor. The rule is marked so and will be superseded, not renumbered.
7. **Roles only for assistive technology in this part** (`Grid`, `ColumnHeader`, `Cell`);
   row and column indices join the `Semantics` struct with selection, where they matter.
8. **The benchmark uses `WidgetTester`** from `tantu-widgets`' benches (criterion as a
   dev-dependency, `autobenches = false`, the scenario shared with a test as the other
   benches), since the whole frame (build, layout, text, paint) is what the 60 fps target
   measures, not layout alone. Numbers are compared by hand, no CI assertion.
9. **The default cell text is one line, clipped,** no ellipsis: an ellipsis needs text
   measuring per cell (a later `Text::overflow` feature); clipping is free.

Part 2, frozen columns (decided by the agent, 2026-10-12; review):

10. **Paint order by element order, not z-index:** the visible-slot list puts the frozen
    cells after the scrolling ones, so the `For` orders them last and they paint on top.
    A z-index set inside a cell's paint would not reorder its transform scope among its
    siblings (the scope's z-index is the one in force when it is pushed), and setting it
    from the strip would need per-child painting the paint API doesn't have.
11. **Frozen cells are opaque** (a `surface` fill) rather than the strip clipping the
    scrolling cells to the region right of the frozen width, which `paint_children` can't
    do per child either. The last frozen column's `outline` line marks the edge.
12. **The frozen width counts against the horizontal extents as before** (max offset =
    content width − grid width): the scrolling region is narrower by the frozen width, and so
    is the scrolling content, so the arithmetic is unchanged and `jump_to`/`animate_to`
    targets keep their meaning.
