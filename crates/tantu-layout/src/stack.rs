//! [`RenderStack`]: Flutter's stack layout, with [`StackParentData`] for `Positioned`
//! children. Spec: `docs/specs/layout/stack.md`.

use tantu_core::{Size, Vec2};

use crate::{Alignment, BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// How non-positioned children are constrained (Flutter's `StackFit`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StackFit {
    /// The incoming constraints, loosened (children may be smaller than the stack).
    Loose,
    /// Tight at the largest allowed size (children fill the stack).
    Expand,
    /// The incoming constraints unchanged.
    Passthrough,
}

/// Parent data read by [`RenderStack`]: distances from the stack's edges, and an optional
/// size. A child is *positioned* when it has this parent data with at least one field set (a
/// NaN field counts as not set).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StackParentData {
    /// Distance from the stack's left edge to the child's left edge.
    pub left: Option<f32>,
    /// Distance from the stack's top edge to the child's top edge.
    pub top: Option<f32>,
    /// Distance from the stack's right edge to the child's right edge.
    pub right: Option<f32>,
    /// Distance from the stack's bottom edge to the child's bottom edge.
    pub bottom: Option<f32>,
    /// The child's width (tight); with both `left` and `right` set, `left` wins for placement.
    pub width: Option<f32>,
    /// The child's height (tight); with both `top` and `bottom` set, `top` wins for placement.
    pub height: Option<f32>,
}

impl StackParentData {
    /// True if any field is set (and not NaN).
    pub fn is_positioned(&self) -> bool {
        [
            self.left,
            self.top,
            self.right,
            self.bottom,
            self.width,
            self.height,
        ]
        .into_iter()
        .any(|v| set(v).is_some())
    }

    /// All four edges at 0: the child covers the stack (`Positioned.fill`).
    pub const fn fill() -> Self {
        StackParentData {
            left: Some(0.0),
            top: Some(0.0),
            right: Some(0.0),
            bottom: Some(0.0),
            width: None,
            height: None,
        }
    }

    /// The child's constraints on one axis (LAYOUT-STACK-05): `start`/`end` are the edge
    /// distances, `extent` the explicit size, `available` the stack's extent.
    fn axis_constraint(
        start: Option<f32>,
        end: Option<f32>,
        extent: Option<f32>,
        available: f32,
    ) -> (f32, f32) {
        match (set(extent), set(start), set(end)) {
            (Some(e), _, _) => (e, e),
            (None, Some(s), Some(e)) => {
                let e = (available - s - e).max(0.0);
                (e, e)
            }
            _ => (0.0, f32::INFINITY),
        }
    }

    /// The child's offset on one axis (LAYOUT-STACK-06).
    fn axis_offset(
        start: Option<f32>,
        end: Option<f32>,
        alignment: f32,
        available: f32,
        child: f32,
    ) -> f32 {
        match (set(start), set(end)) {
            (Some(s), _) => s,
            (None, Some(e)) => available - e - child,
            (None, None) => (alignment + 1.0) / 2.0 * (available - child),
        }
    }
}

/// A parent-data field, with NaN counting as not set.
fn set(value: Option<f32>) -> Option<f32> {
    value.filter(|v| !v.is_nan())
}

/// The child's `StackParentData` if it is positioned.
fn positioned(data: Option<&StackParentData>) -> Option<StackParentData> {
    data.copied().filter(StackParentData::is_positioned)
}

/// Layers its children; the non-positioned ones size the stack (Flutter's `RenderStack`,
/// behind `Stack`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderStack {
    /// Where non-positioned children go, and positioned ones on an axis with no edge set.
    pub alignment: Alignment,
    /// How non-positioned children are constrained.
    pub fit: StackFit,
}

impl RenderStack {
    /// `alignment: TOP_LEFT`, `fit: Loose` (Flutter's defaults with left-to-right text).
    pub const fn new() -> Self {
        RenderStack {
            alignment: Alignment::TOP_LEFT,
            fit: StackFit::Loose,
        }
    }

    /// The largest intrinsic among non-positioned children (LAYOUT-STACK-07).
    fn intrinsic(
        children: &mut IntrinsicChildren<'_>,
        arg: f32,
        query: fn(&mut IntrinsicChildren<'_>, usize, f32) -> f32,
    ) -> f32 {
        let mut result = 0.0f32;
        for i in 0..children.len() {
            if positioned(children.parent_data::<StackParentData>(i)).is_none() {
                result = result.max(query(children, i, arg));
            }
        }
        result
    }
}

impl Default for RenderStack {
    /// [`RenderStack::new`].
    fn default() -> Self {
        RenderStack::new()
    }
}

impl RenderBox for RenderStack {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let n = children.len();
        let child_constraints = match self.fit {
            StackFit::Loose => constraints.loosen(),
            StackFit::Expand => BoxConstraints::tight(constraints.biggest()),
            StackFit::Passthrough => constraints,
        };

        // Non-positioned children size the stack (LAYOUT-STACK-02, 03).
        let (mut width, mut height, mut any) = (0.0f32, 0.0f32, false);
        for i in 0..n {
            if positioned(children.parent_data::<StackParentData>(i)).is_some() {
                continue;
            }
            let size = children.layout(i, child_constraints);
            width = width.max(size.width);
            height = height.max(size.height);
            any = true;
        }
        let size = if any {
            constraints.constrain(Size::new(width, height))
        } else {
            let (big, small) = (constraints.biggest(), constraints.smallest());
            Size::new(
                if big.width.is_finite() {
                    big.width
                } else {
                    small.width
                },
                if big.height.is_finite() {
                    big.height
                } else {
                    small.height
                },
            )
        };

        // Place them (LAYOUT-STACK-04), then lay out and place the positioned ones (05, 06).
        for i in 0..n {
            match positioned(children.parent_data::<StackParentData>(i)) {
                None => {
                    let child = children.size(i);
                    let free = Vec2::new(size.width - child.width, size.height - child.height);
                    children.set_offset(i, self.alignment.along_offset(free));
                }
                Some(p) => {
                    let (min_w, max_w) =
                        StackParentData::axis_constraint(p.left, p.right, p.width, size.width);
                    let (min_h, max_h) =
                        StackParentData::axis_constraint(p.top, p.bottom, p.height, size.height);
                    let child = children.layout(i, BoxConstraints::new(min_w, max_w, min_h, max_h));
                    let x = StackParentData::axis_offset(
                        p.left,
                        p.right,
                        self.alignment.x,
                        size.width,
                        child.width,
                    );
                    let y = StackParentData::axis_offset(
                        p.top,
                        p.bottom,
                        self.alignment.y,
                        size.height,
                        child.height,
                    );
                    children.set_offset(i, Vec2::new(x, y));
                }
            }
        }
        size
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        RenderStack::intrinsic(children, height, |c, i, a| c.min_intrinsic_width(i, a))
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        RenderStack::intrinsic(children, height, |c, i, a| c.max_intrinsic_width(i, a))
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        RenderStack::intrinsic(children, width, |c, i, a| c.min_intrinsic_height(i, a))
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        RenderStack::intrinsic(children, width, |c, i, a| c.max_intrinsic_height(i, a))
    }
}
