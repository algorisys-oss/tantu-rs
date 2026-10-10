# BoxConstraints

- **Status:** Agreed
- **Crate:** `tantu-layout`
- **Plan item:** Phase 2, `tantu-layout` → "`BoxConstraints`"
- **Related:** [ADR 0002](../../adr/0002-flutter-structure-and-layout-protocol.md),
  [ADR 0009](../../adr/0009-layout-tree-in-tantu-layout.md),
  [geometry](../core/geometry.md) (`Size`, `EdgeInsets`)

## Purpose

`BoxConstraints` is what a parent passes down in Flutter's layout protocol: a range of allowed
widths and a range of allowed heights. The child picks a size inside them and returns it. Every
layout algorithm (padding, flex, stack, …) and every custom layout builds, transforms and
applies constraints, so this type has to be cheap (`Copy`, 16 bytes), predictable and never
panic.

The API is Flutter's `BoxConstraints`, in snake_case, so Flutter's documentation and intuition
apply. Where Flutter asserts (in debug builds) on malformed constraints, Tantu defines a result
instead.

## Scope

In scope:

- The `BoxConstraints` value type: constructors, queries, transformations, and constraining a
  size.

Out of scope (and where it goes):

- The layout protocol itself, caching and relayout boundaries: layout tree spec
  (`docs/specs/layout/tree.md`).
- `constrain_size_and_attempt_to_preserve_aspect_ratio` (for images): added with the `Image`
  widget in Phase 3.
- `lerp` (animating constraints) and `Display` formatting: when something needs them.
- Sliver constraints (scrolling): Phase 3, with `ScrollView`.

## Public API

Crate root `tantu_layout`.

```rust
use tantu_core::{EdgeInsets, Size};

/// Allowed widths and heights for a box, in logical pixels: `min_width..=max_width` by
/// `min_height..=max_height`. The maxima may be `f32::INFINITY` (unbounded).
///
/// Values are stored as given. A *normalized* value (see [`BoxConstraints::is_normalized`]) has
/// finite, non-negative minima and maxima at least as large as the minima. Every method is
/// defined for any input and never panics; for constraints that aren't normalized, "the minimum
/// wins" (see [`BoxConstraints::constrain_width`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    /// Smallest allowed width.
    pub min_width: f32,
    /// Largest allowed width (may be infinite).
    pub max_width: f32,
    /// Smallest allowed height.
    pub min_height: f32,
    /// Largest allowed height (may be infinite).
    pub max_height: f32,
}

impl BoxConstraints {
    /// `0 ≤ width ≤ ∞`, `0 ≤ height ≤ ∞`: any size. Flutter's `const BoxConstraints()`.
    pub const UNCONSTRAINED: BoxConstraints;

    /// Constraints from the four values, stored as given.
    pub const fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self;
    /// Exactly `size`.
    pub const fn tight(size: Size) -> Self;
    /// Exactly the given width and/or height; an axis given as `None` is unconstrained
    /// (`0..=∞`).
    pub const fn tight_for(width: Option<f32>, height: Option<f32>) -> Self;
    /// Like `tight_for`, but an axis is tightened only when its value is finite
    /// (Flutter's `tightForFinite`, with `f32::INFINITY` meaning "not given").
    pub fn tight_for_finite(width: f32, height: f32) -> Self;
    /// `0..=size` on both axes.
    pub const fn loose(size: Size) -> Self;
    /// Exactly the given width and/or height; an axis given as `None` is tight at infinity
    /// (fill all space; only meaningful under a parent that bounds it).
    pub const fn expand(width: Option<f32>, height: Option<f32>) -> Self;

    /// `min_width.max(width.min(max_width))`: the width closest to `width` that these
    /// constraints allow; when they conflict (`min > max`) the minimum wins. A NaN width gives
    /// `min_width`.
    pub fn constrain_width(self, width: f32) -> f32;
    /// As `constrain_width`, for heights.
    pub fn constrain_height(self, height: f32) -> f32;
    /// Both axes constrained.
    pub fn constrain(self, size: Size) -> Size;
    /// The largest allowed size: `constrain(∞, ∞)` (so infinite on an unbounded axis).
    pub fn biggest(self) -> Size;
    /// The smallest allowed size: `constrain(0, 0)`.
    pub fn smallest(self) -> Size;

    /// `min_width >= max_width`.
    pub fn has_tight_width(self) -> bool;
    /// `min_height >= max_height`.
    pub fn has_tight_height(self) -> bool;
    /// Both axes tight: exactly one size is allowed.
    pub fn is_tight(self) -> bool;
    /// `max_width` is finite.
    pub fn has_bounded_width(self) -> bool;
    /// `max_height` is finite.
    pub fn has_bounded_height(self) -> bool;
    /// `min_width` is infinite (only expand-style constraints have it).
    pub fn has_infinite_width(self) -> bool;
    /// `min_height` is infinite.
    pub fn has_infinite_height(self) -> bool;
    /// `size` lies in both ranges (bounds included).
    pub fn is_satisfied_by(self, size: Size) -> bool;
    /// No NaN; minima finite and ≥ 0; each maximum ≥ its minimum (maxima may be infinite).
    pub fn is_normalized(self) -> bool;

    /// The nearest normalized constraints: a negative, NaN or infinite minimum becomes 0, a NaN
    /// maximum becomes infinite, and a maximum below its minimum is raised to the minimum.
    pub fn normalize(self) -> Self;
    /// Minima set to 0, maxima kept: the child may be smaller than before.
    pub fn loosen(self) -> Self;
    /// Each of the four values constrained into `outer`'s range for its axis (with
    /// `outer.constrain_width` / `constrain_height`): constraints that respect both `self`'s
    /// intent and `outer`, preferring `outer`.
    pub fn enforce(self, outer: BoxConstraints) -> Self;
    /// Tight on an axis given as `Some(v)`, at `constrain_width(v)` / `constrain_height(v)`;
    /// an axis given as `None` is unchanged.
    pub fn tighten(self, width: Option<f32>, height: Option<f32>) -> Self;
    /// Shrunk by `insets` (what a padded child gets): on each axis the minimum becomes
    /// `max(0, min − insets)` and the maximum `max(new min, max − insets)`. Negative insets
    /// grow the constraints.
    pub fn deflate(self, insets: EdgeInsets) -> Self;
    /// Width and height swapped (for code shared between horizontal and vertical layouts).
    pub fn flip(self) -> Self;
    /// Only the width range kept; the height becomes `0..=∞`.
    pub fn width_constraints(self) -> Self;
    /// Only the height range kept; the width becomes `0..=∞`.
    pub fn height_constraints(self) -> Self;
}

impl Default for BoxConstraints {
    /// [`BoxConstraints::UNCONSTRAINED`].
    fn default() -> Self;
}
```

## Behavior

### Construction

- **LAYOUT-CONS-01:** `new` stores the four values as given (no clamping, reordering or panics,
  NaN included). `UNCONSTRAINED` and `default()` are `(0, ∞, 0, ∞)`.
- **LAYOUT-CONS-02:** `tight(s)` is `(s.width, s.width, s.height, s.height)`; `loose(s)` is
  `(0, s.width, 0, s.height)`.
- **LAYOUT-CONS-03:** `tight_for(w, h)` is tight at `v` on an axis given as `Some(v)` and `0..=∞` on an
  axis given as `None`. `tight_for_finite(w, h)` is tight on an axis whose value is finite and
  `0..=∞` on one whose value is infinite or NaN.
- **LAYOUT-CONS-04:** `expand(w, h)` is tight at `v` on an axis given as `Some(v)` and tight at `∞`
  (`min = max = ∞`) on an axis given as `None`.

### Constraining

- **LAYOUT-CONS-05:** For normalized constraints, `constrain_width(w)` is `w` when
  `min_width ≤ w ≤ max_width`, `min_width` when below and `max_width` when above; the same for
  heights. `±∞` are ordinary values (`∞` constrains to `max_width`, which may itself be `∞`).
- **LAYOUT-CONS-06:** When a minimum exceeds its maximum, the minimum wins: every value constrains to
  `min`. A NaN input constrains to the minimum. A NaN bound is ignored on its side (a NaN
  maximum leaves values unbounded above, a NaN minimum leaves them unbounded below), following
  `f32::max`/`f32::min`. Nothing panics.
- **LAYOUT-CONS-07:** `constrain(s)` is `Size::new(constrain_width(s.width), constrain_height(s.height))`;
  `biggest()` is `constrain(∞, ∞)` and `smallest()` is `constrain(0, 0)`.

### Queries

- **LAYOUT-CONS-08:** `has_tight_width` is `min_width ≥ max_width` (true when the minimum wins);
  `is_tight` is both axes tight. `has_bounded_width` is `max_width` finite. `has_infinite_width`
  is `min_width` infinite. The height versions mirror them. With NaN involved, every comparison
  is false (so NaN bounds are neither tight nor bounded nor infinite).
- **LAYOUT-CONS-09:** `is_satisfied_by(s)` is true exactly when `min_width ≤ s.width ≤ max_width` and
  `min_height ≤ s.height ≤ max_height`. A NaN anywhere makes it false.
- **LAYOUT-CONS-10:** `is_normalized` is true exactly when no value is NaN, both minima are finite and
  ≥ 0, and each maximum is ≥ its minimum. `UNCONSTRAINED`, `tight` and `loose` of a finite,
  non-negative size are normalized; `expand` with a `None` axis is not (its minimum is
  infinite).

### Transformations

- **LAYOUT-CONS-11:** `normalize` returns the value unchanged when it is already normalized. Otherwise
  each minimum that is negative, NaN or infinite becomes 0, each NaN maximum becomes `∞`, and
  each maximum below its (new) minimum becomes that minimum. The result is always normalized.
- **LAYOUT-CONS-12:** `loosen` sets both minima to 0 and keeps the maxima.
- **LAYOUT-CONS-13:** `enforce(outer)` is `(outer.constrain_width(min_width),
  outer.constrain_width(max_width), outer.constrain_height(min_height),
  outer.constrain_height(max_height))`. For normalized inputs the result is normalized, lies
  inside `outer`, and equals `self` when `self` already lies inside `outer`.
- **LAYOUT-CONS-14:** `tighten(Some(w), h)` sets `min_width = max_width = constrain_width(w)`
  (and the same for the height); a `None` axis is unchanged.
- **LAYOUT-CONS-15:** `deflate(insets)` shrinks the width range by `insets.horizontal()` and the height
  range by `insets.vertical()`: new minimum `max(0, min − d)`, new maximum `max(new min,
  max − d)`. An infinite maximum stays infinite. For normalized input the result is normalized.
  Negative insets grow both ends.
- **LAYOUT-CONS-16:** `flip` swaps the width and height ranges; `flip(flip(c)) == c`.
- **LAYOUT-CONS-17:** `width_constraints` keeps `min_width`/`max_width` and sets the height to
  `0..=∞`; `height_constraints` mirrors it.

## Performance and allocation

`BoxConstraints` is `Copy` and 16 bytes. No method allocates. Methods are `#[inline]`-friendly
(small, no branches beyond comparisons). `new`, `tight`, `tight_for`, `loose`, `expand` and
`UNCONSTRAINED` are `const`.

## Open questions

Resolved (2026-10-10, agreed with the user):

1. **Malformed constraints are defined, not asserted.** Every result is defined ("the minimum
   wins", the NaN rules above) and nothing panics, matching `tantu-core`'s geometry. The layout
   tree may report malformed constraints in debug builds in its own spec.
2. **`expand` keeps Flutter's semantics:** an axis given as `None` is tight at infinity.
3. **Public fields**, as in Flutter and like `Size` and `Rect` in `tantu-core`.
