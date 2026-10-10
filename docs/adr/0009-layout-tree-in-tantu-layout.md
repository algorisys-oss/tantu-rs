# 0009. The layout tree lives in `tantu-layout`

- **Status:** Accepted (agreed with the user, 2026-10-10)
- **Date:** 2026-10-10
- **Related:** [ADR 0001](0001-retained-tree-and-fine-grained-reactivity.md) (retained tree),
  [ADR 0002](0002-flutter-structure-and-layout-protocol.md) (Flutter's layout protocol)

## Context

ADR 0002 fixes the protocol: constraints go down, sizes come up, the parent sets offsets, with
relayout boundaries and opt-in intrinsic sizes, and custom layouts implement the same
`RenderBox`-style trait as the built-in ones. It doesn't say where the tree that runs this
protocol lives.

In Flutter one object, the `RenderObject`, does layout, paint and hit-testing, and it lives in
the rendering library. In Tantu, `tantu-layout` sits below `tantu-view` and knows nothing of
views, painting or events (AGENTS.md dependency rule). Layout still needs a tree to run on:
algorithms lay out children, read their parent data (a flex factor, a `Positioned` rect),
cache results by constraints, and stop dirtiness at relayout boundaries. Phase 2 also has a
benchmark target (10k render objects, full layout under 1 ms) that should be measurable without
building views.

## Decision

We will put the layout half of the render tree in `tantu-layout`:

- A `LayoutTree` is an arena of layout nodes, addressed by `LayoutId` (a newtype of
  `tantu_core::Id`). A node holds its layout object (`Box<dyn RenderBox>`), its ordered children,
  its parent data, and the results of the last layout: constraints, size and offset in the
  parent.
- `RenderBox` is the trait every layout implements, built-in or custom: lay out the children
  through a context and return a size, plus optional intrinsic-size methods. It never sees the
  tree directly, only its own children through the context.
- The tree runs the layout pass: it skips nodes whose constraints didn't change and that aren't
  dirty, marks dirtiness up to the nearest relayout boundary, and computes intrinsic sizes only
  when asked.
- Built-in layouts use Flutter's render-object names (`RenderPadding`, `RenderFlex`,
  `RenderStack`, …). They are not widgets: `tantu-widgets` wraps them in views (`Padding`, `Row`,
  `Stack`, …).
- `tantu-view` owns one `LayoutTree` per window. When it reconciles elements it creates, updates
  and removes layout nodes, then runs layout and paints and hit-tests from the computed geometry.
  Paint and hit-testing stay in `tantu-view`.

## Consequences

- Every layout algorithm, the caching and the relayout boundaries are tested in `tantu-layout`
  with plain unit tests, and the layout benchmark runs there.
- Custom layouts implement one small trait and work like built-in ones.
- The render tree is split across two crates: geometry and layout state in `tantu-layout`,
  paint and hit-test state in `tantu-view` keyed by `LayoutId`. `tantu-view` keeps the two in
  step; that is its job anyway (it already keeps elements and layout nodes in step).
- `LayoutBuilder` builds views during layout, so it belongs to `tantu-view`; `tantu-layout`
  only needs a hook that lets the owner run code with a node's constraints before its children
  are laid out.
- Layout is single-threaded per tree, like the rest of a window's frame.

## Alternatives considered

- **Render objects in `tantu-view`, algorithms as free functions in `tantu-layout`.** The
  algorithms would need an abstract child interface anyway, caching and relayout boundaries would
  be in `tantu-view` and only testable with views, and the benchmark would need the element tree.
- **Use taffy as the layout engine.** It brings CSS semantics (flexbox, grid) that ADR 0002
  rejected as the primary model. It may still back a dashboard `Grid` layout in Phase 4, as one
  `RenderBox`.
- **Flutter's single `RenderObject` with layout, paint and hit-test** in one crate. It would
  have to live in `tantu-view` (paint needs the Scene, hit-test needs events), which brings back
  the first alternative's problems.
