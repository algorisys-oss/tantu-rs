# Image

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-11; review
  the Open questions)
- **Crates:** `tantu-widgets` (plus `Resources::intern_image` in `tantu-scene`,
  `TextPainter::image_id` / `PaintCx::image_id` in `tantu-view`; sizing in
  [RenderImage](../layout/image.md))
- **Plan item:** Phase 3, "Widgets" → "`Image` (decoding, fit, sizing)"
- **Related:** [Scene](../scene/scene.md) (`ImageDraw`), [renderer and resources](../scene/renderer.md),
  [text in views](../view/text.md), [RenderImage](../layout/image.md), Flutter's `Image`,
  `BoxFit` and `paintImage`

## Purpose

Shows a decoded raster image in a box, sized by the layout rules of `RenderImage` and placed
in that box by a `BoxFit` and an `Alignment`, exactly as Flutter's `paintImage`. Decoding
PNGs is provided (`decode_png`), since every app needs at least that; other formats and
loading from files or the network come later.

An image must reach the renderer's `Resources`, which a widget can't see. The frame's painter
can: like fonts (which the text system adds to the resources on first use), `PaintCx::image_id`
interns the image's data in the resources and returns its handle, once per pixel buffer.

## Scope

In scope:

- `decode_png(bytes) -> Result<ImageData, ImageError>`.
- `Image::new(data)`, `.width`, `.height`, `.size`, `.fit`, `.alignment`, `.opacity`.
- `BoxFit` (Flutter's seven), `Resources::intern_image`, `TextPainter::image_id`,
  `PaintCx::image_id`.

Out of scope (and where it goes):

- JPEG, WebP, SVG; loading from files/URLs, placeholders and error builders, caching and
  eviction (`Resources::remove_image` when the last `Image` goes): an asset-loading item,
  later in Phase 3 or 4. Until then an interned image stays in the resources.
- `color`/blend modes, `repeat`, `center_slice` (nine-patch), filter quality choices: later.
- DPI-aware image variants (`2.0x`): with assets.

## Public API

```rust
// tantu-widgets
/// Decodes a PNG into straight-alpha RGBA8 (any color type and bit depth).
pub fn decode_png(bytes: &[u8]) -> Result<ImageData, ImageError>;

/// Why an image couldn't be decoded.
#[derive(Debug)]
pub struct ImageError(/* message */);
impl std::fmt::Display for ImageError {}
impl std::error::Error for ImageError {}

/// How an image is inscribed in its box (Flutter's `BoxFit`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoxFit { Fill, Contain, Cover, FitWidth, FitHeight, None, #[default] ScaleDown }

/// Shows a decoded image (Flutter's `Image`).
pub struct Image { /* data, width, height, fit, alignment, opacity */ }

impl Image {
    /// `data` at its natural size, `BoxFit::ScaleDown`, centered, opaque.
    pub fn new(data: ImageData) -> Self;
    pub fn width(self, width: f32) -> Self;
    pub fn height(self, height: f32) -> Self;
    /// Both.
    pub fn size(self, width: f32, height: f32) -> Self;
    pub fn fit(self, fit: BoxFit) -> Self;
    pub fn alignment(self, alignment: Alignment) -> Self;
    /// Multiplies the image's alpha (reactive; default 1).
    pub fn opacity(self, opacity: impl IntoProp<f32>) -> Self;
}

// tantu-scene
impl Resources {
    /// The handle of `image`'s pixel buffer: the id it was added with before (same `Arc`
    /// buffer, width and height), or a new one from `add_image`.
    pub fn intern_image(&mut self, image: &ImageData) -> ImageId;
}

// tantu-view
pub trait TextPainter {
    // ...
    /// Interns `image` in the frame's resources and returns its handle; `None` when the
    /// painter has no resources (`NoTextMeasure`). Default `None`.
    fn image_id(&mut self, image: &ImageData) -> Option<ImageId> { None }
}
impl PaintCx<'_, '_> {
    /// `TextPainter::image_id` on this pass's painter.
    pub fn image_id(&mut self, image: &ImageData) -> Option<ImageId>;
}
```

`Image` is one render element with a `RenderImage` (natural = the data's pixel size,
`width`, `height`) and an `ImagePaint` holding the data, fit, alignment and opacity.

Fitting (Flutter's `applyBoxFit`, with input `i` = the natural size and output `o` = the
element's size, giving a source size `s` in image pixels and a destination size `d`):

| Fit | `s` | `d` |
|---|---|---|
| `Fill` | `i` | `o` |
| `Contain` | `i` | `o.w / i.w ≤ o.h / i.h` ? `(o.w, i.h · o.w / i.w)` : `(i.w · o.h / i.h, o.h)` |
| `Cover` | `o.w / o.h > i.w / i.h` ? `(i.w, i.w · o.h / o.w)` : `(i.h · o.w / o.h, i.h)` | `o` |
| `FitWidth` | as `Cover` when `o.w / o.h > i.w / i.h`, else as `Contain` | likewise |
| `FitHeight` | as `Cover` when `o.w / o.h ≤ i.w / i.h`, else as `Contain` | likewise |
| `None` | `(min(i.w, o.w), min(i.h, o.h))` | `s` |
| `ScaleDown` | `i` | `i`, then shrunk to fit `o` keeping the ratio (`Contain` when larger) |

The source rect is `s` inscribed in `(0, 0, i.w, i.h)` by the alignment, and the destination
rect is `d` inscribed in the element's bounds by it (`origin + alignment.along_offset(box −
size)`). The command is `ImageDraw { image, src: Some(source) (None when it is the whole
image), dest, sampling: Linear, opacity }`.

## Behavior

- **SCENE-RES-11:** `intern_image` returns the same id for images sharing a pixel buffer
  (clones, or `rgba8` from the same `Arc`) and size, adding the image and bumping the revision
  only the first time. An image removed with `remove_image` is added again when interned next.
  Equal pixels in different buffers are different images. (Added to
  [renderer and resources](../scene/renderer.md).)
- **VIEW-TEXT-06:** `NoTextMeasure::image_id` is `None`; `SystemText::image_id` interns in its
  resources and returns `Some`; `PaintCx::image_id` forwards to the pass's painter. (Added to
  [text in views](../view/text.md).)
- **WIDGETS-IMAGE-01:** `decode_png` decodes RGBA, RGB (alpha 255), grayscale and
  grayscale-alpha PNGs, 8-bit or normalized to it, into an `ImageData` of the PNG's size with
  straight alpha. Bytes that aren't a PNG give an error.
- **WIDGETS-IMAGE-02:** An `Image` builds one render element with a `RenderImage` of the
  data's size and the explicit width/height, so it sizes by LAYOUT-IMAGE-01 (a 60 × 40 image in
  a column is 60 × 40; `.width(120.0)` makes it 120 × 80; `.size(50.0, 50.0)` makes it
  50 × 50).
- **WIDGETS-IMAGE-03:** It paints one `ImageDraw` with the source and destination from the
  fitting table (for a 60 × 40 image in a 120 × 120 box, centered: `Contain` dest
  (0, 20)–(120, 100) and no source rect; `Cover` source (10, 0)–(50, 40) and the whole box;
  `Fill` the whole image into the whole box; `FitWidth` as `Contain` here; `FitHeight` as
  `Cover` here; `None` source (0, 0)–(60, 40) at (30, 40)–(90, 80); `ScaleDown` the whole image
  at (30, 40)–(90, 80)), with `Linear` sampling. Checked against a golden
  (`crates/tantu-widgets/tests/goldens/image.png`).
- **WIDGETS-IMAGE-04:** `alignment` places both rects (`TOP_LEFT` puts `Contain`'s destination
  at (0, 0)–(120, 80)); `opacity` is reactive and carried in the command.
- **WIDGETS-IMAGE-05:** Two `Image`s showing clones of one `ImageData` draw the same image id.
  With a painter that has no resources, nothing is painted.

## Open questions

Decided by the agent while the user was away (review):

1. **PNG decoding in `tantu-widgets`** with the `png` crate (already used by three render
   crates), as `decode_png`; the software renderer keeps its own copy for goldens. Moving both
   into `tantu-scene` behind a feature would remove the duplicate.
2. **Images interned through the frame's painter** (`TextPainter::image_id`): the painter is
   what a paint pass has that owns resources. The trait name now understates it; renaming it
   `FramePainter` is a later cleanup.
3. **No eviction yet**: an interned image stays until the app ends.
4. **`Image::new` takes decoded data** (`ImageData`), not bytes: decoding can fail, and a
   builder shouldn't.
