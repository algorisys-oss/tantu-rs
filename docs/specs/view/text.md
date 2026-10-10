# Text in views

- **Status:** Agreed (decided by the agent while the user was away, at their request; review)
- **Crate:** `tantu-view`
- **Plan item:** Phase 2, `tantu-text` → "Text in views"
- **Related:** [ADR 0010](../../adr/0010-text-measurement-in-layout.md), [paint](paint.md),
  [frame](frame.md), [text system](../text/system.md), [text measurement](../layout/text.md)

## Purpose

Connects the text system to the view layer. Layout already measures through `TextMeasure`
(ADR 0010); painting text needs the same system plus the renderer's `Resources` (glyph runs name
fonts registered there). This spec adds a text painter to the paint traversal, a `TextContext`
that a frame uses for both, the paint behavior that draws a `RenderParagraph`, and the adapter
that makes a `TextSystem` (with `Resources`) a `TextContext`.

## Scope

In scope:

- `TextPainter`, `TextContext`, `SystemText` (the `TextSystem` + `Resources` adapter).
- `ViewTree::paint` and `ViewTree::frame` taking the text context (amends the paint and frame
  specs), `PaintCx::text`, `PaintCx::constraints`, `PaintCx::render`.
- `ParagraphPaint`.

Out of scope:

- The `Text` widget (reactive text, styles from the theme): `tantu-widgets` (Phase 3; a minimal
  one comes with the counter example).
- Rich text, selection, editing: Phase 3.

## Public API

Crate root `tantu_view`.

```rust
use tantu_core::{Color, Point};
use tantu_layout::{BoxConstraints, RenderBox, TextMeasure, TextStyleKey};
use tantu_scene::{Resources, SceneBuilder};
use tantu_text::TextSystem;

/// Paints text into a Scene (the counterpart of `TextMeasure`).
pub trait TextPainter {
    /// Paints `text` as `TextMeasure::measure` lays it out with the same arguments, with the first
    /// line's top-left at `origin`, in `color`.
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
/// Measures and paints nothing (tests, no text system).
impl TextPainter for tantu_layout::NoTextMeasure { /* nothing */ }

/// What a frame uses for text: measuring (layout) and painting.
pub trait TextContext: TextMeasure + TextPainter {}
impl<T: TextMeasure + TextPainter> TextContext for T {}

/// A `TextSystem` with the `Resources` its glyph runs' fonts go into.
pub struct SystemText<'a> {
    pub system: &'a mut TextSystem,
    pub resources: &'a mut Resources,
}
impl TextMeasure for SystemText<'_> { /* delegates to the system */ }
impl TextPainter for SystemText<'_> { /* TextSystem::paint with the resources */ }

impl PaintCx<'_, '_> {
    /// The text painter of this paint pass.
    pub fn text(&mut self) -> &mut dyn TextPainter;
    /// Paints text through the painter into this context's Scene builder (`text()` and
    /// `scene()` can't be borrowed at the same time).
    pub fn paint_text(&mut self, text: &str, style: TextStyleKey, max_width: f32, max_lines: Option<u32>, color: Color, origin: Point);
    /// The element's constraints from the last layout.
    pub fn constraints(&self) -> BoxConstraints;
    /// The element's layout object, if it is an `R`.
    pub fn render<R: RenderBox>(&self) -> Option<&R>;
}

impl ViewTree {
    /// (Amended) paints with `text` as the text painter.
    pub fn paint(&self, scene: &mut SceneBuilder<'_>, text: &mut dyn TextPainter);
    /// (Amended) lays out and paints with `text` as the text context.
    pub fn frame(&mut self, constraints: BoxConstraints, text: &mut dyn TextContext, scene: &mut Scene) -> FrameReport;
}

/// Paints the element's `RenderParagraph` text in `color`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParagraphPaint {
    pub color: Color,
}
impl Paint for ParagraphPaint { /* ... */ }
```

## Behavior

- **VIEW-TEXT-01:** `NoTextMeasure` is a `TextPainter` that records nothing, so it is a
  `TextContext`; any type that is both `TextMeasure` and `TextPainter` is a `TextContext`.
- **VIEW-TEXT-02:** During `ViewTree::paint(scene, text)`, `PaintCx::text()` is `text` (calls reach
  it) and `PaintCx::paint_text(...)` calls `text.paint_text` with the context's Scene builder;
  `ViewTree::frame` passes its context both to the layout pass and to the paint pass.
- **VIEW-TEXT-03:** `PaintCx::constraints()` is the element's constraints from the last layout and
  `PaintCx::render::<R>()` its layout object when it is an `R` (else `None`).
- **VIEW-TEXT-04:** `ParagraphPaint` paints the element's `RenderParagraph` through the painter:
  its text, style and `max_lines`, wrapping at `constraints().max_width` when `soft_wrap` (else
  unbounded), in its color, at the element's origin. It paints nothing when the element's layout
  object isn't a `RenderParagraph`. Its children are painted after it, as usual.
- **VIEW-TEXT-05:** `SystemText` measures exactly as its `TextSystem` does and paints with
  `TextSystem::paint` into its `Resources`: a paragraph laid out and painted in a frame with a
  `SystemText` yields glyph runs whose fonts are in those `Resources`.

## Performance and allocation

`ParagraphPaint` measures nothing itself; the text system re-breaks a cached layout (the measure
during layout shaped it). Painting allocates the glyph buffers the text system copies into the
Scene (the Scene's own buffers are reused).

## Open questions

Resolved (2026-10-10, decided by the agent while the user was away; review):

1. **The painter trait lives in `tantu-view`** (it needs the Scene builder and is what the view
   layer calls); `tantu-text` stays free of view types, and `SystemText` bridges the two.
2. **Paint wraps at the layout constraints** (`PaintCx::constraints`), not at the element's
   width, so painted lines always match measured lines.
3. **`ParagraphPaint` reads the paragraph from the layout object** rather than duplicating the
   text and style in the paint behavior.
