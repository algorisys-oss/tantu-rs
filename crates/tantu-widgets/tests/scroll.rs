//! Tests for `docs/specs/widgets/scroll.md`, rules WIDGETS-SCROLL-01..08.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use tantu_test::{Finder, WidgetTester};
use tantu_view::Curve;
use tantu_view::core::{Color, Point, Rect, Vec2};
use tantu_view::scene::Command;
use tantu_view::{
    Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind, View,
};
use tantu_widgets::{Button, Column, Divider, ScrollController, ScrollView, SizedBox};

const THUMB: Color = Color::from_argb32(0x661C_1B1F);
const OUTLINE_VARIANT: Color = Color::from_argb32(0xFFCA_C4D0);

type Out = Rc<Cell<Option<ScrollController>>>;

/// A 300 × 200 window with a scroll view keyed "s" around `content`, exposing its controller.
fn view(make: impl FnOnce() -> ScrollView + 'static) -> (WidgetTester, ScrollController) {
    let out: Out = Rc::default();
    let make = Cell::new(Some(make));
    let tester = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let controller = ScrollController::new();
            out.set(Some(controller));
            let make = make.take().expect("built once");
            Keyed::new("s", make().controller(controller))
        })
    };
    let controller = out.get().expect("built");
    (tester, controller)
}

/// A 1000 px tall content, with wheel steps applied at once (the pre-animation rules).
fn tall() -> ScrollView {
    ScrollView::new(Keyed::new("c", SizedBox::new(300.0, 1000.0))).wheel_duration(Duration::ZERO)
}

fn offset(tester: &WidgetTester, c: ScrollController) -> f32 {
    tester.enter(|| c.offset())
}

fn fills(tester: &WidgetTester, color: Color) -> Vec<Rect> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { shape, color: c } if c == color => Some(shape.rect),
            _ => None,
        })
        .collect()
}

#[test]
fn widgets_scroll_01_structure_and_controller() {
    let (mut tester, c) = view(tall);
    assert_eq!(
        tester.rect(tester.find(&Finder::key("s"))).size(),
        tantu_view::core::Size::new(300.0, 200.0)
    );
    assert_eq!(
        tester.enter(|| (c.offset(), c.max_scroll_extent(), c.viewport_extent())),
        (0.0, 800.0, 200.0)
    );
    assert_eq!(tester.rect(tester.find(&Finder::key("c"))).top, 0.0);
    tester.enter(|| c.jump_to(5000.0));
    assert!(tester.pump());
    assert_eq!(offset(&tester, c), 800.0);
    assert_eq!(tester.rect(tester.find(&Finder::key("c"))).top, -800.0);
    tester.enter(|| c.jump_to(-10.0));
    tester.pump();
    assert_eq!(offset(&tester, c), 0.0);
    tester.enter(|| c.jump_to(100.0));
    tester.pump();
    assert_eq!(tester.rect(tester.find(&Finder::key("c"))).top, -100.0);
    // A fresh controller before any frame.
    let fresh = tester.enter(ScrollController::new);
    assert_eq!(
        tester.enter(|| (fresh.offset(), fresh.max_scroll_extent())),
        (0.0, 0.0)
    );
    // Horizontal.
    let (tester, c) = view(|| ScrollView::new(SizedBox::new(1000.0, 100.0)).horizontal());
    assert_eq!(
        tester.enter(|| (c.max_scroll_extent(), c.viewport_extent())),
        (700.0, 300.0)
    );
}

#[test]
fn widgets_scroll_02_wheel() {
    let (mut tester, c) = view(tall);
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, 50.0));
    assert_eq!(offset(&tester, c), 50.0);
    tester.scroll(&Finder::key("s"), Vec2::new(30.0, 25.0));
    assert_eq!(offset(&tester, c), 75.0);
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, 5000.0));
    assert_eq!(offset(&tester, c), 800.0);
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, -5000.0));
    assert_eq!(offset(&tester, c), 0.0);
    // A horizontal view ignores vertical motion.
    let (mut tester, c) = view(|| {
        ScrollView::new(SizedBox::new(1000.0, 100.0))
            .horizontal()
            .wheel_duration(Duration::ZERO)
    });
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, 50.0));
    assert_eq!(offset(&tester, c), 0.0);
    tester.scroll(&Finder::key("s"), Vec2::new(40.0, 0.0));
    assert_eq!(offset(&tester, c), 40.0);
    // Nested: the inner view scrolls until its end, then the outer one.
    let inner_out: Out = Rc::default();
    let (mut tester, outer) = {
        let inner_out = inner_out.clone();
        view(move || {
            let inner = ScrollController::new();
            inner_out.set(Some(inner));
            ScrollView::new(
                Column::new()
                    .child(SizedBox::new(300.0, 50.0))
                    .child(Keyed::new(
                        "inner",
                        SizedBox::new(300.0, 100.0).child(
                            ScrollView::new(SizedBox::new(300.0, 400.0))
                                .controller(inner)
                                .wheel_duration(Duration::ZERO),
                        ),
                    ))
                    .child(SizedBox::new(300.0, 1000.0)),
            )
            .wheel_duration(Duration::ZERO)
        })
    };
    let inner = inner_out.get().expect("built");
    tester.scroll(&Finder::key("inner"), Vec2::new(0.0, 100.0));
    assert_eq!(
        (offset(&tester, inner), offset(&tester, outer)),
        (100.0, 0.0)
    );
    tester.scroll(&Finder::key("inner"), Vec2::new(0.0, 500.0));
    assert_eq!(
        (offset(&tester, inner), offset(&tester, outer)),
        (300.0, 0.0)
    );
    tester.scroll(&Finder::key("inner"), Vec2::new(0.0, 60.0));
    assert_eq!(
        (offset(&tester, inner), offset(&tester, outer)),
        (300.0, 60.0)
    );
}

#[test]
fn widgets_scroll_03_clipping_and_culling() {
    let (mut tester, c) = view(|| {
        ScrollView::new(Column::new().children((0..50).map(|_| Divider::new().height(40.0))))
    });
    let visible = fills(&tester, OUTLINE_VARIANT).len();
    assert!((5..=7).contains(&visible), "{visible}");
    // The content is clipped to the view.
    let clip = tester
        .frame()
        .scene
        .entries()
        .iter()
        .any(|e| matches!(e.command, Command::PushClip(_)));
    assert!(clip);
    tester.enter(|| c.jump_to(1000.0));
    tester.pump();
    let visible = fills(&tester, OUTLINE_VARIANT).len();
    assert!((5..=7).contains(&visible), "{visible}");
    assert_eq!(tester.enter(|| c.max_scroll_extent()), 1800.0);
}

#[test]
fn widgets_scroll_04_scrollbar() {
    let (mut tester, c) = view(tall);
    let thumb = fills(&tester, THUMB);
    assert_eq!(thumb.len(), 1);
    let thumb = thumb[0];
    assert_eq!((thumb.left, thumb.right), (290.0, 298.0));
    assert!((thumb.top - 2.0).abs() < 0.01 && (thumb.height() - 39.2).abs() < 0.01);
    tester.enter(|| c.jump_to(800.0));
    tester.pump();
    let thumb = fills(&tester, THUMB)[0];
    assert!((thumb.bottom - 198.0).abs() < 0.01);
    // Dragging the thumb scrolls proportionally; pressing it doesn't scroll by itself.
    tester.enter(|| c.jump_to(0.0));
    tester.pump();
    let thumb = fills(&tester, THUMB)[0];
    let start = thumb.center();
    tester.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: start,
    });
    tester.pump();
    assert_eq!(offset(&tester, c), 0.0);
    tester.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(start.x, start.y + 49.0),
    });
    tester.pump();
    assert!(
        (offset(&tester, c) - 250.0).abs() < 0.01,
        "{}",
        offset(&tester, c)
    );
    tester.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(start.x, start.y + 1000.0),
    });
    tester.pointer(PointerEvent {
        kind: PointerKind::Up(PointerButton::Primary),
        position: Point::new(start.x, start.y + 1000.0),
    });
    tester.pump();
    assert_eq!(offset(&tester, c), 800.0);
    // No overflow, no scrollbar; a short thumb is at least 24 px.
    let (tester, _) = view(|| ScrollView::new(SizedBox::new(300.0, 100.0)));
    assert!(fills(&tester, THUMB).is_empty());
    let (tester, _) = view(|| ScrollView::new(SizedBox::new(300.0, 100_000.0)));
    assert!((fills(&tester, THUMB)[0].height() - 24.0).abs() < 0.01);
}

#[test]
fn widgets_scroll_05_hit_testing_follows_the_scroll() {
    let presses = Rc::new(Cell::new(0));
    let (mut tester, c) = {
        let presses = presses.clone();
        view(move || {
            let presses = presses.clone();
            ScrollView::new(
                Column::new()
                    .child(SizedBox::new(300.0, 500.0))
                    .child(Keyed::new(
                        "b",
                        Button::new("Go").on_press(move || presses.set(presses.get() + 1)),
                    ))
                    .child(SizedBox::new(300.0, 500.0)),
            )
        })
    };
    // Scrolled out of view: the press lands on nothing.
    tester.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: Point::new(150.0, 520.0),
    });
    tester.pointer(PointerEvent {
        kind: PointerKind::Up(PointerButton::Primary),
        position: Point::new(150.0, 520.0),
    });
    tester.pump();
    assert_eq!(presses.get(), 0);
    tester.enter(|| c.jump_to(400.0));
    tester.pump();
    let b = tester.rect(tester.find(&Finder::key("b")));
    assert_eq!(b.top, 100.0);
    tester.tap(&Finder::key("b"));
    assert_eq!(presses.get(), 1);
}

fn _assert_view<V: View>(_: V) {}

#[test]
fn widgets_scroll_06_keyboard() {
    let key = |tester: &mut WidgetTester, named: NamedKey| {
        tester.press_key(LogicalKey::Named(named), Modifiers::default());
    };
    let (mut tester, c) = view(tall);
    // Not focused: keys do nothing. A press focuses the view (Tab doesn't stop at it).
    key(&mut tester, NamedKey::ArrowDown);
    assert_eq!(offset(&tester, c), 0.0);
    key(&mut tester, NamedKey::Tab);
    key(&mut tester, NamedKey::ArrowDown);
    assert_eq!(offset(&tester, c), 0.0);
    tester.tap(&Finder::key("s"));
    key(&mut tester, NamedKey::ArrowDown);
    assert_eq!(offset(&tester, c), 40.0);
    key(&mut tester, NamedKey::ArrowUp);
    assert_eq!(offset(&tester, c), 0.0);
    key(&mut tester, NamedKey::PageDown);
    assert_eq!(offset(&tester, c), 200.0);
    key(&mut tester, NamedKey::End);
    assert_eq!(offset(&tester, c), 800.0);
    key(&mut tester, NamedKey::PageUp);
    assert_eq!(offset(&tester, c), 600.0);
    key(&mut tester, NamedKey::Home);
    assert_eq!(offset(&tester, c), 0.0);
    // Cross-axis arrows and other keys pass through (nothing moves).
    key(&mut tester, NamedKey::ArrowRight);
    tester.press_key(LogicalKey::Character("a".into()), Modifiers::default());
    assert_eq!(offset(&tester, c), 0.0);
    // Horizontal views use the horizontal arrows.
    let (mut tester, c) = view(|| {
        ScrollView::new(SizedBox::new(1000.0, 100.0))
            .horizontal()
            .wheel_duration(Duration::ZERO)
    });
    tester.tap(&Finder::key("s"));
    key(&mut tester, NamedKey::ArrowRight);
    assert_eq!(offset(&tester, c), 40.0);
    key(&mut tester, NamedKey::ArrowDown);
    assert_eq!(offset(&tester, c), 40.0);
}

/// A 1000 px tall content with the default (animated) wheel.
fn tall_smooth() -> ScrollView {
    ScrollView::new(Keyed::new("c", SizedBox::new(300.0, 1000.0)))
}

#[test]
fn widgets_scroll_07_smooth_wheel() {
    let ms = Duration::from_millis;
    let (mut tester, c) = view(tall_smooth);
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, 40.0));
    assert!(offset(&tester, c) < 40.0);
    tester.advance(ms(75));
    let half = offset(&tester, c);
    assert!(half > 20.0 && half < 40.0, "ease-out part way: {half}");
    tester.advance(ms(75));
    assert_eq!(offset(&tester, c), 40.0);
    // Steps accumulate.
    for _ in 0..3 {
        tester.scroll(&Finder::key("s"), Vec2::new(0.0, 40.0));
    }
    tester.advance(ms(150));
    assert_eq!(offset(&tester, c), 160.0);
    // Clamped at the end.
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, 5000.0));
    tester.advance(ms(150));
    assert_eq!(offset(&tester, c), 800.0);
    // A jump during the animation takes over.
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, -400.0));
    tester.advance(ms(50));
    tester.enter(|| c.jump_to(300.0));
    tester.pump();
    assert_eq!(offset(&tester, c), 300.0);
    tester.advance(ms(200));
    assert_eq!(offset(&tester, c), 300.0);
    // No overflow: the event continues (an outer view would scroll), nothing animates.
    let (mut tester, c) = view(|| ScrollView::new(SizedBox::new(300.0, 100.0)));
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, 40.0));
    tester.advance(ms(150));
    assert_eq!(offset(&tester, c), 0.0);
}

#[test]
fn widgets_scroll_08_animate_to() {
    let ms = Duration::from_millis;
    let (mut tester, c) = view(tall_smooth);
    tester.enter(|| c.animate_to(400.0, ms(200), Curve::Linear));
    tester.advance(ms(100));
    assert_eq!(offset(&tester, c), 200.0);
    tester.advance(ms(100));
    assert_eq!(offset(&tester, c), 400.0);
    // Beyond the extents: clamped, ends at the maximum.
    tester.enter(|| c.animate_to(5000.0, ms(100), Curve::Linear));
    tester.advance(ms(100));
    assert_eq!(offset(&tester, c), 800.0);
    // A zero duration jumps.
    tester.enter(|| c.animate_to(100.0, Duration::ZERO, Curve::Linear));
    tester.pump();
    assert_eq!(offset(&tester, c), 100.0);
    // A wheel step during it takes over.
    tester.enter(|| c.animate_to(500.0, ms(200), Curve::Linear));
    tester.advance(ms(50));
    tester.scroll(&Finder::key("s"), Vec2::new(0.0, 40.0));
    tester.advance(ms(150));
    let after = offset(&tester, c);
    assert!((after - 240.0).abs() < 1e-3, "{after}");
}
