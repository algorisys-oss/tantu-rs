# Animation: controller, tweens, curves and animated props

- **Status:** Implemented (the user said "ok" on the draft, 2026-10-11, taking its six
  proposals; VIEW-ANIM-08 added after the gallery hung)
- **Crates:** `tantu-view` (`animation` module), `examples/gallery` (an Animation section)
- **Plan item:** Phase 3, "Animation", sub-item "`AnimationController`, `Tween`, curves, and
  `animated()` props"
- **Related:** [timers and tickers](../reactive/timers.md) (the clock),
  [frame](frame.md) (`Prop`, `IntoProp`, bindings), [signals](../reactive/signals.md),
  [gallery](../examples/gallery.md), Flutter's `AnimationController`, `Tween`, `Curves`,
  `CurvedAnimation` and `ImplicitlyAnimatedWidget`

## Purpose

With tickers in place, this adds Flutter's animation vocabulary on top of signals. An
`AnimationController` is a value that moves from one bound to the other over a duration,
driven by a ticker and exposed as tracked reads, so any reactive prop can follow it.
`Tween` and `Curve` shape the value, and `Animation<T>` combines the three into something a
widget setter accepts directly. `animated()` is the implicit form: wrap any reactive prop and
every change of its target becomes a transition, which is Tantu's fine-grained
`AnimatedContainer`: no animated widget variants, since every prop is already reactive.

## Scope

In scope:

- `Curve` (Flutter's named curves and cubic béziers), `Lerp` for the core value types,
  `Tween<T>`, `AnimationController`, `AnimationStatus`, `Animation<T>`, `Animated<T>` and
  `animated()`.
- A gallery section showing both forms.

Out of scope (next sub-item and later): enter/exit transitions (`Opacity`, `AnimatedSwitcher`,
animated `Show`), spring and physics simulations, staggered intervals (`Interval` curve can
come when needed), animating text styles.

## Public API

```rust
// tantu-view::animation (re-exported at the crate root)
use std::time::Duration;

/// Maps progress `t` in `0..=1` to a value (Flutter's `Curve`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
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
    /// The curve's value at `t` (clamped to `0..=1`); exactly 0 at 0 and 1 at 1.
    pub fn transform(self, t: f64) -> f64;
}

/// Linear interpolation between two values (Flutter's `lerp`).
pub trait Lerp: Clone + 'static {
    /// `a` at `t = 0`, `b` at `t = 1`, linear between (and beyond, for overshooting curves).
    fn lerp(a: &Self, b: &Self, t: f64) -> Self;
}
// Implemented for f32, f64, Color, Point, Vec2, Size, Rect, EdgeInsets.

/// An interpolation from `begin` to `end` (Flutter's `Tween`).
#[derive(Clone, Debug, PartialEq)]
pub struct Tween<T> { pub begin: T, pub end: T }

impl<T: Lerp> Tween<T> {
    pub fn new(begin: T, end: T) -> Self;
    /// `Lerp::lerp(begin, end, t)`.
    pub fn transform(&self, t: f64) -> T;
}

/// Where a controller is (Flutter's `AnimationStatus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationStatus { Dismissed, Forward, Reverse, Completed }

/// A value moving between two bounds over a duration (Flutter's `AnimationController`).
/// `Copy`; its signals and ticker belong to the owner current when it was created.
#[derive(Clone, Copy)]
pub struct AnimationController { /* signals, ticker, bounds, duration */ }

impl AnimationController {
    /// Bounds 0 to 1, at 0, `Dismissed`, not animating.
    pub fn new(duration: Duration) -> Self;
    /// Bounds `lower..=upper` (the value starts at `lower`).
    pub fn with_bounds(duration: Duration, lower: f64, upper: f64) -> Self;
    /// The current value (a tracked read).
    pub fn value(self) -> f64;
    /// `(value - lower) / (upper - lower)`, in `0..=1` (a tracked read).
    pub fn progress(self) -> f64;
    /// The status (a tracked read).
    pub fn status(self) -> AnimationStatus;
    /// True while the ticker runs.
    pub fn is_animating(self) -> bool;
    /// Animates from the current value to `upper` over the remaining fraction of `duration`.
    pub fn forward(self);
    /// Animates from the current value to `lower`.
    pub fn reverse(self);
    /// Animates to `target` (clamped to the bounds) over `duration`, or over the controller's
    /// duration scaled by the distance when `None`; ends `Completed`.
    pub fn animate_to(self, target: f64, duration: Option<Duration>);
    /// Like `animate_to`, ending `Dismissed`.
    pub fn animate_back(self, target: f64, duration: Option<Duration>);
    /// Forward, then (with `reverse`) back, forever; without `reverse`, forward from `lower`
    /// again each time.
    pub fn repeat(self, reverse: bool);
    /// Stops where it is (the status stays).
    pub fn stop(self);
    /// Stops at `lower`, `Dismissed`.
    pub fn reset(self);
    /// Jumps to `value` (clamped): `Dismissed` at `lower`, `Completed` at `upper`, else the
    /// status keeps its direction (`Forward` if it was not animating).
    pub fn set_value(self, value: f64);
    /// `Lerp::lerp(begin, end, curve(progress))` as an animation.
    pub fn drive<T: Lerp>(self, tween: Tween<T>) -> Animation<T>;
}

/// A tween driven by a controller through a curve (Flutter's `CurvedAnimation` + `Tween`).
#[derive(Clone)]
pub struct Animation<T> { /* controller, tween, curve */ }

impl<T: Lerp> Animation<T> {
    /// The curve applied to the controller's progress (default `Linear`).
    pub fn curve(self, curve: Curve) -> Self;
    /// The current value (a tracked read).
    pub fn get(&self) -> T;
}
impl<T: Lerp> IntoProp<T> for Animation<T> { /* Prop::Dynamic(move || animation.get()) */ }

/// A prop whose changes become transitions (Flutter's implicitly animated widgets, for one
/// prop).
pub struct Animated<T> { /* target prop, duration, curve */ }

/// Follows `target`; every change moves there over `duration`.
pub fn animated<T: Lerp + PartialEq>(target: impl IntoProp<T>, duration: Duration) -> Animated<T>;

impl<T: Lerp + PartialEq> Animated<T> {
    /// The curve of each transition (default `EaseInOut`).
    pub fn curve(self, curve: Curve) -> Self;
}
impl<T: Lerp + PartialEq> IntoProp<T> for Animated<T> { /* see VIEW-ANIM-07 */ }
```

## Behavior

- **VIEW-ANIM-01:** `Curve::transform` is 0 at 0 and 1 at 1 for every curve, clamps `t` to
  `0..=1`, `Linear` is the identity, `Decelerate` is `1 − (1 − t)²`, and the cubic béziers
  match CSS `cubic-bezier` within 0.005: `EaseIn(0.5) ≈ 0.315`, `EaseOut(0.5) ≈ 0.685`,
  `EaseInOut(0.5) = 0.5`, `FastOutSlowIn(0.5) ≈ 0.776`, `Ease(0.5) ≈ 0.802`.
- **VIEW-ANIM-02:** `Lerp` for `f32`, `f64`, `Color`, `Point`, `Vec2`, `Size`, `Rect` and
  `EdgeInsets` is componentwise linear: `a` at 0, `b` at 1, the midpoint at 0.5, and extrapolated
  outside (so an overshooting curve can go past `end`). `Tween::transform` is that lerp.
- **VIEW-ANIM-03:** `forward` on a new controller with duration *d* makes `status` `Forward`
  and `value` move linearly from `lower` toward `upper` with each tick: at *d*/2 it is the
  midpoint, at or after *d* it is exactly `upper`, `status` is `Completed` and the ticker is
  stopped (`is_animating` false, `next_wake` no longer `Now`). `reverse` does the same toward
  `lower`, ending `Dismissed`. Starting from a value in between takes the remaining fraction
  of *d* (Flutter's behavior), so `forward` from the midpoint completes in *d*/2.
- **VIEW-ANIM-04:** `animate_to(target, None)` moves to `target` over *d* scaled by the distance
  as a fraction of the bounds and ends `Completed`; with `Some(duration)` it takes that
  duration; `animate_back` ends `Dismissed`. `repeat(false)` restarts from `lower` each time
  it reaches `upper`; `repeat(true)` alternates direction, never stopping. `stop` freezes the
  value with the status unchanged; `reset` sets `lower` and `Dismissed`; `set_value` jumps,
  with the status as its doc says. Targets and values are clamped to the bounds; a NaN or
  non-finite value counts as `lower`.
- **VIEW-ANIM-05:** `value`, `progress` and `status` are tracked reads: a prop closure reading
  them is re-applied at the next frame after every tick that changed them, so a bound element
  updates each frame while the controller animates, and a `WidgetTester::advance` of *d*/2
  shows the midpoint. `drive(tween).curve(curve).get()` is
  `tween.transform(curve.transform(progress))`, and an `Animation<T>` is an `IntoProp<T>`.
- **VIEW-ANIM-06:** A controller's signals and ticker belong to the owner current at creation:
  disposing it stops the animation and drops it (no leaks). Control methods on a disposed
  controller are ignored; `value`, `progress` and `status` read like signals (ADR 0007).
- **VIEW-ANIM-07:** `animated(target, d)` as a prop starts at the target's first value with no
  transition; when the target changes, the prop moves from its *current* value to the new
  target over *d* along the curve (default `EaseInOut`), retargeting mid-flight from wherever it
  is; a change to the same value does nothing; a zero duration jumps; a `Prop::Value` target
  never animates. The transition's ticker belongs to the owner current when `into_prop` ran
  (the widget's build), so it stops with the widget.
- **VIEW-ANIM-08:** Control methods (`forward`, `reverse`, `animate_to`, `animate_back`,
  `repeat`, `stop`, `reset`, `set_value`) never subscribe their caller: a controller created
  and started inside a `Dyn`'s closure (or any effect) doesn't make that closure re-run when
  the controller's value changes, so the content is built once and animates in place. Found
  when the gallery hung: `repeat` read the value signal tracked, the `Dyn` subscribed to it
  and rebuilt a new controller on every tick. Likewise `animated(target, d).into_prop()` reads
  the target's first value without subscribing its caller, so a `Dyn` building a widget with
  an animated prop doesn't rebuild when the target changes; the prop transitions in place.
- **GALLERY-06:** An "Animation" section (key `animation`): a `Switch` (key `grow`) that
  animates a box (key `box`) between 100 and 200 px wide and two colors with `animated()` over
  300 ms, and a dot (key `dot`) moving back and forth across a track with a repeating
  controller. `advance(150 ms)` after tapping the switch shows the box at 150 px. Golden
  `tests/goldens/gallery-animation.png` at that moment.

## Performance and allocation

A running controller writes two signals per tick (value, and status only when it changes);
each bound prop re-applies once per frame. `Animated` holds one controller per prop. Nothing
allocates per tick after creation.

## Open questions

Resolved (2026-10-11; the user said "ok" on the draft, taking these proposals):

1. **The module lives in `tantu-view`** (not `tantu-reactive`): it needs `IntoProp` and the
   core value types; `tantu-reactive` keeps only the clock.
2. **`AnimationController` is a `Copy` handle over signals**, like `ScrollController`, so
   closures capture it freely and it is disposed with its owner; Flutter's explicit
   `dispose()` isn't needed.
3. **Curves as an enum with a `Cubic` variant** rather than a trait: covers Flutter's named
   curves and any CSS bézier, stays `Copy`, and leaves a trait for later if someone needs a
   custom shape. Béziers are solved by bisection on *x* as in Flutter (error < 1e-3).
4. **`animated()` as an `IntoProp` adapter** instead of `AnimatedContainer`-style widgets:
   every prop is reactive already, so wrapping the prop is the natural unit; it also works
   for custom widgets with no extra API.
5. **Default implicit curve `EaseInOut`**, Flutter's default for implicit animations is
   `linear`, but ease-in-out is what Material components use; apps pass `.curve(Linear)` for
   Flutter's default.
6. **The gallery gets a section** rather than a separate example, keeping "every feature in
   one window".
