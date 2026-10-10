//! [`RenderFlex`]: Flutter's flex layout, behind `Row` and `Column`, with [`FlexParentData`]
//! for `Expanded`, `Flexible` and `Spacer`. Spec: `docs/specs/layout/flex.md`.

use tantu_core::Size;

use crate::{BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// A direction: the main axis of a flex.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Left to right (`Row`).
    Horizontal,
    /// Top to bottom (`Column`).
    Vertical,
}

impl Axis {
    /// The other axis.
    pub fn flip(self) -> Axis {
        todo!()
    }
}

/// How free space on the main axis is distributed (Flutter's `MainAxisAlignment`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MainAxisAlignment {
    /// Children packed at the start.
    Start,
    /// Children packed at the end.
    End,
    /// Children packed in the middle.
    Center,
    /// Free space evenly between children, none before the first or after the last.
    SpaceBetween,
    /// Free space evenly between children, half as much before the first and after the last.
    SpaceAround,
    /// Free space evenly between children and before the first and after the last.
    SpaceEvenly,
}

/// How much main-axis space the flex takes (Flutter's `MainAxisSize`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MainAxisSize {
    /// As little as the children need.
    Min,
    /// As much as allowed (when bounded).
    Max,
}

/// How children are placed on the cross axis (Flutter's `CrossAxisAlignment`, without
/// `Baseline` for now).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CrossAxisAlignment {
    /// At the cross-axis start (top of a row, left of a column).
    Start,
    /// At the cross-axis end.
    End,
    /// In the middle.
    Center,
    /// Stretched to the full cross extent (tight cross constraints).
    Stretch,
}

/// Whether a flexible child must fill its share (`Tight`, `Expanded`) or may be smaller
/// (`Loose`, `Flexible`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlexFit {
    /// Exactly the share.
    Tight,
    /// At most the share.
    Loose,
}

/// Parent data read by [`RenderFlex`] (set with `LayoutTree::set_parent_data`). A child
/// without it, or with `flex == 0`, is inflexible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlexParentData {
    /// The child's share of the free space, relative to the other flexible children.
    pub flex: u32,
    /// Whether the child must fill its share.
    pub fit: FlexFit,
}

impl FlexParentData {
    /// `Expanded(flex)`: `fit: Tight`.
    pub const fn expanded(flex: u32) -> Self {
        let _ = flex;
        todo!()
    }

    /// `Flexible(flex)`: `fit: Loose`.
    pub const fn flexible(flex: u32) -> Self {
        let _ = flex;
        todo!()
    }
}

/// Lays out its children in a line along `direction` (Flutter's `RenderFlex`, behind `Row`
/// and `Column`).
#[derive(Clone, Copy, Debug)]
pub struct RenderFlex {
    /// The main axis.
    pub direction: Axis,
    /// How free main-axis space is distributed.
    pub main_axis_alignment: MainAxisAlignment,
    /// How much main-axis space the flex takes.
    pub main_axis_size: MainAxisSize,
    /// How children are placed on the cross axis.
    pub cross_axis_alignment: CrossAxisAlignment,
    /// Space between adjacent children, on top of the alignment's (negative or NaN counts as 0).
    pub spacing: f32,
    overflow: f32,
}

impl RenderFlex {
    /// A flex along `direction` with Flutter's defaults: `Start`, `Max`, `Center`, spacing 0.
    pub const fn new(direction: Axis) -> Self {
        let _ = direction;
        todo!()
    }

    /// `RenderFlex::new(Axis::Horizontal)` (`Row`).
    pub const fn row() -> Self {
        todo!()
    }

    /// `RenderFlex::new(Axis::Vertical)` (`Column`).
    pub const fn column() -> Self {
        todo!()
    }

    /// How far the children overflowed the main axis in the last layout (0 if they fit).
    pub fn overflow(&self) -> f32 {
        todo!()
    }
}

/// Compares the properties only (not the last overflow), so `LayoutTree::set` with unchanged
/// properties doesn't re-lay out.
impl PartialEq for RenderFlex {
    fn eq(&self, other: &Self) -> bool {
        let _ = (other, self.overflow);
        todo!()
    }
}

impl RenderBox for RenderFlex {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let _ = (constraints, children);
        todo!()
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!()
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (height, children);
        todo!()
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!()
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        let _ = (width, children);
        todo!()
    }
}
