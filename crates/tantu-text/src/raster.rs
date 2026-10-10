//! [`GlyphRasterizer`]: glyph coverage masks for the renderers (ADR 0005). Spec:
//! `docs/specs/text/raster.md`.

use std::collections::HashMap;
use std::sync::Arc;

use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::{Format, Vector};

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

/// Cache key: font, glyph, size bits, quarter-pixel offset (0..=3).
type Key = (FontId, u32, u32, u8);

/// How many masks each cache generation holds.
const MASKS_PER_GENERATION: usize = 4096;

/// Rasterizes glyphs into coverage masks with swash, caching them.
pub struct GlyphRasterizer {
    scale: ScaleContext,
    current: HashMap<Key, Option<Arc<GlyphMask>>>,
    old: HashMap<Key, Option<Arc<GlyphMask>>>,
}

impl GlyphRasterizer {
    /// An empty rasterizer.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        GlyphRasterizer {
            scale: ScaleContext::new(),
            current: HashMap::new(),
            old: HashMap::new(),
        }
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
        if !(size.is_finite() && size > 0.0) {
            return None;
        }
        let quarter = quarter(subpixel_x);
        let key = (font, glyph, size.to_bits(), quarter);
        if let Some(mask) = self.current.get(&key) {
            return mask.clone();
        }
        let mask = match self.old.remove(&key) {
            Some(mask) => mask,
            None => self.rasterize(data, glyph, size, f32::from(quarter) / 4.0),
        };
        if self.current.len() >= MASKS_PER_GENERATION {
            self.old = std::mem::take(&mut self.current);
        }
        self.current.insert(key, mask.clone());
        mask
    }

    /// Renders one glyph with swash (TEXT-RASTER-01, -02).
    fn rasterize(
        &mut self,
        data: &FontData,
        glyph: u32,
        size: f32,
        offset_x: f32,
    ) -> Option<Arc<GlyphMask>> {
        let glyph = u16::try_from(glyph).ok()?;
        let font = FontRef::from_index(data.bytes(), usize::try_from(data.index()).ok()?)?;
        if glyph >= font.metrics(&[]).glyph_count {
            return None;
        }
        let mut scaler = self.scale.builder(font).size(size).hint(false).build();
        let image = Render::new(&[Source::Outline])
            .format(Format::Alpha)
            .offset(Vector::new(offset_x, 0.0))
            .render(&mut scaler, glyph)?;
        let placement = image.placement;
        if placement.width == 0 || placement.height == 0 {
            return None;
        }
        Some(Arc::new(GlyphMask {
            width: placement.width,
            height: placement.height,
            left: placement.left,
            // swash measures up from the baseline; masks are placed y down.
            top: -placement.top,
            coverage: image.data.into(),
        }))
    }

    /// True if `data` can be read as a font (cached per `font`), so a renderer can tell an
    /// unusable font from a glyph without an outline.
    pub fn readable(&mut self, font: FontId, data: &FontData) -> bool {
        let _ = (font, data);
        todo!()
    }

    /// Number of cached masks (absent glyphs included).
    pub fn len(&self) -> usize {
        self.current.len() + self.old.len()
    }

    /// True with nothing cached.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Forgets every cached mask.
    pub fn clear(&mut self) {
        self.current.clear();
        self.old.clear();
    }
}

/// The fraction of `x` rounded to a quarter pixel, as 0..=3 (TEXT-RASTER-03).
fn quarter(x: f32) -> u8 {
    if !x.is_finite() {
        return 0;
    }
    let fraction = x - x.floor();
    // 0.875 and above rounds to the next whole pixel, which is quarter 0.
    ((fraction * 4.0).round() as u8) % 4
}
