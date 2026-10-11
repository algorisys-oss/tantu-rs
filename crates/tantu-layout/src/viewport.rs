//! [`RenderViewport`]: a window onto a scrolled child. Spec: `docs/specs/layout/viewport.md`.

use tantu_core::Size;

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// The scrolling axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAxis {
    /// Content scrolls up and down.
    #[default]
    Vertical,
    /// Content scrolls left and right.
    Horizontal,
}

/// Shows a window onto its child, shifted by `offset` along `axis` (Flutter's
/// `RenderSingleChildViewport`, without the paint).
#[derive(Clone, Copy, Debug)]
pub struct RenderViewport {
    /// The scrolling axis.
    pub axis: ScrollAxis,
    /// How far the content is scrolled (positive scrolls content up or left). Stored as given;
    /// a non-finite value counts as 0.
    pub offset: f32,
    /// The child's size from the last layout (not part of equality).
    content: Size,
}

impl RenderViewport {
    /// A viewport along `axis` at offset 0.
    pub const fn new(axis: ScrollAxis) -> Self {
        RenderViewport {
            axis,
            offset: 0.0,
            content: Size::ZERO,
        }
    }

    /// The child's size from the last layout (`Size::ZERO` before, or without a child).
    pub fn content(&self) -> Size {
        self.content
    }
}

/// Compares the axis and offset only (not the last content size).
impl PartialEq for RenderViewport {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis && self.offset == other.offset
    }
}

impl RenderBox for RenderViewport {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!("LAYOUT-VIEWPORT-01")
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!("LAYOUT-VIEWPORT-02")
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!("LAYOUT-VIEWPORT-02")
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!("LAYOUT-VIEWPORT-02")
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!("LAYOUT-VIEWPORT-02")
    }
}
