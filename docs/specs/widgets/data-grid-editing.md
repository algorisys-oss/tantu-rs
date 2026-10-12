# DataGrid: cell editors

- **Status:** Implemented (2026-10-12; the user said "continue" on the eight proposals)
- **Crates:** `tantu-widgets`, `tantu-view` (BuildCx::focus), `tantu` (exports), `examples/gallery`
- **Plan item:** Phase 4, `DataGrid` part 6, cell editors
- **Related:** [DataGrid](data-grid.md), [selection and navigation](data-grid-selection.md),
  [sorting and filtering](data-grid-sort.md), [TextField](text-field.md),
  [TextField IME](text-field-ime.md), [gallery](../examples/gallery.md)

## Purpose

Edit one grid cell with an ordinary single-line `TextField`, preserving its typed value
and addressing the data source correctly after sorting, filtering and column reorder.
Applications choose editable columns and can reject writes with a useful error message.
Existing read-only sources and grids continue to work without implementing new methods.

## Scope

In scope:

- Opt-in editable columns; double-click entry and F2 on the active body cell.
- One focused in-cell field, initialized with the cell's display text selected in full.
- Typed conversion, synchronous source writes, rejection reporting and local refresh.
- Enter commit, Escape cancel, focus restoration and safe cancellation on model changes.
- Existing TextField editing, undo and IME; semantics, behavior tests and a gallery golden.

Out of scope: custom editor factories, choice/date editors, asynchronous writes, optimistic
server updates, persistent edit controllers, stable row identities, spreadsheet Tab/Enter
movement, commit-on-blur, copy/paste and undo across committed source writes. Clipboard
and forms remain their own Phase 4 plan items. TextField accessibility text ranges remain
the existing ADR 0013 gap.

## Public API

```rust
// tantu-widgets, re-exported by tantu and its prelude
/// Why a cell edit could not be committed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GridEditError {
    /// The source does not support writing this cell.
    ReadOnly,
    /// The editor text cannot be converted to the original value's type.
    InvalidValue,
    /// The source rejected the value; the message is suitable for an app's error label.
    Rejected(String),
}

/// Display messages: "Cell is read-only", "Invalid cell value", or the rejection message.
impl std::fmt::Display for GridEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::error::Error for GridEditError {}

pub trait DataSource: 'static {
    // Existing row_count and cell methods unchanged.
    /// Synchronously writes a value using source row and source column indices.
    /// The default returns GridEditError::ReadOnly without changing anything.
    /// Ok means cell(row, column) now exposes the accepted value; Err leaves data unchanged.
    /// Implementations use interior mutability, as the grid shares the source in an Rc.
    fn set_cell(
        &self,
        row: usize,
        column: usize,
        value: CellValue,
    ) -> Result<(), GridEditError>;
}

impl GridColumn {
    /// Allows an in-cell text editor (default false), including for a custom builder cell.
    /// The builder is replaced by the field for the duration of the edit.
    pub fn editable(self, editable: bool) -> Self;
}

impl DataGrid {
    /// Reports a conversion error or rejected write with source coordinates.
    /// Called once per failed commit attempt; the field stays open with its draft intact.
    /// The app can show the message in a reactive status label. No callback by default.
    pub fn on_edit_error(
        self,
        handler: impl Fn(GridCell, GridEditError) + 'static,
    ) -> Self;
}
```

`FnSource` inherits the default read-only setter. A writable source implements `DataSource`;
the gallery demonstrates this with sparse overrides of its generated values. The setter
and error callback use source coordinates; selection continues using display coordinates.
There are no new renderer, platform or dependency edges.

## Behavior

- **WIDGETS-GRID-23:** Write contract and opt-in. Existing implementations compile without
  a setter and return ReadOnly when called. Every column defaults to noneditable. Entry
  into a noneditable column, header, scrollbar, blank body area or empty grid creates no
  field and performs no write. Opting a column into editing permits entry even when the
  source's setter will reject it; a grid does not probe write support by calling set_cell.
  Errors have the Display behavior above and implement std::error::Error.

- **WIDGETS-GRID-24:** Entry and source identity. Two unmodified primary presses on the
  same editable body cell, at most 500 ms apart and within 4 logical px, open its editor
  on the second press. A press consumed by a descendant control does not count. A
  different cell, modifiers, a drag beyond the slop or an expired interval resets the
  candidate; a third press in the opened field follows TextField's own pointer handling.
  Frozen-column occlusion and normalized display order use the existing grid hit test.
  Double-click works in selection mode None without creating a selection. In enabled
  selection modes entry selects the target as a single selection, ignoring a previous
  range. An unmodified F2 press (not release) on the focused grid opens the active body
  cell when editable; no selection, a header cursor or a noneditable active cell leaves
  F2 unhandled. Repeats never create a second session. Entry reveals the cell, resolves
  source indices using the current row map and column order, snapshots the value, and
  stores those source indices for the session. No entry performs a source write.

- **WIDGETS-GRID-25:** Field, focus and geometry. At most one editor exists per grid. It
  replaces the target cell's normal content, including a custom builder, and is laid out
  with tight constraints equal to the cell's width and row height; its default minimum
  height yields to those constraints. It adds no row height or column width and is clipped
  to the cell and scrolling strip, including the frozen boundary. A zero width, zero row
  height or empty visible cell rectangle prevents entry. Small nonzero cells clip rather
  than overflow. The field starts with CellValue::Display text selected in full, receives
  actual focus after it is built, and uses the existing theme, TextField undo, caret
  scrolling and IME behavior. It has a TextInput node named "<column title>, row <display
  row>" (zero-based), with its draft as value and existing Focus/SetValue actions. The cell
  retains table coordinates and the committed source value while editing. The grid omits
  its active_descendant while focus is in the field. Selection highlighting remains, but
  the grid's navigation outline is hidden. The field is the grid's only temporary extra
  Tab stop. Grid body/header commands never act on keys from its editor; handled field
  keys do not reach ancestor commands.

- **WIDGETS-GRID-26:** Conversion. Commit preserves the entry value's variant: Text keeps
  the exact text, including whitespace and an empty string; Empty becomes Text for a
  nonempty draft and remains Empty for an empty one. Int accepts trimmed Rust i64 decimal
  syntax, rejecting overflow and nonintegers. Float accepts trimmed Rust f64 syntax but
  rejects NaN and either infinity, including parse overflow. Bool accepts trimmed ASCII
  case-insensitive "true" or "false" only. Numeric/boolean empty drafts are invalid.
  Conversion failure reports InvalidValue without calling set_cell or changing the source,
  and retains the draft, focus and text selection. The original variant determines parsing
  throughout the session. An exact unchanged Display string closes without parsing or
  writing, including an original nonfinite Float; formatting-only changes are attempts.

- **WIDGETS-GRID-27:** Commit, rejection and refresh. An Enter press submits once, using
  the field's single-line submit behavior. While its EditingValue has an active composing
  range, Enter does not submit the source; IME Commit updates the draft and a later Enter
  submits. A changed, valid draft calls set_cell exactly once with the snapshotted source
  indices and converted value. On Err, call on_edit_error if installed; retain the field
  and its draft so correction or retry is possible. Cancellation and successful or unchanged
  submissions never call the error callback. On Ok, close the field and refresh source
  values, custom builder content and semantics on the next pump without requiring an
  external revision bump; other grids sharing the source still require their own revision.
  Client sorting and filtering recompute against accepted data. Without client sorting or
  an active filter, preserve selection. With either, clear it because a display row may
  now denote another source row; do not try to follow the record. Restore actual focus to
  the live grid when submission closes its focused field. Duplicate submits after closing
  cannot write again. A source may normalize its accepted value; refreshed cells read the
  source rather than assuming the submitted value was stored verbatim.

- **WIDGETS-GRID-28:** Cancellation and lifecycle. Escape press cancels the whole draft,
  even during IME composition, performs no write and restores focus to the grid. Tab and
  Shift+Tab cancel and continue normal traversal out of the grid; they neither submit nor
  move to another editor. Losing field focus cancels without stealing focus back, so an
  outside control's press still works. A press on another cell or header cancels before
  its usual selection/sort/resize/reorder behavior proceeds. A sort state, filter,
  external revision or normalized column-order change cancels before any later submit;
  existing selection-reset rules still apply. These model changes also clear a pending
  double-click candidate, so two presses cannot target different source records. Changing external selection to a
  different active cell, or clearing it, also cancels in enabled modes. Width, theme and viewport
  changes preserve the draft while the cell has a nonempty visible rectangle. Scrolling
  preserves it while any part of the cell remains visible; full occlusion or leaving either
  viewport axis cancels, including cells still built by the virtualization buffer. Do not
  pin a row or force scroll back to keep an editor alive. If a model/viewport cancellation
  removes the still-focused field, restore focus to the grid; if focus already moved,
  preserve the new target. Removing/rebuilding the grid drops the session, timers and
  controller, performs no write and never focuses a disposed element. Cleanup cannot
  read disposed reactive state or invalidate a replacement grid's session.

- **GALLERY-17:** The Data grid section keeps its million generated rows and uses a
  writable source storing only edited overrides. Id stays read-only; Name, City, Amount,
  Active and Score opt into editing. Score rejects integers outside 0..=100 with
  "Score must be between 0 and 100". The section explains double-click/F2, Enter and
  Escape and shows a reactive error label keyed `grid-edit-error`, initially
  "Edit error: none"; a rejection displays "Edit error: <message>". Successful writes
  clear that label. Tests replace Person 2 through F2/Enter, cancel another draft through
  Escape, and verify an invalid Score stays editable without changing its source. A
  double-click test proves the pointer entry path. Golden `gallery-grid-editor.png` at
  960 × 640 shows the selected text in the focused Name editor in the existing 32 px
  row. Update existing grid goldens only where the new instructions/status alter layout.

## Performance and allocation

One edit session holds one source/display coordinate pair, one original value and one
TextEditingController with its ordinary per-session undo history. It never allocates per
source row or selected cell. Only the target cell switches content on entry/cancel;
keystrokes update the field without reading or rebuilding the source's other cells.
Unsuccessful commits do not invalidate row maps. Successful commits refresh built cells;
an active client sort/filter has its existing whole-source recomputation cost. Read-only
grids retain WIDGETS-GRID-02's virtualization bounds and the GridScroll target of under
8 ms per step; investigate regressions over 20% on the same machine. Double-click state
and expiry belong to the mounted grid's reactive scope and are disposed with it.

## Open questions

Resolved (2026-10-12; the user said "continue", accepting all eight proposals):

Integration amendment (review): add `BuildCx::focus` under VIEW-FOCUS-10 so the editor
can take focus after building and restore it when closing. This grants no mutable tree
access and changes no dependency edge.

1. **Opt-in per column**, with a default read-only DataSource setter for compatibility.
   Double-click works without selection; F2 needs an active cell in an enabled mode.
2. **A synchronous typed setter taking &self**, so existing shared Rc sources can use
   interior mutability. Add GridEditError and an app callback for visible rejection messages.
   Async persistence and custom editor factories wait for later specs.
3. **Preserve the original value type** with the conversion rules above; keep rejected
   drafts open. Float edits must be finite. Exact unchanged text closes without writing.
4. **Enter commits; Escape, blur and Tab cancel.** Tab then leaves normally. This avoids
   implicitly writing partial input or trapping users in a rejected draft.
5. **Cancel on model changes or complete viewport exit.** Keep no hidden editors alive
   and use no pinned rows; widths/theme changes preserve visible drafts.
6. **Refresh this grid after a successful write**, even without an app revision signal.
   Recompute active sort/filter maps and clear selection only when such a map may change.
7. **Reuse TextField inside tight cell constraints**, retaining 32 px rows. Use a named
   TextInput node and normal IME input, suppressing source submission during preedit.
8. **Use sparse overrides in the million-row gallery**, with Score rejection demonstrating
   source validation and a status label for errors. No full million-row editable store.
