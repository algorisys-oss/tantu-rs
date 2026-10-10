# Software renderer (tiny-skia)

- **Status:** Implemented
- **Crate:** `tantu-render-soft`
- **Plan item:** Phase 1, "`tantu-render-soft`: tiny-skia renderer, PNG output, golden-image
  helpers"
- **Related:** [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md),
  [ADR 0006](../../adr/0006-wgpu-as-the-default-gpu-backend.md) (software fallback),
  [scene](../scene/scene.md) (drawing semantics 1–12), [renderer and
  resources](../scene/renderer.md) (renderer contract 1–9, `RenderReport::for_scene`),
  [headless](../render-headless/recorder.md)

## Purpose

A CPU `Renderer` that draws a Scene into pixels with [tiny-skia](https://github.com/linebender/tiny-skia).
It is the fallback where there is no usable GPU (VMs, RDP/Citrix, CI), and the reference
renderer for golden-image tests: the Phase 1 milestone compares wgpu output against it.

This spec turns the drawing semantics in `scene.md` and the renderer contract in `renderer.md`
into numbered, pixel-tested rules for this backend. It also provides the image helpers golden
tests need (PNG encode/decode, image diff), until `tantu-test` takes over golden-file
management in Phase 2.

## Scope

In scope:

- `SoftRenderer`: implements `Renderer` into an in-memory RGBA8 target; `snapshot()` reads it
  back as `ImageData`.
- Every Phase 1 command: fills and strokes of (rounded) rects, box shadows (with Gaussian blur),
  images (nearest/bilinear, sub-rects, opacity), clips, transforms, layers (opacity, overlay
  color), custom commands through registered handlers.
- Image upload cache keyed by `ImageId`, kept in sync with `Resources::revision`.
- `encode_png` / `decode_png` (straight-alpha RGBA8, lossless) and `diff_images`.
- A golden test workflow for this crate's own tests: `tests/goldens/*.png`, refreshed with
  `TANTU_UPDATE_GOLDENS=1`.

Out of scope (and where it goes):

- Drawing glyph runs. Phase 1's command list for backends (PLAN.md, the wgpu item) has no text;
  rasterizing glyphs (swash, ADR 0005) comes with `tantu-text` in Phase 2. Until then every
  glyph run counts as a font this backend couldn't use (RENDER-SOFT-04). See open question 1.
- Presenting to a window (softbuffer or similar): with the platform crates, later in Phase 1 or
  Phase 2.
- Partial repaint using damage, layer bounds, multithreaded rasterization: Phase 5.
- Golden-file management for widgets (`tantu-test`, Phase 2). The helpers here are reused.

## Dependencies

- `tiny-skia` 0.12 with default features off and `std` + `simd` on (no `png-format`). Allowed
  in `tantu-render-*` only (AGENTS.md dependency rule). License BSD-3-Clause, compatible with
  distributing Tantu under MIT OR Apache-2.0. Builds on Rust 1.85 (checked).
- `png` 0.18 (MIT OR Apache-2.0, MSRV 1.73) for lossless straight-alpha PNG I/O. tiny-skia's
  own PNG support goes through premultiplied pixels, which loses precision for semi-transparent
  pixels and would make goldens round-trip inexactly.
- `tantu-core` for geometry (already reachable through `tantu-scene`; added as a direct
  dependency since the API uses its types).

## Public API

Crate root `tantu_render_soft`.

```rust
use tantu_core::Rect;
use tantu_scene::{CustomKind, ImageData, RenderError, RenderReport, Renderer, Resources, Scene};

/// The tiny-skia version this crate draws with, for custom handlers.
pub use tiny_skia;

/// A renderer that draws Scenes into an in-memory RGBA8 target on the CPU.
pub struct SoftRenderer { /* private */ }

impl SoftRenderer {
    /// A `width × height` physical-pixel target, scale factor 1, fully transparent.
    pub fn new(width: u32, height: u32) -> SoftRenderer;
    /// Current target width and height in physical pixels.
    pub fn size(&self) -> (u32, u32);
    /// Current scale factor.
    pub fn scale_factor(&self) -> f32;
    /// Draw custom commands of `kind` with `handler`, replacing any earlier handler for it.
    pub fn register_custom(&mut self, kind: CustomKind, handler: impl CustomHandler + 'static);
    /// The target's pixels as straight-alpha RGBA8. `None` for a 0-width or 0-height target.
    pub fn snapshot(&self) -> Option<ImageData>;
}

impl Renderer for SoftRenderer { /* resize, render */ }
impl std::fmt::Debug for SoftRenderer { /* size, scale factor, registered kinds */ }

/// Draws one kind of custom command.
pub trait CustomHandler {
    /// Draw `canvas.data` into `canvas.pixmap`, inside `canvas.bounds`.
    fn draw(&mut self, canvas: CustomCanvas<'_>);
}

/// What a custom handler draws into.
pub struct CustomCanvas<'a> {
    /// The target (or the open layer's offscreen target), premultiplied RGBA8.
    pub pixmap: tiny_skia::PixmapMut<'a>,
    /// Maps the command's local coordinates to target pixels (current transform × scale factor).
    pub transform: tiny_skia::Transform,
    /// The current clip as a mask over the pixmap, if any clip is open.
    pub clip: Option<&'a tiny_skia::Mask>,
    /// The command's bounds, in local coordinates.
    pub bounds: Rect,
    /// The command's bytes (`Scene::custom_data`).
    pub data: &'a [u8],
}

/// Why PNG encoding or decoding failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum PngError {
    /// The bytes are not a PNG this crate can read.
    Decode(Box<dyn std::error::Error + Send + Sync>),
    /// The encoder failed.
    Encode(Box<dyn std::error::Error + Send + Sync>),
}
impl std::fmt::Display for PngError { /* ... */ }
impl std::error::Error for PngError { /* source() is the inner error */ }

/// Lossless PNG (8-bit RGBA, straight alpha) of `image`.
pub fn encode_png(image: &ImageData) -> Result<Vec<u8>, PngError>;
/// Decodes a PNG into straight-alpha RGBA8. Accepts 8-bit RGBA, RGB, gray and gray-alpha;
/// other formats (16-bit, palette) are expanded to RGBA8.
pub fn decode_png(bytes: &[u8]) -> Result<ImageData, PngError>;

/// How two images differ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageDiff {
    /// Pixels where some channel differs by more than the tolerance.
    pub differing_pixels: u64,
    /// The largest channel difference anywhere.
    pub max_channel_delta: u8,
}

/// Compares two images channel by channel. `None` if their sizes differ.
pub fn diff_images(a: &ImageData, b: &ImageData, tolerance: u8) -> Option<ImageDiff>;
```

## Behavior

Unless a rule says otherwise, "inside" and "outside" a shape mean at least 1 physical pixel
away from its edge (anti-aliased edge pixels are not checked by rules, only by goldens), and
colors are compared on `snapshot()` bytes within ±1 per channel.

Target and frame

- **RENDER-SOFT-01:** `new(w, h)` has `size() == (w, h)`, scale factor 1 and a fully transparent
  target. `snapshot()` is `None` when `w` or `h` is 0, and otherwise a `w × h` image.
- **RENDER-SOFT-02:** `resize(w, h, s)` sets the size and the scale factor (1 if `s` is not finite
  or not positive) and leaves a fully transparent target of the new size.
- **RENDER-SOFT-03:** Every `render` first clears the whole target to transparent: nothing from
  an earlier frame remains.
- **RENDER-SOFT-04:** `render` returns `Ok` with
  `RenderReport::for_scene(scene, resources, <kind has a registered handler>)`, plus 1 in
  `missing_fonts` for every glyph run that `for_scene` counted as neither invalid nor missing
  (this backend draws no text yet). Glyph runs draw nothing.
- **RENDER-SOFT-05:** With a 0 width or height, `render` draws nothing and returns
  `Ok(RenderReport::default())`.
- **RENDER-SOFT-06:** Scene coordinates are multiplied by the scale factor: at scale 2, a fill of
  `(10, 10, 20, 20)` covers physical pixels 20..60 on both axes. Content outside the target is
  clipped; target pixels the Scene doesn't cover stay transparent.
- **RENDER-SOFT-07:** The same Scene, resources, size, scale factor and handlers give
  byte-identical snapshots, from one renderer rendering twice or from two renderers.

Fills, blending and strokes

- **RENDER-SOFT-08:** Inside a fill, the snapshot has the fill's `to_rgba8()` color (an opaque
  fill exactly; a translucent one over transparent within ±1 after the premultiply round trip).
- **RENDER-SOFT-09:** Overlapping draws composite source-over on premultiplied sRGB-encoded values:
  white at alpha 0.5 over opaque black gives (128, 128, 128, 255) ±1, not the linear-light
  result (188).
- **RENDER-SOFT-10:** Rounded corners: with radius `r`, the pixel at the corner of the rect is
  transparent and pixels inside the rounded shape are filled. When adjacent radii add up to more
  than their side, all radii are scaled by the same factor to fit (a 40×20 rect with radius 100
  becomes a pill: radius 10). Negative radii count as 0.
- **RENDER-SOFT-11:** A stroke of width `w > 0` covers the band from the shape's edge to `w` inside
  it: inside the band is the stroke color, the area further in and the area outside are
  untouched. If `w` is at least half the shorter side, the whole shape is filled. A width of 0 or
  less draws nothing.
- **RENDER-SOFT-12:** Empty or reversed rects, and transparent colors, draw nothing.

Box shadows

- **RENDER-SOFT-13:** A shadow with `blur_radius` 0 is the shape inflated by `spread_radius`
  (radii grown by the same amount, not below 0) and moved by `offset`, filled with the shadow
  color, hard-edged.
- **RENDER-SOFT-14:** With `blur_radius` > 0, that shape is blurred with a Gaussian of sigma
  `blur_radius · 0.57735 + 0.5`, in physical pixels times the transform's scale (the square root
  of the absolute determinant, times the scale factor). Deep inside the shape (more than 3 sigma
  from its edge) the color is the shadow color; more than 3 sigma outside it is transparent; at
  the middle of a long straight edge the alpha is half the shadow's ±10 %. Negative blur counts
  as 0.

Images

- **RENDER-SOFT-15:** An image fills `dest`, scaled with no aspect-ratio correction, from `src`
  (image pixels) or the whole image. With `Nearest` sampling, scaling a 2×2 image by an integer
  factor reproduces each source pixel as a uniform block. With `Linear`, it is bilinear filtered.
- **RENDER-SOFT-16:** Image alpha is multiplied by `opacity` clamped to 0..=1 (NaN never reaches
  here: it is an invalid command).
- **RENDER-SOFT-17:** An image whose handle isn't in the resources draws nothing. Uploaded images
  are cached by handle; after a handle is removed from the resources, the next frame draws
  nothing for it and counts it missing.

Transforms, clips and layers

- **RENDER-SOFT-18:** Transforms apply to everything in their scope, nested transforms compose
  (outer first), and a pop restores the outer transform. A translate, a scale and a 90° rotation
  each move a fill to the expected pixels.
- **RENDER-SOFT-19:** A clip limits drawing in its scope to its shape (rect or rounded rect, in the
  coordinates in effect when it was pushed); nested clips intersect; a pop restores the outer
  clip.
- **RENDER-SOFT-20:** A layer is drawn as a group: two overlapping opaque fills in a layer of
  opacity 0.5 give the top fill's color at 50 % in the overlap (not a mix of both). Opacity is
  clamped to 0..=1; NaN counts as 0 (the layer draws nothing).
- **RENDER-SOFT-21:** A layer's overlay color mixes every pixel of the group toward the overlay's
  rgb by the overlay's alpha, keeping the pixel's alpha (source-atop): an opaque red fill under an
  opaque blue overlay becomes blue, and pixels the group left transparent stay transparent. A
  non-finite overlay color is ignored (the layer draws as if it had none).

Custom commands and invalid values

- **RENDER-SOFT-22:** For each `Custom` entry whose kind has a registered handler, `draw` is called
  once, in paint order, with the entry's bounds and bytes, the current transform times the scale
  factor, and the current clip (`None` with no open clip). A later `register_custom` for the same
  kind replaces the handler. Without a handler nothing is drawn.
- **RENDER-SOFT-23:** Commands `RenderReport::for_scene` counts as invalid draw nothing, and nothing
  inside a scope opened by a non-finite transform or clip is drawn (custom handlers included).
- **RENDER-SOFT-24:** `render` never panics, including for huge or tiny coordinates, huge blur
  radii, deep nesting of scopes and layers, and images far larger than the target. To get there,
  geometry is rasterized in device pixels with coordinates clamped to ±2^24 (tiny-skia's
  fixed-point scan converter fails on paths spanning about ±1e30): content within the target is
  unchanged, and only shapes reaching beyond 16.7 million pixels from it may be bent.

PNG and image diff

- **RENDER-SOFT-25:** `decode_png(&encode_png(&img)?)` equals `img` for every straight-alpha RGBA8
  image, semi-transparent pixels included. `decode_png` of an RGB, gray or gray-alpha 8-bit PNG
  gives the matching RGBA8 pixels (alpha 255 where the PNG has none). Bytes that are not a PNG
  give `Err(PngError::Decode(_))`, never a panic.
- **RENDER-SOFT-26:** `diff_images(a, b, t)` is `None` when the sizes differ. Otherwise
  `max_channel_delta` is the largest absolute difference of any channel of any pixel, and
  `differing_pixels` counts pixels with some channel differing by more than `t`. Identical
  images give `ImageDiff::default()`.

## Golden tests

This crate's tests also render the reference Scenes (one per area: shapes and strokes, shadows,
images, clips and transforms, layers) and compare them with their goldens using `diff_images`
with tolerance 2 and no differing pixels allowed. With `TANTU_UPDATE_GOLDENS=1` the tests write
the PNGs instead of comparing. These are regular tests (not `#[ignore]`): they run in a few
milliseconds. The tolerance covers SIMD rounding differences between x86-64 and arm64 CI runners.

Goldens are not rules; they catch anti-aliasing and blur regressions the probe-based rules
can't. Since the Phase 1 milestone, the reference Scenes and their PNGs live in
`tantu-render-conformance` (spec [`render-conformance/conformance.md`](../render-conformance/conformance.md),
RENDER-CONF-11), where every other backend is checked against them with a cross-backend
tolerance. This renderer produces them.

## Performance and allocation

- Offscreen pixmaps for layers and masks for clips are pooled per nesting depth and reused
  between frames; uploaded images are cached by handle. Resizing reallocates the target.
- tiny-skia allocates internally while building and filling paths. That is accepted for this
  fallback renderer; no per-frame allocation rule is promised here. To be revisited with
  profiling in Phase 5.
- Box-shadow blur is separable (two 1D passes), O(pixels × kernel size).

## Open questions

Resolved (2026-10-06, decided in autopilot: the user asked for the proposals to be taken and the
work continued; review these):

1. **Text.** No glyph rendering in Phase 1 (the PLAN.md backend items and the milestone don't
   include text). Glyph runs count as `missing_fonts` (RENDER-SOFT-04) so missing text is
   visible in reports. Rasterizing with swash comes with `tantu-text` in Phase 2, together with a
   bundled OFL test font.
2. **Public tiny-skia types in the custom-handler API.** Yes, re-exported as
   `tantu_render_soft::tiny_skia`, because a handler for this backend draws with tiny-skia. A
   tiny-skia major upgrade is then a breaking change for handler authors.
3. **Goldens in normal test runs.** This crate's goldens run with plain `cargo test` (they are
   fast), with tolerance 2. AGENTS.md's `cargo test -p tantu-test -- --ignored` applies to widget
   goldens in Phase 2.
4. **Blur sigma under transforms (RENDER-SOFT-14).** Blur in device space with sigma scaled by
   the transform's average scale. A non-uniform scale or a skew blurs equally in both directions,
   slightly unlike Flutter (which blurs in local space). Revisit if the wgpu milestone shows a
   visible difference.
5. **Layer offscreens are target-sized** for now (scene.md deferred question 7).
6. **Presenting to a window** (softbuffer) is out of scope here, decided with the platform crates.

Found while implementing (2026-10-06):

7. **Huge coordinates.** tiny-skia's scan converter hits a debug assertion for paths spanning
   about ±1e30 device pixels. Paths are now transformed to device space by the renderer and their
   points clamped to ±2^24 (RENDER-SOFT-24).
8. **Goldens and exact pixel-boundary edges.** Changing a rotation by 1e-6 radians moved 9 edge
   pixels of a golden by up to 14, where edges sit exactly on pixel boundaries. If CI on another
   architecture disagrees for the same reason, allow a small number of differing pixels instead
   of none.
