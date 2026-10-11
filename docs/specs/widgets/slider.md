# Slider

- **Status:** Agreed (the user said "continue" on the draft, taking its proposals)
- **Crates:** `tantu-widgets` (plus `WidgetTester::drag` in `tantu-test`)
- **Plan item:** Phase 3, "Widgets" → "`Slider`"
- **Related:** [Divider](divider.md) (`LimitedBox`), [Text and Button](basic.md),
  [Button keyboard](button-keyboard.md), [events](../view/events.md) (pointer capture),
  [focus](../view/focus.md), [WidgetTester](../test/widget-tester.md), Flutter's `Slider`

## Purpose

A slider picks a number in a range by dragging a thumb along a track, with the keyboard as an
equal citizen. It is the first widget with a drag gesture (pressed, moved while captured,
released), and the first with a value that the app owns: like Flutter's `Slider` it is
*controlled*. The widget reports changes through `on_change`, and the thumb shows `value`,
which the app updates (normally by writing the signal it passed in). The look is Material 3's
until `tantu-theme`.

## Scope

In scope:

- `Slider::new(value)`, `range`, `divisions`, `enabled`, `on_change`, `on_change_start`,
  `on_change_end`.
- Press and drag with the pointer; arrow keys, Home and End; focus on press.
- Material 3 look: 4 px track, 20 px thumb, 48 px state layer, tick marks for discrete sliders.
- `WidgetTester::drag`.

Out of scope (and where it goes):

- The value indicator (a label over the thumb while dragging): with the animation item, since
  it fades in and out; it will use the overlay.
- `RangeSlider`, vertical sliders (Flutter rotates), a secondary track value: later.
- Theming and density: `tantu-theme`. Accessibility (value, increase/decrease actions):
  `tantu-a11y`.
- Thumb movement animations: the animation item.

## Public API

```rust
// tantu-widgets
/// Picks a number in a range by dragging a thumb (Flutter's `Slider`, Material 3 look).
///
/// The slider is controlled: it shows `value` and reports changes through `on_change`.
///
/// ```
/// let volume = signal(0.5);
/// Slider::new(volume).on_change(move |v| volume.set(v))
/// ```
pub struct Slider { /* value, min, max, divisions, enabled, on_change, on_change_start, on_change_end */ }

impl Slider {
    /// A continuous slider showing `value` in the range 0 to 1, enabled, reporting nothing.
    pub fn new(value: impl IntoProp<f64>) -> Self;
    /// The range (default 0 to 1).
    pub fn range(self, min: impl IntoProp<f64>, max: impl IntoProp<f64>) -> Self;
    /// A discrete slider with `divisions` equal steps (0, the default, is continuous).
    pub fn divisions(self, divisions: u32) -> Self;
    /// A disabled slider ignores input and draws dimmed. A slider with no `on_change` is
    /// disabled too, as in Flutter.
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
    /// Called with the new value while the user drags or presses a key.
    pub fn on_change(self, f: impl Fn(f64) + 'static) -> Self;
    /// Called with the value before a drag starts.
    pub fn on_change_start(self, f: impl Fn(f64) + 'static) -> Self;
    /// Called with the value when a drag ends.
    pub fn on_change_end(self, f: impl Fn(f64) + 'static) -> Self;
}

// tantu-test
impl WidgetTester {
    /// A primary press at the center of the single element `finder` matches, a move by
    /// `delta`, and a release there, pumping after each.
    pub fn drag(&mut self, finder: &Finder, delta: Vec2);
}
```

Structure: a render element with a `RenderLimitedBox` (`max_width` 192) around a render
element with a `RenderConstrainedBox` of `BoxConstraints::expand(None, Some(48.0))`. The
outer element, which `Keyed` and finders see and which is the same size as the inner one,
holds the pointer and key handlers, the focus and the `SliderPaint`; the inner one only
provides the size.

Geometry, in the element's coordinates, for a width `w` (Material 3 metrics, as Flutter's
defaults): thumb radius 10, track height 4, state-layer radius 24, so the track runs from
`x = 10` to `x = w − 10` (length `L = max(w − 20, 0)`) at `y = 24`, and the thumb's center is
at `x = 10 + t · L` for the fraction `t` below.

Colors ("over" is source-over in sRGB, as for `Button`):

| Part | Enabled | Disabled |
|---|---|---|
| active track (left of the thumb) | primary `0xFF6750A4` | on-surface 38 % `0x611C1B1F` |
| inactive track (right of it) | surface-container-highest `0xFFE6E0E9` | on-surface 12 % `0x1F1C1B1F` |
| thumb | primary | on-surface 38 % |
| tick on the active track | white 38 % `0x61FFFFFF` | the same |
| tick on the inactive track | on-surface-variant 38 % `0x6149454F` | the same |
| state layer | primary at 8 % hovered, 10 % focused, 12 % dragged | none |

## Behavior

- **WIDGETS-SLIDER-01:** A `Slider` is 48 px tall and as wide as the incoming maximum width,
  or 192 px wide when that is unbounded.
- **WIDGETS-SLIDER-02:** The fraction `t` is `(value − min) / (max − min)` clamped to 0..=1; it
  is 0 when `value` isn't finite or the range's span isn't finite and positive. With
  `divisions` `n ≥ 1`, `t` is snapped to `round(t · n) / n`. The thumb is centered at
  `(10 + t · L, 24)`.
- **WIDGETS-SLIDER-03:** Enabled look, painted in this order: the active track as a rounded
  rectangle (radius 2) from `x = 10` to the thumb's center, 4 px tall centered on `y = 24`;
  the inactive track from the thumb's center to `x = w − 10`; with divisions, a 2 px dot at
  each of the `n + 1` division points; the state layer (WIDGETS-SLIDER-07); then the thumb as
  a 20 px circle (a `RoundedRect` with circular radius 10) at its center. Colors from the
  table. Checked against a golden (`crates/tantu-widgets/tests/goldens/slider.png`).
- **WIDGETS-SLIDER-04:** A slider is disabled when `enabled` is false or it has no
  `on_change`. It draws the disabled colors with no state layer, ignores the pointer and keys,
  isn't focusable (so it leaves traversal, VIEW-FOCUS-08), and becoming disabled clears
  hovered, dragging and focus.
- **WIDGETS-SLIDER-05:** Pointer: a primary `Down` on an enabled slider stops the event, calls
  `on_change_start` with the current value, then reports the value under the pointer:
  `t = (local.x − 10) / L` clamped (0 when `L` is 0), snapped, as `min + t · (max − min)`.
  Every `Move` until the `Up` (captured, so also outside the bounds, VIEW-EVENT-03) reports
  the value under the pointer the same way, and the `Up` calls `on_change_end` with the last
  one. `on_change` is called only when the value differs from the one last reported in the
  gesture (`value` before the first). The thumb follows `value`, not the pointer: an app that
  doesn't update `value` sees the thumb stay.
- **WIDGETS-SLIDER-06:** Keyboard: the slider is focusable and traversable while enabled, and a
  primary press focuses it (`focus_on_press`). While focused, a press (repeats included) of
  ArrowRight or ArrowUp reports `value + step`, ArrowLeft or ArrowDown `value − step`, Home
  `min` and End `max`, clamped to the range and snapped, where `step` is `(max − min) /
  divisions` for a discrete slider and 5 % of the range otherwise. These keys stop the event
  whether or not the value changes; `on_change` is called only when it does. Other keys and
  releases pass through.
- **WIDGETS-SLIDER-07:** State layer: a 48 px circle centered on the thumb, in primary at 8 %
  while hovered (`Enter` until `Leave`), 10 % while focused, 12 % while dragging (`Down` until
  `Up`); the strongest wins. The state layer paints under the thumb and over the tracks.
- **WIDGETS-SLIDER-08:** `value`, `min`, `max` and `enabled` are reactive and applied at the
  next frame; a change repaints the slider without re-laying it out.
- **TEST-WT-08** (in [WidgetTester](../test/widget-tester.md)): `drag` presses the primary button at the found element's center, moves by
  `delta` and releases there, pumping after each step, and panics like `tap` when the finder
  matches no element or several.

## Performance and allocation

A slider paints at most 5 + (n + 1) Scene commands. A pointer move during a drag writes no
signal of its own: it reports through `on_change`, and the app's signal write schedules the
repaint.

## Open questions

Resolved (2026-10-11; the user said "continue" on the draft):

1. **`f64` values.** The value is app data, not geometry, so it takes the precision apps
   use for numbers (Flutter uses `double`). Geometry stays `f32`.
2. **Strictly controlled, as Flutter.** The thumb never moves on its own; `on_change` reports
   and the app writes the signal. The one-line `Slider::new(v).on_change(move |x| v.set(x))`
   is the common case. A `bind(signal)` shortcut can be added later if it proves tedious.
3. **Flutter's keyboard model:** 5 % steps for continuous sliders, one division otherwise,
   Home/End to the ends, repeats accepted, `on_change` only (no start/end) for keys.
4. **Focus on press**, unlike `Button`, so that click-then-arrow-keys works, as Material's
   slider does.
5. **A disabled slider is one without `on_change`** too (Flutter), not only `enabled(false)`.
6. **Fixed 48 px height**, where Flutter's render slider takes the bounded maximum height.
   Taking the maximum would make a slider in a `Column` fill the remaining height.
