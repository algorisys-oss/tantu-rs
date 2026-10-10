//! Tests for `docs/specs/test/widget-tester.md`, rules TEST-WT-01..06.

use std::panic::{AssertUnwindSafe, catch_unwind};

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Point, Rect, Size};
use tantu_view::layout::{RenderConstrainedBox, RenderParagraph};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::{Dyn, Keyed, PointerButton, PointerEvent, PointerKind};
use tantu_widgets::{Button, Center, Column, Padding, SizedBox, Text};

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
