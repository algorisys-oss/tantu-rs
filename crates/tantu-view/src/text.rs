//! Text in views: [`TextPainter`], [`TextContext`], [`SystemText`] and [`ParagraphPaint`].
//! Spec: `docs/specs/view/text.md`.

use tantu_core::{Color, Point};
use tantu_layout::{NoTextMeasure, RenderParagraph, TextMeasure, TextMetrics, TextStyleKey};
use tantu_scene::{Resources, SceneBuilder};
use tantu_text::TextSystem;

use crate::{Paint, PaintCx};

/// Paints text into a Scene (the counterpart of `TextMeasure`).
pub trait TextPainter {
    /// Paints `text` as `TextMeasure::measure` lays it out with the same arguments, with the
    /// first line's top-left at `origin`, in `color`.
    #[allow(clippy::too_many_arguments)]
    fn paint_text(
        &mut self,
        scene: &mut SceneBuilder<'_>,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
        color: Color,
        origin: Point,
    );
}

/// Paints nothing (tests, no text system).
impl TextPainter for NoTextMeasure {
    fn paint_text(
        &mut self,
        scene: &mut SceneBuilder<'_>,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
        color: Color,
        origin: Point,
    ) {
        let _ = (scene, text, style, max_width, max_lines, color, origin);
    }
}

/// What a frame uses for text: measuring (layout) and painting.
pub trait TextContext: TextMeasure + TextPainter {}

impl<T: TextMeasure + TextPainter> TextContext for T {}

/// A [`TextSystem`] with the [`Resources`] its glyph runs' fonts go into.
pub struct SystemText<'a> {
    /// The text system.
    pub system: &'a mut TextSystem,
    /// The renderer resources fonts are added to.
    pub resources: &'a mut Resources,
}

impl TextMeasure for SystemText<'_> {
    fn measure(
        &mut self,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
    ) -> TextMetrics {
        self.system.measure(text, style, max_width, max_lines)
    }

    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32 {
        self.system.min_intrinsic_width(text, style)
    }
}

impl TextPainter for SystemText<'_> {
    fn paint_text(
        &mut self,
        scene: &mut SceneBuilder<'_>,
        text: &str,
        style: TextStyleKey,
        max_width: f32,
        max_lines: Option<u32>,
        color: Color,
        origin: Point,
    ) {
        self.system.paint(
            scene,
            self.resources,
            text,
            style,
            max_width,
            max_lines,
            color,
            origin,
        );
    }
}

/// Paints the element's `RenderParagraph` text in `color`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParagraphPaint {
    /// The text color.
    pub color: Color,
}

impl Paint for ParagraphPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let Some(paragraph) = cx.render::<RenderParagraph>() else {
            return;
        };
        let (text, style, max_lines) =
            (paragraph.text.clone(), paragraph.style, paragraph.max_lines);
        // Wrap where layout wrapped (VIEW-TEXT-04).
        let width = if paragraph.soft_wrap {
            cx.constraints().max_width
        } else {
            f32::INFINITY
        };
        cx.paint_text(&text, style, width, max_lines, self.color, Point::ZERO);
    }
}
