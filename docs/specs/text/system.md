# Text system

- **Status:** Implemented (decided by the agent while the user was away, at their request; review)
- **Crate:** `tantu-text`
- **Plan item:** Phase 2, `tantu-text` → "Text system"
- **Related:** [ADR 0005](../../adr/0005-text-stack-parley-swash-fontique.md) (parley, swash,
  fontique), [ADR 0010](../../adr/0010-text-measurement-in-layout.md) (`TextMeasure`,
  `TextStyleKey`), [text measurement](../layout/text.md), [Scene](../scene/scene.md) (glyph runs)

## Purpose

The real text stack behind ADR 0010's `TextMeasure`: a `TextSystem` owns the fonts (registered
font files, and the system's fonts through fontique), turns text styles into `TextStyleKey`s,
measures paragraphs with parley (shaping, bidi, line breaking, font fallback), and paints them
into a Scene as glyph runs whose fonts are registered in the renderer `Resources`. Rasterizing
the glyphs is the renderers' job (a later item).

## Scope

In scope:

- `TextSystem`: system fonts or none, registering fonts, the default family, style interning,
  `TextMeasure`, painting glyph runs.
- `TextStyle`, `FontFamily`.

Out of scope (and where it goes):

- Text in views (frames that measure and paint text, the paragraph's paint): the "text in views"
  item.
- Glyph rasterization in renderers: the "glyph rasterization" item.
- Rich text (several styles per paragraph), text editing, selection, IME: Phase 3.
- Text alignment within the paragraph width (start only for now) and ellipsis: Phase 3 `Text`.

## Dependencies

- `parley` 0.12 (Apache-2.0 OR MIT), default features (`system`, through fontique), which needs
  Rust 1.88 (the MSRV was raised for it). parley brings fontique, swash/skrifa and harfrust for
  shaping.
- `tantu-layout` for `TextMeasure`, `TextMetrics` and `TextStyleKey`: a new edge in the AGENTS.md
  table (`tantu-text` may depend on `layout`); `tantu-layout` doesn't depend on `tantu-text`, so
  there's no cycle.
- Tests use Liberation Sans Regular (SIL Open Font License 1.1), committed with its license in
  `crates/tantu-text/tests/fonts/`, so results don't depend on the machine's fonts.

## Public API

Crate root `tantu_text`.

```rust
use std::sync::Arc;
use tantu_core::{Color, Point};
use tantu_layout::{TextMeasure, TextMetrics, TextStyleKey};
use tantu_scene::{Resources, SceneBuilder};

/// A font family: a name, or a generic family resolved by the font system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontFamily { Named(Arc<str>), SansSerif, Serif, Monospace }

/// A text style. Sizes are logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub family: FontFamily,
    /// Font size; non-finite or not positive counts as 16.
    pub size: f32,
    /// Weight, 100 (thin) to 900 (black); 400 is regular. Clamped to 1..=1000.
    pub weight: f32,
    pub italic: bool,
    /// Line height as a multiple of the size; `None` uses the font's metrics.
    pub line_height: Option<f32>,
}
impl Default for TextStyle { /* SansSerif, 16, 400, upright, font metrics */ }

/// Fonts, styles, shaping and line breaking for one app.
pub struct TextSystem { /* private */ }

impl TextSystem {
    /// A text system that can use the fonts installed on the system.
    pub fn new() -> Self;
    /// A text system with no fonts until some are registered (tests, embedded apps).
    pub fn without_system_fonts() -> Self;
    /// Registers a font file (TTF, OTF or a collection) and returns the family names it added
    /// (empty, and nothing registered, if the data isn't a font).
    pub fn register_font(&mut self, data: Vec<u8>) -> Vec<String>;
    /// The family used when a style's family has no matching font (default: `SansSerif`).
    pub fn set_default_family(&mut self, family: FontFamily);
    /// The key for `style`: equal styles (after sanitizing) get the same key.
    pub fn style(&mut self, style: TextStyle) -> TextStyleKey;
    /// The style behind `key`.
    pub fn text_style(&self, key: TextStyleKey) -> Option<&TextStyle>;
    /// Paints `text` laid out as `measure` lays it out, with the first line's top-left at
    /// `origin`, as glyph runs in `color`; fonts are added to `resources` the first time they
    /// are used (and reused after).
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
    );
}

impl TextMeasure for TextSystem { /* parley */ }
```

## Behavior

Tests register Liberation Sans in a `without_system_fonts` system and set it as the default
family.

### Fonts and styles

- **TEXT-SYS-01:** `without_system_fonts()` measures text as empty (zero width) until a font is
  registered. `register_font` with Liberation Sans returns `["Liberation Sans"]`; with bytes that
  aren't a font it returns an empty list and changes nothing. Nothing panics.
- **TEXT-SYS-02:** `style` interns: equal styles return the same key, different styles different
  keys; `text_style(key)` returns the (sanitized) style; an unknown key returns `None`.
  Sanitizing: a non-finite or non-positive size becomes 16, a weight is clamped to 1..=1000
  (NaN becomes 400), a non-finite or non-positive line height becomes `None`.
- **TEXT-SYS-03:** Fallback: a style whose family has no font is measured with the default family
  (so a style naming a missing family, or a generic family with no system fonts, still measures
  with the registered font). An unknown style key is measured with `TextStyle::default()`.

### Measuring

- **TEXT-SYS-04:** `measure(text, style, max_width, max_lines)` lays out `text` with parley,
  breaking lines at `max_width` (see TEXT-SYS-08 for infinite, NaN and non-positive widths)
  and at hard breaks (`\n`). The result: `size.width` is the widest line, `size.height` the sum of the
  kept lines' heights, `line_count` the number of kept lines, `first_baseline` / `last_baseline`
  the first and last kept lines' baselines from the top. Empty text gives `TextMetrics::default()`.
- **TEXT-SYS-05:** Lines: a longer text measures at least as wide on one line; wrapping at a
  width narrower than the one-line width gives more lines, each no wider than the width unless a
  single word is wider; `\n` always starts a new line. With one style, every line has the same
  height, and `last_baseline - first_baseline` is `(line_count - 1)` line heights.
- **TEXT-SYS-06:** `max_lines = Some(n)` keeps the first `n` lines (`line_count ≤ n`, the height
  and last baseline those of line `n`); `None` keeps all.
- **TEXT-SYS-07:** `min_intrinsic_width` is the width of the widest unbreakable piece (for
  Latin text, the widest word), within 0.5 of measuring that word alone.
- **TEXT-SYS-08:** A `max_width` that is NaN or infinite means no wrapping; zero or negative wraps
  at every break opportunity (each word on its own line). Nothing panics for any input.

### Painting

- **TEXT-SYS-09:** `paint` records one or more glyph runs whose glyphs are the shaped glyphs of
  the kept lines (for "Hello" in Liberation Sans, five glyphs), positioned relative to the run
  origin so that the first line's top is at `origin.y` and its baseline at `origin.y +
  first_baseline`, with `color` and the style's size. The run's `FontId` names a font in
  `resources` holding the font file; painting again with the same font reuses the same `FontId`.
  Empty text records nothing; lines cut by `max_lines` record no glyphs.

## Performance and allocation

The system keeps parley's font and layout contexts for reuse. It caches shaped layouts by
(text, style) (two generations, like `MeasureCache`), so a paint after a measure, or a measure
at another width, re-breaks lines without shaping again. Allocation on cache misses is
accepted; the word-level cache ADR 0010 mentions is this layout cache.

## Open questions

Resolved (2026-10-10, decided by the agent while the user was away; review):

1. **`tantu-text` depends on `tantu-layout`** for the `TextMeasure` trait (ADR 0010 says
   `tantu-text` implements it); AGENTS.md's table gains `layout` in the `tantu-text` row.
2. **MSRV 1.88** for parley (raised in `chore:` commit `533335a`).
3. **A committed test font** (Liberation Sans, OFL) instead of relying on system fonts in CI.
4. **A default-family fallback** configured on the system, so apps can ship one font and use it
   for every generic family.
