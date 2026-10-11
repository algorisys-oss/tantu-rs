//! Tests for `docs/specs/widgets/text-field.md`, rules WIDGETS-TEXTFIELD-01..11.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Color, Point, Rect, Size};
use tantu_view::layout::{BoxConstraints, NoTextMeasure};
use tantu_view::reactive::signal;
use tantu_view::scene::{Command, Scene};
use tantu_view::{
    CursorIcon, ElementId, ImeEvent, Key, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton,
    PointerEvent, PointerKind, ViewTree,
};
use tantu_widgets::{
    Button, Column, Row, TextEditingController, TextField, TextRange, TextSelection,
};

const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);
const OUTLINE: Color = Color::from_argb32(0xFF79_747E);
const ON_SURFACE: Color = Color::from_argb32(0xFF1C_1B1F);
const ON_SURFACE_VARIANT: Color = Color::from_argb32(0xFF49_454F);
const ON_SURFACE_38: Color = Color::from_argb32(0x611C_1B1F);
const ON_SURFACE_12: Color = Color::from_argb32(0x1F1C_1B1F);

type Shared = Rc<RefCell<Option<TextEditingController>>>;

/// A 400 × 200 window with a field keyed "f" built by `make` from a controller over `text`.
fn field(
    text: &str,
    make: impl FnOnce(TextField) -> TextField + 'static,
) -> (WidgetTester, TextEditingController) {
    let out: Shared = Rc::default();
    let text = text.to_string();
    let make = Cell::new(Some(make));
    let tester = {
        let out = out.clone();
        WidgetTester::with_size(400.0, 200.0, move || {
            let controller = TextEditingController::new(text);
            *out.borrow_mut() = Some(controller);
            let make = make.take().expect("built once");
            Column::new()
                .child(Keyed::new("f", make(TextField::new(controller))))
                .child(Button::new("Next"))
        })
    };
    let controller = out.borrow().expect("built");
    (tester, controller)
}

fn plain(text: &str) -> (WidgetTester, TextEditingController) {
    field(text, |f| f)
}

fn key(tester: &mut WidgetTester, key: NamedKey, modifiers: Modifiers) {
    tester.press_key(LogicalKey::Named(key), modifiers);
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

fn control() -> Modifiers {
    Modifiers {
        control: true,
        ..Modifiers::default()
    }
}

/// The fills of the field: (rect, color).
fn fills(tester: &WidgetTester) -> Vec<(Rect, Color)> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { shape, color } => Some((shape.rect, color)),
            _ => None,
        })
        .collect()
}

/// The caret fill (2 px wide, primary), if painted.
fn caret(tester: &WidgetTester) -> Option<Rect> {
    fills(tester)
        .into_iter()
        .find(|(r, c)| r.width() == 2.0 && *c == PRIMARY)
        .map(|(r, _)| r)
}

fn selection(tester: &WidgetTester) -> Vec<Rect> {
    fills(tester)
        .into_iter()
        .filter(|(_, c)| *c == PRIMARY.with_alpha(0.4))
        .map(|(r, _)| r)
        .collect()
}

/// The outline stroke: (width, color).
fn outline(tester: &WidgetTester) -> Option<(f32, Color)> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .find_map(|e| match e.command {
            Command::Stroke { width, color, .. } => Some((width, color)),
            _ => None,
        })
}

/// The color of the first glyph run of the element.
fn glyph_color(tester: &WidgetTester, element: ElementId) -> Option<Color> {
    tester
        .frame()
        .entries_for(element)
        .find_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(run.color),
            _ => None,
        })
}

fn sel(tester: &WidgetTester, c: TextEditingController) -> TextSelection {
    tester.enter(|| c.value().selection)
}

#[test]
fn widgets_textfield_01_structure() {
    let (mut tester, c) = plain("hello");
    let rect = tester.rect(tester.find(&Finder::key("f")));
    assert_eq!(rect.size(), Size::new(400.0, 40.0));
    let editor = tester.find(&Finder::text("hello"));
    let inner = tester.rect(editor);
    assert_eq!((inner.left, inner.top), (16.0, 8.0));
    tester.enter(|| c.set_text("bye"));
    assert!(tester.pump());
    tester.find(&Finder::text("bye"));
    assert_eq!(
        glyph_color(&tester, tester.find(&Finder::text("bye"))),
        Some(ON_SURFACE)
    );
    // Unbounded width: 280.
    let tester = WidgetTester::with_size(400.0, 200.0, || {
        Row::new().child(Keyed::new(
            "f",
            TextField::new(TextEditingController::new("x")),
        ))
    });
    assert_eq!(tester.rect(tester.find(&Finder::key("f"))).width(), 280.0);
    // The placeholder while empty.
    let (tester, _) = field("", |f| f.placeholder("Name"));
    let placeholder = tester.find(&Finder::text("Name"));
    assert_eq!(glyph_color(&tester, placeholder), Some(ON_SURFACE_VARIANT));
}

#[test]
fn widgets_textfield_02_painting() {
    let (mut tester, c) = plain("hello");
    assert_eq!(outline(&tester), Some((1.0, OUTLINE)));
    assert_eq!(caret(&tester), None);
    assert!(selection(&tester).is_empty());
    tester.focus(&Finder::key("f"));
    assert_eq!(outline(&tester), Some((2.0, PRIMARY)));
    // Scene commands are in the editor's coordinates: the caret sits at the text's end.
    let caret_rect = caret(&tester).expect("a caret");
    let editor = tester.rect(tester.find(&Finder::text("hello")));
    assert!(caret_rect.left > 0.0 && caret_rect.left <= editor.width());
    assert_eq!(caret_rect.top, 0.0);
    assert!(caret_rect.height() > 10.0);
    tester.enter(|| c.select(TextSelection::new(0, 2)));
    assert!(tester.pump());
    let rects = selection(&tester);
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].left, 0.0);
    assert!(rects[0].right > 0.0 && rects[0].right < editor.width());
    // Painted under the text: the selection fill comes before the glyph run.
    let entries = tester.frame().scene.entries();
    let selection_index = entries
        .iter()
        .position(|e| matches!(e.command, Command::Fill { color, .. } if color == PRIMARY.with_alpha(0.4)))
        .expect("selection");
    let glyph_index = entries
        .iter()
        .position(|e| matches!(e.command, Command::GlyphRun(_)))
        .expect("glyphs");
    assert!(selection_index < glyph_index);
    let mut tester = WidgetTester::with_size(240.0, 110.0, || {
        Column::new()
            .child(Keyed::new(
                "a",
                TextField::new(TextEditingController::new("Hello world")),
            ))
            .child(TextField::new(TextEditingController::new("")).placeholder("Search"))
    });
    tester.focus(&Finder::key("a"));
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/text-field.png"
    ));
}

#[test]
fn widgets_textfield_03_focus_and_disabled() {
    let (mut tester, _) = plain("hello");
    key(&mut tester, NamedKey::Tab, Modifiers::default());
    assert!(caret(&tester).is_some());
    let (mut tester, _) = plain("hello");
    tester.tap(&Finder::key("f"));
    assert!(caret(&tester).is_some());
    let (mut tester, c) = field("hello", |f| f.enabled(false));
    assert_eq!(outline(&tester), Some((1.0, ON_SURFACE_12)));
    assert_eq!(
        glyph_color(&tester, tester.find(&Finder::text("hello"))),
        Some(ON_SURFACE_38)
    );
    tester.tap(&Finder::key("f"));
    assert!(caret(&tester).is_none());
    key(&mut tester, NamedKey::Tab, Modifiers::default());
    assert!(caret(&tester).is_none());
    tester.press_key(LogicalKey::Character("x".into()), Modifiers::default());
    assert_eq!(tester.enter(|| c.text()), "hello");
    // A signal re-enables it.
    let out: Shared = Rc::default();
    let on = Rc::new(Cell::new(None));
    let mut tester = {
        let (out, on) = (out.clone(), on.clone());
        WidgetTester::with_size(400.0, 200.0, move || {
            let enabled = signal(false);
            on.set(Some(enabled));
            let controller = TextEditingController::new("hi");
            *out.borrow_mut() = Some(controller);
            Column::new().child(Keyed::new("f", TextField::new(controller).enabled(enabled)))
        })
    };
    let enabled = on.get().expect("built");
    tester.enter(|| enabled.set(true));
    tester.pump();
    tester.tap(&Finder::key("f"));
    assert!(caret(&tester).is_some());
}

#[test]
fn widgets_textfield_04_keys() {
    let changes: Rc<RefCell<Vec<String>>> = Rc::default();
    let submits: Rc<RefCell<Vec<String>>> = Rc::default();
    let (mut tester, c) = {
        let (changes, submits) = (changes.clone(), submits.clone());
        field("", move |f| {
            f.on_change(move |t| changes.borrow_mut().push(t))
                .on_submit(move |t| submits.borrow_mut().push(t))
        })
    };
    tester.focus(&Finder::key("f"));
    for ch in ["a", "b"] {
        tester.press_key(LogicalKey::Character(ch.into()), Modifiers::default());
    }
    assert_eq!(tester.enter(|| c.text()), "ab");
    assert_eq!(changes.borrow().clone(), ["a", "ab"]);
    key(&mut tester, NamedKey::Backspace, Modifiers::default());
    assert_eq!(tester.enter(|| c.text()), "a");
    key(&mut tester, NamedKey::ArrowLeft, Modifiers::default());
    assert_eq!(sel(&tester, c), TextSelection::collapsed(0));
    key(&mut tester, NamedKey::ArrowRight, shift());
    assert_eq!(sel(&tester, c), TextSelection::new(0, 1));
    assert_eq!(changes.borrow().len(), 3);
    key(&mut tester, NamedKey::Delete, Modifiers::default());
    assert_eq!(tester.enter(|| c.text()), "");
    // Words and lines.
    tester.enter(|| c.set_text("hello world"));
    tester.pump();
    key(&mut tester, NamedKey::End, Modifiers::default());
    assert_eq!(sel(&tester, c), TextSelection::collapsed(11));
    key(&mut tester, NamedKey::ArrowLeft, control());
    assert_eq!(sel(&tester, c), TextSelection::collapsed(6));
    key(&mut tester, NamedKey::Home, Modifiers::default());
    assert_eq!(sel(&tester, c), TextSelection::collapsed(0));
    key(&mut tester, NamedKey::End, shift());
    assert_eq!(sel(&tester, c), TextSelection::new(0, 11));
    key(&mut tester, NamedKey::Home, Modifiers::default());
    key(&mut tester, NamedKey::Delete, control());
    assert_eq!(tester.enter(|| c.text()), " world");
    // Enter submits a single-line field.
    key(&mut tester, NamedKey::Enter, Modifiers::default());
    assert_eq!(submits.borrow().clone(), [" world"]);
    assert_eq!(tester.enter(|| c.text()), " world");
    // Tab isn't handled: focus moves on, the caret goes.
    key(&mut tester, NamedKey::Tab, Modifiers::default());
    assert!(caret(&tester).is_none());
    // Multi-line: Enter inserts a newline; Up and Down move between lines at the caret's x.
    let (mut tester, c) = field("", |f| f.max_lines(None));
    tester.focus(&Finder::key("f"));
    for ch in ["a", "b"] {
        tester.press_key(LogicalKey::Character(ch.into()), Modifiers::default());
    }
    key(&mut tester, NamedKey::Enter, Modifiers::default());
    for ch in ["c", "d"] {
        tester.press_key(LogicalKey::Character(ch.into()), Modifiers::default());
    }
    assert_eq!(tester.enter(|| c.text()), "ab\ncd");
    key(&mut tester, NamedKey::ArrowLeft, Modifiers::default());
    assert_eq!(sel(&tester, c), TextSelection::collapsed(4));
    key(&mut tester, NamedKey::ArrowUp, Modifiers::default());
    assert_eq!(sel(&tester, c), TextSelection::collapsed(1));
    key(&mut tester, NamedKey::ArrowDown, shift());
    assert_eq!(sel(&tester, c), TextSelection::new(1, 4));
    key(&mut tester, NamedKey::ArrowUp, Modifiers::default());
    key(&mut tester, NamedKey::ArrowUp, Modifiers::default());
    assert_eq!(sel(&tester, c), TextSelection::collapsed(0));
    key(&mut tester, NamedKey::ArrowDown, Modifiers::default());
    key(&mut tester, NamedKey::ArrowDown, Modifiers::default());
    assert_eq!(sel(&tester, c), TextSelection::collapsed(5));
}

#[test]
fn widgets_textfield_05_chords() {
    let (mut tester, c) = plain("");
    tester.focus(&Finder::key("f"));
    for ch in ["a", "b"] {
        tester.press_key(LogicalKey::Character(ch.into()), Modifiers::default());
    }
    tester.press_key(LogicalKey::Character("a".into()), control());
    assert_eq!(sel(&tester, c), TextSelection::new(0, 2));
    tester.press_key(LogicalKey::Character("z".into()), control());
    assert_eq!(tester.enter(|| c.text()), "");
    tester.press_key(
        LogicalKey::Character("z".into()),
        Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(tester.enter(|| c.text()), "ab");
    tester.press_key(LogicalKey::Character("z".into()), control());
    assert_eq!(tester.enter(|| c.text()), "");
    tester.press_key(LogicalKey::Character("y".into()), control());
    assert_eq!(tester.enter(|| c.text()), "ab");
}

#[test]
fn widgets_textfield_06_pointer() {
    let (mut tester, c) = plain("hello world");
    let editor = tester.rect(tester.find(&Finder::text("hello world")));
    let y = editor.center().y;
    let press = |tester: &mut WidgetTester, x: f32| {
        tester.pointer(PointerEvent {
            kind: PointerKind::Down(PointerButton::Primary),
            position: Point::new(x, y),
        });
        tester.pump();
    };
    let release = |tester: &mut WidgetTester, x: f32| {
        tester.pointer(PointerEvent {
            kind: PointerKind::Up(PointerButton::Primary),
            position: Point::new(x, y),
        });
        tester.pump();
    };
    press(&mut tester, editor.left + 0.5);
    release(&mut tester, editor.left + 0.5);
    assert_eq!(sel(&tester, c), TextSelection::collapsed(0));
    assert!(caret(&tester).is_some());
    press(&mut tester, editor.right - 0.5);
    release(&mut tester, editor.right - 0.5);
    assert_eq!(sel(&tester, c), TextSelection::collapsed(11));
    // Dragging extends from the pressed offset, even outside the field.
    press(&mut tester, editor.left + 0.5);
    tester.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(editor.right + 50.0, y),
    });
    tester.pump();
    assert_eq!(sel(&tester, c), TextSelection::new(0, 11));
    release(&mut tester, editor.right + 50.0);
    assert_eq!(sel(&tester, c), TextSelection::new(0, 11));
    // The cursor over the field.
    let mut tree =
        ViewTree::new(|| Keyed::new("f", TextField::new(TextEditingController::new("x"))));
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::tight(Size::new(400.0, 200.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    tree.dispatch_pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(20.0, 20.0),
    });
    assert_eq!(tree.cursor(), CursorIcon::Text);
}

fn preedit(text: &str) -> ImeEvent {
    ImeEvent::Preedit {
        text: text.into(),
        cursor: None,
    }
}

#[test]
fn widgets_textfield_07_ime_events_and_area() {
    let (mut tester, c) = plain("");
    tester.focus(&Finder::key("f"));
    tester.ime(preedit("ni"));
    assert_eq!(tester.enter(|| c.value().text), "ni");
    assert_eq!(
        tester.enter(|| c.value().composing),
        Some(TextRange { start: 0, end: 2 })
    );
    tester.ime(preedit("你"));
    assert_eq!(tester.enter(|| c.value().text), "你");
    tester.ime(ImeEvent::Commit("你好".into()));
    assert_eq!(tester.enter(|| c.value()).text, "你好");
    assert_eq!(tester.enter(|| c.value().composing), None);
    // `Disabled` commits the pending preedit.
    tester.ime(preedit("x"));
    tester.ime(ImeEvent::Disabled);
    assert_eq!(tester.enter(|| c.value()).text, "你好x");
    assert_eq!(tester.enter(|| c.value().composing), None);
    // The IME area: the caret rect plus the padding, while focused and enabled.
    let mut tree = ViewTree::new(|| {
        Column::new()
            .child(Keyed::new(
                "f",
                TextField::new(TextEditingController::new("x")),
            ))
            .child(Keyed::new(
                "d",
                TextField::new(TextEditingController::new("y")).enabled(false),
            ))
    });
    let mut scene = Scene::new();
    let constraints = BoxConstraints::tight(Size::new(400.0, 200.0));
    tree.frame(constraints, &mut NoTextMeasure, &mut scene);
    assert_eq!(tree.ime_area(), None);
    let field = tree.find_key(&Key::from("f"))[0];
    assert!(tree.focus(field));
    tree.frame(constraints, &mut NoTextMeasure, &mut scene);
    assert_eq!(tree.ime_area(), Some(Rect::from_ltrb(16.0, 8.0, 18.0, 8.0)));
    tree.unfocus();
    assert_eq!(tree.ime_area(), None);
    assert!(!tree.focus(tree.find_key(&Key::from("d"))[0]));
}

#[test]
fn widgets_textfield_08_composing_underline() {
    let (mut tester, _) = plain("");
    tester.focus(&Finder::key("f"));
    let underline = |tester: &WidgetTester| {
        fills(tester)
            .into_iter()
            .find(|(r, c)| r.height() == 1.0 && *c == PRIMARY)
            .map(|(r, _)| r)
    };
    assert_eq!(underline(&tester), None);
    tester.ime(preedit("ab"));
    let line = underline(&tester).expect("an underline under the composing text");
    let editor = tester.rect(tester.find(&Finder::text("ab")));
    assert_eq!(line.left, 0.0);
    assert!(line.right > 0.0 && line.right <= editor.width());
    assert!(line.bottom <= editor.height());
    tester.ime(ImeEvent::Commit("ab".into()));
    assert_eq!(underline(&tester), None);
}

/// The translation of the transform scope the editor paints its content in.
fn content_shift(tester: &WidgetTester, text: &str) -> f32 {
    let editor = tester.find(&Finder::text(text));
    tester
        .frame()
        .entries_for(editor)
        .find_map(|e| match e.command {
            Command::PushTransform(t) => Some(t.coeffs()[4]),
            _ => None,
        })
        .unwrap_or(0.0)
}

#[test]
fn widgets_textfield_09_caret_following() {
    let (mut tester, c) = plain("");
    tester.focus(&Finder::key("f"));
    let long: String = "w".repeat(80);
    tester.enter(|| c.set_text(long.clone()));
    tester.pump();
    let editor = tester.rect(tester.find(&Finder::text(long.as_str())));
    let caret_rect = caret(&tester).expect("a caret");
    assert!(
        caret_rect.left > editor.width(),
        "the text is wider than the editor"
    );
    let painted = caret_rect.left + content_shift(&tester, &long);
    assert!(painted >= 0.0 && painted <= editor.width(), "{painted}");
    // The shift stays within the overflow, and Home brings the content back.
    assert!(-content_shift(&tester, &long) <= caret_rect.left - editor.width() + 2.0);
    key(&mut tester, NamedKey::Home, Modifiers::default());
    assert_eq!(content_shift(&tester, &long), 0.0);
    // Moving right within the visible part doesn't shift; a press lands where it is painted.
    for _ in 0..3 {
        key(&mut tester, NamedKey::ArrowRight, Modifiers::default());
    }
    assert_eq!(content_shift(&tester, &long), 0.0);
    key(&mut tester, NamedKey::End, Modifiers::default());
    let s = content_shift(&tester, &long);
    assert!(s < 0.0);
    let x = editor.left + 5.0;
    tester.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: Point::new(x, editor.center().y),
    });
    tester.pointer(PointerEvent {
        kind: PointerKind::Up(PointerButton::Primary),
        position: Point::new(x, editor.center().y),
    });
    tester.pump();
    let offset = sel(&tester, c).extent;
    assert!(offset > 0 && offset < 80, "{offset}");
    // A multi-line field never shifts.
    let (mut tester, c) = field("", |f| f.max_lines(None));
    tester.focus(&Finder::key("f"));
    let wrapped = "w ".repeat(80);
    tester.enter(|| c.set_text(wrapped.clone()));
    tester.pump();
    assert_eq!(content_shift(&tester, &wrapped), 0.0);
}

#[test]
fn widgets_textfield_10_caret_blinks() {
    use std::time::Duration;
    let ms = Duration::from_millis;
    let (mut tester, _) = plain("hi");
    tester.tap(&Finder::key("f"));
    assert!(caret(&tester).is_some(), "visible on focus");
    tester.advance(ms(499));
    assert!(caret(&tester).is_some());
    tester.advance(ms(1));
    assert!(caret(&tester).is_none(), "hidden after half a period");
    tester.advance(ms(500));
    assert!(caret(&tester).is_some(), "visible again");
    tester.advance(ms(600));
    assert!(caret(&tester).is_none());
    // An edit shows it at once and restarts the cycle.
    tester.press_key(LogicalKey::Character("a".into()), Modifiers::default());
    assert!(caret(&tester).is_some());
    tester.advance(ms(400));
    assert!(caret(&tester).is_some());
    tester.advance(ms(100));
    assert!(caret(&tester).is_none());
    // A caret move too.
    key(&mut tester, NamedKey::ArrowLeft, Modifiers::default());
    assert!(caret(&tester).is_some());
    // The selection highlight doesn't blink.
    key(&mut tester, NamedKey::ArrowRight, shift());
    tester.advance(ms(600));
    assert!(caret(&tester).is_none());
    assert!(!selection(&tester).is_empty());
    // Losing focus stops it.
    key(&mut tester, NamedKey::Tab, Modifiers::default());
    assert!(caret(&tester).is_none());
    tester.advance(ms(500));
    assert!(caret(&tester).is_none());
}

#[test]
fn widgets_textfield_11_double_click_selects_a_word() {
    use std::time::Duration;
    let ms = Duration::from_millis;
    let (mut tester, c) = plain("hello world");
    let editor = tester.rect(tester.find(&Finder::text("hello world")));
    let y = editor.center().y;
    let click = |tester: &mut WidgetTester, x: f32| {
        for kind in [
            PointerKind::Down(PointerButton::Primary),
            PointerKind::Up(PointerButton::Primary),
        ] {
            tester.pointer(PointerEvent {
                kind,
                position: Point::new(x, y),
            });
        }
        tester.pump();
    };
    let in_world = editor.right - 4.0;
    let in_hello = editor.left + 3.0;
    click(&mut tester, in_world);
    assert!(sel(&tester, c).is_collapsed());
    tester.advance(ms(100));
    click(&mut tester, in_world);
    assert_eq!(
        sel(&tester, c),
        TextSelection::new(6, 11),
        "double-click selects the word"
    );
    // A third press within the interval starts a single-click selection again.
    tester.advance(ms(100));
    click(&mut tester, in_world);
    assert!(sel(&tester, c).is_collapsed());
    // Too slow: two single clicks.
    tester.advance(ms(600));
    click(&mut tester, in_world);
    tester.advance(ms(600));
    click(&mut tester, in_world);
    assert!(sel(&tester, c).is_collapsed());
    // Dragging after a double-click extends by whole words.
    tester.advance(ms(600));
    click(&mut tester, in_world);
    tester.advance(ms(100));
    tester.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: Point::new(in_world, y),
    });
    tester.pump();
    assert_eq!(sel(&tester, c), TextSelection::new(6, 11));
    tester.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(in_hello, y),
    });
    tester.pump();
    let dragged = sel(&tester, c);
    assert_eq!((dragged.start(), dragged.end()), (0, 11));
    tester.pointer(PointerEvent {
        kind: PointerKind::Up(PointerButton::Primary),
        position: Point::new(in_hello, y),
    });
    tester.pump();
    // Too far apart: not a double-click.
    tester.advance(ms(600));
    click(&mut tester, in_hello);
    tester.advance(ms(100));
    click(&mut tester, in_hello + 10.0);
    assert!(sel(&tester, c).is_collapsed());
    // Whitespace between words selects the whitespace run.
    let (mut tester, c) = plain("a   b");
    let editor = tester.rect(tester.find(&Finder::text("a   b")));
    // Inside the spaces (the paragraph spans the field; "a" is about 9 px wide).
    let (x, y) = (editor.left + 14.0, editor.center().y);
    for _ in 0..2 {
        for kind in [
            PointerKind::Down(PointerButton::Primary),
            PointerKind::Up(PointerButton::Primary),
        ] {
            tester.pointer(PointerEvent {
                kind,
                position: Point::new(x, y),
            });
        }
        tester.advance(ms(50));
    }
    assert_eq!(sel(&tester, c), TextSelection::new(1, 4));
}
