# Shortcuts and commands

- **Status:** Implemented (the user said "continue" on the draft, taking its proposals)
- **Crate:** `tantu-view`
- **Plan item:** Phase 3, "Focus system, keyboard navigation, shortcuts/command registry" →
  "Shortcuts and commands"
- **Related:** [keyboard events and focus](focus.md), Flutter's `Shortcuts`, `Actions`,
  `Intent` and `SingleActivator`

## Purpose

Apps bind keys to commands: Ctrl+S saves, Escape closes, Ctrl+Z undoes. Enterprise apps have
hundreds of these, and the same command can come from a shortcut, a menu item or a toolbar
button. Flutter separates the three parts, and Tantu follows it:

- an **intent** is a value saying *what* to do, e.g. `struct Save;`;
- **`Shortcuts`** maps key combinations (**activators**) to intents;
- **`Actions`** maps intent types to the code that does them.

Both are resolved along the focused path, so the innermost binding wins: a text field's
Ctrl+A can select its text while the window's Ctrl+A selects all items. Menus and toolbars
(Phase 4) invoke intents the same way, through `ViewTree::invoke`. Together, intents and
actions are the command registry the PLAN.md item asks for.

## Scope

In scope:

- `SingleActivator`: a key plus exact modifiers, with `primary()` for Ctrl or Cmd per OS.
- `Shortcuts` and `Actions` wrapper views, and the `BuildCx::shortcut` and `BuildCx::action`
  calls behind them.
- Resolving shortcuts during `dispatch_key`, and `ViewTree::invoke` for intents from code.

Out of scope (and where it goes):

- Key sequences (chords like Ctrl+K, Ctrl+C), `LogicalKeySet`, character activators that ignore
  modifiers: later, if needed.
- Enabled/disabled actions and listing commands for menus and a command palette: with menus
  (Phase 4). For now an action is always enabled.
- Default text-editing shortcuts: with `TextField`.

## Public API

```rust
/// A key with exact modifiers (Flutter's `SingleActivator`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SingleActivator { /* key, control, shift, alt, super_key */ }

impl SingleActivator {
    /// `key` with no modifiers.
    pub fn new(key: LogicalKey) -> Self;
    /// `SingleActivator::new(LogicalKey::Character(c))`.
    pub fn character(c: &str) -> Self;
    pub fn control(self) -> Self;
    pub fn shift(self) -> Self;
    pub fn alt(self) -> Self;
    pub fn super_key(self) -> Self;
    /// The platform's primary modifier: Command (`super_key`) on macOS, Control elsewhere.
    pub fn primary(self) -> Self;
    /// Whether a key press matches.
    pub fn accepts(&self, event: &KeyEvent) -> bool;
}

/// Binds activators to intents for key events on the focused path through its child.
pub struct Shortcuts<V> { /* child, bindings */ }
impl<V: View> Shortcuts<V> {
    pub fn new(child: V) -> Self;
    /// When `activator` matches, the intent `intent` is invoked.
    pub fn bind<I: Any + Clone>(self, activator: SingleActivator, intent: I) -> Self;
}

/// Handles intents of given types for focus inside its child (and `invoke` from it).
pub struct Actions<V> { /* child, handlers */ }
impl<V: View> Actions<V> {
    pub fn new(child: V) -> Self;
    /// Runs `handler` for intents of type `I`.
    pub fn on<I: Any>(self, handler: impl Fn(&I) + 'static) -> Self;
}

impl BuildCx<'_> {
    /// Binds `activator` to `intent` on `element` (what `Shortcuts` uses).
    pub fn shortcut<I: Any + Clone>(&mut self, element: ElementId, activator: SingleActivator, intent: I);
    /// Handles intents of type `I` on `element` (what `Actions` uses).
    pub fn action<I: Any>(&mut self, element: ElementId, handler: impl Fn(&I) + 'static);
}

impl ViewTree {
    /// Invokes `intent` from the focused element (the root when nothing is focused): the
    /// nearest action for `I` on the path up runs. Returns whether one did.
    pub fn invoke<I: Any>(&mut self, intent: &I) -> bool;
}
```

## Behavior

- **VIEW-SHORT-01:** `accepts` is true for a press (including repeats) of the activator's key
  with exactly its modifiers. Releases don't match. Character keys compare without case, so
  Shift+"A" matches `character("a").shift()`, and `primary()` means `super_key` on macOS and
  `control` elsewhere.
- **VIEW-SHORT-02:** During `dispatch_key`, in the bubble phase, an element with a binding whose
  activator accepts the press invokes its intent, as `invoke` does. If an action ran, the
  dispatch stops there: later bubble handlers don't run, and `dispatch_key` returns true.
  Capture handlers still run first, so they can intercept keys before any shortcut.
- **VIEW-SHORT-03:** A binding whose intent has no action on the path does nothing, and the
  dispatch goes on as if there were no binding: a Tab binding without an action still moves
  focus.
- **VIEW-SHORT-04:** The innermost binding wins. When `Shortcuts` near the focus and `Shortcuts`
  at the root both bind the same activator, only the inner one's intent is tried. If it finds
  no action, the outer binding is tried next (VIEW-SHORT-03).
- **VIEW-SHORT-05:** `invoke` searches from the focused element (or the root) up for an action
  for `I`. The nearest one runs, with the tree's runtime current, and receives the intent.
  `invoke` returns false when none is found.
- **VIEW-SHORT-06:** Removing an element drops its bindings and actions.

## Open questions

Resolved (2026-10-10; the user said "continue" on the draft):

1. **Flutter's three-part model:** intents, `Shortcuts`, `Actions`.
2. **Exact-modifier matching, with characters compared without case.**
3. **Actions are always enabled** until menus (Phase 4).
4. **Bindings match on the focused path;** window-wide shortcuts go at the top of the app.
