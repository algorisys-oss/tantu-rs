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

impl RenderBox for RenderImage {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!("LAYOUT-IMAGE-01")
    }

    fn min_intrinsic_width(&self, height: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = height;
        todo!("LAYOUT-IMAGE-02")
    }

    fn max_intrinsic_width(&self, height: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = height;
        todo!("LAYOUT-IMAGE-02")
    }

    fn min_intrinsic_height(&self, width: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = width;
        todo!("LAYOUT-IMAGE-02")
    }

    fn max_intrinsic_height(&self, width: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = width;
        todo!("LAYOUT-IMAGE-02")
    }
}
