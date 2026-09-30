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
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }

    /// The vector from the origin to this point.
    #[inline]
    pub const fn to_vec2(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    /// True if both coordinates are finite.
    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Default for Point {
    #[inline]
    fn default() -> Self {
        Point::ZERO
    }
}

impl Add<Vec2> for Point {
    type Output = Point;
    #[inline]
    fn add(self, rhs: Vec2) -> Point {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<Vec2> for Point {
    type Output = Point;
    #[inline]
    fn sub(self, rhs: Vec2) -> Point {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub for Point {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Point) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl AddAssign<Vec2> for Point {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs;
    }
}

impl SubAssign<Vec2> for Point {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs;
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
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    /// The point at this offset from the origin.
    #[inline]
    pub const fn to_point(self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    /// Euclidean length.
    #[inline]
    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    /// True if both components are finite.
    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Default for Vec2 {
    #[inline]
    fn default() -> Self {
        Vec2::ZERO
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    #[inline]
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    #[inline]
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs;
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs;
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
    #[inline]
    pub const fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }

    /// True if the size has no area: either side is ≤ 0 or NaN.
    #[inline]
    pub fn is_empty(self) -> bool {
        !(self.width > 0.0 && self.height > 0.0)
    }

    /// True if both sides are finite.
    #[inline]
    pub fn is_finite(self) -> bool {
        self.width.is_finite() && self.height.is_finite()
    }

    /// Component-wise minimum. A NaN side yields the other size's side.
    #[inline]
    pub fn min(self, other: Size) -> Size {
        Size::new(self.width.min(other.width), self.height.min(other.height))
    }

    /// Component-wise maximum. A NaN side yields the other size's side.
    #[inline]
    pub fn max(self, other: Size) -> Size {
        Size::new(self.width.max(other.width), self.height.max(other.height))
    }

    /// A rect with this size at the origin.
    #[inline]
    pub const fn to_rect(self) -> Rect {
        Rect::from_ltrb(0.0, 0.0, self.width, self.height)
    }
}

impl Default for Size {
    #[inline]
    fn default() -> Self {
        Size::ZERO
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
    #[inline]
    pub const fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    /// From the top-left corner and a size.
    #[inline]
    pub fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        Rect::from_ltrb(left, top, left + width, top + height)
    }

    /// From the top-left corner and a size.
    #[inline]
    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Rect::from_ltwh(origin.x, origin.y, size.width, size.height)
    }

    /// `right - left` (negative if the edges are reversed).
    #[inline]
    pub fn width(self) -> f32 {
        self.right - self.left
    }

    /// `bottom - top` (negative if the edges are reversed).
    #[inline]
    pub fn height(self) -> f32 {
        self.bottom - self.top
    }

    /// Width and height.
    #[inline]
    pub fn size(self) -> Size {
        Size::new(self.width(), self.height())
    }

    /// The top-left corner.
    #[inline]
    pub const fn origin(self) -> Point {
        Point::new(self.left, self.top)
    }

    /// The center point.
    #[inline]
    pub fn center(self) -> Point {
        Point::new(
            (self.left + self.right) * 0.5,
            (self.top + self.bottom) * 0.5,
        )
    }

    /// True if the rect has no area: width or height ≤ 0, or any edge NaN.
    #[inline]
    pub fn is_empty(self) -> bool {
        !(self.width() > 0.0 && self.height() > 0.0)
    }

    /// True if all edges are finite.
    #[inline]
    pub fn is_finite(self) -> bool {
        self.left.is_finite()
            && self.top.is_finite()
            && self.right.is_finite()
            && self.bottom.is_finite()
    }

    /// True if `point` is inside. Left and top edges are inside, right and bottom are not.
    #[inline]
    pub fn contains(self, point: Point) -> bool {
        // Comparisons with NaN are false, so NaN points and empty (or reversed) rects fail here.
        self.left <= point.x && point.x < self.right && self.top <= point.y && point.y < self.bottom
    }

    /// True if the two rects share an area greater than zero.
    #[inline]
    pub fn overlaps(self, other: Rect) -> bool {
        !self.is_empty()
            && !other.is_empty()
            && self.left < other.right
            && other.left < self.right
            && self.top < other.bottom
            && other.top < self.bottom
    }

    /// The shared area, or `None` if they don't overlap.
    #[inline]
    pub fn intersect(self, other: Rect) -> Option<Rect> {
        if !self.overlaps(other) {
            return None;
        }
        Some(Rect::from_ltrb(
            self.left.max(other.left),
            self.top.max(other.top),
            self.right.min(other.right),
            self.bottom.min(other.bottom),
        ))
    }

    /// The smallest rect containing both. Empty rects are ignored; if both are empty, returns
    /// `self`.
    #[inline]
    pub fn union(self, other: Rect) -> Rect {
        if other.is_empty() {
            return self;
        }
        if self.is_empty() {
            return other;
        }
        Rect::from_ltrb(
            self.left.min(other.left),
            self.top.min(other.top),
            self.right.max(other.right),
            self.bottom.max(other.bottom),
        )
    }

    /// Moved by `offset`.
    #[inline]
    pub fn translate(self, offset: Vec2) -> Rect {
        Rect::from_ltrb(
            self.left + offset.x,
            self.top + offset.y,
            self.right + offset.x,
            self.bottom + offset.y,
        )
    }

    /// Grown by `delta` on every side.
    #[inline]
    pub fn inflate(self, delta: f32) -> Rect {
        Rect::from_ltrb(
            self.left - delta,
            self.top - delta,
            self.right + delta,
            self.bottom + delta,
        )
    }

    /// Shrunk by `delta` on every side. Not clamped: deflating past the center gives an empty
    /// rect with reversed edges.
    #[inline]
    pub fn deflate(self, delta: f32) -> Rect {
        self.inflate(-delta)
    }
}

impl Default for Rect {
    #[inline]
    fn default() -> Self {
        Rect::ZERO
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
    #[inline]
    pub const fn all(value: f32) -> Self {
        EdgeInsets::from_ltrb(value, value, value, value)
    }

    /// `horizontal` on left and right, `vertical` on top and bottom.
    #[inline]
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        EdgeInsets::from_ltrb(horizontal, vertical, horizontal, vertical)
    }

    /// From the four sides.
    #[inline]
    pub const fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        EdgeInsets {
            left,
            top,
            right,
            bottom,
        }
    }

    /// `left + right`.
    #[inline]
    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    /// `top + bottom`.
    #[inline]
    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }

    /// `size` minus the insets, never below zero on either side (a NaN side becomes 0).
    #[inline]
    pub fn deflate_size(self, size: Size) -> Size {
        // `f32::max` returns the non-NaN operand, so a NaN side becomes 0.
        Size::new(
            (size.width - self.horizontal()).max(0.0),
            (size.height - self.vertical()).max(0.0),
        )
    }

    /// `size` plus the insets.
    #[inline]
    pub fn inflate_size(self, size: Size) -> Size {
        Size::new(
            size.width + self.horizontal(),
            size.height + self.vertical(),
        )
    }

    /// `rect` with each edge moved inward by its inset. Not clamped.
    #[inline]
    pub fn deflate_rect(self, rect: Rect) -> Rect {
        Rect::from_ltrb(
            rect.left + self.left,
            rect.top + self.top,
            rect.right - self.right,
            rect.bottom - self.bottom,
        )
    }

    /// `rect` with each edge moved outward by its inset. Not clamped.
    #[inline]
    pub fn inflate_rect(self, rect: Rect) -> Rect {
        Rect::from_ltrb(
            rect.left - self.left,
            rect.top - self.top,
            rect.right + self.right,
            rect.bottom + self.bottom,
        )
    }
}

impl Default for EdgeInsets {
    #[inline]
    fn default() -> Self {
        EdgeInsets::ZERO
    }
}

impl Add for EdgeInsets {
    type Output = EdgeInsets;
    #[inline]
    fn add(self, rhs: EdgeInsets) -> EdgeInsets {
        EdgeInsets::from_ltrb(
            self.left + rhs.left,
            self.top + rhs.top,
            self.right + rhs.right,
            self.bottom + rhs.bottom,
        )
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
    #[inline]
    pub const fn new(coeffs: [f32; 6]) -> Self {
        Affine { coeffs }
    }

    /// The coefficients `[a, b, c, d, e, f]`.
    #[inline]
    pub const fn coeffs(self) -> [f32; 6] {
        self.coeffs
    }

    /// Translation by `offset`.
    #[inline]
    pub const fn translate(offset: Vec2) -> Self {
        Affine::new([1.0, 0.0, 0.0, 1.0, offset.x, offset.y])
    }

    /// Uniform scale about the origin.
    #[inline]
    pub const fn scale(s: f32) -> Self {
        Affine::new([s, 0.0, 0.0, s, 0.0, 0.0])
    }

    /// Scale about the origin by `sx` horizontally and `sy` vertically.
    #[inline]
    pub const fn scale_non_uniform(sx: f32, sy: f32) -> Self {
        Affine::new([sx, 0.0, 0.0, sy, 0.0, 0.0])
    }

    /// Rotation about the origin by `radians`. With y pointing down, a positive angle turns
    /// clockwise on screen.
    #[inline]
    pub fn rotate(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Affine::new([cos, sin, -sin, cos, 0.0, 0.0])
    }

    /// `a·d − b·c`.
    #[inline]
    pub fn determinant(self) -> f32 {
        let [a, b, c, d, ..] = self.coeffs;
        a * d - b * c
    }

    /// The inverse, or `None` if the transform can't be inverted: the determinant is 0 or not
    /// finite, or the result would have a non-finite coefficient.
    #[inline]
    pub fn inverse(self) -> Option<Affine> {
        // Invertibility follows the f32 `determinant`, so the two always agree. The inverse is
        // computed in f64 for accuracy, and the f32 result is checked for overflow.
        let det32 = self.determinant();
        if det32 == 0.0 || !det32.is_finite() {
            return None;
        }
        let [a, b, c, d, e, f] = self.coeffs.map(f64::from);
        let det = a * d - b * c;
        let inv = [
            d / det,
            -b / det,
            -c / det,
            a / det,
            (c * f - d * e) / det,
            (b * e - a * f) / det,
        ]
        .map(|x| x as f32);
        let inverse = Affine::new(inv);
        inverse.is_finite().then_some(inverse)
    }

    /// Applies the linear part only (no translation), for offsets and directions.
    #[inline]
    pub fn transform_vec(self, v: Vec2) -> Vec2 {
        let [a, b, c, d, ..] = self.coeffs;
        Vec2::new(a * v.x + c * v.y, b * v.x + d * v.y)
    }

    /// The smallest axis-aligned rect containing the transformed corners of `rect`.
    #[inline]
    pub fn transform_rect_bbox(self, rect: Rect) -> Rect {
        let corners = [
            self * Point::new(rect.left, rect.top),
            self * Point::new(rect.right, rect.top),
            self * Point::new(rect.left, rect.bottom),
            self * Point::new(rect.right, rect.bottom),
        ];
        let mut bbox = Rect::from_ltrb(corners[0].x, corners[0].y, corners[0].x, corners[0].y);
        for p in &corners[1..] {
            bbox.left = bbox.left.min(p.x);
            bbox.top = bbox.top.min(p.y);
            bbox.right = bbox.right.max(p.x);
            bbox.bottom = bbox.bottom.max(p.y);
        }
        bbox
    }

    /// True if all coefficients are finite.
    #[inline]
    pub fn is_finite(self) -> bool {
        self.coeffs.iter().all(|c| c.is_finite())
    }
}

impl Default for Affine {
    #[inline]
    fn default() -> Self {
        Affine::IDENTITY
    }
}

impl Mul for Affine {
    type Output = Affine;
    #[inline]
    fn mul(self, rhs: Affine) -> Affine {
        let [a1, b1, c1, d1, e1, f1] = self.coeffs;
        let [a2, b2, c2, d2, e2, f2] = rhs.coeffs;
        Affine::new([
            a1 * a2 + c1 * b2,
            b1 * a2 + d1 * b2,
            a1 * c2 + c1 * d2,
            b1 * c2 + d1 * d2,
            a1 * e2 + c1 * f2 + e1,
            b1 * e2 + d1 * f2 + f1,
        ])
    }
}

impl Mul<Point> for Affine {
    type Output = Point;
    #[inline]
    fn mul(self, rhs: Point) -> Point {
        let [a, b, c, d, e, f] = self.coeffs;
        Point::new(a * rhs.x + c * rhs.y + e, b * rhs.x + d * rhs.y + f)
    }
}
