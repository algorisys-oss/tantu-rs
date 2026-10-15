# Cursor icons through the platform

- **Status:** Draft
- **Crates:** `tantu-platform`, `tantu-platform-winit`, `tantu-view`, `tantu` (runner),
  `tantu-test`
- **Plan item:** Phase 4, "`SplitView`", part 1 (cursor icons)
- **Related:** [platform](platform.md), [winit shell](../platform-winit/shell.md),
  [events](../view/events.md), [facade](../facade/app.md),
  [WidgetTester](../test/widget-tester.md), [SplitView](../widgets/split-view.md)

## Purpose

Elements already ask for cursor shapes (`BuildCx::set_cursor`, VIEW-EVENT-05) and the view
tree already works out which one applies (`ViewTree::cursor`), but the `Platform` trait has
no way to show it, so every window keeps the default arrow. Text fields should show a caret,
buttons a hand, and the split view and the grid's column handles a resize cursor; without
it a splitter can't be discovered. This part carries the chosen cursor to the window.

## Scope

In scope: a platform-side cursor enum and `PlatformContext::set_cursor`; the fake
platform recording it; the winit shell mapping it to winit's cursors; the runner keeping
each window's cursor in step with its view tree; the cursor staying put while a drag holds
the pointer capture; `WidgetTester::cursor`.

Out of scope: custom (image) cursors, hiding or grabbing the cursor, cursor position
warping, animated cursors, diagonal resize shapes (added when something needs them).

## Public API

```rust
// tantu-platform
/// A cursor shape, independent of any windowing library. Mirrors `tantu_view::CursorIcon`
/// (the view crate does not depend on the platform; the runner converts).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CursorIcon {
    #[default]
    Default, Pointer, Text, Grab, Grabbing, NotAllowed, ResizeHorizontal, ResizeVertical,
}

pub trait PlatformContext {
    // existing methods …
    /// Shows `cursor` while the pointer is over `window`. Unknown windows are ignored.
    fn set_cursor(&mut self, window: WindowId, cursor: CursorIcon);
}

// tantu-platform, fake.rs
impl FakeWindow {
    /// Every cursor set on this window, in order (PLATFORM-FAKE-14).
    pub cursors: Vec<CursorIcon>,   // a public field, like `accessibility_updates`
}

// tantu-test
impl WidgetTester {
    /// The cursor the window would show for the pointer's last position (TEST-WT-17).
    pub fn cursor(&self) -> CursorIcon;
}
```

`set_cursor` is a required trait method; the two implementations in the workspace (fake
and winit) gain it. No new feature flags or dependencies.

## Behavior

- **PLATFORM-CURSOR-01:** `PlatformContext::set_cursor(window, cursor)` changes the shape
  shown while the pointer is over that window, from then on, until the next call. A new
  window starts with `Default`. Unknown or closed windows are ignored; setting the shape it
  already has is allowed and changes nothing.
- **PLATFORM-FAKE-14:** The fake platform appends every `set_cursor` call to the window's
  `FakeWindow::cursors`, in order, including repeats; unknown windows record nothing.
- **PLATFORM-WINIT-16:** The winit shell maps `Default`, `Pointer`, `Text`, `Grab`,
  `Grabbing` and `NotAllowed` to winit's cursors of the same names, `ResizeHorizontal` to
  `EwResize` and `ResizeVertical` to `NsResize` (the shapes every platform has), and calls
  `Window::set_cursor`. Checked by the opt-in window test.
- **VIEW-EVENT-09:** While a pointer capture is held (VIEW-EVENT-03), `ViewTree::cursor` is
  the cursor of the innermost element of the captured path that set one (else `Default`),
  wherever the pointer is, so a resize drag keeps its cursor when the pointer outruns the
  handle. After the matching `Up` it follows the hit test again.
- **FACADE-APP-17:** After dispatching each pointer event and after each frame, the runner
  converts the window's `ViewTree::cursor` to the platform's `CursorIcon` and calls
  `set_cursor` only when it differs from the last one it sent to that window (none sent
  counts as `Default`). A pointer leaving the window sends nothing.
- **TEST-WT-17:** `WidgetTester::cursor` returns `ViewTree::cursor` for the tester's last
  pointer position (`Default` before any pointer event); `hover`/`tap`/`drag` positions
  count.

## Performance and allocation

One hit test per pointer event and per frame, which dispatch already does; no allocation.
`set_cursor` reaches the platform only on a change.

## Open questions

Proposals, to accept or change:

1. **A platform-side copy of the enum** rather than moving `CursorIcon` into `tantu-core`
   or making `tantu-view` depend on `tantu-platform` (neither edge exists; the runner
   already converts events this way).
2. **`EwResize`/`NsResize`** rather than winit's `ColResize`/`RowResize`: the latter are
   missing or fall back on some platforms, and the split view and grid handles both mean
   "move this edge".
3. **Capture keeps the cursor** (VIEW-EVENT-09), a small amendment to VIEW-EVENT-05,
   matching every desktop toolkit's drag behavior.
4. **The runner sends only changes**, compared per window, after pointer events and frames
   (a frame can move a widget under a still pointer).
