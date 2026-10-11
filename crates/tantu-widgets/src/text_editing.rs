//! The text editing model: [`EditingValue`] and its edits, and [`TextEditingController`].
//! Spec: `docs/specs/widgets/text-editing.md`.

use tantu_view::reactive::{Signal, signal};
use unicode_segmentation::UnicodeSegmentation;

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

    /// The value with `selection`, clamped to the text's character boundaries
    /// (WIDGETS-EDIT-01).
    pub fn with_selection(mut self, selection: TextSelection) -> Self {
        self.selection = TextSelection::new(
            boundary_at_or_before(&self.text, selection.base),
            boundary_at_or_before(&self.text, selection.extent),
        );
        self
    }

    /// The selected text (empty when collapsed).
    pub fn selected_text(&self) -> &str {
        &self.text[self.selection.start()..self.selection.end()]
    }

    /// Replaces the selection with `text`; the caret ends after it (WIDGETS-EDIT-02).
    pub fn insert(&self, text: &str) -> Self {
        self.replace(self.selection.start(), self.selection.end(), text)
    }

    /// Deletes the selection, or one `by` unit in `direction` from a collapsed caret
    /// (WIDGETS-EDIT-03).
    pub fn delete(&self, direction: Direction, by: Movement) -> Self {
        if !self.selection.is_collapsed() {
            return self.replace(self.selection.start(), self.selection.end(), "");
        }
        let caret = self.selection.extent;
        let target = self.boundary(caret, direction, by);
        if target == caret {
            return self.clone();
        }
        self.replace(target.min(caret), target.max(caret), "")
    }

    /// Moves the caret one `by` unit; with `select` the base stays (extending the selection)
    /// (WIDGETS-EDIT-04).
    pub fn move_caret(&self, direction: Direction, by: Movement, select: bool) -> Self {
        let mut moved = self.clone();
        if select {
            moved.selection.extent = self.boundary(self.selection.extent, direction, by);
        } else if !self.selection.is_collapsed() && by == Movement::Grapheme {
            moved.selection = TextSelection::collapsed(match direction {
                Direction::Backward => self.selection.start(),
                Direction::Forward => self.selection.end(),
            });
        } else {
            moved.selection =
                TextSelection::collapsed(self.boundary(self.selection.extent, direction, by));
        }
        moved
    }

    /// Replaces the composing region (or the selection, when there is none) with `text`,
    /// marks it as composing, and puts the caret at `cursor` within it (its end when `None`)
    /// (WIDGETS-EDIT-09).
    pub fn compose(&self, text: &str, cursor: Option<(usize, usize)>) -> Self {
        let _ = (text, cursor);
        todo!("WIDGETS-EDIT-09")
    }

    /// Replaces the composing region (or the selection) with `text` and clears it
    /// (WIDGETS-EDIT-09).
    pub fn commit(&self, text: &str) -> Self {
        let _ = text;
        todo!("WIDGETS-EDIT-09")
    }

    /// Selects the whole text (WIDGETS-EDIT-05).
    pub fn select_all(&self) -> Self {
        self.clone()
            .with_selection(TextSelection::new(0, self.text.len()))
    }

    /// The text with `start..end` replaced by `with`, the caret after it and no composing
    /// region.
    fn replace(&self, start: usize, end: usize, with: &str) -> Self {
        let mut text = String::with_capacity(self.text.len() + with.len());
        text.push_str(&self.text[..start]);
        text.push_str(with);
        text.push_str(&self.text[end..]);
        EditingValue {
            text,
            selection: TextSelection::collapsed(start + with.len()),
            composing: None,
        }
    }

    /// The offset one `by` unit from `from` in `direction`, stopping at the text's ends.
    fn boundary(&self, from: usize, direction: Direction, by: Movement) -> usize {
        let text = self.text.as_str();
        let (before, after) = (&text[..from], &text[from..]);
        match (by, direction) {
            (Movement::Grapheme, Direction::Backward) => before
                .grapheme_indices(true)
                .next_back()
                .map_or(0, |(index, _)| index),
            (Movement::Grapheme, Direction::Forward) => after
                .graphemes(true)
                .next()
                .map_or(from, |grapheme| from + grapheme.len()),
            (Movement::Word, Direction::Backward) => before
                .split_word_bound_indices()
                .rev()
                .find(|(_, segment)| is_word(segment))
                .map_or(0, |(index, _)| index),
            (Movement::Word, Direction::Forward) => after
                .split_word_bound_indices()
                .find(|(_, segment)| is_word(segment))
                .map_or(text.len(), |(index, segment)| from + index + segment.len()),
            (Movement::LineBoundary, Direction::Backward) => {
                before.rfind('\n').map_or(0, |index| index + 1)
            }
            (Movement::LineBoundary, Direction::Forward) => {
                after.find('\n').map_or(text.len(), |index| from + index)
            }
            (Movement::Document, Direction::Backward) => 0,
            (Movement::Document, Direction::Forward) => text.len(),
        }
    }
}

/// `offset` clamped to the text and moved back to a character boundary.
fn boundary_at_or_before(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

/// A UAX #29 segment counts as a word when it has an alphanumeric character.
fn is_word(segment: &str) -> bool {
    segment.chars().any(char::is_alphanumeric)
}

/// What `next` inserted at `current`'s caret, if it is exactly one grapheme typed there
/// (WIDGETS-EDIT-07); `None` for any other change.
fn typed_grapheme<'a>(current: &EditingValue, next: &'a EditingValue) -> Option<&'a str> {
    if !current.selection.is_collapsed() || next.text.len() <= current.text.len() {
        return None;
    }
    let caret = current.selection.extent;
    let inserted_len = next.text.len() - current.text.len();
    let (head, rest) = current.text.split_at(caret);
    let inserted = next.text.get(caret..caret + inserted_len)?;
    let matches = next.text.starts_with(head)
        && next.text.ends_with(rest)
        && next.selection == TextSelection::collapsed(caret + inserted_len)
        && inserted.graphemes(true).count() == 1;
    matches.then_some(inserted)
}

/// The undo and redo stacks and the coalescing state (WIDGETS-EDIT-06, -07).
#[derive(Clone, Debug, Default)]
struct History {
    undo: Vec<EditingValue>,
    redo: Vec<EditingValue>,
    /// True while single-grapheme inserts are being coalesced into the last undo step.
    typing: bool,
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

    /// Replaces the value (an undoable step of its own); clamps the selection
    /// (WIDGETS-EDIT-06).
    pub fn set_value(&self, value: EditingValue) {
        let selection = value.selection;
        let value = value.with_selection(selection);
        let current = self.value.get_untracked();
        self.history.update(|h| {
            h.undo.push(current);
            h.redo.clear();
            h.typing = false;
        });
        self.value.set(value);
    }

    /// `set_value` with `text` and the caret at its end.
    pub fn set_text(&self, text: impl Into<String>) {
        self.set_value(EditingValue::new(text));
    }

    /// Applies an edit produced from the current value, recording it for undo
    /// (WIDGETS-EDIT-06, -07). An edit that leaves the text alone records no step.
    pub fn edit(&self, f: impl FnOnce(&EditingValue) -> EditingValue) {
        let current = self.value.get_untracked();
        let next = f(&current);
        if next == current {
            return;
        }
        if next.text == current.text {
            self.history.update(|h| h.typing = false);
            self.value.set(next);
            return;
        }
        let typed = typed_grapheme(&current, &next);
        let whitespace = typed.is_some_and(|g| g.chars().all(char::is_whitespace));
        self.history.update(|h| {
            if !(typed.is_some() && h.typing) {
                h.undo.push(current.clone());
            }
            h.redo.clear();
            h.typing = typed.is_some() && !whitespace;
        });
        self.value.set(next);
    }

    /// Changes the selection only (not undoable, but it ends a coalescing group).
    pub fn select(&self, selection: TextSelection) {
        self.history.update(|h| h.typing = false);
        let next = self.value.get_untracked().with_selection(selection);
        self.value.set(next);
    }

    /// Restores the previous value; false when there is none.
    pub fn undo(&self) -> bool {
        let current = self.value.get_untracked();
        let previous = self.history.with_untracked(|h| h.undo.last().cloned());
        let Some(previous) = previous else {
            return false;
        };
        self.history.update(|h| {
            h.undo.pop();
            h.redo.push(current);
            h.typing = false;
        });
        self.value.set(previous);
        true
    }

    /// Reapplies an undone value; false when there is none.
    pub fn redo(&self) -> bool {
        let current = self.value.get_untracked();
        let next = self.history.with_untracked(|h| h.redo.last().cloned());
        let Some(next) = next else {
            return false;
        };
        self.history.update(|h| {
            h.redo.pop();
            h.undo.push(current);
            h.typing = false;
        });
        self.value.set(next);
        true
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
