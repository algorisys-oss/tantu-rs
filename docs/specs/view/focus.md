# Keyboard events and focus

- **Status:** Implemented (the user said "continue" on the draft, taking its proposals)
- **Crates:** `tantu-view` (focus, key dispatch), `tantu` (the runner forwards keyboard events)
- **Plan item:** Phase 3, "Focus system, keyboard navigation, shortcuts/command registry" →
  "Keyboard events and focus"
- **Related:** [events](events.md) (pointer dispatch, whose phases this mirrors),
  [platform](../platform/platform.md) (`KeyEvent`, `Modifiers`), [app runner](../facade/app.md),
  [WidgetTester](../test/widget-tester.md)

## Purpose

Keyboard input needs a target. Flutter's model: one element has the *primary focus*, key
events travel along the path from the root to it, and Tab moves focus through the focusable
elements. This spec adds that to the view tree:

- elements opt in to focus;
- the tree tracks the focused element;
- key events are dispatched to the focused path in capture and bubble phases, like pointer
  events;
- an unhandled Tab or Shift+Tab moves focus;
- focus changes are reported to the elements involved.

The app runner forwards the platform's keyboard events, and `WidgetTester` gets `press_key`
and `focus`.

## Scope

In scope:

- `KeyEvent`, `LogicalKey`, `NamedKey`, `Modifiers` (tantu-view's own types, as decided for
  pointer events; the runner converts).
- `BuildCx::focusable`, `BuildCx::on_key`, `BuildCx::on_focus_change`.
- `ViewTree::focus`, `focused`, `unfocus`, `focus_next`, `focus_previous`, `dispatch_key`.
- Pointer presses focusing an element that asks for it (`focus_on_press`).
- The runner forwarding `WindowEvent::Keyboard` and `ModifiersChanged`.
- `WidgetTester::press_key` and `WidgetTester::focus`.

Out of scope (and where it goes):

- Shortcuts, intents and actions: the next spec, "Shortcuts and commands".
- `Button` activation with Space/Enter and its focus indicator: the `Button` amendment after
  this.
- Text entry, IME and `TextField`: the Phase 3 widgets item.
- Focus scopes and groups (`FocusScope`, `FocusTraversalGroup`), directional (arrow-key)
  traversal and restoring focus when a window regains it: later, when dialogs and grids need
  them.
- Accessibility focus: `tantu-a11y`.

## Public API

```rust
// tantu-view
/// A key that isn't a character.
pub enum NamedKey { Enter, Tab, Escape, Backspace, Delete, Insert, ArrowLeft, ArrowRight,
    ArrowUp, ArrowDown, Home, End, PageUp, PageDown, F(u8), Other }

/// What a key means (Flutter's `LogicalKeyboardKey`, simplified).
pub enum LogicalKey { Named(NamedKey), Character(Arc<str>) } // Space is Character(" ")

/// Modifier keys held during a key event.
pub struct Modifiers { pub shift: bool, pub control: bool, pub alt: bool, pub super_key: bool }

/// A key press or release.
pub struct KeyEvent {
    pub key: LogicalKey,
    pub pressed: bool,          // false for a release
    pub repeat: bool,
    pub modifiers: Modifiers,
    pub text: Option<Arc<str>>, // text the key produced (until IME)
}

/// How an element takes part in focus.
pub struct FocusOptions {
    /// Tab and Shift+Tab stop here (default true).
    pub traversable: bool,
    /// A primary press on the element focuses it (default false; text fields set it).
    pub focus_on_press: bool,
}

/// What a key handler receives.
pub struct KeyCx<'a> { pub event: &'a KeyEvent, pub element: ElementId, pub phase: Phase }

impl BuildCx<'_> {
    /// Makes `element` focusable.
    pub fn focusable(&mut self, element: ElementId, options: FocusOptions);
    /// Calls `handler` for key events in `phase` on the focused path through `element`.
    pub fn on_key(&mut self, element: ElementId, phase: Phase,
        handler: impl Fn(&KeyCx<'_>) -> Handled + 'static);
    /// Calls `handler(true)` when `element` gains focus and `handler(false)` when it loses it,
    /// with the tree's runtime current.
    pub fn on_focus_change(&mut self, element: ElementId, handler: impl Fn(bool) + 'static);
}

impl ViewTree {
    /// Focuses `element` if it is focusable; returns whether it did.
    pub fn focus(&mut self, element: ElementId) -> bool;
    /// The focused element.
    pub fn focused(&self) -> Option<ElementId>;
    /// Removes focus.
    pub fn unfocus(&mut self);
    /// Moves focus to the next traversable element in tree order, wrapping at the end
    /// (the first one when nothing is focused); returns the new focus.
    pub fn focus_next(&mut self) -> Option<ElementId>;
    /// The previous one, wrapping at the start (the last one when nothing is focused).
    pub fn focus_previous(&mut self) -> Option<ElementId>;
    /// Delivers `event` to the focused path: capture handlers root to focused element, then
    /// bubble handlers back, until one returns `Stop`. An unhandled Tab (Shift+Tab) press
    /// moves focus forward (backward). Returns whether anything handled it.
    pub fn dispatch_key(&mut self, event: KeyEvent) -> bool;
}

// tantu-test
impl WidgetTester {
    /// Presses and releases `key` (with `modifiers`) on the focused element, then pumps.
    pub fn press_key(&mut self, key: LogicalKey, modifiers: Modifiers);
    /// Focuses the single element `finder` matches, then pumps.
    pub fn focus(&mut self, finder: &Finder);
}
```

## Behavior

- **VIEW-FOCUS-01:** `focus` focuses a focusable element and returns true. For an element
  that isn't focusable, or an unknown one, it returns false and changes nothing. `focused`
  reports the focus, and `unfocus` clears it.
- **VIEW-FOCUS-02:** Focus changes call `on_focus_change` handlers: `false` on the element
  losing focus first, then `true` on the one gaining it. Handlers run with the tree's runtime
  current, and refocusing the focused element calls nothing.
- **VIEW-FOCUS-03:** `focus_next` and `focus_previous` visit the traversable focusable elements
  in tree order (depth first, children in order) and wrap around. With nothing focused they
  start at the first or last. Elements with `traversable: false` can still be focused
  directly, and a tree with no traversable element leaves the focus unchanged.
- **VIEW-FOCUS-04:** `dispatch_key` delivers to the focused element's path, root first in the
  capture phase, then the focused element first in the bubble phase. `Stop` ends the dispatch.
  With nothing focused, only the root's handlers run (so app-wide keys still work).
- **VIEW-FOCUS-05:** An unhandled Tab press moves focus forward, and Shift+Tab backward. The
  dispatch then returns true. Releases and handled Tabs don't move focus.
- **VIEW-FOCUS-06:** A primary pointer press on an element with `focus_on_press` (or inside
  one: the innermost such element on the hit path) focuses it before the pointer handlers run.
- **VIEW-FOCUS-07:** Removing the focused element (or an ancestor) clears the focus and calls
  its `on_focus_change(false)`. Removing elements drops their key and focus handlers.
- **FACADE-APP-11:** The runner converts `WindowEvent::Keyboard` to a `KeyEvent` with the
  current modifiers (tracked from `ModifiersChanged`) and dispatches it to the window's tree.
  Named keys map one to one, `Character` keys keep their text, and modifier keys and
  unidentified keys aren't dispatched.
- **TEST-WT-07:** `press_key` dispatches a press and a release of the key to the tester's tree,
  then pumps. `focus` focuses the found element, then pumps, and panics if it isn't
  focusable.

## Open questions

Resolved (2026-10-10; the user said "continue" on the draft):

1. **Traversal in tree order.**
2. **Buttons don't take focus on press;** text fields will set `focus_on_press`.
3. **No focus scopes yet:** one focus per window.
4. **Own `LogicalKey` and `Modifiers` types in `tantu-view`.**
