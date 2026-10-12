# DataGrid: sorting and filtering

- **Status:** Implemented (the user said "continue" on the draft, 2026-10-12, taking its seven
  proposals)
- **Crates:** `tantu-widgets` (`DataGrid`, `GridColumn`, `SortState`, `RowFilter`,
  `CellValue::compare`), `examples/gallery`
- **Plan item:** Phase 4, "`DataGrid`" → "sorting and filtering"
- **Related:** [DataGrid](data-grid.md) (parts 1 and 2, which this amends),
  [gallery](../examples/gallery.md)

## Purpose

Part 3 of the data grid: a header click sorts, the sorted column shows which way, the app
learns the sort through a shared signal and reorders its source (the database case), or asks
the grid to order the rows itself (the in-memory case); a filter predicate hides rows. The
`DataSource` trait stays a plain row store.

## Scope

In scope: the sort state and header clicks, the chevron, `client_sort` ordering by
`CellValue::compare`, the row filter, the row map behind both, the gallery's sort echo.

Out of scope (and where it goes): per-column filter rows and menus (UI over this filter,
later); locale-aware collation (Phase 4 i18n, as a comparator hook); sort direction in the
accessibility tree (with the selection part, which adds row and column indices); multi-column
sorts (later, if a grid needs them).

## Public API

```rust
// tantu-widgets
impl DataGrid {
    /// Shares the sort state with the app (the grid owns one otherwise) (WIDGETS-GRID-10).
    pub fn sort_state(self, state: Signal<Option<SortState>>) -> Self;
    /// The grid orders the rows itself by the sorted column's values, and every column is
    /// sortable unless it says otherwise (default false: the app reorders its source)
    /// (WIDGETS-GRID-11).
    pub fn client_sort(self, enabled: bool) -> Self;
    /// Shows only the source rows the predicate accepts (`None`: every row) (WIDGETS-GRID-12).
    pub fn filter(self, filter: Signal<Option<RowFilter>>) -> Self;
}

impl GridColumn {
    /// Whether a header click sorts by this column (default: only with `client_sort`)
    /// (WIDGETS-GRID-10).
    pub fn sortable(self, sortable: bool) -> Self;
}

/// Which column the grid is sorted by, and how (Flutter's `sortColumnIndex` and
/// `sortAscending`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortState {
    pub column: usize,
    pub ascending: bool,
}

/// A predicate over a source's rows.
pub type RowFilter = Rc<dyn Fn(&dyn DataSource, usize) -> bool>;

impl CellValue {
    /// A total order over values for sorting: `Empty`, then `Bool` (false first), then
    /// numbers (`Int` and `Float` compared as `f64`, NaN lowest), then `Text` (byte order)
    /// (WIDGETS-GRID-11).
    pub fn compare(&self, other: &CellValue) -> Ordering;

```

## Behavior

- **WIDGETS-GRID-10:** Sort state (part 3): the grid holds a `Signal<Option<SortState>>`
  (its own, or the app's through `sort_state`). A primary press on a sortable column's
  header (since part 4 a *click*, a press and release without a drag: WIDGETS-GRID-15 in
  [column resize and reorder](data-grid-columns.md)) cycles the state: an unsorted or differently sorted column becomes ascending, an
  ascending one descending, a descending one unsorted (`None`); presses on a column that
  isn't sortable change nothing; the press doesn't bubble past the grid when it sorted.
  The sorted column's header shows a chevron after its title (a 10 px mark of two 2 px bars
  in `on_surface_variant`, pointing up for ascending and down for descending), and the other
  headers show none; setting the signal from the app moves the chevron the same way. The
  state alone reorders nothing: without `client_sort` the app reorders its source (and bumps
  `revision`), as a database-backed source would.
- **WIDGETS-GRID-11:** Client-side sorting: with `client_sort(true)`, a sort state orders the
  displayed rows by `CellValue::compare` of the sorted column's values (a stable sort, so
  equal values keep their source order; descending reverses the comparison, not the stable
  order), recomputed when the state, the filter or the revision changes and never while
  scrolling; `None` shows the source order. Display row `d` shows source row `map[d]`: the
  default text reads `cell(map[d], column)`, a column `builder` is called with the source
  row, and the header stays. `compare` orders `Empty` < `Bool(false)` < `Bool(true)` <
  numbers (`Int(2)` < `Float(2.5)`, NaN lowest) < `Text` (byte order, so "B" < "a").
- **WIDGETS-GRID-12:** Filtering: with `filter(signal)` holding `Some(predicate)`, the grid
  shows only the source rows the predicate accepts (called with the source and the row, in
  source order, once per row when the filter or the revision changes), then sorts them when
  `client_sort` applies; the row count, the vertical extents and the scrollbar follow the
  matches; `None` shows every row. The map of matches is shared, not copied, between the
  frames that reuse it.
- **GALLERY-11:** The Data grid section's columns are sortable and its sort state is echoed
  as "Sort: none" / "Sort: City ascending" / "Sort: City descending" (key `sort`); a tap on
  the "City" header cycles it (the Score column sits mostly past the grid's right edge). The gallery does not enable `client_sort` (a million
  generated rows would be compared in a debug test); the grid stays in source order.

## Performance and allocation

The row map is an `Rc<[usize]>` built once per change of the sort state, the filter or the
revision (8 bytes per shown row, 8 MB for a million), shared by the frames that reuse it;
nothing is rebuilt while scrolling. A client sort costs n log n `CellValue::compare` calls,
each two `cell` reads, so it suits sources whose `cell` is cheap; a filter costs one predicate
call per source row. Without a filter and without `client_sort` there is no map at all.

## Open questions

Resolved (2026-10-12; the user said "continue" on the draft, taking these proposals):

1. **The sort state is a shared signal the grid cycles; ordering is the app's job unless
    `client_sort` is on.** Enterprise sources sort in the database, so PLAN.md's "the
    source's sort and filter hooks" become: the app reads the state (or is told through the
    signal) and reorders its source, then bumps `revision`. `DataSource` stays a plain row
    store with no new methods. The in-memory case gets `client_sort`.
2. **Client sorting compares `CellValue`s with a fixed cross-kind order and byte-order
    text**, deterministic and locale-free; locale-aware collation comes with Phase 4 i18n
    as a comparator hook on the column. A sort over a million rows costs n log n `cell`
    calls (about 20 million), so the doc says it is for sources whose `cell` is cheap.
3. **The filter is a predicate signal over source rows** (`Rc<dyn Fn(&dyn DataSource,
    usize) -> bool>`), applied before sorting, rather than per-column filter models: those
    are UI (filter rows under the header, filter menus) and can be built on this later.
4. **Builders and the source see source rows, not display rows.** A builder that looks its
    row up in app data needs the source index; the display index is a presentation detail.
    The selection part will have to decide which index its signal carries (display, since
    keys move through display rows, with a helper to map to source rows).
5. **The chevron is painted** (two bars, like the checkbox's check mark), not an icon-font
    glyph, so it needs no font and goldens stay deterministic.
6. **Columns are sortable only when asked** (`sortable(true)`) or when `client_sort` is
    on: a header click on a database-backed grid should do nothing until the app wires the
    state, rather than show a chevron over an unsorted table.
7. **The gallery echoes the state only;** `client_sort` over its million generated rows
    would take seconds in a debug test.
