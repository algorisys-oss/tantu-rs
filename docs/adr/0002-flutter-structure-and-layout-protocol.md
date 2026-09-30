# 0002. Flutter structure and layout protocol

- **Status:** Accepted
- **Date:** 2026-09-30

## Context

The layout model is the part of a UI framework that app developers touch most, and it is the
hardest to change later. It has to express any layout, from a toolbar to a docking manager, and
it has to be fast enough for large, frequently changing screens.

Options on the table were Flutter's box protocol, CSS flexbox/grid (for example via taffy), Clay's
per-container sizing modes (`Fit`/`Grow`/`Fixed`/`Percent`) and a constraint solver. We briefly
adopted Clay's sizing vocabulary (commit `584bb1d`) and reverted it (`c02ae26`): having two layout
vocabularies side by side made the API harder to learn and gave no layout that Flutter's protocol
can't express.

## Decision

The user-facing structure is **Flutter's**:

- Apps compose widgets. Layout is done with layout widgets that wrap other widgets: `Row`,
  `Column`, `Expanded`, `Flexible`, `Spacer`, `Padding`, `Align`/`Center`, `SizedBox`,
  `ConstrainedBox`, `FractionallySizedBox`, `AspectRatio`, `Stack`/`Positioned`, `Wrap`,
  `LayoutBuilder`, and so on.
- Names and semantics follow Flutter, in snake_case (`main_axis_alignment`, `cross_axis_alignment`,
  `main_axis_size`, `flex`, …). Someone who knows Flutter should be able to read Tantu code.
- Layout uses Flutter's protocol: the parent passes `BoxConstraints` down, the child returns a
  `Size`, and the parent sets the child's offset.
- **Relayout boundaries** stop a change from spreading up the tree.
- **Intrinsic-size queries** exist but are opt-in. They can be expensive, so no standard layout
  depends on them.
- Custom layouts implement the same `RenderBox`-style trait as the built-in ones.
- `tantu-layout` measures text only through a `TextMeasure` trait, so it does not depend on
  `tantu-text`.

Clay influences only the implementation (see `PLAN.md`): a word-level text measure cache,
culling, arena storage and no allocation per frame. None of it shows up in the API.

## Consequences

- Flutter developers can read and write Tantu layouts straight away, and Flutter's documentation
  on layout mostly applies.
- Layout is a single pass over the tree in the common case, which keeps the 10k-object target
  (< 1 ms full layout, Phase 2) realistic.
- Some layouts, such as "make every child as wide as the widest one", need intrinsic queries or a
  `LayoutBuilder`, just as in Flutter.
- Every new layout feature must be expressed as a widget in this protocol. A dashboard `Grid`
  (Phase 4) may use taffy internally, but it will be exposed as a layout widget, not as a second
  layout model.

## Alternatives considered

- **Clay's sizing modes on every container.** Tried and reverted (see Context).
- **CSS flexbox/grid as the primary model** (taffy). Well known on the web, but its rules
  (min-content sizing, `flex-basis`, margin collapsing in some modes) are harder to predict than
  box constraints, and mixing it with Flutter-style widgets would give two models again.
- **A constraint solver** (Cassowary-style). Very expressive, but slower, harder to debug when
  constraints conflict, and unfamiliar to most app developers.
