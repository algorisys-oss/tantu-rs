//! [`RenderFlex`]: Flutter's flex layout, behind `Row` and `Column`, with [`FlexParentData`]
//! for `Expanded`, `Flexible` and `Spacer`. Spec: `docs/specs/layout/flex.md`.

use tantu_core::{Size, Vec2};

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
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }

    /// The extent of `size` along this axis.
    fn of(self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    /// A size from its extent along this axis (`main`) and the other (`cross`).
    fn size(self, main: f32, cross: f32) -> Size {
        match self {
            Axis::Horizontal => Size::new(main, cross),
            Axis::Vertical => Size::new(cross, main),
        }
    }

    /// An offset from its components along this axis and the other.
    fn offset(self, main: f32, cross: f32) -> Vec2 {
        match self {
            Axis::Horizontal => Vec2::new(main, cross),
            Axis::Vertical => Vec2::new(cross, main),
        }
    }

    /// Constraints from main-axis and cross-axis ranges.
    fn constraints(self, main: (f32, f32), cross: (f32, f32)) -> BoxConstraints {
        match self {
            Axis::Horizontal => BoxConstraints::new(main.0, main.1, cross.0, cross.1),
            Axis::Vertical => BoxConstraints::new(cross.0, cross.1, main.0, main.1),
        }
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
        FlexParentData {
            flex,
            fit: FlexFit::Tight,
        }
    }

    /// `Flexible(flex)`: `fit: Loose`.
    pub const fn flexible(flex: u32) -> Self {
        FlexParentData {
            flex,
            fit: FlexFit::Loose,
        }
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
        RenderFlex {
            direction,
            main_axis_alignment: MainAxisAlignment::Start,
            main_axis_size: MainAxisSize::Max,
            cross_axis_alignment: CrossAxisAlignment::Center,
            spacing: 0.0,
            overflow: 0.0,
        }
    }

    /// `RenderFlex::new(Axis::Horizontal)` (`Row`).
    pub const fn row() -> Self {
        RenderFlex::new(Axis::Horizontal)
    }

    /// `RenderFlex::new(Axis::Vertical)` (`Column`).
    pub const fn column() -> Self {
        RenderFlex::new(Axis::Vertical)
    }

    /// How far the children overflowed the main axis in the last layout (0 if they fit).
    pub fn overflow(&self) -> f32 {
        self.overflow
    }

    /// `spacing`, with negative and NaN counting as 0.
    fn spacing(&self) -> f32 {
        if self.spacing >= 0.0 {
            self.spacing
        } else {
            0.0
        }
    }

    /// The spacing between `n` children in total (avoiding `0 · ∞`).
    fn total_spacing(&self, n: usize) -> f32 {
        if n > 1 {
            self.spacing() * (n - 1) as f32
        } else {
            0.0
        }
    }

    /// The main-axis intrinsic (LAYOUT-FLEX-12), with `child` giving a child's intrinsic on
    /// the main axis at `cross`.
    fn main_intrinsic(
        &self,
        cross: f32,
        children: &mut IntrinsicChildren<'_>,
        child: fn(&mut IntrinsicChildren<'_>, usize, f32) -> f32,
    ) -> f32 {
        let n = children.len();
        let (mut inflexible, mut max_fraction, mut total_flex) = (0.0f32, 0.0f64, 0.0f64);
        for i in 0..n {
            let value = child(children, i, cross);
            match flex_factor(children.parent_data::<FlexParentData>(i)) {
                0 => inflexible += value,
                flex => {
                    total_flex += f64::from(flex);
                    max_fraction = max_fraction.max(f64::from(value) / f64::from(flex));
                }
            }
        }
        (max_fraction * total_flex) as f32 + inflexible + self.total_spacing(n)
    }

    /// The cross-axis intrinsic (LAYOUT-FLEX-13) at main extent `extent`, with `main` giving a
    /// child's max main intrinsic at a cross extent and `cross` a child's cross intrinsic at a
    /// main extent.
    fn cross_intrinsic(
        &self,
        extent: f32,
        children: &mut IntrinsicChildren<'_>,
        main: fn(&mut IntrinsicChildren<'_>, usize, f32) -> f32,
        cross: fn(&mut IntrinsicChildren<'_>, usize, f32) -> f32,
    ) -> f32 {
        let n = children.len();
        let (mut inflexible, mut total_flex, mut result) = (0.0f32, 0.0f64, 0.0f32);
        for i in 0..n {
            match flex_factor(children.parent_data::<FlexParentData>(i)) {
                0 => {
                    let child_main = main(children, i, f32::INFINITY);
                    inflexible += child_main;
                    result = result.max(cross(children, i, child_main));
                }
                flex => total_flex += f64::from(flex),
            }
        }
        if total_flex > 0.0 {
            let per_flex = if extent.is_finite() {
                (f64::from(extent - inflexible - self.total_spacing(n)) / total_flex).max(0.0)
            } else {
                f64::INFINITY
            };
            for i in 0..n {
                let flex = flex_factor(children.parent_data::<FlexParentData>(i));
                if flex > 0 {
                    let share = (per_flex * f64::from(flex)) as f32;
                    result = result.max(cross(children, i, share));
                }
            }
        }
        result
    }
}

/// The flex factor of a child's parent data; 0 (inflexible) without `FlexParentData`.
fn flex_factor(data: Option<&FlexParentData>) -> u32 {
    data.map_or(0, |d| d.flex)
}

/// Compares the properties only (not the last overflow), so `LayoutTree::set` with unchanged
/// properties doesn't re-lay out.
impl PartialEq for RenderFlex {
    fn eq(&self, other: &Self) -> bool {
        self.direction == other.direction
            && self.main_axis_alignment == other.main_axis_alignment
            && self.main_axis_size == other.main_axis_size
            && self.cross_axis_alignment == other.cross_axis_alignment
            && self.spacing == other.spacing
    }
}

impl RenderBox for RenderFlex {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let axis = self.direction;
        let maxima = Size::new(constraints.max_width, constraints.max_height);
        let (max_main, max_cross) = (axis.of(maxima), axis.flip().of(maxima));
        let can_flex = max_main.is_finite();
        let cross = match self.cross_axis_alignment {
            CrossAxisAlignment::Stretch => (max_cross, max_cross),
            _ => (0.0, max_cross),
        };
        let n = children.len();
        let flex_of = |children: &LayoutChildren<'_>, i: usize| {
            flex_factor(children.parent_data::<FlexParentData>(i))
        };

        // Inflexible children first (LAYOUT-FLEX-03).
        let mut allocated = self.total_spacing(n);
        let mut max_child_cross = 0.0f32;
        let mut total_flex = 0.0f64;
        let mut last_flexible = None;
        for i in 0..n {
            let flex = flex_of(children, i);
            if flex > 0 && can_flex {
                total_flex += f64::from(flex);
                last_flexible = Some(i);
                continue;
            }
            let size = children.layout(i, axis.constraints((0.0, f32::INFINITY), cross));
            allocated += axis.of(size);
            max_child_cross = max_child_cross.max(axis.flip().of(size));
        }
        if !can_flex && cfg!(debug_assertions) && flex_present(children, n) {
            tracing::warn!(
                ?constraints,
                "RenderFlex has flexible children but an unbounded main axis; \
                 they are laid out as inflexible"
            );
        }

        // Then the flexible ones share the free space (LAYOUT-FLEX-04).
        if let Some(last) = last_flexible {
            let free = (max_main - allocated).max(0.0);
            let per_flex = f64::from(free) / total_flex;
            let mut given = 0.0f32;
            for i in 0..n {
                let flex = flex_of(children, i);
                if flex == 0 {
                    continue;
                }
                let share = if i == last {
                    (free - given).max(0.0)
                } else {
                    (per_flex * f64::from(flex)) as f32
                };
                given += share;
                let fit = children
                    .parent_data::<FlexParentData>(i)
                    .map_or(FlexFit::Tight, |d| d.fit);
                let min = if fit == FlexFit::Tight { share } else { 0.0 };
                let size = children.layout(i, axis.constraints((min, share), cross));
                allocated += axis.of(size);
                max_child_cross = max_child_cross.max(axis.flip().of(size));
            }
        }

        // The flex's own size (LAYOUT-FLEX-06) and overflow (LAYOUT-FLEX-07).
        let ideal_main = if can_flex && self.main_axis_size == MainAxisSize::Max {
            max_main
        } else {
            allocated
        };
        let size = constraints.constrain(axis.size(ideal_main, max_child_cross));
        let (main_size, cross_size) = (axis.of(size), axis.flip().of(size));
        self.overflow = (allocated - main_size).max(0.0);

        // Positions (LAYOUT-FLEX-08..11).
        let remaining = (main_size - allocated).max(0.0);
        let count = n as f32;
        let (leading, between) = match self.main_axis_alignment {
            MainAxisAlignment::Start => (0.0, 0.0),
            MainAxisAlignment::End => (remaining, 0.0),
            MainAxisAlignment::Center => (remaining / 2.0, 0.0),
            MainAxisAlignment::SpaceBetween if n > 1 => (0.0, remaining / (count - 1.0)),
            MainAxisAlignment::SpaceBetween => (0.0, 0.0),
            MainAxisAlignment::SpaceAround if n > 0 => (remaining / count / 2.0, remaining / count),
            MainAxisAlignment::SpaceEvenly if n > 0 => {
                (remaining / (count + 1.0), remaining / (count + 1.0))
            }
            MainAxisAlignment::SpaceAround | MainAxisAlignment::SpaceEvenly => (0.0, 0.0),
        };
        let between = between + self.spacing();
        let mut position = leading;
        for i in 0..n {
            let child = children.size(i);
            let child_cross = axis.flip().of(child);
            let cross_offset = match self.cross_axis_alignment {
                CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
                CrossAxisAlignment::End => cross_size - child_cross,
                CrossAxisAlignment::Center => (cross_size - child_cross) / 2.0,
            };
            children.set_offset(i, axis.offset(position, cross_offset));
            position += axis.of(child) + between;
        }
        size
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.direction {
            Axis::Horizontal => {
                self.main_intrinsic(height, children, |c, i, a| c.min_intrinsic_width(i, a))
            }
            Axis::Vertical => self.cross_intrinsic(
                height,
                children,
                |c, i, a| c.max_intrinsic_height(i, a),
                |c, i, a| c.min_intrinsic_width(i, a),
            ),
        }
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.direction {
            Axis::Horizontal => {
                self.main_intrinsic(height, children, |c, i, a| c.max_intrinsic_width(i, a))
            }
            Axis::Vertical => self.cross_intrinsic(
                height,
                children,
                |c, i, a| c.max_intrinsic_height(i, a),
                |c, i, a| c.max_intrinsic_width(i, a),
            ),
        }
    }

    fn min_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.direction {
            Axis::Vertical => {
                self.main_intrinsic(width, children, |c, i, a| c.min_intrinsic_height(i, a))
            }
            Axis::Horizontal => self.cross_intrinsic(
                width,
                children,
                |c, i, a| c.max_intrinsic_width(i, a),
                |c, i, a| c.min_intrinsic_height(i, a),
            ),
        }
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.direction {
            Axis::Vertical => {
                self.main_intrinsic(width, children, |c, i, a| c.max_intrinsic_height(i, a))
            }
            Axis::Horizontal => self.cross_intrinsic(
                width,
                children,
                |c, i, a| c.max_intrinsic_width(i, a),
                |c, i, a| c.max_intrinsic_height(i, a),
            ),
        }
    }
}

/// Whether any of the `n` children is flexible.
fn flex_present(children: &LayoutChildren<'_>, n: usize) -> bool {
    (0..n).any(|i| flex_factor(children.parent_data::<FlexParentData>(i)) > 0)
}
