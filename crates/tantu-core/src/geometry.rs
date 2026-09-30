//! 2D geometry in logical pixels: [`Point`], [`Vec2`], [`Size`], [`Rect`], [`EdgeInsets`] and
//! [`Affine`].
//!
//! All values are `f32` logical pixels with the origin at the top left and y pointing down.
//! Nothing here panics, whatever the input: NaN and infinities propagate through arithmetic
//! (IEEE 754), and predicates such as [`Rect::contains`] treat NaN as "no".
//!
//! Spec: `docs/specs/core/geometry.md`.

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// A position in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// Horizontal coordinate, growing to the right.
    pub x: f32,
    /// Vertical coordinate, growing downward.
    pub y: f32,
}

impl Point {
    /// The origin, (0, 0).
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    /// A point at (x, y).
    pub const fn new(x: f32, y: f32) -> Self {
        let _ = (x, y);
        todo!()
    }

    /// The vector from the origin to this point.
    pub const fn to_vec2(self) -> Vec2 {
        todo!()
    }

    /// True if both coordinates are finite.
    pub fn is_finite(self) -> bool {
        todo!()
    }
}

impl Default for Point {
    fn default() -> Self {
        todo!()
    }
}

impl Add<Vec2> for Point {
    type Output = Point;
    fn add(self, rhs: Vec2) -> Point {
        let _ = rhs;
        todo!()
    }
}

impl Sub<Vec2> for Point {
    type Output = Point;
    fn sub(self, rhs: Vec2) -> Point {
        let _ = rhs;
        todo!()
    }
}

impl Sub for Point {
    type Output = Vec2;
    fn sub(self, rhs: Point) -> Vec2 {
        let _ = rhs;
        todo!()
    }
}

impl AddAssign<Vec2> for Point {
    fn add_assign(&mut self, rhs: Vec2) {
        let _ = rhs;
        todo!()
    }
}

impl SubAssign<Vec2> for Point {
    fn sub_assign(&mut self, rhs: Vec2) {
        let _ = rhs;
        todo!()
    }
}

/// A displacement (offset) in logical pixels. The parent-to-child offset in layout is a `Vec2`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    /// Horizontal component, positive to the right.
    pub x: f32,
    /// Vertical component, positive downward.
    pub y: f32,
}

impl Vec2 {
    /// The zero vector.
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    /// A vector (x, y).
    pub const fn new(x: f32, y: f32) -> Self {
        let _ = (x, y);
        todo!()
    }

    /// The point at this offset from the origin.
    pub const fn to_point(self) -> Point {
        todo!()
    }

    /// Euclidean length.
    pub fn length(self) -> f32 {
        todo!()
    }

    /// True if both components are finite.
    pub fn is_finite(self) -> bool {
        todo!()
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        todo!()
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        let _ = rhs;
        todo!()
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        let _ = rhs;
        todo!()
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        todo!()
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        let _ = rhs;
        todo!()
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        let _ = rhs;
        todo!()
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        let _ = rhs;
        todo!()
    }
}

/// A width and height in logical pixels. May be infinite: unbounded layout constraints use
/// `f32::INFINITY`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}

impl Size {
    /// 0 × 0.
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    /// ∞ × ∞.
    pub const INFINITY: Size = Size {
        width: f32::INFINITY,
        height: f32::INFINITY,
    };

    /// A size of width × height, stored as given.
    pub const fn new(width: f32, height: f32) -> Self {
        let _ = (width, height);
        todo!()
    }

    /// True if the size has no area: either side is ≤ 0 or NaN.
    pub fn is_empty(self) -> bool {
        todo!()
    }

    /// True if both sides are finite.
    pub fn is_finite(self) -> bool {
        todo!()
    }

    /// Component-wise minimum. A NaN side yields the other size's side.
    pub fn min(self, other: Size) -> Size {
        let _ = other;
        todo!()
    }

    /// Component-wise maximum. A NaN side yields the other size's side.
    pub fn max(self, other: Size) -> Size {
        let _ = other;
        todo!()
    }

    /// A rect with this size at the origin.
    pub const fn to_rect(self) -> Rect {
        todo!()
    }
}

impl Default for Size {
    fn default() -> Self {
        todo!()
    }
}

/// An axis-aligned rectangle, stored as its edges (Flutter's representation).
///
/// Edges are kept as given: a rect whose `right` is left of its `left` is valid data and
/// [empty](Rect::is_empty).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// X of the left edge.
    pub left: f32,
    /// Y of the top edge.
    pub top: f32,
    /// X of the right edge.
    pub right: f32,
    /// Y of the bottom edge.
    pub bottom: f32,
}

impl Rect {
    /// All edges at 0.
    pub const ZERO: Rect = Rect {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// From edges, stored as given (not reordered).
    pub const fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        let _ = (left, top, right, bottom);
        todo!()
    }

    /// From the top-left corner and a size.
    pub fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        let _ = (left, top, width, height);
        todo!()
    }

    /// From the top-left corner and a size.
    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        let _ = (origin, size);
        todo!()
    }

    /// `right - left` (negative if the edges are reversed).
    pub fn width(self) -> f32 {
        todo!()
    }

    /// `bottom - top` (negative if the edges are reversed).
    pub fn height(self) -> f32 {
        todo!()
    }

    /// Width and height.
    pub fn size(self) -> Size {
        todo!()
    }

    /// The top-left corner.
    pub const fn origin(self) -> Point {
        todo!()
    }

    /// The center point.
    pub fn center(self) -> Point {
        todo!()
    }

    /// True if the rect has no area: width or height ≤ 0, or any edge NaN.
    pub fn is_empty(self) -> bool {
        todo!()
    }

    /// True if all edges are finite.
    pub fn is_finite(self) -> bool {
        todo!()
    }

    /// True if `point` is inside. Left and top edges are inside, right and bottom are not.
    pub fn contains(self, point: Point) -> bool {
        let _ = point;
        todo!()
    }

    /// True if the two rects share an area greater than zero.
    pub fn overlaps(self, other: Rect) -> bool {
        let _ = other;
        todo!()
    }

    /// The shared area, or `None` if they don't overlap.
    pub fn intersect(self, other: Rect) -> Option<Rect> {
        let _ = other;
        todo!()
    }

    /// The smallest rect containing both. Empty rects are ignored; if both are empty, returns
    /// `self`.
    pub fn union(self, other: Rect) -> Rect {
        let _ = other;
        todo!()
    }

    /// Moved by `offset`.
    pub fn translate(self, offset: Vec2) -> Rect {
        let _ = offset;
        todo!()
    }

    /// Grown by `delta` on every side.
    pub fn inflate(self, delta: f32) -> Rect {
        let _ = delta;
        todo!()
    }

    /// Shrunk by `delta` on every side. Not clamped: deflating past the center gives an empty
    /// rect with reversed edges.
    pub fn deflate(self, delta: f32) -> Rect {
        let _ = delta;
        todo!()
    }
}

impl Default for Rect {
    fn default() -> Self {
        todo!()
    }
}

/// Distances from each edge of a box, in logical pixels, with Flutter's name and meaning. Used
/// for padding, margins and borders. May be negative.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeInsets {
    /// Inset from the left edge.
    pub left: f32,
    /// Inset from the top edge.
    pub top: f32,
    /// Inset from the right edge.
    pub right: f32,
    /// Inset from the bottom edge.
    pub bottom: f32,
}

impl EdgeInsets {
    /// No insets.
    pub const ZERO: EdgeInsets = EdgeInsets {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// The same inset on all four sides.
    pub const fn all(value: f32) -> Self {
        let _ = value;
        todo!()
    }

    /// `horizontal` on left and right, `vertical` on top and bottom.
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        let _ = (horizontal, vertical);
        todo!()
    }

    /// From the four sides.
    pub const fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        let _ = (left, top, right, bottom);
        todo!()
    }

    /// `left + right`.
    pub fn horizontal(self) -> f32 {
        todo!()
    }

    /// `top + bottom`.
    pub fn vertical(self) -> f32 {
        todo!()
    }

    /// `size` minus the insets, never below zero on either side (a NaN side becomes 0).
    pub fn deflate_size(self, size: Size) -> Size {
        let _ = size;
        todo!()
    }

    /// `size` plus the insets.
    pub fn inflate_size(self, size: Size) -> Size {
        let _ = size;
        todo!()
    }

    /// `rect` with each edge moved inward by its inset. Not clamped.
    pub fn deflate_rect(self, rect: Rect) -> Rect {
        let _ = rect;
        todo!()
    }

    /// `rect` with each edge moved outward by its inset. Not clamped.
    pub fn inflate_rect(self, rect: Rect) -> Rect {
        let _ = rect;
        todo!()
    }
}

impl Default for EdgeInsets {
    fn default() -> Self {
        todo!()
    }
}

impl Add for EdgeInsets {
    type Output = EdgeInsets;
    fn add(self, rhs: EdgeInsets) -> EdgeInsets {
        let _ = rhs;
        todo!()
    }
}

/// A 2D affine transform.
///
/// Coefficients `[a, b, c, d, e, f]` map (x, y) to (a·x + c·y + e, b·x + d·y + f).
/// `A * B` applies `B` first, then `A`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    coeffs: [f32; 6],
}

impl Affine {
    /// The identity transform.
    pub const IDENTITY: Affine = Affine {
        coeffs: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// From coefficients `[a, b, c, d, e, f]`.
    pub const fn new(coeffs: [f32; 6]) -> Self {
        let _ = coeffs;
        todo!()
    }

    /// The coefficients `[a, b, c, d, e, f]`.
    pub const fn coeffs(self) -> [f32; 6] {
        todo!()
    }

    /// Translation by `offset`.
    pub const fn translate(offset: Vec2) -> Self {
        let _ = offset;
        todo!()
    }

    /// Uniform scale about the origin.
    pub const fn scale(s: f32) -> Self {
        let _ = s;
        todo!()
    }

    /// Scale about the origin by `sx` horizontally and `sy` vertically.
    pub const fn scale_non_uniform(sx: f32, sy: f32) -> Self {
        let _ = (sx, sy);
        todo!()
    }

    /// Rotation about the origin by `radians`. With y pointing down, a positive angle turns
    /// clockwise on screen.
    pub fn rotate(radians: f32) -> Self {
        let _ = radians;
        todo!()
    }

    /// `a·d − b·c`.
    pub fn determinant(self) -> f32 {
        todo!()
    }

    /// The inverse, or `None` if the transform can't be inverted: the determinant is 0 or not
    /// finite, or the result would have a non-finite coefficient.
    pub fn inverse(self) -> Option<Affine> {
        todo!()
    }

    /// Applies the linear part only (no translation), for offsets and directions.
    pub fn transform_vec(self, v: Vec2) -> Vec2 {
        let _ = v;
        todo!()
    }

    /// The smallest axis-aligned rect containing the transformed corners of `rect`.
    pub fn transform_rect_bbox(self, rect: Rect) -> Rect {
        let _ = rect;
        todo!()
    }

    /// True if all coefficients are finite.
    pub fn is_finite(self) -> bool {
        todo!()
    }
}

impl Default for Affine {
    fn default() -> Self {
        todo!()
    }
}

impl Mul for Affine {
    type Output = Affine;
    fn mul(self, rhs: Affine) -> Affine {
        let _ = rhs;
        todo!()
    }
}

impl Mul<Point> for Affine {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        let _ = rhs;
        todo!()
    }
}
