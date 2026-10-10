# View tree

- **Status:** Implemented (decided by the agent while the user was away, at their request; review)
- **Crate:** `tantu-view`
- **Plan item:** Phase 2, `tantu-view` → "View tree"
- **Related:** [ADR 0011](../../adr/0011-view-layer.md), [ADR 0009](../../adr/0009-layout-tree-in-tantu-layout.md),
  [layout tree](../layout/tree.md), [signals](../reactive/signals.md)

## Purpose

The skeleton of the view layer (ADR 0011): views that build elements once, the per-window
`ViewTree` that owns the reactive runtime, the element arena and the layout tree, the two kinds
of elements (render elements with a layout node, region elements without one), each element's
reactive scope, and removing elements. Painting, reactive props, frames and dynamic content
build on this in their own specs.

Users: widget authors (every widget's view builds elements through `BuildCx`), the app runner
(one `ViewTree` per window), and the later view specs.

## Scope

In scope:

- `View`, `AnyView`, `BuildCx` (render elements, region elements, parent data).
- `ViewTree`: creation from an app closure, the implicit root element, queries, removal,
  running a layout pass, entering the runtime, teardown.

Out of scope (and where it goes):

- Painting and the `Paint` trait: paint spec. Render elements built here paint nothing but
  their children.
- Reactive props, the update queue and frames: frame spec.
- `Dyn`, `Show`, `For`: dynamic-content spec (they use region elements and `remove`).
- Events, hit-testing, focus: the event-dispatch item.
- Keys: with `For` (dynamic-content spec).

## Public API

Crate root `tantu_view`. Element ids are `tantu_scene::ElementId` (re-exported), built from the
arena id (`ElementId::from(Id)`).

```rust
use std::any::Any;
use tantu_core::Size;
use tantu_layout::{BoxConstraints, LayoutId, LayoutTree, RenderBox, TextMeasure};
use tantu_reactive::{Runtime, Scope};
pub use tantu_scene::ElementId;

/// A description of part of the UI that builds its element(s) once, when consumed.
pub trait View: 'static {
    /// Builds this view's element under `cx`'s current parent and returns it.
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId;
}

/// A type-erased view, for child lists.
pub struct AnyView { /* Box<dyn …> */ }
impl AnyView {
    pub fn new(view: impl View) -> Self;
}
impl View for AnyView { /* builds the wrapped view */ }

/// What kind of element an id names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementKind {
    /// Owns a layout node (and, later, a paint behavior).
    Render,
    /// Owns no layout node; its children take its place in the nearest render ancestor's
    /// layout child list.
    Region,
}

/// The context a view builds in: the tree and the current parent element.
pub struct BuildCx<'a> { /* private */ }

impl BuildCx<'_> {
    /// Creates a render element under the current parent, owning a new layout node with
    /// `render`, then builds `children` under it, in order, inside its scope.
    pub fn render(&mut self, render: impl RenderBox, children: impl IntoIterator<Item = AnyView>) -> ElementId;
    /// Creates a region element under the current parent, then runs `build` with the region as
    /// the parent (inside its scope) to build its children.
    pub fn region(&mut self, build: impl FnOnce(&mut BuildCx<'_>)) -> ElementId;
    /// Sets the parent data of a render element's layout node (read by its layout parent, e.g.
    /// `FlexParentData` for `Expanded`). Returns false (and does nothing) for a region or an
    /// unknown id.
    pub fn set_parent_data<T: Any>(&mut self, element: ElementId, data: T) -> bool;
    /// The current parent element.
    pub fn parent(&self) -> ElementId;
    /// The tree, read-only.
    pub fn tree(&self) -> &ViewTree;
}

/// The views, elements and layout of one window, with its reactive runtime.
pub struct ViewTree { /* private */ }

impl ViewTree {
    /// A tree whose content is the view returned by `app`. `app` runs once, with the tree's
    /// runtime current and inside the root element's scope, so it can create signals.
    pub fn new<V: View>(app: impl FnOnce() -> V) -> Self;
    /// The implicit root render element; the app's view is built under it.
    pub fn root(&self) -> ElementId;
    /// Number of elements, the root included.
    pub fn len(&self) -> usize;
    /// True if `id` is an element of this tree.
    pub fn contains(&self, id: ElementId) -> bool;
    /// The element's kind.
    pub fn kind(&self, id: ElementId) -> Option<ElementKind>;
    /// The element's parent (`None` for the root or an unknown id).
    pub fn parent(&self, id: ElementId) -> Option<ElementId>;
    /// The element's children, in order (empty for an unknown id).
    pub fn children(&self, id: ElementId) -> &[ElementId];
    /// A render element's layout node.
    pub fn layout_id(&self, id: ElementId) -> Option<LayoutId>;
    /// The element's reactive scope.
    pub fn scope(&self, id: ElementId) -> Option<Scope>;
    /// The layout tree (read-only; it changes only through the view tree).
    pub fn layout_tree(&self) -> &LayoutTree;
    /// Removes `id` and its descendants: disposes their scopes (effects stop, cleanups run),
    /// removes their layout nodes, and updates the parent. Returns the number of elements
    /// removed (0 for the root or an unknown id).
    pub fn remove(&mut self, id: ElementId) -> usize;
    /// Runs a layout pass over the whole tree with `constraints` for the root (the window) and
    /// `text` as the text measurer; returns the root's size.
    pub fn layout(&mut self, constraints: BoxConstraints, text: &mut dyn TextMeasure) -> Size;
    /// Runs `f` with the tree's runtime current (to read or write signals from outside).
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R;
}

impl Drop for ViewTree { /* disposes every element's scope, then the runtime */ }
```

## Behavior

Tests build small trees from test views: a leaf view that makes a render element with a
fixed-size `RenderBox`, and a column view that makes a render element with `RenderFlex::column()`
and children.

### Building

- **VIEW-TREE-01:** `ViewTree::new(app)` calls `app` exactly once, with the tree's runtime current
  and inside the root element's scope (signals and effects created there work and belong to
  the root scope), then builds the returned view under the root. The root is a render element
  with no parent; `len` counts every element, the root included.
- **VIEW-TREE-02:** `BuildCx::render(r, children)` creates one render element under the current
  parent, with a new layout node holding `r`, appended to the parent's children; it builds
  `children` in order with the new element as their parent and returns its id. Ids are unique
  in the tree, never reused after removal, and equal `ElementId::from` of the arena id.
- **VIEW-TREE-03:** `BuildCx::region(build)` creates a region element under the current parent (no
  layout node) and runs `build` with the region as the parent. `parent()` inside a build is
  the element being built under.
- **VIEW-TREE-04:** Layout children: a render element's layout node has, as its layout children,
  the layout nodes of its element children in order, where each region child is replaced by
  its own children, recursively (regions nest). So a region inside a column contributes its
  render children directly to the column's flex.
- **VIEW-TREE-05:** Scopes: each element has its own reactive scope, owned by its parent element's
  scope. Signals, effects and cleanups created while building an element's children (and, for
  `region`, inside `build`) belong to that element's scope.
- **VIEW-TREE-06:** `set_parent_data(el, data)` sets `data` on a render element's layout node and
  returns true; for a region or an unknown id it returns false and changes nothing.
- **VIEW-TREE-07:** `AnyView::new(v)` builds exactly what `v` builds; a `Vec<AnyView>` works as the
  children of `render`.

### The root and layout

- **VIEW-TREE-08:** The root's layout object lays out each child with the root's constraints at
  offset zero and sizes the root to `constraints.biggest()` when that is finite (the window),
  otherwise to the largest child size, constrained. `ViewTree::layout(c, text)` runs a layout
  pass from the root with `c` and `text` (through `LayoutTree::with_text`) and returns the
  root's size; afterwards every element's geometry is in `layout_tree()`.

### Removal and teardown

- **VIEW-TREE-09:** `remove(id)` removes the element and all its descendants: their scopes are
  disposed (each cleanup runs once; effects stop, so writing a signal they read no longer runs
  them), their layout nodes are removed from the layout tree, the element leaves its parent's
  children, and the nearest render ancestor's layout children are recomputed (VIEW-TREE-04).
  It returns the number of elements removed. Removing the root or an unknown id returns 0 and
  changes nothing.
- **VIEW-TREE-10:** Dropping the tree disposes every element's scope (every cleanup runs once) and
  then the runtime.

### Robustness

- **VIEW-TREE-11:** Queries on removed or unknown ids return `None`, an empty slice or false, and
  never panic. `enter` makes the tree's runtime current for its closure and restores the
  previous one afterwards (signals created by the tree can be read and written inside it).

## Performance and allocation

Building allocates (elements, layout nodes, child lists); that's a one-time cost per element.
Queries are O(1). `remove` is O(size of the removed subtree + the render ancestor's child
count). Element storage is a `tantu_core::Arena`.

## Open questions

Resolved (2026-10-10, decided by the agent while the user was away; review):

1. **An implicit root render element**, so the app's view may be a region (a `Dyn` at the top)
   or a single render element alike, and the window constraints have one entry point.
2. **Element ids are Scene `ElementId`s** (no separate view id type): Scene commands, elements and
   later AccessKit nodes share one id.
3. **`remove` is public**, so dynamic regions and custom region-based widgets outside this crate
   can drop content; ordinary apps never call it.
4. **Parent data on a region is ignored** (VIEW-TREE-06). `Expanded(For(...))` has no single
   layout node to mark; wrap each item instead.
