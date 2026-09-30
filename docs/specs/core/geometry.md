# Geometry

- **Status:** Implemented
- **Crate:** `tantu-core`
- **Plan item:** Phase 0, "`tantu-core` → geometry"
- **Related:** [ADR 0002](../../adr/0002-flutter-structure-and-layout-protocol.md) (layout
  protocol), [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md) (Scene)

## Purpose

The basic 2D types every other crate uses: positions, offsets, sizes, rectangles, edge insets and
2D affine transforms. Layout uses them for constraints, sizes and child offsets; the Scene uses
them for command geometry and transforms; hit-testing, culling and damage tracking use rectangle
tests.

All values are **logical pixels** in `f32`, with the origin at the top left and **y pointing
down** (`AGENTS.md`: logical pixels everywhere above the renderer).

## Scope

In scope:

- `Point`, `Vec2`, `Size`, `Rect`, `EdgeInsets`, `Affine` and their arithmetic.
- Rectangle tests used by hit-testing, culling and damage tracking (`contains`, `overlaps`,
  `intersect`, `union`).
- Behavior for NaN, infinite, negative and empty values.

Out of scope:

- `Color` (its own spec, `core/color.md`).
- Rounded rectangles, paths, shapes: `tantu-scene`.
- Directional insets (start/end, for RTL mirroring): `tantu-layout`, with the i18n work.
- Physical-pixel and integer types, pixel snapping: the renderers and `tantu-platform`.
- 3D and perspective transforms (Flutter's `Matrix4`). Only 2D affine transforms are supported.
- Serialization (`serde`). Decided with the Scene spec, which needs it.
- Interop with `kurbo` (which is `f64`). `From` impls can live in the crates that use kurbo.

## Public API

Module `tantu_core::geometry`, with every type re-exported at the crate root.

All six types derive `Clone`, `Copy`, `Debug` and `PartialEq` (exact float comparison). They
implement `Default`: all zeros, except `Affine`, whose default is the identity. Fields are public
unless stated otherwise.

```rust
/// A position in logical pixels.
pub struct Point { pub x: f32, pub y: f32 }

impl Point {
    /// The origin, (0, 0).
    pub const ZERO: Point;
    /// A point at (x, y).
    pub const fn new(x: f32, y: f32) -> Self;
    /// The vector from the origin to this point.
    pub const fn to_vec2(self) -> Vec2;
    /// True if both coordinates are finite.
    pub fn is_finite(self) -> bool;
}
// Point + Vec2 -> Point, Point - Vec2 -> Point, Point - Point -> Vec2
// (and the AddAssign / SubAssign forms with a Vec2).

/// A displacement (offset) in logical pixels. The parent-to-child offset in layout is a `Vec2`.
pub struct Vec2 { pub x: f32, pub y: f32 }

impl Vec2 {
    /// The zero vector.
    pub const ZERO: Vec2;
    /// A vector (x, y).
    pub const fn new(x: f32, y: f32) -> Self;
    /// The point at this offset from the origin.
    pub const fn to_point(self) -> Point;
    /// Euclidean length.
    pub fn length(self) -> f32;
    /// True if both components are finite.
    pub fn is_finite(self) -> bool;
}
// Vec2 + Vec2, Vec2 - Vec2, -Vec2, Vec2 * f32 (and AddAssign / SubAssign).

/// A width and height in logical pixels. May be infinite: unbounded layout constraints use
/// `f32::INFINITY`.
pub struct Size { pub width: f32, pub height: f32 }

impl Size {
    /// 0 × 0.
    pub const ZERO: Size;
    /// ∞ × ∞.
    pub const INFINITY: Size;
    /// A size of width × height, stored as given.
    pub const fn new(width: f32, height: f32) -> Self;
    /// True if the size has no area: either side is ≤ 0 or NaN.
    pub fn is_empty(self) -> bool;
    /// True if both sides are finite.
    pub fn is_finite(self) -> bool;
    /// Component-wise minimum.
    pub fn min(self, other: Size) -> Size;
    /// Component-wise maximum.
    pub fn max(self, other: Size) -> Size;
    /// A rect with this size at the origin.
    pub const fn to_rect(self) -> Rect;
}

/// An axis-aligned rectangle, stored as its edges (Flutter's representation).
pub struct Rect { pub left: f32, pub top: f32, pub right: f32, pub bottom: f32 }

impl Rect {
    /// All edges at 0.
    pub const ZERO: Rect;
    /// From edges, stored as given (not reordered).
    pub const fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self;
    /// From the top-left corner and a size.
    pub fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self;
    /// From the top-left corner and a size.
    pub fn from_origin_size(origin: Point, size: Size) -> Self;
    /// `right - left` (negative if the edges are reversed).
    pub fn width(self) -> f32;
    /// `bottom - top` (negative if the edges are reversed).
    pub fn height(self) -> f32;
    /// Width and height.
    pub fn size(self) -> Size;
    /// The top-left corner.
    pub const fn origin(self) -> Point;
    /// The center point.
    pub fn center(self) -> Point;
    /// True if the rect has no area: width or height ≤ 0, or any edge NaN.
    pub fn is_empty(self) -> bool;
    /// True if all edges are finite.
    pub fn is_finite(self) -> bool;
    /// True if `point` is inside. Left and top edges are inside, right and bottom are not.
    pub fn contains(self, point: Point) -> bool;
    /// True if the two rects share an area greater than zero.
    pub fn overlaps(self, other: Rect) -> bool;
    /// The shared area, or `None` if they don't overlap.
    pub fn intersect(self, other: Rect) -> Option<Rect>;
    /// The smallest rect containing both. Empty rects are ignored.
    pub fn union(self, other: Rect) -> Rect;
    /// Moved by `offset`.
    pub fn translate(self, offset: Vec2) -> Rect;
    /// Grown by `delta` on every side.
    pub fn inflate(self, delta: f32) -> Rect;
    /// Shrunk by `delta` on every side.
    pub fn deflate(self, delta: f32) -> Rect;
}

/// Distances from each edge of a box, in logical pixels (Flutter's name and meaning). Used for
/// padding, margins and borders. May be negative.
pub struct EdgeInsets { pub left: f32, pub top: f32, pub right: f32, pub bottom: f32 }

impl EdgeInsets {
    /// No insets.
    pub const ZERO: EdgeInsets;
    /// The same inset on all four sides.
    pub const fn all(value: f32) -> Self;
    /// `horizontal` on left and right, `vertical` on top and bottom.
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self;
    /// From the four sides.
    pub const fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self;
    /// `left + right`.
    pub fn horizontal(self) -> f32;
    /// `top + bottom`.
    pub fn vertical(self) -> f32;
    /// `size` minus the insets, never below zero on either side.
    pub fn deflate_size(self, size: Size) -> Size;
    /// `size` plus the insets.
    pub fn inflate_size(self, size: Size) -> Size;
    /// `rect` with each edge moved inward by its inset.
    pub fn deflate_rect(self, rect: Rect) -> Rect;
    /// `rect` with each edge moved outward by its inset.
    pub fn inflate_rect(self, rect: Rect) -> Rect;
}
// EdgeInsets + EdgeInsets (component-wise).

/// A 2D affine transform. Coefficients `[a, b, c, d, e, f]` map (x, y) to
/// (a·x + c·y + e, b·x + d·y + f). Coefficients are private; use `new` and `coeffs`.
pub struct Affine { /* [f32; 6] */ }

impl Affine {
    /// The identity transform.
    pub const IDENTITY: Affine;
    /// From coefficients `[a, b, c, d, e, f]`.
    pub const fn new(coeffs: [f32; 6]) -> Self;
    /// The coefficients `[a, b, c, d, e, f]`.
    pub const fn coeffs(self) -> [f32; 6];
    /// Translation by `offset`.
    pub const fn translate(offset: Vec2) -> Self;
    /// Uniform scale about the origin.
    pub const fn scale(s: f32) -> Self;
    /// Scale about the origin by `sx` horizontally and `sy` vertically.
    pub const fn scale_non_uniform(sx: f32, sy: f32) -> Self;
    /// Rotation about the origin by `radians`. With y pointing down, a positive angle turns
    /// clockwise on screen.
    pub fn rotate(radians: f32) -> Self;
    /// `a·d − b·c`.
    pub fn determinant(self) -> f32;
    /// The inverse, or `None` if the transform can't be inverted.
    pub fn inverse(self) -> Option<Affine>;
    /// Applies the linear part only (no translation), for offsets and directions.
    pub fn transform_vec(self, v: Vec2) -> Vec2;
    /// The smallest axis-aligned rect containing the transformed corners of `rect`.
    pub fn transform_rect_bbox(self, rect: Rect) -> Rect;
    /// True if all coefficients are finite.
    pub fn is_finite(self) -> bool;
}
// Affine * Affine -> Affine, Affine * Point -> Point.
```

## Behavior

General

- **CORE-GEOM-01:** `Default` gives the zero value for `Point`, `Vec2`, `Size`, `Rect` and
  `EdgeInsets`, and `Affine::IDENTITY` for `Affine`.
- **CORE-GEOM-02:** No function or operator in this module panics, for any input, including NaN,
  infinities, negative sizes and reversed rect edges. Arithmetic follows IEEE 754, so NaN
  propagates through arithmetic results.
- **CORE-GEOM-03:** Constructors store what they're given: nothing is clamped, reordered or
  normalized, except where a rule below says so.
- **CORE-GEOM-04:** `is_finite` on every type is true exactly when all its components (or
  coefficients) are finite.

Point and Vec2

- **CORE-GEOM-05:** `Point + Vec2`, `Point - Vec2` give a `Point`; `Point - Point` gives the `Vec2`
  from the right operand to the left one; all component-wise. `to_vec2` and `to_point` keep the
  components.
- **CORE-GEOM-06:** `Vec2` addition, subtraction, negation and multiplication by `f32` are
  component-wise. `length` is `sqrt(x² + y²)`.

Size

- **CORE-GEOM-07:** `Size::is_empty` is true when either side is ≤ 0 or NaN. An infinite positive
  size is not empty.
- **CORE-GEOM-08:** `Size::min` and `max` are component-wise and use `f32::min` / `f32::max`, so a
  NaN side yields the other size's side.
- **CORE-GEOM-09:** `Size::to_rect` is `Rect::from_ltrb(0, 0, width, height)`.

Rect

- **CORE-GEOM-10:** `from_ltwh(l, t, w, h)` and `from_origin_size` give `right = left + width`
  and `bottom = top + height`. `from_ltrb` keeps the edges as given, even if `right < left`.
- **CORE-GEOM-11:** `width` is `right − left` and `height` is `bottom − top`, and may be negative.
  `size` is `Size::new(width, height)`. `origin` is `(left, top)`. `center` is the midpoint of the
  edges.
- **CORE-GEOM-12:** `Rect::is_empty` is true when width or height is ≤ 0, or any edge is NaN.
- **CORE-GEOM-13:** `contains(p)` is true exactly when `left ≤ p.x < right` and
  `top ≤ p.y < bottom`. An empty rect contains nothing, and a point with a NaN coordinate is
  never contained.
- **CORE-GEOM-14:** `overlaps(other)` is true exactly when the two rects share an area greater
  than zero. Rects that only touch along an edge or at a corner don't overlap. An empty rect
  overlaps nothing.
- **CORE-GEOM-15:** `intersect(other)` is `None` when the rects don't overlap (rule 14), and
  otherwise the rect with the larger `left`/`top` and the smaller `right`/`bottom`.
- **CORE-GEOM-16:** `union(other)` is the rect with the smaller `left`/`top` and the larger
  `right`/`bottom`. If one operand is empty it returns the other; if both are empty it returns
  `self`.
- **CORE-GEOM-17:** `translate(v)` adds `v.x` to `left` and `right` and `v.y` to `top` and
  `bottom`.
- **CORE-GEOM-18:** `inflate(d)` moves every edge outward by `d`; `deflate(d)` is `inflate(−d)`.
  Deflating past the center gives an empty rect with reversed edges; it is not clamped.

EdgeInsets

- **CORE-GEOM-19:** `all(v)` sets all four sides to `v`; `symmetric(h, v)` sets left and right to
  `h`, top and bottom to `v`; `from_ltrb` sets them as given. `horizontal` is `left + right`,
  `vertical` is `top + bottom`. `EdgeInsets + EdgeInsets` is component-wise.
- **CORE-GEOM-20:** `deflate_size(s)` is `(s.width − horizontal, s.height − vertical)`, each side
  clamped to at least 0 (a NaN side becomes 0). `inflate_size(s)` adds `horizontal` and
  `vertical` without clamping. An infinite size stays infinite.
- **CORE-GEOM-21:** `deflate_rect(r)` moves each edge of `r` inward by the matching inset
  (`left + insets.left`, `top + insets.top`, `right − insets.right`, `bottom − insets.bottom`);
  `inflate_rect` moves them outward. Neither clamps. Negative insets move edges the other way.

Affine

- **CORE-GEOM-22:** `Affine::new(c).coeffs() == c`. `Affine * Point` maps (x, y) to
  (a·x + c·y + e, b·x + d·y + f). `IDENTITY` is `[1, 0, 0, 1, 0, 0]` and leaves every point
  unchanged.
- **CORE-GEOM-23:** `translate(v)` is `[1, 0, 0, 1, v.x, v.y]`; `scale(s)` is
  `[s, 0, 0, s, 0, 0]`; `scale_non_uniform(sx, sy)` is `[sx, 0, 0, sy, 0, 0]`; `rotate(θ)` is
  `[cos θ, sin θ, −sin θ, cos θ, 0, 0]`, so it maps (1, 0) to (cos θ, sin θ).
- **CORE-GEOM-24:** Composition applies the right operand first: `(A * B) * p` equals
  `A * (B * p)` (within float rounding).
- **CORE-GEOM-25:** `transform_vec(v)` applies only the linear part: (a·x + c·y, b·x + d·y).
- **CORE-GEOM-26:** `transform_rect_bbox(r)` is the smallest rect with `left ≤ right` and
  `top ≤ bottom` that contains all four transformed corners of `r`. For translations and
  positive scales it equals the exactly transformed rect.
- **CORE-GEOM-27:** `determinant` is `a·d − b·c`.
- **CORE-GEOM-28:** `inverse` is `None` when the determinant is 0 or not finite, or when any
  coefficient of the result would not be finite. Otherwise, for any composition of a rotation, a
  scale with factors between 0.1 and 10 (possibly non-uniform) and a translation with components
  within ±1000, both `A * A.inverse()` and `A.inverse() * A` match `IDENTITY` within 1e-4 on
  `a`–`d` and within 1e-4 × max(1, |e|, |f|) on `e` and `f` (`e`, `f` being `A`'s translation).
  No accuracy is promised for ill-conditioned transforms: in `f32` it can't be.

## Performance and allocation

- **CORE-GEOM-29:** The types are plain values: `size_of` is 8 bytes for `Point`, `Vec2` and
  `Size`, 16 bytes for `Rect` and `EdgeInsets`, and 24 bytes for `Affine`.
- No function allocates. All are candidates for `#[inline]`; functions that can be `const` on
  Rust 1.85 are `const` as listed in the API.

## Open questions

Resolved (2026-09-30):

1. **One offset type or two?** Two. Flutter uses a single `Offset`; we keep `Point` (where) and
   `Vec2` (how far), as kurbo and most Rust graphics crates do, so the type system catches adding
   two positions. The parent-to-child offset in layout is a `Vec2`.
2. **`Insets` or `EdgeInsets`?** `EdgeInsets`, Flutter's name, so Flutter users can read Tantu code
   straight away. `PLAN.md` updated to match.
3. **Rect representation.** Edges (`left, top, right, bottom`) like Flutter, which makes
   intersection, union and clipping direct, with `origin()` and `size()` accessors.
4. **Accuracy of `Affine::inverse` (CORE-GEOM-28).** The first wording promised identity within
   1e-5 for any coefficients within ±1e4 and |det| ≥ 1e-4. That allows condition numbers near
   1e12, which `f32` can't meet. Found while writing the tests; the rule now covers
   well-conditioned transforms (rotation, scale 0.1–10, translation within ±1000), with a
   tolerance on `e`/`f` relative to the translation's size.
