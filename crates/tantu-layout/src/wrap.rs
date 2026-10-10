//! [`RenderWrap`]: Flutter's wrap layout, which starts a new run when the next child doesn't
//! fit. Spec: `docs/specs/layout/wrap.md`.

use tantu_core::{Size, Vec2};

use crate::{Axis, BoxConstraints, IntrinsicChildren, LayoutChildren, RenderBox};

/// How children are distributed within a run, or runs within the wrap (Flutter's
/// `WrapAlignment`; the same meanings as `MainAxisAlignment`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WrapAlignment {
    /// Packed at the start.
    Start,
    /// Packed at the end.
    End,
    /// Packed in the middle.
    Center,
    /// Free space evenly between, none at the ends.
    SpaceBetween,
    /// Free space evenly between, half as much at the ends.
    SpaceAround,
    /// Free space evenly between and at the ends.
    SpaceEvenly,
}

/// How children are placed on the cross axis within their run (Flutter's
/// `WrapCrossAlignment`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WrapCrossAlignment {
    /// At the run's cross-axis start.
    Start,
    /// At the run's cross-axis end.
    End,
    /// In the middle of the run.
    Center,
}

/// Lays out children in runs along `direction`, starting a new run when the next child
/// doesn't fit (Flutter's `RenderWrap`, behind `Wrap`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderWrap {
    /// The main axis of each run.
    pub direction: Axis,
    /// Distribution of children within each run.
    pub alignment: WrapAlignment,
    /// Space between children in a run (negative or NaN counts as 0).
    pub spacing: f32,
    /// Distribution of the runs on the cross axis.
    pub run_alignment: WrapAlignment,
    /// Space between runs (negative or NaN counts as 0).
    pub run_spacing: f32,
    /// Placement of children on the cross axis within their run.
    pub cross_axis_alignment: WrapCrossAlignment,
}

impl RenderWrap {
    /// Flutter's defaults: `Start` everywhere, spacing 0.
    pub const fn new(direction: Axis) -> Self {
        RenderWrap {
            direction,
            alignment: WrapAlignment::Start,
            spacing: 0.0,
            run_alignment: WrapAlignment::Start,
            run_spacing: 0.0,
            cross_axis_alignment: WrapCrossAlignment::Start,
        }
    }

    /// `new(Axis::Horizontal)`.
    pub const fn horizontal() -> Self {
        RenderWrap::new(Axis::Horizontal)
    }

    /// `new(Axis::Vertical)`.
    pub const fn vertical() -> Self {
        RenderWrap::new(Axis::Vertical)
    }

    /// `spacing` with negative and NaN counting as 0.
    fn spacing(&self) -> f32 {
        non_negative(self.spacing)
    }

    /// `run_spacing` with negative and NaN counting as 0.
    fn run_spacing(&self) -> f32 {
        non_negative(self.run_spacing)
    }

    /// The cross-axis intrinsic at main extent `extent` (LAYOUT-WRAP-08): runs of children at
    /// their max main intrinsics (capped at `extent`), with their cross intrinsics there.
    fn cross_intrinsic(
        &self,
        extent: f32,
        children: &mut IntrinsicChildren<'_>,
        main: fn(&mut IntrinsicChildren<'_>, usize, f32) -> f32,
        cross: fn(&mut IntrinsicChildren<'_>, usize, f32) -> f32,
    ) -> f32 {
        let n = children.len();
        let mut runs = Runs::new(self.spacing(), extent);
        let (mut total, mut count) = (0.0f32, 0usize);
        for i in 0..n {
            let child_main = main(children, i, f32::INFINITY).min(extent);
            let child_cross = cross(children, i, child_main);
            if let Some((_, run_cross)) = runs.push(child_main, child_cross) {
                total += run_cross;
                count += 1;
            }
        }
        if let Some((_, run_cross)) = runs.finish() {
            total += run_cross;
            count += 1;
        }
        total + gaps(self.run_spacing(), count)
    }
}

/// A spacing value with negative and NaN counting as 0.
fn non_negative(v: f32) -> f32 {
    if v >= 0.0 { v } else { 0.0 }
}

/// The total spacing between `count` items (avoiding `0 · ∞`).
fn gaps(spacing: f32, count: usize) -> f32 {
    if count > 1 {
        spacing * (count - 1) as f32
    } else {
        0.0
    }
}

/// Leading and between space for `count` items over `free` space (LAYOUT-FLEX-08 rules).
fn distribute(alignment: WrapAlignment, free: f32, count: usize) -> (f32, f32) {
    let n = count as f32;
    match alignment {
        WrapAlignment::Start => (0.0, 0.0),
        WrapAlignment::End => (free, 0.0),
        WrapAlignment::Center => (free / 2.0, 0.0),
        WrapAlignment::SpaceBetween if count > 1 => (0.0, free / (n - 1.0)),
        WrapAlignment::SpaceAround if count > 0 => (free / n / 2.0, free / n),
        WrapAlignment::SpaceEvenly if count > 0 => (free / (n + 1.0), free / (n + 1.0)),
        _ => (0.0, 0.0),
    }
}

/// Breaks a sequence of children into runs (LAYOUT-WRAP-03, 04).
struct Runs {
    spacing: f32,
    limit: f32,
    main: f32,
    cross: f32,
    count: usize,
}

impl Runs {
    fn new(spacing: f32, limit: f32) -> Self {
        Runs {
            spacing,
            limit,
            main: 0.0,
            cross: 0.0,
            count: 0,
        }
    }

    /// Adds a child; returns the finished run's (main, cross) when the child starts a new one.
    fn push(&mut self, main: f32, cross: f32) -> Option<(f32, f32)> {
        let finished = if self.count > 0 && self.main + self.spacing + main > self.limit {
            self.finish()
        } else {
            None
        };
        if self.count > 0 {
            self.main += self.spacing;
        }
        self.main += main;
        self.cross = self.cross.max(cross);
        self.count += 1;
        finished
    }

    /// Ends the current run, if it has children.
    fn finish(&mut self) -> Option<(f32, f32)> {
        if self.count == 0 {
            return None;
        }
        let run = (self.main, self.cross);
        (self.main, self.cross, self.count) = (0.0, 0.0, 0);
        Some(run)
    }
}

impl RenderBox for RenderWrap {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        let axis = self.direction;
        let n = children.len();
        let maxima = Size::new(constraints.max_width, constraints.max_height);
        let limit = axis_of(axis, maxima);
        let child_constraints = match axis {
            Axis::Horizontal => BoxConstraints::new(0.0, limit, 0.0, f32::INFINITY),
            Axis::Vertical => BoxConstraints::new(0.0, f32::INFINITY, 0.0, limit),
        };
        let (spacing, run_spacing) = (self.spacing(), self.run_spacing());

        // Size every child (LAYOUT-WRAP-02).
        for i in 0..n {
            children.layout(i, child_constraints);
        }
        let item = |children: &LayoutChildren<'_>, i: usize| {
            let size = children.size(i);
            (axis_of(axis, size), axis_of(axis.flip(), size))
        };
        // The run starting at child `start`: (end, main extent, cross extent) (LAYOUT-WRAP-03).
        let next_run = |children: &LayoutChildren<'_>, start: usize| {
            let (mut main, mut cross) = item(children, start);
            let mut end = start + 1;
            while end < n {
                let (m, c) = item(children, end);
                if main + spacing + m > limit {
                    break;
                }
                main += spacing + m;
                cross = cross.max(c);
                end += 1;
            }
            (end, main, cross)
        };

        // Measure the runs and size the wrap (LAYOUT-WRAP-04).
        let (mut runs_main, mut runs_cross, mut run_count, mut start) = (0.0f32, 0.0f32, 0, 0);
        while start < n {
            let (end, main, cross) = next_run(children, start);
            runs_main = runs_main.max(main);
            runs_cross += cross;
            run_count += 1;
            start = end;
        }
        let total_cross = runs_cross + gaps(run_spacing, run_count);
        let size = constraints.constrain(size_of(axis, runs_main, total_cross));
        let (main_size, cross_size) = (axis_of(axis, size), axis_of(axis.flip(), size));

        // Place the runs (LAYOUT-WRAP-05) and the children in them (06, 07), finding each run
        // again with the same comparisons on the same sizes.
        let (run_leading, run_between) = distribute(
            self.run_alignment,
            (cross_size - total_cross).max(0.0),
            run_count,
        );
        let mut cross_position = run_leading;
        let mut start = 0;
        while start < n {
            let (end, run_main, run_cross) = next_run(children, start);
            let (leading, between) =
                distribute(self.alignment, (main_size - run_main).max(0.0), end - start);
            let mut main_position = leading;
            for i in start..end {
                let (m, c) = item(children, i);
                let cross_offset = match self.cross_axis_alignment {
                    WrapCrossAlignment::Start => 0.0,
                    WrapCrossAlignment::End => run_cross - c,
                    WrapCrossAlignment::Center => (run_cross - c) / 2.0,
                };
                children.set_offset(
                    i,
                    offset_of(axis, main_position, cross_position + cross_offset),
                );
                main_position += m + between + spacing;
            }
            cross_position += run_cross + run_between + run_spacing;
            start = end;
        }
        size
    }

    fn min_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.direction {
            Axis::Horizontal => (0..children.len())
                .map(|i| children.min_intrinsic_width(i, f32::INFINITY))
                .fold(0.0, f32::max),
            Axis::Vertical => self.cross_intrinsic(
                height,
                children,
                |c, i, a| c.max_intrinsic_height(i, a),
                |c, i, a| c.max_intrinsic_width(i, a),
            ),
        }
    }

    fn max_intrinsic_width(&self, height: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.direction {
            Axis::Horizontal => {
                let n = children.len();
                (0..n)
                    .map(|i| children.max_intrinsic_width(i, f32::INFINITY))
                    .sum::<f32>()
                    + gaps(self.spacing(), n)
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
            Axis::Vertical => (0..children.len())
                .map(|i| children.min_intrinsic_height(i, f32::INFINITY))
                .fold(0.0, f32::max),
            Axis::Horizontal => self.cross_intrinsic(
                width,
                children,
                |c, i, a| c.max_intrinsic_width(i, a),
                |c, i, a| c.max_intrinsic_height(i, a),
            ),
        }
    }

    fn max_intrinsic_height(&self, width: f32, children: &mut IntrinsicChildren<'_>) -> f32 {
        match self.direction {
            Axis::Vertical => {
                let n = children.len();
                (0..n)
                    .map(|i| children.max_intrinsic_height(i, f32::INFINITY))
                    .sum::<f32>()
                    + gaps(self.spacing(), n)
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

/// The extent of `size` along `axis`.
fn axis_of(axis: Axis, size: Size) -> f32 {
    match axis {
        Axis::Horizontal => size.width,
        Axis::Vertical => size.height,
    }
}

/// A size from main and cross extents.
fn size_of(axis: Axis, main: f32, cross: f32) -> Size {
    match axis {
        Axis::Horizontal => Size::new(main, cross),
        Axis::Vertical => Size::new(cross, main),
    }
}

/// An offset from main and cross components.
fn offset_of(axis: Axis, main: f32, cross: f32) -> Vec2 {
    match axis {
        Axis::Horizontal => Vec2::new(main, cross),
        Axis::Vertical => Vec2::new(cross, main),
    }
}
