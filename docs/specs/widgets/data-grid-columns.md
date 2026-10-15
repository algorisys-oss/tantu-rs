# DataGrid: column resize and reorder

- **Status:** Implemented (the user said "continue" on the draft, 2026-10-12, taking its seven
  proposals)
- **Crates:** `tantu-widgets` (`DataGrid`, `GridColumn`), `examples/gallery`
- **Plan item:** Phase 4, "`DataGrid`" → "column resize and reorder"
- **Related:** [DataGrid](data-grid.md) (parts 1 and 2), [DataGrid sorting](data-grid-sort.md)
  (part 3, whose header click this amends), [gallery](../examples/gallery.md)

## Purpose

Part 4 of the data grid: a user drags a header's right edge to change a column's width, and
drags a header sideways to move the column; both are app-visible state (shared signals) so an
app can persist a layout and restore it. Sorting moves from the header *press* to the header
*click*, since a press may now start a drag.

## Scope

In scope: reactive column widths (`column_widths`), the resize handle and drag, a minimum
width, the display order (`column_order`), the reorder drag with a drop indicator, the click
amendment to sorting, the gallery's echoes.

Out of scope (and where it goes): a resize cursor (added later as WIDGETS-GRID-29 in
[SplitView](split-view.md), once cursors reached the window); auto-fit on double-click (later);
keyboard resize and reorder (specified in [part 5](data-grid-selection.md)); column
hiding and a column chooser (later, with menus); persisting to disk (the app's job over the
two signals).

## Public API

```rust
// tantu-widgets
impl DataGrid {
    /// Shares the column widths, indexed by source column (the grid owns one, seeded from
    /// each `GridColumn::width`, otherwise) (WIDGETS-GRID-13).
    pub fn column_widths(self, widths: Signal<Vec<f32>>) -> Self;
    /// Lets the user drag headers to reorder columns (default false) (WIDGETS-GRID-14).
    pub fn reorderable(self, enabled: bool) -> Self;
    /// Shares the display order: source column indices in display order (the grid owns one,
    /// the identity, otherwise) (WIDGETS-GRID-14).
    pub fn column_order(self, order: Signal<Vec<usize>>) -> Self;
}

impl GridColumn {
    /// Whether the user can drag the column's right edge (default true) (WIDGETS-GRID-13).
    pub fn resizable(self, resizable: bool) -> Self;
    /// The narrowest the user can drag the column (default 40 px) (WIDGETS-GRID-13).
    pub fn min_width(self, width: f32) -> Self;
}
```

Geometry words: a column's *source index* is its position in `DataGrid::new`'s list, which
titles, builders, `SortState::column`, filters and widths use; its *display position* is where
it shows, given by `column_order`; `frozen_columns` counts display positions.

## Behavior

- **WIDGETS-GRID-13:** Column widths. The grid's extents follow `column_widths`: changing the
  signal (from the app or a drag) re-lays the header and the body at the next frame; an entry
  below the column's `min_width` counts as the minimum, a NaN or negative one as 0 then the
  minimum, and a vector shorter than the columns leaves the missing columns at their
  `GridColumn::width`. A primary press in the header within 6 px left of a column's right
  boundary (the column's own edge; for the last column, the content's end) starts a resize of
  that column when it is `resizable`: each move sets the width to `width at press + dx`
  floored at `min_width` (live, the signal updates and the grid re-lays), the release ends it,
  and neither the press nor the release sorts. A column with `resizable(false)` has no
  handle: a press there is an ordinary header press. The handle lies inside the column it
  resizes, so a column's first pixels are never a handle (and the first column's left edge
  isn't one).
- **WIDGETS-GRID-14:** Reorder. With `reorderable(true)`, a primary press on a header (not on
  a handle) followed by a move of more than 4 px horizontally picks the column up: while it is
  held, the header strip paints a 2 px `primary` drop line at the column boundary nearest the
  pointer's x (between two display positions, or after the last) and the held header's cell
  a `surface_container_highest` background (the scheme's raised surface; it has no
  `surface_variant`); the release moves the column to that boundary by rewriting
  `column_order` (source indices in display order; a drop at the column's own position or
  the one after it changes nothing) and ends the drag without sorting. Without `reorderable`
  a header press never starts a drag. `column_order` shorter than the columns, or with
  duplicates or out-of-range entries, is normalized: missing columns follow in source order
  and bad entries are dropped. Sort state, filters, builders and widths refer to source
  columns, so reordering never changes what a column shows; frozen columns are the first `n`
  display positions, whatever they hold.
- **WIDGETS-GRID-15:** Click sorts (amends WIDGETS-GRID-10): the sort state cycles on a
  primary *click* on a sortable header, a press and a release without a drag (no move beyond
  4 px and no resize); a press that became a resize or a reorder leaves the sort state alone.
  `WidgetTester::tap` is a click, so the earlier tests hold.
- **GALLERY-15:** The Data grid section is `reorderable` with shared widths and order, and
  echoes "Name width: 200" (key `name-width`) and "Order: Id, Name, City, Amount, Active,
  Score" (key `order`); a drag of the Name header's right edge by 40 px gives "Name width:
  240" and keeps the sort text; a drag of the City header 200 px to the left (past the middle
  of Name, so the nearest boundary is the one before Name) gives "Order: Id, City, Name,
  Amount, Active, Score" and the City header now sits where Name was.

## Performance and allocation

A width or order change rebuilds the column extent table (one allocation of `column_count`
floats) and re-lays the visible cells; nothing per frame while idle. During a drag, each
move writes one signal.

## Open questions

Resolved (2026-10-12; the user said "continue" on the draft, taking these proposals):

1. **Widths and order as shared signals, indexed by source column,** rather than mutable
   fields on `GridColumn`: an app can persist and restore them, and every consumer (sort
   state, builders, filters) keeps addressing columns by their stable source index, as
   Flutter's `DataTable` and AG Grid's column state do.
2. **Sorting moves to the click** (release without a drag) so a press can begin a drag.
   Users won't notice; the tests use `tap`.
3. **The handle is the 6 px left of a column's right boundary,** on the column being resized,
   so the last column is resizable too and the first column's left edge is not a handle.
4. **Reorder is opt-in, resize is opt-out:** every grid wants adjustable widths; reorder
   changes the app's mental model of "column 2", so an app enables it.
5. **The drop indicator is a line at the nearest boundary,** not a ghost of the dragged
   header following the pointer: one Scene rect, no overlay, deterministic goldens.
6. **No resize cursor in this part:** `CursorIcon` lacks a column-resize variant and adding
   one touches the view and winit crates; noted as a later amendment.
7. **Frozen columns count display positions,** so a column dragged into the first `n`
   positions becomes frozen and one dragged out stops being frozen (AG Grid pins per column
   instead; that can come with a pin menu).
