# Checkbox, Radio and Switch

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-11; review
  the Open questions)
- **Crate:** `tantu-widgets`
- **Plan item:** Phase 3, "Widgets" → "`Checkbox`, `Radio`, `Switch`"
- **Related:** [Text and Button](basic.md), [Button keyboard](button-keyboard.md),
  [Slider](slider.md) (controlled widgets, state layers), [events](../view/events.md),
  [focus](../view/focus.md), [Scene](../scene/scene.md) (transforms), Flutter's `Checkbox`,
  `Radio` and `Switch`

## Purpose

The three Material toggles. Like `Slider` they are controlled: each shows a value the app owns
and reports the value the user asked for through `on_change`; the app writes its signal and
the widget follows at the next frame. All three share one interaction model (press and
release inside, Space or Enter while focused, hover/focus/pressed state layers, disabled when
there is no `on_change`), which this spec states once. The look is Material 3's until
`tantu-theme`.

The check mark is the first shape that isn't a (rounded) rectangle. The Scene has no paths
yet; it does have transforms, so the mark is drawn as two rotated bars inside a transform
scope. That works on every renderer today and needs no new Scene command. Paths come later
(charts need them anyway) and the mark can move to one then.

## Scope

In scope:

- `Checkbox::new(value)`, `.indeterminate(..)`, `.enabled(..)`, `.on_change(..)`.
- `Radio::new(value, group)`, `.enabled(..)`, `.on_change(..)` for any `T: Copy + PartialEq`.
- `Switch::new(value)`, `.enabled(..)`, `.on_change(..)`.
- Pointer and keyboard activation, focus, state layers, disabled look.
- Material 3 metrics and colors (desktop tap targets); one golden per widget.

Out of scope (and where it goes):

- Labels (`CheckboxListTile`): compose a `Row` for now; list tiles come with lists.
- Animations (the switch thumb sliding, the check drawing in): the animation item.
- Arrow keys moving the selection within a radio group (Flutter's `RadioGroup`): with menus
  and forms, which bring groups.
- A user-cycled tristate checkbox (Flutter's `tristate`): `indeterminate` is app-driven here
  (a "select all" box shows it while some rows are selected); see Open questions.
- Theming, density, custom shapes: `tantu-theme`. Accessibility: `tantu-a11y`.

## Public API

```rust
// tantu-widgets
/// A box that is checked or not (Flutter's `Checkbox`, Material 3 look). Controlled.
pub struct Checkbox { /* value, indeterminate, enabled, on_change */ }

impl Checkbox {
    /// A checkbox showing `value`, enabled, reporting nothing (so drawn disabled until
    /// `on_change` is set).
    pub fn new(value: impl IntoProp<bool>) -> Self;
    /// Shows a dash instead of the check or the empty box while true (a partial selection).
    pub fn indeterminate(self, indeterminate: impl IntoProp<bool>) -> Self;
    /// A disabled checkbox ignores input and draws dimmed. No `on_change` disables it too.
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
    /// Called with `!value` when the user activates the checkbox.
    pub fn on_change(self, f: impl Fn(bool) + 'static) -> Self;
}

/// One choice of a group; selected while the group's value equals its own (Flutter's
/// `Radio<T>`). Controlled.
pub struct Radio<T> { /* value, group, enabled, on_change */ }

impl<T: Copy + PartialEq + 'static> Radio<T> {
    /// A radio for `value`, selected while `group` (a closure or a signal) equals it.
    pub fn new(value: T, group: impl IntoProp<T>) -> Self;
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
    /// Called with this radio's `value` when the user activates it.
    pub fn on_change(self, f: impl Fn(T) + 'static) -> Self;
}

/// An on/off switch (Flutter's `Switch`, Material 3 look). Controlled.
pub struct Switch { /* value, enabled, on_change */ }

impl Switch {
    pub fn new(value: impl IntoProp<bool>) -> Self;
    pub fn enabled(self, enabled: impl IntoProp<bool>) -> Self;
    /// Called with `!value` when the user activates the switch.
    pub fn on_change(self, f: impl Fn(bool) + 'static) -> Self;
}
```

Each widget is one render element with a `RenderConstrainedBox` tight at its size (Flutter's
desktop `shrinkWrap` tap targets): `Checkbox` and `Radio` 40 × 40, `Switch` 60 × 40. The
element holds the pointer and key handlers, the focus and the widget's `Paint`; the visual is
centered in it. Values: the state layer is a 40 px circle (radius 20) centered on the visual
(on the switch's thumb), in primary while selected and in on-surface `0xFF1C1B1F` while not,
at 8 % hovered, 10 % focused, 12 % pressed (the strongest wins), painted first.

Colors ("38 %" means that alpha over the base, as a packed ARGB constant):

| Part | Enabled, selected | Enabled, unselected | Disabled |
|---|---|---|---|
| checkbox box | fill primary `0xFF6750A4` | 2 px outline on-surface-variant `0xFF49454F`, no fill | outline or fill on-surface 38 % `0x611C1B1F` |
| check mark / dash | on-primary (white) | – | white |
| radio ring | primary | on-surface-variant | on-surface 38 % |
| radio dot | primary | – | on-surface 38 % |
| switch track | fill primary | fill surface-container-highest `0xFFE6E0E9`, 2 px outline `0xFF79747E` | fill on-surface 12 % `0x1F1C1B1F`, outline on-surface 12 % when unselected |
| switch thumb | white | outline `0xFF79747E` | on-surface 38 % |

Geometry, in the element's coordinates (`c` is the element's center):

- Checkbox: an 18 × 18 box with 2 px radius centered on `c`. The box is a filled rounded
  rect when selected, a 2 px stroke (inside its edge) when not. The check mark is Flutter's:
  from `(0.15, 0.45)` through `(0.40, 0.70)` to `(0.85, 0.25)` of the box, 2 px wide,
  drawn as two bars of width 2 (each extended by 1 px at both ends) inside a Scene transform
  scope per bar (`push_transform(translate(mid) · rotate(angle))`, a filled rect centered on
  the origin, `pop`). The dash is a 2 px tall rect from 20 % to 80 % of the box's width at
  its vertical center.
- Radio: a circle of radius 8 centered on `c` as a 2 px stroke (ring); when selected, a
  filled circle of radius 4.5 inside it.
- Switch: a 52 × 32 track (radius 16) centered on `c`, with a 2 px stroke inside its edge when
  unselected. The thumb is a circle centered vertically: unselected, radius 8 at 16 px from
  the track's left edge; selected, radius 12 at 16 px from its right edge; while pressed,
  radius 14 at the same center.

## Behavior

- **WIDGETS-TOGGLE-01:** Each widget builds one render element tight at its size: `Checkbox`
  and `Radio` 40 × 40, `Switch` 60 × 40, with the pointer and key handlers, the focus and
  the paint on it.
- **WIDGETS-TOGGLE-02:** `Checkbox` paints as described: unselected, the outline only;
  selected, the filled box and the two check-mark bars in two transform scopes; with
  `indeterminate`, the filled box (selected or not) and the dash. Checked against a golden
  (`crates/tantu-widgets/tests/goldens/checkbox.png`).
- **WIDGETS-TOGGLE-03:** `Radio` paints the ring, plus the dot while `group == value`. Golden
  `radio.png`.
- **WIDGETS-TOGGLE-04:** `Switch` paints the track (outlined when unselected) and the thumb at
  the unselected or selected position and size; while pressed the thumb has radius 14.
  Golden `switch.png`.
- **WIDGETS-TOGGLE-05:** A toggle is disabled when `enabled` is false or it has no
  `on_change`. It draws the disabled column, has no state layer, ignores the pointer and
  keys, isn't focusable (so it leaves traversal and loses focus), and becoming disabled clears
  hovered and pressed.
- **WIDGETS-TOGGLE-06:** Pointer: a primary `Down` on an enabled toggle makes it pressed and
  stops the event; the matching `Up` clears pressed and, if inside the bounds, activates it:
  `Checkbox` and `Switch` call `on_change(!value)` (whatever `indeterminate` is), `Radio`
  calls `on_change(value)` (also when already selected). Other buttons do nothing.
- **WIDGETS-TOGGLE-07:** Keyboard: a toggle is focusable and traversable while enabled (a
  press doesn't focus it, as for `Button`). While focused, a press of Space or Enter (not a
  repeat or a release) activates it as a pointer release does, and stops the event.
- **WIDGETS-TOGGLE-08:** State layer as described: hovered from `Enter` until `Leave`, pressed
  from `Down` until `Up`, focused while the element has focus; none when disabled.
- **WIDGETS-TOGGLE-09:** `value`, `group`, `indeterminate` and `enabled` are reactive and
  applied at the next frame, repainting only.

## Performance and allocation

A toggle paints at most 6 Scene commands (the checkbox: layer, box, two transform scopes of
three commands each). Nothing is re-laid out for a value change.

## Open questions

Decided by the agent while the user was away (review):

1. **The check mark as two rotated bars** in Scene transform scopes, rather than adding a
   path command to the Scene now (both renderers would need a path rasterizer; the wgpu one
   is real work) or an icon font (the next item). Revisit when paths or icons exist.
2. **App-driven `indeterminate`** instead of Flutter's user-cycled `tristate`: a partial
   selection is what enterprise UIs need (a "select all" header), and activation then simply
   reports `!value`. A cycling tristate can be built on it.
3. **Desktop tap targets** (40 × 40, Flutter's `shrinkWrap`), since Tantu targets the desktop.
4. **Direct Space/Enter handlers** as `Button` has, not the shortcut/intent machinery: shortcuts
   fire on key repeats (VIEW-SHORT-01) and an intent doesn't say whether the key repeated.
   The shared `ActivateIntent` stays deferred.
5. **`Radio::new(value, group)`** with the group value as a prop, as Flutter's `groupValue`,
   rather than a `RadioGroup` widget; a group widget can come with forms.
6. **No focus on press**, consistent with `Button` (the slider differs because click-then-arrows
   is its main keyboard use).
