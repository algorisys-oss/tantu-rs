# Grid body layout

- **Status:** Agreed (the user said "continue" on the draft, 2026-10-12, taking its three
  proposals)
- **Crate:** `tantu-layout`
- **Plan item:** Phase 4, "`DataGrid`" → "data source, columns, header, row and column
  virtualization, scrolling on both axes"
- **Related:** [list](list.md) (`ListExtents`, the one-dimensional version of this),
  [viewport](viewport.md) (the one-axis viewport this replaces for grids),
  [DataGrid](../widgets/data-grid.md) (the widget built on it)

## Purpose

A data grid scrolls on both axes and holds only its visible cells. `RenderListContent` does
this for one axis; nesting it in a horizontal viewport would make every row its own list and
give the header and frozen columns no way to follow one axis only. This spec adds the
two-dimensional equivalent: a box that knows the extents of every row and column (prefix-sum
tables, as the list), is scrolled by a two-dimensional offset, and places the cells it is given
by their row and column. Visibility, building and clipping are the widget's business.

## Scope

In scope:

- `RenderGridBody` and `GridSlot` parent data.
- Row and column extents through `ListExtents`.
- A two-dimensional offset; frozen (non-scrolling) leading columns.

Out of scope (and where it goes):

- Which cells to build, and clipping: the `DataGrid` widget.
- Row spans and column spans: later, if a grid needs them.
- Variable row heights measured from content: later (enterprise grids use fixed heights).

## Public API

```rust
// tantu-layout
/// Which cell a built child of a `RenderGridBody` is (parent data).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridSlot {
    pub row: usize,
    pub column: usize,
}

/// A two-dimensional virtualized content: `row_count` × `column_count` cells with the given
/// extents, scrolled by `offset`, placing the children it is given by their `GridSlot`.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderGridBody {
    pub row_count: usize,
    pub rows: ListExtents,
    pub column_count: usize,
    pub columns: ListExtents,
    /// How far the content is scrolled (positive scrolls content up and left); a non-finite
    /// component counts as 0.
    pub offset: Vec2,
    /// The first `frozen_columns` columns ignore `offset.x` (pinned at the left).
    pub frozen_columns: usize,
}

impl RenderGridBody {
    /// `row_count` rows and `column_count` columns of the given extents, at offset zero with
    /// no frozen columns.
    pub fn new(rows: ListExtents, row_count: usize, columns: ListExtents, column_count: usize) -> Self;
    /// The content's size: every column's width by every row's height.
    pub fn content(&self) -> Size;
}
```

## Behavior

- **LAYOUT-GRID-01:** The body's size is the incoming maximum on each bounded axis, and the
  content's extent on an unbounded one, constrained (`columns.total(column_count)` wide,
  `rows.total(row_count)` tall): a 20-column × 1 000 000-row body with 120 px columns and
  32 px rows under loose 600 × 400 is 600 × 400; under an unbounded height it is
  600 × 32 000 000. Without children the size is the same.
- **LAYOUT-GRID-02:** Each child is laid out with both axes tight at its slot's extents
  (`columns.extent(column)` by `rows.extent(row)`) and placed at
  `(columns.start(column) − offset.x, rows.start(row) − offset.y)`, where the slot is its
  `GridSlot` parent data (a child without one counts as slot (0, 0)); a slot past the counts
  gets the extents `ListExtents` gives (0 past a table). Children are laid out in their order,
  all of them, whether or not they are inside the body's bounds.
- **LAYOUT-GRID-03:** `content()` is the content's size; intrinsic widths and heights are the
  content's. Equal properties compare equal (`LayoutTree::set` with the same values doesn't
  re-lay out) and a changed offset does.
- **LAYOUT-GRID-04:** A child whose column is below `frozen_columns` is placed at
  `columns.start(column)` without subtracting `offset.x` (its row offset still applies), so
  frozen columns stay at the left while the others scroll under them.

## Performance and allocation

Layout is one `ListExtents` lookup per axis per child plus the child's own layout; no
allocation. The content size is computed from the extents (two lookups), not cached.

## Open questions

Proposals (decided by the agent for the draft; say "continue" to accept, or change them):

1. **One layout object for the header and the body.** The widget builds two instances: one
   with a single row (the header, offset `(x, 0)`) and one with every row. The object needs
   no notion of a header band, and each instance clips its own bounds.
2. **Frozen columns are a layout property from the start** (LAYOUT-GRID-04) even though the
   widget uses them only in the next part, since placement is one line and testing it now
   avoids changing the object's shape later. The widget's part adds paint order and clipping.
3. **No cross-axis sizing from children** (unlike `RenderListContent` in an unbounded cross
   axis): a grid's columns have widths; the content size is always the extents'.
