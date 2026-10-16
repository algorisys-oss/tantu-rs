# 0016. One reactive runtime per app, shared by its windows

- **Status:** Accepted (2026-10-16; the user said "continue" on the draft)
- **Date:** 2026-10-16
- **Supersedes:** the "one runtime per window" part of [ADR 0011](0011-view-layer.md) and
  decision 20 ("an explicit `Runtime` per app/window") in HANDOFF.md
- **Related:** [ADR 0007](0007-using-disposed-reactive-handles.md),
  [multi-window spec](../specs/facade/windows.md), [timers](../specs/reactive/timers.md)

## Context

Every `ViewTree` creates its own `Runtime` (ADR 0011). A handle never resolves in another
runtime (REACTIVE-SIG), so a signal created in one window reads as disposed in another.
That is fine while an app has one window, but Phase 4's multi-window item needs windows
that share state: an inspector window following the main window's selection, a second
document window editing the same model, and later the dock's floating panels, whose
content moves between windows while keeping its signals.

The runner (`AppHandler`) already keeps a list of windows, and both platform shells can
create windows at any point of the run, not only at start. What is missing is shared
reactive state, a way for views to open and close windows, and state owned by the app
rather than by any one window.

## Decision

1. **One `Runtime` per app.** `ViewTree` gains a constructor taking an `Rc<Runtime>`; the
   runner creates one runtime and passes it to every window's tree. `ViewTree::new` keeps
   creating a private runtime, so tests and single-window embedders don't change.
2. **Ownership stays per element.** Each window's root scope is owned by the runtime's top
   level, and closing a window disposes that scope (as today). What a window creates dies
   with the window, so sharing across windows goes through state owned by the app (3).
3. **An app scope.** The runner creates a scope that lives as long as the app. App state is
   built in it (`App::provide`), and windows opened at run time build their content in
   their own root scope, not in the scope of the view that opened them.
4. **Timers tick once per app frame round.** With one runtime, a ticker would run once per
   window redraw if each tree ticked it. The runner ticks the runtime once when it is due
   (in `idle` and before a redraw round) and then redraws only the windows whose trees need
   a frame.
5. **Windows stay single-threaded.** All windows run on the platform's main thread (winit's
   model); a shared runtime adds no locking.

## Consequences

- Signals, memos and effects can cross windows when they are owned by the app scope. A
  signal owned by a window and read from another after the first closes panics, as any
  disposed handle does (ADR 0007); the spec's API steers app state into the app scope.
- `ViewTree::runtime()` may be shared, so code that assumed "the tree's runtime is mine"
  (the tester, drop) must not dispose the runtime itself. Dropping a tree disposes its root
  scope and its own runtime-level nodes (the drag session's signals), not the runtime.
- One hot window's ticker no longer redraws the others: frames follow each tree's
  `needs_frame`, and the platform wake-up is the runtime's single `next_wake`.
- Dock floating panels become possible later: a panel's content could move to another
  window's tree only if element trees can migrate, which this ADR does not provide;
  floating panels would rebuild their content in the new window from app-owned state.
