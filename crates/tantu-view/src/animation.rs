//! Animation: [`AnimationController`], [`Tween`], [`Curve`], [`Animation`] and [`animated`]
//! props (spec `docs/specs/view/animation.md`). Built on the reactive clock
//! (`tantu_reactive::ticker`): a controller is a value moving between two bounds over a
//! duration, read through tracked signals, so any reactive prop can follow it.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use tantu_core::{Color, EdgeInsets, Point, Rect, Size, Vec2};
use tantu_reactive::{Signal, Ticker, effect, signal, ticker, untrack};

use crate::{IntoProp, Prop};

/// Maps progress `t` in `0..=1` to a value (Flutter's `Curve`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    /// The identity.
    Linear,
    /// Flutter's `Curves.ease`: cubic(0.25, 0.1, 0.25, 1).
    Ease,
    /// cubic(0.42, 0, 1, 1).
    EaseIn,
    /// cubic(0, 0, 0.58, 1).
    EaseOut,
    /// cubic(0.42, 0, 0.58, 1).
    EaseInOut,
    /// Material's standard curve, cubic(0.4, 0, 0.2, 1).
    FastOutSlowIn,
    /// `1 - (1 - t)²`.
    Decelerate,
    /// A cubic bézier with control points (x1, y1), (x2, y2), as CSS `cubic-bezier`.
    Cubic(f32, f32, f32, f32),
}

impl Curve {
    /// The curve's value at `t` (clamped to `0..=1`); exactly 0 at 0 and 1 at 1
    /// (VIEW-ANIM-01).
    pub fn transform(self, t: f64) -> f64 {
        if t.is_nan() || t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        match self {
            Curve::Linear => t,
            Curve::Ease => cubic(0.25, 0.1, 0.25, 1.0, t),
            Curve::EaseIn => cubic(0.42, 0.0, 1.0, 1.0, t),
            Curve::EaseOut => cubic(0.0, 0.0, 0.58, 1.0, t),
            Curve::EaseInOut => cubic(0.42, 0.0, 0.58, 1.0, t),
            Curve::FastOutSlowIn => cubic(0.4, 0.0, 0.2, 1.0, t),
            Curve::Decelerate => 1.0 - (1.0 - t) * (1.0 - t),
            Curve::Cubic(x1, y1, x2, y2) => cubic(
                f64::from(x1),
                f64::from(y1),
                f64::from(x2),
                f64::from(y2),
                t,
            ),
        }
    }
}

/// A cubic bézier through (0, 0), (x1, y1), (x2, y2), (1, 1), evaluated at the parameter
/// where its x equals `t` (found by bisection, as Flutter's `Cubic`).
fn cubic(x1: f64, y1: f64, x2: f64, y2: f64, t: f64) -> f64 {
    let at = |s: f64, a: f64, b: f64| {
        3.0 * (1.0 - s) * (1.0 - s) * s * a + 3.0 * (1.0 - s) * s * s * b + s * s * s
    };
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if at(mid, x1, x2) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    at((lo + hi) / 2.0, y1, y2)
}

/// Linear interpolation between two values (Flutter's `lerp`; VIEW-ANIM-02).
pub trait Lerp: Clone + 'static {
    /// `a` at `t = 0`, `b` at `t = 1`, linear between (and beyond, for overshooting curves).
    fn lerp(a: &Self, b: &Self, t: f64) -> Self;
}

/// `a + (b - a) * t` in `f32`.
fn mix(a: f32, b: f32, t: f64) -> f32 {
    (f64::from(a) + (f64::from(b) - f64::from(a)) * t) as f32
}

impl Lerp for f32 {
    fn lerp(a: &Self, b: &Self, t: f64) -> Self {
        mix(*a, *b, t)
    }
}

impl Lerp for f64 {
    fn lerp(a: &Self, b: &Self, t: f64) -> Self {
        a + (b - a) * t
    }
}

/// `Lerp` for a struct with `f32` fields, componentwise.
macro_rules! lerp_fields {
    ($ty:ty { $($field:ident),* $(,)? }) => {
        impl Lerp for $ty {
            fn lerp(a: &Self, b: &Self, t: f64) -> Self {
                <$ty>::new($(mix(a.$field, b.$field, t)),*)
            }
        }
    };
}

lerp_fields!(Point { x, y });
lerp_fields!(Vec2 { x, y });
lerp_fields!(Size { width, height });

impl Lerp for Color {
    fn lerp(a: &Self, b: &Self, t: f64) -> Self {
        Color {
            r: mix(a.r, b.r, t),
            g: mix(a.g, b.g, t),
            b: mix(a.b, b.b, t),
            a: mix(a.a, b.a, t),
        }
    }
}

impl Lerp for Rect {
    fn lerp(a: &Self, b: &Self, t: f64) -> Self {
        Rect::from_ltrb(
            mix(a.left, b.left, t),
            mix(a.top, b.top, t),
            mix(a.right, b.right, t),
            mix(a.bottom, b.bottom, t),
        )
    }
}

impl Lerp for EdgeInsets {
    fn lerp(a: &Self, b: &Self, t: f64) -> Self {
        EdgeInsets {
            left: mix(a.left, b.left, t),
            top: mix(a.top, b.top, t),
            right: mix(a.right, b.right, t),
            bottom: mix(a.bottom, b.bottom, t),
        }
    }
}

/// An interpolation from `begin` to `end` (Flutter's `Tween`).
#[derive(Clone, Debug, PartialEq)]
pub struct Tween<T> {
    /// The value at progress 0.
    pub begin: T,
    /// The value at progress 1.
    pub end: T,
}

impl<T: Lerp> Tween<T> {
    /// A tween from `begin` to `end`.
    pub fn new(begin: T, end: T) -> Self {
        Tween { begin, end }
    }

    /// `Lerp::lerp(begin, end, t)` (VIEW-ANIM-02).
    pub fn transform(&self, t: f64) -> T {
        T::lerp(&self.begin, &self.end, t)
    }
}

/// Where a controller is (Flutter's `AnimationStatus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationStatus {
    /// Stopped at `lower`.
    Dismissed,
    /// Moving toward `upper`.
    Forward,
    /// Moving toward `lower`.
    Reverse,
    /// Stopped at `upper`.
    Completed,
}

/// A value moving between two bounds over a duration (Flutter's `AnimationController`;
/// VIEW-ANIM-03..06). `Copy`; its signals and ticker belong to the owner current when it was
/// created.
#[derive(Clone, Copy)]
pub struct AnimationController {
    value: Signal<f64>,
    status: Signal<AnimationStatus>,
    /// The run in progress (read untracked by the ticker).
    plan: Signal<Plan>,
    ticker: Ticker,
    lower: f64,
    upper: f64,
    duration: Duration,
}

/// One run of a controller: from a value to another over a duration.
#[derive(Clone, Copy, Debug)]
struct Plan {
    from: f64,
    to: f64,
    duration: Duration,
    /// The status at the end of the run.
    end: AnimationStatus,
    /// `Some(reverse)` to run again at the end.
    repeat: Option<bool>,
    /// Time already elapsed when the run started (a repeat carries the overshoot over).
    skip: Duration,
}

/// `duration × fraction`, with non-finite fractions as 0.
fn scaled(duration: Duration, fraction: f64) -> Duration {
    if fraction.is_finite() && fraction > 0.0 {
        duration.mul_f64(fraction.min(1.0))
    } else {
        Duration::ZERO
    }
}

impl AnimationController {
    /// Bounds 0 to 1, at 0, `Dismissed`, not animating.
    pub fn new(duration: Duration) -> Self {
        AnimationController::with_bounds(duration, 0.0, 1.0)
    }

    /// Bounds `lower..=upper` (the value starts at `lower`).
    pub fn with_bounds(duration: Duration, lower: f64, upper: f64) -> Self {
        let (lower, upper) = if lower.is_finite() && upper.is_finite() && lower <= upper {
            (lower, upper)
        } else {
            (0.0, 1.0)
        };
        let value = signal(lower);
        let status = signal(AnimationStatus::Dismissed);
        let plan = signal(Plan {
            from: lower,
            to: lower,
            duration: Duration::ZERO,
            end: AnimationStatus::Dismissed,
            repeat: None,
            skip: Duration::ZERO,
        });
        // The ticker needs its own handle to stop itself; it is filled in right after.
        let me: Rc<Cell<Option<Ticker>>> = Rc::default();
        let ticker = {
            let me = Rc::clone(&me);
            ticker(move |elapsed| {
                let p = plan.get_untracked();
                let elapsed = elapsed + p.skip;
                let done = elapsed >= p.duration;
                let t = if done || p.duration.is_zero() {
                    1.0
                } else {
                    elapsed.as_secs_f64() / p.duration.as_secs_f64()
                };
                value.set(p.from + (p.to - p.from) * t);
                if !done {
                    return;
                }
                match p.repeat {
                    Some(reverse) => {
                        let overshoot = elapsed - p.duration;
                        let skip = if p.duration.is_zero() {
                            Duration::ZERO
                        } else {
                            Duration::from_nanos(
                                (overshoot.as_nanos() % p.duration.as_nanos()) as u64,
                            )
                        };
                        let (from, to) = if reverse {
                            (p.to, p.from)
                        } else {
                            (p.from, p.to)
                        };
                        let next = Plan {
                            from,
                            to,
                            skip,
                            ..p
                        };
                        plan.set(next);
                        set_status(status, direction(from, to));
                        // The value at the carried-over time.
                        let t = if next.duration.is_zero() {
                            1.0
                        } else {
                            skip.as_secs_f64() / next.duration.as_secs_f64()
                        };
                        value.set(from + (to - from) * t);
                        if let Some(me) = me.get() {
                            me.start();
                        }
                    }
                    None => {
                        set_status(status, p.end);
                        if let Some(me) = me.get() {
                            me.stop();
                        }
                    }
                }
            })
        };
        me.set(Some(ticker));
        AnimationController {
            value,
            status,
            plan,
            ticker,
            lower,
            upper,
            duration,
        }
    }

    /// The current value (a tracked read).
    pub fn value(self) -> f64 {
        self.value.get()
    }

    /// `(value - lower) / (upper - lower)`, in `0..=1` (a tracked read).
    pub fn progress(self) -> f64 {
        let span = self.upper - self.lower;
        if span > 0.0 {
            ((self.value() - self.lower) / span).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The status (a tracked read).
    pub fn status(self) -> AnimationStatus {
        self.status.get()
    }

    /// True while the ticker runs.
    pub fn is_animating(self) -> bool {
        self.ticker.is_active()
    }

    /// The value now, or `None` once disposed (control methods are then ignored). Untracked:
    /// control methods never subscribe their caller (a build inside an effect would otherwise
    /// re-run on every tick).
    fn current(self) -> Option<f64> {
        untrack(|| self.value.try_get())
    }

    /// `value` clamped to the bounds, with non-finite values as `lower`.
    fn clamp(self, value: f64) -> f64 {
        if value.is_finite() {
            value.clamp(self.lower, self.upper)
        } else {
            self.lower
        }
    }

    /// Starts a run from the current value to `to`.
    fn run(self, to: f64, duration: Duration, end: AnimationStatus, repeat: Option<bool>) {
        let Some(from) = self.current() else {
            return;
        };
        let to = self.clamp(to);
        self.plan.set(Plan {
            from,
            to,
            duration,
            end,
            repeat,
            skip: Duration::ZERO,
        });
        set_status(self.status, direction(from, to));
        self.ticker.start();
    }

    /// The controller's duration scaled by the distance from the current value to `to`.
    fn remaining(self, to: f64) -> Duration {
        let Some(from) = self.current() else {
            return Duration::ZERO;
        };
        let span = self.upper - self.lower;
        if span > 0.0 {
            scaled(self.duration, (to - from).abs() / span)
        } else {
            Duration::ZERO
        }
    }

    /// Animates from the current value to `upper` over the remaining fraction of the duration.
    pub fn forward(self) {
        self.run(
            self.upper,
            self.remaining(self.upper),
            AnimationStatus::Completed,
            None,
        );
    }

    /// Animates from the current value to `lower`.
    pub fn reverse(self) {
        self.run(
            self.lower,
            self.remaining(self.lower),
            AnimationStatus::Dismissed,
            None,
        );
    }

    /// Animates to `target` (clamped to the bounds) over `duration`, or over the controller's
    /// duration scaled by the distance when `None`; ends `Completed` (VIEW-ANIM-04).
    pub fn animate_to(self, target: f64, duration: Option<Duration>) {
        let target = self.clamp(target);
        let duration = duration.unwrap_or_else(|| self.remaining(target));
        self.run(target, duration, AnimationStatus::Completed, None);
    }

    /// Like `animate_to`, ending `Dismissed`.
    pub fn animate_back(self, target: f64, duration: Option<Duration>) {
        let target = self.clamp(target);
        let duration = duration.unwrap_or_else(|| self.remaining(target));
        self.run(target, duration, AnimationStatus::Dismissed, None);
    }

    /// Forward, then (with `reverse`) back, forever; without `reverse`, forward from `lower`
    /// again each time.
    pub fn repeat(self, reverse: bool) {
        if self.current().is_none() {
            return;
        }
        self.value.set(self.lower);
        self.plan.set(Plan {
            from: self.lower,
            to: self.upper,
            duration: self.duration,
            end: AnimationStatus::Completed,
            repeat: Some(reverse),
            skip: Duration::ZERO,
        });
        set_status(self.status, AnimationStatus::Forward);
        self.ticker.start();
    }

    /// Stops where it is (the status stays).
    pub fn stop(self) {
        self.ticker.stop();
    }

    /// Stops at `lower`, `Dismissed`.
    pub fn reset(self) {
        if self.current().is_none() {
            return;
        }
        self.ticker.stop();
        self.value.set(self.lower);
        set_status(self.status, AnimationStatus::Dismissed);
    }

    /// Jumps to `value` (clamped): `Dismissed` at `lower`, `Completed` at `upper`, else the
    /// status keeps its direction (`Forward` if it was not animating).
    pub fn set_value(self, value: f64) {
        if self.current().is_none() {
            return;
        }
        let value = self.clamp(value);
        let status = if value <= self.lower {
            AnimationStatus::Dismissed
        } else if value >= self.upper {
            AnimationStatus::Completed
        } else {
            match untrack(|| self.status.try_get()) {
                Some(AnimationStatus::Reverse) => AnimationStatus::Reverse,
                _ => AnimationStatus::Forward,
            }
        };
        self.ticker.stop();
        self.value.set(value);
        set_status(self.status, status);
    }

    /// `Lerp::lerp(begin, end, curve(progress))` as an animation (VIEW-ANIM-05).
    pub fn drive<T: Lerp>(self, tween: Tween<T>) -> Animation<T> {
        Animation {
            controller: self,
            tween,
            curve: Curve::Linear,
        }
    }
}

/// `Forward` when moving up, `Reverse` when moving down (or staying).
fn direction(from: f64, to: f64) -> AnimationStatus {
    if to >= from {
        AnimationStatus::Forward
    } else {
        AnimationStatus::Reverse
    }
}

/// Writes `status` only when it changes, so subscribers see each status once.
fn set_status(signal: Signal<AnimationStatus>, status: AnimationStatus) {
    if untrack(|| signal.try_get()) != Some(status) {
        signal.set(status);
    }
}

/// A tween driven by a controller through a curve (Flutter's `CurvedAnimation` + `Tween`).
#[derive(Clone)]
pub struct Animation<T> {
    controller: AnimationController,
    tween: Tween<T>,
    curve: Curve,
}

impl<T: Lerp> Animation<T> {
    /// The curve applied to the controller's progress (default `Linear`).
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    /// The current value (a tracked read).
    pub fn get(&self) -> T {
        self.tween
            .transform(self.curve.transform(self.controller.progress()))
    }
}

impl<T: Lerp> IntoProp<T> for Animation<T> {
    fn into_prop(self) -> Prop<T> {
        Prop::Dynamic(Box::new(move || self.get()))
    }
}

/// A prop whose changes become transitions (Flutter's implicitly animated widgets, for one
/// prop; VIEW-ANIM-07).
pub struct Animated<T> {
    target: Prop<T>,
    duration: Duration,
    curve: Curve,
}

/// Follows `target`; every change moves there over `duration`.
pub fn animated<T: Lerp + PartialEq>(target: impl IntoProp<T>, duration: Duration) -> Animated<T> {
    Animated {
        target: target.into_prop(),
        duration,
        curve: Curve::EaseInOut,
    }
}

impl<T: Lerp + PartialEq> Animated<T> {
    /// The curve of each transition (default `EaseInOut`).
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
}

impl<T: Lerp + PartialEq> IntoProp<T> for Animated<T> {
    /// VIEW-ANIM-07: a fixed target stays fixed; a dynamic one is watched by an effect that
    /// starts a transition (from the value shown now) on every change.
    fn into_prop(self) -> Prop<T> {
        let target = match self.target {
            Prop::Value(value) => return Prop::Value(value),
            Prop::Dynamic(f) => f,
        };
        // Untracked: building inside a Dyn must not subscribe it to the target (VIEW-ANIM-08).
        let initial = untrack(&target);
        let from = signal(initial.clone());
        let to = signal(initial.clone());
        let last: Rc<RefCell<T>> = Rc::new(RefCell::new(initial));
        let controller = AnimationController::new(self.duration);
        let curve = self.curve;
        let shown = move || {
            let t = curve.transform(controller.progress());
            from.with(|a| to.with(|b| T::lerp(a, b, t)))
        };
        effect(move || {
            let next = target();
            if *last.borrow() == next {
                return;
            }
            *last.borrow_mut() = next.clone();
            let now = untrack(shown);
            from.set(now);
            to.set(next);
            controller.set_value(0.0);
            controller.forward();
        });
        Prop::Dynamic(Box::new(shown))
    }
}
