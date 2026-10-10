//! [`BoxConstraints`]: the allowed sizes a parent passes down in the layout protocol.

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
    pub const UNCONSTRAINED: BoxConstraints = BoxConstraints::new(0.0, 0.0, 0.0, 0.0);

    /// Constraints from the four values, stored as given.
    #[inline]
    pub const fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        BoxConstraints {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    /// Exactly `size`.
    pub const fn tight(size: Size) -> Self {
        let _ = size;
        todo!()
    }

    /// Exactly the given width and/or height; an axis given as `None` is unconstrained
    /// (`0..=∞`).
    pub const fn tight_for(width: Option<f32>, height: Option<f32>) -> Self {
        let _ = (width, height);
        todo!()
    }

    /// Like [`tight_for`](Self::tight_for), but an axis is tightened only when its value is
    /// finite (Flutter's `tightForFinite`, with `f32::INFINITY` meaning "not given").
    pub fn tight_for_finite(width: f32, height: f32) -> Self {
        let _ = (width, height);
        todo!()
    }

    /// `0..=size` on both axes.
    pub const fn loose(size: Size) -> Self {
        let _ = size;
        todo!()
    }

    /// Exactly the given width and/or height; an axis given as `None` is tight at infinity
    /// (fill all space; only meaningful under a parent that bounds it, through
    /// [`enforce`](Self::enforce)).
    pub const fn expand(width: Option<f32>, height: Option<f32>) -> Self {
        let _ = (width, height);
        todo!()
    }

    /// `min_width.max(width.min(max_width))`: the width closest to `width` that these
    /// constraints allow; when they conflict (`min > max`) the minimum wins. A NaN width gives
    /// `min_width`; a NaN bound is ignored on its side.
    pub fn constrain_width(self, width: f32) -> f32 {
        let _ = width;
        todo!()
    }

    /// As [`constrain_width`](Self::constrain_width), for heights.
    pub fn constrain_height(self, height: f32) -> f32 {
        let _ = height;
        todo!()
    }

    /// Both axes constrained.
    pub fn constrain(self, size: Size) -> Size {
        let _ = size;
        todo!()
    }

    /// The largest allowed size: `constrain(∞, ∞)` (so infinite on an unbounded axis).
    pub fn biggest(self) -> Size {
        todo!()
    }

    /// The smallest allowed size: `constrain(0, 0)`.
    pub fn smallest(self) -> Size {
        todo!()
    }

    /// `min_width >= max_width`.
    pub fn has_tight_width(self) -> bool {
        todo!()
    }

    /// `min_height >= max_height`.
    pub fn has_tight_height(self) -> bool {
        todo!()
    }

    /// Both axes tight: exactly one size is allowed.
    pub fn is_tight(self) -> bool {
        todo!()
    }

    /// `max_width` is finite.
    pub fn has_bounded_width(self) -> bool {
        todo!()
    }

    /// `max_height` is finite.
    pub fn has_bounded_height(self) -> bool {
        todo!()
    }

    /// `min_width` is infinite (only expand-style constraints have it).
    pub fn has_infinite_width(self) -> bool {
        todo!()
    }

    /// `min_height` is infinite.
    pub fn has_infinite_height(self) -> bool {
        todo!()
    }

    /// `size` lies in both ranges (bounds included). False if anything is NaN.
    pub fn is_satisfied_by(self, size: Size) -> bool {
        let _ = size;
        todo!()
    }

    /// No NaN; minima finite and ≥ 0; each maximum ≥ its minimum (maxima may be infinite).
    pub fn is_normalized(self) -> bool {
        todo!()
    }

    /// The nearest normalized constraints: a negative, NaN or infinite minimum becomes 0, a NaN
    /// maximum becomes infinite, and a maximum below its minimum is raised to the minimum.
    pub fn normalize(self) -> Self {
        todo!()
    }

    /// Minima set to 0, maxima kept: the child may be smaller than before.
    pub fn loosen(self) -> Self {
        todo!()
    }

    /// Each of the four values constrained into `outer`'s range for its axis: constraints that
    /// respect both `self`'s intent and `outer`, preferring `outer`.
    pub fn enforce(self, outer: BoxConstraints) -> Self {
        let _ = outer;
        todo!()
    }

    /// Tight on an axis given as `Some(v)`, at `constrain_width(v)` / `constrain_height(v)`;
    /// an axis given as `None` is unchanged.
    pub fn tighten(self, width: Option<f32>, height: Option<f32>) -> Self {
        let _ = (width, height);
        todo!()
    }

    /// Shrunk by `insets` (what a padded child gets): on each axis the minimum becomes
    /// `max(0, min − insets)` and the maximum `max(new min, max − insets)`. Negative insets
    /// grow the constraints.
    pub fn deflate(self, insets: EdgeInsets) -> Self {
        let _ = insets;
        todo!()
    }

    /// Width and height swapped (for code shared between horizontal and vertical layouts).
    pub fn flip(self) -> Self {
        todo!()
    }

    /// Only the width range kept; the height becomes `0..=∞`.
    pub fn width_constraints(self) -> Self {
        todo!()
    }

    /// Only the height range kept; the width becomes `0..=∞`.
    pub fn height_constraints(self) -> Self {
        todo!()
    }
}

impl Default for BoxConstraints {
    /// [`BoxConstraints::UNCONSTRAINED`].
    fn default() -> Self {
        todo!()
    }
}
