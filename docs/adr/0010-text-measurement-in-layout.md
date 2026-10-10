# 0010. Text measurement in layout

- **Status:** Accepted (the three choices below were made by the user, 2026-10-10)
- **Date:** 2026-10-10
- **Related:** [ADR 0002](0002-flutter-structure-and-layout-protocol.md) (layout measures text
  only through `TextMeasure`), [ADR 0005](0005-text-stack-parley-swash-fontique.md) (parley,
  swash, fontique), [ADR 0009](0009-layout-tree-in-tantu-layout.md) (layout tree)

## Context

ADR 0002 says `tantu-layout` measures text only through a `TextMeasure` trait, so it never
depends on `tantu-text`. `PLAN.md` borrowed Clay's technique for it: a measure cache keyed by
(font, size, word), with layout breaking lines from cached word widths. Three things were left
open: what the trait measures, how text layout objects reach the measurer during a layout pass,
and where the paragraph layout object lives.

Breaking lines from word widths is only right for simple scripts. Bidirectional text, CJK
(which breaks between characters), shaping and kerning across spaces, and hyphenation all need
the shaper's line breaker. ADR 0005 chose parley, which shapes and breaks whole paragraphs.

## Decision

1. **`TextMeasure` is paragraph-level.** It measures a whole string in a style at a maximum
   width (and an optional line limit) and returns metrics (size, line count, baselines), plus
   the minimum intrinsic width. `tantu-layout` caches whole results in a `MeasureCache`, keyed
   by the exact text, style, width and line limit, with a shortcut when the text fits on one
   line. The word-level cache moves into `tantu-text`, where shaping happens, as an
   optimization of its own.
2. **The measurer is passed into the layout pass.** `LayoutTree` runs a pass (and intrinsic
   queries) with a `&mut dyn TextMeasure` context that layouts read through `LayoutChildren`
   and `IntrinsicChildren`. A plain pass uses a measurer that measures everything as empty.
   There is no shared global measurer and no handle stored in nodes; `tantu-view` passes its
   window's measurer.
3. **`RenderParagraph` lives in `tantu-layout` and only measures.** It sizes itself through the
   context's `TextMeasure` and keeps no glyphs. `tantu-text` shapes the text again at paint
   time (hitting its own caches) and `tantu-view` emits the glyph runs. Styles are opaque
   `TextStyleKey` handles assigned by `tantu-text`.

## Consequences

- Text wraps correctly for every script, because parley breaks the lines.
- Grids and lists still avoid re-measuring: identical cells hit the paragraph cache, and text
  shorter than its column skips wrapping entirely through the one-line shortcut.
- Text layout is tested in `tantu-layout` with a fake measurer, like every other layout.
- The layout-tree API grows a text context (`LayoutTree::with_text`), and `LayoutChildren` /
  `IntrinsicChildren` expose it.
- When fonts or styles change, the owner (`tantu-view`) must clear the cache and mark text
  nodes as needing layout; the tree can't see font changes.
- The Clay row in `PLAN.md` changes from "word-level measure cache" to this split.

## Alternatives considered

- **Clay's word cache with line breaking in layout:** wrong for bidi, CJK and cross-word shaping
  (see Context).
- **Each text node holding a shared measurer handle** (`Rc<RefCell<dyn TextMeasure>>`): hidden
  shared mutable state in every text node, and harder to swap per window.
- **Text layout objects in `tantu-view`:** text layout would only be testable with views, unlike
  every other layout (ADR 0009).
