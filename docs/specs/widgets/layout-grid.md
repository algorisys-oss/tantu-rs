# `LayoutGrid`: a two-dimensional track layout

- **Status:** Implemented (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-layout` (`RenderLayoutGrid`, `TrackSize`, `GridPlacementData`),
  `tantu-widgets` (`LayoutGrid`, `GridPlacement`), `examples/gallery`
- **Plan item:** Phase 4, "`Grid` layout (possibly via taffy) for dashboard-style screens"
- **Related:** [flex](../layout/flex.md), [layout tree](../layout/tree.md),
  [layout widgets](layout.md), [directionality](directionality.md),
  [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Dashboards and settings screens place tiles and fields on rows and columns that line up
across the whole screen: a label column sized to its widest label, a value column taking
the rest, a chart tile spanning two columns. Nesting `Row`s and `Column`s can't align
across siblings. CSS Grid solves this with tracks; Flutter apps use the
`flutter_layout_grid` package (`LayoutGrid`, `columnSizes: [auto, 1.fr, 100.px]`,
`GridPlacement`). Tantu adds the same widget, implemented in `tantu-layout` on the
existing constraint protocol.

## Scope

In scope: column and row tracks sized fixed (`px`), flexible (`fr`) or to content
(`auto`); gaps; explicit placement with spans and automatic row-major placement; implicit
auto rows; right-to-left mirroring; intrinsic sizes; a gallery card.

Out of scope: named areas and lines; `minmax()`, percentage tracks and content-based
sizing of spanning items (spanning items take the sizes the single-track items give);
dense packing; column-major auto flow; baseline alignment; per-item alignment inside a
cell (items fill their cells; wrap them in `Align`).

## Public API

```rust
// tantu-layout
/// How a column or row is sized.
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
    pub const fn px(extent: f32) -> Self;
    pub const fn fr(factor: f32) -> Self;
    pub const AUTO: TrackSize;
}

/// Where an item sits (parent data of a `RenderLayoutGrid` child).
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
/// `flutter_layout_grid`).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderLayoutGrid {
    pub columns: Vec<TrackSize>,
    pub rows: Vec<TrackSize>,
    pub column_gap: f32,
    pub row_gap: f32,
    pub text_direction: TextDirection,
}
impl RenderLayoutGrid {
    pub fn new(columns: Vec<TrackSize>, rows: Vec<TrackSize>) -> Self;
}

// tantu-widgets
/// Lays children out on tracks (`flutter_layout_grid`'s `LayoutGrid`).
pub struct LayoutGrid { /* private */ }
impl LayoutGrid {
    pub fn new(columns: impl IntoIterator<Item = TrackSize>) -> Self;
    /// Row tracks (default none: every row is an implicit auto row).
    pub fn rows(self, rows: impl IntoIterator<Item = TrackSize>) -> Self;
    pub fn column_gap(self, gap: f32) -> Self;
    pub fn row_gap(self, gap: f32) -> Self;
    pub fn child(self, child: impl View) -> Self;
    pub fn children<V: View>(self, children: impl IntoIterator<Item = V>) -> Self;
}
/// Places its child on a `LayoutGrid` (`flutter_layout_grid`'s `GridPlacement`).
pub struct GridPlacement { /* private */ }
impl GridPlacement {
    pub fn new(child: impl View) -> Self;
    pub fn column(self, column: usize) -> Self;
    pub fn row(self, row: usize) -> Self;
    pub fn column_span(self, span: usize) -> Self;
    pub fn row_span(self, span: usize) -> Self;
}
```

## Behavior

- **LAYOUT-LGRID-01:** `TrackSize::px` and `fr` build `Fixed` and `Flex`; a negative or NaN
  extent or factor counts as 0. A grid with no columns behaves as one with a single `Flex(1)`
  column.
- **LAYOUT-LGRID-02:** Placement: spans of 0 count as 1 and a column span is clamped to
  the column count; an item with a column too far right moves left until its span fits.
  Items with both a column and a row are placed first, in child order; the others are
  placed in child order, row-major, each in the first position (scanning rows from 0, then
  columns, from its given column or row if it has one) where its span covers no occupied
  cell. Rows past the defined `rows` are implicit `Auto` rows.
- **LAYOUT-LGRID-03:** Columns: `Fixed` columns take their extent; `Auto` columns the
  largest maximum intrinsic width of the items spanning only that column (0 without);
  `Flex` columns share the free width (the maximum width minus fixed and auto columns and
  the gaps, at least 0) by factor. With an unbounded maximum width, `Flex` columns size as
  `Auto`.
- **LAYOUT-LGRID-04:** Rows and children: every child gets a tight width of the columns it
  spans plus the gaps between them. A child spanning only `Fixed` and `Flex` rows gets a
  tight height of those rows once they are sized; other children are laid out with an
  unbounded height first. `Auto` rows take the largest height of the items spanning only
  that row; `Flex` rows share the free height (bounded maximum height minus the others and
  the gaps) by factor, or size as `Auto` when the height is unbounded. Then the children
  laid out with their own height are laid out again with a tight height of their rows, so
  every item fills its cells (CSS's default stretch; found with the gallery, review).
- **LAYOUT-LGRID-05:** Size and offsets: the grid takes the sum of its tracks and gaps,
  constrained; each child sits at the start of its first cell. With `text_direction: Rtl`,
  columns run from the right (the offsets mirrored within the grid's width).
- **LAYOUT-LGRID-06:** Intrinsic widths are the sum of the columns sized with an unbounded
  width (fixed, then auto and flex columns at their items' minimum or maximum intrinsic
  widths) plus the gaps; intrinsic heights are the sum of the rows for that width, the
  items measured at their columns' widths, plus the gaps.
- **WIDGETS-LGRID-01:** `LayoutGrid` builds a `RenderLayoutGrid` over its children with
  the nearest `Directionality`'s direction (following it); `GridPlacement` sets its
  child's placement data (a child without one is placed automatically).
- **GALLERY-47:** A "dashboard" kitchen-sink card after "time" (twenty-five `SINK_CARDS`
  entries, sample `layout_grid`): a 3-column grid (`auto`, `1fr`, `1fr`, 8 px gaps) with a
  header tile spanning all three columns, labels in the auto column and tiles, echoed by
  `sink-dashboard-state` ("Tiles: 6"). Test: the header spans the grid's width and the
  value tiles line up with each other.

## Performance and allocation

Placement and track sizing allocate small per-layout vectors (tracks and an occupancy
grid); each child is laid out at most twice (an auto pass and a final tight pass).

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **In-house, not taffy**: the protocol-native implementation keeps `tantu-layout`
   dependency-free and its results Flutter-shaped; taffy can still back a future
   CSS-compatible mode.
2. **`flutter_layout_grid`'s names** (`LayoutGrid`, `GridPlacement`, `px`/`fr`/`auto`),
   since Flutter's core has no such layout and `Grid*` names are taken by the data grid.
3. **Spanning items don't grow auto tracks** (a known simplification of CSS's algorithm).
