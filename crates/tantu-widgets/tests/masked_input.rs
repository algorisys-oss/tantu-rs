//! Tests for `docs/specs/widgets/masked-input.md`, rules WIDGETS-MASK-01..06.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::{ImeEvent, Keyed, LogicalKey, Modifiers, NamedKey, SemanticsAction};
use tantu_widgets::{
    Column, EditingValue, FilteringFormatter, LengthLimitingFormatter, MaskFormatter,
    TextEditingController, TextField, TextFormField, TextInputFormatter, TextSelection,
};

type Shared = Rc<RefCell<Option<TextEditingController>>>;

/// A 400 × 200 window with a field keyed "f" built by `make`.
fn field(
    make: impl FnOnce(TextField) -> TextField + 'static,
) -> (WidgetTester, TextEditingController) {
    let out: Shared = Rc::default();
    let make = Cell::new(Some(make));
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(400.0, 200.0, move || {
            let controller = TextEditingController::new("");
            *out.borrow_mut() = Some(controller);
            let make = make.take().expect("built once");
            Column::new().child(Keyed::new("f", make(TextField::new(controller))))
        })
    };
    let controller = out.borrow().expect("built");
    tester.focus(&Finder::key("f"));
    (tester, controller)
}

fn type_text(tester: &mut WidgetTester, text: &str) {
    for ch in text.chars() {
        tester.press_key(
            LogicalKey::Character(ch.to_string().into()),
            Modifiers::default(),
        );
    }
}

/// The platform's primary modifier for chords: Command on macOS, Control elsewhere.
fn primary() -> Modifiers {
    Modifiers {
        control: !cfg!(target_os = "macos"),
        super_key: cfg!(target_os = "macos"),
        ..Modifiers::default()
    }
}

/// `value` with the caret at `offset`.
fn at(text: &str, offset: usize) -> EditingValue {
    EditingValue::new(text).with_selection(TextSelection::collapsed(offset))
}

/// Types `text` into an empty value through `f`, one character at a time.
fn typed(f: &impl TextInputFormatter, text: &str) -> EditingValue {
    let mut value = EditingValue::default();
    for ch in text.chars() {
        let next = value.insert(&ch.to_string());
        value = f.format(&value, next);
    }
    value
}

#[test]
fn widgets_mask_01_formatters_run_on_user_edits() {
    let calls = Rc::new(Cell::new(0));
    let changes: Rc<RefCell<Vec<String>>> = Rc::default();
    let (mut t, c) = {
        let (calls, changes) = (calls.clone(), changes.clone());
        field(move |f| {
            f.input_formatter(move |_: &EditingValue, mut new: EditingValue| {
                calls.set(calls.get() + 1);
                new.text = new.text.to_uppercase();
                new
            })
            .on_change(move |text| changes.borrow_mut().push(text))
        })
    };
    type_text(&mut t, "ab");
    assert_eq!(t.enter(|| c.text()), "AB");
    assert_eq!(*changes.borrow(), ["A", "AB"]);
    assert_eq!(calls.get(), 2);
    // A composition isn't formatted; its commit is.
    t.ime(ImeEvent::Preedit {
        text: "x".into(),
        cursor: None,
    });
    assert_eq!(t.enter(|| c.text()), "ABx");
    assert_eq!(calls.get(), 2);
    t.ime(ImeEvent::Commit("x".into()));
    assert_eq!(t.enter(|| c.text()), "ABX");
    assert_eq!(calls.get(), 3);
    // Moving the caret isn't formatted.
    t.press_key(LogicalKey::Named(NamedKey::ArrowLeft), Modifiers::default());
    assert_eq!(calls.get(), 3);
    // Paste and the SetValue action are.
    t.set_clipboard_text("cd");
    t.press_key(LogicalKey::Character("v".into()), primary());
    assert_eq!(t.enter(|| c.text()), "ABCDX");
    assert!(t.perform_action(&Finder::key("f"), SemanticsAction::SetValue("yo".into())));
    assert_eq!(t.enter(|| c.text()), "YO");
    // Text the app sets isn't.
    t.enter(|| c.set_text("lower"));
    t.pump();
    assert_eq!(t.enter(|| c.text()), "lower");

    // A rejected edit reports nothing; formatters chain in order.
    let changes: Rc<RefCell<Vec<String>>> = Rc::default();
    let (mut t, c) = {
        let changes = changes.clone();
        field(move |f| {
            f.input_formatter(FilteringFormatter::digits_only())
                .input_formatter(LengthLimitingFormatter::new(3))
                .on_change(move |text| changes.borrow_mut().push(text))
        })
    };
    type_text(&mut t, "1a2345");
    assert_eq!(t.enter(|| c.text()), "123");
    assert_eq!(*changes.borrow(), ["1", "12", "123"]);

    // TextFormField forwards its formatters.
    let out: Shared = Rc::default();
    let mut t = {
        let out = out.clone();
        WidgetTester::with_size(400.0, 200.0, move || {
            let controller = TextEditingController::new("");
            *out.borrow_mut() = Some(controller);
            Keyed::new(
                "f",
                TextFormField::new(controller)
                    .label("Code")
                    .input_formatter(FilteringFormatter::digits_only()),
            )
        })
    };
    let c = out.borrow().expect("built");
    t.pump();
    let input = t
        .a11y_tree()
        .nodes
        .into_iter()
        .find(|(_, n)| {
            n.role() == tantu_test::accesskit::Role::TextInput && n.label() == Some("Code")
        })
        .map(|(id, _)| tantu_view::ElementId::from_raw(id.0).expect("element"))
        .expect("the field");
    let center = t.rect(input).center();
    for kind in [
        tantu_view::PointerKind::Down(tantu_view::PointerButton::Primary),
        tantu_view::PointerKind::Up(tantu_view::PointerButton::Primary),
    ] {
        t.pointer(tantu_view::PointerEvent {
            kind,
            position: center,
            modifiers: Modifiers::default(),
        });
    }
    t.pump();
    type_text(&mut t, "a1");
    assert_eq!(t.enter(|| c.text()), "1");
}

#[test]
fn widgets_mask_02_filtering_and_length() {
    let digits = FilteringFormatter::digits_only();
    let out = digits.format(&at("12", 2), at("1a2b", 4));
    assert_eq!(out.text, "12");
    assert_eq!(out.selection, TextSelection::collapsed(2));
    let out = digits.format(
        &at("", 0),
        EditingValue::new("x1y2z3").with_selection(TextSelection::new(1, 5)),
    );
    assert_eq!(out.text, "123");
    assert_eq!(out.selection, TextSelection::new(0, 2));
    let no_spaces = FilteringFormatter::deny(|c| c == ' ');
    assert_eq!(no_spaces.format(&at("", 0), at("a b", 3)).text, "ab");
    let allow = FilteringFormatter::allow(|c| c.is_alphabetic());
    assert_eq!(allow.format(&at("", 0), at("é1à", 4)).text, "éà");

    let limit = LengthLimitingFormatter::new(3);
    let out = limit.format(&at("abc", 3), at("abcd", 4));
    assert_eq!(out.text, "abc");
    assert_eq!(out.selection, TextSelection::collapsed(3));
    assert_eq!(limit.format(&at("a", 1), at("ab", 2)), at("ab", 2));
    let out = LengthLimitingFormatter::new(2).format(&at("", 0), at("éàü", 6));
    assert_eq!(out.text, "éà");
    assert_eq!(out.selection, TextSelection::collapsed(4));
}

#[test]
fn widgets_mask_03_mask_syntax() {
    let letters = MaskFormatter::new("AA-##");
    assert_eq!(typed(&letters, "ab12").text, "ab-12");
    assert_eq!(
        typed(&letters, "1ab").text,
        "ab",
        "a digit doesn't fill a letter slot"
    );
    let any = MaskFormatter::new("**-**");
    assert_eq!(typed(&any, "a1b2").text, "a1-b2");
    let escaped = MaskFormatter::new("\\#-##");
    assert_eq!(typed(&escaped, "12").text, "#-12");
    let trailing = MaskFormatter::new("##\\");
    assert_eq!(typed(&trailing, "123").text, "12");
}

#[test]
fn widgets_mask_04_formatting() {
    let phone = MaskFormatter::new("(###) ###-####");
    assert_eq!(typed(&phone, "5").text, "(5");
    assert_eq!(typed(&phone, "555").text, "(555");
    assert_eq!(typed(&phone, "5551").text, "(555) 1");
    assert_eq!(typed(&phone, "5551234567").text, "(555) 123-4567");
    assert_eq!(
        typed(&phone, "55512345678").text,
        "(555) 123-4567",
        "overflow dropped"
    );
    assert_eq!(typed(&phone, "x").text, "");
    let pasted = phone.format(&at("", 0), at("555.123.4567", 12));
    assert_eq!(pasted.text, "(555) 123-4567");
    let letter = phone.format(&at("(555", 4), at("(555x", 5));
    assert_eq!(letter, at("(555", 4));
    // A literal that a slot would accept is consumed as the literal.
    let prefixed = MaskFormatter::new("+1 ###");
    assert_eq!(typed(&prefixed, "2").text, "+1 2");
    assert_eq!(prefixed.format(&at("", 0), at("+1 23", 5)).text, "+1 23");
}

#[test]
fn widgets_mask_05_caret_and_deleting_literals() {
    let phone = MaskFormatter::new("(###) ###-####");
    assert_eq!(typed(&phone, "5551").selection, TextSelection::collapsed(7));
    // Typing in the middle keeps the caret after the typed character.
    let old = at("(555) 123", 2);
    let out = phone.format(&old, old.insert("9"));
    assert_eq!(out.text, "(595) 512-3");
    assert_eq!(out.selection, TextSelection::collapsed(3));
    // Backspace over the separator removes the digit before it.
    let old = at("(555) 123", 6);
    let out = phone.format(&old, at("(555)123", 5));
    assert_eq!(out.text, "(551) 23");
    assert_eq!(out.selection, TextSelection::collapsed(3));
    // Replacing a selection.
    let old = EditingValue::new("(555) 1234").select_all();
    let out = phone.format(&old, old.insert("7"));
    assert_eq!(out, at("(7", 2));
    // Deleting the only digit empties the field.
    let out = phone.format(&at("(5", 2), at("(", 1));
    assert_eq!(out, at("", 0));
    // Both ends of a selection map.
    let out = phone.format(
        &at("", 0),
        EditingValue::new("5551234").with_selection(TextSelection::new(1, 4)),
    );
    assert_eq!(out.text, "(555) 123-4");
    assert_eq!(out.selection, TextSelection::new(2, 7));
}

#[test]
fn widgets_mask_06_unmask_and_validate() {
    let phone = MaskFormatter::new("(###) ###-####");
    assert_eq!(phone.unmask("(555) 123-4567"), "5551234567");
    assert_eq!(phone.unmask("(555"), "555");
    assert_eq!(phone.unmask(""), "");
    assert!(phone.is_complete("(555) 123-4567"));
    assert!(!phone.is_complete("(555) 123"));
    let validate = phone.validator("Incomplete");
    assert_eq!(validate(""), None);
    assert_eq!(validate("(555").as_deref(), Some("Incomplete"));
    assert_eq!(validate("(555) 123-4567"), None);
}
