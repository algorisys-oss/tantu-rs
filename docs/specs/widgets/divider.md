# Divider and VerticalDivider

- **Status:** Agreed (the user said "continue" on the draft, taking its proposals)
- **Crates:** `tantu-widgets` (plus one layout object added to `tantu-layout` and one widget to
  the layout widgets)
- **Plan item:** Phase 3, "Widgets" → "`Divider`"
- **Related:** [layout widgets](layout.md), [single-child layouts](../layout/single-child.md),
  [Text and Button](basic.md), Flutter's `Divider`, `VerticalDivider` and `LimitedBox`

## Purpose

A thin line that separates content: horizontal in a column (`Divider`), vertical in a row
(`VerticalDivider`). It's the simplest widget with a look, and the first to need "fill the
available width, but don't blow up when it's unbounded", which is Flutter's `LimitedBox`. That
layout object is added here so the slider and later widgets can use it too. The look is
Material 3's divider until `tantu-theme` arrives.

## Scope

In scope:

- `Divider`: `height` (the space it takes), `thickness`, `indent`, `end_indent`, `color`, all
  reactive. Flutter's names and defaults, except the default thickness (see below).
- `VerticalDivider`: the same with `width` instead of `height`.
- `LimitedBox` / `RenderLimitedBox` (Flutter's), amending the layout specs.
- Material 3 look: 1 px line in outline-variant `0xFFCAC4D0`, 16 px of space.

Out of scope (and where it goes):

- Theming (color, thickness and space from tokens): `tantu-theme`.
- Dividers with a label, or inset by the theme's list padding: later, with lists.
- A hairline (1 physical pixel) at any scale: Flutter's default thickness is 0 for that. Tantu
  uses 1 logical px (the Material 3 value) until renderers expose the scale to paint.
- RTL mirroring of `indent`/`end_indent`: Phase 4 i18n (decision 33 defers directional
  variants).

## Public API

```rust
// tantu-widgets
/// A horizontal line separating content (Flutter's `Divider`, Material 3 look).
pub struct Divider { /* height, thickness, indent, end_indent, color */ }

impl Divider {
    /// A divider taking 16 px of height, with a 1 px line in `0xFFCAC4D0` across the full
    /// width.
    pub fn new() -> Self;
    /// The height the widget takes; the line is centered in it (default 16).
    pub fn height(self, height: impl IntoProp<f32>) -> Self;
    /// The line's thickness (default 1).
    pub fn thickness(self, thickness: impl IntoProp<f32>) -> Self;
    /// Space before the line's start (default 0).
    pub fn indent(self, indent: impl IntoProp<f32>) -> Self;
    /// Space after the line's end (default 0).
    pub fn end_indent(self, end_indent: impl IntoProp<f32>) -> Self;
    /// The line's color.
    pub fn color(self, color: impl IntoProp<Color>) -> Self;
}

/// A vertical line separating content in a row (Flutter's `VerticalDivider`).
pub struct VerticalDivider { /* width, thickness, indent, end_indent, color */ }

impl VerticalDivider {
    pub fn new() -> Self;
    /// The width the widget takes; the line is centered in it (default 16).
    pub fn width(self, width: impl IntoProp<f32>) -> Self;
    pub fn thickness(self, thickness: impl IntoProp<f32>) -> Self;
    /// Space above the line's start.
    pub fn indent(self, indent: impl IntoProp<f32>) -> Self;
    /// Space below the line's end.
    pub fn end_indent(self, end_indent: impl IntoProp<f32>) -> Self;
    pub fn color(self, color: impl IntoProp<Color>) -> Self;
}

/// Caps the constraints its child gets on an axis only when that axis is unbounded
/// (Flutter's `LimitedBox`).
pub struct LimitedBox { /* max_width, max_height, child */ }

impl LimitedBox {
    /// No limits (both infinite).
    pub fn new() -> Self;
    pub fn max_width(self, max_width: impl IntoProp<f32>) -> Self;
    pub fn max_height(self, max_height: impl IntoProp<f32>) -> Self;
    pub fn child(self, child: impl View) -> Self;
}

// tantu-layout
/// Flutter's `RenderLimitedBox`: limits the child's maximum width (height) to `max_width`
/// (`max_height`) only when the incoming constraints are unbounded on that axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderLimitedBox {
    /// The limit on an unbounded width (default `f32::INFINITY`, no limit).
    pub max_width: f32,
    /// The limit on an unbounded height.
    pub max_height: f32,
}

impl RenderLimitedBox {
    pub const fn new(max_width: f32, max_height: f32) -> Self;
}
```

`Divider` builds two render elements: a `LimitedBox` with `max_width` 0 around a render
element with a `RenderConstrainedBox` of `BoxConstraints::expand(None, Some(height))`, which
holds the `DividerPaint`. So in a bounded width the divider spans it, and in an unbounded one
(a `Row`, a horizontal scroll view) it is 0 wide, as Flutter's is. `VerticalDivider` is the
same with the axes swapped. Colors: outline-variant `0xFFCAC4D0` (Material 3's divider color).

## Behavior

- **LAYOUT-SINGLE-16** (in [single-child layouts](../layout/single-child.md)): `RenderLimitedBox` lays out its child with the incoming constraints,
  except that on an axis whose incoming maximum is infinite the maximum becomes
  `constrain_width(max_width)` (`constrain_height(max_height)`, so a limit below the minimum
  is raised to it). The size is the child's, or the limited constraints' smallest size without
  a child. Intrinsics are the child's (0 without one). A limit that is negative or NaN counts
  as infinite. (Added to `tantu-layout` with this spec.)
- **WIDGETS-LAYOUT-07** (in [layout widgets](layout.md)): `LimitedBox` builds a `RenderLimitedBox` with reactive `max_width` and
  `max_height` (default infinite), like the other layout widgets (WIDGETS-LAYOUT-01..03, 06).
  (Added to the layout widgets with this spec.)
- **WIDGETS-DIVIDER-01:** A `Divider` is `height` tall and as wide as the incoming maximum
  width, or 0 wide when that is unbounded. In a `Column` it spans the column's width.
- **WIDGETS-DIVIDER-02:** It paints one filled rectangle in `color`: from `x = indent` to
  `x = width − end_indent`, `thickness` tall and centered vertically (at `(height − thickness)
  / 2`). A negative or NaN `thickness`, `indent` or `end_indent` counts as 0, and nothing is
  painted when the line would have no width or no thickness. Checked against a golden
  (`crates/tantu-widgets/tests/goldens/divider.png`).
- **WIDGETS-DIVIDER-03:** A `VerticalDivider` is `width` wide and as tall as the incoming
  maximum height, or 0 tall when that is unbounded, and paints its line from `y = indent` to
  `y = height − end_indent`, `thickness` wide and centered horizontally.
- **WIDGETS-DIVIDER-04:** Every prop is reactive: a new `height` (`width` for the vertical one)
  re-lays out the divider at the next frame, and a new `thickness`, `indent`, `end_indent` or
  `color` only repaints it.

## Performance and allocation

A divider is two elements and one Scene command. Paint-only props go through `update_paint`;
only the extent goes through `update_render`.

## Open questions

Resolved (2026-10-11; the user said "continue" on the draft):

1. **`LimitedBox` rather than a custom render object.** The divider (and the slider after it)
   needs "fill a bounded width, else a fixed one". Flutter expresses that with `LimitedBox`
   around an expanding box, which is a general layout widget worth having. The alternative, a
   private `RenderDivider` leaf, is one element fewer but a new layout vocabulary.
2. **Thickness 1 by default** (Material 3), not Flutter's 0 (hairline), which needs the scale
   factor in paint. Revisit with the theme.
3. **One golden per widget** from here on (`WidgetTester::matches_golden` exists now), under
   `crates/tantu-widgets/tests/goldens/`. `Button` and `Tooltip` get theirs with the gallery.
