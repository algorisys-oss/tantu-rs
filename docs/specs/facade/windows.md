# Multiple windows

- **Status:** Agreed (2026-10-16; the user said "continue" on the six proposals)
- **Crates:** `tantu-reactive` (shared runtime ticking), `tantu-view` (a tree on a given
  runtime), `tantu` (the runner, `App::provide`, `Windows`), `examples/gallery`
- **Plan item:** Phase 4, "Multi-window, drag & drop, clipboard", part "multi-window"
- **Related:** [ADR 0016](../../adr/0016-one-reactive-runtime-per-app.md),
  [app runner](app.md), [view tree](../view/tree.md), [timers](../reactive/timers.md)

## Purpose

Desktop apps open more than one window: a second document, an inspector or a preferences
window, all looking at the same state. Today `App::window` opens windows only at start,
and each window has its own reactive runtime, so they can't share signals. This spec lets
views open and close windows while the app runs, and lets windows share state owned by the
app.

## Scope

In scope: one runtime per app (ADR 0016); app-owned state provided to every window;
opening, closing and retitling windows from views; closing behavior and the app's end;
timers and frames with several windows; testing through `FakePlatform`; a gallery sample.

Out of scope: owned/modal child windows and their stacking (platform support varies; a
later part), window positions and restoring them, moving elements between windows (dock
floating panels rebuild from app state, later), multiple monitors and per-window themes
beyond what each window already provides, and `WidgetTester` with several windows (tests
drive the runner through `FakePlatform`).

## Public API

```rust
// tantu-view
impl ViewTree {
    /// Like `with_text_styles`, on `runtime` (shared with other trees). The tree's root
    /// scope is created at the runtime's top level; dropping the tree disposes it and the
    /// tree's own runtime-level nodes, never the runtime.
    pub fn with_runtime<V: View>(runtime: Rc<Runtime>, styles: TextStyles,
                                 app: impl FnOnce() -> V) -> Self;
}

// tantu (facade)
impl App {
    /// Builds a value once, in the app scope, before the first window opens, and provides
    /// it as an inherited value at the root of every window (`cx.inherit::<T>()`).
    /// Signals created in `init` live as long as the app.
    pub fn provide<T: Clone + 'static>(self, init: impl FnOnce() -> T + 'static) -> Self;
}

/// Opens and closes the app's windows from views. Provided at the root of every window
/// the runner opens. A `Copy` handle; requests take effect after the current event.
#[derive(Clone, Copy)]
pub struct Windows { /* signals */ }

impl Windows {
    /// The app's windows, or `None` outside a window opened by the runner (a
    /// `WidgetTester` tree, a bare `ViewTree`).
    pub fn of(cx: &BuildCx<'_>) -> Option<Windows>;
    /// Opens `window` showing `content`, built in the new window's own root scope.
    pub fn open<V: View>(self, window: Window, content: impl FnOnce() -> V + 'static)
        -> WindowHandle;
    /// The window this handle was taken from.
    pub fn current(self) -> WindowHandle;
    /// How many windows are open (a tracked read).
    pub fn count(self) -> usize;
}

/// One window of the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowHandle { /* id */ }

impl WindowHandle {
    /// Closes the window (as the user closing it, without the veto).
    pub fn close(self);
    /// Whether it is still open (a tracked read).
    pub fn is_open(self) -> bool;
    /// Changes its title.
    pub fn set_title(self, title: impl Into<String>);
}

impl Window {
    /// Called when the user asks to close the window; returning `false` keeps it open.
    pub fn on_close_request(self, f: impl FnMut() -> bool + 'static) -> Self;
}
```

## Behavior

- **FACADE-WIN-01:** The runner creates one reactive runtime for the app and builds every
  window's tree on it (`ViewTree::with_runtime`), so a signal readable in one window is
  readable in every other (ADR 0016). `ViewTree::new` still makes a private runtime.
- **FACADE-WIN-02:** `App::provide` runs its `init` once, in an app-level scope that lives
  until the run ends, before any window opens; each provided value is inherited at the root
  of every window (startup windows and windows opened later). Two `provide` calls of the
  same type: the later one wins.
- **FACADE-WIN-03:** `Windows::open` queues the window; after the current event (or frame)
  the runner creates the platform window, builds the content in the new window's own root
  scope (with the runtime current, the app-provided values and `Windows` inherited) and
  draws it. The returned handle's `is_open` turns true then; `count` follows.
- **FACADE-WIN-04:** Closing: the user's close request calls the window's
  `on_close_request` (default: allow). Allowed, or `WindowHandle::close`: the platform
  window closes, the tree is dropped (its root scope disposed), `is_open` turns false.
  When the last window closes the run ends. Closing a window never disposes app-scope
  state or another window's elements.
- **FACADE-WIN-05:** `WindowHandle::set_title` changes the platform window's title; a
  handle to a closed window ignores every call.
- **FACADE-WIN-06:** The runner ticks the app's runtime at most once between two `idle`
  calls: at `idle` when a timer is due or a ticker runs (then it asks for frames only for
  the trees that changed), otherwise before the first redraw after an `idle`. A ticker
  started in one window doesn't redraw the others; the platform wake-up is the runtime's
  next wake. (This replaces FACADE-APP-14's "each tree before its frame"; its single-window
  behavior is unchanged.)
- **FACADE-WIN-07:** Platform events go only to their window's tree; a signal written by
  one window's handler redraws every window showing it (and only those).
- **VIEW-TREE-13:** `ViewTree::with_runtime` builds on the given runtime; dropping the tree
  disposes its root scope and its own runtime-level nodes (the drag session's signals),
  leaving the runtime and other trees' nodes alive.
- **GALLERY-53:** `gallery::app()` builds the gallery's `App`, providing a `GalleryState`
  (the dark-mode flag and the current section, app-owned signals). The View menu has "New
  window", opening a 360 × 200 window titled "Gallery · second window" with a `Text` keyed
  `second-section` ("Section: Buttons") and a dark-mode switch keyed `second-dark`, bound to
  the same flag as the main window's: toggling it in either window changes both windows'
  themes. Without the runner (`WidgetTester`), the item is absent and the flag is local.

## Performance and allocation

Opening and closing windows allocates (a tree, a renderer). Per frame, nothing new: the
tick is one call per round instead of one per window.

## Open questions

Proposals from the agent, for review:

1. **`App::provide` for app state** (Flutter's provider-above-the-app pattern) instead of
   a global store or passing state into every content closure by hand. Values are plain
   `Clone` types, usually structs of `Copy` signal handles.
2. **A `Windows` handle inherited at every window's root**, rather than a global
   function: it keeps "no hidden global state" (PLAN.md, from Clay) and works in tests.
3. **Window content is built in the new window's root scope**, never in the opener's, so
   closing the opener doesn't take the new window's state with it.
4. **The last window closing ends the run** (today's behavior), not a "main window"
   concept. `on_close_request` gives apps the veto ("unsaved changes").
5. **No child/owned windows yet** (macOS sheets, Windows owner windows, always-on-top
   tool windows): winit supports some of it per platform; a later part with its own
   rules.
6. **Testing through `FakePlatform`** and the runner (`App::handler`), not a multi-window
   `WidgetTester`, which stays one tree. Facade tests check events, frames and windows
   in the fake log; the gallery test drives the second window through the runner.
