//! Tests for `docs/specs/test/widget-tester.md`, rules TEST-WT-01..11.

use std::panic::{AssertUnwindSafe, catch_unwind};

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tantu_test::{Finder, WidgetTester};
use tantu_theme::Theme;

use tantu_view::Builder;
use tantu_view::core::{Point, Rect, Size, Vec2};
use tantu_view::layout::RenderConstrainedBox as Sized100;
use tantu_view::layout::{RenderConstrainedBox, RenderParagraph};
use tantu_view::reactive::{Signal, signal};
use tantu_view::reactive::{ticker, timer};
use tantu_view::scene::Command;
use tantu_view::{
    BuildCx, Dyn, ElementId, Handled, ImeEvent, Keyed, Phase, PointerButton, PointerEvent,
    PointerKind, View,
};
use tantu_widgets::{
    Button, Center, Column, Padding, ScrollController, ScrollView, SizedBox, Text,
    TextEditingController, TextField,
};

fn glyph_counts(tester: &WidgetTester) -> Vec<usize> {
    let frame = tester.frame();
    frame
        .scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(frame.scene.glyphs(run).len()),
            _ => None,
        })
        .collect()
}

#[test]
fn test_wt_01_builds_and_pumps_the_first_frame() {
    let tester = WidgetTester::new(|| Text::new("Hi"));
    assert_eq!((tester.frame().width, tester.frame().height), (800, 600));
    assert_eq!(tester.frame().scene.size(), Size::new(800.0, 600.0));
    assert_eq!(glyph_counts(&tester), [2]);
    let tester = WidgetTester::with_size(200.0, 100.0, SizedBox::shrink);
    assert_eq!(tester.frame().scene.size(), Size::new(200.0, 100.0));
}

#[test]
fn test_wt_02_pump() {
    let mut tester = WidgetTester::new(|| Text::new("x"));
    assert!(!tester.pump());
    let text: std::rc::Rc<std::cell::Cell<Option<Signal<String>>>> = Default::default();
    let mut tester = {
        let text = text.clone();
        WidgetTester::new(move || {
            let s = signal("ab".to_owned());
            text.set(Some(s));
            Text::new(s)
        })
    };
    let s = text.get().expect("built");
    tester.enter(|| s.set("abcd".to_owned()));
    assert!(tester.pump());
    assert_eq!(glyph_counts(&tester), [4]);
    assert!(!tester.pump());
}

#[test]
fn test_wt_03_finders() {
    let tester = WidgetTester::new(|| {
        Column::new()
            .child(Keyed::new("a", Text::new("one")))
            .child(Text::new("two"))
            .child(Keyed::new("b", SizedBox::new(10.0, 10.0)))
            .child(Text::new("two"))
    });
    let a = tester.find(&Finder::key("a"));
    assert_eq!(tester.text(a).as_deref(), Some("one"));
    assert_eq!(tester.find_all(&Finder::text("two")).len(), 2);
    assert_eq!(tester.find_all(&Finder::text("tw")).len(), 0);
    let paragraphs = tester.find_all(&Finder::render::<RenderParagraph>());
    assert_eq!(paragraphs.len(), 3);
    assert_eq!(paragraphs[0], a);
    let b = tester.find(&Finder::key("b"));
    assert!(
        tester
            .find_all(&Finder::render::<RenderConstrainedBox>())
            .contains(&b)
    );
    // Zero or several matches: a panic naming the finder.
    for finder in [Finder::key("missing"), Finder::text("two")] {
        let message = catch_unwind(AssertUnwindSafe(|| tester.find(&finder)))
            .expect_err("find panics")
            .downcast::<String>()
            .map(|m| *m)
            .unwrap_or_default();
        assert!(message.contains(&format!("{finder:?}")), "{message}");
    }
}

/// The counter from AGENTS.md, with a key on its button.
fn counter() -> impl tantu_view::View {
    let count = signal(0);
    Padding::all(16.0).child(
        Column::new()
            .spacing(8.0)
            .child(Text::new(move || format!("Count: {}", count.get())))
            .child(Keyed::new(
                "inc",
                Button::new("Increment").on_press(move || count.update(|c| *c += 1)),
            )),
    )
}

#[test]
fn test_wt_04_tap_and_hover() {
    let mut tester = WidgetTester::new(counter);
    tester.find(&Finder::text("Count: 0"));
    tester.tap(&Finder::key("inc"));
    tester.find(&Finder::text("Count: 1"));
    tester.tap(&Finder::key("inc"));
    tester.tap(&Finder::key("inc"));
    tester.find(&Finder::text("Count: 3"));

    // Hover: the button's fill changes.
    let fill = |tester: &WidgetTester| {
        let button = tester.find(&Finder::key("inc"));
        tester
            .frame()
            .entries_for(button)
            .find_map(|e| match e.command {
                Command::Fill { color, .. } => Some(color),
                _ => None,
            })
            .expect("a fill")
    };
    let mut tester = WidgetTester::new(counter);
    let idle = fill(&tester);
    tester.hover(&Finder::key("inc"));
    assert_ne!(fill(&tester), idle);

    // A raw event goes through `pointer`, without a pump.
    let mut tester = WidgetTester::new(counter);
    let center = tester.rect(tester.find(&Finder::key("inc"))).center();
    for kind in [
        PointerKind::Down(PointerButton::Primary),
        PointerKind::Up(PointerButton::Primary),
    ] {
        assert!(tester.pointer(PointerEvent {
            kind,
            position: center,
            modifiers: Default::default(),
        }));
    }
    tester.find(&Finder::text("Count: 0"));
    tester.pump();
    tester.find(&Finder::text("Count: 1"));
}

#[test]
fn test_wt_05_rect_and_text() {
    let tester = WidgetTester::with_size(200.0, 200.0, || {
        Center::new().child(Padding::all(10.0).child(Keyed::new("box", SizedBox::new(40.0, 20.0))))
    });
    let rect = tester.rect(tester.find(&Finder::key("box")));
    assert_eq!(rect, Rect::from_ltwh(80.0, 90.0, 40.0, 20.0));
    assert_eq!(tester.text(tester.find(&Finder::key("box"))), None);
    // A region: the union of its render children.
    let tester = WidgetTester::with_size(200.0, 200.0, || {
        Column::new().child(Keyed::new("region", Dyn::new(|| SizedBox::new(30.0, 10.0))))
    });
    let rect = tester.rect(tester.find(&Finder::key("region")));
    assert_eq!((rect.width(), rect.height()), (30.0, 10.0));
    let _ = Point::ZERO;
}

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/goldens/counter.png");

#[test]
fn test_wt_06_goldens() {
    let mut tester = WidgetTester::with_size(240.0, 120.0, counter);
    tester.matches_golden(GOLDEN);
    if std::env::var_os("TANTU_UPDATE_GOLDENS").is_some() {
        return; // writing goldens: the checks below would overwrite this one
    }
    // A different frame doesn't match.
    tester.hover(&Finder::key("inc"));
    let mismatch = catch_unwind(AssertUnwindSafe(|| tester.matches_golden(GOLDEN)));
    assert!(mismatch.is_err());
    // A missing file panics, naming it.
    let missing = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/goldens/missing.png");
    let mut tester = WidgetTester::with_size(240.0, 120.0, counter);
    let message = catch_unwind(AssertUnwindSafe(|| tester.matches_golden(missing)))
        .expect_err("a missing golden panics")
        .downcast::<String>()
        .map(|m| *m)
        .unwrap_or_default();
    assert!(message.contains("missing.png"), "{message}");
}

/// A focusable box that counts Enter presses into the text below it.
struct EnterCounter;

impl tantu_view::View for EnterCounter {
    fn build(self, cx: &mut tantu_view::BuildCx<'_>) -> tantu_view::ElementId {
        use tantu_view::{FocusOptions, Handled, LogicalKey, NamedKey, Phase};
        let count = signal(0u32);
        let column = Column::new()
            .child(Keyed::new("target", SizedBox::new(50.0, 20.0)))
            .child(Text::new(move || format!("Enters: {}", count.get())))
            .build(cx);
        let target = cx.tree().children(column)[0];
        cx.focusable(target, FocusOptions::default());
        cx.on_key(target, Phase::Bubble, move |k| {
            if k.event.pressed && k.event.key == LogicalKey::Named(NamedKey::Enter) {
                count.update(|c| *c += 1);
                Handled::Stop
            } else {
                Handled::Continue
            }
        });
        column
    }
}

#[test]
fn test_wt_07_press_key_and_focus() {
    use tantu_view::{LogicalKey, Modifiers, NamedKey};
    let mut tester = WidgetTester::new(|| EnterCounter);
    // Nothing focused: Enter reaches no handler.
    tester.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    tester.find(&Finder::text("Enters: 0"));
    tester.focus(&Finder::key("target"));
    tester.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    tester.press_key(LogicalKey::Named(NamedKey::Enter), Modifiers::default());
    tester.find(&Finder::text("Enters: 2"));
    // Focusing something that isn't focusable panics.
    let not_focusable = catch_unwind(AssertUnwindSafe(|| {
        tester.focus(&Finder::text("Enters: 2"))
    }));
    assert!(not_focusable.is_err());
}

/// A 100 × 100 box recording the pointer events it receives, with their local positions.
struct Recorder(Rc<RefCell<Vec<(PointerKind, Point)>>>);

impl View for Recorder {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(Sized100::sized(Some(100.0), Some(100.0)), []);
        let log = self.0;
        cx.on_pointer(id, Phase::Bubble, move |p| {
            // Hover enter/leave (after the release) isn't part of the drag.
            if !matches!(p.event.kind, PointerKind::Enter | PointerKind::Leave) {
                log.borrow_mut().push((p.event.kind, p.local));
            }
            Handled::Stop
        });
        id
    }
}

#[test]
fn test_wt_08_drag() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let events = Rc::clone(&log);
    let mut tester =
        WidgetTester::new(move || Center::new().child(Keyed::new("box", Recorder(events))));
    tester.drag(&Finder::key("box"), Vec2::new(30.0, 10.0));
    assert_eq!(
        log.borrow().clone(),
        vec![
            (
                PointerKind::Down(PointerButton::Primary),
                Point::new(50.0, 50.0)
            ),
            (PointerKind::Move, Point::new(80.0, 60.0)),
            (
                PointerKind::Up(PointerButton::Primary),
                Point::new(80.0, 60.0)
            ),
        ]
    );
    let result = catch_unwind(AssertUnwindSafe(|| {
        tester.drag(&Finder::key("nope"), Vec2::new(1.0, 1.0));
    }));
    let message = *result.unwrap_err().downcast::<String>().expect("a message");
    assert!(message.contains("nope"), "{message}");
}

#[test]
fn test_wt_09_enter_text() {
    let out: Rc<RefCell<Option<TextEditingController>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::new(move || {
            let controller = TextEditingController::new("ab");
            *out.borrow_mut() = Some(controller);
            Column::new().child(Keyed::new("f", TextField::new(controller)))
        })
    };
    let controller = out.borrow().expect("built");
    tester.enter_text(&Finder::key("f"), "é!");
    assert_eq!(tester.enter(|| controller.text()), "abé!");
    tester.find(&Finder::text("abé!"));
}

#[test]
fn test_wt_10_ime() {
    let out: Rc<RefCell<Option<TextEditingController>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::new(move || {
            let controller = TextEditingController::new("");
            *out.borrow_mut() = Some(controller);
            Column::new().child(Keyed::new("f", TextField::new(controller)))
        })
    };
    let controller = out.borrow().expect("built");
    tester.tap(&Finder::key("f"));
    tester.ime(ImeEvent::Preedit {
        text: "ab".into(),
        cursor: None,
    });
    assert_eq!(tester.enter(|| controller.value().text), "ab");
    assert!(tester.enter(|| controller.value().composing).is_some());
    tester.ime(ImeEvent::Commit("ab".into()));
    assert_eq!(tester.enter(|| controller.value().composing), None);
    tester.find(&Finder::text("ab"));
}

#[test]
fn test_wt_11_scroll() {
    let out: Rc<RefCell<Option<ScrollController>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::new(move || {
            let controller = ScrollController::new();
            *out.borrow_mut() = Some(controller);
            Keyed::new(
                "s",
                ScrollView::new(SizedBox::new(800.0, 5000.0))
                    .controller(controller)
                    .wheel_duration(std::time::Duration::ZERO),
            )
        })
    };
    let controller = out.borrow().expect("built");
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, 30.0));
    assert_eq!(tester.enter(|| controller.offset()), 30.0);
}

#[test]
fn test_wt_12_theme() {
    let seen: Rc<RefCell<Option<Signal<Theme>>>> = Rc::default();
    let tester = {
        let seen = seen.clone();
        WidgetTester::new(move || {
            Builder::new(move |cx| {
                *seen.borrow_mut() = Theme::signal(cx);
                SizedBox::new(10.0, 10.0)
            })
        })
    };
    let inner = seen.borrow().expect("the tester provides a theme");
    assert_eq!(tester.enter(|| tester.theme().get()), Theme::light());
    assert_eq!(tester.enter(|| inner.get()), Theme::light());
    tester.enter(|| tester.theme().set(Theme::dark()));
    assert_eq!(tester.enter(|| inner.get()), Theme::dark());
}

#[test]
fn test_wt_13_clock_and_advance() {
    let mut tester = WidgetTester::new(|| {
        let (fired, ticks) = (signal(false), signal(0u32));
        timer(std::time::Duration::from_millis(500), move || {
            fired.set(true)
        });
        ticker(move |_| ticks.update(|n| *n += 1)).start();
        Column::new()
            .child(Keyed::new(
                "fired",
                Text::new(move || if fired.get() { "yes" } else { "no" }.to_owned()),
            ))
            .child(Keyed::new(
                "ticks",
                Text::new(move || ticks.get().to_string()),
            ))
    });
    let text = |tester: &WidgetTester, key: &str| {
        let id = tester.find(&Finder::key(key));
        tester.text(id).expect("a text")
    };
    let start = tester.now();
    // The first pump ticked once.
    assert_eq!(text(&tester, "ticks"), "1");
    assert_eq!(text(&tester, "fired"), "no");
    tester.pump();
    assert_eq!(text(&tester, "ticks"), "2");
    assert_eq!(tester.now(), start);
    tester.advance(std::time::Duration::from_millis(499));
    assert_eq!(text(&tester, "fired"), "no");
    assert_eq!(text(&tester, "ticks"), "3");
    tester.advance(std::time::Duration::from_millis(1));
    assert_eq!(text(&tester, "fired"), "yes");
    assert_eq!(tester.now(), start + std::time::Duration::from_millis(500));
}

#[test]
fn test_wt_14_a11y_tree_and_actions() {
    let presses = Rc::new(Cell::new(0u32));
    let mut tester = {
        let presses = presses.clone();
        WidgetTester::new(move || {
            let presses = presses.clone();
            Column::new().child(Keyed::new(
                "b",
                Button::new("Go").on_press(move || presses.set(presses.get() + 1)),
            ))
        })
    };
    let update = tester.a11y_tree();
    let root = update.tree.as_ref().expect("a tree").root;
    let window = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == root)
        .expect("root")
        .1;
    assert_eq!(window.label(), Some("test"));
    assert_eq!(window.children().len(), 1);
    let button_id = tester.find(&Finder::key("b"));
    assert_eq!(
        window.children()[0],
        tantu_test::accesskit::NodeId(button_id.to_raw())
    );
    assert!(tester.perform_action(&Finder::key("b"), tantu_view::SemanticsAction::Click));
    assert_eq!(presses.get(), 1);
    assert!(!tester.perform_action(&Finder::key("b"), tantu_view::SemanticsAction::Increment));
}

#[test]
fn test_wt_15_ensure_visible() {
    let out: Rc<RefCell<Option<ScrollController>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let controller = ScrollController::new();
            *out.borrow_mut() = Some(controller);
            ScrollView::new(Column::new().children(
                (0..10).map(|i| Keyed::new(format!("c{i}"), SizedBox::new(300.0, 100.0))),
            ))
            .controller(controller)
            .wheel_duration(std::time::Duration::ZERO)
        })
    };
    let controller = out.borrow().expect("built");
    let c7 = tester.find(&Finder::key("c7"));
    assert_eq!(tester.rect(c7).top, 700.0);
    // Scrolled and pumped: the element's rect is inside the window.
    tester.ensure_visible(&Finder::key("c7"));
    assert_eq!(tester.enter(|| controller.offset()), 600.0);
    assert_eq!(tester.rect(c7), Rect::from_ltwh(0.0, 100.0, 300.0, 100.0));
}

/// A box filling its constraints that asks for `cursor`.
struct CursorZone(tantu_view::CursorIcon);

impl View for CursorZone {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::expand(), []);
        cx.set_cursor(id, self.0);
        id
    }
}

#[test]
fn test_wt_17_cursor() {
    use tantu_view::CursorIcon;
    let mut tester = WidgetTester::with_size(200.0, 200.0, || {
        Column::new()
            .child(SizedBox::height(100.0).child(Keyed::new("text", CursorZone(CursorIcon::Text))))
            .child(SizedBox::height(100.0).child(Keyed::new(
                "resize",
                CursorZone(CursorIcon::ResizeHorizontal),
            )))
    });
    assert_eq!(tester.cursor(), CursorIcon::Default, "no pointer event yet");
    tester.hover(&Finder::key("text"));
    assert_eq!(tester.cursor(), CursorIcon::Text);
    tester.tap(&Finder::key("resize"));
    assert_eq!(tester.cursor(), CursorIcon::ResizeHorizontal);
}

/// A box with a semantics label.
struct Labelled(&'static str);

impl View for Labelled {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(Sized100::sized(Some(50.0), Some(50.0)), []);
        cx.set_semantics(
            id,
            tantu_view::Semantics {
                name: Some(self.0.to_owned()),
                ..Default::default()
            },
        );
        id
    }
}

#[test]
fn test_wt_18_semantics_label() {
    let mut tester = WidgetTester::with_size(200.0, 200.0, || {
        Column::new()
            .child(Keyed::new("first", Labelled("Resize Folders")))
            .child(Labelled("Other"))
    });
    let found = tester.find(&Finder::semantics_label("Resize Folders"));
    assert_eq!(found, tester.find(&Finder::key("first")));
    assert!(
        tester
            .find_all(&Finder::semantics_label("Resize"))
            .is_empty(),
        "exact matches only"
    );
    // Usable like any finder.
    tester.tap(&Finder::semantics_label("Other"));
}

#[test]
fn test_wt_19_clipboard() {
    use tantu_view::Clipboard;
    type Out = Rc<RefCell<Option<Rc<dyn Clipboard>>>>;
    let out: Out = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(100.0, 100.0, move || {
            Builder::new(move |cx| {
                *out.borrow_mut() = Some(cx.clipboard());
                SizedBox::new(10.0, 10.0)
            })
        })
    };
    let clipboard = out.borrow_mut().take().expect("built");
    assert_eq!(tester.clipboard_text(), None);
    tester.set_clipboard_text("from the test");
    assert_eq!(clipboard.read_text().as_deref(), Some("from the test"));
    clipboard.write_text("from a view");
    assert_eq!(tester.clipboard_text().as_deref(), Some("from a view"));
}

/// A 100 × 100 file-drop target logging what it receives.
struct DropLog(Rc<RefCell<Vec<String>>>);

impl tantu_view::View for DropLog {
    fn build(self, cx: &mut tantu_view::BuildCx<'_>) -> tantu_view::ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(100.0), Some(100.0)), []);
        let log = self.0;
        cx.on_file_drop(id, move |event| {
            let entry = match event {
                tantu_view::FileDropEvent::Hovered(p) => format!("hover {}", p.len()),
                tantu_view::FileDropEvent::Cancelled => "cancel".to_owned(),
                tantu_view::FileDropEvent::Dropped(p) => format!("drop {}", p.len()),
            };
            log.borrow_mut().push(entry);
        });
        id
    }
}

#[test]
fn test_wt_20_drop_files() {
    let log: Rc<RefCell<Vec<String>>> = Rc::default();
    let mut t = {
        let log = log.clone();
        WidgetTester::with_size(300.0, 200.0, move || DropLog(log))
    };
    let paths = [
        std::path::PathBuf::from("/tmp/a.txt"),
        std::path::PathBuf::from("/tmp/b.png"),
    ];
    t.drop_files(&paths, Some(Point::new(50.0, 50.0)));
    t.drop_files(&paths[..1], None);
    assert_eq!(*log.borrow(), ["hover 2", "drop 2", "hover 1", "drop 1"]);
}
