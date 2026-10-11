//! Tests for `docs/specs/widgets/transitions.md`, rules WIDGETS-OPACITY-01..02 and
//! WIDGETS-SWITCHER-01..03.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::Rect;
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::{AnimationController, AnyView, Curve, Keyed, Show, Tween};
use tantu_widgets::{AnimatedSwitcher, Button, Column, Opacity, SizedBox};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The opacities of the layers pushed in the last frame, in order.
fn layers(tester: &WidgetTester) -> Vec<f32> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::PushLayer(layer) => Some(layer.opacity),
            _ => None,
        })
        .collect()
}

/// The fills painted by the element keyed `key`.
fn fills_of(tester: &WidgetTester, key: &str) -> usize {
    let element = tester.find(&Finder::key(key));
    tester
        .frame()
        .entries_for(element)
        .filter(|e| matches!(e.command, Command::Fill { .. }))
        .count()
}

fn rect(tester: &WidgetTester, key: &str) -> Rect {
    tester.rect(tester.find(&Finder::key(key)))
}

#[test]
fn widgets_opacity_01_layer_and_reactivity() {
    let out: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::new(move || {
            let opacity = signal(1.0f32);
            out.set(Some(opacity));
            Column::new().child(Keyed::new(
                "o",
                Opacity::new(opacity, Keyed::new("b", Button::new("x").on_press(|| {}))),
            ))
        })
    };
    let opacity = out.get().expect("built");
    // Same geometry as the child.
    assert_eq!(rect(&tester, "o"), rect(&tester, "b"));
    assert!(layers(&tester).is_empty(), "no layer at 1");
    assert_eq!(fills_of(&tester, "b"), 1);
    tester.enter(|| opacity.set(0.5));
    tester.pump();
    assert_eq!(layers(&tester), [0.5]);
    assert_eq!(
        fills_of(&tester, "b"),
        1,
        "the child paints inside the layer"
    );
    // The layer wraps the child's commands: push, fill, pop in order.
    let kinds: Vec<&'static str> = tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::PushLayer(_) => Some("push"),
            Command::PopLayer => Some("pop"),
            Command::Fill { .. } => Some("fill"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, ["push", "fill", "pop"]);
    tester.enter(|| opacity.set(0.0));
    tester.pump();
    assert!(layers(&tester).is_empty(), "no layer at 0");
    assert_eq!(fills_of(&tester, "b"), 0, "nothing painted at 0");
    // Clamping and NaN.
    tester.enter(|| opacity.set(7.0));
    tester.pump();
    assert!(layers(&tester).is_empty());
    assert_eq!(fills_of(&tester, "b"), 1);
    tester.enter(|| opacity.set(f32::NAN));
    tester.pump();
    assert!(layers(&tester).is_empty());
    assert_eq!(fills_of(&tester, "b"), 1);
    tester.enter(|| opacity.set(-1.0));
    tester.pump();
    assert_eq!(fills_of(&tester, "b"), 0);
    // An animation as the prop.
    let ctrl: Rc<Cell<Option<AnimationController>>> = Rc::default();
    let mut tester = {
        let ctrl = ctrl.clone();
        WidgetTester::new(move || {
            let c = AnimationController::new(ms(100));
            ctrl.set(Some(c));
            Column::new().child(Opacity::new(
                c.drive(Tween::new(1.0f32, 0.0)).curve(Curve::Linear),
                Keyed::new("b", Button::new("x").on_press(|| {})),
            ))
        })
    };
    let c = ctrl.get().expect("built");
    tester.enter(|| c.forward());
    tester.advance(ms(50));
    assert_eq!(layers(&tester), [0.5]);
    tester.advance(ms(50));
    assert_eq!(fills_of(&tester, "b"), 0);
}

#[test]
fn widgets_opacity_02_hit_testing_unchanged() {
    let presses = Rc::new(Cell::new(0u32));
    let mut tester = {
        let presses = presses.clone();
        WidgetTester::new(move || {
            let presses = presses.clone();
            Column::new().child(Opacity::new(
                0.0f32,
                Keyed::new(
                    "b",
                    Button::new("x").on_press(move || presses.set(presses.get() + 1)),
                ),
            ))
        })
    };
    tester.tap(&Finder::key("b"));
    assert_eq!(presses.get(), 1);
}

/// A switcher keyed "s" over an `i32` signal, with cards keyed "card-N".
fn switcher(duration: Option<Duration>) -> (WidgetTester, Signal<i32>) {
    let out: Rc<Cell<Option<Signal<i32>>>> = Rc::default();
    let tester = {
        let out = out.clone();
        WidgetTester::new(move || {
            let key = signal(1);
            out.set(Some(key));
            let mut s = AnimatedSwitcher::new(key, |k| {
                AnyView::new(Keyed::new(
                    format!("card-{k}"),
                    Button::new(k.to_string()).on_press(|| {}),
                ))
            })
            .curve(Curve::Linear);
            if let Some(duration) = duration {
                s = s.duration(duration);
            }
            Column::new().child(Keyed::new("s", s))
        })
    };
    (tester, out.get().expect("built"))
}

fn present(tester: &WidgetTester, key: &str) -> bool {
    !tester.find_all(&Finder::key(key)).is_empty()
}

#[test]
fn widgets_switcher_01_cross_fade() {
    let (mut tester, key) = switcher(None);
    assert!(present(&tester, "card-1"));
    assert!(
        layers(&tester).is_empty(),
        "the first child at full opacity"
    );
    let card1 = tester.find(&Finder::key("card-1"));
    tester.enter(|| key.set(2));
    tester.pump();
    assert!(present(&tester, "card-1") && present(&tester, "card-2"));
    let card2 = tester.find(&Finder::key("card-2"));
    tester.advance(ms(150));
    let mut mid = layers(&tester);
    mid.sort_by(f32::total_cmp);
    assert_eq!(mid, [0.5, 0.5]);
    assert_eq!(
        tester.find(&Finder::key("card-1")),
        card1,
        "the old child keeps its element"
    );
    tester.advance(ms(150));
    assert!(!present(&tester, "card-1"), "removed when its fade ends");
    assert_eq!(tester.find(&Finder::key("card-2")), card2);
    assert!(layers(&tester).is_empty(), "the new child at full opacity");
}

#[test]
fn widgets_switcher_02_retarget_third_key_same_key_zero_duration() {
    let (mut tester, key) = switcher(None);
    tester.enter(|| key.set(2));
    tester.advance(ms(150));
    // Back to 1: card-1 fades back in from 0.5, card-2 out from 0.5.
    tester.enter(|| key.set(1));
    tester.advance(ms(75));
    let mut mid = layers(&tester);
    mid.sort_by(f32::total_cmp);
    assert_eq!(mid, [0.25, 0.75]);
    tester.advance(ms(75));
    assert!(present(&tester, "card-1") && !present(&tester, "card-2"));
    assert!(layers(&tester).is_empty());
    // A third key while two fade: all three present, each removed as its fade ends.
    tester.enter(|| key.set(2));
    tester.advance(ms(100));
    tester.enter(|| key.set(3));
    tester.pump();
    assert!(present(&tester, "card-1") && present(&tester, "card-2") && present(&tester, "card-3"));
    tester.advance(ms(200));
    assert!(
        !present(&tester, "card-1"),
        "card-1's fade out (from 2/3) is over"
    );
    assert!(present(&tester, "card-2") && present(&tester, "card-3"));
    tester.advance(ms(100));
    assert!(!present(&tester, "card-2"));
    assert!(present(&tester, "card-3"));
    // The same key again: nothing.
    let card3 = tester.find(&Finder::key("card-3"));
    tester.enter(|| key.set(3));
    tester.pump();
    assert_eq!(tester.find_all(&Finder::key("card-3")), [card3]);
    assert!(layers(&tester).is_empty());
    // A zero duration swaps at the next frame.
    let (mut tester, key) = switcher(Some(Duration::ZERO));
    tester.enter(|| key.set(2));
    tester.pump();
    tester.pump();
    assert!(!present(&tester, "card-1") && present(&tester, "card-2"));
    assert!(layers(&tester).is_empty());
}

#[test]
fn widgets_switcher_03_order_and_removal() {
    let (mut tester, key) = switcher(None);
    tester.enter(|| key.set(2));
    tester.advance(ms(150));
    // The newest child paints last: its fill comes after the old child's.
    let order: Vec<String> = tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { .. } => e.element.map(|id| {
                if id == tester.find(&Finder::key("card-1")) {
                    "1".to_owned()
                } else if id == tester.find(&Finder::key("card-2")) {
                    "2".to_owned()
                } else {
                    "?".to_owned()
                }
            }),
            _ => None,
        })
        .collect();
    assert_eq!(order, ["1", "2"]);
    // Removing the switcher disposes every child, fading or not.
    let show_out: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let mut tester = {
        let show_out = show_out.clone();
        WidgetTester::new(move || {
            let (show, key) = (signal(true), signal(1));
            show_out.set(Some(show));
            Column::new().child(Show::new(
                move || show.get(),
                move || {
                    AnimatedSwitcher::new(key, |k| {
                        AnyView::new(Keyed::new(format!("card-{k}"), SizedBox::new(10.0, 10.0)))
                    })
                },
            ))
        })
    };
    let show = show_out.get().expect("built");
    assert!(present(&tester, "card-1"));
    tester.enter(|| show.set(false));
    tester.pump();
    assert!(!present(&tester, "card-1"));
    tester.advance(ms(300));
    assert!(!present(&tester, "card-1"));
}
