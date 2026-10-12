//! Tests for `docs/specs/widgets/slider.md`, rules WIDGETS-SLIDER-01..08.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Color, Point, Rect, Size, Vec2};
use tantu_view::layout::{BoxConstraints, NoTextMeasure};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{BorderRadius, Command, RoundedRect, Scene};
use tantu_view::{
    Key, KeyEvent, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent,
    PointerKind, ViewTree,
};
use tantu_widgets::{Button, Column, Padding, Row, Slider};

const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);
const SURFACE_CONTAINER_HIGHEST: Color = Color::from_argb32(0xFFE6_E0E9);
const ON_SURFACE_38: Color = Color::from_argb32(0x611C_1B1F);
const ON_SURFACE_12: Color = Color::from_argb32(0x1F1C_1B1F);
const ACTIVE_TICK: Color = Color::from_argb32(0x61FF_FFFF);
const INACTIVE_TICK: Color = Color::from_argb32(0x6149_454F);

/// Every filled shape in the last frame, in paint order (the slider is the only painter).
fn fills(tester: &WidgetTester) -> Vec<(RoundedRect, Color)> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { shape, color } => Some((shape, color)),
            _ => None,
        })
        .collect()
}

fn circle(cx: f32, cy: f32, r: f32) -> RoundedRect {
    RoundedRect::new(
        Rect::from_ltrb(cx - r, cy - r, cx + r, cy + r),
        BorderRadius::circular(r),
    )
}

fn track(from: f32, to: f32) -> RoundedRect {
    RoundedRect::new(
        Rect::from_ltrb(from, 22.0, to, 26.0),
        BorderRadius::circular(2.0),
    )
}

/// The thumb's center x (the 20 px circle), if drawn.
fn thumb_x(tester: &WidgetTester) -> Option<f32> {
    fills(tester).iter().find_map(|(shape, _)| {
        (shape.rect.width() == 20.0 && shape.rect.height() == 20.0 && shape.rect.top == 14.0)
            .then(|| shape.rect.center().x)
    })
}

/// The state layer's color (the 48 px circle), if drawn.
fn layer(tester: &WidgetTester) -> Option<Color> {
    fills(tester)
        .iter()
        .find_map(|(shape, color)| (shape.rect.width() == 48.0).then_some(*color))
}

fn layer_alpha(tester: &WidgetTester) -> Option<f32> {
    layer(tester).map(|c| c.a)
}

/// A 300 × 200 window with the slider in a column (so it spans the 300 px).
fn window(slider: Slider) -> WidgetTester {
    let slider = Cell::new(Some(slider));
    WidgetTester::with_size(300.0, 200.0, move || {
        Column::new().child(Keyed::new("s", slider.take().expect("built once")))
    })
}

/// The calls a slider's callbacks made, as `("start" | "change" | "end", value)`.
type Calls = Rc<RefCell<Vec<(&'static str, f64)>>>;

fn recording(slider: Slider, calls: &Calls) -> Slider {
    let (a, b, c) = (calls.clone(), calls.clone(), calls.clone());
    slider
        .on_change_start(move |v| a.borrow_mut().push(("start", v)))
        .on_change(move |v| b.borrow_mut().push(("change", v)))
        .on_change_end(move |v| c.borrow_mut().push(("end", v)))
}

/// A slider over 0..100 showing `value` (a signal) and writing it back on change.
fn bound(calls: &Calls) -> (WidgetTester, Signal<f64>) {
    let out: Rc<Cell<Option<Signal<f64>>>> = Rc::default();
    let tester = {
        let (out, calls) = (out.clone(), calls.clone());
        WidgetTester::with_size(300.0, 200.0, move || {
            let value = signal(20.0f64);
            out.set(Some(value));
            let changes = calls.clone();
            let slider = Slider::new(value).range(0.0, 100.0).on_change(move |v| {
                changes.borrow_mut().push(("change", v));
                value.set(v);
            });
            let (a, c) = (calls.clone(), calls.clone());
            let slider = slider
                .on_change_start(move |v| a.borrow_mut().push(("start", v)))
                .on_change_end(move |v| c.borrow_mut().push(("end", v)));
            Column::new()
                .child(Keyed::new("s", slider))
                .child(Keyed::new("b", Button::new("B")))
        })
    };
    (tester, out.get().expect("built"))
}

fn pointer(tester: &mut WidgetTester, kind: PointerKind, x: f32) -> bool {
    let handled = tester.pointer(PointerEvent {
        kind,
        position: Point::new(x, 24.0),
        modifiers: Default::default(),
    });
    tester.pump();
    handled
}

fn down(tester: &mut WidgetTester, x: f32) -> bool {
    pointer(tester, PointerKind::Down(PointerButton::Primary), x)
}

fn up(tester: &mut WidgetTester, x: f32) -> bool {
    pointer(tester, PointerKind::Up(PointerButton::Primary), x)
}

fn mv(tester: &mut WidgetTester, x: f32) -> bool {
    pointer(tester, PointerKind::Move, x)
}

fn key(tester: &mut WidgetTester, key: NamedKey) {
    tester.press_key(LogicalKey::Named(key), Modifiers::default());
}

#[test]
fn widgets_slider_01_size() {
    let tester = window(Slider::new(0.5));
    assert_eq!(
        tester.rect(tester.find(&Finder::key("s"))),
        Rect::from_ltrb(0.0, 0.0, 300.0, 48.0)
    );
    let tester = WidgetTester::with_size(300.0, 200.0, || {
        Row::new().child(Keyed::new("s", Slider::new(0.5)))
    });
    let rect = tester.rect(tester.find(&Finder::key("s")));
    assert_eq!((rect.width(), rect.height()), (192.0, 48.0));
}

#[test]
fn widgets_slider_02_fraction_and_thumb() {
    let cases: [(Slider, f32); 8] = [
        (Slider::new(0.25), 80.0),
        (Slider::new(2.0), 290.0),
        (Slider::new(-1.0), 10.0),
        (Slider::new(f64::NAN), 10.0),
        (Slider::new(15.0).range(10.0, 20.0), 150.0),
        (Slider::new(15.0).range(20.0, 10.0), 10.0),
        (Slider::new(5.0).range(5.0, f64::INFINITY), 10.0),
        (Slider::new(0.3).divisions(4), 80.0),
    ];
    for (i, (slider, x)) in cases.into_iter().enumerate() {
        let tester = window(slider);
        assert_eq!(thumb_x(&tester), Some(x), "case {i}");
    }
}

#[test]
fn widgets_slider_03_enabled_look() {
    let tester = window(Slider::new(0.25).on_change(|_| {}));
    assert_eq!(
        fills(&tester),
        vec![
            (track(10.0, 80.0), PRIMARY),
            (track(80.0, 290.0), SURFACE_CONTAINER_HIGHEST),
            (circle(80.0, 24.0, 10.0), PRIMARY),
        ]
    );
    // Discrete: a dot per division point, active up to the thumb, painted before it.
    let tester = window(Slider::new(0.5).divisions(2).on_change(|_| {}));
    assert_eq!(
        fills(&tester),
        vec![
            (track(10.0, 150.0), PRIMARY),
            (track(150.0, 290.0), SURFACE_CONTAINER_HIGHEST),
            (circle(10.0, 24.0, 1.0), ACTIVE_TICK),
            (circle(150.0, 24.0, 1.0), ACTIVE_TICK),
            (circle(290.0, 24.0, 1.0), INACTIVE_TICK),
            (circle(150.0, 24.0, 10.0), PRIMARY),
        ]
    );
    let mut tester = WidgetTester::with_size(240.0, 180.0, || {
        Padding::all(8.0).child(
            Column::new()
                .child(Slider::new(0.3).on_change(|_| {}))
                .child(Slider::new(0.5).divisions(4).on_change(|_| {}))
                .child(Slider::new(0.7)),
        )
    });
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/slider.png"
    ));
}

#[test]
fn widgets_slider_04_disabled() {
    let disabled = vec![
        (track(10.0, 80.0), ON_SURFACE_38),
        (track(80.0, 290.0), ON_SURFACE_12),
        (circle(80.0, 24.0, 10.0), ON_SURFACE_38),
    ];
    // No `on_change` or `enabled(false)` both disable.
    let tester = window(Slider::new(0.25));
    assert_eq!(fills(&tester), disabled);
    let calls: Calls = Rc::default();
    let mut tester = window(recording(Slider::new(0.25).enabled(false), &calls));
    assert_eq!(fills(&tester), disabled);
    // Input is ignored: no calls, no layers, no focus.
    tester.hover(&Finder::key("s"));
    down(&mut tester, 150.0);
    mv(&mut tester, 200.0);
    up(&mut tester, 200.0);
    key(&mut tester, NamedKey::Tab);
    key(&mut tester, NamedKey::ArrowRight);
    assert!(calls.borrow().is_empty());
    assert_eq!(fills(&tester), disabled);
    // Becoming disabled clears focus and hover; re-enabling doesn't bring them back.
    let on: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let mut tester = {
        let on = on.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let enabled = signal(true);
            on.set(Some(enabled));
            Column::new().child(Keyed::new(
                "s",
                Slider::new(0.25).enabled(enabled).on_change(|_| {}),
            ))
        })
    };
    let enabled = on.get().expect("built");
    tester.focus(&Finder::key("s"));
    tester.hover(&Finder::key("s"));
    assert_eq!(layer_alpha(&tester), Some(0.10));
    tester.enter(|| enabled.set(false));
    tester.pump();
    assert_eq!(fills(&tester), disabled);
    tester.enter(|| enabled.set(true));
    tester.pump();
    assert_eq!(layer(&tester), None);
}

#[test]
fn widgets_slider_05_pointer() {
    let calls: Calls = Rc::default();
    let (mut tester, value) = bound(&calls);
    assert_eq!(thumb_x(&tester), Some(66.0));
    // Press at the center: start with the current value, then the value under the pointer.
    assert!(down(&mut tester, 150.0));
    assert_eq!(
        calls.borrow().clone(),
        vec![("start", 20.0), ("change", 50.0)]
    );
    assert_eq!(thumb_x(&tester), Some(150.0));
    // Moves, also outside the bounds (captured), report; a repeated value doesn't.
    mv(&mut tester, 340.0);
    mv(&mut tester, 340.0);
    mv(&mut tester, 10.0);
    up(&mut tester, 10.0);
    assert_eq!(
        calls.borrow()[2..],
        [("change", 100.0), ("change", 0.0), ("end", 0.0)]
    );
    assert_eq!(tester.enter(|| value.get()), 0.0);
    // After the release, moves report nothing.
    calls.borrow_mut().clear();
    mv(&mut tester, 150.0);
    assert!(calls.borrow().is_empty());
    // `drag` from the center by 140 px.
    tester.drag(&Finder::key("s"), Vec2::new(140.0, 0.0));
    assert_eq!(
        calls.borrow().clone(),
        vec![
            ("start", 0.0),
            ("change", 50.0),
            ("change", 100.0),
            ("end", 100.0)
        ]
    );
    // Pressing where the thumb already is calls start and end but not change.
    calls.borrow_mut().clear();
    down(&mut tester, 290.0);
    up(&mut tester, 290.0);
    assert_eq!(
        calls.borrow().clone(),
        vec![("start", 100.0), ("end", 100.0)]
    );
    // The thumb follows `value`, not the pointer.
    let calls: Calls = Rc::default();
    let mut tester = window(recording(Slider::new(0.25), &calls));
    down(&mut tester, 150.0);
    assert_eq!(
        calls.borrow().clone(),
        vec![("start", 0.25), ("change", 0.5)]
    );
    assert_eq!(thumb_x(&tester), Some(80.0));
    up(&mut tester, 150.0);
    // Discrete values snap.
    let calls: Calls = Rc::default();
    let mut tester = window(recording(Slider::new(0.0).divisions(4), &calls));
    down(&mut tester, 100.0);
    assert_eq!(
        calls.borrow().clone(),
        vec![("start", 0.0), ("change", 0.25)]
    );
    up(&mut tester, 100.0);
    // Secondary presses do nothing.
    let calls: Calls = Rc::default();
    let mut tester = window(recording(Slider::new(0.0), &calls));
    pointer(
        &mut tester,
        PointerKind::Down(PointerButton::Secondary),
        150.0,
    );
    assert!(calls.borrow().is_empty());
}

#[test]
fn widgets_slider_06_keyboard() {
    let calls: Calls = Rc::default();
    let (mut tester, value) = bound(&calls);
    // Tab reaches it; a press focuses it.
    key(&mut tester, NamedKey::Tab);
    assert_eq!(layer_alpha(&tester), Some(0.10));
    key(&mut tester, NamedKey::Tab);
    assert_eq!(layer(&tester), None);
    down(&mut tester, 66.0);
    up(&mut tester, 66.0);
    assert_eq!(layer_alpha(&tester), Some(0.10));
    calls.borrow_mut().clear();
    tester.enter(|| value.set(50.0));
    tester.pump();
    for (k, expected) in [
        (NamedKey::ArrowRight, 55.0),
        (NamedKey::ArrowUp, 60.0),
        (NamedKey::ArrowLeft, 55.0),
        (NamedKey::ArrowDown, 50.0),
        (NamedKey::End, 100.0),
        (NamedKey::Home, 0.0),
    ] {
        key(&mut tester, k);
        assert_eq!(tester.enter(|| value.get()), expected, "{k:?}");
    }
    assert_eq!(calls.borrow().len(), 6);
    assert!(calls.borrow().iter().all(|(what, _)| *what == "change"));
    // At the end, the key is still handled but reports nothing.
    key(&mut tester, NamedKey::ArrowLeft);
    assert_eq!(calls.borrow().len(), 6);
    assert_eq!(tester.enter(|| value.get()), 0.0);
    // Other keys change nothing.
    tester.press_key(LogicalKey::Character("a".into()), Modifiers::default());
    assert_eq!(calls.borrow().len(), 6);
    // Discrete steps are one division.
    let calls: Calls = Rc::default();
    let mut tester = window(recording(Slider::new(0.0).divisions(4), &calls));
    tester.focus(&Finder::key("s"));
    key(&mut tester, NamedKey::ArrowRight);
    assert_eq!(calls.borrow().clone(), vec![("change", 0.25)]);
    // Repeats count; releases don't.
    let calls: Calls = Rc::default();
    let changes = calls.clone();
    let mut tree = ViewTree::new(move || {
        Keyed::new(
            "s",
            Slider::new(0.0).on_change(move |v| changes.borrow_mut().push(("change", v))),
        )
    });
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::tight(Size::new(300.0, 48.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    let slider = tree.find_key(&Key::from("s"))[0];
    assert!(tree.focus(slider));
    let right = LogicalKey::Named(NamedKey::ArrowRight);
    let repeat = KeyEvent {
        repeat: true,
        ..KeyEvent::press(right.clone(), Modifiers::default())
    };
    assert!(tree.dispatch_key(repeat));
    assert!(tree.dispatch_key(KeyEvent::release(right, Modifiers::default())));
    assert_eq!(calls.borrow().clone(), vec![("change", 0.05)]);
}

#[test]
fn widgets_slider_07_state_layer() {
    let calls: Calls = Rc::default();
    let (mut tester, _) = bound(&calls);
    assert_eq!(layer(&tester), None);
    tester.hover(&Finder::key("s"));
    assert_eq!(layer(&tester), Some(Color { a: 0.08, ..PRIMARY }));
    // The layer is a 48 px circle on the thumb, painted after the tracks and before the thumb.
    let shapes = fills(&tester);
    assert_eq!(shapes[2].0, circle(66.0, 24.0, 24.0));
    assert_eq!(shapes[3].0, circle(66.0, 24.0, 10.0));
    // Dragging: 12 %, and focus (from the press) is 10 % after the release.
    down(&mut tester, 66.0);
    assert_eq!(layer_alpha(&tester), Some(0.12));
    up(&mut tester, 66.0);
    assert_eq!(layer_alpha(&tester), Some(0.10));
    // Leaving keeps the focus layer; moving the focus away removes it.
    mv(&mut tester, 150.0);
    mv(&mut tester, 150.0);
    pointer(&mut tester, PointerKind::Move, 150.0);
    tester.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(150.0, 150.0),
        modifiers: Default::default(),
    });
    tester.pump();
    assert_eq!(layer_alpha(&tester), Some(0.10));
    key(&mut tester, NamedKey::Tab);
    assert_eq!(layer(&tester), None);
}

#[test]
fn widgets_slider_08_reactive() {
    type Signals = (Signal<f64>, Signal<f64>, Signal<f64>, Signal<bool>);
    let out: Rc<Cell<Option<Signals>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let (value, min, max, enabled) = (
                signal(50.0f64),
                signal(0.0f64),
                signal(100.0f64),
                signal(true),
            );
            out.set(Some((value, min, max, enabled)));
            Column::new().child(Keyed::new(
                "s",
                Slider::new(value)
                    .range(min, max)
                    .enabled(enabled)
                    .on_change(|_| {}),
            ))
        })
    };
    let (value, min, max, enabled) = out.get().expect("built");
    assert_eq!(thumb_x(&tester), Some(150.0));
    tester.enter(|| value.set(75.0));
    assert!(tester.pump());
    assert_eq!(thumb_x(&tester), Some(220.0));
    tester.enter(|| {
        min.set(50.0);
        max.set(150.0);
    });
    assert!(tester.pump());
    assert_eq!(thumb_x(&tester), Some(80.0));
    assert_eq!(fills(&tester)[0].1, PRIMARY);
    tester.enter(|| enabled.set(false));
    assert!(tester.pump());
    assert_eq!(fills(&tester)[0].1, ON_SURFACE_38);
    assert!(!tester.pump());
    assert_eq!(
        tester.rect(tester.find(&Finder::key("s"))),
        Rect::from_ltrb(0.0, 0.0, 300.0, 48.0)
    );
}
