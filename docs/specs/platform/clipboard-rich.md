# Clipboard: images, HTML and the primary selection

- **Status:** Agreed (2026-10-16; the user said "continue" on the six proposals)
- **Crates:** `tantu-platform` (trait, fake), `tantu-platform-winit` (arboard),
  `tantu-view` (trait, memory clipboard), `tantu` (runner bridge), `tantu-widgets`
  (`TextField`), `tantu-test`, `examples/gallery`
- **Plan item:** Phase 4, "Multi-window, drag & drop, clipboard", part "rich text and
  image clipboard, primary selection"
- **Related:** [plain-text clipboard](clipboard.md) (part 1), [text field](../widgets/text-field.md),
  [image](../widgets/image.md)

## Purpose

Part 1 copies plain text. Desktop apps also copy and paste images (a screenshot into an
editor, a chart out of a dashboard), put formatted text on the clipboard so other apps
keep the styling, and on Linux use the *primary selection*: selecting text makes it
available to a middle click elsewhere, without Ctrl+C.

## No new dependency

HANDOFF listed the image clipboard as waiting for a decision because arboard's
`image-data` feature pulls in the `image` crate. Checking the build shows it already
does: decision 90 kept arboard's default features, `image-data` is a default, and
`image` 0.25 is in `Cargo.lock` through `tantu-platform-winit` today. This part adds no
dependency; it uses what is already compiled. Primary selection is arboard's
`LinuxClipboardKind::Primary` (X11 and Wayland through XWayland), already available.

## Scope

In scope: reading and writing RGBA images; writing HTML with a plain-text alternative,
and reading HTML; reading and writing the primary selection on Linux; `TextField`
publishing its selection to the primary selection and pasting it on a middle click; the
fake platform, the tester and a gallery card.

Out of scope: rich text *in* Tantu's own fields (a `TextField` is plain text; it pastes
the plain-text form), file lists on the clipboard, image formats other than RGBA8
(encoded PNG/JPEG bytes), clipboard change notifications, and Wayland's native data
control protocol (arboard's `wayland-data-control` feature; XWayland covers it).

## Public API

```rust
// tantu-platform (depends on core only, so it has its own image type)
/// A clipboard image: straight-alpha RGBA8 rows, top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipboardImage { pub width: u32, pub height: u32, pub rgba: Vec<u8> }

pub trait Clipboard {
    fn read_text(&self) -> Option<String>;
    fn write_text(&self, text: &str) -> bool;
    /// The clipboard's image, if it holds one. Default: none.
    fn read_image(&self) -> Option<ClipboardImage> { None }
    /// Puts an image on the clipboard. Default: unsupported (false).
    fn write_image(&self, image: &ClipboardImage) -> bool { false }
    /// Puts HTML on the clipboard, with `alt_text` for apps that take plain text.
    /// Default: writes `alt_text` only.
    fn write_html(&self, html: &str, alt_text: &str) -> bool { self.write_text(alt_text) }
    /// The clipboard's HTML, if it holds any. Default: none.
    fn read_html(&self) -> Option<String> { None }
    /// The primary selection (X11/Wayland); `None` where there is none.
    fn read_primary(&self) -> Option<String> { None }
    /// Sets the primary selection; false where there is none.
    fn write_primary(&self, text: &str) -> bool { false }
}

// tantu-view: the same methods on its own `Clipboard` trait, with `ImageData`
// (tantu-scene's straight-alpha RGBA8 image) instead of `ClipboardImage`;
// `MemoryClipboard` keeps all four kinds (text, image, HTML, primary) in memory.

// tantu-test
impl WidgetTester {
    pub fn clipboard_image(&self) -> Option<ImageData>;
    pub fn set_clipboard_image(&mut self, image: ImageData);
    pub fn primary_selection(&self) -> Option<String>;
    pub fn set_primary_selection(&mut self, text: &str);
}
```

## Behavior

- **PLATFORM-CLIP-02:** The trait's new methods have defaults (no image, no HTML, no
  primary selection; `write_html` falls back to plain text), so existing implementations
  keep compiling and behave as before.
- **PLATFORM-FAKE-17:** The fake platform's clipboard keeps an image, HTML and a primary
  selection in memory beside the text; writing HTML also sets the text to `alt_text`;
  `FakeLog` reports the final image, HTML and primary selection.
- **PLATFORM-WINIT-19:** The winit shell maps the methods to arboard (`get_image`/
  `set_image`, `set_html` with the alternative, `get().html()`, and on Linux the
  `Primary` clipboard kind); elsewhere the primary methods keep their defaults. Failures
  are logged with `tracing` and reported as `None`/`false`.
- **VIEW-CLIP-02:** The view's `Clipboard` trait gains the same methods with `ImageData`;
  `BuildCx::clipboard` handles forward them to the tree's current clipboard (as
  VIEW-CLIP-01); `MemoryClipboard` keeps all four, with `write_html` setting the text too.
- **FACADE-APP-20:** The runner's bridge converts between `ImageData` and
  `ClipboardImage` and forwards the other methods.
- **WIDGETS-TEXTFIELD-13:** When a text field's selection becomes non-empty through the
  user (pointer, keys or select-all), it writes the selected text to the primary
  selection (a no-op without one). A middle click in the field pastes the primary
  selection at the clicked position (one undo step, through the field's formatters, as a
  paste), moving the caret there; without a primary selection it does nothing.
- **TEST-WT-21:** The tester's memory clipboard exposes and sets the image and primary
  selection.
- **GALLERY-55:** A "clipboard" kitchen-sink card (after "files"; 29 cards) has "Copy
  image" (the gallery's generated gradient, 64 × 64) and "Paste image" (shows the
  clipboard image below, scaled to at most 96 px tall) and "Copy as HTML" (a bold
  "Tantu" with the plain text "Tantu"); its echo `sink-clipboard-state` reports the last
  action ("Copied image 64 × 64", "Pasted image 64 × 64", "No image on the clipboard",
  "Copied HTML").

## Performance and allocation

Images are copied once per clipboard call (RGBA buffers of the image's size).

## Open questions

Proposals from the agent, for review:

1. **Use the `image` crate already compiled through arboard's defaults**, adding no new
   dependency and no new feature; Tantu's own crates never use `image` directly.
2. **A platform-side `ClipboardImage`** (the platform crate depends on core only), with
   the runner converting to `ImageData`, as `CursorIcon` is converted.
3. **Default trait methods**, so third-party platforms and clipboards keep compiling.
4. **Primary selection only on Linux**, written by text fields on user selection and
   pasted on middle click (the GTK/Qt convention); other platforms get the defaults.
5. **HTML is write-mostly**: apps write HTML with a plain-text alternative, and
   `TextField` pastes plain text; reading HTML is offered for apps that want it.
6. **No encoded image bytes** on the clipboard API (decoding stays in the app or
   `decode_png`).
