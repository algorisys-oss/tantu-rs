//! Animation: [`AnimationController`], [`Tween`], [`Curve`], [`Animation`] and [`animated`]
//! props (spec `docs/specs/view/animation.md`). Built on the reactive clock
//! (`tantu_reactive::ticker`): a controller is a value moving between two bounds over a
//! duration, read through tracked signals, so any reactive prop can follow it.

use std::time::Duration;

use tantu_core::{Color, EdgeInsets, Point, Rect, Size, Vec2};

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
        let _ = t;
        todo!()
    }
}

/// Linear interpolation between two values (Flutter's `lerp`; VIEW-ANIM-02).
pub trait Lerp: Clone + 'static {
    /// `a` at `t = 0`, `b` at `t = 1`, linear between (and beyond, for overshooting curves).
    fn lerp(a: &Self, b: &Self, t: f64) -> Self;
}

macro_rules! lerp_todo {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Lerp for $ty {
                fn lerp(a: &Self, b: &Self, t: f64) -> Self {
                    let _ = (a, b, t);
                    todo!()
                }
            }
        )*
    };
}

lerp_todo!(f32, f64, Color, Point, Vec2, Size, Rect, EdgeInsets);

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
    #[allow(dead_code)]
    inner: (),
}

impl AnimationController {
    /// Bounds 0 to 1, at 0, `Dismissed`, not animating.
    pub fn new(duration: Duration) -> Self {
        AnimationController::with_bounds(duration, 0.0, 1.0)
    }

    /// Bounds `lower..=upper` (the value starts at `lower`).
    pub fn with_bounds(duration: Duration, lower: f64, upper: f64) -> Self {
        let _ = (duration, lower, upper);
        todo!()
    }

    /// The current value (a tracked read).
    pub fn value(self) -> f64 {
        todo!()
    }

    /// `(value - lower) / (upper - lower)`, in `0..=1` (a tracked read).
    pub fn progress(self) -> f64 {
        todo!()
    }

    /// The status (a tracked read).
    pub fn status(self) -> AnimationStatus {
        todo!()
    }

    /// True while the ticker runs.
    pub fn is_animating(self) -> bool {
        todo!()
    }

    /// Animates from the current value to `upper` over the remaining fraction of the duration.
    pub fn forward(self) {
        todo!()
    }

    /// Animates from the current value to `lower`.
    pub fn reverse(self) {
        todo!()
    }

    /// Animates to `target` (clamped to the bounds) over `duration`, or over the controller's
    /// duration scaled by the distance when `None`; ends `Completed` (VIEW-ANIM-04).
    pub fn animate_to(self, target: f64, duration: Option<Duration>) {
        let _ = (target, duration);
        todo!()
    }

    /// Like `animate_to`, ending `Dismissed`.
    pub fn animate_back(self, target: f64, duration: Option<Duration>) {
        let _ = (target, duration);
        todo!()
    }

    /// Forward, then (with `reverse`) back, forever; without `reverse`, forward from `lower`
    /// again each time.
    pub fn repeat(self, reverse: bool) {
        let _ = reverse;
        todo!()
    }

    /// Stops where it is (the status stays).
    pub fn stop(self) {
        todo!()
    }

    /// Stops at `lower`, `Dismissed`.
    pub fn reset(self) {
        todo!()
    }

    /// Jumps to `value` (clamped): `Dismissed` at `lower`, `Completed` at `upper`, else the
    /// status keeps its direction (`Forward` if it was not animating).
    pub fn set_value(self, value: f64) {
        let _ = value;
        todo!()
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
    fn into_prop(self) -> Prop<T> {
        let _ = (self.target, self.duration, self.curve);
        todo!()
    }
}
