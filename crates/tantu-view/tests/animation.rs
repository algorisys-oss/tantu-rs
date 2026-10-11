//! Tests for `docs/specs/view/animation.md`, rules VIEW-ANIM-01..08.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::time::{Duration, Instant};

use tantu_core::{Color, EdgeInsets, Point, Rect, Size, Vec2};
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox};
use tantu_reactive::{Scope, Signal, Wake, effect, signal};
use tantu_scene::Scene;
use tantu_view::{
    Animated, AnimationController, AnimationStatus, AnyView, BuildCx, Curve, Dyn, ElementId,
    IntoProp, Lerp, Prop, Tween, View, ViewTree, animated,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// A box whose width follows a prop.
struct Leaf {
    width: Prop<f32>,
    out: Rc<Cell<Option<ElementId>>>,
}

impl View for Leaf {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(0.0), Some(10.0)), []);
        cx.bind(id, self.width, |el, width| {
            el.update_render(|r: &mut RenderConstrainedBox| {
                r.additional.min_width = width;
                r.additional.max_width = width;
            });
        });
        self.out.set(Some(id));
        id
    }
}

/// An empty view, for trees that only host a runtime.
struct Empty;

impl View for Empty {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderConstrainedBox::sized(Some(0.0), Some(0.0)), [])
    }
}

fn frame(tree: &mut ViewTree) {
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(800.0, 600.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
}

fn width_of(tree: &ViewTree, element: ElementId) -> f32 {
    let layout = tree.layout_id(element).expect("a render element");
    tree.layout_tree().size(layout).expect("laid out").width
}

#[test]
fn view_anim_01_curves() {
    let all = [
        Curve::Linear,
        Curve::Ease,
        Curve::EaseIn,
        Curve::EaseOut,
        Curve::EaseInOut,
        Curve::FastOutSlowIn,
        Curve::Decelerate,
        Curve::Cubic(0.1, 0.9, 0.9, 0.1),
    ];
    for curve in all {
        assert_eq!(curve.transform(0.0), 0.0, "{curve:?}");
        assert_eq!(curve.transform(1.0), 1.0, "{curve:?}");
        assert_eq!(curve.transform(-1.0), 0.0, "{curve:?}");
        assert_eq!(curve.transform(2.0), 1.0, "{curve:?}");
        assert_eq!(curve.transform(f64::NAN), 0.0, "{curve:?}");
    }
    assert!(close(Curve::Linear.transform(0.3), 0.3));
    assert!(close(Curve::Decelerate.transform(0.5), 0.75));
    for (curve, at, expected) in [
        (Curve::EaseIn, 0.5, 0.315),
        (Curve::EaseOut, 0.5, 0.685),
        (Curve::EaseInOut, 0.5, 0.5),
        (Curve::EaseInOut, 0.25, 0.129),
        (Curve::FastOutSlowIn, 0.5, 0.776),
        (Curve::Ease, 0.5, 0.802),
        (Curve::Cubic(0.42, 0.0, 1.0, 1.0), 0.5, 0.315),
    ] {
        let got = curve.transform(at);
        assert!((got - expected).abs() < 0.005, "{curve:?}({at}) = {got}");
    }
}

#[test]
fn view_anim_02_lerp_and_tween() {
    assert_eq!(f32::lerp(&10.0, &20.0, 0.5), 15.0);
    assert_eq!(f64::lerp(&10.0, &20.0, 0.0), 10.0);
    assert_eq!(f64::lerp(&10.0, &20.0, 1.0), 20.0);
    // Extrapolation for overshooting curves.
    assert_eq!(f64::lerp(&10.0, &20.0, 1.5), 25.0);
    assert_eq!(
        <Color as Lerp>::lerp(&Color::BLACK, &Color::WHITE, 0.5),
        Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0
        }
    );
    assert_eq!(
        Point::lerp(&Point::new(0.0, 0.0), &Point::new(10.0, 20.0), 0.5),
        Point::new(5.0, 10.0)
    );
    assert_eq!(
        Vec2::lerp(&Vec2::new(0.0, 0.0), &Vec2::new(10.0, 20.0), 0.25),
        Vec2::new(2.5, 5.0)
    );
    assert_eq!(
        Size::lerp(&Size::new(0.0, 0.0), &Size::new(10.0, 20.0), 0.5),
        Size::new(5.0, 10.0)
    );
    assert_eq!(
        Rect::lerp(
            &Rect::from_ltrb(0.0, 0.0, 10.0, 10.0),
            &Rect::from_ltrb(10.0, 10.0, 30.0, 30.0),
            0.5
        ),
        Rect::from_ltrb(5.0, 5.0, 20.0, 20.0)
    );
    assert_eq!(
        EdgeInsets::lerp(&EdgeInsets::all(0.0), &EdgeInsets::all(8.0), 0.5),
        EdgeInsets::all(4.0)
    );
    let tween = Tween::new(100.0f32, 200.0);
    assert_eq!(tween.transform(0.0), 100.0);
    assert_eq!(tween.transform(0.5), 150.0);
    assert_eq!(tween.transform(1.0), 200.0);
}

/// A tree hosting a controller created at build.
fn host(make: impl FnOnce() -> AnimationController + 'static) -> (ViewTree, AnimationController) {
    let out: Rc<Cell<Option<AnimationController>>> = Rc::default();
    let tree = {
        let out = out.clone();
        ViewTree::new(move || {
            out.set(Some(make()));
            Empty
        })
    };
    let controller = out.get().expect("built");
    (tree, controller)
}

#[test]
fn view_anim_03_forward_and_reverse() {
    let (mut tree, c) = host(|| AnimationController::new(ms(100)));
    let t0 = Instant::now();
    tree.enter(|| {
        assert_eq!(c.value(), 0.0);
        assert_eq!(c.status(), AnimationStatus::Dismissed);
        assert!(!c.is_animating());
        c.forward();
        assert_eq!(c.status(), AnimationStatus::Forward);
        assert!(c.is_animating());
    });
    // Started before the first tick: counts from it.
    tree.tick(t0);
    tree.enter(|| assert_eq!(c.value(), 0.0));
    tree.tick(t0 + ms(50));
    tree.enter(|| {
        assert!(close(c.value(), 0.5), "{}", c.value());
        assert!(close(c.progress(), 0.5));
        assert_eq!(c.status(), AnimationStatus::Forward);
    });
    tree.tick(t0 + ms(130));
    tree.enter(|| {
        assert_eq!(c.value(), 1.0);
        assert_eq!(c.status(), AnimationStatus::Completed);
        assert!(!c.is_animating());
    });
    assert_eq!(tree.next_wake(), None);
    tree.enter(|| c.reverse());
    tree.tick(t0 + ms(180));
    tree.enter(|| {
        assert!(close(c.value(), 0.5));
        assert_eq!(c.status(), AnimationStatus::Reverse);
    });
    tree.tick(t0 + ms(230));
    tree.enter(|| {
        assert_eq!(c.value(), 0.0);
        assert_eq!(c.status(), AnimationStatus::Dismissed);
    });
    // From the midpoint, forward takes the remaining half.
    tree.enter(|| {
        c.set_value(0.5);
        c.forward();
    });
    // The run started at the clock's last tick (230 ms): 25 ms in, it is at 0.75.
    tree.tick(t0 + ms(255));
    tree.enter(|| assert!(close(c.value(), 0.75), "{}", c.value()));
    tree.tick(t0 + ms(290));
    tree.enter(|| assert_eq!(c.status(), AnimationStatus::Completed));
    // Other bounds.
    let (mut tree, c) = host(|| AnimationController::with_bounds(ms(100), 10.0, 20.0));
    tree.tick(t0);
    tree.enter(|| {
        assert_eq!(c.value(), 10.0);
        c.forward();
    });
    tree.tick(t0 + ms(25));
    tree.enter(|| {
        assert!(close(c.value(), 12.5));
        assert!(close(c.progress(), 0.25));
    });
}

#[test]
fn view_anim_04_animate_to_repeat_stop_reset_set_value() {
    let (mut tree, c) = host(|| AnimationController::new(ms(100)));
    let t0 = Instant::now();
    tree.tick(t0);
    // animate_to over the duration scaled by the distance.
    tree.enter(|| c.animate_to(0.5, None));
    tree.tick(t0 + ms(25));
    tree.enter(|| assert!(close(c.value(), 0.25)));
    tree.tick(t0 + ms(50));
    tree.enter(|| {
        assert!(close(c.value(), 0.5));
        assert_eq!(c.status(), AnimationStatus::Completed);
        assert!(!c.is_animating());
        c.animate_to(1.0, Some(ms(20)));
    });
    tree.tick(t0 + ms(60));
    tree.enter(|| assert!(close(c.value(), 0.75)));
    tree.tick(t0 + ms(70));
    tree.enter(|| {
        assert_eq!(c.value(), 1.0);
        c.animate_back(0.0, None);
    });
    tree.tick(t0 + ms(170));
    tree.enter(|| {
        assert_eq!(c.value(), 0.0);
        assert_eq!(c.status(), AnimationStatus::Dismissed);
        c.repeat(false);
    });
    tree.tick(t0 + ms(170));
    tree.tick(t0 + ms(320));
    tree.enter(|| {
        assert!(close(c.value(), 0.5), "wrapped: {}", c.value());
        assert_eq!(c.status(), AnimationStatus::Forward);
        assert!(c.is_animating());
        c.reset();
        assert_eq!(c.value(), 0.0);
        assert_eq!(c.status(), AnimationStatus::Dismissed);
        assert!(!c.is_animating());
        c.repeat(true);
    });
    tree.tick(t0 + ms(320));
    tree.tick(t0 + ms(470));
    tree.enter(|| {
        assert!(close(c.value(), 0.5), "bouncing back: {}", c.value());
        assert_eq!(c.status(), AnimationStatus::Reverse);
        c.stop();
        assert!(!c.is_animating());
        assert_eq!(c.status(), AnimationStatus::Reverse);
    });
    tree.tick(t0 + ms(600));
    tree.enter(|| {
        assert!(close(c.value(), 0.5), "frozen");
        c.set_value(1.0);
        assert_eq!(c.status(), AnimationStatus::Completed);
        c.set_value(0.0);
        assert_eq!(c.status(), AnimationStatus::Dismissed);
        c.set_value(0.3);
        assert_eq!(c.status(), AnimationStatus::Forward);
        c.set_value(f64::NAN);
        assert_eq!(c.value(), 0.0);
        c.set_value(7.0);
        assert_eq!(c.value(), 1.0);
        c.animate_to(5.0, Some(ms(10)));
    });
    tree.tick(t0 + ms(700));
    tree.enter(|| assert_eq!(c.value(), 1.0));
}

#[test]
fn view_anim_05_tracked_reads_drive_props() {
    let out: Rc<Cell<Option<ElementId>>> = Rc::default();
    let ctrl: Rc<Cell<Option<AnimationController>>> = Rc::default();
    let statuses: Rc<RefCell<Vec<AnimationStatus>>> = Rc::default();
    let mut tree = {
        let (out, ctrl, statuses) = (out.clone(), ctrl.clone(), statuses.clone());
        ViewTree::new(move || {
            let c = AnimationController::new(ms(100));
            ctrl.set(Some(c));
            effect(move || statuses.borrow_mut().push(c.status()));
            Leaf {
                width: c
                    .drive(Tween::new(100.0f32, 200.0))
                    .curve(Curve::EaseInOut)
                    .into_prop(),
                out,
            }
        })
    };
    let (leaf, c) = (out.get().expect("built"), ctrl.get().expect("built"));
    let t0 = Instant::now();
    tree.tick(t0);
    frame(&mut tree);
    assert_eq!(width_of(&tree, leaf), 100.0);
    assert!(!tree.needs_frame());
    tree.enter(|| c.forward());
    assert_eq!(tree.next_wake(), Some(Wake::Now));
    tree.tick(t0 + ms(50));
    assert!(tree.needs_frame());
    frame(&mut tree);
    assert_eq!(width_of(&tree, leaf), 150.0);
    tree.tick(t0 + ms(25 + 50));
    frame(&mut tree);
    // EaseInOut(0.75) = 1 − EaseInOut(0.25) ≈ 0.871.
    let w = width_of(&tree, leaf);
    assert!((w - 187.1).abs() < 0.5, "{w}");
    tree.tick(t0 + ms(150));
    frame(&mut tree);
    assert_eq!(width_of(&tree, leaf), 200.0);
    assert_eq!(
        *statuses.borrow(),
        [
            AnimationStatus::Dismissed,
            AnimationStatus::Forward,
            AnimationStatus::Completed
        ]
    );
    // A linear drive through `get`.
    tree.enter(|| {
        let anim = c.drive(Tween::new(Color::BLACK, Color::WHITE));
        assert_eq!(anim.get(), Color::WHITE);
        c.set_value(0.5);
        assert_eq!(anim.get().r, 0.5);
    });
}

#[test]
fn view_anim_06_ownership() {
    let (mut tree, _) = host(|| AnimationController::new(ms(100)));
    let t0 = Instant::now();
    tree.tick(t0);
    let c = tree.enter(|| {
        let scope = Scope::new();
        let c = scope.run(|| {
            let c = AnimationController::new(ms(100));
            c.forward();
            c
        });
        assert_eq!(tree.next_wake(), Some(Wake::Now));
        scope.dispose();
        c
    });
    assert_eq!(tree.next_wake(), None);
    tree.enter(|| {
        c.forward();
        c.stop();
        c.reset();
        c.set_value(0.5);
        assert!(!c.is_animating());
        assert!(catch_unwind(AssertUnwindSafe(|| c.value())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| c.status())).is_err());
    });
    tree.tick(t0 + ms(50));
}

#[test]
fn view_anim_07_animated_props() {
    let out: Rc<Cell<Option<ElementId>>> = Rc::default();
    let target_out: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let mut tree = {
        let (out, target_out) = (out.clone(), target_out.clone());
        ViewTree::new(move || {
            let target = signal(100.0f32);
            target_out.set(Some(target));
            Leaf {
                width: animated(target, ms(100)).curve(Curve::Linear).into_prop(),
                out,
            }
        })
    };
    let (leaf, target) = (out.get().expect("built"), target_out.get().expect("built"));
    let t0 = Instant::now();
    tree.tick(t0);
    frame(&mut tree);
    assert_eq!(
        width_of(&tree, leaf),
        100.0,
        "the first value, no transition"
    );
    assert_eq!(tree.next_wake(), None);
    tree.enter(|| target.set(200.0));
    assert_eq!(tree.next_wake(), Some(Wake::Now));
    tree.tick(t0 + ms(50));
    frame(&mut tree);
    assert_eq!(width_of(&tree, leaf), 150.0);
    // Retargeting mid-flight starts from the current value.
    tree.enter(|| target.set(100.0));
    tree.tick(t0 + ms(100));
    frame(&mut tree);
    assert_eq!(width_of(&tree, leaf), 125.0);
    tree.tick(t0 + ms(150));
    frame(&mut tree);
    assert_eq!(width_of(&tree, leaf), 100.0);
    assert_eq!(tree.next_wake(), None);
    // The same value again: nothing starts.
    tree.enter(|| target.set(100.0));
    assert_eq!(tree.next_wake(), None);
    // A fixed target never animates; a zero duration jumps.
    let fixed: Prop<f32> = tree.enter(|| animated(42.0f32, ms(100)).into_prop());
    assert!(!fixed.is_dynamic());
    assert_eq!(fixed.get(), 42.0);
    let jump: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let out2: Rc<Cell<Option<ElementId>>> = Rc::default();
    let mut tree2 = {
        let (jump, out2) = (jump.clone(), out2.clone());
        ViewTree::new(move || {
            let target = signal(1.0f32);
            jump.set(Some(target));
            Leaf {
                width: animated(target, Duration::ZERO).into_prop(),
                out: out2,
            }
        })
    };
    let (leaf2, target2) = (out2.get().expect("built"), jump.get().expect("built"));
    tree2.tick(t0);
    tree2.enter(|| target2.set(9.0));
    tree2.tick(t0);
    frame(&mut tree2);
    assert_eq!(width_of(&tree2, leaf2), 9.0);
    let _: Option<Animated<f32>> = None;
}

#[test]
fn view_anim_08_control_methods_do_not_subscribe_the_caller() {
    let builds = Rc::new(Cell::new(0u32));
    let out: Rc<Cell<Option<ElementId>>> = Rc::default();
    let mut tree = {
        let (builds, out) = (builds.clone(), out.clone());
        ViewTree::new(move || {
            let (builds, out) = (builds.clone(), out.clone());
            Dyn::new(move || {
                builds.set(builds.get() + 1);
                let c = AnimationController::new(ms(100));
                c.repeat(true);
                c.set_value(0.0);
                c.forward();
                AnyView::new(Leaf {
                    width: c.drive(Tween::new(0.0f32, 100.0)).into_prop(),
                    out: out.clone(),
                })
            })
        })
    };
    let leaf = out.get().expect("built");
    let t0 = Instant::now();
    tree.tick(t0);
    frame(&mut tree);
    for i in 1..=5u64 {
        tree.tick(t0 + ms(10 * i));
        frame(&mut tree);
    }
    assert_eq!(builds.get(), 1, "the Dyn built once");
    assert_eq!(out.get(), Some(leaf), "and kept its element");
    assert!((width_of(&tree, leaf) - 50.0).abs() < 1e-3);

    // An animated prop inside a Dyn: the Dyn doesn't follow the target either.
    let builds = Rc::new(Cell::new(0u32));
    let out: Rc<Cell<Option<ElementId>>> = Rc::default();
    let target_out: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let mut tree = {
        let (builds, out, target_out) = (builds.clone(), out.clone(), target_out.clone());
        ViewTree::new(move || {
            let target = signal(100.0f32);
            target_out.set(Some(target));
            let (builds, out) = (builds.clone(), out.clone());
            Dyn::new(move || {
                builds.set(builds.get() + 1);
                AnyView::new(Leaf {
                    width: animated(target, ms(100)).curve(Curve::Linear).into_prop(),
                    out: out.clone(),
                })
            })
        })
    };
    let (leaf, target) = (out.get().expect("built"), target_out.get().expect("built"));
    tree.tick(t0);
    frame(&mut tree);
    tree.enter(|| target.set(200.0));
    tree.tick(t0 + ms(50));
    frame(&mut tree);
    assert_eq!(builds.get(), 1, "the Dyn built once");
    assert_eq!(out.get(), Some(leaf));
    assert_eq!(width_of(&tree, leaf), 150.0);
}
