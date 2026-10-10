//! Single-child layouts: [`RenderPadding`], [`RenderPositionedBox`], [`RenderConstrainedBox`],
//! [`RenderFractionallySizedBox`] and [`RenderAspectRatio`]. Each lays out at most its first
//! child. Spec: `docs/specs/layout/single-child.md`.

use tantu_core::{EdgeInsets, Size, Vec2};

use crate::{Alignment, BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// A size factor with negative and NaN values counting as 0 (LAYOUT-SINGLE-06, 10).
fn factor(f: Option<f32>) -> Option<f32> {
    f.map(|f| if f >= 0.0 { f } else { 0.0 })
}

/// The free space left when `child` is placed in `size`.
fn free(size: Size, child: Size) -> Vec2 {
    Vec2::new(size.width - child.width, size.height - child.height)
}

/// Insets its child by `padding` (Flutter's `RenderPadding`, behind `Padding`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPadding {
    /// Space around the child.
    pub padding: EdgeInsets,
}

impl RenderPadding {
    /// Padding of `padding`.
    pub const fn new(padding: EdgeInsets) -> Self {
        RenderPadding { padding }
    }
}

impl RenderBox for RenderPadding {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let (h, v) = (self.padding.horizontal(), self.padding.vertical());
        if children.is_empty() {
            return constraints.constrain(Size::new(h, v));
        }
        let child = children.layout(0, constraints.deflate(self.padding));
        children.set_offset(0, Vec2::new(self.padding.left, self.padding.top));
        Size::new(child.width + h, child.height + v)
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let inner = (height - self.padding.vertical()).max(0.0);
        children.min_intrinsic_width(0, inner) + self.padding.horizontal()
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let inner = (height - self.padding.vertical()).max(0.0);
        children.max_intrinsic_width(0, inner) + self.padding.horizontal()
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let inner = (width - self.padding.horizontal()).max(0.0);
        children.min_intrinsic_height(0, inner) + self.padding.vertical()
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let inner = (width - self.padding.horizontal()).max(0.0);
        children.max_intrinsic_height(0, inner) + self.padding.vertical()
    }
}

/// Positions its child within itself by `alignment`. On an axis with a factor, or with
/// unbounded constraints, it sizes itself to the child's extent times the factor (1 without
/// one); on the other axes it is as large as allowed (Flutter's `RenderPositionedBox`, behind
/// `Align` and `Center`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPositionedBox {
    /// Where the child goes.
    pub alignment: Alignment,
    /// Width as a multiple of the child's width; `None` fills the width when bounded.
    pub width_factor: Option<f32>,
    /// Height as a multiple of the child's height; `None` fills the height when bounded.
    pub height_factor: Option<f32>,
}

impl RenderPositionedBox {
    /// `alignment`, no factors.
    pub const fn new(alignment: Alignment) -> Self {
        RenderPositionedBox {
            alignment,
            width_factor: None,
            height_factor: None,
        }
    }

    /// Centered, no factors (`Center`).
    pub const fn center() -> Self {
        RenderPositionedBox::new(Alignment::CENTER)
    }
}

impl Default for RenderPositionedBox {
    /// [`RenderPositionedBox::center`].
    fn default() -> Self {
        RenderPositionedBox::center()
    }
}

impl RenderBox for RenderPositionedBox {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let (wf, hf) = (factor(self.width_factor), factor(self.height_factor));
        let shrink_width = wf.is_some() || constraints.max_width == f32::INFINITY;
        let shrink_height = hf.is_some() || constraints.max_height == f32::INFINITY;
        let extent = |shrink: bool, child: f32, f: Option<f32>| {
            if shrink {
                child * f.unwrap_or(1.0)
            } else {
                f32::INFINITY
            }
        };
        if children.is_empty() {
            return constraints.constrain(Size::new(
                extent(shrink_width, 0.0, None),
                extent(shrink_height, 0.0, None),
            ));
        }
        let child = children.layout(0, constraints.loosen());
        let size = constraints.constrain(Size::new(
            extent(shrink_width, child.width, wf),
            extent(shrink_height, child.height, hf),
        ));
        children.set_offset(0, self.alignment.along_offset(free(size, child)));
        size
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.min_intrinsic_width(0, height)
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.max_intrinsic_width(0, height)
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.min_intrinsic_height(0, width)
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.max_intrinsic_height(0, width)
    }
}

/// Imposes `additional` constraints on its child, within the incoming ones (Flutter's
/// `RenderConstrainedBox`, behind `ConstrainedBox` and `SizedBox`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderConstrainedBox {
    /// The constraints added to the incoming ones (through `BoxConstraints::enforce`).
    pub additional: BoxConstraints,
}

impl RenderConstrainedBox {
    /// Adds `additional`.
    pub const fn new(additional: BoxConstraints) -> Self {
        RenderConstrainedBox { additional }
    }

    /// `SizedBox(width, height)`: `BoxConstraints::tight_for(width, height)`.
    pub const fn sized(width: Option<f32>, height: Option<f32>) -> Self {
        RenderConstrainedBox::new(BoxConstraints::tight_for(width, height))
    }

    /// `SizedBox.expand()`: `BoxConstraints::expand(None, None)`, filling bounded constraints.
    pub const fn expand() -> Self {
        RenderConstrainedBox::new(BoxConstraints::expand(None, None))
    }

    /// `SizedBox.shrink()`: tight at 0 × 0, as small as allowed.
    pub const fn shrink() -> Self {
        RenderConstrainedBox::new(BoxConstraints::tight(Size::ZERO))
    }

    /// An intrinsic width given the child's (LAYOUT-SINGLE-09).
    fn width(&self, child: impl FnOnce() -> f32) -> f32 {
        let a = self.additional;
        if a.has_bounded_width() && a.has_tight_width() {
            return a.min_width;
        }
        let width = child();
        if a.has_infinite_width() {
            width
        } else {
            a.constrain_width(width)
        }
    }

    /// An intrinsic height given the child's.
    fn height(&self, child: impl FnOnce() -> f32) -> f32 {
        let a = self.additional;
        if a.has_bounded_height() && a.has_tight_height() {
            return a.min_height;
        }
        let height = child();
        if a.has_infinite_height() {
            height
        } else {
            a.constrain_height(height)
        }
    }
}

impl RenderBox for RenderConstrainedBox {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let inner = self.additional.enforce(constraints);
        if children.is_empty() {
            return inner.constrain(Size::ZERO);
        }
        children.layout(0, inner)
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.width(|| children.min_intrinsic_width(0, height))
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.width(|| children.max_intrinsic_width(0, height))
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.height(|| children.min_intrinsic_height(0, width))
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        self.height(|| children.max_intrinsic_height(0, width))
    }
}

/// Sizes its child to a fraction of the incoming maximum on an axis with a factor, and
/// positions it by `alignment`; the child may overflow (Flutter's
/// `RenderFractionallySizedOverflowBox`, behind `FractionallySizedBox`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFractionallySizedBox {
    /// Where the child goes (it may be larger than this box).
    pub alignment: Alignment,
    /// The child's width as a fraction of the incoming maximum width; `None` passes the width
    /// constraints through.
    pub width_factor: Option<f32>,
    /// As `width_factor`, for heights.
    pub height_factor: Option<f32>,
}

impl RenderFractionallySizedBox {
    /// Centered, with the given factors.
    pub const fn new(width_factor: Option<f32>, height_factor: Option<f32>) -> Self {
        RenderFractionallySizedBox {
            alignment: Alignment::CENTER,
            width_factor,
            height_factor,
        }
    }

    /// The child's constraints (LAYOUT-SINGLE-10).
    fn inner(&self, c: BoxConstraints) -> BoxConstraints {
        let axis = |f: Option<f32>, min: f32, max: f32| match factor(f) {
            Some(f) => (max * f, max * f),
            None => (min, max),
        };
        let (min_width, max_width) = axis(self.width_factor, c.min_width, c.max_width);
        let (min_height, max_height) = axis(self.height_factor, c.min_height, c.max_height);
        BoxConstraints::new(min_width, max_width, min_height, max_height)
    }
}

impl RenderBox for RenderFractionallySizedBox {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let inner = self.inner(constraints);
        if children.is_empty() {
            return constraints.constrain(inner.smallest());
        }
        let child = children.layout(0, inner);
        let size = constraints.constrain(child);
        children.set_offset(0, self.alignment.along_offset(free(size, child)));
        size
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.min_intrinsic_width(0, height) / factor(self.width_factor).unwrap_or(1.0)
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.max_intrinsic_width(0, height) / factor(self.width_factor).unwrap_or(1.0)
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.min_intrinsic_height(0, width) / factor(self.height_factor).unwrap_or(1.0)
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        children.max_intrinsic_height(0, width) / factor(self.height_factor).unwrap_or(1.0)
    }
}

/// Sizes itself (and its child, tightly) to `aspect_ratio` = width / height, as large as the
/// constraints allow (Flutter's `RenderAspectRatio`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderAspectRatio {
    /// Width divided by height; must be finite and positive to have an effect.
    pub aspect_ratio: f32,
}

impl RenderAspectRatio {
    /// A box with `aspect_ratio` = width / height.
    pub const fn new(aspect_ratio: f32) -> Self {
        RenderAspectRatio { aspect_ratio }
    }

    /// The ratio, if it is finite and positive (LAYOUT-SINGLE-13).
    fn ratio(&self) -> Option<f32> {
        (self.aspect_ratio.is_finite() && self.aspect_ratio > 0.0).then_some(self.aspect_ratio)
    }

    /// The size for `c` (LAYOUT-SINGLE-12, Flutter's `_applyAspectRatio`).
    fn size(&self, c: BoxConstraints) -> Size {
        let Some(ratio) = self.ratio() else {
            return c.smallest();
        };
        if c.is_tight() {
            return c.smallest();
        }
        let (mut width, mut height);
        if c.max_width.is_finite() {
            width = c.max_width;
            height = width / ratio;
        } else if c.max_height.is_finite() {
            height = c.max_height;
            width = height * ratio;
        } else {
            width = c.min_width;
            height = width / ratio;
        }
        if width > c.max_width {
            width = c.max_width;
            height = width / ratio;
        }
        if height > c.max_height {
            height = c.max_height;
            width = height * ratio;
        }
        if width < c.min_width {
            width = c.min_width;
            height = width / ratio;
        }
        if height < c.min_height {
            height = c.min_height;
            width = height * ratio;
        }
        c.constrain(Size::new(width, height))
    }
}

impl RenderBox for RenderAspectRatio {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let size = self.size(constraints);
        if !children.is_empty() {
            children.layout(0, BoxConstraints::tight(size));
            children.set_offset(0, Vec2::ZERO);
        }
        size
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.ratio() {
            Some(ratio) if height.is_finite() => height * ratio,
            _ => children.min_intrinsic_width(0, height),
        }
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.ratio() {
            Some(ratio) if height.is_finite() => height * ratio,
            _ => children.max_intrinsic_width(0, height),
        }
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.ratio() {
            Some(ratio) if width.is_finite() => width / ratio,
            _ => children.min_intrinsic_height(0, width),
        }
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.ratio() {
            Some(ratio) if width.is_finite() => width / ratio,
            _ => children.max_intrinsic_height(0, width),
        }
    }
}
