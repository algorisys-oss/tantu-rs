# Paint

- **Status:** Implemented (decided by the agent while the user was away, at their request; review)
- **Crate:** `tantu-view`
- **Plan item:** Phase 2, `tantu-view` → "Paint"
- **Related:** [ADR 0011](../../adr/0011-view-layer.md) (point 8), [view tree](tree.md),
  [Scene](../scene/scene.md)

## Purpose

How elements become Scene commands. Each render element has a `Paint` behavior that draws the
element in its own coordinates and decides where its children are painted, so it can wrap them
in a clip, a transform or a layer (Flutter's `paintChild`). `ViewTree::paint` walks the render
elements, applies each one's layout offset, tags the commands with the element's id, and skips
elements that are off-screen where that's safe (culling, from Clay).

## Scope

In scope:

- The `Paint` trait, `PaintCx`, `BuildCx::set_paint`, `ViewTree::paint`.
- Offsets, element ids and z-index scoping, child order, culling.

Out of scope (and where it goes):

- Opening and finishing the Scene, damage and when to repaint: the frame spec.
- Text glyphs: `tantu-text` provides shaping at paint time (its own item).
- Paint caching, layers reused across frames, damage regions: Phase 5.

## Public API

Crate root `tantu_view`.

```rust
use std::any::Any;
use tantu_core::Size;
use tantu_scene::{ElementId, SceneBuilder};

/// How a render element draws itself and where its children are painted. (`Any`, so
/// `ElementMut::paint` can downcast it; every `'static` type is `Any`.)
pub trait Paint: Any {
    /// Draws the element. `cx` is in the element's coordinates ((0, 0) is its top-left corner,
    /// its size is `cx.size()`), with its id as the current element. The children are painted
    /// where this calls `cx.paint_children()`, or after it returns if it doesn't (unless it
    /// calls `cx.skip_children()`). It must leave the builder's scopes balanced.
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {}
    /// How far the element's own drawing may extend beyond its bounds (shadows, focus rings),
    /// for culling. Default 0.
    fn overflow(&self) -> f32 { 0.0 }
    /// True if the element clips its children to its bounds, so culling may skip the whole
    /// subtree with it. Default false.
    fn clips_children(&self) -> bool { false }
}

/// Paints nothing (the default for render elements).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoPaint;
impl Paint for NoPaint {}

/// What a `Paint` draws with.
pub struct PaintCx<'t, 'b> { /* private: the tree, the element, the builder */ }

impl<'b> PaintCx<'_, 'b> {
    /// The element being painted.
    pub fn element(&self) -> ElementId;
    /// Its size from the last layout.
    pub fn size(&self) -> Size;
    /// The Scene builder, in the element's coordinates.
    pub fn scene(&mut self) -> &mut SceneBuilder<'b>;
    /// Paints the element's children here (once; later calls do nothing).
    pub fn paint_children(&mut self);
    /// Don't paint the children at all.
    pub fn skip_children(&mut self);
}

impl BuildCx<'_> {
    /// Gives a render element its paint behavior (replacing the previous one). Returns false
    /// for a region or an unknown id.
    pub fn set_paint(&mut self, element: ElementId, paint: impl Paint) -> bool;
}

impl ViewTree {
    /// Paints the whole tree into `scene` (an open builder for the window's Scene), using the
    /// geometry of the last layout pass.
    pub fn paint(&self, scene: &mut SceneBuilder<'_>);
}
```

## Behavior

Tests paint into a Scene and inspect its entries, with a test `Paint` that fills its bounds
with a color and records each call.

- **VIEW-PAINT-01:** `set_paint` on a render element returns true and replaces its paint behavior;
  on a region or an unknown id it returns false. A render element without one paints nothing
  of its own (`NoPaint`); its children are still painted.
- **VIEW-PAINT-02:** `ViewTree::paint` visits the render elements depth-first, in child order, with
  regions transparent (their children are visited in their place), calling each element's
  `paint` exactly once per call. Each render element is painted inside its own transform scope:
  `push_transform(translate(offset))`, where the offset is its layout offset relative to its
  render parent, then `set_element(Some(id))`, its painting and its children, then `pop`. So
  every command it records carries its id, and element id and z-index changes made by a `Paint`
  don't leak to its siblings.
- **VIEW-PAINT-03:** Children are painted exactly once per element: where `paint_children` is
  called (a second call does nothing), or right after `paint` returns if it wasn't called;
  never if `skip_children` was called first. Painting children inside a clip or layer the
  `Paint` pushed puts their commands inside that scope.
- **VIEW-PAINT-04:** `PaintCx::size` is the element's size from the last layout and `element` its
  id. An element that has never been laid out (no size) is skipped with its subtree.
- **VIEW-PAINT-05:** Culling. Before its scope is pushed, a render element's bounds `(0, 0, size)`,
  inflated by `overflow()` and moved by its offset, are tested with `SceneBuilder::is_culled`
  (in the parent's coordinates). If they are culled, the element is skipped (no scope, no
  `paint` call) when it has no children, or when `clips_children()` is true (then its whole
  subtree is skipped). Other elements are always visited, because their
  children may extend past their bounds.
- **VIEW-PAINT-06:** Painting is deterministic: painting the same tree twice into fresh Scenes gives
  equal Scenes. Painting doesn't change the tree.

## Performance and allocation

The traversal allocates nothing (it recurses over child slices); each painted render element
adds two scope entries to the Scene besides its own commands. Merging consecutive
transforms, skipping identity translations and caching subtrees are Phase 5 work, measured
against the Scene's allocation-free recording.

## Open questions

Resolved (2026-10-10, decided by the agent while the user was away; review):

1. **A transform scope per render element**, even at offset zero: the builder has no getter for
   its element id and z-index, so a scope is the only way to restore them reliably after a
   `Paint` changes them. Two entries per element are accepted until Phase 5.
2. **Children painted after `paint` by default**, with an explicit `skip_children`, so a
   decoration's `Paint` can't forget its children, while clips and layers still choose where
   children go.
3. **Conservative culling** (VIEW-PAINT-05): only leaves and clipping elements are culled, so a
   `Stack` child positioned outside its parent or a row's overflow is never wrongly skipped.
   Large scrolling content gets culled because scroll views clip.
