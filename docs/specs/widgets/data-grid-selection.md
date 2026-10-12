# DataGrid: selection and keyboard navigation

- **Status:** Agreed (2026-10-12; the user authorized all permissions after the draft;
  the agent takes the nine recommended proposals)
- **Crates:** `tantu-widgets`, `tantu-view` (semantics and pointer modifiers), `tantu-a11y`
  (property mapping), `tantu` (pointer translation and exports), `tantu-test` (modified taps),
  `examples/gallery`
- **Plan item:** Phase 4, `DataGrid` part 5, selection models (row, cell, range), keyboard
  navigation and accessibility indices
- **Related:** [DataGrid](data-grid.md), [sorting and filtering](data-grid-sort.md),
  [column resize and reorder](data-grid-columns.md), [semantics](../a11y/semantics.md),
  [focus](../view/focus.md), [gallery](../examples/gallery.md)

## Purpose

Let users select rows, individual cells or a rectangular range and navigate a virtualized
grid with the keyboard. Applications can read or set the selection and translate display
positions to their data source. Assistive technology receives the table dimensions, cell
positions, values, selection and current navigation target.

This amends WIDGETS-GRID-05's provisional keyboard scrolling only when selection is enabled.
The implementation follows agreement of this draft and failing rule tests.

## Scope

In scope:

- Selection modes, anchor/active endpoints and an app-visible selection controller.
- Primary click and Shift-click selection; Shift navigation extends contiguous selections.
- Pointer modifier snapshots, forwarded by the runner and supplied by the tester.
- A single grid Tab stop, body and header navigation, active-cell reveal and painting.
- Keyboard sorting, resize and reorder of the current header (promised by part 4).
- Backend-neutral semantics properties and their AccessKit mapping; gallery controls,
  selection echo, behavior tests and selection goldens.

Out of scope: disjoint selections, Control/Command-click toggles, pointer-drag selection
and edge auto-scroll, select-all, copy/paste, row identities stable across data revisions,
row/column spans, selection callbacks separate from reactivity, cell editing (part 6),
and virtual accessibility nodes for unbuilt cells. Interactive cell descendants retain
their existing focus and event handling; managing editor entry/exit belongs to part 6.

## Public API

```rust
// tantu-widgets, re-exported by tantu and its prelude
/// How a grid selects its body cells. Default: None.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridSelectionMode {
    #[default]
    None,
    /// One row, or a contiguous interval of rows extended with Shift.
    Row,
    /// Exactly one cell; Shift does not extend it.
    Cell,
    /// An inclusive rectangle between anchor and active cells.
    Range,
}

/// A zero-based display position. Source positions use the same shape but are returned
/// explicitly by GridSelectionController::source_cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridCell {
    pub row: usize,
    pub column: usize,
}

/// Selection endpoints in display coordinates; active is the keyboard cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSelection {
    pub anchor: GridCell,
    pub active: GridCell,
}

impl GridSelection {
    /// A selection whose two endpoints are cell.
    pub fn single(cell: GridCell) -> Self;
    /// Membership without enumerating cells: None never contains a cell, Row checks the
    /// inclusive row interval, Cell checks active, Range checks both inclusive intervals.
    pub fn contains(self, cell: GridCell, mode: GridSelectionMode) -> bool;
}

/// Shares selection and the latest display-to-source mapping with an application.
/// Clones share state. Create within a reactive runtime, as with ScrollController.
/// Use one controller per mounted grid.
#[derive(Clone)]
pub struct GridSelectionController { /* ... */ }

impl GridSelectionController {
    /// Starts with no selection and no attached grid.
    pub fn new() -> Self;
    /// A tracked read of the selection; a reactive label can echo it.
    pub fn selection(&self) -> Option<GridSelection>;
    /// Sets display endpoints, or clears them with None. A mounted grid normalizes them
    /// and reveals active on its next frame (WIDGETS-GRID-16, -20).
    pub fn set_selection(&self, selection: Option<GridSelection>);
    /// Maps a display cell to source row and source column using the latest pumped model.
    /// None when out of bounds or detached. A tracked read, with constant-time lookup;
    /// sorting/filtering share their existing row map, without copying it.
    pub fn source_cell(&self, display: GridCell) -> Option<GridCell>;
}

/// Default creates the same empty, detached controller as new().
impl Default for GridSelectionController {
    fn default() -> Self;
}

impl DataGrid {
    /// Enables a selection model (default None).
    pub fn selection_mode(self, mode: GridSelectionMode) -> Self;
    /// Shares selection and mapping with the app; the grid creates its own otherwise.
    pub fn selection_controller(self, controller: GridSelectionController) -> Self;
}

// tantu-view, backend-neutral additions to Semantics (defaults shown)
pub struct Semantics {
    // existing fields unchanged
    /// Number of displayed body rows (the header is excluded).
    pub row_count: Option<usize>,             // None
    pub column_count: Option<usize>,          // None
    /// Zero-based body-row and display-column position; headers have no row_index.
    pub row_index: Option<usize>,             // None
    pub column_index: Option<usize>,          // None
    /// Direction of a column header's sort, or None for unsorted.
    pub sort_direction: Option<SortDirection>, // None
    /// Whether the container permits selecting more than one item.
    pub multiselectable: bool,                // false
    /// The semantic descendant currently addressed by a composite widget's keyboard
    /// navigation; actual keyboard focus stays on the composite.
    pub active_descendant: Option<ElementId>, // None
}

/// A header's sorting direction for assistive technology.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection { Ascending, Descending }

// tantu-view: addition to the existing PointerEvent
pub struct PointerEvent {
    // existing kind and position
    /// The modifiers held when this event occurred.
    pub modifiers: Modifiers,
}

// tantu-test
impl WidgetTester {
    /// Presses and releases at the finder's center with modifiers, then pumps.
    pub fn tap_with_modifiers(&mut self, finder: &Finder, modifiers: Modifiers);
}
```

The controller maps individual cells, including unbuilt ones; it does not enumerate a
selection. With sorted or filtered rows, a displayed interval need not be contiguous in
source coordinates. Applications map each displayed row they need. The column in
`SortState`, widths and builders remains a source index, as before.

## Behavior

- **VIEW-EVENT-08:** Pointer modifier snapshots. Capture and bubble handlers receive the
  exact modifiers supplied in PointerEvent, also through pointer capture; local coordinate
  conversion preserves them. Synthesized accessibility Click uses default modifiers so a
  screen-reader click does not accidentally extend a prior selection. Existing callers
  supply default modifiers to retain their behavior.

- **FACADE-APP-16:** The runner snapshots the target window's latest ModifiersChanged
  state onto pointer move, press, release and wheel events. A window starts with default
  modifiers; changes in another window do not overwrite its snapshot. The platform trait
  and winit adapter already expose ModifiersChanged and need no new event type.

- **TEST-WT-16:** tap_with_modifiers sends primary Down and Up with the given modifiers
  at the same center, then pumps as tap does. tap, hover, drag and scroll continue using
  default modifiers; raw pointer events retain exactly what the caller supplied. A test
  verifies both phases and the frame update, not just a helper's event construction.

- **WIDGETS-GRID-16:** State and bounds. Without a selection mode the grid keeps part 1's
  keyboard scrolling and non-traversable focus, renders no selection and normalizes its
  controller's selection to None. In enabled modes it starts unselected. External state
  updates affect paint and semantics on the next pump. If either endpoint is outside the
  displayed row/column counts, the whole selection becomes None (no partial clamping).
  Zero rows or zero columns admit no selection. Cell normalizes anchor to active; Row
  retains both endpoints' column positions for navigation but membership uses rows only.
  `single` and `contains` also work on reversed endpoints and usize::MAX without overflow.

- **WIDGETS-GRID-17:** Pointer selection. A primary press on an ordinary body cell that
  reaches the grid's bubble handler selects that cell and makes the grid the navigation
  focus. Without Shift it replaces both endpoints; with Shift, Row and Range preserve
  the previous anchor and move active, or start a single selection if none existed. Cell
  always selects a single cell. Secondary presses, the header, scrollbars and blank body
  space do not change body selection. Hit-testing uses the displayed column order and
  frozen-column occlusion: pressing a frozen cell selects it, not content under it.
  Scrollbar drags and header drags preserve selection until an actual reorder (rule 21).
  Descendant controls that stop a press keep their existing behavior. No new per-cell
  pointer handler may capture or suppress a descendant's press.

- **WIDGETS-GRID-18:** Keyboard body navigation. Enabled grids are one Tab stop; Tab and
  Shift+Tab leave through normal focus traversal rather than visiting each cell. Receiving
  focus alone does not select. From no selection the first recognized navigation press
  selects (0, 0), without taking an extra step. Thereafter arrows change active by one
  display row/column (including in Row mode). Home/End choose the first/last display
  column of the current row; Control+Home/End, or Command+Home/End on macOS, choose the
  first/last cell of the grid. PageUp/Down change the row by
  max(1, floor(body viewport height / row height)), saturating at the bounds; zero row
  height uses one row. Shift applies rule 17's extension policy to all these moves.
  Unmodified moves replace the anchor. Escape clears selection. Recognized keys stop at
  the grid even at a boundary; when the grid is empty navigation continues unhandled.
  Releases and unrelated modifier combinations continue unhandled; repeats act as presses.
  A descendant that consumes a key prevents grid navigation. Selection never wraps.

- **WIDGETS-GRID-19:** Header navigation and commands. With selection enabled and a
  nonzero header height, F6 switches between the body cursor and a virtual header cursor;
  entering the header uses active's column, or column 0; returning preserves body
  selection (or leaves it empty). F6 is handled only when columns and a visible header
  exist. A primary header press sets this header cursor without changing body selection;
  existing resize/reorder/click-to-sort behavior is preserved. In the header, arrows left
  and right move by one display column, Home/End to the first/last; Enter or Space cycles
  sorting when sortable. Alt+Left/Right resizes the current header by -10/+10 logical px,
  respecting min_width and resizable. The platform primary modifier plus Shift+Left/Right
  moves it one display position when reorderable; source-indexed widths and sort state
  follow it, including across the frozen boundary. Recognized supported commands stop
  even at bounds; unsupported commands continue. Keyboard reorder keeps the header
  cursor on the moved source column. Header navigation reveals that header horizontally.

- **WIDGETS-GRID-20:** Reveal and paint. A changed active cell, by input or external
  selection, jumps controllers by the least amount needed to reveal it, without a wheel
  animation. Vertical reveal uses the body viewport; horizontal reveal uses the region
  to the right of the frozen columns. Frozen cells do not change horizontal scroll. A
  cell larger than the available viewport aligns its leading edge; a zero-sized viewport
  does not cause repeated scrolling. Extents clamp every target. Selection painting uses
  the theme's surface blended with primary at 12% (an opaque background, including
  frozen cells) and on_surface for default text; custom builder content controls its own colors.
  The active body cell or header has a 2 px primary outline inside its bounds while the
  grid itself has focus. Losing focus keeps selection backgrounds and hides this outline.
  The header cursor hides the body outline. Only built cells are painted or instantiated:
  selecting a huge range never builds that range.

- **WIDGETS-GRID-21:** Model changes and source mapping. `source_cell` returns source
  indices after client sorting, filtering and normalized column reorder, identity indices
  without a map. Any sort-state, filter or revision change clears body selection and the
  body cursor, since DataSource has no stable row keys. A changed normalized column order
  also clears body selection; width changes, scrolling and theme changes preserve it.
  The header cursor follows its source column across reorder and survives row changes.
  A model reset wins over external selection writes in the same pump: pump the model
  first, then restore a display selection if desired. Initial attachment preserves a
  valid selection supplied before build. Removal detaches the mapping, so source_cell
  returns None, but retains the controller's selection for later reattachment. Detaching
  one grid must not invalidate a newer attachment of the same controller.

- **VIEW-SEM-04:** Table and composite properties. Semantics stores and reactively updates
  all new properties with the documented defaults, using the same set/update/query/remove
  contract as VIEW-SEM-01. `active_descendant` is an ElementId, never an AccessKit NodeId;
  it grants neither keyboard focus nor actions on its own.

- **A11Y-TREE-05:** Table and composite mapping. The new semantics fields map to
  AccessKit's row_count, column_count, row_index, column_index, sort_direction and
  is_multiselectable properties without changing their zero-based indices. Only a live
  semantic strict descendant is emitted as active_descendant; removed, nonsemantic,
  self or unrelated ids omit the property. Actual TreeUpdate focus still follows the view
  tree's focus (the grid for grid navigation); clearing properties removes them on the
  next update. Other widgets retain their previous output.

- **WIDGETS-GRID-22:** Grid accessibility, in every selection mode including None. The
  Grid node reports displayed body row count and total column count; header nodes report
  display column index, name, and the current sort direction. Body cells report display
  row/column indices and their source value's Display text as value, including custom
  builder cells, reactively on revision and row-map changes. With selection enabled each
  body cell reports selected true/false according to the mode (all columns of selected
  rows in Row); None omits selected. Row and Range set multiselectable on the grid; Cell
  and None do not. While the grid has focus its active_descendant names the built active
  body cell or header; if scrolling unbuilds that cell the property is absent until it is
  built again. Cells expose Click for selection when enabled, sortable headers Click for
  sort; the grid exposes Focus. Synthesized Click selects/sorts through existing input
  behavior, and ScrollIntoView on a built buffered cell or its descendant reveals that
  cell via grid geometry.
  No node is created for an unbuilt cell or for a million-row selection. No extra cell()
  reads occur while merely moving selection across already built cells.

- **GALLERY-16:** The Data grid section starts in Row mode and offers None/Row/Cell/Range
  controls (key `grid-selection-mode`) plus a reactive selection text (key
  `grid-selection`), initially "Selection: none". Selecting displayed row 2, column 1
  echoes "Selection: rows 2..2" in Row, "Selection: cell (2, 1)" in Cell, or
  "Selection: (2, 1)..(2, 1)" in Range. Shift-selecting (4, 2) in Range echoes
  "Selection: (2, 1)..(4, 2)"; reversing endpoints keeps anchor and active order in the
  echo. Changing mode clears selection and updates the grid. Tests cover click, Shift
  navigation, reveal beyond the initial viewport, source mapping, header commands and
  semantics; goldens `gallery-grid-row-selection.png` and
  `gallery-grid-range-selection.png` show row and rectangular highlights. Refresh the
  existing grid golden for the added controls. The million-row gallery keeps client_sort
  disabled as before; mapped sorting/filtering is exercised by small widget fixtures.

## Performance and allocation

Selection is two endpoints, not a collection of selected rows or cells. Membership and
display-to-source mapping are O(1); normalization never scans the data source. Changes
update only built cells' paint/semantics, with O(built cells) work, independent of selected
area and total rows. Reveal may binary-search the existing column extent table. The
controller shares the existing Rc row map and column order, never a million-row copy.
Selection changes do not rerun sorting/filtering or rebuild cell content. Store/cache
semantic values when the cell content is refreshed to avoid new source reads on selection.
Navigation and range extension allocate no collection proportional to selected area.
Keep WIDGETS-GRID-02's virtualization bounds and the GridScroll benchmark target (under
8 ms, investigate regressions over 20% on the same machine).

## Open questions

Resolved for implementation on 2026-10-12; recommendations accepted under the user's
authorization to continue autonomously:

1. **Opt-in selection**, default None, preserving existing grids' keyboard behavior.
   The gallery enables Row and offers all four modes.
2. **Display-coordinate endpoints and a selection controller with source_cell mapping.**
   This adds a controller rather than exposing a bare signal: apps can read the selected
   data after sorting/filtering without reconstructing the grid's private row map.
3. **Row supports contiguous Shift ranges; Cell is single; Range is one rectangle.**
   Disjoint toggle selection and pointer dragging wait for a later amendment.
4. **Clear on sort, filter, revision or actual column reorder.** No stable row-key API
   exists, so preserving record identity would need a separate data-source design.
5. **One grid Tab stop and a virtual cursor; F6 switches body/header.** The grid retains
   actual focus through virtualization; active_descendant tells assistive technology
   which built cell or header is current. Interactive descendants keep their own handling.
6. **Home/End are row-local; primary+Home/End go to grid corners.** Header Enter/Space
   sorts, Alt+Left/Right resizes by 10 px, primary+Shift+Left/Right reorders one position.
7. **Selection uses a 12% primary tint over surface and an inset primary cursor outline.** Reveal
   jumps immediately and uses the scrolling region beside frozen columns.
8. **Zero-based accessibility positions exclude the header from body row count.** Values,
   selection, sort direction and a validated active_descendant accompany the indices;
   unbuilt cells remain absent from the accessibility tree.
9. **Pointer events gain modifier snapshots.** This is needed for Shift-click; the runner
   forwards each window's state and the tester gains tap_with_modifiers. Existing pointer
   literals will need a default-modifier field, with no other behavior change.

Before agreement, settle these proposals. Then commit the Agreed spec and amendments to
the existing grid/column/semantics specs, write failing tests (including pure membership
and normalization cases, pointer/keyboard fixtures, frozen/reordered/mapped cells,
controller disposal, a11y property mapping and the gallery), and implement. No architecture
dependency edge or pipeline stage changes are proposed.

Implementation amendment (2026-10-12): ColorScheme has no secondary_container tokens;
WIDGETS-GRID-20 uses the existing surface/primary blend and on_surface instead.
