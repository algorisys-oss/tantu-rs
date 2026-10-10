# Stack layout

- **Status:** Implemented
- **Crate:** `tantu-layout`
- **Plan item:** Phase 2, `tantu-layout` → "`RenderStack` with `Positioned` parent data"
- **Related:** [ADR 0002](../../adr/0002-flutter-structure-and-layout-protocol.md),
  [layout tree](tree.md), [single-child layouts](single-child.md) (`Alignment`)

## Purpose

`RenderStack` layers its children on top of each other, in child order (later children paint
on top, which `tantu-view` handles). Non-positioned children are sized by the stack's `fit` and
placed by its `alignment`; they decide the stack's size. Positioned children (`Positioned`
widget, `StackParentData`) are placed by distances from the stack's edges and don't affect its
size. This is Flutter's `RenderStack`: the layout behind badges, overlays within a widget,
floating buttons and any "this on top of that" design.

## Scope

In scope:

- `StackFit`, `StackParentData`, `RenderStack`: layout and intrinsics.

Out of scope (and where it goes):

- `AlignmentDirectional`, start/end positioning for RTL: Phase 4 with RTL mirroring.
- `clipBehavior` and painting order: `tantu-view` (layout only records geometry; children
  outside the stack keep their offsets).
- `IndexedStack` (shows one child): later, when a widget needs it.
- App-wide overlays (tooltips, menus, dialogs): the Phase 3 `Overlay` item, which uses its own
  anchored positioning, not this stack.

## Public API

Crate root `tantu_layout`.

```rust
/// How non-positioned children are constrained (Flutter's `StackFit`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StackFit {
    /// The incoming constraints, loosened (children may be smaller than the stack).
    Loose,
    /// Tight at the largest allowed size (children fill the stack).
    Expand,
    /// The incoming constraints unchanged.
    Passthrough,
}

/// Parent data read by `RenderStack`: distances from the stack's edges, and an optional size.
/// A child is *positioned* when it has this parent data with at least one field set.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StackParentData {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl StackParentData {
    /// True if any field is set.
    pub fn is_positioned(&self) -> bool;
    /// All four edges at 0: the child covers the stack (`Positioned.fill`).
    pub const fn fill() -> Self;
}

/// Layers its children; non-positioned ones size the stack (Flutter's `RenderStack`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderStack {
    /// Where non-positioned children go, and positioned ones on an axis with no edge set.
    pub alignment: Alignment,
    /// How non-positioned children are constrained.
    pub fit: StackFit,
}

impl RenderStack {
    /// `alignment: TOP_LEFT`, `fit: Loose` (Flutter's defaults with left-to-right text).
    pub const fn new() -> Self;
}
impl Default for RenderStack { /* new() */ }
```

## Behavior

### Types

- **LAYOUT-STACK-01:** `RenderStack::new()` and `default()` are `{ TOP_LEFT, Loose }`.
  `StackParentData::default()` has every field `None` and is not positioned; any field set
  makes it positioned; `fill()` has the four edges at 0 and no size.

### Non-positioned children and the stack's size

- **LAYOUT-STACK-02:** Non-positioned children (no `StackParentData`, one with no field set, or
  parent data of another type) are laid out in order with `constraints.loosen()` (`Loose`),
  `BoxConstraints::tight(constraints.biggest())` (`Expand`) or `constraints` (`Passthrough`).
- **LAYOUT-STACK-03:** With at least one non-positioned child, the stack's size is the largest
  width and the largest height among them (separately), constrained by the incoming
  constraints. With none, it is `constraints.biggest()` when that is finite, otherwise the
  incoming minimum size (`smallest`) on an unbounded axis.
- **LAYOUT-STACK-04:** Each non-positioned child is placed at
  `alignment.along_offset(stack size − child size)`.

### Positioned children

- **LAYOUT-STACK-05:** A positioned child's width constraint: tight at `width` when set (the edges
  then only position it); otherwise, with both `left` and `right` set, tight at
  `stack width − left − right` (not below 0); otherwise unconstrained (`0..=∞`). The height
  constraint works the same with `height`, `top` and `bottom`. Positioned children are laid
  out after the stack's size is known, and never change it.
- **LAYOUT-STACK-06:** A positioned child's x offset is `left` when set; otherwise
  `stack width − right − child width` when `right` is set; otherwise from `alignment` on that
  axis (`(alignment.x + 1) / 2 · (stack width − child width)`). The y offset works the same
  with `top`, `bottom` and `alignment.y`. Children may extend outside the stack (negative or
  large offsets are kept).

### Intrinsics

- **LAYOUT-STACK-07:** Each intrinsic (min and max, width and height) is the largest of the
  non-positioned children's corresponding intrinsics, 0 with none; positioned children are
  ignored.

### Robustness

- **LAYOUT-STACK-08:** No children: the size of LAYOUT-STACK-03. Nothing panics for any
  constraints, alignment, fit or parent-data values (NaN, infinities, negative distances and
  sizes); `StackParentData` fields are used as given (a negative `left` places the child
  partly outside), and a NaN distance or size counts as not set.

## Performance and allocation

O(n) per layout (two passes over the indices: non-positioned, then positioned), no allocation.

## Open questions

Resolved (2026-10-10, decided by the agent: the user said "your pick"; review these):

1. **Default alignment `TOP_LEFT`**, replaced by the directional default when RTL lands.
2. **A NaN field in `StackParentData` counts as not set.**
3. **Only positioned children, unbounded constraints:** the minimum on the unbounded axis
   (LAYOUT-STACK-03), instead of Flutter's assertion.
