# 0005. Text stack: parley, swash and fontique

- **Status:** Accepted
- **Date:** 2026-09-30

## Context

Text is where most UI toolkits fall short, and Tantu's 0.1 success criteria require full IME
(CJK) and RTL text in `TextField`. That means complex-script shaping, bidirectional text, line
breaking, font fallback across scripts, and rich text with mixed styles. Knots only does simple
text, which is not enough.

Layout must not depend on the text implementation (ADR 0002): it measures text through the
`TextMeasure` trait.

## Decision

`tantu-text` is built on the Linebender text stack:

- **parley** for text layout: shaping, bidi, line breaking and rich-text styles.
- **fontique** for finding system fonts and choosing fallback fonts.
- **swash** for glyph outlines and rasterization.

`tantu-text` implements `TextMeasure` for `tantu-layout`, gives shaped paragraphs to text render
objects, and those emit glyph runs into the Scene. Text work starts early in Phase 2, not at the
end.

## Consequences

- The same stack is used by Xilem/Masonry and Vello, so we share fixes and improvements with
  other Rust UI projects.
- These crates are pre-1.0 and their APIs change. We pin versions and keep them behind
  `tantu-text`, so churn stays inside one crate.
- Every renderer needs glyphs: the GPU backend needs a glyph atlas, and the software backend
  draws glyphs on the CPU. Both get them through `tantu-text`/swash.
- System fonts come from fontique, so apps don't have to bundle fonts, but golden tests need a
  fixed bundled test font to be reproducible across machines.

## Alternatives considered

- **cosmic-text** (used by Iced through glyphon). Capable and widely used, but parley's rich-text
  model and its connection to the Linebender ecosystem fit better.
- **Platform text APIs** (DirectWrite, Core Text, Pango). Native look, but three
  implementations to keep consistent, and golden tests would differ per OS.
- **Our own layout on top of a shaper alone.** Bidi, line breaking and fallback are exactly the
  hard parts; we would be rebuilding parley.
