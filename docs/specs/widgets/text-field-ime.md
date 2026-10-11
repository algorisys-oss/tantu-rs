# TextField: IME composition

- **Status:** Agreed (the user said "continue" on the draft, taking its proposals)
- **Crates:** `tantu-platform` (IME events and context calls), `tantu-platform-winit`
  (conversion), `tantu-view` (dispatch to the focused element, the IME area), `tantu`
  (the runner), `tantu-widgets` (`EditingValue` composing edits, `TextField`), `tantu-test`
- **Plan item:** Phase 3, "Widgets" → "`TextField`, in parts" → "IME composition"
- **Related:** [text editing model](text-editing.md) (`composing`), [TextField](text-field.md),
  [platform](../platform/platform.md), [winit shell](../platform-winit/shell.md),
  [app runner](../facade/app.md), [focus](../view/focus.md); winit's `Ime` event

## Purpose

Input methods (CJK, dead keys on some platforms, emoji pickers) compose text before committing
it: the application shows the *preedit* text in place, with the IME's own cursor, and replaces
it when the IME commits or clears. The platform tells the IME where the caret is so its
candidate window can follow. This part threads winit's IME events through the platform trait,
the runner and the view tree to the focused `TextField`, which edits its value's composing
region, and reports the caret area back.

## Scope

In scope:

- `ImeEvent` and `WindowEvent::Ime`; `PlatformContext::set_ime_allowed` and
  `set_ime_cursor_area`; the winit conversion and calls; `FakePlatform` scripting IME events.
- `BuildCx::on_ime`, `ViewTree::dispatch_ime` (the focused element only), `BuildCx::ime_area`
  and `ViewTree::ime_area` (window coordinates).
- The runner forwarding IME events and keeping the platform's IME state in step with the
  focused element's area (allowed while an element declares an area).
- `EditingValue::compose(text, cursor)` and `commit(text)`; `TextField` handling the events
  and painting the composing range with an underline.
- `WidgetTester::ime(event)`.

Out of scope: candidate windows drawn by Tantu, inline predictions, Android/iOS keyboards
(their shells), keys swallowed by the IME (the platform doesn't deliver them).

## Public API

```rust
// tantu-platform
/// An input-method event (winit's shape; offsets are byte offsets into `text`).
pub enum ImeEvent {
    /// The IME became active for the window (after `set_ime_allowed(true)`).
    Enabled,
    /// The text being composed, and the IME's cursor range within it (`None`: hidden).
    Preedit { text: String, cursor: Option<(usize, usize)> },
    /// The composed text is final; a `Preedit` with empty text follows or preceded it.
    Commit(String),
    /// The IME was deactivated.
    Disabled,
}
pub enum WindowEvent { /* ... */ Ime(ImeEvent) }

pub trait PlatformContext {
    // ...
    /// Lets the window receive IME events (off by default).
    fn set_ime_allowed(&mut self, window: WindowId, allowed: bool);
    /// Where the caret is, in logical pixels, so the IME can place its candidate window.
    fn set_ime_cursor_area(&mut self, window: WindowId, origin: Point, size: Size);
}

// tantu-view
pub struct ImeCx<'a> { pub event: &'a ImeEvent, pub element: ElementId }
impl BuildCx<'_> {
    /// Calls `handler` for IME events while `element` has focus.
    pub fn on_ime(&mut self, element: ElementId, handler: impl Fn(&ImeCx<'_>) + 'static);
}
impl BuildCx<'_> {
    /// Registers how `element` reports its caret area, in its own coordinates, while it wants
    /// IME input (`None`: it doesn't). Asked through `ViewTree::ime_area` while focused, so
    /// it can read paint-time data such as the text geometry.
    pub fn ime_area(&mut self, element: ElementId, provider: impl Fn() -> Option<Rect> + 'static);
}
impl ViewTree {
    /// Delivers `event` to the focused element's IME handlers; returns whether any ran.
    pub fn dispatch_ime(&mut self, event: ImeEvent) -> bool;
    /// The focused element's IME area in window coordinates, if it declared one.
    pub fn ime_area(&self) -> Option<Rect>;
}

// tantu-widgets
impl EditingValue {
    /// Replaces the composing region (or the selection, when there is none) with `text`,
    /// marks it as composing, and puts the caret at `cursor` within it (its end when `None`).
    pub fn compose(&self, text: &str, cursor: Option<(usize, usize)>) -> Self;
    /// Replaces the composing region (or the selection) with `text` and clears it.
    pub fn commit(&self, text: &str) -> Self;
}

// tantu-test
impl WidgetTester {
    /// Dispatches an IME event to the focused element, then pumps.
    pub fn ime(&mut self, event: ImeEvent);
}
```

## Behavior

- **PLATFORM-IME-01:** `ImeEvent` and `WindowEvent::Ime` exist as above, and
  `PlatformContext` has `set_ime_allowed` and `set_ime_cursor_area`. (In
  [platform](../platform/platform.md).)
- **PLATFORM-FAKE-11:** `FakePlatform` delivers scripted `Ime` events like other window events
  and records `set_ime_allowed` and `set_ime_cursor_area` per window (`FakeWindow::ime_allowed`,
  `ime_area`).
- **PLATFORM-WINIT-13:** winit's `Ime::Enabled/Preedit/Commit/Disabled` convert one to one;
  `set_ime_allowed` and `set_ime_cursor_area` call the winit window (the area in physical
  pixels, from the window's scale factor).
- **VIEW-IME-01:** `dispatch_ime` calls the handlers registered with `on_ime` on the focused
  element (not its ancestors); with nothing focused, or no handlers, it returns false.
  Removing the element drops its handlers and area. (In [focus](../view/focus.md).)
- **VIEW-IME-02:** `ime_area` asks the focused element's provider and offsets the area to
  window coordinates by the element's layout position; `None` when the focused element has
  no provider or it returns `None`, nothing is focused, or focus moved to another element.
- **FACADE-APP-12:** The runner forwards `WindowEvent::Ime` to the window's tree. After each
  frame it compares `ime_area()` with what it last told the platform: a new area is sent with
  `set_ime_cursor_area` (and `set_ime_allowed(true)` the first time); `None` after `Some`
  sends `set_ime_allowed(false)`.
- **WIDGETS-EDIT-09:** `compose` and `commit` as documented; `compose("")` clears the region
  (the IME cancelled); composing text counts as one undo step with the commit.
- **WIDGETS-TEXTFIELD-07:** A focused field handles `Preedit` with `compose`, `Commit` with
  `commit`, and `Disabled` by committing the current composing text; it declares its caret
  rect (plus the padding) as its IME area while focused and enabled, and `None` otherwise.
- **WIDGETS-TEXTFIELD-08:** The composing range is painted with a 1 px underline in primary
  (the first text decoration), and the caret sits at the IME's cursor.
- **TEST-WT-10:** `ime` dispatches to the tester's tree and pumps.

## Open questions

Resolved (2026-10-12; the user said "continue" on the draft):

1. **winit's event shape as the platform's** (`Preedit { text, cursor }`, `Commit`, byte
   offsets), as for keys and pointers.
2. **IME allowed only while a focused element declares an area**, so plain apps never
   enable the IME and buttons don't get composition events.
3. **The composing underline as a Scene rect** (1 px), since there are no text decorations
   yet; proper underline/strikethrough spans can replace it later.
4. **Composition and its commit are one undo step** (the model records the value before the
   first `Preedit`).
5. **`Disabled` commits** the pending preedit rather than dropping it (what most toolkits
   do).
