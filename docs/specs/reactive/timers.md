# Timers and tickers

- **Status:** Implemented (the user said "continue" on the draft, 2026-10-11, taking its seven
  proposals; PLATFORM-WINIT-14 checked in a real window on Wayland)
- **Crates:** `tantu-reactive` (the clock: timers, tickers, `Runtime::tick`), `tantu-platform`
  and `tantu-platform-winit` (`now`, `wake_at`), `tantu-view` (forwarding on `ViewTree`),
  `tantu` (the runner ticking and scheduling wake-ups), `tantu-test` (`advance`),
  `tantu-widgets` (the tooltip's show delay, the first user)
- **Plan item:** Phase 3, "Animation", sub-item "timers and tickers"
- **Related:** [signals](signals.md) (owners, `on_cleanup`), [platform](../platform/platform.md),
  [frame](../view/frame.md) (needs-frame), [facade](../facade/app.md),
  [WidgetTester](../test/widget-tester.md), [tooltip](../widgets/tooltip.md), Flutter's
  `Ticker`, `SchedulerBinding` and `Timer`

## Purpose

Nothing in Tantu can wait yet: the tooltip shows at once instead of after 500 ms, the caret
can't blink, scrolling can't coast, and no widget can animate. This spec adds time to the
reactive runtime: one-shot **timers** and per-frame **tickers**, driven by `Runtime::tick(now)`.
The app runner calls `tick` at the start of every frame with the platform's clock, and asks
the platform to wake it up when the next timer is due (winit's `WaitUntil`), so a waiting app
costs nothing between deadlines. Tests drive the same clock by hand (`WidgetTester::advance`),
so every animation is deterministic under test. Animation controllers, tweens and curves are
built on tickers in the next sub-item.

Timers and tickers live in `tantu-reactive` because they belong to the same owner tree as
signals: a widget creates them where it creates signals, from builds and handlers alike, and
they stop when the widget's scope is disposed.

## Scope

In scope:

- `tantu-reactive`: `timer`, `Timer`, `ticker`, `Ticker`, `Wake`, `Runtime::tick`,
  `Runtime::now`, `Runtime::next_wake`.
- `tantu-platform`: `PlatformContext::now` and `wake_at`; `FakeStep::Advance` and the fake
  clock; the winit implementation.
- `tantu-view`: `ViewTree::tick`, `now`, `next_wake`.
- `tantu`: the runner ticks before each frame and schedules wake-ups at idle.
- `tantu-test`: `WidgetTester::advance`.
- `Tooltip`: a 500 ms show delay on hover (`wait_duration`).

Out of scope (next sub-items): `AnimationController`, `Tween`, curves, implicit transitions,
enter/exit transitions, caret blink, double-click, kinetic scrolling. Also out: wall-clock
dates, periodic timers (restart a one-shot, or use a ticker), timers on other threads.

## Public API

```rust
// tantu-reactive
use std::time::{Duration, Instant};

/// A one-shot callback after a delay. `Copy`; owned by the owner current when created.
#[derive(Clone, Copy)]
pub struct Timer { /* id, runtime */ }

/// Runs `f` once, at the first `tick` whose time is at least `delay` after now (the
/// runtime's time: see REACTIVE-TIME-01).
pub fn timer(delay: Duration, f: impl FnOnce() + 'static) -> Timer;

impl Timer {
    /// Drops the callback without running it. Ignored once fired, cancelled or disposed.
    pub fn cancel(self);
    /// True until it fires, is cancelled or is disposed.
    pub fn is_pending(self) -> bool;
}

/// A callback run once per tick while active, with the time since it started (Flutter's
/// `Ticker`). `Copy`; owned by the owner current when created.
#[derive(Clone, Copy)]
pub struct Ticker { /* id, runtime */ }

/// A ticker calling `f(elapsed)`; created stopped.
pub fn ticker(f: impl FnMut(Duration) + 'static) -> Ticker;

impl Ticker {
    /// Starts (or restarts) it: `elapsed` counts from now.
    pub fn start(self);
    /// Stops it; `f` isn't called until the next `start`.
    pub fn stop(self);
    pub fn is_active(self) -> bool;
}

/// When the runtime next needs a `tick`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    /// A ticker is active: as soon as possible (the next frame).
    Now,
    /// The earliest pending timer's deadline.
    At(Instant),
}

impl Runtime {
    /// Advances the clock to `now` and runs what is due: timers whose deadline has passed
    /// (earliest first), then every active ticker once. Returns how many callbacks ran.
    pub fn tick(&self, now: Instant) -> usize;
    /// The clock: the time of the last `tick`, or `None` before the first.
    pub fn now(&self) -> Option<Instant>;
    /// `Wake::Now` while a ticker is active, else the earliest pending timer, else `None`.
    pub fn next_wake(&self) -> Option<Wake>;
}

// tantu-platform
pub trait PlatformContext {
    /// The platform's monotonic clock.
    fn now(&self) -> Instant;
    /// Asks for an `idle` call at `deadline` (or earlier) even if no event arrives; `None`
    /// cancels. Each call replaces the previous deadline.
    fn wake_at(&mut self, deadline: Option<Instant>);
    // ... unchanged
}

pub enum FakeStep {
    // ... unchanged, plus:
    /// Moves the fake clock forward, then runs a redraw round and `idle`.
    Advance(Duration),
}

pub struct FakeLog {
    // ... unchanged, plus:
    /// Every `wake_at` call, as offsets from the run's start (`None` for a cancel).
    pub wakes: Vec<Option<Duration>>,
}

// tantu-view
impl ViewTree {
    /// `Runtime::tick` with the tree's runtime current.
    pub fn tick(&mut self, now: Instant) -> usize;
    pub fn now(&self) -> Option<Instant>;
    pub fn next_wake(&self) -> Option<Wake>;
}

// tantu-test
impl WidgetTester {
    /// Moves the tester's clock by `by`, ticks the tree and pumps one frame.
    pub fn advance(&mut self, by: Duration);
    /// The tester's clock (starts at the tester's creation).
    pub fn now(&self) -> Instant;
}

// tantu-widgets
impl<V: View> Tooltip<V> {
    /// How long the pointer must rest on the child before the tooltip shows (default 500 ms;
    /// zero shows at once).
    pub fn wait_duration(self, duration: Duration) -> Self;
}
```

## Behavior

### `tantu-reactive`

- **REACTIVE-TIME-01:** `Runtime::tick(now)` sets the clock to `now`, then runs the timers whose
  deadline is at or before `now` (earliest deadline first, creation order among equals), each
  once, then calls every active ticker once with `now − start`, and returns the number of
  callbacks run. A timer created before the first tick, or a ticker started before it, counts
  from the first tick's time. `now()` is the last tick's time, `None` before the first.
- **REACTIVE-TIME-02:** `timer(delay, f)` fires at the first tick at or after `now + delay`
  (a zero delay: the next tick) and never again; `is_pending` is true until then.
  `cancel` drops `f` without running it. A callback runs with the runtime current and the
  timer's owner as the current owner, so signals it writes notify and nodes it creates belong
  to that owner.
- **REACTIVE-TIME-03:** `ticker(f)` is created stopped; after `start`, every tick calls
  `f(elapsed)` with the time since the start; `stop` ends that, and `start` again counts from
  zero. A ticker that stops itself inside `f` isn't called again in that tick; a timer or
  ticker created inside a callback first runs at a later tick.
- **REACTIVE-TIME-04:** Timers and tickers belong to the owner current when they were created:
  disposing it cancels pending timers and stops tickers, dropping each callback exactly once
  (no leaks, as REACTIVE-SIG-22). Methods on a cancelled, fired or disposed handle are ignored
  and `is_pending`/`is_active` return false; nothing panics.
- **REACTIVE-TIME-05:** `next_wake` is `Wake::Now` while any ticker is active or a timer is
  pending before the first tick (its deadline needs a tick to resolve), else
  `Wake::At(deadline)` of the earliest pending timer, else `None`. Firing, cancelling or
  disposing updates it.

### Platform

- **PLATFORM-TIME-01:** `PlatformContext::now` is monotonic and `wake_at(Some(t))` makes the
  platform call `idle` at `t` or soon after, even with no event; a later `wake_at` replaces the
  deadline and `wake_at(None)` cancels it. An event arriving earlier doesn't cancel it.
- **PLATFORM-FAKE-12:** The fake clock starts at `run_logged` and only `Advance(by)` moves it:
  `now` returns the clock; `Advance` moves it, counts as a delivered step, then runs a redraw
  round and `idle` like other steps. `wake_at` calls are logged in `wakes` as offsets from the
  start.
- **PLATFORM-WINIT-14:** `now` is `Instant::now()`; `wake_at(Some(t))` sets winit's control
  flow to `WaitUntil(t)` and `None` to `Wait`, so `about_to_wait` (`idle`) runs when the
  deadline passes.

### View, runner, tester

- **VIEW-TIME-01:** `ViewTree::tick`, `now` and `next_wake` forward to the tree's runtime with
  it current; a callback that writes a bound signal makes the tree need a frame as any write
  does (VIEW-FRAME-04).
- **FACADE-APP-14:** The runner ticks each window's tree with `cx.now()` at the start of every
  redraw, before the frame, so what timers and tickers change is in that frame. At `idle`, it
  takes the earliest wake over all windows: `Wake::Now` or a passed deadline requests a redraw
  of that window; a future deadline is passed to `wake_at`; none cancels the wake-up. An app
  with no pending timer or active ticker makes no further callbacks between events.
- **TEST-WT-13:** The tester's clock starts at its creation; `pump` ticks the tree at the
  current clock before each frame; `advance(by)` moves the clock, ticks and pumps once. A
  timer due within the advance fires in that pump; a ticker runs once per pump.

### Tooltip

- **WIDGETS-TOOLTIP-07:** Hover shows the tooltip after `wait_duration` (default 500 ms) of
  the pointer resting on the child; leaving or pressing before then cancels the show. Focus
  within shows it at once. `wait_duration(Duration::ZERO)` shows on hover at once.

## Performance and allocation

Timers and tickers are arena nodes of the runtime, like signals; `tick` walks the pending
timers (a small sorted list) and the active tickers without allocating. The runner's idle
work is one comparison per window.

## Open questions

Resolved (2026-10-11; the user said "continue" on the draft, taking these proposals):

1. **The clock lives in `tantu-reactive`**, not `tantu-view`: timers share the owner tree with
   signals, so they can be created from handlers (which have no `BuildCx`) and are disposed
   with the scope that made them. The AGENTS.md row already says "scheduler".
2. **`std::time::Instant`/`Duration` as the time types**, no abstraction: both are monotonic
   and testable (a fake clock is an `Instant` advanced by `Duration`s).
3. **One tick per frame, at the start of the redraw** (Flutter's transient-frame callbacks),
   with `idle` only scheduling: a passed deadline requests a redraw rather than ticking in
   place, so callbacks always run once per frame.
4. **`wake_at` on the platform context** (one deadline per loop, replaced by each call) rather
   than per-window timers: timers are app-level, and winit's control flow is loop-level.
5. **`WidgetTester::advance` is one frame**: tests needing n frames of an animation call it n
   times (or `pump` n times after a single advance for same-time frames). A `pump_for`
   helper can come with the animation controller if tests want it.
6. **The tooltip's delay is Flutter's 500 ms**, and focus still shows at once (keyboard users
   get no delay in Flutter either). The existing tooltip tests advance 500 ms.
7. **PLAN.md split** of "Animation" into three sub-items: timers and tickers (this spec),
   `AnimationController`/`Tween`/curves with `animated()` props (implicit transitions on any
   reactive prop, which is Tantu's fine-grained `AnimatedContainer`), and enter/exit
   transitions (`Opacity`, `AnimatedSwitcher`, animated `Show`) with caret blink, double-click
   and kinetic scrolling as users.
