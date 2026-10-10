# Overlay and anchored positioning

- **Status:** Draft
- **Crate:** `tantu-view`
- **Plan item:** Phase 3, "`Overlay` + anchored positioning (target/follower): 9-point anchors,
  offset, z-index, pointer passthrough, flip/clamp to window"
- **Related:** [ADR 0011](../../adr/0011-view-layer.md), [view tree](tree.md),
  [paint](paint.md), [events](events.md), [focus](focus.md), PLAN.md "What we take from Clay"
  (floating elements), decision 23 (popups escape clips through an overlay at the root)

## Purpose

Tooltips, menus, dropdowns, popovers and (later) dialogs have to draw above everything else,
outside their parent's clip and layout, and stay attached to the widget that opened them. In
Flutter that's `Overlay`, plus `OverlayPortal` (declarative) and
`CompositedTransformTarget`/`Follower` (anchoring). Clay calls them floating elements, with
9-point attach anchors, an offset, a z-index and pointer passthrough.

This spec adds one positioning model for all of them: an **overlay layer** at the root of each
view tree, and an **`OverlayPortal`** view. A portal shows its child in place and, while
visible, shows its overlay content in the overlay layer. The content is anchored to the child
by a point on each, with an offset, flipped or clamped to stay inside the window.

## Scope

In scope:

- The overlay layer: laid out after the content, painted after it at the root (so above it and
  outside any clip), and hit-tested before it.
- `OverlayPortal` (declarative, reactive visibility) and `Anchor` (target point, follower point,
  offset, flip, clamp).
- `z_index` ordering between overlays, and pointer passthrough (`OverlayPointer::Passthrough`
  for tooltips).
- `on_outside_press`: a callback when a primary press lands outside the overlay content and
  outside the target (menus and dropdowns close on it).

Out of scope (and where it goes):

- Modal barriers, dialogs, focus trapping in a popup: Phase 4 "Dialogs".
- `Tooltip`, menus and dropdowns themselves: Phase 3 widgets and Phase 4 menus, built on this.
- Animated show and hide: Phase 3 animation.
- Overlays in their own OS windows (native popups): later, with multi-window.

## Public API (proposal)

```rust
/// Where an overlay goes relative to its target (Flutter's follower, Clay's attach points).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    /// The point on the target (one of Alignment's 9 points, or any value in between).
    pub target: Alignment,
    /// The point on the overlay placed at the target point.
    pub follower: Alignment,
    /// Added after anchoring, in logical pixels.
    pub offset: Vec2,
    /// Mirror vertically (then horizontally) when the overlay would leave the window.
    pub flip: bool,
    /// Shift the overlay to stay inside the window (after flipping).
    pub clamp: bool,
}

impl Anchor {
    /// Below the target, start edges aligned (dropdowns, menus): BOTTOM_LEFT to TOP_LEFT,
    /// flip and clamp on.
    pub fn below() -> Self;
    /// Above the target, centered (tooltips): TOP_CENTER to BOTTOM_CENTER, flip and clamp on.
    pub fn above() -> Self;
    /// `target` to `follower`, no offset, flip and clamp on.
    pub fn new(target: Alignment, follower: Alignment) -> Self;
    pub fn offset(self, dx: f32, dy: f32) -> Self;
    pub fn flip(self, flip: bool) -> Self;
    pub fn clamp(self, clamp: bool) -> Self;
}

/// Whether an overlay takes pointer events.
pub enum OverlayPointer {
    /// It is hit-tested like other elements (menus, popovers). Default.
    Capture,
    /// Pointer events pass through it to what's below (tooltips).
    Passthrough,
}

/// Shows `child` in place and, while `visible`, `overlay` in the overlay layer, anchored to
/// the child (Flutter's `OverlayPortal`).
pub struct OverlayPortal<V> { /* child, visible, overlay builder, anchor, z_index, pointer, on_outside_press */ }

impl<V: View> OverlayPortal<V> {
    pub fn new(child: V) -> Self;
    /// Whether the overlay shows (a value, a closure or a signal). Default false.
    pub fn visible(self, visible: impl IntoProp<bool>) -> Self;
    /// Builds the overlay content each time it becomes visible (dropped when hidden).
    pub fn overlay<W: View>(self, content: impl Fn() -> W + 'static) -> Self;
    pub fn anchor(self, anchor: Anchor) -> Self;
    /// Higher draws above lower; equal values keep insertion order. Default 0.
    pub fn z_index(self, z: i32) -> Self;
    pub fn pointer(self, pointer: OverlayPointer) -> Self;
    /// Called on a primary press outside both the overlay content and the child.
    pub fn on_outside_press(self, f: impl Fn() + 'static) -> Self;
}
```

## Behavior (proposal)

- **VIEW-OVERLAY-01:** While visible, the overlay content is built under the overlay layer.
  It's laid out with loose window constraints and placed so that its `follower` point is at
  the target's `target` point (in window coordinates, from the child's last layout) plus
  `offset`. While hidden, there's no content, and becoming hidden removes it (scope disposed).
- **VIEW-OVERLAY-02:** Overlays paint after all content, at the root (outside every clip and
  transform scope of the content), ordered by `z_index`, then insertion order.
- **VIEW-OVERLAY-03:** Flip: when the placed overlay crosses the window's top or bottom edge and
  `flip` is on, the vertical alignments are mirrored (y → −y) and the vertical offset negated.
  The mirrored placement is kept if it fits vertically. The same then applies horizontally.
  Clamp: after flipping, when `clamp` is on, the overlay is shifted to lie inside the window,
  and one larger than the window is placed at its top-left.
- **VIEW-OVERLAY-04:** Hit-testing tests overlays first, topmost first. A `Capture` overlay
  receives pointer events over its bounds; a `Passthrough` overlay is skipped, so the content
  below gets them. Hover, pointer capture and focus on press work in overlays as elsewhere.
- **VIEW-OVERLAY-05:** `on_outside_press` runs (with the runtime current) on a primary press
  whose hit path includes neither the overlay content nor the portal's child, before that press
  is dispatched.
- **VIEW-OVERLAY-06:** The overlay follows its target. After any layout in which the target
  moved or resized, the overlay is placed again in the same frame. Removing the portal removes
  its overlay.
- **VIEW-OVERLAY-07:** Overlay content is part of the tree for focus. Its focusable elements come
  after all content in traversal order, and keys reach a focused element in an overlay as
  usual.

## Open questions (for the user)

1. **Declarative `OverlayPortal`** (visibility as a prop) rather than imperative
   `Overlay::insert(entry)` / `entry.remove()`. Proposal: declarative, which fits run-once
   components and signals. An imperative handle can wrap it later.
2. **Anchoring with `Alignment` for both points** (9 named points, or anything in between), as
   Flutter's `CompositedTransformFollower` does. Proposal: yes. Clay's 9-point attach is the
   special case.
3. **Flip vertically first, then horizontally, then clamp**, as menus usually do. Proposal: yes.
4. **One overlay layer per window**, at the root; no nested `Overlay` widgets. Proposal: yes.
   Nested overlays (e.g. inside a dialog) come with dialogs in Phase 4.
