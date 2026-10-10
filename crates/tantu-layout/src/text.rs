//! Text measurement (ADR 0010): the [`TextMeasure`] trait the text system implements, the
//! [`MeasureCache`] in front of it, and [`RenderParagraph`], the layout object behind `Text`.
//! Spec: `docs/specs/layout/text.md`.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use tantu_core::Size;

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// An opaque text style (font family, size, weight, …), assigned by the text system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextStyleKey(pub u64);

/// The result of measuring a paragraph.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextMetrics {
    /// Width of the longest line and total height of the laid-out lines.
    pub size: Size,
    /// Number of lines (0 for empty text).
    pub line_count: u32,
    /// Distance from the top to the first line's alphabetic baseline.
    pub first_baseline: f32,
    /// Distance from the top to the last line's alphabetic baseline.
    pub last_baseline: f32,
}

/// Measures paragraphs. `tantu-text` implements it over parley; tests use fakes.
pub trait TextMeasure {
    /// Lays out `text` in `style`, wrapping at `max_width` (`f32::INFINITY`: only hard line
    /// breaks) and keeping at most `max_lines` lines, and returns its metrics.
    fn measure(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics;

    /// The width of the widest unbreakable piece of `text` (its minimum intrinsic width).
    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32;
}

/// A measurer with no text system: every text measures as empty (zero metrics).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoTextMeasure;

impl TextMeasure for NoTextMeasure {
    fn measure(&mut self, _: &str, _: TextStyleKey, _: f32, _: Option<u32>) -> TextMetrics {
        TextMetrics::default()
    }

    fn min_intrinsic_width(&mut self, _: &str, _: TextStyleKey) -> f32 {
        0.0
    }
}

/// A hash of `text`, used with a full comparison so collisions can't return wrong results.
fn text_hash(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Paragraph key: text hash, style, `max_width` bits, `max_lines`.
type ParagraphKey = (u64, TextStyleKey, u32, Option<u32>);
/// Intrinsic-width key: text hash, style.
type WidthKey = (u64, TextStyleKey);

/// Two generations of cached values (LAYOUT-TEXT-05): when the current generation is full it
/// becomes the old one and the previous old one is dropped; old hits move to the current one.
struct Generations<K, V> {
    half: usize,
    current: HashMap<K, (Box<str>, V)>,
    old: HashMap<K, (Box<str>, V)>,
}

impl<K: Copy + Eq + Hash, V: Copy> Generations<K, V> {
    fn new(capacity: usize) -> Self {
        Generations {
            half: capacity.div_ceil(2),
            current: HashMap::new(),
            old: HashMap::new(),
        }
    }

    fn len(&self) -> usize {
        self.current.len() + self.old.len()
    }

    fn clear(&mut self) {
        self.current.clear();
        self.old.clear();
    }

    /// The value for `key` if it was stored for exactly `text`.
    fn get(&mut self, key: K, text: &str) -> Option<V> {
        if let Some((stored, value)) = self.current.get(&key) {
            return (**stored == *text).then_some(*value);
        }
        let (stored, value) = self.old.get(&key)?;
        if **stored != *text {
            return None;
        }
        let value = *value;
        if let Some(entry) = self.old.remove(&key) {
            self.store(key, entry);
        }
        Some(value)
    }

    fn insert(&mut self, key: K, text: &str, value: V) {
        if self.half > 0 {
            self.store(key, (text.into(), value));
        }
    }

    fn store(&mut self, key: K, entry: (Box<str>, V)) {
        if self.current.len() >= self.half && !self.current.contains_key(&key) {
            self.old = std::mem::take(&mut self.current);
        }
        self.current.insert(key, entry);
    }
}

/// Caches another measurer's results (ADR 0010): whole paragraphs keyed by the exact text,
/// style, width and line limit, minimum intrinsic widths keyed by text and style, and a
/// shortcut for text that already fits on one line.
pub struct MeasureCache<M> {
    inner: M,
    paragraphs: Generations<ParagraphKey, TextMetrics>,
    widths: Generations<WidthKey, f32>,
    hits: u64,
    misses: u64,
}

impl<M: TextMeasure> MeasureCache<M> {
    /// A cache in front of `inner` holding up to about `capacity` paragraph results.
    pub fn new(inner: M, capacity: usize) -> Self {
        MeasureCache {
            inner,
            paragraphs: Generations::new(capacity),
            widths: Generations::new(capacity),
            hits: 0,
            misses: 0,
        }
    }

    /// The wrapped measurer.
    pub fn inner(&self) -> &M {
        &self.inner
    }

    /// The wrapped measurer, for changing it; call [`clear`](Self::clear) if its results
    /// change.
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.inner
    }

    /// Forgets every cached result (fonts or styles changed).
    pub fn clear(&mut self) {
        self.paragraphs.clear();
        self.widths.clear();
    }

    /// Number of cached paragraph results.
    pub fn len(&self) -> usize {
        self.paragraphs.len()
    }

    /// True with nothing cached.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Lookups answered from the cache since creation.
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Lookups passed to the inner measurer since creation.
    pub fn misses(&self) -> u64 {
        self.misses
    }
}

impl<M: TextMeasure> TextMeasure for MeasureCache<M> {
    fn measure(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics {
        let hash = text_hash(text);
        let key = (hash, style, max_width.to_bits(), max_lines);
        if let Some(metrics) = self.paragraphs.get(key, text) {
            self.hits += 1;
            return metrics;
        }
        // Text that already fits on one line wraps the same at any wider width
        // (LAYOUT-TEXT-03).
        if max_width.is_finite() {
            let single_key = (hash, style, f32::INFINITY.to_bits(), None);
            if let Some(single) = self.paragraphs.get(single_key, text) {
                if single.size.width <= max_width
                    && max_lines.is_none_or(|lines| lines >= single.line_count)
                {
                    self.hits += 1;
                    return single;
                }
            }
        }
        self.misses += 1;
        let metrics = self.inner.measure(text, style, max_width, max_lines);
        self.paragraphs.insert(key, text, metrics);
        metrics
    }

    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32 {
        let key = (text_hash(text), style);
        if let Some(width) = self.widths.get(key, text) {
            self.hits += 1;
            return width;
        }
        self.misses += 1;
        let width = self.inner.min_intrinsic_width(text, style);
        self.widths.insert(key, text, width);
        width
    }
}

/// How a paragraph's width is chosen (Flutter's `TextWidthBasis`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextWidthBasis {
    /// As wide as the text on one line, up to the incoming maximum: wrapped text takes the
    /// full width (Flutter's default).
    #[default]
    Parent,
    /// As wide as the longest laid-out line.
    LongestLine,
}

/// Lays out a paragraph of text in one style (Flutter's `RenderParagraph`, behind `Text`). It
/// only measures, through the pass's [`TextMeasure`]; glyphs come later, at paint time.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderParagraph {
    /// The text.
    pub text: Arc<str>,
    /// Its style.
    pub style: TextStyleKey,
    /// Wrap at the incoming maximum width (`true`), or only at hard line breaks.
    pub soft_wrap: bool,
    /// Keep at most this many lines.
    pub max_lines: Option<u32>,
    /// How the width is chosen.
    pub text_width_basis: TextWidthBasis,
}

impl RenderParagraph {
    /// `text` in `style`, wrapping, no line limit, `Parent` width basis.
    pub fn new(text: impl Into<Arc<str>>, style: TextStyleKey) -> Self {
        RenderParagraph {
            text: text.into(),
            style,
            soft_wrap: true,
            max_lines: None,
            text_width_basis: TextWidthBasis::Parent,
        }
    }

    /// The width to wrap at for an incoming maximum (LAYOUT-TEXT-08).
    fn wrap_width(&self, max_width: f32) -> f32 {
        if self.soft_wrap {
            max_width
        } else {
            f32::INFINITY
        }
    }

    /// The text on one line, ignoring the line limit (Flutter's max intrinsic width).
    fn single_line(&self, text: &mut dyn TextMeasure) -> TextMetrics {
        text.measure(&self.text, self.style, f32::INFINITY, None)
    }

    /// The height of the text wrapped at `width`.
    fn height_at(&self, width: f32, text: &mut dyn TextMeasure) -> f32 {
        text.measure(
            &self.text,
            self.style,
            self.wrap_width(width),
            self.max_lines,
        )
        .size
        .height
    }
}

impl RenderBox for RenderParagraph {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let text = children.text();
        let metrics = text.measure(
            &self.text,
            self.style,
            self.wrap_width(constraints.max_width),
            self.max_lines,
        );
        let width = match self.text_width_basis {
            TextWidthBasis::Parent => self.single_line(text).size.width.min(constraints.max_width),
            TextWidthBasis::LongestLine => metrics.size.width,
        };
        Size::new(width, metrics.size.height)
    }

    fn min_intrinsic_width(&self, _: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.text().min_intrinsic_width(&self.text, self.style)
    }

    fn max_intrinsic_width(&self, _: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.single_line(children.text()).size.width
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.height_at(width, children.text())
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.height_at(width, children.text())
    }
}
