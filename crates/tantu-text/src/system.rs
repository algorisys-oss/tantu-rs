//! [`TextSystem`], [`TextStyle`] and [`FontFamily`]. Spec: `docs/specs/text/system.md`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use parley::fontique::{Blob, Collection, CollectionOptions};
use parley::{
    FontContext, FontFamilyName, FontStyle, FontWeight, GenericFamily, Layout, LayoutContext,
    LineHeight, PositionedLayoutItem, StyleProperty,
};
use tantu_core::{Color, Point};
use tantu_layout::{TextMeasure, TextMetrics, TextStyleKey};
use tantu_scene::{FontData, FontId, Glyph, Resources, SceneBuilder};

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

impl FontFamily {
    /// The parley family name.
    fn to_parley(&self) -> FontFamilyName<'static> {
        match self {
            FontFamily::Named(name) => FontFamilyName::Named(Cow::Owned(name.to_string())),
            FontFamily::SansSerif => FontFamilyName::Generic(GenericFamily::SansSerif),
            FontFamily::Serif => FontFamilyName::Generic(GenericFamily::Serif),
            FontFamily::Monospace => FontFamilyName::Generic(GenericFamily::Monospace),
        }
    }
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

/// The font size used when a style's size is unusable.
const DEFAULT_SIZE: f32 = 16.0;

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            family: FontFamily::SansSerif,
            size: DEFAULT_SIZE,
            weight: 400.0,
            italic: false,
            line_height: None,
        }
    }
}

impl TextStyle {
    /// The style with unusable values replaced (TEXT-SYS-02).
    pub(crate) fn sanitized(mut self) -> Self {
        if !(self.size.is_finite() && self.size > 0.0) {
            self.size = DEFAULT_SIZE;
        }
        self.weight = if self.weight.is_nan() {
            400.0
        } else {
            self.weight.clamp(1.0, 1000.0)
        };
        self.line_height = self.line_height.filter(|h| h.is_finite() && *h > 0.0);
        self
    }
}

/// Shaped layouts kept between calls: two generations, like `MeasureCache`.
#[derive(Default)]
struct LayoutCache {
    current: HashMap<(String, TextStyleKey), Layout<()>>,
    old: HashMap<(String, TextStyleKey), Layout<()>>,
}

/// One glyph run read out of a layout: the font (a cheap handle), face index, size, glyphs.
type ShapedRun = (Blob<u8>, u32, f32, Vec<Glyph>);

/// How many shaped layouts each cache generation holds.
const LAYOUTS_PER_GENERATION: usize = 512;

/// Fonts, styles, shaping and line breaking for one app.
pub struct TextSystem {
    fonts: FontContext,
    layouts: LayoutContext<()>,
    styles: crate::TextStyles,
    default_family: FontFamily,
    /// Families added by `register_font`, in registration order (the fallback list,
    /// TEXT-SCRIPT-03).
    registered: Vec<Arc<str>>,
    /// Scene font handles for parley fonts, by (blob id, face index).
    font_ids: HashMap<(u64, u32), FontId>,
    cache: LayoutCache,
}

impl TextSystem {
    /// A text system that can use the fonts installed on the system.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        TextSystem::with_collection(Collection::new(CollectionOptions {
            shared: false,
            system_fonts: true,
        }))
    }

    /// A text system with no fonts until some are registered (tests, embedded apps).
    pub fn without_system_fonts() -> Self {
        TextSystem::with_collection(Collection::new(CollectionOptions {
            shared: false,
            system_fonts: false,
        }))
    }

    fn with_collection(collection: Collection) -> Self {
        TextSystem {
            fonts: FontContext {
                collection,
                source_cache: Default::default(),
            },
            layouts: LayoutContext::new(),
            styles: crate::TextStyles::new(),
            default_family: FontFamily::SansSerif,
            registered: Vec::new(),
            font_ids: HashMap::new(),
            cache: LayoutCache::default(),
        }
    }

    /// Registers a font file (TTF, OTF or a collection) and returns the family names it added
    /// (empty, and nothing registered, if the data isn't a font).
    pub fn register_font(&mut self, data: Vec<u8>) -> Vec<String> {
        if data.is_empty() {
            return Vec::new();
        }
        let families = self
            .fonts
            .collection
            .register_fonts(Blob::new(Arc::new(data)), None);
        let names: Vec<String> = families
            .iter()
            .filter_map(|(family, _)| self.fonts.collection.family_name(*family).map(String::from))
            .collect();
        for name in &names {
            if !self.registered.iter().any(|r| **r == **name) {
                self.registered.push(Arc::from(name.as_str()));
            }
        }
        self.cache = LayoutCache::default();
        names
    }

    /// The family used when a style's family has no matching font (default: `SansSerif`).
    pub fn set_default_family(&mut self, family: FontFamily) {
        self.default_family = family;
        self.cache = LayoutCache::default();
    }

    /// The key for `style`: equal styles (after sanitizing) get the same key.
    pub fn style(&mut self, style: TextStyle) -> TextStyleKey {
        self.styles.key(style)
    }

    /// The system's style table (a clone of the handle).
    pub fn styles(&self) -> crate::TextStyles {
        self.styles.clone()
    }

    /// Uses `styles` as the system's table instead of its own.
    pub fn with_styles(mut self, styles: crate::TextStyles) -> Self {
        self.styles = styles;
        self.cache = LayoutCache::default();
        self
    }

    /// The style behind `key`.
    pub fn text_style(&self, key: TextStyleKey) -> Option<TextStyle> {
        self.styles.get(key)
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
        if text.is_empty() {
            return;
        }
        let keep = lines_to_keep(max_lines);
        let layout = self.broken_layout(text, style, max_width);
        let mut runs: Vec<ShapedRun> = Vec::new();
        for line in layout.lines().take(keep) {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(run) = item else {
                    continue;
                };
                let font = run.run().font();
                let glyphs: Vec<Glyph> = run
                    .positioned_glyphs()
                    .map(|g| Glyph {
                        id: g.id,
                        x: g.x,
                        y: g.y,
                    })
                    .collect();
                runs.push((font.data.clone(), font.index, run.run().font_size(), glyphs));
            }
        }
        for (blob, index, size, glyphs) in runs {
            let Some(font) = self.font_id(resources, &blob, index) else {
                continue;
            };
            scene.glyph_run(font, size, color, origin, &glyphs);
        }
    }

    /// The Scene font for a parley font, adding it to `resources` if needed.
    fn font_id(
        &mut self,
        resources: &mut Resources,
        blob: &Blob<u8>,
        index: u32,
    ) -> Option<FontId> {
        let key = (blob.id(), index);
        if let Some(id) = self.font_ids.get(&key) {
            if resources.font(*id).is_some() {
                return Some(*id);
            }
        }
        // The font file is copied once, when it is first added to `resources`.
        let bytes: Arc<[u8]> = Arc::from(blob.data());
        let id = resources.add_font(FontData::new(bytes, index).ok()?);
        self.font_ids.insert(key, id);
        Some(id)
    }

    /// The shaped layout of `text` in `style` (cached), with its lines broken at `max_width`.
    fn broken_layout(&mut self, text: &str, style: TextStyleKey, max_width: f32) -> &Layout<()> {
        let layout = self.layout(text, style);
        layout.break_all_lines(wrap_width(max_width));
        layout
    }

    /// The shaped layout of `text` in `style`, from the cache or shaped now.
    fn layout(&mut self, text: &str, style: TextStyleKey) -> &mut Layout<()> {
        let key = (text.to_owned(), style);
        if !self.cache.current.contains_key(&key) {
            let layout = match self.cache.old.remove(&key) {
                Some(layout) => layout,
                None => self.shape(text, style),
            };
            if self.cache.current.len() >= LAYOUTS_PER_GENERATION {
                self.cache.old = std::mem::take(&mut self.cache.current);
            }
            self.cache.current.insert(key.clone(), layout);
        }
        self.cache
            .current
            .get_mut(&key)
            .expect("inserted just above if it was missing")
    }

    /// Shapes `text` in `style` (the default style for an unknown key) with parley.
    fn shape(&mut self, text: &str, style: TextStyleKey) -> Layout<()> {
        let style = self.text_style(style).unwrap_or_default();
        // The style's family, the default family, then every registered family in order
        // (TEXT-SCRIPT-03); parley falls back per character along this list.
        let mut stack = vec![style.family.to_parley(), self.default_family.to_parley()];
        for name in &self.registered {
            let family = FontFamily::Named(Arc::clone(name)).to_parley();
            if !stack.contains(&family) {
                stack.push(family);
            }
        }
        let mut builder = self
            .layouts
            .ranged_builder(&mut self.fonts, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(style.size));
        builder.push_default(StyleProperty::FontFamily(parley::FontFamily::List(
            Cow::Owned(stack),
        )));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(style.weight)));
        if style.italic {
            builder.push_default(StyleProperty::FontStyle(FontStyle::Italic));
        }
        if let Some(height) = style.line_height {
            builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(
                height,
            )));
        }
        builder.build(text)
    }
}

/// parley's maximum advance for `max_width` (TEXT-SYS-08).
fn wrap_width(max_width: f32) -> Option<f32> {
    if max_width.is_nan() || max_width.is_infinite() && max_width > 0.0 {
        None
    } else if max_width > 0.0 {
        Some(max_width)
    } else {
        Some(0.0)
    }
}

/// How many lines `max_lines` keeps.
fn lines_to_keep(max_lines: Option<u32>) -> usize {
    max_lines.map_or(usize::MAX, |n| usize::try_from(n).unwrap_or(usize::MAX))
}

impl TextMeasure for TextSystem {
    fn measure(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics {
        if text.is_empty() {
            return TextMetrics::default();
        }
        let keep = lines_to_keep(max_lines);
        let layout = self.broken_layout(text, style, max_width);
        let mut metrics = TextMetrics::default();
        for line in layout.lines().take(keep) {
            let m = line.metrics();
            if metrics.line_count == 0 {
                metrics.first_baseline = m.baseline;
            }
            metrics.last_baseline = m.baseline;
            metrics.line_count += 1;
            metrics.size.height += m.line_height;
            metrics.size.width = metrics.size.width.max(m.advance - m.hanging_advance);
        }
        metrics
    }

    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        self.layout(text, style).calculate_content_widths().min
    }
}
