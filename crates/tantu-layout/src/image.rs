//! [`RenderImage`]: an image's sizing. Spec: `docs/specs/layout/image.md`.

use tantu_core::Size;

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// Sizes itself to an image's natural size, or an explicit width/height, keeping the aspect
/// ratio within the constraints (Flutter's `RenderImage`, sizing only; the widget paints).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderImage {
    /// The image's size in logical pixels (its pixel size at scale 1).
    pub natural: Size,
    /// An explicit width (tight), if any.
    pub width: Option<f32>,
    /// An explicit height (tight), if any.
    pub height: Option<f32>,
}

impl RenderImage {
    /// An image of `natural` size with no explicit size.
    pub const fn new(natural: Size) -> Self {
        RenderImage {
            natural,
            width: None,
            height: None,
        }
    }
}

impl RenderImage {
    /// The size for `constraints` (LAYOUT-IMAGE-01, Flutter's `_sizeForConstraints`).
    fn size_for(&self, constraints: BoxConstraints) -> Size {
        BoxConstraints::tight_for(self.width, self.height)
            .enforce(constraints)
            .constrain_size_and_attempt_to_preserve_aspect_ratio(self.natural)
    }

    fn has_explicit_size(&self) -> bool {
        self.width.is_some() || self.height.is_some()
    }
}

impl RenderBox for RenderImage {
    fn perform_layout(&mut self, constraints: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        self.size_for(constraints)
    }

    fn min_intrinsic_width(&self, height: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        if !self.has_explicit_size() {
            return 0.0;
        }
        self.size_for(BoxConstraints::tight_for_finite(f32::INFINITY, height))
            .width
    }

    fn max_intrinsic_width(&self, height: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.size_for(BoxConstraints::tight_for_finite(f32::INFINITY, height))
            .width
    }

    fn min_intrinsic_height(&self, width: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        if !self.has_explicit_size() {
            return 0.0;
        }
        self.size_for(BoxConstraints::tight_for_finite(width, f32::INFINITY))
            .height
    }

    fn max_intrinsic_height(&self, width: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.size_for(BoxConstraints::tight_for_finite(width, f32::INFINITY))
            .height
    }
}
