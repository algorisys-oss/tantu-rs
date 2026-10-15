# Clipboard, part 1: plain text and TextField copy/paste

- **Status:** Agreed (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-platform` (`Clipboard`, fake clipboard), `tantu-platform-winit`
  (`arboard`), `tantu-view` (the view-side clipboard), `tantu` (runner), `tantu-test`,
  `tantu-widgets` (`TextField` copy, cut and paste), `examples/gallery`
- **Plan item:** Phase 4, "Multi-window, drag & drop (intra-app + OS), clipboard (text, rich,
  images)", part 1 (plain-text clipboard)
- **Related:** [platform](platform.md), [winit shell](../platform-winit/shell.md),
  [cursor icons](cursor.md) (the same platform-to-view bridging),
  [TextField](../widgets/text-field.md), [shortcuts](../view/shortcuts.md),
  [WidgetTester](../test/widget-tester.md), HANDOFF open question "Clipboard for TextField"

## Purpose

Every text field needs copy, cut and paste, and apps need to put text on the clipboard
("Copy link"). This part adds a plain-text clipboard through the platform trait (the OS
clipboard in the winit shell, an in-memory one in the fake platform and in tests), makes it
reachable from views, and binds the usual chords in `TextField`.

## Scope

In scope: reading and writing plain text; the platform, view and tester sides; the runner
connecting them; `TextField` copy/cut/paste with rebindable intents; a gallery test.

Out of scope (and where it goes): rich text and images, the X11/Wayland primary selection
(middle-click paste), clipboard change notifications, drag and drop, multi-window (later
parts of this item); context-menu Cut/Copy/Paste on text fields (later, with the menus).

## Public API

```rust
// tantu-platform
/// The OS clipboard (plain text).
pub trait Clipboard {
    /// The clipboard's text, or `None` when it is empty, holds no text or can't be read.
    fn read_text(&self) -> Option<String>;
    /// Replaces the clipboard's content with `text`; false when it couldn't be written.
    fn write_text(&self, text: &str) -> bool;
}
pub trait PlatformContext { /* existing, */
    /// The clipboard, or `None` on a platform without one.
    fn clipboard(&mut self) -> Option<Rc<dyn Clipboard>>;
}
impl FakePlatform { pub fn clipboard_text(self, text: impl Into<String>) -> Self; }
pub struct FakeLog { /* existing, */ pub clipboard_text: Option<String> }

// tantu-view (the view layer doesn't see the platform crate; the runner bridges)
pub trait Clipboard {
    fn read_text(&self) -> Option<String>;
    fn write_text(&self, text: &str) -> bool;
}
/// An in-memory clipboard: every view tree starts with one.
#[derive(Default)]
pub struct MemoryClipboard { /* private */ }
impl Clipboard for MemoryClipboard;
impl ViewTree { pub fn set_clipboard(&mut self, clipboard: Rc<dyn Clipboard>); }
impl BuildCx<'_> { pub fn clipboard(&self) -> Rc<dyn Clipboard>; }

// tantu-test
impl WidgetTester {
    pub fn clipboard_text(&self) -> Option<String>;
    pub fn set_clipboard_text(&mut self, text: &str);
}

// tantu-widgets
pub struct CopyIntent;  // primary + C
pub struct CutIntent;   // primary + X
pub struct PasteIntent; // primary + V
```

## Behavior

- **PLATFORM-CLIP-01:** `PlatformContext::clipboard` returns the same clipboard handle for
  every call; `write_text` replaces the content (an empty string empties it) and
  `read_text` returns what was written, `None` when empty or holding no text.
- **PLATFORM-FAKE-15:** The fake platform's clipboard is in memory, starts with
  `FakePlatform::clipboard_text` (empty without it) and its final text is in
  `FakeLog::clipboard_text`.
- **PLATFORM-WINIT-17:** The winit shell's clipboard uses `arboard` (created on first use);
  a failure to open, read or write returns `None`/false with a debug `tracing` message,
  never a panic. Checked by the opt-in window test (a write then a read round-trips).
- **VIEW-CLIP-01:** Every `ViewTree` starts with its own `MemoryClipboard`;
  `set_clipboard` replaces it for the tree, and `BuildCx::clipboard` returns the current
  one (handlers keep the handle they took at build time, so `set_clipboard` is meant to run
  before the views build).
- **FACADE-APP-18:** The runner gives each window's tree the platform's clipboard (through
  an adapter) before building its view, when the platform has one.
- **TEST-WT-19:** The tester's tree keeps a `MemoryClipboard`; `clipboard_text` reads it and
  `set_clipboard_text` writes it.
- **WIDGETS-TEXTFIELD-12:** In an enabled `TextField`, primary + C (`CopyIntent`) writes the
  selected text to the clipboard (nothing when the selection is collapsed); primary + X
  (`CutIntent`) copies it and deletes it as one undo step; primary + V (`PasteIntent`)
  replaces the selection with the clipboard's text as one undo step, line breaks becoming
  spaces in a single-line field, nothing happening when the clipboard holds no text.
  `on_change` is called when the text changes. The chords go through the shortcut
  machinery, so apps can rebind them; a disabled field ignores them.
- **GALLERY-35:** A gallery test copies text from the Text field section's field and pastes
  it back (select all, copy, move to the end, paste: the text appears twice).

## Performance and allocation

None beyond the general rules; the OS clipboard is touched only on a chord.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **`arboard` in `tantu-platform-winit`** (the clipboard crate egui and others use; it
   needs no winit types, and only `tantu-platform-*` touches OS APIs). Default features: X11
   and macOS/Windows; on Wayland it goes through XWayland (the `wayland-data-control`
   feature can come later).
2. **A view-side `Clipboard` trait** mirroring the platform's, bridged by the runner, as with
   cursor icons, so `tantu-view` keeps its dependency edges.
3. **Synchronous reads** at the moment of the chord (no async clipboard API).
4. **Copy/cut/paste are intents** next to undo/redo, rebindable by apps.
