//! Tests for `docs/specs/widgets/button-keyboard.md`, rules WIDGETS-BUTTON-05..07.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Color, Rect, Size};
use tantu_view::layout::{BoxConstraints, NoTextMeasure};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{BorderRadius, Command, RoundedRect, Scene};
use tantu_view::{
    BuildCx, ElementId, Handled, KeyEvent, Keyed, LogicalKey, Modifiers, NamedKey, Phase,
    PointerButton, PointerEvent, PointerKind, View, ViewTree,
};
use tantu_widgets::{Button, Column};

const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);

fn space() -> LogicalKey {
    LogicalKey::Character(" ".into())
}

fn enter() -> LogicalKey {
    LogicalKey::Named(NamedKey::Enter)
}

fn tab() -> LogicalKey {
    LogicalKey::Named(NamedKey::Tab)
}

/// Buttons a (enabled), b (disabled) and c (enabled while `c_on`), counting presses.
struct Three {
    counts: Rc<[Cell<u32>; 3]>,
    c_on: Signal<bool>,
}

impl View for Three {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let counter = |i: usize| {
            let counts = self.counts.clone();
            move || counts[i].set(counts[i].get() + 1)
        };
        Column::new()
            .spacing(8.0)
            .child(Keyed::new("a", Button::new("A").on_press(counter(0))))
            .child(Keyed::new(
                "b",
                Button::new("B").enabled(false).on_press(counter(1)),
            ))
            .child(Keyed::new(
                "c",
                Button::new("C").enabled(self.c_on).on_press(counter(2)),
            ))
            .build(cx)
    }
}

type Handles = (Rc<[Cell<u32>; 3]>, Signal<bool>);

fn three() -> (WidgetTester, Handles) {
    let handles: Rc<RefCell<Option<Handles>>> = Rc::default();
    let h = handles.clone();
    let tester = WidgetTester::with_size(300.0, 300.0, move || {
        let counts: Rc<[Cell<u32>; 3]> = Rc::new(Default::default());
        let c_on = signal(true);
        *h.borrow_mut() = Some((counts.clone(), c_on));
        Three { counts, c_on }
    });
    let handles = handles.borrow_mut().take().expect("built");
    (tester, handles)
}

/// The focus outline strokes of `element` in the last frame.
fn outlines(tester: &WidgetTester, key: &str) -> Vec<(RoundedRect, f32, Color)> {
    let element = tester.find(&Finder::key(key));
    tester
        .frame()
        .entries_for(element)
        .filter_map(|e| match e.command {
            Command::Stroke {
                shape,
                width,
                color,
            } => Some((shape, width, color)),
            _ => None,
        })
        .collect()
}

#[test]
fn widgets_button_05_focusable_and_traversable() {
    let (mut tester, (counts, c_on)) = three();
    tester.press_key(tab(), Modifiers::default());
    assert_eq!(outlines(&tester, "a").len(), 1);
    // b is disabled: Tab skips it.
    tester.press_key(tab(), Modifiers::default());
    assert!(outlines(&tester, "a").is_empty());
    assert!(outlines(&tester, "b").is_empty());
    assert_eq!(outlines(&tester, "c").len(), 1);
    let not_focusable = catch_unwind(AssertUnwindSafe(|| tester.focus(&Finder::key("b"))));
    assert!(not_focusable.is_err());
    // c becomes disabled: it loses focus.
    tester.enter(|| c_on.set(false));
    tester.pump();
    assert!(outlines(&tester, "c").is_empty());
    tester.press_key(enter(), Modifiers::default());
    assert_eq!(counts[2].get(), 0);
    // A click doesn't focus a button.
    let mut tester = three().0;
    tester.tap(&Finder::key("a"));
    assert!(outlines(&tester, "a").is_empty());
}

#[test]
fn widgets_button_06_space_and_enter_activate() {
    let (mut tester, (counts, _)) = three();
    tester.focus(&Finder::key("a"));
    tester.press_key(space(), Modifiers::default());
    assert_eq!(counts[0].get(), 1);
    tester.press_key(enter(), Modifiers::default());
    assert_eq!(counts[0].get(), 2);
    tester.press_key(LogicalKey::Character("x".into()), Modifiers::default());
    assert_eq!(counts[0].get(), 2);

    // Raw events: releases and repeats don't press; a handled key stops; a disabled
    // (still focused, before the frame applies it) button lets the key through.
    let count = Rc::new(Cell::new(0u32));
    let seen = Rc::new(Cell::new(0u32));
    let sig: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let mut tree = {
        let (count, seen, sig) = (count.clone(), seen.clone(), sig.clone());
        ViewTree::new(move || Probe { count, seen, sig })
    };
    tree.layout(
        BoxConstraints::loose(Size::new(300.0, 100.0)),
        &mut NoTextMeasure,
    );
    let button = tree.children(tree.children(tree.root())[0])[0];
    assert!(tree.focus(button));
    tree.dispatch_key(KeyEvent::release(enter(), Modifiers::default()));
    let mut repeat = KeyEvent::press(enter(), Modifiers::default());
    repeat.repeat = true;
    tree.dispatch_key(repeat);
    assert_eq!(count.get(), 0);
    // The ignored repeat bubbled on to the outer handler.
    assert_eq!(seen.get(), 1);
    tree.dispatch_key(KeyEvent::press(enter(), Modifiers::default()));
    assert_eq!((count.get(), seen.get()), (1, 1));
    let on = sig.get().expect("built");
    tree.enter(|| on.set(false));
    tree.dispatch_key(KeyEvent::press(enter(), Modifiers::default()));
    assert_eq!((count.get(), seen.get()), (1, 2));
}

/// A button inside a box that counts the Enter presses that bubble past the button.
struct Probe {
    count: Rc<Cell<u32>>,
    seen: Rc<Cell<u32>>,
    sig: Rc<Cell<Option<Signal<bool>>>>,
}

impl View for Probe {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let on = signal(true);
        self.sig.set(Some(on));
        let count = self.count;
        let outer = Column::new()
            .child(
                Button::new("Go")
                    .enabled(on)
                    .on_press(move || count.set(count.get() + 1)),
            )
            .build(cx);
        let seen = self.seen;
        cx.on_key(outer, Phase::Bubble, move |k| {
            if k.event.pressed && k.event.key == LogicalKey::Named(NamedKey::Enter) {
                seen.set(seen.get() + 1);
            }
            Handled::Continue
        });
        outer
    }
}

fn over_white(color: Color, a: f32) -> Color {
    let mix = |c: f32| c * (1.0 - a) + a;
    Color {
        r: mix(color.r),
        g: mix(color.g),
        b: mix(color.b),
        a: color.a,
    }
}

fn close(a: Color, b: Color) -> bool {
    [a.r - b.r, a.g - b.g, a.b - b.b, a.a - b.a]
        .iter()
        .all(|d| d.abs() < 1e-5)
}

fn fill(tester: &WidgetTester, key: &str) -> Color {
    let element = tester.find(&Finder::key(key));
    tester
        .frame()
        .entries_for(element)
        .find_map(|e| match e.command {
            Command::Fill { color, .. } => Some(color),
            _ => None,
        })
        .expect("a fill")
}

#[test]
fn widgets_button_07_focus_indicator() {
    let (mut tester, _) = three();
    let a = tester.find(&Finder::key("a"));
    let size = tester.rect(a).size();
    assert!(outlines(&tester, "a").is_empty());
    tester.focus(&Finder::key("a"));
    let expected = RoundedRect::new(
        Rect::from_ltwh(-5.0, -5.0, size.width + 10.0, size.height + 10.0),
        BorderRadius::circular(25.0),
    );
    assert_eq!(outlines(&tester, "a"), [(expected, 3.0, PRIMARY)]);
    assert!(close(fill(&tester, "a"), over_white(PRIMARY, 0.10)));
    // Hovered and focused: the strongest layer (focus, 10 %).
    tester.hover(&Finder::key("a"));
    assert!(close(fill(&tester, "a"), over_white(PRIMARY, 0.10)));
    // Pressed and focused: 12 %.
    let center = tester.rect(a).center();
    tester.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: center,
    });
    tester.pump();
    assert!(close(fill(&tester, "a"), over_white(PRIMARY, 0.12)));
    let _ = Scene::new();
}
