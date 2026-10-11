# RenderListContent

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-12; review)
- **Crate:** `tantu-layout`
- **Plan item:** Phase 3, "Virtualized `ListView`" (the layout part)
- **Related:** [viewport](viewport.md), [ListView](../widgets/list-view.md), Flutter's
  `RenderSliverFixedExtentList`

## Purpose

The content of a virtualized list: it is as long as *all* its items (so the viewport scrolls
over the whole list) but holds only the items that were built, each placed at its index's
position. Fixed item extents keep the arithmetic trivial: index × extent.

## Public API

```rust
/// Which item a built child of a `RenderListContent` is (parent data).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListIndex(pub usize);

/// Sizes to `count` items of `item_extent` along `axis` and places its children by their
/// `ListIndex`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderListContent {
    pub axis: ScrollAxis,
    pub count: usize,
    /// Each item's extent along the axis; negative or NaN counts as 0.
    pub item_extent: f32,
}

impl RenderListContent {
    pub const fn new(axis: ScrollAxis, count: usize, item_extent: f32) -> Self;
}
```

## Behavior

- **LAYOUT-LIST-01:** The main-axis size is `count · item_extent` and the cross-axis size is
  the incoming maximum when bounded, else the largest child's cross size (0 without children),
  both then constrained. Every child is laid out with the main axis tight at `item_extent` and
  the cross axis tight at the content's cross size (loose when it was unbounded), and placed at
  `index · item_extent` along the axis, where `index` is its `ListIndex` parent data, or its
  position among the children without one.
- **LAYOUT-LIST-02:** Intrinsics: along the axis `count · item_extent` (min and max); across it
  the largest child's (0 without). Equal properties compare equal (`LayoutTree::set` with the
  same values doesn't re-lay out).

- **LAYOUT-LIST-03:** `ListExtents` (a fixed extent or a table of prefix sums) on
  `RenderListContent`; see [ListView: variable extents](../widgets/list-view-advanced.md).

## Open questions

Decided by the agent (review): fixed extents only for now; variable extents (measuring items
as they appear, with estimated positions) come when a real list needs them.
