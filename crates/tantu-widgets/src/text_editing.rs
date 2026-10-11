//! The text editing model: [`EditingValue`] and its edits, and [`TextEditingController`].
//! Spec: `docs/specs/widgets/text-editing.md`.

use tantu_view::reactive::{Signal, signal};

/// A selection as two byte offsets: `base` (where it started) and `extent` (the caret).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextSelection {
    /// Where the selection started.
    pub base: usize,
    /// Where it ends: the caret.
    pub extent: usize,
}

impl TextSelection {
    /// A caret at `offset`.
    pub const fn collapsed(offset: usize) -> Self {
        TextSelection {
            base: offset,
            extent: offset,
        }
    }

    /// A selection from `base` to `extent`.
    pub const fn new(base: usize, extent: usize) -> Self {
        TextSelection { base, extent }
    }

    /// True when the selection is just a caret.
    pub fn is_collapsed(self) -> bool {
        self.base == self.extent
    }

    /// The smaller offset.
    pub fn start(self) -> usize {
        self.base.min(self.extent)
    }

    /// The larger offset.
    pub fn end(self) -> usize {
        self.base.max(self.extent)
    }
}

/// A byte range of the text (the composing region).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRange {
    /// The range's start.
    pub start: usize,
    /// Its end (exclusive).
    pub end: usize,
}

/// The text, selection and composing region of a text field (Flutter's `TextEditingValue`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditingValue {
    /// The text.
    pub text: String,
    /// The selection (byte offsets on character boundaries).
    pub selection: TextSelection,
    /// The IME's composing region, if any.
    pub composing: Option<TextRange>,
}

/// A unit of caret movement or deletion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movement {
    /// One extended grapheme cluster.
    Grapheme,
    /// To the previous word's start or the next word's end.
    Word,
    /// To the start or end of the line of hard breaks.
    LineBoundary,
    /// To the start or end of the text.
    Document,
}

/// Backward (towards the start) or forward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Towards the text's start.
    Backward,
    /// Towards its end.
    Forward,
}

impl EditingValue {
    /// `text` with the caret at its end and no composing region.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let end = text.len();
        EditingValue {
            text,
            selection: TextSelection::collapsed(end),
            composing: None,
        }
    }

    /// The value with `selection`, clamped to the text's character boundaries.
    pub fn with_selection(self, selection: TextSelection) -> Self {
        let _ = selection;
        todo!("WIDGETS-EDIT-01")
    }

    /// The selected text (empty when collapsed).
    pub fn selected_text(&self) -> &str {
        todo!("WIDGETS-EDIT-05")
    }

    /// Replaces the selection with `text`; the caret ends after it.
    pub fn insert(&self, text: &str) -> Self {
        let _ = text;
        todo!("WIDGETS-EDIT-02")
    }

    /// Deletes the selection, or one `by` unit in `direction` from a collapsed caret.
    pub fn delete(&self, direction: Direction, by: Movement) -> Self {
        let _ = (direction, by);
        todo!("WIDGETS-EDIT-03")
    }

    /// Moves the caret one `by` unit; with `select` the base stays (extending the selection).
    pub fn move_caret(&self, direction: Direction, by: Movement, select: bool) -> Self {
        let _ = (direction, by, select);
        todo!("WIDGETS-EDIT-04")
    }

    /// Selects the whole text.
    pub fn select_all(&self) -> Self {
        todo!("WIDGETS-EDIT-05")
    }
}

/// The undo and redo stacks and the coalescing state (WIDGETS-EDIT-06, -07).
#[derive(Clone, Debug, Default)]
struct History {
    undo: Vec<EditingValue>,
    redo: Vec<EditingValue>,
}

/// A text field's value shared between the app and the widget, with undo (Flutter's
/// `TextEditingController` plus `UndoHistory`). `Copy`; create it inside a runtime.
#[derive(Clone, Copy)]
pub struct TextEditingController {
    value: Signal<EditingValue>,
    history: Signal<History>,
}

impl TextEditingController {
    /// A controller holding `text` with the caret at its end.
    pub fn new(text: impl Into<String>) -> Self {
        TextEditingController {
            value: signal(EditingValue::new(text)),
            history: signal(History::default()),
        }
    }

    /// The value (a tracked read).
    pub fn value(&self) -> EditingValue {
        self.value.get()
    }

    /// The text (a tracked read).
    pub fn text(&self) -> String {
        self.value.with(|v| v.text.clone())
    }

    /// Replaces the value (an undoable step of its own); clamps the selection.
    pub fn set_value(&self, value: EditingValue) {
        let _ = value;
        todo!("WIDGETS-EDIT-06")
    }

    /// `set_value` with `text` and the caret at its end.
    pub fn set_text(&self, text: impl Into<String>) {
        self.set_value(EditingValue::new(text));
    }

    /// Applies an edit produced from the current value, recording it for undo.
    pub fn edit(&self, f: impl FnOnce(&EditingValue) -> EditingValue) {
        let _ = f;
        todo!("WIDGETS-EDIT-06")
    }

    /// Changes the selection only (not undoable, but it ends a coalescing group).
    pub fn select(&self, selection: TextSelection) {
        let _ = selection;
        todo!("WIDGETS-EDIT-07")
    }

    /// Restores the previous value; false when there is none.
    pub fn undo(&self) -> bool {
        todo!("WIDGETS-EDIT-06")
    }

    /// Reapplies an undone value; false when there is none.
    pub fn redo(&self) -> bool {
        todo!("WIDGETS-EDIT-06")
    }

    /// True when `undo` would do something.
    pub fn can_undo(&self) -> bool {
        self.history.with(|h| !h.undo.is_empty())
    }

    /// True when `redo` would do something.
    pub fn can_redo(&self) -> bool {
        self.history.with(|h| !h.redo.is_empty())
    }
}
