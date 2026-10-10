# Wrap layout

- **Status:** Implemented
- **Crate:** `tantu-layout`
- **Plan item:** Phase 2, `tantu-layout` → "`RenderWrap`"
- **Related:** [ADR 0002](../../adr/0002-flutter-structure-and-layout-protocol.md),
  [layout tree](tree.md), [flex](flex.md) (`Axis`)

## Purpose

`RenderWrap` lays children out in runs: along the main axis until the next child doesn't fit,
then on a new run below (or beside, for a vertical wrap). It is Flutter's `RenderWrap`, used for
chips, tags, toolbars that reflow and any "flow of items" that should break into lines instead
of overflowing.

## Scope

In scope:

- `WrapAlignment`, `WrapCrossAlignment`, `RenderWrap`: layout and intrinsics.

Out of scope (and where it goes):

- `TextDirection`/`VerticalDirection` (RTL and bottom-up runs): Phase 4, with RTL mirroring.
- `clipBehavior` and painting: `tantu-view`.
- Exact intrinsic heights through dry layout: the tree has no dry layout yet (layout tree spec,
  out of scope); intrinsics use Flutter's older intrinsic-based estimate (LAYOUT-WRAP-08).

## Public API

Crate root `tantu_layout`. Enums are `Clone + Copy + Debug + PartialEq + Eq + Hash`.

```rust
/// How children are distributed within a run, or runs within the wrap (Flutter's
/// `WrapAlignment`; the same meanings as `MainAxisAlignment`).
pub enum WrapAlignment { Start, End, Center, SpaceBetween, SpaceAround, SpaceEvenly }

/// How children are placed on the cross axis within their run (Flutter's
/// `WrapCrossAlignment`).
pub enum WrapCrossAlignment { Start, End, Center }

/// Lays out children in runs along `direction`, starting a new run when the next child
/// doesn't fit (Flutter's `RenderWrap`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderWrap {
    pub direction: Axis,
    /// Distribution of children within each run.
    pub alignment: WrapAlignment,
    /// Space between children in a run.
    pub spacing: f32,
    /// Distribution of the runs on the cross axis.
    pub run_alignment: WrapAlignment,
    /// Space between runs.
    pub run_spacing: f32,
    /// Placement of children on the cross axis within their run.
    pub cross_axis_alignment: WrapCrossAlignment,
}

impl RenderWrap {
    /// Flutter's defaults: `Start` everywhere, spacing 0.
    pub const fn new(direction: Axis) -> Self;
    /// `new(Axis::Horizontal)`.
    pub const fn horizontal() -> Self;
    /// `new(Axis::Vertical)`.
    pub const fn vertical() -> Self;
}
```

## Behavior

"Main" and "cross" are as in the flex spec (for a horizontal wrap: width and height). The
*main limit* is the incoming maximum main extent. Negative or NaN `spacing` and `run_spacing`
count as 0.

- **LAYOUT-WRAP-01:** `new(d)` has `direction: d`, `Start` for `alignment`, `run_alignment` and
  `cross_axis_alignment`, and both spacings 0; `horizontal()`/`vertical()` are
  `new(Horizontal)`/`new(Vertical)`. Equality compares all fields.
- **LAYOUT-WRAP-02:** Every child is laid out with main `0..=main limit` and cross `0..=∞`.
- **LAYOUT-WRAP-03:** Runs: children are taken in order; a child starts a new run when the current
  run has at least one child and `run main + spacing + child main > main limit`. So a child
  larger than the limit sits alone in its run (and overflows it), and an unbounded limit gives
  one run.
- **LAYOUT-WRAP-04:** A run's main extent is the sum of its children's main extents plus
  `spacing` between them; its cross extent is its largest child cross extent. The wrap's size is
  `constraints.constrain((largest run main, sum of run cross extents + run_spacing · (runs −
  1)))`; with no children, `constraints.constrain(0, 0)`.
- **LAYOUT-WRAP-05:** Runs are placed on the cross axis by `run_alignment` over the free cross
  space `max(0, cross size − runs' total cross extent)`, with the same leading and between
  spaces as `MainAxisAlignment` (LAYOUT-FLEX-08), plus `run_spacing` between runs.
- **LAYOUT-WRAP-06:** Within a run, children are placed on the main axis by `alignment` over the
  run's free space `max(0, main size − run main)`, the same way, plus `spacing` between
  children.
- **LAYOUT-WRAP-07:** On the cross axis, a child is at its run's cross position plus 0 (`Start`),
  `run cross − child cross` (`End`) or half that (`Center`). A vertical wrap swaps the axes
  (offsets `(cross, main)`).
- **LAYOUT-WRAP-08:** Intrinsics, for a horizontal wrap (vertical mirrors it): min intrinsic width
  is the largest child min intrinsic width; max intrinsic width is the sum of the children's
  max intrinsic widths plus `spacing · (n − 1)`; the intrinsic heights at a width are the
  height of the runs formed by each child's max intrinsic width (capped at that width) as its
  main extent and its max intrinsic height at that width as its cross extent, with the same
  run-breaking rule and spacings (Flutter's pre-dry-layout estimate). Arguments are passed to
  children as `∞` for the main-axis queries.
- **LAYOUT-WRAP-09:** No children: size `constraints.constrain(0, 0)`, intrinsics 0. Nothing panics
  for any constraints, spacings or child sizes; run breaking never loops (each child is placed
  exactly once).

## Performance and allocation

O(n) per layout, no allocation: a first pass sizes children and measures runs on the fly, a
second pass places them, recomputing the run breaks from the recorded child sizes
(`LayoutChildren::size`) instead of storing run lists.

## Open questions

Resolved (2026-10-10, decided by the agent: the user said "your pick"; review these):

1. **Intrinsic heights use Flutter's older estimate** (LAYOUT-WRAP-08) until the tree has dry
   layout.
2. **Run breaking compares exactly** (`>`).
3. **Two passes instead of stored runs**, keeping layout allocation-free.
