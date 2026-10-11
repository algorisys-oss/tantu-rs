//! Tests for `docs/specs/widgets/text-editing.md`, rules WIDGETS-EDIT-01..08.

use std::cell::Cell;
use std::rc::Rc;

use tantu_view::reactive::{Runtime, effect};
use tantu_widgets::{
    Direction, EditingValue, Movement, TextEditingController, TextRange, TextSelection,
};

use Direction::{Backward, Forward};
use Movement::{Document, Grapheme, LineBoundary, Word};

fn at(text: &str, caret: usize) -> EditingValue {
    EditingValue::new(text).with_selection(TextSelection::collapsed(caret))
}

fn sel(text: &str, base: usize, extent: usize) -> EditingValue {
    EditingValue::new(text).with_selection(TextSelection::new(base, extent))
}

fn caret(value: &EditingValue) -> usize {
    assert!(value.selection.is_collapsed(), "{:?}", value.selection);
    value.selection.extent
}

#[test]
fn widgets_edit_01_selection_and_value_basics() {
    let c = TextSelection::collapsed(3);
    assert_eq!((c.base, c.extent), (3, 3));
    assert!(c.is_collapsed());
    let s = TextSelection::new(4, 1);
    assert!(!s.is_collapsed());
    assert_eq!((s.start(), s.end()), (1, 4));
    assert_eq!(TextSelection::new(1, 4).start(), 1);
    let v = EditingValue::new("ab");
    assert_eq!(v.selection, TextSelection::collapsed(2));
    assert_eq!(v.composing, None);
    assert_eq!(EditingValue::default(), at("", 0));
    // Clamping to the text and to character boundaries ("é" is two bytes).
    let v = EditingValue::new("héllo").with_selection(TextSelection::new(2, 10));
    assert_eq!(v.selection, TextSelection::new(1, 6));
    let v = EditingValue::new("héllo").with_selection(TextSelection::new(3, 1));
    assert_eq!(v.selection, TextSelection::new(3, 1));
}

#[test]
fn widgets_edit_02_insert() {
    let v = at("hello", 2).insert("XY");
    assert_eq!((v.text.as_str(), caret(&v)), ("heXYllo", 4));
    let v = sel("hello", 1, 4).insert("XY");
    assert_eq!((v.text.as_str(), caret(&v)), ("hXYo", 3));
    let v = sel("hello", 4, 1).insert("");
    assert_eq!((v.text.as_str(), caret(&v)), ("ho", 1));
    let mut composing = at("hello", 5);
    composing.composing = Some(TextRange { start: 0, end: 2 });
    assert_eq!(composing.insert("!").composing, None);
    let v = EditingValue::new("hello").select_all().insert("bye");
    assert_eq!((v.text.as_str(), caret(&v)), ("bye", 3));
}

#[test]
fn widgets_edit_03_delete() {
    // A selection is deleted whatever the direction and unit.
    for (direction, by) in [(Backward, Grapheme), (Forward, Word), (Backward, Document)] {
        let v = sel("hello", 4, 1).delete(direction, by);
        assert_eq!(
            (v.text.as_str(), caret(&v)),
            ("ho", 1),
            "{direction:?} {by:?}"
        );
    }
    let v = at("hello", 3).delete(Backward, Grapheme);
    assert_eq!((v.text.as_str(), caret(&v)), ("helo", 2));
    let v = at("hello", 3).delete(Forward, Grapheme);
    assert_eq!((v.text.as_str(), caret(&v)), ("helo", 3));
    // Grapheme clusters go as one.
    let v = at("e\u{301}x", 3).delete(Backward, Grapheme);
    assert_eq!((v.text.as_str(), caret(&v)), ("x", 0));
    let thumbs = "\u{1F44D}\u{1F3FD}a";
    let v = at(thumbs, 8).delete(Backward, Grapheme);
    assert_eq!((v.text.as_str(), caret(&v)), ("a", 0));
    let v = at(thumbs, 0).delete(Forward, Grapheme);
    assert_eq!((v.text.as_str(), caret(&v)), ("a", 0));
    // Words.
    let v = at("hello big world", 15).delete(Backward, Word);
    assert_eq!((v.text.as_str(), caret(&v)), ("hello big ", 10));
    let v = at("hello big world", 0).delete(Forward, Word);
    assert_eq!((v.text.as_str(), caret(&v)), (" big world", 0));
    let v = at("hello big world", 5).delete(Forward, Word);
    assert_eq!((v.text.as_str(), caret(&v)), ("hello world", 5));
    let v = at("hello big world", 10).delete(Backward, Word);
    assert_eq!((v.text.as_str(), caret(&v)), ("hello world", 6));
    // Lines and the document.
    let v = at("ab\ncd", 4).delete(Backward, LineBoundary);
    assert_eq!((v.text.as_str(), caret(&v)), ("ab\nd", 3));
    let v = at("ab\ncd", 3).delete(Forward, LineBoundary);
    assert_eq!((v.text.as_str(), caret(&v)), ("ab\n", 3));
    let v = at("hello", 2).delete(Backward, Document);
    assert_eq!((v.text.as_str(), caret(&v)), ("llo", 0));
    let v = at("hello", 2).delete(Forward, Document);
    assert_eq!((v.text.as_str(), caret(&v)), ("he", 2));
    // The edges.
    assert_eq!(at("hello", 0).delete(Backward, Grapheme), at("hello", 0));
    assert_eq!(at("hello", 0).delete(Backward, Word), at("hello", 0));
    assert_eq!(at("hello", 5).delete(Forward, Grapheme), at("hello", 5));
    assert_eq!(at("hello", 5).delete(Forward, Document), at("hello", 5));
    let mut composing = at("hello", 5);
    composing.composing = Some(TextRange { start: 0, end: 2 });
    assert_eq!(composing.delete(Backward, Grapheme).composing, None);
}

#[test]
fn widgets_edit_04_move_caret() {
    // A selection collapses to its start or end for graphemes, else moves from the extent.
    assert_eq!(
        sel("hello world", 2, 5)
            .move_caret(Backward, Grapheme, false)
            .selection,
        TextSelection::collapsed(2)
    );
    assert_eq!(
        sel("hello world", 5, 2)
            .move_caret(Forward, Grapheme, false)
            .selection,
        TextSelection::collapsed(5)
    );
    assert_eq!(
        sel("hello world", 2, 5)
            .move_caret(Backward, Word, false)
            .selection,
        TextSelection::collapsed(0)
    );
    assert_eq!(
        sel("hello world", 2, 5)
            .move_caret(Forward, Word, false)
            .selection,
        TextSelection::collapsed(11)
    );
    // Graphemes.
    assert_eq!(
        caret(&at("héllo", 3).move_caret(Backward, Grapheme, false)),
        1
    );
    assert_eq!(
        caret(&at("héllo", 1).move_caret(Forward, Grapheme, false)),
        3
    );
    // Words, stopping at the ends.
    let mut v = at("hello big world", 0);
    let mut stops = Vec::new();
    for _ in 0..4 {
        v = v.move_caret(Forward, Word, false);
        stops.push(caret(&v));
    }
    assert_eq!(stops, [5, 9, 15, 15]);
    let mut stops = Vec::new();
    for _ in 0..4 {
        v = v.move_caret(Backward, Word, false);
        stops.push(caret(&v));
    }
    assert_eq!(stops, [10, 6, 0, 0]);
    // Lines and the document.
    assert_eq!(
        caret(&at("ab\ncd", 4).move_caret(Backward, LineBoundary, false)),
        3
    );
    assert_eq!(
        caret(&at("ab\ncd", 4).move_caret(Forward, LineBoundary, false)),
        5
    );
    assert_eq!(
        caret(&at("ab\ncd", 1).move_caret(Forward, LineBoundary, false)),
        2
    );
    assert_eq!(
        caret(&at("ab\ncd", 1).move_caret(Backward, LineBoundary, false)),
        0
    );
    assert_eq!(
        caret(&at("ab\ncd", 4).move_caret(Backward, Document, false)),
        0
    );
    assert_eq!(
        caret(&at("ab\ncd", 1).move_caret(Forward, Document, false)),
        5
    );
    // Selecting keeps the base.
    let v = at("hello", 2).move_caret(Forward, Grapheme, true);
    assert_eq!(v.selection, TextSelection::new(2, 3));
    let v = v.move_caret(Backward, Word, true);
    assert_eq!(v.selection, TextSelection::new(2, 0));
    // Nothing else changes.
    let mut composing = at("hello", 2);
    composing.composing = Some(TextRange { start: 0, end: 2 });
    let moved = composing.move_caret(Forward, Grapheme, false);
    assert_eq!(
        (moved.text.as_str(), moved.composing),
        ("hello", composing.composing)
    );
}

#[test]
fn widgets_edit_05_select_all_and_selected_text() {
    let v = EditingValue::new("héllo").select_all();
    assert_eq!(v.selection, TextSelection::new(0, 6));
    assert_eq!(v.selected_text(), "héllo");
    assert_eq!(sel("hello", 4, 1).selected_text(), "ell");
    assert_eq!(at("hello", 2).selected_text(), "");
}

#[test]
fn widgets_edit_06_controller_undo_redo() {
    Runtime::new().enter(|| {
        let c = TextEditingController::new("ab");
        assert_eq!(c.text(), "ab");
        assert!(!c.can_undo() && !c.can_redo());
        assert!(!c.undo() && !c.redo());
        c.edit(|v| v.insert("c"));
        assert_eq!(c.text(), "abc");
        assert!(c.can_undo() && !c.can_redo());
        assert!(c.undo());
        assert_eq!(c.value(), EditingValue::new("ab"));
        assert!(c.can_redo());
        assert!(c.redo());
        assert_eq!(c.value(), EditingValue::new("abc"));
        assert!(!c.redo());
        // A new edit clears the redo stack; the selection is part of a step.
        c.undo();
        c.edit(|v| {
            v.clone()
                .with_selection(TextSelection::collapsed(0))
                .insert("z")
        });
        assert_eq!(c.value(), at("zab", 1));
        assert!(!c.can_redo());
        c.undo();
        assert_eq!(c.value(), EditingValue::new("ab"));
        // Programmatic values are steps of their own, and clamp the selection.
        c.set_text("hello");
        c.set_value(EditingValue::new("hé").with_selection(TextSelection::new(2, 9)));
        assert_eq!(c.value().selection, TextSelection::new(1, 3));
        assert!(c.undo());
        assert_eq!(c.text(), "hello");
        assert!(c.undo());
        assert_eq!(c.text(), "ab");
    });
}

#[test]
fn widgets_edit_07_coalescing() {
    Runtime::new().enter(|| {
        let c = TextEditingController::new("");
        for ch in ["a", "b", " ", "c", "d"] {
            c.edit(|v| v.insert(ch));
        }
        assert_eq!(c.text(), "ab cd");
        assert!(c.undo());
        assert_eq!(c.text(), "ab ");
        assert!(c.undo());
        assert_eq!(c.text(), "");
        assert!(!c.undo());
        assert!(c.redo());
        assert_eq!(c.text(), "ab ");
        // Typing after a redo starts a new group and clears the rest of the redo stack.
        c.edit(|v| v.insert("x"));
        assert!(!c.can_redo());
        assert!(c.undo());
        assert_eq!(c.text(), "ab ");
        // A deletion is its own step and ends the group.
        let c = TextEditingController::new("");
        c.edit(|v| v.insert("a"));
        c.edit(|v| v.insert("b"));
        c.edit(|v| v.delete(Backward, Grapheme));
        c.edit(|v| v.insert("c"));
        assert_eq!(c.text(), "ac");
        c.undo();
        assert_eq!(c.text(), "a");
        c.undo();
        assert_eq!(c.text(), "ab");
        c.undo();
        assert_eq!(c.text(), "");
        // Selecting ends the group; the restored value has the selection of the time.
        let c = TextEditingController::new("");
        c.edit(|v| v.insert("a"));
        c.select(TextSelection::collapsed(0));
        c.edit(|v| v.insert("b"));
        assert_eq!(c.text(), "ba");
        c.undo();
        assert_eq!(c.value(), at("a", 0));
        c.undo();
        assert_eq!(c.text(), "");
        // A multi-grapheme insert is its own step.
        let c = TextEditingController::new("");
        c.edit(|v| v.insert("a"));
        c.edit(|v| v.insert("xy"));
        c.edit(|v| v.insert("b"));
        c.undo();
        assert_eq!(c.text(), "axy");
        c.undo();
        assert_eq!(c.text(), "a");
        // An edit that changes nothing records nothing.
        let c = TextEditingController::new("a");
        c.edit(|v| v.delete(Forward, Grapheme));
        assert!(!c.can_undo());
    });
}

#[test]
fn widgets_edit_08_reactive() {
    Runtime::new().enter(|| {
        let c = TextEditingController::new("a");
        let runs: Rc<Cell<u32>> = Rc::default();
        let seen: Rc<Cell<usize>> = Rc::default();
        {
            let (runs, seen) = (runs.clone(), seen.clone());
            effect(move || {
                runs.set(runs.get() + 1);
                seen.set(c.text().len() + c.value().selection.extent);
            });
        }
        assert_eq!(runs.get(), 1);
        c.edit(|v| v.insert("b"));
        assert_eq!((runs.get(), seen.get()), (2, 4));
        c.select(TextSelection::collapsed(0));
        assert_eq!((runs.get(), seen.get()), (3, 2));
        c.undo();
        assert_eq!((runs.get(), seen.get()), (4, 2));
        c.redo();
        assert_eq!((runs.get(), seen.get()), (5, 4));
        c.set_text("xyz");
        assert_eq!((runs.get(), seen.get()), (6, 6));
        // `value()` inside `edit` sees the current value.
        c.edit(|v| {
            assert_eq!(c.value(), *v);
            v.insert("!")
        });
        assert_eq!(c.text(), "xyz!");
    });
}
