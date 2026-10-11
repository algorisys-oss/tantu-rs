# Text editing model

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-11; review
  the Open questions)
- **Crate:** `tantu-widgets` (module `text_editing`; no view code)
- **Plan item:** Phase 3, "Widgets" → "`TextField`, in parts" → "editing model (text,
  selection, undo)"
- **Related:** [reactive signals](../reactive/signals.md), Flutter's `TextEditingValue`,
  `TextSelection`, `TextEditingController` and `UndoHistory`; UAX #29 (graphemes and words)

## Purpose

Everything a text field does to its text that doesn't need a layout: a value (text,
selection, composing region), the edits (insert, delete, caret movement by grapheme, word,
line boundary or document, selection), and an undo history, in a controller the app and the
widget share through signals. It is pure data with pure operations, so it is testable without
a window, and `TextField` (the next part) is a thin layer that maps keys and pointer hits to
it. Vertical caret movement and hit-testing need the paragraph's layout and come with the
widget; IME composition uses `composing` and comes third.

Offsets are byte offsets into the UTF-8 text (Rust's native unit; Flutter's are UTF-16 code
units), always on character boundaries. Movement and deletion work in extended grapheme
clusters and UAX #29 words through `unicode-segmentation`, which the workspace already
builds.

## Scope

In scope:

- `TextSelection`, `TextRange`, `EditingValue`, `Movement`, `Direction`.
- Edits on `EditingValue`: `insert`, `delete`, `move_caret`, `select_all`, `with_selection`,
  `selected_text`.
- `TextEditingController`: a shared, reactive value with an undo/redo history and edit
  coalescing.

Out of scope (and where it goes):

- Up/down movement, hit-testing, affinity at wrapped line ends, bidi caret logic: the widget
  part, with parley's layout.
- IME composition updates: the IME part (`composing` is carried, not edited, here).
- Clipboard: the widget part, through the platform.
- Time-based undo coalescing (Flutter throttles at 500 ms): no timers yet; coalescing is by
  edit kind.
- Input formatters, max length, obscured text: with the widget or forms.

## Public API

```rust
// tantu-widgets (module text_editing, re-exported at the crate root)
/// A selection as two byte offsets: `base` (where it started) and `extent` (the caret).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextSelection { pub base: usize, pub extent: usize }
impl TextSelection {
    pub const fn collapsed(offset: usize) -> Self;
    pub const fn new(base: usize, extent: usize) -> Self;
    pub fn is_collapsed(self) -> bool;
    pub fn start(self) -> usize;   // min
    pub fn end(self) -> usize;     // max
}

/// A byte range (the composing region).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRange { pub start: usize, pub end: usize }

/// The text, selection and composing region of a text field (Flutter's `TextEditingValue`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditingValue { pub text: String, pub selection: TextSelection, pub composing: Option<TextRange> }

/// A unit of caret movement or deletion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movement { Grapheme, Word, LineBoundary, Document }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction { Backward, Forward }

impl EditingValue {
    /// `text` with the caret at its end and no composing region.
    pub fn new(text: impl Into<String>) -> Self;
    /// The value with `selection`, clamped to the text's character boundaries.
    pub fn with_selection(self, selection: TextSelection) -> Self;
    pub fn selected_text(&self) -> &str;
    /// Replaces the selection with `text`; the caret ends after it.
    pub fn insert(&self, text: &str) -> Self;
    /// Deletes the selection, or one `by` unit in `direction` from a collapsed caret.
    pub fn delete(&self, direction: Direction, by: Movement) -> Self;
    /// Moves the caret one `by` unit; with `select` the base stays (extending the selection).
    pub fn move_caret(&self, direction: Direction, by: Movement, select: bool) -> Self;
    pub fn select_all(&self) -> Self;
}

/// A text field's value shared between the app and the widget, with undo (Flutter's
/// `TextEditingController` plus `UndoHistory`). `Copy`; create it inside a runtime.
#[derive(Clone, Copy)]
pub struct TextEditingController { /* value: Signal<EditingValue>, history: Signal<History> */ }

impl TextEditingController {
    pub fn new(text: impl Into<String>) -> Self;
    /// The value (a tracked read).
    pub fn value(&self) -> EditingValue;
    /// The text (a tracked read).
    pub fn text(&self) -> String;
    /// Replaces the value (an undoable step of its own); clamps the selection.
    pub fn set_value(&self, value: EditingValue);
    /// `set_value` with `text` and the caret at its end.
    pub fn set_text(&self, text: impl Into<String>);
    /// Applies an edit produced from the current value, recording it for undo.
    pub fn edit(&self, f: impl FnOnce(&EditingValue) -> EditingValue);
    /// Changes the selection only (not undoable, but it ends a coalescing group).
    pub fn select(&self, selection: TextSelection);
    pub fn undo(&self) -> bool;
    pub fn redo(&self) -> bool;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}
```

Word boundaries: the text is split at UAX #29 word boundaries; a segment is a *word* when it
contains an alphanumeric character. Moving forward by `Word` goes to the end of the next word
after the caret (skipping non-word segments); backward goes to the start of the previous
word. `LineBoundary` is the start or end of the line of hard breaks (`\n`) the caret is on.

## Behavior

- **WIDGETS-EDIT-01:** `TextSelection` and `EditingValue` basics: `collapsed(n)` has `base ==
  extent == n`; `start`/`end` order the offsets; `EditingValue::new("ab")` has the caret at 2
  and no composing region; `with_selection` clamps each offset to the text length and back
  to a character boundary (an offset inside "é" moves to its start), and `Default` is empty.
- **WIDGETS-EDIT-02:** `insert` replaces the selection (collapsed: inserts at the caret) with
  the text, puts a collapsed caret after it, and clears `composing`. Inserting into a
  selection that spans the text replaces it all.
- **WIDGETS-EDIT-03:** `delete` with a non-collapsed selection removes it (any direction or
  unit) and collapses the caret at its start. With a collapsed caret: `Grapheme` removes one
  extended grapheme cluster before or after it (a combining sequence or an emoji with
  modifiers goes as one), `Word` removes to the previous word start or next word end,
  `LineBoundary` to the line's start or end, `Document` to the text's start or end. At the
  start (backward) or end (forward) nothing changes. `composing` is cleared by a change.
- **WIDGETS-EDIT-04:** `move_caret` without `select`: a non-collapsed selection collapses to
  its start (`Backward`) or end (`Forward`) for `Grapheme`; otherwise, and for every other
  unit, the caret moves from the extent one unit in the direction, stopping at the text's
  ends. With `select`, the base stays and only the extent moves. Movement never changes the
  text or `composing`.
- **WIDGETS-EDIT-05:** `select_all` selects the whole text (base 0, extent at the end);
  `selected_text` is the text between start and end (empty when collapsed).
- **WIDGETS-EDIT-06:** The controller: `new(text)` holds `EditingValue::new(text)`; `edit`
  applies the function and records the previous value; `undo` restores it (text and
  selection) and `redo` reapplies; a new `edit` or `set_value` clears the redo stack;
  `can_undo`/`can_redo` report the stacks; `undo`/`redo` with nothing to do return false and
  change nothing. `set_value` and `set_text` are steps of their own.
- **WIDGETS-EDIT-07:** Coalescing: consecutive `edit`s that each insert a single grapheme at
  the caret (a collapsed selection) form one undo step; a whitespace grapheme joins the
  group and then ends it, and a deletion, a multi-grapheme insert, `select`, `set_value` or
  an undo/redo ends it (each a step of its own where it changes the value), so typing
  "ab cd" then undoing twice gives "ab " then "".
- **WIDGETS-EDIT-08:** The controller is reactive: a memo or effect reading `value()` or
  `text()` re-runs after `edit`, `set_text`, `select`, `undo` and `redo`; `value()` inside
  `edit`'s closure sees the current value.
- **WIDGETS-EDIT-09:** `compose` and `commit` edit the composing region; see
  [TextField IME](text-field-ime.md).

## Open questions

Decided by the agent while the user was away (review):

1. **Byte offsets** (not UTF-16 as Flutter), kept on character boundaries by clamping.
2. **`unicode-segmentation`** for graphemes and words (already a transitive dependency of the
   text stack), used in `tantu-widgets`; parley's cluster data serves the widget part for
   layout-dependent movement.
3. **Coalescing by edit kind** (single non-whitespace graphemes at the caret) instead of
   Flutter's 500 ms throttle, since there are no timers yet; it also makes undo deterministic
   in tests.
4. **The controller is two signals** (`value`, `history`) behind a `Copy` handle, so widgets
   and apps capture it in closures like any signal, and the value is readable reactively.
5. **The model lives in `tantu-widgets`** next to the widget that uses it; it has no view
   dependency and could move to its own crate if forms need it without widgets.
