# RenderViewport

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-12; review)
- **Crate:** `tantu-layout`
- **Plan item:** Phase 3, "Scroll: `ScrollView`, scrollbars, kinetic/wheel/trackpad handling"
  (the layout part)
- **Related:** [layout tree](tree.md), [ScrollView](../widgets/scroll.md), Flutter's
  `RenderSingleChildViewport`

## Purpose

A viewport lays its child out with an unbounded main axis and shows a window onto it, shifted
by a scroll offset. The offset is applied as the child's *layout* offset, so hit-testing (which
follows layout offsets) finds scrolled content where it is drawn, and a changed offset is a
cheap re-layout (the child's constraints don't change, so its layout is cached).

## Public API

```rust
/// The scrolling axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAxis { #[default] Vertical, Horizontal }

/// Shows a window onto its child, shifted by `offset` along `axis` (Flutter's
/// `RenderSingleChildViewport`).
#[derive(Clone, Copy, Debug)]
pub struct RenderViewport {
    pub axis: ScrollAxis,
    /// How far the content is scrolled (positive scrolls content up/left). Stored as given;
    /// a non-finite value counts as 0.
    pub offset: f32,
    /* content: Size, from the last layout; not part of equality */
}

impl RenderViewport {
    pub const fn new(axis: ScrollAxis) -> Self;   // offset 0
    /// The child's size from the last layout (`Size::ZERO` before, or without a child).
    pub fn content(&self) -> Size;
}
impl PartialEq for RenderViewport { /* axis and offset only */ }
```

## Behavior

- **LAYOUT-VIEWPORT-01:** The child gets the incoming constraints with the main axis made
  unbounded (`0..=∞`), the cross axis as incoming. The viewport's size is the child's size
  constrained by the incoming constraints (a 300 × 1000 child under loose 300 × 200 gives
  300 × 200; a 300 × 50 child gives 300 × 50), the smallest size without a child. The child's
  offset is `(0, −offset)` (vertical) or `(−offset, 0)` (horizontal), with a non-finite offset
  counting as 0. `content()` is the child's size after layout. Only the first child is laid
  out.
- **LAYOUT-VIEWPORT-02:** Intrinsics are the child's (0 without one). Equality ignores the
  last content size, so `LayoutTree::set` with the same axis and offset doesn't re-lay out, and
  a changed offset does.

## Open questions

Decided by the agent (review): the offset as a layout offset (not a paint transform), so
hit-testing and culling need nothing new; the viewport doesn't clamp (the widget does, knowing
the extents), so programmatic offsets can be set before the first layout.
