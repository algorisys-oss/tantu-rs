# RichText

- **Status:** Agreed (decided by the agent while the user was away, 2026-10-11; review the
  Open questions)
- **Crates:** `tantu-widgets` (`TextSpan`, `RichText`), `tantu-layout` (`StyleSpan`, rich
  measuring, `RenderParagraph::spans`), `tantu-text` (parley ranges and brushes),
  `tantu-view` (`paint_rich`)
- **Plan item:** Phase 3, "Widgets" → "`RichText` (multi-style paragraphs in `tantu-text`)"
- **Related:** [text measurement](../layout/text.md), [text system](../text/system.md),
  [text in views](../view/text.md), [Text and Button](basic.md), Flutter's `RichText` and
  `TextSpan`

## Purpose

A paragraph whose runs have different styles and colors: a bold word, a link in blue, a
larger first letter. Flutter composes a `TextSpan` tree; the paragraph is laid out as one
unit so line breaking, bidi and fallback work across the runs. Tantu flattens the tree into
*style spans* (byte ranges with a style key and an optional color) that travel with the text
through measuring and painting. Parley lays the ranges out in one layout, and each span's
color rides along as the run's brush, so painting emits one glyph run per (font, color) run.

The plain-text paths stay as they are: every rich method has a default that ignores spans,
so measurers and painters that don't know spans keep working, and a paragraph with no spans
is exactly the paragraph of today.

## Scope

In scope:

- `StyleSpan`, `TextMeasure::measure_rich` / `min_intrinsic_width_rich`,
  `RenderParagraph::spans`, `MeasureCache` caching rich paragraphs.
- `TextSystem` laying out spans with parley (styles and brushes), `paint_rich`.
- `TextPainter::paint_rich`, `PaintCx::paint_rich`, `ParagraphPaint` painting spans.
- `TextSpan` (text, style and color overrides, children) and `RichText`.

Out of scope (and where it goes):

- Underline and strikethrough (parley has them; the painter must draw lines): with
  `TextField`/links, next.
- Inline widgets (`WidgetSpan`), tap recognizers on spans (links): with gestures and
  `TextField`.
- Selection: `TextField`. Accessibility: `tantu-a11y`.
- Per-span letter spacing, word spacing, shadows, background colors: the theme or later.

## Public API

```rust
// tantu-layout
/// A style for a byte range of a paragraph's text (a flattened `TextSpan`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleSpan {
    /// The range's start, a byte offset into the text.
    pub start: usize,
    /// Its end (exclusive).
    pub end: usize,
    /// The range's style.
    pub style: TextStyleKey,
    /// The range's color; `None` paints it in the paragraph's color.
    pub color: Option<Color>,
}

pub trait TextMeasure {
    // ... measure, min_intrinsic_width as before ...
    /// `measure` with `spans` styling parts of `text` (ranges outside the text, empty, or not
    /// on character boundaries are ignored; later spans win where they overlap). Default:
    /// `measure`, ignoring the spans.
    fn measure_rich(&mut self, text: &str, style: TextStyleKey, spans: &[StyleSpan],
        max_width: f32, max_lines: Option<u32>) -> TextMetrics;
    /// `min_intrinsic_width` with spans. Default: ignores them.
    fn min_intrinsic_width_rich(&mut self, text: &str, style: TextStyleKey, spans: &[StyleSpan]) -> f32;
}

pub struct RenderParagraph {
    // ... as before ...
    /// Style spans over `text` (empty for plain text).
    pub spans: Arc<[StyleSpan]>,
}

// tantu-text
impl TextSystem {
    /// `paint` with spans (TEXT-SYS-11).
    pub fn paint_rich(&mut self, scene, resources, text, style, spans: &[StyleSpan], max_width, max_lines, color, origin);
}

// tantu-view
impl BuildCx<'_> {
    /// The tree's style table (a clone of the handle), so a binding can resolve style keys
    /// when a reactive span changes (VIEW-STYLES-03).
    pub fn text_styles(&self) -> TextStyles;
}

pub trait TextPainter {
    // ...
    /// `paint_text` with spans. Default: `paint_text`, ignoring them.
    fn paint_rich(&mut self, scene, text, style, spans: &[StyleSpan], max_width, max_lines, color, origin);
}
impl PaintCx<'_, '_> {
    pub fn paint_rich(&mut self, text: &str, style: TextStyleKey, spans: &[StyleSpan], max_width: f32, max_lines: Option<u32>, color: Color, origin: Point);
}

// tantu-widgets
/// A run of text with a style, or overrides of its parent's, and children (Flutter's
/// `TextSpan`).
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan { /* text, style, size, weight, italic, family, color, children */ }

impl TextSpan {
    /// A span of `text` inheriting everything.
    pub fn new(text: impl Into<String>) -> Self;
    /// A span with no text of its own, only children.
    pub fn group() -> Self;
    /// A full style (replaces the inherited one; the overrides below still apply on top).
    pub fn style(self, style: TextStyle) -> Self;
    pub fn size(self, size: f32) -> Self;
    pub fn weight(self, weight: f32) -> Self;
    pub fn bold(self) -> Self;        // weight 700
    pub fn italic(self, italic: bool) -> Self;
    pub fn family(self, family: FontFamily) -> Self;
    pub fn color(self, color: Color) -> Self;
    pub fn child(self, child: TextSpan) -> Self;
    pub fn children(self, children: impl IntoIterator<Item = TextSpan>) -> Self;
    /// The concatenated text of the span and its children, in order.
    pub fn to_plain_text(&self) -> String;
}
impl IntoProp<TextSpan> for TextSpan {}

/// A paragraph of styled spans (Flutter's `RichText`).
pub struct RichText { /* span, style, color, max_lines, soft_wrap */ }

impl RichText {
    /// `span` on `TextStyle::body()` in on-surface, wrapping.
    pub fn new(span: impl IntoProp<TextSpan>) -> Self;
    /// The base style (the root span inherits it).
    pub fn style(self, style: TextStyle) -> Self;
    /// The base color.
    pub fn color(self, color: impl IntoProp<Color>) -> Self;
    pub fn max_lines(self, max_lines: u32) -> Self;
    pub fn soft_wrap(self, soft_wrap: bool) -> Self;
}
```

Flattening: the tree is walked depth first; each span's effective style is its parent's
effective style, replaced by `style` if set, then with `size`, `weight`, `italic` and
`family` overrides applied; its effective color is its own `color` or the parent's (the root's
parent is the paragraph's base style and base color). Each span's own `text` (before its
children) becomes one `StyleSpan` over its byte range with the effective style's key and the
effective color (`None` when it is the base color and the style is the base style's key, so
plain spans add nothing). The paragraph's text is the concatenation.

## Behavior

- **LAYOUT-TEXT-11:** `measure_rich` and `min_intrinsic_width_rich` default to the plain
  methods (`NoTextMeasure` measures rich text as zero, like plain). `RenderParagraph::new`
  has no spans; it lays out, computes intrinsics and paints with `measure_rich` /
  `min_intrinsic_width_rich` and its `spans`, so a plain paragraph behaves as before and a
  measurer that handles spans sees them.
- **LAYOUT-TEXT-12:** `MeasureCache` caches rich results like plain ones, keyed by the text,
  style, width, line limit *and* the spans: the same text with different spans is measured
  separately, and a repeat is a hit. The one-line shortcut (LAYOUT-TEXT-03) applies per spans.
- **TEXT-SYS-10:** `measure_rich` lays the text out with each span's style (size, family stack,
  weight, italic, line height) applied to its range, so a span in a larger size makes its
  line taller and the text wider than the same text measured plain, and a span whose range is
  empty, beyond the text or off a character boundary changes nothing. Plain `measure` equals
  `measure_rich` with no spans. Results are cached per (text, style, spans).
- **TEXT-SYS-11:** `paint_rich` emits one glyph run per parley run, so a span with a color
  gives runs in that color and the rest in the paragraph's color; glyph counts across the runs
  equal the plain paint's, and a span's glyphs are positioned as `measure_rich` laid them out.
- **VIEW-TEXT-07:** `TextPainter::paint_rich` defaults to `paint_text`; `SystemText` forwards to
  `TextSystem::paint_rich`; `PaintCx::paint_rich` forwards to the pass's painter; and
  `ParagraphPaint` paints the paragraph's spans, so a `RenderParagraph` with spans paints
  its colored runs through a `SystemText`.
- **VIEW-STYLES-03:** `BuildCx::text_styles` returns the tree's table: keys made through it
  resolve in the tree, as `text_style` does. (Added to [shared text styles](../text/styles.md).)
- **WIDGETS-RICHTEXT-01:** `TextSpan` flattens as described: `to_plain_text` concatenates;
  the `StyleSpan`s cover each span's own text with the effective style key (resolved through
  the tree's style table) and color, inheriting through the tree, and spans with the base
  style and no color add no `StyleSpan`.
- **WIDGETS-RICHTEXT-02:** `RichText` builds a render element with a `RenderParagraph` of the
  flattened text, base style key, `max_lines`, `soft_wrap` and the spans, painted with a
  `ParagraphPaint` in the base color; `Finder::text` finds it by its plain text. Checked
  against a golden (`crates/tantu-widgets/tests/goldens/rich-text.png`).
- **WIDGETS-RICHTEXT-03:** `span` and `color` are reactive: a new span re-measures (new text
  and spans), a new color only repaints.

## Open questions

Decided by the agent while the user was away (review):

1. **Spans as a parallel list, not a new text model**: `measure_rich`/`paint_rich` with
   defaults keep every existing measurer and painter valid; a flattened `&[StyleSpan]` is
   cheap to hash and pass. A richer `Paragraph` value type can come with `TextField`.
2. **Colors ride as parley brushes** (`Layout<Option<Color>>` inside the text system), so
   runs split by color for free; underline and strikethrough use the same mechanism later.
3. **`TextSpan` overrides instead of Flutter's partial `TextStyle.merge`**: `TextStyle` has no
   optional fields, so a span sets a full style or individual overrides (`size`, `bold`, …).
4. **The cache key includes the spans**, so rich paragraphs get the same two-generation cache;
   the text system's layout cache likewise.
