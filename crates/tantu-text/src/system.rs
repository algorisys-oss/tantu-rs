//! [`TextSystem`], [`TextStyle`] and [`FontFamily`]. Spec: `docs/specs/text/system.md`.

use std::sync::Arc;

use tantu_core::{Color, Point};
use tantu_layout::{TextMeasure, TextMetrics, TextStyleKey};
use tantu_scene::{Resources, SceneBuilder};

/// A font family: a name, or a generic family resolved by the font system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// A family by name, e.g. "Liberation Sans".
    Named(Arc<str>),
    /// The default sans-serif family.
    SansSerif,
    /// The default serif family.
    Serif,
    /// The default monospace family.
    Monospace,
}

/// A text style. Sizes are logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    /// The font family.
    pub family: FontFamily,
    /// Font size; non-finite or not positive counts as 16.
    pub size: f32,
    /// Weight, 100 (thin) to 900 (black); 400 is regular. Clamped to 1..=1000.
    pub weight: f32,
    /// Italic (or oblique) instead of upright.
    pub italic: bool,
    /// Line height as a multiple of the size; `None` uses the font's metrics.
    pub line_height: Option<f32>,
}

impl Default for TextStyle {
    fn default() -> Self {
        todo!()
    }
}

/// Fonts, styles, shaping and line breaking for one app.
pub struct TextSystem {}

impl TextSystem {
    /// A text system that can use the fonts installed on the system.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        todo!()
    }

    /// A text system with no fonts until some are registered (tests, embedded apps).
    pub fn without_system_fonts() -> Self {
        todo!()
    }

    /// Registers a font file (TTF, OTF or a collection) and returns the family names it added
    /// (empty, and nothing registered, if the data isn't a font).
    pub fn register_font(&mut self, data: Vec<u8>) -> Vec<String> {
        let _ = data;
        todo!()
    }

    /// The family used when a style's family has no matching font (default: `SansSerif`).
    pub fn set_default_family(&mut self, family: FontFamily) {
        let _ = family;
        todo!()
    }

    /// The key for `style`: equal styles (after sanitizing) get the same key.
    pub fn style(&mut self, style: TextStyle) -> TextStyleKey {
        let _ = style;
        todo!()
    }

    /// The style behind `key`.
    pub fn text_style(&self, key: TextStyleKey) -> Option<&TextStyle> {
        let _ = key;
        todo!()
    }

    /// Paints `text` laid out as `measure` lays it out, with the first line's top-left at
    /// `origin`, as glyph runs in `color`; fonts are added to `resources` the first time they
    /// are used (and reused after).
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        scene: &mut SceneBuilder<'_>,
        resources: &mut Resources,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
        color: Color,
        origin: Point,
    ) {
        let _ = (
            scene, resources, text, style, max_width, max_lines, color, origin,
        );
        todo!()
    }
}

impl TextMeasure for TextSystem {
    fn measure(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics {
        let _ = (text, style, max_width, max_lines);
        todo!()
    }

    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32 {
        let _ = (text, style);
        todo!()
    }
}
