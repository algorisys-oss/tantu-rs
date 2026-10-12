//! Tests for `docs/specs/widgets/toggles.md`, rules WIDGETS-TOGGLE-01..09.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Color, Point, Rect, Size};
use tantu_view::layout::{BoxConstraints, NoTextMeasure};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{BorderRadius, Command, RoundedRect, Scene};
use tantu_view::{
    Key, KeyEvent, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent,
    PointerKind, View, ViewTree,
};
use tantu_widgets::{Checkbox, Column, Radio, Row, Switch};

const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);
const ON_SURFACE: Color = Color::from_argb32(0xFF1C_1B1F);
const ON_SURFACE_VARIANT: Color = Color::from_argb32(0xFF49_454F);
const OUTLINE: Color = Color::from_argb32(0xFF79_747E);
const SURFACE_CONTAINER_HIGHEST: Color = Color::from_argb32(0xFFE6_E0E9);
const ON_SURFACE_38: Color = Color::from_argb32(0x611C_1B1F);
const ON_SURFACE_12: Color = Color::from_argb32(0x1F1C_1B1F);

/// The drawing commands the toggle keyed `key` painted in the last frame (the transform scope
/// the tree opens around every render element belongs to its parent, so it isn't included).
fn commands_of(tester: &WidgetTester, key: &str) -> Vec<Command> {
    let toggle = tester.find(&Finder::key(key));
    tester
        .frame()
        .entries_for(toggle)
        .filter(|e| {
            matches!(
                e.command,
                Command::Fill { .. }
                    | Command::Stroke { .. }
                    | Command::PushTransform(_)
                    | Command::PopTransform
            )
        })
        .map(|e| e.command.clone())
        .collect()
}

/// The commands of the toggle keyed "t".
fn commands(tester: &WidgetTester) -> Vec<Command> {
    commands_of(tester, "t")
}

fn circle(cx: f32, cy: f32, r: f32) -> RoundedRect {
    RoundedRect::new(
        Rect::from_ltrb(cx - r, cy - r, cx + r, cy + r),
        BorderRadius::circular(r),
    )
}

fn fill(shape: RoundedRect, color: Color) -> Command {
    Command::Fill { shape, color }
}

fn stroke(shape: RoundedRect, color: Color) -> Command {
    Command::Stroke {
        shape,
        width: 2.0,
        color,
    }
}

/// The checkbox's 18 px box in a 40 × 40 element.
fn checkbox_box() -> RoundedRect {
    RoundedRect::new(
        Rect::from_ltrb(11.0, 11.0, 29.0, 29.0),
        BorderRadius::circular(2.0),
    )
}

/// The switch's track in a 60 × 40 element.
fn track() -> RoundedRect {
    RoundedRect::new(
        Rect::from_ltrb(4.0, 4.0, 56.0, 36.0),
        BorderRadius::circular(16.0),
    )
}

/// The state layer's color (the radius-20 circle), if drawn.
fn layer(tester: &WidgetTester) -> Option<Color> {
    commands(tester).iter().find_map(|c| match c {
        Command::Fill { shape, color } if shape.rect.width() == 40.0 => Some(*color),
        _ => None,
    })
}

fn layer_alpha(tester: &WidgetTester) -> Option<f32> {
    layer(tester).map(|c| c.a)
}

/// A 300 × 200 window showing `view` in a column, keyed "t".
fn window(view: impl View + 'static) -> WidgetTester {
    let view = Cell::new(Some(view));
    WidgetTester::with_size(300.0, 200.0, move || {
        Column::new().child(Keyed::new("t", view.take().expect("built once")))
    })
}

type Calls<T> = Rc<RefCell<Vec<T>>>;

fn recorder<T: 'static>() -> (Calls<T>, impl Fn(T) + 'static) {
    let calls: Calls<T> = Rc::default();
    let sink = calls.clone();
    (calls, move |v| sink.borrow_mut().push(v))
}

fn at(tester: &mut WidgetTester, kind: PointerKind, p: Point) {
    tester.pointer(PointerEvent {
        kind,
        position: p,
        modifiers: Default::default(),
    });
    tester.pump();
}

fn center(tester: &WidgetTester) -> Point {
    tester.rect(tester.find(&Finder::key("t"))).center()
}

fn key(tester: &mut WidgetTester, key: LogicalKey) {
    tester.press_key(key, Modifiers::default());
}

fn space() -> LogicalKey {
    LogicalKey::Character(" ".into())
}

fn enter() -> LogicalKey {
    LogicalKey::Named(NamedKey::Enter)
}

fn close(a: Point, b: Point) -> bool {
    (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() < 0.01
}

#[test]
fn widgets_toggle_01_sizes() {
    let size = |tester: &WidgetTester| tester.rect(tester.find(&Finder::key("t"))).size();
    assert_eq!(size(&window(Checkbox::new(false))), Size::new(40.0, 40.0));
    assert_eq!(size(&window(Radio::new(1, || 1))), Size::new(40.0, 40.0));
    assert_eq!(size(&window(Switch::new(false))), Size::new(60.0, 40.0));
}

#[test]
fn widgets_toggle_02_checkbox_look() {
    let tester = window(Checkbox::new(false).on_change(|_| {}));
    assert_eq!(
        commands(&tester),
        vec![stroke(checkbox_box(), ON_SURFACE_VARIANT)]
    );
    // Selected: the box, then two bars in transform scopes, joining Flutter's check points.
    let tester = window(Checkbox::new(true).on_change(|_| {}));
    let cmds = commands(&tester);
    assert_eq!(cmds.len(), 7);
    assert_eq!(cmds[0], fill(checkbox_box(), PRIMARY));
    let points = [
        Point::new(11.0 + 0.15 * 18.0, 11.0 + 0.45 * 18.0),
        Point::new(11.0 + 0.40 * 18.0, 11.0 + 0.70 * 18.0),
        Point::new(11.0 + 0.85 * 18.0, 11.0 + 0.25 * 18.0),
    ];
    for (i, bar) in [&cmds[1..4], &cmds[4..7]].into_iter().enumerate() {
        let Command::PushTransform(transform) = bar[0] else {
            panic!("bar {i}: {:?}", bar[0]);
        };
        let Command::Fill { shape, color } = bar[1] else {
            panic!("bar {i}: {:?}", bar[1]);
        };
        assert_eq!(bar[2], Command::PopTransform);
        assert_eq!(color, Color::WHITE);
        assert_eq!((shape.rect.top, shape.rect.bottom), (-1.0, 1.0));
        // The bar extends 1 px past the segment at both ends.
        let from = transform * Point::new(shape.rect.left + 1.0, 0.0);
        let to = transform * Point::new(shape.rect.right - 1.0, 0.0);
        assert!(
            close(from, points[i]),
            "bar {i}: {from:?} vs {:?}",
            points[i]
        );
        assert!(
            close(to, points[i + 1]),
            "bar {i}: {to:?} vs {:?}",
            points[i + 1]
        );
        assert!(transform.is_finite());
    }
    // Indeterminate: the filled box and a dash, whatever the value.
    for value in [false, true] {
        let tester = window(Checkbox::new(value).indeterminate(true).on_change(|_| {}));
        let cmds = commands(&tester);
        assert_eq!(cmds.len(), 2, "{value}");
        assert_eq!(cmds[0], fill(checkbox_box(), PRIMARY));
        let Command::Fill { shape, color } = cmds[1] else {
            panic!("{:?}", cmds[1]);
        };
        assert_eq!(color, Color::WHITE);
        assert!(close(
            Point::new(shape.rect.left, shape.rect.top),
            Point::new(11.0 + 0.2 * 18.0, 19.0)
        ));
        assert!(close(
            Point::new(shape.rect.right, shape.rect.bottom),
            Point::new(11.0 + 0.8 * 18.0, 21.0)
        ));
    }
    let mut tester = WidgetTester::with_size(160.0, 40.0, || {
        Row::new()
            .child(Checkbox::new(false).on_change(|_| {}))
            .child(Checkbox::new(true).on_change(|_| {}))
            .child(Checkbox::new(false).indeterminate(true).on_change(|_| {}))
            .child(Checkbox::new(true))
    });
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/checkbox.png"
    ));
}

#[test]
fn widgets_toggle_03_radio_look() {
    let tester = window(Radio::new('a', || 'b').on_change(|_| {}));
    assert_eq!(
        commands(&tester),
        vec![stroke(circle(20.0, 20.0, 8.0), ON_SURFACE_VARIANT)]
    );
    let tester = window(Radio::new('a', || 'a').on_change(|_| {}));
    assert_eq!(
        commands(&tester),
        vec![
            stroke(circle(20.0, 20.0, 8.0), PRIMARY),
            fill(circle(20.0, 20.0, 4.5), PRIMARY),
        ]
    );
    let mut tester = WidgetTester::with_size(120.0, 40.0, || {
        Row::new()
            .child(Radio::new(1, || 2).on_change(|_| {}))
            .child(Radio::new(1, || 1).on_change(|_| {}))
            .child(Radio::new(1, || 1))
    });
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/radio.png"
    ));
}

#[test]
fn widgets_toggle_04_switch_look() {
    let tester = window(Switch::new(false).on_change(|_| {}));
    assert_eq!(
        commands(&tester),
        vec![
            fill(track(), SURFACE_CONTAINER_HIGHEST),
            stroke(track(), OUTLINE),
            fill(circle(20.0, 20.0, 8.0), OUTLINE),
        ]
    );
    let mut tester = window(Switch::new(true).on_change(|_| {}));
    assert_eq!(
        commands(&tester),
        vec![
            fill(track(), PRIMARY),
            fill(circle(40.0, 20.0, 12.0), Color::WHITE),
        ]
    );
    // Pressed: a bigger thumb (after the state layer).
    let c = center(&tester);
    at(&mut tester, PointerKind::Down(PointerButton::Primary), c);
    assert_eq!(
        commands(&tester)[1..],
        [
            fill(track(), PRIMARY),
            fill(circle(40.0, 20.0, 14.0), Color::WHITE),
        ]
    );
    at(&mut tester, PointerKind::Up(PointerButton::Primary), c);
    let mut tester = WidgetTester::with_size(240.0, 40.0, || {
        Row::new()
            .child(Switch::new(false).on_change(|_| {}))
            .child(Switch::new(true).on_change(|_| {}))
            .child(Switch::new(false))
            .child(Switch::new(true))
    });
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/switch.png"
    ));
}

#[test]
fn widgets_toggle_05_disabled() {
    // The disabled looks: no `on_change`, or `enabled(false)`.
    let tester = window(Checkbox::new(true));
    let cmds = commands(&tester);
    assert_eq!(cmds.len(), 7);
    assert_eq!(cmds[0], fill(checkbox_box(), ON_SURFACE_38));
    assert!(matches!(cmds[2], Command::Fill { color, .. } if color == Color::WHITE));
    let tester = window(Checkbox::new(false).enabled(false).on_change(|_| {}));
    assert_eq!(
        commands(&tester),
        vec![stroke(checkbox_box(), ON_SURFACE_38)]
    );
    let tester = window(Radio::new(1, || 1));
    assert_eq!(
        commands(&tester),
        vec![
            stroke(circle(20.0, 20.0, 8.0), ON_SURFACE_38),
            fill(circle(20.0, 20.0, 4.5), ON_SURFACE_38),
        ]
    );
    let tester = window(Switch::new(false));
    assert_eq!(
        commands(&tester),
        vec![
            fill(track(), ON_SURFACE_12),
            stroke(track(), ON_SURFACE_12),
            fill(circle(20.0, 20.0, 8.0), ON_SURFACE_38),
        ]
    );
    let tester = window(Switch::new(true).enabled(false).on_change(|_| {}));
    assert_eq!(
        commands(&tester),
        vec![
            fill(track(), ON_SURFACE_12),
            fill(circle(40.0, 20.0, 12.0), ON_SURFACE_38),
        ]
    );
    // Input is ignored, nothing is reported, no layer, not traversable.
    let (calls, record) = recorder::<bool>();
    let mut tester = window(Checkbox::new(false).enabled(false).on_change(record));
    tester.hover(&Finder::key("t"));
    tester.tap(&Finder::key("t"));
    key(&mut tester, LogicalKey::Named(NamedKey::Tab));
    key(&mut tester, space());
    assert!(calls.borrow().is_empty());
    assert_eq!(layer(&tester), None);
    let (calls, record) = recorder::<bool>();
    let mut tester = WidgetTester::with_size(300.0, 200.0, move || {
        Column::new()
            .child(Checkbox::new(false).enabled(false).on_change(|_| {}))
            .child(Keyed::new("t", Checkbox::new(false).on_change(record)))
    });
    key(&mut tester, LogicalKey::Named(NamedKey::Tab));
    assert_eq!(layer_alpha(&tester), Some(0.10));
    key(&mut tester, space());
    assert_eq!(calls.borrow().clone(), vec![true]);
    // Becoming disabled clears hover, pressed and focus; re-enabling brings none back.
    let out: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let enabled = signal(true);
            out.set(Some(enabled));
            Column::new().child(Keyed::new(
                "t",
                Switch::new(true).enabled(enabled).on_change(|_| {}),
            ))
        })
    };
    let enabled = out.get().expect("built");
    tester.focus(&Finder::key("t"));
    tester.hover(&Finder::key("t"));
    let c = center(&tester);
    at(&mut tester, PointerKind::Down(PointerButton::Primary), c);
    assert_eq!(layer_alpha(&tester), Some(0.12));
    tester.enter(|| enabled.set(false));
    tester.pump();
    assert_eq!(
        commands(&tester),
        vec![
            fill(track(), ON_SURFACE_12),
            fill(circle(40.0, 20.0, 12.0), ON_SURFACE_38),
        ]
    );
    tester.enter(|| enabled.set(true));
    tester.pump();
    assert_eq!(layer(&tester), None);
    assert_eq!(
        commands(&tester)[1],
        fill(circle(40.0, 20.0, 12.0), Color::WHITE)
    );
}

#[test]
fn widgets_toggle_06_pointer() {
    // A checkbox reports !value on a release inside; the value it shows is the app's.
    let (calls, record) = recorder::<bool>();
    let out: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let value = signal(false);
            out.set(Some(value));
            Column::new().child(Keyed::new(
                "t",
                Checkbox::new(value).on_change(move |v| {
                    record(v);
                    value.set(v);
                }),
            ))
        })
    };
    tester.tap(&Finder::key("t"));
    tester.tap(&Finder::key("t"));
    assert_eq!(calls.borrow().clone(), vec![true, false]);
    assert_eq!(out.get().map(|v| tester.enter(|| v.get())), Some(false));
    // A release outside, or another button, reports nothing.
    let c = center(&tester);
    at(&mut tester, PointerKind::Down(PointerButton::Primary), c);
    at(
        &mut tester,
        PointerKind::Up(PointerButton::Primary),
        Point::new(200.0, 150.0),
    );
    at(&mut tester, PointerKind::Down(PointerButton::Secondary), c);
    at(&mut tester, PointerKind::Up(PointerButton::Secondary), c);
    assert_eq!(calls.borrow().len(), 2);
    // Indeterminate still reports !value.
    let (calls, record) = recorder::<bool>();
    let mut tester = window(Checkbox::new(true).indeterminate(true).on_change(record));
    tester.tap(&Finder::key("t"));
    assert_eq!(calls.borrow().clone(), vec![false]);
    // A radio reports its own value, selected or not.
    let (calls, record) = recorder::<char>();
    let mut tester = window(Radio::new('b', || 'b').on_change(record));
    tester.tap(&Finder::key("t"));
    tester.tap(&Finder::key("t"));
    assert_eq!(calls.borrow().clone(), vec!['b', 'b']);
    // A switch reports !value.
    let (calls, record) = recorder::<bool>();
    let mut tester = window(Switch::new(true).on_change(record));
    tester.tap(&Finder::key("t"));
    assert_eq!(calls.borrow().clone(), vec![false]);
}

#[test]
fn widgets_toggle_07_keyboard() {
    let (calls, record) = recorder::<bool>();
    let mut tester = window(Checkbox::new(false).on_change(record));
    // A press doesn't focus it (only the hover layer shows after the tap); Tab does.
    tester.tap(&Finder::key("t"));
    assert_eq!(layer_alpha(&tester), Some(0.08));
    key(&mut tester, LogicalKey::Named(NamedKey::Tab));
    assert_eq!(layer_alpha(&tester), Some(0.10));
    key(&mut tester, space());
    key(&mut tester, enter());
    key(&mut tester, LogicalKey::Character("a".into()));
    assert_eq!(calls.borrow().clone(), vec![true, true, true]);
    let (calls, record) = recorder::<char>();
    let mut tester = window(Radio::new('x', || 'y').on_change(record));
    tester.focus(&Finder::key("t"));
    key(&mut tester, space());
    assert_eq!(calls.borrow().clone(), vec!['x']);
    let (calls, record) = recorder::<bool>();
    let mut tester = window(Switch::new(false).on_change(record));
    tester.focus(&Finder::key("t"));
    key(&mut tester, enter());
    assert_eq!(calls.borrow().clone(), vec![true]);
    // Repeats and releases don't activate; the press is stopped.
    let (calls, record) = recorder::<bool>();
    let mut tree = ViewTree::new(move || Keyed::new("t", Switch::new(false).on_change(record)));
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::tight(Size::new(300.0, 200.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    let switch = tree.find_key(&Key::from("t"))[0];
    assert!(tree.focus(switch));
    let repeat = KeyEvent {
        repeat: true,
        ..KeyEvent::press(space(), Modifiers::default())
    };
    tree.dispatch_key(repeat);
    tree.dispatch_key(KeyEvent::release(space(), Modifiers::default()));
    assert!(calls.borrow().is_empty());
    assert!(tree.dispatch_key(KeyEvent::press(space(), Modifiers::default())));
    assert_eq!(calls.borrow().clone(), vec![true]);
}

#[test]
fn widgets_toggle_08_state_layer() {
    // Unselected: on-surface; selected: primary. Hover 8 %, focus 10 %, pressed 12 %.
    let mut tester = window(Checkbox::new(false).on_change(|_| {}));
    assert_eq!(layer(&tester), None);
    tester.hover(&Finder::key("t"));
    assert_eq!(layer(&tester), Some(ON_SURFACE.with_alpha(0.08)));
    assert_eq!(
        commands(&tester)[0],
        fill(circle(20.0, 20.0, 20.0), ON_SURFACE.with_alpha(0.08))
    );
    let c = center(&tester);
    at(&mut tester, PointerKind::Down(PointerButton::Primary), c);
    assert_eq!(layer_alpha(&tester), Some(0.12));
    at(&mut tester, PointerKind::Up(PointerButton::Primary), c);
    assert_eq!(layer_alpha(&tester), Some(0.08));
    at(&mut tester, PointerKind::Move, Point::new(200.0, 150.0));
    assert_eq!(layer(&tester), None);
    tester.focus(&Finder::key("t"));
    assert_eq!(layer_alpha(&tester), Some(0.10));
    tester.hover(&Finder::key("t"));
    assert_eq!(layer_alpha(&tester), Some(0.10));
    let mut tester = window(Checkbox::new(true).on_change(|_| {}));
    tester.hover(&Finder::key("t"));
    assert_eq!(layer(&tester), Some(PRIMARY.with_alpha(0.08)));
    // The switch's layer sits on the thumb.
    let mut tester = window(Switch::new(true).on_change(|_| {}));
    tester.hover(&Finder::key("t"));
    assert_eq!(
        commands(&tester)[0],
        fill(circle(40.0, 20.0, 20.0), PRIMARY.with_alpha(0.08))
    );
    let mut tester = window(Switch::new(false).on_change(|_| {}));
    tester.hover(&Finder::key("t"));
    assert_eq!(
        commands(&tester)[0],
        fill(circle(20.0, 20.0, 20.0), ON_SURFACE.with_alpha(0.08))
    );
}

#[test]
fn widgets_toggle_09_reactive() {
    type Signals = (Signal<bool>, Signal<bool>, Signal<char>, Signal<bool>);
    let out: Rc<Cell<Option<Signals>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let (value, indeterminate, group, enabled) =
                (signal(false), signal(false), signal('a'), signal(true));
            out.set(Some((value, indeterminate, group, enabled)));
            Column::new()
                .child(Keyed::new(
                    "c",
                    Checkbox::new(value)
                        .indeterminate(indeterminate)
                        .enabled(enabled)
                        .on_change(|_| {}),
                ))
                .child(Keyed::new("r", Radio::new('b', group).on_change(|_| {})))
                .child(Keyed::new("s", Switch::new(value).on_change(|_| {})))
        })
    };
    let (value, indeterminate, group, enabled) = out.get().expect("built");
    let count = |tester: &WidgetTester| {
        ["c", "r", "s"]
            .iter()
            .map(|key| commands_of(tester, key).len())
            .sum::<usize>()
    };
    assert_eq!(count(&tester), 1 + 1 + 3);
    tester.enter(|| value.set(true));
    assert!(tester.pump());
    assert_eq!(count(&tester), 7 + 1 + 2);
    tester.enter(|| indeterminate.set(true));
    assert!(tester.pump());
    assert_eq!(count(&tester), 2 + 1 + 2);
    tester.enter(|| group.set('b'));
    assert!(tester.pump());
    assert_eq!(count(&tester), 2 + 2 + 2);
    tester.enter(|| enabled.set(false));
    assert!(tester.pump());
    assert_eq!(
        commands_of(&tester, "c")[0],
        fill(checkbox_box(), ON_SURFACE_38)
    );
    assert!(!tester.pump());
}
