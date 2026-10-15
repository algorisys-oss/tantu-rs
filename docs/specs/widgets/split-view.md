# SplitView: resizable panes

- **Status:** Implemented (2026-10-15; agreed the same day, the user said "continue" on the seven proposals)
- **Crates:** `tantu-layout` (`RenderSplit`), `tantu-widgets` (`SplitView`, `SplitPane`),
  `tantu-view` and `tantu-a11y` (the splitter role), `examples/gallery`
- **Plan item:** Phase 4, "`SplitView`", part 2 (the layout and the widget)
- **Related:** [cursor icons](../platform/cursor.md) (part 1), [flex](../layout/flex.md),
  [layout tree](../layout/tree.md), [events](../view/events.md), [focus](../view/focus.md),
  [semantics](../a11y/semantics.md), [DataGrid columns](data-grid-columns.md),
  [gallery](../examples/gallery.md), [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Mail clients, IDEs, admin consoles and dashboards divide a window into panes the user can
resize: a navigation sidebar, a list, a detail view, an output panel below. `SplitView`
lays out two or more panes along one axis with draggable dividers between them; nesting a
vertical split inside a horizontal one gives the usual 2-D arrangements. Pane sizes live in
a signal the app can share, so it can persist and restore the layout. The docking layout
manager (the next PLAN.md item) will build on the same `RenderSplit`.

Flutter has no split view in its framework; the shape follows the common desktop toolkits
(Qt's `QSplitter`, WPF's `GridSplitter`, VS Code's sash) and the keyboard and semantics
follow the WAI-ARIA window splitter pattern.

## Scope

In scope: N ≥ 1 panes along a horizontal or vertical axis; sizes as fractions in a
shareable signal; per-pane minimum sizes; collapsible panes; pointer dragging of dividers
with a resize cursor; keyboard resizing on focusable dividers; splitter semantics; a gallery
section and a kitchen-sink card.

Out of scope: adding, removing or moving panes at run time while dragging (docking);
dividers that move several panes at once (each divider moves only its two neighbours);
maximum sizes; panes that keep their pixel size when the window resizes (see open
question 2); animated collapse; RTL mirroring (Phase 4 i18n).

## Public API

```rust
// tantu-layout
/// Lays out panes along `axis` with a fixed-thickness gap between neighbours. Children are
/// the `n` panes followed by the `n - 1` divider handles.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderSplit {
    pub axis: Axis,
    /// Requested share of the space left after the gaps, per pane (cleaned at layout).
    pub fractions: Arc<[f32]>,
    /// Smallest size of each pane on the axis, in logical pixels (missing entries are 0).
    pub min_sizes: Arc<[f32]>,
    /// Panes that may shrink to 0 when their fraction is 0 (missing entries are false).
    pub collapsible: Arc<[bool]>,
    /// The gap between neighbouring panes, in which the divider's line is drawn.
    pub gap: f32,
    /// The divider handles' thickness along the axis, centred on each gap; a value below
    /// `gap` counts as `gap`, and a negative or non-finite gap or handle as 0.
    pub handle: f32,
    // private: the pane count and the last layout's sizes (not part of equality)
}
impl RenderSplit {
    /// `panes` panes at equal shares, no minimums, a 1 px gap and 8 px handles.
    pub fn new(axis: Axis, panes: usize) -> Self;
    /// The pane count given to `new` (children beyond `2 × panes - 1` get zero size).
    pub fn panes(&self) -> usize;
    /// The pane sizes from the last layout, in order (empty before the first).
    pub fn sizes(&self) -> &[f32];
}

// tantu-widgets
/// One pane: its content and how it may be resized.
pub struct SplitPane { /* private */ }
impl SplitPane {
    pub fn new(content: impl View) -> Self;
    /// Smallest size on the split's axis; default 0.
    pub fn min_size(self, min: f32) -> Self;
    /// Lets the user collapse the pane to nothing; default false.
    pub fn collapsible(self, collapsible: bool) -> Self;
    /// Names the pane for assistive technology (its divider is "Resize <label>").
    pub fn label(self, label: impl Into<String>) -> Self;
}

/// Panes along one axis with draggable dividers.
pub struct SplitView { /* private */ }
impl SplitView {
    /// Panes side by side (`Axis::Horizontal`) or stacked (`Axis::Vertical`).
    pub fn new(axis: Axis) -> Self;
    pub fn horizontal() -> Self;
    pub fn vertical() -> Self;
    pub fn pane(self, pane: SplitPane) -> Self;
    /// Shares the fractions (one per pane, summing to 1) with the app, which may set them
    /// and persist them. Without it the split owns a signal starting at equal shares.
    pub fn fractions(self, fractions: Signal<Vec<f32>>) -> Self;
    /// The visible line's thickness (default 1 px) and the grab area (default 8 px).
    pub fn divider(self, thickness: f32, grab: f32) -> Self;
}

// tantu-view
pub enum Role { /* existing, */ Splitter }

// tantu-test (TEST-WT-18, added while implementing; review)
impl Finder {
    /// Elements whose semantics label is exactly `label` (Flutter's `find.bySemanticsLabel`).
    pub fn semantics_label(label: impl Into<String>) -> Self;
}
```

`SplitView` implements `View`; `SplitPane` is consumed by it. Both are in the facade prelude.

## Behavior

### Layout (`RenderSplit`)

- **LAYOUT-SPLIT-01:** On a bounded main axis the split takes the whole main extent. The
  space for panes is that extent minus `(n - 1) × gap`, floored at 0. Fractions are cleaned
  before use: a missing entry counts as `1 / n`, a negative or non-finite one as 0, and the
  result is normalized to sum to 1 (all zero: equal shares). The `fractions` field itself is
  never rewritten.
- **LAYOUT-SPLIT-02:** Each pane's size is its share of the space, raised to its minimum,
  except that a collapsible pane with a cleaned fraction of 0 gets exactly 0. Space taken by
  raising panes to their minimum is removed from the other panes in proportion to their
  size above their own minimum. When the minimums alone don't fit, every non-collapsed pane
  gets its minimum scaled down by the same factor so the panes fill the space exactly; no
  overflow, nothing panics.
- **LAYOUT-SPLIT-03:** Panes are placed in order from the start, separated by `gap`, and get
  tight constraints for their size on the main axis and the incoming constraints' cross
  axis tightened to the split's cross size. On a bounded cross axis the split takes the
  maximum cross extent; on an unbounded one the panes are first laid out loose on the cross
  axis and the split takes the largest pane's cross size.
- **LAYOUT-SPLIT-04:** Divider handle `i` (child `n + i`) is laid out tight at `handle` ×
  the cross size, centred on gap `i`, so it overlaps its neighbours by
  `(handle - gap) / 2` each side, and is placed after the panes so it is hit first.
- **LAYOUT-SPLIT-05:** An unbounded main axis gives each pane its minimum (0 for a collapsed
  one) and the split their sum plus the gaps, with a debug `tracing` warning, as a flexible
  child in an unbounded `Row` does (decision 33). A split with no panes takes the smallest
  size. Children beyond `2n - 1` get zero size.

### Widget (`SplitView`)

- **WIDGETS-SPLIT-01:** The split shows its panes in order, sized by `RenderSplit` from the
  fractions signal (shared or owned). Changing the signal relayouts the split without
  rebuilding any pane. A `Vec` shorter or longer than the pane count is cleaned as in
  LAYOUT-SPLIT-01 and is not rewritten until the user resizes.
- **WIDGETS-SPLIT-02:** Look: each gap shows a `thickness` line of the theme's
  `outline_variant` centred in it (the gap equals `thickness`); a hovered or dragged divider
  draws its line in `primary` at 3 px; a focused divider shows the 2 px `primary` focus
  outline around its grab area. Theme changes repaint only.
- **WIDGETS-SPLIT-03:** Dragging: a primary press in divider `i`'s grab area captures the
  pointer; moving moves the boundary between panes `i` and `i + 1` by the pointer's movement
  on the axis (no slop), keeping the sum of the two panes and every other pane's size
  unchanged, and clamped so neither neighbour goes below its minimum. Each move writes the
  whole fractions vector (one entry per pane, the sizes divided by the pane space). The
  pointer's position on the cross axis is ignored. Secondary buttons do nothing.
- **WIDGETS-SPLIT-04:** Collapsing: dragging a collapsible pane below half its minimum
  collapses it (fraction 0, its space given to the neighbour); dragging back past half its
  minimum restores it at its minimum. A non-collapsible pane stops at its minimum.
- **WIDGETS-SPLIT-05:** The divider sets `ResizeHorizontal` (side-by-side panes) or
  `ResizeVertical` (stacked panes) as its cursor; during a drag the cursor stays (VIEW-EVENT-09).
- **WIDGETS-SPLIT-06:** Each divider is a focus stop. Dividers come after all the panes'
  contents in traversal order, in order (they are the split's last children so they win the
  hit test over the panes they overlap, LAYOUT-SPLIT-04; review). When focused: Left/Up move its boundary 10 px towards the start and Right/Down
  10 px towards the end, whatever the axis (so accessibility increments work, VIEW-SEM-03);
  Home and End move it as far as the minimums allow; Enter collapses the nearest
  collapsible neighbour (the preceding pane first) and restores it to its fraction before
  collapsing when pressed again. Repeats move again; modified keys pass through. Keys that
  can't move anything do nothing and are not handled.
- **WIDGETS-SPLIT-07:** A press on a divider doesn't take focus (as `Button`), so dragging
  never steals focus from a pane's text field.
- **WIDGETS-SPLIT-08:** Semantics: each divider has `Splitter`, the label "Resize <label of
  the preceding pane>" (or "Resize pane <i + 1>"), a numeric value giving the boundary's
  position as a percentage of the pane space (min 0, max 100, step the 10 px move as a
  percentage), `Focus`, `Increment` and `Decrement`. Panes add no node of their own unless
  labelled, when they are a `Group` with that label. A split with one pane has no divider.

### Accessibility and gallery

- **A11Y-TREE-08:** `Role::Splitter` maps to AccessKit's `Splitter`; its numeric value uses
  the existing value mapping (A11Y-TREE-02).
- **WIDGETS-GRID-29:** (amends [DataGrid columns](data-grid-columns.md), review) A header
  cell's resize handle sets `ResizeHorizontal` as its cursor, so the column handles show
  the resize cursor now that cursors reach the window ([cursor icons](../platform/cursor.md)).
- **TEST-WT-18:** (amends [WidgetTester](../test/widget-tester.md), review)
  `Finder::semantics_label(label)` matches the elements whose `Semantics::name` (the
  accessible label) equals `label`, so tests can find elements that have a label but no key or text, such as a
  split view's dividers ("Resize Folders").
- **GALLERY-24:** A "Split view" section before Tabs (twelve sections, thirteen since
  GALLERY-26): a horizontal split
  (key `split`, 720 × 360) of a 160 px-minimum collapsible "Folders" list (200 px
  would override its 25 % share in the gallery's 728 px content area), a "Messages" list and a
  vertical split of a "Preview" text and an "Output" pane (keys `split-*`), with fractions
  shared with the section, echoed as "Sizes: 25 % · 35 % · 40 %" (`split-echo`) and a
  "Reset" button. Tests: dragging the first divider changes the echo; Enter on the focused
  first divider collapses Folders and Enter restores it; Reset restores. Golden
  `gallery-split.png` at 960 × 640.
- **GALLERY-25:** A "split" kitchen-sink card after "tabs" (fifteen `SINK_CARDS` entries, sixteen since GALLERY-27,
  sample `split_view`): a two-pane horizontal split 160 px tall (panes labelled "Left"
  and "Right"), echoed by `sink-split-state` as "Sizes: 50 % · 50 %". Goldens touched by the new section and card are regenerated.

## Performance and allocation

Layout is O(panes). A drag step writes one signal, causing a relayout of the split's
subtree and a repaint; panes are not rebuilt. No allocation per frame; a drag step updates
the fractions vector in place and allocates only the layout object's new fraction list.

## Open questions

Accepted by the user ("continue", 2026-10-15); WIDGETS-SPLIT-06's traversal order and
WIDGETS-GRID-29 were added while agreeing (review):

1. **N panes along one axis**, nested for two dimensions, rather than a fixed two-pane
   splitter: IDE layouts need three or more, and docking wants an N-ary node.
2. **Fractions, not pixels**, so panes scale with the window and a saved layout fits any
   screen. Sidebars that keep their pixel width while the window grows (VS Code-style)
   would be a later `SplitPane::fixed(true)`, kept out of this item.
3. **Each divider moves only its two neighbours**, clamped at their minimums (Qt's
   default pushes further panes; that can come later as an option).
4. **A thin visible line with a wider invisible grab area** (1 px and 8 px), with the
   handle laid out over the panes' edges so it wins the hit test.
5. **WAI-ARIA window splitter keyboard model**: focusable dividers, arrows in 10 px
   steps, Home/End, Enter to collapse and restore; arrows of both axes accepted so the
   synthesized accessibility increments work for either orientation.
6. **Collapsing by dragging below half the minimum**, and with Enter, only for panes
   marked collapsible.
7. **Gallery "Split view" section and a kitchen-sink card**, with behavior tests and goldens.
