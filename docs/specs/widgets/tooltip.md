# Tooltip

- **Status:** Implemented (the user said "continue" on the draft, taking its proposals)
- **Crate:** `tantu-widgets`
- **Plan item:** Phase 3, "Widgets" → "`Tooltip`"
- **Related:** [overlay](../view/overlay.md), [events](../view/events.md),
  [focus](../view/focus.md), [Text and Button](basic.md), Flutter's `Tooltip`

## Purpose

A tooltip shows a short message about a widget while the pointer rests on it, or while it has
keyboard focus. It's the first widget built on the overlay layer: an `OverlayPortal` around the
child, shown on hover, anchored above or below it, and passing pointer events through so it
never gets in the way. The look is Material 3's plain tooltip, in the theme's inverse colors ([theme](../theme/theme.md)).

## Scope

In scope:

- `Tooltip::new(message, child)`: show on pointer enter, hide on leave or press, also show
  while the child (or something inside it) has focus.
- Placement below the child by default, `.prefer_above()`, with the overlay's flip and clamp.
- Material 3 plain-tooltip look:
  - colors: inverse surface `0xFF313033` behind inverse on-surface text `0xFFF4EFF4`;
  - shape: 4 px corner radius, 8 px horizontal and 4 px vertical padding, at least 24 px tall;
  - text: 12 px;
  - placement: 4 px from the child.

Out of scope (and where it goes):

- Fade animations: they need the animation controller, which comes with the Phase 3 animation
  item (the 500 ms show delay arrived with WIDGETS-TOOLTIP-07).
- Rich tooltips (title, actions), touch long-press: later.
- Accessibility (the message as the child's description): `tantu-a11y`.

## Public API

```rust
/// A short message shown near `child` while hovered or focused (Material 3 plain tooltip).
pub struct Tooltip<V> { /* message, child, above */ }

impl<V: View> Tooltip<V> {
    /// A tooltip showing `message` (a value, a closure or a signal) for `child`.
    pub fn new(message: impl IntoProp<String>, child: V) -> Self;
    /// Show above the child instead of below (it still flips when there's no room).
    pub fn prefer_above(self) -> Self;
    /// How long the pointer must rest on the child before the tooltip shows (default 500 ms;
    /// zero shows at once) (WIDGETS-TOOLTIP-07).
    pub fn wait_duration(self, duration: std::time::Duration) -> Self;
}
```

## Behavior

- **WIDGETS-TOOLTIP-01:** The tooltip is hidden at first. Pointer enter on the child shows it,
  and pointer leave or a primary press on the child hides it. It's shown in the overlay layer
  as a rounded box with the message, laid out from `TextStyle` 12 px in the colors above, and
  at least 24 px tall.
- **WIDGETS-TOOLTIP-02:** It sits 4 px below the child, horizontally centered on it, or 4 px above
  with `prefer_above`. Near a window edge it flips and clamps like any overlay.
- **WIDGETS-TOOLTIP-03:** It passes pointer events through: pressing where the tooltip is reaches
  what's under it.
- **WIDGETS-TOOLTIP-04:** While the child, or an element inside it, has keyboard focus, the
  tooltip shows. It hides when the focus leaves, unless the pointer is still over the child.
- **VIEW-FOCUS-09:** `BuildCx::on_focus_within(element, handler)` calls `handler(true)` when the
  focus moves into `element`'s subtree (the element or a descendant) from outside it, and
  `handler(false)` when it leaves the subtree. Moves inside the subtree call nothing. It runs
  with the tree's runtime current, after the focused elements' own `on_focus_change`
  handlers. (Added to `tantu-view` while agreeing this spec, for WIDGETS-TOOLTIP-04.)
- **WIDGETS-TOOLTIP-05:** The message is reactive; a change shows at the next frame.
- **WIDGETS-TOOLTIP-06:** The child must be a render element to receive hover. A region child
  (`Dyn`, `Show`, `For`) logs a warning, and the tooltip never shows.
- **WIDGETS-TOOLTIP-07:** Hover shows the tooltip after `wait_duration` (default 500 ms);
  leaving or pressing before then cancels; focus shows at once. WIDGETS-TOOLTIP-01's hover
  case is read with the delay elapsed. See [timers and tickers](../reactive/timers.md).

## Open questions

Resolved (2026-10-11; the user said "continue" on the draft):

1. **No show delay until timers exist** (they do now: WIDGETS-TOOLTIP-07).
2. **The tooltip shows on focus as well as hover.**
3. **Material 3 plain-tooltip look.**

Added while agreeing (review): VIEW-FOCUS-09, `BuildCx::on_focus_within`.
