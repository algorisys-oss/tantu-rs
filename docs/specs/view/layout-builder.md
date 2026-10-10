# LayoutBuilder

- **Status:** Implemented (decided by the agent while the user was away, at their request; review)
- **Crate:** `tantu-view`
- **Plan item:** Phase 2, `tantu-view` → "`LayoutBuilder`"
- **Related:** [ADR 0011](../../adr/0011-view-layer.md), [ADR 0009](../../adr/0009-layout-tree-in-tantu-layout.md)
  (`LayoutBuilder` belongs to `tantu-view`), [dynamic content](dynamic.md), [view tree](tree.md)

## Purpose

Flutter's `LayoutBuilder`: content that depends on the constraints its parent gives it (a
sidebar that collapses below a width, a grid whose column count follows the space). Views are
built outside layout in Tantu, so the content can't be built in the middle of a layout pass.
Instead the element records the constraints of each layout, and `ViewTree::layout` rebuilds the
content when they changed and lays out again, within the same call. The content is a `Dyn` over
a constraints signal, so it rebuilds, and owns what it creates, exactly like dynamic content.

## Scope

In scope:

- `LayoutBuilder` (the view), its render object, and the extra layout rounds in
  `ViewTree::layout`.

Out of scope:

- Intrinsic sizes of a `LayoutBuilder` (Flutter doesn't support them either): they are 0.

## Public API

Crate root `tantu_view`.

```rust
use tantu_layout::BoxConstraints;

/// Content built from the constraints its parent gives it.
pub struct LayoutBuilder { /* private */ }
impl LayoutBuilder {
    pub fn new<V: View>(builder: impl Fn(BoxConstraints) -> V + 'static) -> Self;
}
impl View for LayoutBuilder { /* a render element whose content is rebuilt per constraints */ }
```

## Behavior

- **VIEW-LB-01:** A `LayoutBuilder` is a render element. Its layout lays out its content (the
  render elements its content builds, at offset zero) with its own constraints and sizes itself
  to the largest content size, constrained; with no content yet it is `constraints.smallest()`.
  It reports 0 for every intrinsic size.
- **VIEW-LB-02:** The content is `builder(constraints)` for the constraints of the element's latest
  layout. It is built during the `ViewTree::layout` call that first lays the element out: the
  call notices the new constraints after its pass, builds the content, and lays out again, so
  when it returns the content is built and laid out.
- **VIEW-LB-03:** When a layout gives the element different constraints from the ones its content
  was built for, the same `ViewTree::layout` call rebuilds the content (old elements removed,
  then the new view built) and lays out again. Equal constraints (compared bit for bit, so NaN
  never counts as a change) cause no rebuild.
- **VIEW-LB-04:** Signals the builder reads also rebuild the content, at the next frame (the builder
  runs in a `Dyn`). What the builder creates is owned like `Dyn` content (VIEW-DYN-02).
- **VIEW-LB-05:** Nested `LayoutBuilder`s settle within one `ViewTree::layout` call: rounds repeat
  while some constraints changed, up to 16 rounds; past that, a `tracing` warning is logged and
  the call returns with the content of the last round.
- **VIEW-LB-06:** Removing the element (or an ancestor) disposes its content and stops its
  rebuilds.

## Performance and allocation

A layout without constraint changes costs one extra check per `LayoutBuilder` (comparing the
recorded constraints). A change costs a rebuild plus a second pass that only re-lays out what
the rebuild marked.

## Open questions

Resolved (2026-10-10, decided by the agent while the user was away; review):

1. **Rebuild after the pass, then lay out again**, rather than building inside the pass (which
   would need the view tree mutably while the layout tree is mid-pass). The visible result is
   the same, since painting happens after layout.
2. **The content is a `Dyn` over a constraints signal**, reusing dynamic content's rebuilds and
   ownership instead of a second mechanism.
3. **A 16-round limit** for nested builders, with a warning, instead of looping forever on a
   builder whose content changes its own constraints.
