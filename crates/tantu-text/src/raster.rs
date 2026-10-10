//! [`GlyphRasterizer`]: glyph coverage masks for the renderers (ADR 0005). Spec:
//! `docs/specs/text/raster.md`.

use std::sync::Arc;

use tantu_scene::{FontData, FontId};

/// An 8-bit coverage mask for one glyph, placed relative to the glyph's origin (the point on
/// the baseline where the glyph is drawn), y down.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphMask {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Offset of the mask's left edge from the origin, in pixels.
    pub left: i32,
    /// Offset of the mask's top edge from the origin, in pixels (negative above the baseline).
    pub top: i32,
    /// `width · height` coverage values, rows top to bottom, 0 (none) to 255 (full).
    pub coverage: Arc<[u8]>,
}

/// Rasterizes glyphs into coverage masks with swash, caching them.
pub struct GlyphRasterizer {}

impl GlyphRasterizer {
    /// An empty rasterizer.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        todo!()
    }

    /// The mask of `glyph` in `font` (`data` is its file; `font` keys the cache) at `size`
    /// pixels per em, shifted right by `subpixel_x` (its fraction is rounded to a quarter
    /// pixel). `None` for a glyph with no outline (a space), an unusable font or size.
    pub fn mask(
        &mut self,
        font: FontId,
        data: &FontData,
        glyph: u32,
        size: f32,
        subpixel_x: f32,
    ) -> Option<Arc<GlyphMask>> {
        let _ = (font, data, glyph, size, subpixel_x);
        todo!()
    }

    /// Number of cached masks (absent glyphs included).
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True with nothing cached.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Forgets every cached mask.
    pub fn clear(&mut self) {
        todo!()
    }
}
