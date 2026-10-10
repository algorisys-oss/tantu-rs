# Layout tree and protocol

- **Status:** Implemented
- **Crate:** `tantu-layout`
- **Plan item:** Phase 2, `tantu-layout` → "Layout tree and protocol"
- **Related:** [ADR 0002](../../adr/0002-flutter-structure-and-layout-protocol.md),
  [ADR 0009](../../adr/0009-layout-tree-in-tantu-layout.md), [constraints](constraints.md),
  [arena](../core/arena.md)

## Purpose

The machinery every layout runs on (ADR 0009). A `LayoutTree` is an arena of nodes; each node
holds a layout object implementing `RenderBox`, its ordered children, the parent data its parent
reads (a flex factor, a `Positioned` rect), and the results of its last layout. The tree runs
Flutter's protocol: a parent lays out each child with `BoxConstraints`, gets back a size, and
sets the child's offset. It skips work that can't have changed, and when something changes it
re-lays out as little as possible, stopping at relayout boundaries.

Users of this spec: authors of layouts (the built-in `RenderPadding`, `RenderFlex`, … in the
next specs, and custom layouts in apps), and `tantu-view`, which owns one tree per window and
keeps its nodes in step with elements.

## Scope

In scope:

- `LayoutId`, `LayoutTree`: creating, removing and arranging nodes; typed access to layout
  objects and parent data.
- The `RenderBox` trait and the `LayoutChildren` / `IntrinsicChildren` contexts it works through.
- The layout pass: caching by constraints, `mark_needs_layout`, relayout boundaries.
- Intrinsic sizes: opt-in, computed only when asked, cached.

Out of scope (and where it goes):

- The built-in layouts: single-child, flex, stack and wrap specs.
- Paint, hit-testing, the mapping from elements to layout nodes: `tantu-view`.
- Baselines (text alignment in rows): with `tantu-text`, as an addition to `RenderBox`.
- Dry layout (Flutter's `computeDryLayout`): not needed by Phase 2 layouts; added when one is.
- Text measurement (`TextMeasure`): its own Phase 2 item.
- Thread safety: a tree is used from one thread (`!Sync` is fine; it is `Send` when its layout
  objects are).

## Public API

Crate root `tantu_layout`.

```rust
use std::any::Any;
use tantu_core::{Id, Size, Vec2};

/// A node in a [`LayoutTree`]. `Copy`, 8 bytes; stale after the node is removed (never reused).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayoutId(/* Id */);

impl LayoutId {
    /// The underlying arena id.
    pub fn id(self) -> Id;
    /// Stable 64-bit form, never 0 (`Id::to_bits`).
    pub fn to_bits(self) -> u64;
}

/// Why a tree change was refused. Nothing changes when an error is returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreeError {
    /// The id is stale (or from another tree, when that isn't mistaken for a node here).
    UnknownNode(LayoutId),
    /// The child already has a different parent.
    HasParent { child: LayoutId, parent: LayoutId },
    /// The child is the parent itself or one of its ancestors.
    Cycle(LayoutId),
    /// The same child appears twice in the list.
    Duplicate(LayoutId),
}
impl std::fmt::Display for TreeError { /* ... */ }
impl std::error::Error for TreeError {}

/// A layout algorithm: what a node does in a layout pass. Built-in layouts and app-defined
/// custom layouts implement it the same way.
pub trait RenderBox: Any {
    /// Lays out the children through `children` (each gets constraints and returns its size,
    /// then gets an offset) and returns this node's size. The tree constrains the returned size
    /// to `constraints`.
    fn perform_layout(&mut self, constraints: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size;

    /// True when this node's size depends only on its constraints (not on its children). Such a
    /// node is a relayout boundary. Default `false`.
    fn sized_by_parent(&self) -> bool { false }

    /// The smallest width this node can have without its content overflowing, at the given
    /// height (`f32::INFINITY` for "any"). Default 0.
    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 { 0.0 }
    /// The width beyond which more width doesn't reduce the height, at the given height.
    /// Default 0.
    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 { 0.0 }
    /// As `min_intrinsic_width`, for the height at a given width. Default 0.
    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 { 0.0 }
    /// As `max_intrinsic_width`, for the height at a given width. Default 0.
    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 { 0.0 }
}

/// A node's children during its `perform_layout`, by index in child order.
pub struct LayoutChildren<'a> { /* private */ }

impl LayoutChildren<'_> {
    /// Number of children.
    pub fn len(&self) -> usize;
    /// True with no children.
    pub fn is_empty(&self) -> bool;
    /// The child's id.
    pub fn id(&self, index: usize) -> Option<LayoutId>;
    /// Lays out child `index` with `constraints` and returns its size; this node's layout depends
    /// on that size.
    pub fn layout(&mut self, index: usize, constraints: BoxConstraints) -> Size;
    /// Lays out child `index` when this node doesn't use the child's size (the child becomes a
    /// relayout boundary).
    pub fn layout_ignoring_size(&mut self, index: usize, constraints: BoxConstraints);
    /// Sets child `index`'s offset from this node's top-left corner.
    pub fn set_offset(&mut self, index: usize, offset: Vec2);
    /// The child's size from its last layout (`Size::ZERO` if it was never laid out or the
    /// index is out of range), so a layout can position children after sizing them all.
    pub fn size(&self, index: usize) -> Size;
    /// The child's parent data, if it has some of type `T`.
    pub fn parent_data<T: Any>(&self, index: usize) -> Option<&T>;
    /// The child's intrinsic sizes (see `LayoutTree::min_intrinsic_width`).
    pub fn min_intrinsic_width(&mut self, index: usize, height: f32) -> f32;
    pub fn max_intrinsic_width(&mut self, index: usize, height: f32) -> f32;
    pub fn min_intrinsic_height(&mut self, index: usize, width: f32) -> f32;
    pub fn max_intrinsic_height(&mut self, index: usize, width: f32) -> f32;
}

/// A node's children while it computes an intrinsic size: the same queries, no layout.
pub struct IntrinsicChildren<'a> { /* private */ }

impl IntrinsicChildren<'_> {
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn parent_data<T: Any>(&self, index: usize) -> Option<&T>;
    pub fn min_intrinsic_width(&mut self, index: usize, height: f32) -> f32;
    pub fn max_intrinsic_width(&mut self, index: usize, height: f32) -> f32;
    pub fn min_intrinsic_height(&mut self, index: usize, width: f32) -> f32;
    pub fn max_intrinsic_height(&mut self, index: usize, width: f32) -> f32;
}

/// An arena of layout nodes and the layout pass over them.
#[derive(Default)]
pub struct LayoutTree { /* private */ }

impl LayoutTree {
    /// An empty tree.
    pub fn new() -> Self;
    /// Number of nodes.
    pub fn len(&self) -> usize;
    /// True with no nodes.
    pub fn is_empty(&self) -> bool;
    /// True if `id` is a node of this tree.
    pub fn contains(&self, id: LayoutId) -> bool;

    /// Adds a node with no parent, no children and no parent data. It needs layout.
    pub fn insert(&mut self, render: impl RenderBox) -> LayoutId;
    /// Removes `id` and all its descendants; detaches it from its parent (which then needs
    /// layout). Returns the number of nodes removed (0 for an unknown id).
    pub fn remove(&mut self, id: LayoutId) -> usize;
    /// Makes `children` the ordered children of `parent`. Each child must have no parent or
    /// already be a child of `parent`; previous children not in the list are detached (they
    /// become roots, not removed). The parent needs layout if the list changed.
    pub fn set_children(&mut self, parent: LayoutId, children: &[LayoutId]) -> Result<(), TreeError>;
    /// The ordered children (empty for an unknown id).
    pub fn children(&self, id: LayoutId) -> &[LayoutId];
    /// The parent, if any.
    pub fn parent(&self, id: LayoutId) -> Option<LayoutId>;

    /// The node's layout object, if it is a `T`.
    pub fn get<T: RenderBox>(&self, id: LayoutId) -> Option<&T>;
    /// The node's layout object for changing it, if it is a `T`. Marks the node as needing
    /// layout (the escape hatch for layouts that can't be compared; prefer `set`).
    pub fn get_mut<T: RenderBox>(&mut self, id: LayoutId) -> Option<&mut T>;
    /// Replaces the node's layout object with `render` and marks the node as needing layout
    /// only if it differs from the current one (a different type, or `!=`). Children and
    /// parent data are kept. Returns whether anything changed (false for an unknown id).
    pub fn set<T: RenderBox + PartialEq>(&mut self, id: LayoutId, render: T) -> bool;
    /// Replaces the node's layout object (children and parent data are kept). Marks it as
    /// needing layout. Returns false for an unknown id.
    pub fn replace(&mut self, id: LayoutId, render: impl RenderBox) -> bool;
    /// Sets the data the node's parent reads (`None` clears it). Marks the parent as needing
    /// layout. Returns false for an unknown id.
    pub fn set_parent_data<T: Any>(&mut self, id: LayoutId, data: Option<T>) -> bool;
    /// The node's parent data, if it has some of type `T`.
    pub fn parent_data<T: Any>(&self, id: LayoutId) -> Option<&T>;

    /// Marks the node as needing layout, and its ancestors up to the nearest relayout boundary.
    pub fn mark_needs_layout(&mut self, id: LayoutId);
    /// True if the node will be laid out by the next pass that reaches it.
    pub fn needs_layout(&self, id: LayoutId) -> bool;
    /// True if the node was a relayout boundary in its last layout.
    pub fn is_relayout_boundary(&self, id: LayoutId) -> bool;

    /// Runs a layout pass over `root`'s subtree with `constraints` for `root`, and returns
    /// `root`'s size (`Size::ZERO` for an unknown id).
    pub fn layout(&mut self, root: LayoutId, constraints: BoxConstraints) -> Size;
    /// The node's size from its last layout; `None` if it was never laid out.
    pub fn size(&self, id: LayoutId) -> Option<Size>;
    /// The node's offset from its parent's top-left corner (`Vec2::ZERO` until set); `None`
    /// for an unknown id.
    pub fn offset(&self, id: LayoutId) -> Option<Vec2>;
    /// The constraints of the node's last layout.
    pub fn constraints(&self, id: LayoutId) -> Option<BoxConstraints>;

    /// The node's intrinsic sizes, computed on demand and cached until the node or one of its
    /// descendants needs layout. 0 for an unknown id.
    pub fn min_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32;
    pub fn max_intrinsic_width(&mut self, id: LayoutId, height: f32) -> f32;
    pub fn min_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32;
    pub fn max_intrinsic_height(&mut self, id: LayoutId, width: f32) -> f32;
}
```

## Behavior

Tests use small test layouts (a fixed-size leaf, a "column" that stacks children, a node that
counts its `perform_layout` calls) rather than the built-in layouts, so this spec doesn't depend
on the next ones.

### Nodes and structure

- **LAYOUT-TREE-01:** `insert` returns a new id each time; `len` counts nodes; a new node has no
  parent, no children, no parent data, no size, offset `Vec2::ZERO`, and needs layout.
- **LAYOUT-TREE-02:** After `remove(id)`, `id` and every descendant are gone (`contains` false, every
  query returns `None`, an empty slice, `false` or 0, and nothing panics). Ids are never
  reused, so a removed id never resolves again, even after more inserts. An id from another
  tree is not detected (as CORE-ARENA-07, `LayoutId` stays 8 bytes): it behaves like a stale id
  when its slot doesn't exist here, and may name an unrelated node when it does; either way
  nothing panics. `remove` returns the number of nodes removed; the removed node's parent loses
  it from its children and needs layout.
- **LAYOUT-TREE-03:** `set_children(parent, list)` sets the ordered children; `children` and `parent`
  reflect it. Previous children not in `list` are detached: they keep their subtree and become
  roots. Reordering is allowed. The parent is marked as needing layout only when the list
  differs from the previous one.
- **LAYOUT-TREE-04:** `set_children` returns an error and changes nothing when: `parent` or a child is
  unknown (`UnknownNode`), a child has a different parent (`HasParent`), a child is `parent`
  or one of its ancestors (`Cycle`), or a child appears twice (`Duplicate`).

### Layout objects and parent data

- **LAYOUT-TREE-05:** `get::<T>` returns the layout object when it is a `T`, `None` otherwise or for an
  unknown id, and doesn't mark anything. `get_mut::<T>` returns it under the same conditions
  and marks the node as needing layout (whether or not the caller changes anything).
  `replace` swaps the layout object, keeps children and parent data, and marks the node.
- **LAYOUT-TREE-06:** `set_parent_data(id, Some(v))` stores `v`; `parent_data::<T>` returns it when it
  is a `T` (else `None`); `set_parent_data(id, None::<T>)` clears it. Either marks the parent
  as needing layout (nothing to mark for a root). Inside `perform_layout`,
  `children.parent_data::<T>(i)` returns the same.

### The layout pass

- **LAYOUT-TREE-07:** `layout(root, c)` calls `root`'s `perform_layout` with `c` when the root needs
  layout or `c` differs from its last constraints, and returns the root's size. The root's
  offset is not changed. Nodes outside `root`'s subtree are not touched.
- **LAYOUT-TREE-08:** Inside `perform_layout`, `children.layout(i, c)` lays out child `i` and returns
  its size; `set_offset(i, v)` sets its offset, which `offset` reports after the pass and which
  persists until set again. `len` and `id` describe the children in order. An index out of range
  does nothing: `layout` returns `Size::ZERO`, `set_offset` is ignored, `id` and `parent_data`
  return `None`, intrinsics return 0.
- **LAYOUT-TREE-09:** A size returned by `perform_layout` is replaced by `constraints.constrain(size)`
  (so it always satisfies normalized constraints, and NaN becomes the minimum). The tree
  records the constraints and that size; `size` and `constraints` report them.
- **LAYOUT-TREE-10:** Caching: laying out a node (as root or as a child) that doesn't need layout, with
  the same constraints as its last layout, doesn't call its `perform_layout` and returns its
  recorded size. After a full pass, no node in the subtree that was laid out needs layout.
- **LAYOUT-TREE-11:** A node is a relayout boundary for its next layout when any of these holds: it is
  the root of the pass, its parent laid it out with `layout_ignoring_size`, its constraints are
  tight, or its `sized_by_parent` is true. `is_relayout_boundary` reports what held at its last
  layout.
- **LAYOUT-TREE-12:** `mark_needs_layout(id)` marks `id`; while the marked node is not a relayout
  boundary and has a parent, the parent is marked too. A node that was never laid out counts as
  not a boundary. Marking an unknown id does nothing.
- **LAYOUT-TREE-13:** Minimal relayout. After a full pass, marking one node and running
  `layout(root, same constraints)` calls `perform_layout` on exactly the marked nodes (the
  marked node and the ancestors up to its boundary), plus any children they lay out with
  different constraints or that need layout. A dirty boundary whose ancestors are all clean is
  laid out with its last constraints, without laying out its ancestors. Boundaries are processed
  from the top down, so a boundary inside another dirty one is laid out once.
- **LAYOUT-TREE-14:** Structural changes count as marks: a new child list, a removed child and a new or
  cleared parent data mark the parent (LAYOUT-TREE-03, 02, 06), and the next pass re-lays out
  accordingly. A detached node keeps its last size and offset but isn't laid out by passes over
  its old tree.

### Intrinsic sizes

- **LAYOUT-TREE-15:** The tree's `*_intrinsic_*` methods call the node's corresponding `RenderBox`
  method, through `IntrinsicChildren` (which gives the children's intrinsics the same way), and
  never call `perform_layout`. A result that is NaN or negative is reported as 0. With the
  default methods every intrinsic is 0.
- **LAYOUT-TREE-16:** Intrinsic results are cached per node, per method and per argument, until the
  node or any descendant is marked as needing layout (which clears the cache of the marked node
  and its ancestors). When a marked node has cached intrinsics, marking continues to its parent
  even past a relayout boundary, because the parent may have used them.

- **LAYOUT-TREE-18:** `set(id, render)` with a value equal to the current layout object (same type,
  `==`) changes nothing, marks nothing and returns false. With a different value or a different
  type it replaces the layout object, keeps children and parent data, marks the node (as
  `mark_needs_layout`) and returns true. An unknown id returns false.

- **LAYOUT-TREE-19:** Inside `perform_layout`, `children.size(i)` is child `i`'s size from its last
  layout (the size `layout(i, …)` returned in this pass, if it was laid out in it), and
  `Size::ZERO` for a child never laid out or an index out of range.

### Robustness

- **LAYOUT-TREE-17:** Nothing panics for stale ids, out-of-range indices, any constraints (NaN and
  infinities included) or any value a layout object returns. A tree 1 000 nodes deep lays out
  in a debug build on the test thread's default stack.

## Performance and allocation

- A layout pass over a tree whose structure didn't change since the last pass allocates nothing
  (checked with a counting allocator, as for the Scene). Dirty-boundary lists and scratch buffers
  are kept between passes.
- A pass with nothing marked and unchanged root constraints calls no `perform_layout` and is
  O(1).
- The full-layout benchmark (10k nodes, under 1 ms) is its own PLAN.md item, measured with the
  built-in layouts once they exist.

## Open questions

Resolved (2026-10-10, agreed with the user):

1. **Marking without property setters.** Layout objects hold only what affects layout (paint
   state lives in `tantu-view`, ADR 0009), so any real change is a layout change. `set` compares
   with `PartialEq` and marks only on a real change (LAYOUT-TREE-18); `tantu-view` rebuilds the
   layout object from a view's properties and calls `set`. `get_mut` stays as the escape hatch
   and marks unconditionally. Flutter-style setters taking a context were rejected: every
   property of every layout would need a hand-written setter, and a forgotten mark would be a
   silent bug.
2. **Sizes that break their constraints** are constrained (LAYOUT-TREE-09), with a `tracing`
   warning in debug builds naming the layout's type.
3. **`remove` takes the subtree; `set_children` detaches**, so moving nodes never loses state.
4. **Typed parent data via `Any`**; built-in parent data types come with their layouts.
5. **The pass recurses**, with the 1 000-level promise in debug; an explicit stack is deferred.
