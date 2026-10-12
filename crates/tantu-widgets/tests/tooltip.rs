//! Tests for `docs/specs/widgets/tooltip.md`, rules WIDGETS-TOOLTIP-01..07.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Color, Point, Rect};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{Command, ElementId};
use tantu_view::{Dyn, Keyed, PointerButton, PointerEvent, PointerKind};
use tantu_widgets::{Button, Column, Padding, SizedBox, Tooltip};

const INVERSE_SURFACE: Color = Color::from_argb32(0xFF31_3033);
const INVERSE_ON_SURFACE: Color = Color::from_argb32(0xFFF4_EFF4);

/// The tooltip box (the element filled with the inverse surface color), if shown.
fn tooltip_box(tester: &WidgetTester) -> Option<ElementId> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .find_map(|e| match e.command {
            Command::Fill { color, .. } if color == INVERSE_SURFACE => e.element,
            _ => None,
        })
}

/// Hovers the element and waits out the show delay (WIDGETS-TOOLTIP-07).
fn hover_and_wait(tester: &mut WidgetTester, key: &str) {
    tester.hover(&Finder::key(key));
    tester.advance(Duration::from_millis(500));
}

fn move_to(tester: &mut WidgetTester, p: Point) {
    tester.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: p,
        modifiers: Default::default(),
    });
    tester.pump();
}

/// A 100 × 20 target with a tooltip, at (40, 40) in a 300 × 300 window.
fn target(above: bool) -> WidgetTester {
    WidgetTester::with_size(300.0, 300.0, move || {
        let tip = Tooltip::new("Tip", Keyed::new("target", SizedBox::new(100.0, 20.0)));
        let tip = if above { tip.prefer_above() } else { tip };
        Padding::all(40.0).child(Column::new().child(tip))
    })
}

#[test]
fn widgets_tooltip_01_hover_shows_leave_and_press_hide() {
    let mut tester = target(false);
    assert!(tooltip_box(&tester).is_none());
    assert!(tester.find_all(&Finder::text("Tip")).is_empty());
    hover_and_wait(&mut tester, "target");
    let tip_box = tooltip_box(&tester).expect("shown");
    let text = tester.find(&Finder::text("Tip"));
    assert!(tester.rect(tip_box).height() >= 24.0);
    // The message in 12 px, inverse on-surface.
    let frame = tester.frame();
    let run = frame
        .entries_for(text)
        .find_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(run.clone()),
            _ => None,
        })
        .expect("the message is drawn");
    assert_eq!((run.font_size, run.color), (12.0, INVERSE_ON_SURFACE));
    // Leaving hides it.
    move_to(&mut tester, Point::new(250.0, 250.0));
    assert!(tooltip_box(&tester).is_none());
    // A press hides it too.
    hover_and_wait(&mut tester, "target");
    assert!(tooltip_box(&tester).is_some());
    let center = tester.rect(tester.find(&Finder::key("target"))).center();
    tester.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: center,
        modifiers: Default::default(),
    });
    tester.pump();
    assert!(tooltip_box(&tester).is_none());
}

#[test]
fn widgets_tooltip_02_placement() {
    let mut tester = target(false);
    hover_and_wait(&mut tester, "target");
    let child = tester.rect(tester.find(&Finder::key("target")));
    let tip = tester.rect(tooltip_box(&tester).expect("shown"));
    assert!(
        (tip.top - (child.bottom + 4.0)).abs() < 0.01,
        "{child:?} {tip:?}"
    );
    assert!(
        (tip.center().x - child.center().x).abs() < 0.01,
        "{child:?} {tip:?}"
    );

    let mut tester = target(true);
    hover_and_wait(&mut tester, "target");
    let child = tester.rect(tester.find(&Finder::key("target")));
    let tip = tester.rect(tooltip_box(&tester).expect("shown"));
    assert!(
        (tip.bottom - (child.top - 4.0)).abs() < 0.01,
        "{child:?} {tip:?}"
    );
}

#[test]
fn widgets_tooltip_03_pointer_passes_through() {
    let pressed = Rc::new(Cell::new(0u32));
    let p = pressed.clone();
    let mut tester = WidgetTester::with_size(300.0, 300.0, move || {
        let p = p.clone();
        Padding::all(40.0).child(
            Column::new()
                .child(Tooltip::new(
                    "Tip",
                    Keyed::new("target", SizedBox::new(100.0, 20.0)),
                ))
                .child(Keyed::new(
                    "under",
                    Button::new("Under").on_press(move || p.set(p.get() + 1)),
                )),
        )
    });
    hover_and_wait(&mut tester, "target");
    let tip = tester.rect(tooltip_box(&tester).expect("shown"));
    let under = tester.rect(tester.find(&Finder::key("under")));
    let spot = Point::new(tip.center().x, tip.bottom - 2.0);
    assert!(
        under.contains(spot),
        "the tooltip covers part of the button: {tip:?} {under:?}"
    );
    for kind in [
        PointerKind::Down(PointerButton::Primary),
        PointerKind::Up(PointerButton::Primary),
    ] {
        tester.pointer(PointerEvent {
            kind,
            position: spot,
            modifiers: Default::default(),
        });
    }
    assert_eq!(pressed.get(), 1);
}

#[test]
fn widgets_tooltip_04_focus_shows_it() {
    let mut tester = WidgetTester::with_size(300.0, 300.0, || {
        Padding::all(40.0).child(
            Column::new()
                .spacing(60.0)
                .child(Tooltip::new(
                    "Tip",
                    Column::new().child(Keyed::new("inner", Button::new("Inner"))),
                ))
                .child(Keyed::new("other", Button::new("Other"))),
        )
    });
    tester.focus(&Finder::key("inner"));
    assert!(tooltip_box(&tester).is_some());
    tester.focus(&Finder::key("other"));
    assert!(tooltip_box(&tester).is_none());
    // Hovered while focus leaves: still shown.
    tester.focus(&Finder::key("inner"));
    hover_and_wait(&mut tester, "inner");
    tester.focus(&Finder::key("other"));
    assert!(tooltip_box(&tester).is_some());
}

#[test]
fn widgets_tooltip_05_reactive_message() {
    let sig: Rc<RefCell<Option<Signal<String>>>> = Rc::default();
    let s = sig.clone();
    let mut tester = WidgetTester::with_size(300.0, 300.0, move || {
        let message = signal("Tip".to_owned());
        *s.borrow_mut() = Some(message);
        Padding::all(40.0).child(Tooltip::new(
            message,
            Keyed::new("target", SizedBox::new(100.0, 20.0)),
        ))
    });
    let message = sig.borrow_mut().take().expect("built");
    hover_and_wait(&mut tester, "target");
    tester.find(&Finder::text("Tip"));
    tester.enter(|| message.set("Changed".to_owned()));
    tester.pump();
    tester.find(&Finder::text("Changed"));
}

#[test]
fn widgets_tooltip_06_region_child_never_shows() {
    let mut tester = WidgetTester::with_size(300.0, 300.0, || {
        Padding::all(40.0).child(Tooltip::new(
            "Tip",
            Dyn::new(|| Keyed::new("target", SizedBox::new(100.0, 20.0))),
        ))
    });
    hover_and_wait(&mut tester, "target");
    assert!(tooltip_box(&tester).is_none());
    let _ = Rect::ZERO;
}

#[test]
fn widgets_tooltip_07_show_delay() {
    let mut tester = target(false);
    tester.hover(&Finder::key("target"));
    assert!(tooltip_box(&tester).is_none());
    tester.advance(Duration::from_millis(499));
    assert!(tooltip_box(&tester).is_none());
    tester.advance(Duration::from_millis(1));
    assert!(tooltip_box(&tester).is_some());
    // Leaving before the delay cancels the show.
    move_to(&mut tester, Point::new(250.0, 250.0));
    tester.hover(&Finder::key("target"));
    tester.advance(Duration::from_millis(100));
    move_to(&mut tester, Point::new(250.0, 250.0));
    tester.advance(Duration::from_millis(500));
    assert!(tooltip_box(&tester).is_none());
    // A press before the delay cancels it too.
    tester.hover(&Finder::key("target"));
    tester.advance(Duration::from_millis(100));
    let center = tester.rect(tester.find(&Finder::key("target"))).center();
    tester.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: center,
        modifiers: Default::default(),
    });
    tester.advance(Duration::from_millis(500));
    assert!(tooltip_box(&tester).is_none());
    // A zero wait shows at once.
    let mut tester = WidgetTester::with_size(300.0, 300.0, || {
        Padding::all(40.0).child(
            Tooltip::new("Tip", Keyed::new("target", SizedBox::new(100.0, 20.0)))
                .wait_duration(Duration::ZERO),
        )
    });
    tester.hover(&Finder::key("target"));
    assert!(tooltip_box(&tester).is_some());
}

#[test]
fn widgets_tooltip_08_theme_provided_on_target_is_live_in_overlay() {
    use tantu_theme::Theme;
    use tantu_view::Builder;
    let output = Rc::new(Cell::new(None));
    let mut tester = {
        let output = output.clone();
        WidgetTester::with_size(300.0, 300.0, move || {
            let mut theme = Theme::dark();
            theme.colors.inverse_surface = Color::from_rgb8(10, 90, 30);
            theme.colors.inverse_on_surface = Color::WHITE;
            let theme = signal(theme);
            output.set(Some(theme));
            Padding::all(40.0).child(Column::new().child(Tooltip::new(
                "Local tip",
                Keyed::new(
                    "target",
                    SizedBox::new(100.0, 20.0).child(Builder::new(move |cx| {
                        cx.provide(theme);
                        SizedBox::shrink()
                    })),
                ),
            )))
        })
    };
    hover_and_wait(&mut tester, "target");
    let green = Color::from_rgb8(10, 90, 30);
    let popup = tester
        .frame()
        .scene
        .entries()
        .iter()
        .find_map(|e| match e.command {
            Command::Fill { color, .. } if color == green => e.element,
            _ => None,
        })
        .expect("tooltip inherits target's inverse surface");
    let text = tester.find(&Finder::text("Local tip"));
    let mut next = Theme::light();
    next.colors.inverse_surface = Color::from_rgb8(120, 20, 40);
    next.colors.inverse_on_surface = Color::BLACK;
    let expected = next.colors.inverse_surface;
    tester.enter(|| output.get().expect("theme").set(next));
    tester.pump();
    assert_eq!(tester.find(&Finder::text("Local tip")), text);
    assert!(
        tester
            .frame()
            .entries_for(popup)
            .any(|e| matches!(e.command, Command::Fill { color, .. } if color == expected))
    );
    assert!(tester.frame().entries_for(text).any(|e| matches!(&e.command, Command::GlyphRun(run) if run.color == Color::BLACK && run.font_size == 12.0)));
}
