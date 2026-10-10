//! Text measurement (ADR 0010): the [`TextMeasure`] trait the text system implements, the
//! [`MeasureCache`] in front of it, and [`RenderParagraph`], the layout object behind `Text`.
//! Spec: `docs/specs/layout/text.md`.

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

/// Caches another measurer's results (ADR 0010): whole paragraphs keyed by the exact text,
/// style, width and line limit, minimum intrinsic widths keyed by text and style, and a
/// shortcut for text that already fits on one line.
pub struct MeasureCache<M> {
    inner: M,
}

impl<M: TextMeasure> MeasureCache<M> {
    /// A cache in front of `inner` holding up to about `capacity` paragraph results.
    pub fn new(inner: M, capacity: usize) -> Self {
        let _ = capacity;
        let _ = inner;
        todo!()
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
        todo!()
    }

    /// Number of cached paragraph results.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// True with nothing cached.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Lookups answered from the cache since creation.
    pub fn hits(&self) -> u64 {
        todo!()
    }

    /// Lookups passed to the inner measurer since creation.
    pub fn misses(&self) -> u64 {
        todo!()
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
        let _ = (text, style, max_width, max_lines);
        todo!()
    }

    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32 {
        let _ = (text, style);
        todo!()
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
        let _ = (text.into(), style);
        todo!()
    }
}

impl RenderBox for RenderParagraph {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!()
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!()
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!()
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!()
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!()
    }
}
