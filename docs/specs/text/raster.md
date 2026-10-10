# Glyph rasterizer

- **Status:** Implemented (decided by the agent while the user was away, at their request; review)
- **Crate:** `tantu-text`
- **Plan item:** Phase 2, "Glyph rasterization" → "Glyph rasterizer"
- **Related:** [ADR 0005](../../adr/0005-text-stack-parley-swash-fontique.md) (swash),
  [text system](system.md), [Scene](../scene/scene.md) (glyph runs), [software
  renderer](../render-soft/renderer.md), [wgpu renderer](../render-wgpu/renderer.md)

## Purpose

Scenes carry text as glyph runs (font, size, glyph ids and positions). Renderers need coverage
masks to draw them. `GlyphRasterizer` turns a glyph of a font at a pixel size into an 8-bit
coverage mask with swash, and caches the masks. Both the software and the wgpu renderer use it,
so their text is built from the same masks (which keeps them within the conformance tolerance)
and the rasterization code exists once.

## Scope

In scope:

- `GlyphRasterizer`, `GlyphMask`: rasterizing outline glyphs, subpixel horizontal positioning in
  quarter pixels, caching.

Out of scope (and where it goes):

- Drawing masks: each renderer's glyph-run spec amendment.
- Color glyphs (emoji bitmaps and COLR), hinting, variable-font axes beyond the defaults,
  synthetic bold/italic: later, when text quality work needs them (glyphs without an outline
  give no mask).
- Glyph atlases (GPU texture packing): the wgpu renderer.

## Dependencies

`swash` 0.2 (Apache-2.0 OR MIT), named by ADR 0005 for glyph rendering; its scaler produces
anti-aliased alpha masks from outlines (it carries its own small font parser and the `zeno`
rasterizer). `tantu-render-soft` and `tantu-render-wgpu` already may depend on `tantu-text`.

## Public API

Crate root `tantu_text`.

```rust
use std::sync::Arc;
use tantu_scene::{FontData, FontId};

/// An 8-bit coverage mask for one glyph, placed relative to the glyph's origin (the point on the
/// baseline where the glyph is drawn), y down.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphMask {
    pub width: u32,
    pub height: u32,
    /// Offset of the mask's left edge from the origin, in pixels.
    pub left: i32,
    /// Offset of the mask's top edge from the origin, in pixels (negative above the baseline).
    pub top: i32,
    /// `width · height` coverage values, rows top to bottom, 0 (none) to 255 (full).
    pub coverage: Arc<[u8]>,
}

/// Rasterizes glyphs into coverage masks with swash, caching them.
pub struct GlyphRasterizer { /* private */ }

impl GlyphRasterizer {
    pub fn new() -> Self;
    /// The mask of `glyph` in `font` (`data` is its file; `font` keys the cache) at `size` pixels
    /// per em, shifted right by `subpixel_x` (its fraction is rounded to a quarter pixel). `None`
    /// for a glyph with no outline (a space), an unusable font or size.
    pub fn mask(&mut self, font: FontId, data: &FontData, glyph: u32, size: f32, subpixel_x: f32) -> Option<Arc<GlyphMask>>;
    /// True if `data` can be read as a font (cached per `font`), so a renderer can tell an
    /// unusable font from a glyph without an outline.
    pub fn readable(&mut self, font: FontId, data: &FontData) -> bool;
    /// Number of cached masks (absent glyphs included).
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    /// Forgets every cached mask.
    pub fn clear(&mut self);
}
```

## Behavior

Tests use Liberation Sans (the committed test font).

- **TEXT-RASTER-01:** The mask of a visible glyph (for example "H") at 16 px is non-empty: width and
  height at least 1, `coverage.len() == width · height`, some value is 255 (inside a stem) and
  some is below 255 (anti-aliased edges). Its box lies above the baseline (`top < 0`, `top +
  height` near 0) and starts near the origin (`left` within the glyph's side bearing, 0..=3).
- **TEXT-RASTER-02:** Sizes: the mask at 32 px is about twice as tall as at 16 px (within 2 pixels).
  A glyph without an outline (the space) gives `None`. A size that is not finite or not positive
  gives `None`, and so do font bytes that aren't a font or a glyph id the font doesn't have.
- **TEXT-RASTER-03:** Subpixel positioning: `subpixel_x` is reduced to its fractional part and
  rounded to the nearest quarter (0, 0.25, 0.5, 0.75; 1.0 counts as 0). Values that round to the
  same quarter give the same mask; different quarters give different coverage.
- **TEXT-RASTER-04:** Caching: asking again for the same font, glyph, size and quarter returns an
  equal mask (the same `Arc`) without rasterizing again; `len` counts the cached entries
  (including `None` results); `clear` empties the cache. The cache keeps two generations of up to
  4 096 entries each, like the other caches.

- **TEXT-RASTER-05:** `readable(font, data)` is true for Liberation Sans and false for bytes that
  aren't a font; the answer is cached per `font` (asking again doesn't parse again).

## Performance and allocation

A cache hit is a hash lookup and an `Arc` clone. A miss rasterizes with swash (allocating the
mask). Masks are shared between renderers' frames through the `Arc`.

## Open questions

Resolved (2026-10-10, decided by the agent while the user was away; review):

1. **swash for rasterizing** (ADR 0005) rather than skrifa outlines filled with tiny-skia: the
   same masks for both renderers, and the GPU renderer doesn't need a path filler.
2. **The shared rasterizer lives in `tantu-text`**, which both render crates may use, rather than
   being duplicated per renderer.
3. **Quarter-pixel horizontal subpixel positioning, none vertically**: text is laid out on
   pixel-aligned baselines in practice, and four variants per glyph keep the cache small.
4. **Unhinted masks**: hinting differs per platform and would make goldens platform-specific.
