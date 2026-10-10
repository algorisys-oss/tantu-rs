# Bidi and font fallback

- **Status:** Draft
- **Crate:** `tantu-text`
- **Plan item:** Phase 2, "`tantu-text`: shaping, line breaking, bidi, font fallback, glyph-run
  output into Scene" (the parent item; its sub-items are done)
- **Related:** [text system](system.md) (TEXT-SYS-03, the family-level fallback this extends),
  [ADR 0005](../../adr/0005-text-stack-parley-swash-fontique.md)

## Purpose

The PLAN.md item promises bidi and font fallback, and no rule covered either. Checking them
(2026-10-10) found:

- **Bidi already works.** parley reorders right-to-left runs. "ab שלום cd" in Liberation Sans
  comes out as visual runs: "ab ", the Hebrew run (glyphs in visual order), " cd". It only
  needs rules and tests.
- **Per-character fallback doesn't work** without system fonts. `TextSystem` asks parley for a
  two-family stack (the style's family, then the default family). A character missing from
  both is drawn as glyph 0 (the "missing glyph" box), even when another registered font has
  it. For example, Arabic stays boxes with Noto Sans Arabic registered next to Liberation
  Sans. A bundled-font app (`App::without_system_fonts`) can't mix scripts across its fonts.

## Scope

In scope:

- Rules for right-to-left and mixed-direction paragraphs (base direction from the first strong
  character, the Unicode default).
- Per-character fallback through every registered family. The family stack becomes: the style's
  family, the default family, then the other registered families in registration order, then
  the system's fallback when system fonts are on.
- A second test font: Noto Sans Hebrew Regular (27 KB, SIL OFL 1.1), committed with its license.
  It has Hebrew but no Latin, so with it as the default family, Latin text has to fall back
  to Liberation Sans.

Out of scope:

- An explicit paragraph direction and RTL layout mirroring (`TextDirection`,
  `Directionality`): Phase 4 "i18n: RTL layout mirroring".
- Cursor movement and selection in bidi text: `TextField`, Phase 3.
- Emoji and color fonts: later, with color glyph rendering.

## Public API

No new API. The behavior of `TextSystem::measure`, `min_intrinsic_width` and `paint` changes
for text whose characters the style's family lacks.

## Behavior (proposal)

- **TEXT-SCRIPT-01:** A right-to-left paragraph is shaped in visual order: glyph x positions
  increase along the run. The first glyph drawn is the paragraph's last character, so for
  "שלום" in Liberation Sans, the first glyph is the glyph of "ם" alone.
- **TEXT-SCRIPT-02:** A mixed paragraph whose first strong character is left-to-right ("ab
  שלום cd") keeps its left-to-right runs in logical order and draws the right-to-left run
  between them, in visual order. Its measured width equals the sum of the runs' advances.
- **TEXT-SCRIPT-03:** A character the style's family lacks is drawn from the default family,
  else from the first other registered family (in registration order) that has it. With Noto
  Sans Hebrew as the default family and Liberation Sans registered, "abc" is drawn with
  Liberation Sans's glyphs (its font in the glyph run), "שלום" with Noto Sans Hebrew's, and
  "ab שלום" uses both fonts.
- **TEXT-SCRIPT-04:** A character no font has is still drawn as the default family's glyph 0 and
  measured with its advance, so a paragraph never measures as empty because of missing glyphs.

## Open questions (for the user)

1. **Fallback order:** the style's family, the default family, the other registered families
   in registration order, then system fallback. Proposal: yes. It is predictable, and an app
   controls it by the order it registers fonts in.
2. **A second test font** (Noto Sans Hebrew, 27 KB, OFL 1.1) in `crates/tantu-text/tests/fonts`.
   Proposal: yes, test-only.
3. **CJK and Thai segmentation:** parley's `complex-scripts` feature adds ICU4X segmentation
   models for scripts that don't separate words with spaces. Without it, a debug build printed
   "ICU4X data error: No segmentation model for complex script: Chinese/Japanese" to stderr
   when shaping Japanese. A quick release-build comparison showed no size change and no
   message, which isn't conclusive. Proposal: leave it out of this spec. Enabling it, behind a
   `tantu-text` feature or by default, is decided with i18n work in Phase 4, after measuring
   its size and checking where the message comes from.
