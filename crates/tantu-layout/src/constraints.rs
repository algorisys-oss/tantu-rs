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
    pub const UNCONSTRAINED: BoxConstraints =
        BoxConstraints::new(0.0, f32::INFINITY, 0.0, f32::INFINITY);

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
        BoxConstraints::new(size.width, size.width, size.height, size.height)
    }

    /// Exactly the given width and/or height; an axis given as `None` is unconstrained
    /// (`0..=∞`).
    pub const fn tight_for(width: Option<f32>, height: Option<f32>) -> Self {
        let (min_width, max_width) = match width {
            Some(w) => (w, w),
            None => (0.0, f32::INFINITY),
        };
        let (min_height, max_height) = match height {
            Some(h) => (h, h),
            None => (0.0, f32::INFINITY),
        };
        BoxConstraints::new(min_width, max_width, min_height, max_height)
    }

    /// Like [`tight_for`](Self::tight_for), but an axis is tightened only when its value is
    /// finite (Flutter's `tightForFinite`, with `f32::INFINITY` meaning "not given").
    pub fn tight_for_finite(width: f32, height: f32) -> Self {
        let finite = |v: f32| v.is_finite().then_some(v);
        BoxConstraints::tight_for(finite(width), finite(height))
    }

    /// `0..=size` on both axes.
    pub const fn loose(size: Size) -> Self {
        BoxConstraints::new(0.0, size.width, 0.0, size.height)
    }

    /// Exactly the given width and/or height; an axis given as `None` is tight at infinity
    /// (fill all space; only meaningful under a parent that bounds it, through
    /// [`enforce`](Self::enforce)).
    pub const fn expand(width: Option<f32>, height: Option<f32>) -> Self {
        let width = match width {
            Some(w) => w,
            None => f32::INFINITY,
        };
        let height = match height {
            Some(h) => h,
            None => f32::INFINITY,
        };
        BoxConstraints::new(width, width, height, height)
    }

    /// `min_width.max(width.min(max_width))`: the width closest to `width` that these
    /// constraints allow; when they conflict (`min > max`) the minimum wins. A NaN width gives
    /// `min_width`; a NaN bound is ignored on its side.
    pub fn constrain_width(self, width: f32) -> f32 {
        constrain(width, self.min_width, self.max_width)
    }

    /// As [`constrain_width`](Self::constrain_width), for heights.
    pub fn constrain_height(self, height: f32) -> f32 {
        constrain(height, self.min_height, self.max_height)
    }

    /// Both axes constrained.
    pub fn constrain(self, size: Size) -> Size {
        Size::new(
            self.constrain_width(size.width),
            self.constrain_height(size.height),
        )
    }

    /// The largest allowed size: `constrain(∞, ∞)` (so infinite on an unbounded axis).
    pub fn biggest(self) -> Size {
        self.constrain(Size::INFINITY)
    }

    /// The smallest allowed size: `constrain(0, 0)`.
    pub fn smallest(self) -> Size {
        self.constrain(Size::ZERO)
    }

    /// `min_width >= max_width`.
    pub fn has_tight_width(self) -> bool {
        self.min_width >= self.max_width
    }

    /// `min_height >= max_height`.
    pub fn has_tight_height(self) -> bool {
        self.min_height >= self.max_height
    }

    /// Both axes tight: exactly one size is allowed.
    pub fn is_tight(self) -> bool {
        self.has_tight_width() && self.has_tight_height()
    }

    /// `max_width` is finite.
    pub fn has_bounded_width(self) -> bool {
        self.max_width.is_finite()
    }

    /// `max_height` is finite.
    pub fn has_bounded_height(self) -> bool {
        self.max_height.is_finite()
    }

    /// `min_width` is infinite (only expand-style constraints have it).
    pub fn has_infinite_width(self) -> bool {
        self.min_width.is_infinite()
    }

    /// `min_height` is infinite.
    pub fn has_infinite_height(self) -> bool {
        self.min_height.is_infinite()
    }

    /// `size` lies in both ranges (bounds included). False if anything is NaN.
    pub fn is_satisfied_by(self, size: Size) -> bool {
        (self.min_width..=self.max_width).contains(&size.width)
            && (self.min_height..=self.max_height).contains(&size.height)
    }

    /// No NaN; minima finite and ≥ 0; each maximum ≥ its minimum (maxima may be infinite).
    pub fn is_normalized(self) -> bool {
        let axis = |min: f32, max: f32| min.is_finite() && min >= 0.0 && max >= min;
        axis(self.min_width, self.max_width) && axis(self.min_height, self.max_height)
    }

    /// The nearest normalized constraints: a negative, NaN or infinite minimum becomes 0, a NaN
    /// maximum becomes infinite, and a maximum below its minimum is raised to the minimum.
    pub fn normalize(self) -> Self {
        if self.is_normalized() {
            return self;
        }
        let axis = |min: f32, max: f32| {
            let min = if min.is_finite() && min > 0.0 {
                min
            } else {
                0.0
            };
            let max = if max.is_nan() { f32::INFINITY } else { max };
            (min, if max >= min { max } else { min })
        };
        let (min_width, max_width) = axis(self.min_width, self.max_width);
        let (min_height, max_height) = axis(self.min_height, self.max_height);
        BoxConstraints::new(min_width, max_width, min_height, max_height)
    }

    /// Minima set to 0, maxima kept: the child may be smaller than before.
    pub fn loosen(self) -> Self {
        BoxConstraints::new(0.0, self.max_width, 0.0, self.max_height)
    }

    /// Each of the four values constrained into `outer`'s range for its axis: constraints that
    /// respect both `self`'s intent and `outer`, preferring `outer`.
    pub fn enforce(self, outer: BoxConstraints) -> Self {
        BoxConstraints::new(
            outer.constrain_width(self.min_width),
            outer.constrain_width(self.max_width),
            outer.constrain_height(self.min_height),
            outer.constrain_height(self.max_height),
        )
    }

    /// Tight on an axis given as `Some(v)`, at `constrain_width(v)` / `constrain_height(v)`;
    /// an axis given as `None` is unchanged.
    pub fn tighten(self, width: Option<f32>, height: Option<f32>) -> Self {
        let mut result = self;
        if let Some(w) = width {
            let w = self.constrain_width(w);
            (result.min_width, result.max_width) = (w, w);
        }
        if let Some(h) = height {
            let h = self.constrain_height(h);
            (result.min_height, result.max_height) = (h, h);
        }
        result
    }

    /// Shrunk by `insets` (what a padded child gets): on each axis the minimum becomes
    /// `max(0, min − insets)` and the maximum `max(new min, max − insets)`. Negative insets
    /// grow the constraints.
    pub fn deflate(self, insets: EdgeInsets) -> Self {
        let axis = |min: f32, max: f32, d: f32| {
            let min = (min - d).max(0.0);
            (min, (max - d).max(min))
        };
        let (min_width, max_width) = axis(self.min_width, self.max_width, insets.horizontal());
        let (min_height, max_height) = axis(self.min_height, self.max_height, insets.vertical());
        BoxConstraints::new(min_width, max_width, min_height, max_height)
    }

    /// Width and height swapped (for code shared between horizontal and vertical layouts).
    pub fn flip(self) -> Self {
        BoxConstraints::new(
            self.min_height,
            self.max_height,
            self.min_width,
            self.max_width,
        )
    }

    /// Only the width range kept; the height becomes `0..=∞`.
    pub fn width_constraints(self) -> Self {
        BoxConstraints::new(self.min_width, self.max_width, 0.0, f32::INFINITY)
    }

    /// Only the height range kept; the width becomes `0..=∞`.
    pub fn height_constraints(self) -> Self {
        BoxConstraints::new(0.0, f32::INFINITY, self.min_height, self.max_height)
    }
}

impl Default for BoxConstraints {
    /// [`BoxConstraints::UNCONSTRAINED`].
    fn default() -> Self {
        BoxConstraints::UNCONSTRAINED
    }
}

/// `value` clamped to `min..=max` where the minimum wins on a conflict, a NaN value gives
/// `min`, and a NaN bound is ignored on its side (`f32::max`/`f32::min` return the other
/// operand). Never panics, unlike `f32::clamp`.
#[inline]
fn constrain(value: f32, min: f32, max: f32) -> f32 {
    if value.is_nan() {
        return min;
    }
    min.max(value.min(max))
}
