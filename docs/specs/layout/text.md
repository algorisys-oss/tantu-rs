# Text measurement and paragraphs

- **Status:** Agreed
- **Crate:** `tantu-layout`
- **Plan item:** Phase 2, "`TextMeasure`, `MeasureCache` and `RenderParagraph`"
- **Related:** [ADR 0010](../../adr/0010-text-measurement-in-layout.md),
  [ADR 0002](../../adr/0002-flutter-structure-and-layout-protocol.md),
  [layout tree](tree.md) (amended here: the text context)

## Purpose

How layout sizes text without depending on the text crate (ADR 0010). `TextMeasure` is the
trait `tantu-text` will implement over parley: measure a paragraph at a width. `MeasureCache`
wraps any measurer so repeated text (grid cells, list rows, labels) is measured once.
`RenderParagraph` is the layout object behind `Text`: it sizes itself through the measurer that
the layout pass carries. Glyphs are produced later, at paint time, by `tantu-text`.

## Scope

In scope:

- `TextStyleKey`, `TextMetrics`, the `TextMeasure` trait, `NoTextMeasure`.
- `MeasureCache`: paragraph and intrinsic-width caching, the one-line shortcut, bounded size.
- The text context of the layout pass: `LayoutTree::with_text` and `LayoutSession`,
  `LayoutChildren::text`, `IntrinsicChildren::text` (an amendment to the layout-tree API).
- `TextWidthBasis` and `RenderParagraph` (layout and intrinsics).

Out of scope (and where it goes):

- Shaping, fonts, styles, bidi, line breaking, the word cache: `tantu-text` (its own Phase 2
  item), which implements `TextMeasure` and assigns `TextStyleKey`s.
- Text alignment, overflow (clip, ellipsis, fade), selection, painting: `tantu-view` and
  `tantu-text`. They don't change a paragraph's size with the default width basis.
- Rich text (spans with several styles): Phase 3 `RichText`; it will measure through the same
  trait with a span list.
- Baseline alignment in rows: later (the metrics already carry baselines).

## Public API

Crate root `tantu_layout`.

```rust
use std::sync::Arc;
use tantu_core::Size;

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
    fn measure(&mut self, text: &str, style: TextStyleKey, max_width: f32, max_lines: Option<u32>) -> TextMetrics;
    /// The width of the widest unbreakable piece of `text` (its minimum intrinsic width).
    fn min_intrinsic_width(&mut self, text: &str, style: TextStyleKey) -> f32;
}

/// A measurer with no text system: every text measures as empty (zero metrics).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoTextMeasure;
impl TextMeasure for NoTextMeasure { /* zeros */ }

/// Caches another measurer's results (ADR 0010).
pub struct MeasureCache<M> { /* private */ }

impl<M: TextMeasure> MeasureCache<M> {
    /// A cache in front of `inner` holding up to about `capacity` paragraph results.
    pub fn new(inner: M, capacity: usize) -> Self;
    /// The wrapped measurer.
    pub fn inner(&self) -> &M;
    /// The wrapped measurer, for changing it; call `clear` if its results change.
    pub fn inner_mut(&mut self) -> &mut M;
    /// Forgets every cached result (fonts or styles changed).
    pub fn clear(&mut self);
    /// Number of cached paragraph results.
    pub fn len(&self) -> usize;
    /// True with nothing cached.
    pub fn is_empty(&self) -> bool;
    /// Lookups answered from the cache, and lookups passed to the inner measurer, since
    /// creation.
    pub fn hits(&self) -> u64;
    pub fn misses(&self) -> u64;
}
impl<M: TextMeasure> TextMeasure for MeasureCache<M> { /* ... */ }

// Additions to the layout tree (docs/specs/layout/tree.md):

impl LayoutTree {
    /// A session that runs layout passes and intrinsic queries with `text` as the text
    /// context. `LayoutTree::layout` and the tree's intrinsic methods use `NoTextMeasure`.
    pub fn with_text<'a>(&'a mut self, text: &'a mut dyn TextMeasure) -> LayoutSession<'a>;
}

/// A layout tree borrowed together with a text measurer.
pub struct LayoutSession<'a> { /* private */ }
impl LayoutSession<'_> {
    /// As `LayoutTree::layout`, with the session's measurer.
    pub fn layout(&mut self, root: LayoutId, constraints: BoxConstraints) -> Size;
    /// As the tree's intrinsic methods, with the session's measurer.
    pub fn min_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32;
    pub fn max_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32;
    pub fn min_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32;
    pub fn max_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32;
}

impl LayoutChildren<'_> {
    /// The pass's text measurer.
    pub fn text(&mut self) -> &mut dyn TextMeasure;
}
impl IntrinsicChildren<'_> {
    /// The query's text measurer.
    pub fn text(&mut self) -> &mut dyn TextMeasure;
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

/// Lays out a paragraph of text in one style (Flutter's `RenderParagraph`, behind `Text`).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderParagraph {
    pub text: Arc<str>,
    pub style: TextStyleKey,
    /// Wrap at the incoming maximum width (`true`), or only at hard line breaks.
    pub soft_wrap: bool,
    /// Keep at most this many lines.
    pub max_lines: Option<u32>,
    pub text_width_basis: TextWidthBasis,
}
impl RenderParagraph {
    /// `text` in `style`, wrapping, no line limit, `Parent` width basis.
    pub fn new(text: impl Into<Arc<str>>, style: TextStyleKey) -> Self;
}
```

## Behavior

Tests use a fake measurer: each character is 10 wide, lines are 20 tall (baseline 16),
words break at spaces, `\n` is a hard break, and it counts its calls.

### Types and the plain measurer

- **LAYOUT-TEXT-01:** `TextStyleKey` and `TextMetrics` are plain data (`TextMetrics::default()`
  is all zeros). `NoTextMeasure` returns `TextMetrics::default()` and a minimum intrinsic
  width of 0 for every input.

### The cache

- **LAYOUT-TEXT-02:** `MeasureCache::measure` returns exactly what the inner measurer returns
  for the same arguments. The first call for a key is a miss (passed to the inner measurer);
  a repeated call is a hit and doesn't call it. The key is the exact text (compared in full,
  so hash collisions can't return wrong results), the style, the bits of `max_width` and
  `max_lines`.
- **LAYOUT-TEXT-03:** One-line shortcut: when the cache holds the single-line result of a text
  and style (`max_width = ∞`, `max_lines = None`), a call with a finite `max_width` at least as
  wide as that result, and `max_lines` either `None` or at least its line count, returns the
  single-line result as a hit without calling the inner measurer. (Wrapping can't change
  lines that already fit.)
- **LAYOUT-TEXT-04:** `MeasureCache::min_intrinsic_width` is cached per text and style the same
  way.
- **LAYOUT-TEXT-05:** Bounded size: the cache keeps two generations of up to `capacity / 2`
  results each; when the current generation is full it becomes the old one and the previous
  old one is dropped. A hit in the old generation moves the entry to the current one. So
  `len() ≤ capacity` (rounded up to an even number), recently used entries survive, and
  `capacity 0` caches nothing. `clear` empties both generations; `hits`/`misses` count calls
  and are not reset by `clear`.

### The text context

- **LAYOUT-TEXT-06:** In a pass run through `tree.with_text(m)`, `children.text()` inside
  `perform_layout` and the intrinsic methods is `m` (calls reach it). In `LayoutTree::layout`
  and the tree's own intrinsic methods it is a `NoTextMeasure`. The session's `layout` and
  intrinsic methods otherwise behave exactly as the tree's (LAYOUT-TREE rules).

### RenderParagraph

- **LAYOUT-TEXT-07:** `RenderParagraph::new(t, s)` has `soft_wrap: true`, `max_lines: None` and
  `TextWidthBasis::Parent`. Equality compares all fields (text by content).
- **LAYOUT-TEXT-08:** Layout measures `text` with the incoming `max_width` when `soft_wrap` (`∞`
  when unbounded) or `∞` otherwise, and `max_lines`. The height is the measured height. The
  width is, for `Parent`, the single-line width capped at the incoming maximum (the text's
  natural width if it fits, the full maximum if it wraps), and for `LongestLine`, the measured
  longest line. The size is then constrained by the tree. A paragraph has no children (any are
  not laid out).
- **LAYOUT-TEXT-09:** Intrinsics: min intrinsic width is the measurer's `min_intrinsic_width`;
  max intrinsic width is the single-line width; both intrinsic heights at a width are the
  measured height at that width (or at `∞` without `soft_wrap`), with `max_lines`.
- **LAYOUT-TEXT-10:** With `NoTextMeasure` (a plain `LayoutTree::layout`), a paragraph is
  `constraints.smallest()`. Nothing panics for any constraints or any metrics a measurer
  returns (NaN, negative, infinite: the tree constrains sizes and sanitizes intrinsics).

## Performance and allocation

- A cache hit hashes the text once and compares it once; it doesn't allocate. A miss allocates
  one copy of the text for the key.
- Laying out a `RenderParagraph` calls the measurer at most twice (the wrapped measure and, for
  `Parent`, the single-line one), both normally cache hits after the first frame.

## Open questions

Resolved (2026-10-10, agreed with the user): the three ADR 0010 choices, and:

1. **Two-generation eviction** instead of exact LRU: O(1), no per-entry bookkeeping, and close to
   LRU for "same cells every frame" workloads.
2. **`max_lines` in the measure call** (it changes height and line count, so it is part of the
   key); ellipsis and overflow stay at paint time.
3. **`TextStyleKey` as a public `u64` newtype**, assigned by `tantu-text` (like `ImageId` in the
   Scene).
