# Transitions: Opacity, AnimatedSwitcher, and the animation's first users

- **Status:** Draft (for review; proposals in Open questions)
- **Crates:** `tantu-widgets` (`Opacity`, `AnimatedSwitcher`, the `TextField` caret blink and
  double-click, `ScrollView` smooth wheel scrolling and `ScrollController::animate_to`),
  `examples/gallery`
- **Plan item:** Phase 3, "Animation", sub-item "enter/exit transitions: `Opacity`,
  `AnimatedSwitcher`, animated `Show`; caret blink, double-click and kinetic scrolling as
  users"; and the scroll item's "smooth wheel animation" sub-item
- **Related:** [animation](../view/animation.md), [timers and tickers](../reactive/timers.md),
  [dynamic content](../view/dynamic.md), [paint](../view/paint.md) (layers),
  [text field](text-field.md), [scroll](scroll.md), [gallery](../examples/gallery.md),
  Flutter's `Opacity`, `FadeTransition`, `AnimatedSwitcher`, `ScrollController.animateTo`

## Purpose

The controller and `animated()` move a prop; this item lets content *appear and disappear*
with a transition, and puts the clock to its first everyday uses. `Opacity` is the one
primitive the fades need (a Scene layer with group opacity). `AnimatedSwitcher` cross-fades
between children keyed by a value, keeping the leaving child alive until its fade ends; with a
`bool` key it is the animated `Show`. The text field's caret blinks and double-click selects a
word; the scroll view animates wheel steps instead of jumping, and apps can scroll to an
offset smoothly.

## Scope

In scope:

- `Opacity` and `AnimatedSwitcher` in `tantu-widgets`.
- `TextField`: caret blink (WIDGETS-TEXTFIELD-10), double-click word selection
  (WIDGETS-TEXTFIELD-11).
- `ScrollView`: smooth wheel scrolling (WIDGETS-SCROLL-07) and
  `ScrollController::animate_to` (WIDGETS-SCROLL-08).
- The gallery's Animation section gains a switcher sample (GALLERY-07).

Out of scope: slide and scale transitions (`SlideTransition`, `ScaleTransition` are a
`Transform` widget away, which the Scene supports; later), `AnimatedList` (needs `For` to keep
leaving items; later, with the list item), kinetic fling (needs touch or trackpad gesture
events winit doesn't deliver on desktop; the wheel is smoothed instead), triple-click,
`ensure_visible` (scroll leftovers, with focus scopes).

## Public API

```rust
// tantu-widgets
/// Paints its child through a Scene layer with group opacity (Flutter's `Opacity`).
pub struct Opacity<V> { /* opacity: Prop<f32>, child */ }
impl<V: View> Opacity<V> {
    /// `opacity` in `0..=1` (clamped; NaN counts as 1); a value, closure, signal or animation.
    pub fn new(opacity: impl IntoProp<f32>, child: V) -> Self;
}

/// Shows the child built for the current key, cross-fading when the key changes (Flutter's
/// `AnimatedSwitcher`). With a `bool` key and an empty view for `false`, an animated `Show`.
pub struct AnimatedSwitcher<K> { /* key: Prop<K>, build, duration, curve */ }
impl<K: Clone + PartialEq + 'static> AnimatedSwitcher<K> {
    /// `build(key)` for each new key; default 300 ms, `EaseInOut`.
    pub fn new(key: impl IntoProp<K>, build: impl Fn(K) -> AnyView + 'static) -> Self;
    pub fn duration(self, duration: Duration) -> Self;
    pub fn curve(self, curve: Curve) -> Self;
}

impl ScrollController {
    /// Animates the offset to `offset` (clamped to the extents at each frame) over `duration`
    /// along `curve`; a wheel step or a jump during the animation takes over.
    pub fn animate_to(&self, offset: f32, duration: Duration, curve: Curve);
}

impl ScrollView {
    /// How long a wheel step takes (default 150 ms; zero applies deltas at once, as before).
    pub fn wheel_duration(self, duration: Duration) -> Self;
}
```

## Behavior

### Opacity

- **WIDGETS-OPACITY-01:** `Opacity::new(o, child)` lays out as its child (same size, same
  position of the child) and paints the child's commands inside a `PushLayer`/`PopLayer`
  scope with opacity `o` clamped to `0..=1`; at exactly 1 no layer is pushed (the commands are
  emitted as without the widget); at 0 nothing of the child is painted (no layer, no
  commands). The prop is reactive: an `Animation<f32>` or `animated()` value repaints each
  frame.
- **WIDGETS-OPACITY-02:** Hit-testing is unchanged: a button inside an `Opacity` at 0.5 or 0
  still receives taps (Flutter's `Opacity` behaves the same; use `IgnorePointer`-style
  wrapping, later, to block input).

### AnimatedSwitcher

- **WIDGETS-SWITCHER-01:** On build, `AnimatedSwitcher` shows `build(key)` for the key's first
  value at full opacity, inside a `Stack` sized by its children. When the key's value changes,
  the current child starts fading out and the new key's child is built and starts fading in,
  both over `duration` along `curve`; halfway, the old child is at opacity 0.5 and the new one
  at 0.5. The old child's element stays (same id, its state and bindings alive) until its fade
  ends, then it is removed and its scope disposed.
- **WIDGETS-SWITCHER-02:** A key that changes back to a child still fading out reverses that
  child's fade from where it is (no rebuild); a key changing to a third value while two
  children fade keeps all of them, each on its own fade, and removes each leaving child as
  its fade ends. The key is compared with `PartialEq`: setting the same value again does
  nothing. A zero duration swaps at the next frame.
- **WIDGETS-SWITCHER-03:** The Stack's children are ordered by arrival: the newest child
  paints last (on top), as Flutter's switcher. Removing the switcher disposes every child,
  fading or not.

### Text field

- **WIDGETS-TEXTFIELD-10:** While focused and enabled, the caret blinks: visible for 500 ms,
  hidden for 500 ms, from the moment the field gains focus or the text or selection changes
  (an edit or a caret move shows it at once and restarts the cycle). `advance(600 ms)` after
  focusing shows no caret; `advance(500 ms)` more shows it again. Losing focus stops the
  blink; the selection highlight does not blink.
- **WIDGETS-TEXTFIELD-11:** A second primary press within 500 ms of the first and within 4 px
  of it selects the word under the pointer (a UAX #29 word, as `Movement::Word` uses; a press
  on whitespace between words selects that whitespace run); dragging after a double-click
  extends the selection by whole words. A third press within the interval starts a new
  single-click selection (triple-click is later).

### Scroll view

- **WIDGETS-SCROLL-07:** A wheel step animates the offset to `current target + delta` over
  `wheel_duration` (default 150 ms) with `EaseOut`, where the target accumulates: three quick
  steps of 40 px end at +120 px without waiting for each animation; `advance(75 ms)` after one
  step shows the offset part way (more than half, with ease-out) and `advance(150 ms)` the
  full step. The event is stopped when the target moved, as WIDGETS-SCROLL-02; with a zero
  `wheel_duration` the delta applies at once, as before. Keys, the thumb drag and `jump_to`
  apply at once and cancel a running wheel animation.
- **WIDGETS-SCROLL-08:** `ScrollController::animate_to(offset, duration, curve)` animates from
  the current offset to `offset` (clamped to the extents at each frame) and ends exactly
  there; `jump_to`, a key, a thumb drag or a wheel step during it takes over. Calling it with
  a zero duration is `jump_to`.

### Gallery

- **GALLERY-07:** The Animation section gains a "Next" button (key `next`) cycling a
  switcher (key `switcher`) through three labelled cards ("One", "Two", "Three"); after
  tapping it and `advance(150 ms)`, both the old and the new card are present, the old at
  opacity 0.5; after `advance(150 ms)` more only the new card remains. The animation golden
  (GALLERY-06) is regenerated with the sample in its initial state.

## Performance and allocation

An `Opacity` at 1 costs nothing beyond the paint call; a fading child costs one layer per
frame (the renderers allocate layer targets once and pool them). A switcher keeps at most as
many children as are fading plus one. The caret blink is one timer per focused field; the
wheel animation one controller per scroll view.

## Open questions

Proposals (say "continue" to take them, or change them):

1. **`Opacity` at 1 pushes no layer and at 0 paints nothing**, so the common cases cost
   nothing (Flutter's `Opacity` does the same).
2. **`AnimatedSwitcher` keeps leaving children alive** (element, state, bindings) until their
   fade ends, built on a region with one scope per child; the animated `Show` is a switcher
   on a `bool`, not a separate widget or a `Show` change in `tantu-view`.
3. **Fades only** for now: `AnimatedSwitcher` has no `transition_builder`; slide/scale come
   with a `Transform` widget when something needs them.
4. **Caret blink at 500 ms/500 ms** (Flutter's `_kCursorBlinkHalfPeriod`) and restarting on
   edits, with a timer rather than a ticker (one wake-up per half period, not a frame per
   vsync).
5. **Double-click only**, with Flutter's 4 px slop and a 500 ms interval; triple-click when a
   line-selection use case appears.
6. **Wheel steps animate to an accumulating target** (as browsers do) with `EaseOut`, 150 ms
   by default, and `wheel_duration(ZERO)` keeps today's behavior; kinetic fling is deferred
   until touch or trackpad gestures exist.
7. **`ScrollController::animate_to` takes a curve and duration** rather than Flutter's
   named-argument form; `ensure_visible` stays with the scroll leftovers.
