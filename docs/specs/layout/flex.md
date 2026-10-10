# Flex layout

- **Status:** Agreed
- **Crate:** `tantu-layout`
- **Plan item:** Phase 2, `tantu-layout` → "Flex"
- **Related:** [ADR 0002](../../adr/0002-flutter-structure-and-layout-protocol.md),
  [layout tree](tree.md), [single-child layouts](single-child.md) (`Alignment` is not used here;
  flex has its own alignment enums, as in Flutter)

## Purpose

`RenderFlex` lays children out in a line, horizontally (`Row`) or vertically (`Column`), with
Flutter's flex algorithm: inflexible children take their natural size, flexible children
(`Expanded`, `Flexible`, `Spacer`) share the remaining space in proportion to their flex factor,
and the free space is distributed by the main-axis alignment. It is the layout most screens are
built from, so it follows Flutter's `RenderFlex` exactly, including the newer `spacing`
property (Flutter 3.27).

## Scope

In scope:

- `Axis`, `MainAxisAlignment`, `MainAxisSize`, `CrossAxisAlignment`, `FlexFit`,
  `FlexParentData`, and `RenderFlex` (layout, intrinsics, overflow amount).

Out of scope (and where it goes):

- `CrossAxisAlignment::Baseline` and `TextBaseline`: with text (baselines need `tantu-text`).
- `TextDirection` and `VerticalDirection` (start/end mirroring for RTL, bottom-up columns):
  with i18n and RTL mirroring (Phase 4). Until then "start" is the left (row) or top (column).
- The widgets (`Row`, `Column`, `Expanded`, `Flexible`, `Spacer`): `tantu-widgets`. `Spacer` is
  an `Expanded` around an empty box, as in Flutter, so it needs nothing here.
- The overflow indicator (Flutter's yellow-and-black stripes): `tantu-view` paints it in debug
  builds from `RenderFlex::overflow`.

## Public API

Crate root `tantu_layout`. All enums are `Clone + Copy + Debug + PartialEq + Eq + Hash`.

```rust
/// A direction: the main axis of a flex.
pub enum Axis { Horizontal, Vertical }
impl Axis {
    /// The other axis.
    pub fn flip(self) -> Axis;
}

/// How free space on the main axis is distributed (Flutter's `MainAxisAlignment`).
pub enum MainAxisAlignment { Start, End, Center, SpaceBetween, SpaceAround, SpaceEvenly }

/// How much main-axis space the flex takes (Flutter's `MainAxisSize`).
pub enum MainAxisSize { Min, Max }

/// How children are placed on the cross axis (Flutter's `CrossAxisAlignment`, without
/// `Baseline` for now).
pub enum CrossAxisAlignment { Start, End, Center, Stretch }

/// Whether a flexible child must fill its share (`Tight`, `Expanded`) or may be smaller
/// (`Loose`, `Flexible`).
pub enum FlexFit { Tight, Loose }

/// Parent data read by `RenderFlex` (set with `LayoutTree::set_parent_data`). A child without
/// it, or with `flex == 0`, is inflexible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlexParentData { pub flex: u32, pub fit: FlexFit }
impl FlexParentData {
    /// `Expanded(flex)`: `fit: Tight`.
    pub const fn expanded(flex: u32) -> Self;
    /// `Flexible(flex)`: `fit: Loose`.
    pub const fn flexible(flex: u32) -> Self;
}

/// Lays out its children in a line along `direction` (Flutter's `RenderFlex`).
#[derive(Clone, Copy, Debug)]
pub struct RenderFlex {
    pub direction: Axis,
    pub main_axis_alignment: MainAxisAlignment,
    pub main_axis_size: MainAxisSize,
    pub cross_axis_alignment: CrossAxisAlignment,
    /// Space between adjacent children, on top of the alignment's.
    pub spacing: f32,
    /* private: the overflow of the last layout */
}

impl RenderFlex {
    /// A flex along `direction` with Flutter's defaults: `Start`, `Max`, `Center`, spacing 0.
    pub const fn new(direction: Axis) -> Self;
    /// `RenderFlex::new(Axis::Horizontal)` (`Row`).
    pub const fn row() -> Self;
    /// `RenderFlex::new(Axis::Vertical)` (`Column`).
    pub const fn column() -> Self;
    /// How far the children overflowed the main axis in the last layout (0 if they fit).
    pub fn overflow(&self) -> f32;
}

/// Compares the properties only (not the last overflow), so `LayoutTree::set` with unchanged
/// properties doesn't re-lay out.
impl PartialEq for RenderFlex { /* ... */ }
```

## Behavior

Below, "main" and "cross" are the extents along `direction` and the other axis: for a row,
main is the width and cross the height. `max_main`, `max_cross` are the incoming maxima on
those axes; `n` is the number of children. A child is *flexible* when its parent data is a
`FlexParentData` with `flex > 0`. `spacing` that is negative or NaN counts as 0.

### Types

- **LAYOUT-FLEX-01:** `RenderFlex::new(d)` has `direction: d`, `Start`, `Max`, `Center`, spacing 0
  and overflow 0; `row()`/`column()` are `new(Horizontal)`/`new(Vertical)`. `Axis::flip` swaps
  the two. `FlexParentData::expanded(f)` is `{ flex: f, fit: Tight }` and `flexible(f)` is
  `{ flex: f, fit: Loose }`.
- **LAYOUT-FLEX-02:** `RenderFlex` equality compares `direction`, the three alignments/sizes and
  `spacing` (as `f32 ==`), and ignores the last overflow.

### Sizing children

- **LAYOUT-FLEX-03:** Inflexible children are laid out first, in order, with main `0..=∞` and cross
  `0..=max_cross` (`Stretch`: tight at `max_cross`). Their main sizes, plus
  `spacing · (n − 1)`, make the *allocated* space.
- **LAYOUT-FLEX-04:** With bounded `max_main`, the free space is `max(0, max_main − allocated)`, and
  each flexible child's share is `free · flex / total_flex`; the last flexible child (in child
  order) gets what is left after the others, so the shares add up to the free space exactly.
  A `Tight` child is laid out with main tight at its share (`Expanded`); a `Loose` child with
  main `0..=share` (`Flexible`). Cross constraints are as for inflexible children.
- **LAYOUT-FLEX-05:** With unbounded `max_main` (a row in a horizontally unbounded parent),
  flexible children are laid out like inflexible ones (main `0..=∞`), instead of Flutter's
  error, and a `tracing` warning is logged in debug builds.

### Sizing the flex

- **LAYOUT-FLEX-06:** The main size is `max_main` for `MainAxisSize::Max` when `max_main` is
  bounded, otherwise the allocated space (now including the flexible children's actual sizes);
  then constrained by the incoming constraints. The cross size is the largest child cross
  extent, constrained by the incoming constraints (so 0 without children, or the minimum).
- **LAYOUT-FLEX-07:** `overflow()` is `max(0, allocated − main size)` after the layout: how far the
  children (with spacing) extend past the flex's end. Children are still placed as if there
  were room, so they extend past the end.

### Positioning children

- **LAYOUT-FLEX-08:** The remaining space is `max(0, main size − allocated)`. Leading space and
  the space between children (on top of `spacing`) come from `main_axis_alignment`: `Start`
  (0, 0), `End` (remaining, 0), `Center` (remaining / 2, 0), `SpaceBetween` (0, remaining /
  (n − 1), or 0 for one child), `SpaceAround` (remaining / 2n, remaining / n), `SpaceEvenly`
  (remaining / (n + 1), remaining / (n + 1)).
- **LAYOUT-FLEX-09:** Children are placed in order from the start: the first at the leading space,
  each next one after the previous child's main extent plus the between space and `spacing`.
- **LAYOUT-FLEX-10:** On the cross axis, a child is at 0 (`Start`, `Stretch`), at
  `cross size − child cross` (`End`), or at half that (`Center`). A child larger than the
  cross size gets a negative offset with `End` and `Center`.
- **LAYOUT-FLEX-11:** For a column the same rules apply with width and height swapped (offsets
  `(cross, main)` instead of `(main, cross)`).

### Intrinsics

- **LAYOUT-FLEX-12:** Main-axis intrinsics (min and max width of a row, height of a column) at a
  cross extent: the sum of the inflexible children's corresponding intrinsics, plus the largest
  `child intrinsic / flex` among flexible children times the total flex, plus
  `spacing · (n − 1)`. Children are queried at the given cross extent.
- **LAYOUT-FLEX-13:** Cross-axis intrinsics (height of a row, width of a column) at a main extent:
  the largest child cross intrinsic, where each inflexible child is queried at its max main
  intrinsic (at an infinite cross extent) and each flexible child at its share of the space
  left by the inflexible children and spacing (`max(0, (extent − inflexible − spacing) /
  total_flex) · flex`; at an infinite extent, an infinite share).

### Robustness

- **LAYOUT-FLEX-14:** No children: the size is the main size of LAYOUT-FLEX-06 with allocated 0 and
  cross 0, constrained; overflow 0. Nothing panics for any constraints, spacing, flex factors
  (including `u32::MAX` and a total that overflows `u32`, which is summed as `f64`) or child
  sizes; a child with parent data of another type is inflexible.

## Performance and allocation

O(n) per layout plus the children's layouts; no allocation (flexible children are found by a
second pass over the indices, not collected into a list).

## Open questions

Resolved (2026-10-10, decided by the agent: the user said "your pick"; review these):

1. **Flexible children in an unbounded main axis** are laid out as inflexible
   (LAYOUT-FLEX-05) with a debug warning, instead of Flutter's error.
2. **The last overflow is kept in the layout object** (private, read through `overflow()`,
   excluded from equality), for `tantu-view`'s debug overflow indicator.
3. **`TextDirection`/`VerticalDirection` come with RTL in Phase 4**, as fields defaulting to
   left-to-right and down.
4. **`flex` is `u32`**, with totals summed in `f64`.
