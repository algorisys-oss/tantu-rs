# Button: keyboard activation and focus indicator

- **Status:** Agreed (the user said "continue" on the draft, taking its proposals)
- **Crate:** `tantu-widgets`
- **Plan item:** Phase 3, "Focus system, keyboard navigation, shortcuts/command registry" →
  "`Button`: Space/Enter activation when focused, focus indicator"
- **Related:** [Text and Button](basic.md), [keyboard events and focus](../view/focus.md),
  [shortcuts and commands](../view/shortcuts.md)

## Purpose

A button must be usable without a mouse: Tab reaches it, Space or Enter presses it, and you
can see which button has focus. This extends `Button` (WIDGETS-BUTTON-01..04) with focus,
keyboard activation and a focus indicator, using Flutter's Material 3 defaults.

## Scope

In scope:

- `Button` is focusable and traversable. A click doesn't focus it (decided in the focus spec).
- Space and Enter activate a focused, enabled button.
- A focus indicator drawn around the focused button, and a slightly darker fill while focused.

Out of scope:

- An `ActivateIntent` that other widgets share: with more activatable widgets (checkbox,
  switch) later in Phase 3; then `Button` moves to it.
- Showing the indicator only after keyboard use ("focus visible", as the web does): later, with
  the theme. For now it shows whenever the button has focus.

## Behavior

- **WIDGETS-BUTTON-05:** A `Button` is focusable and traversable: Tab reaches it in tree order,
  and `WidgetTester::focus` works on it. A disabled button isn't traversable and loses focus
  when it becomes disabled.
- **WIDGETS-BUTTON-06:** While focused and enabled, a press of Space (`Character(" ")`) or Enter
  calls `on_press` once and stops the key event. Releases and repeats don't call it, and the
  key isn't handled when the button is disabled.
- **WIDGETS-BUTTON-07:** While focused, the button draws a 3 px focus outline in its primary
  color (`0xFF6750A4`), 2 to 5 px outside its bounds, following its rounded shape. It is a
  Scene stroke, which lies inside its shape, of width 3 on the bounds inflated by 5 px with
  radius 25. Its fill gets 10 % white over the idle color (Material 3's focus
  state layer), which falls between hovered (8 %) and pressed (12 %) and combines with them by
  taking the strongest. The outline counts in `Paint::overflow` (5 px), so culling keeps it.
- **VIEW-FOCUS-08:** `ElementMut::set_focusable(options)` changes whether an element is
  focusable after it was built, so a prop can drive it. `Some(options)` makes the element
  focusable with those options, and `None` makes it unfocusable and clears the focus if it
  had it, reporting the change. (Added to `tantu-view` while agreeing this spec: a button
  that becomes disabled must leave traversal.)

## Open questions

Resolved (2026-10-10; the user said "continue" on the draft):

1. **Material 3's focus indicator** and state layer.
2. **Activation on key press.**
3. **The indicator shows whenever the button has focus.**

Added while agreeing (review): VIEW-FOCUS-08, `ElementMut::set_focusable`.
