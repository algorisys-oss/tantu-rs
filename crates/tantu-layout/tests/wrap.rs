//! Tests for `docs/specs/layout/wrap.md`, rules LAYOUT-WRAP-01..09.

mod common;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{
    Axis, BoxConstraints, LayoutId, LayoutTree, RenderWrap, WrapAlignment, WrapCrossAlignment,
};

const INF: f32 = f32::INFINITY;
const NAN: f32 = f32::NAN;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

struct Wrap {
    tree: LayoutTree,
    root: LayoutId,
    kids: Vec<LayoutId>,
}

fn wrap(render: RenderWrap, kids: &[(f32, f32)]) -> Wrap {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let ids: Vec<LayoutId> = kids
        .iter()
        .map(|&(w, h)| tree.insert(leaf(&log, "kid", w, h)))
        .collect();
    tree.set_children(root, &ids).expect("valid");
    Wrap {
        tree,
        root,
        kids: ids,
    }
}

impl Wrap {
    fn layout(&mut self, c: BoxConstraints) -> Size {
        self.tree.layout(self.root, c)
    }

    fn offsets(&self) -> Vec<Vec2> {
        self.kids
            .iter()
            .map(|id| self.tree.offset(*id).expect("exists"))
            .collect()
    }
}

fn with(f: impl FnOnce(&mut RenderWrap)) -> RenderWrap {
    let mut r = RenderWrap::horizontal();
    f(&mut r);
    r
}

/// Three 40-wide children with spacing 10 in a 100-wide wrap: two runs, [0, 1] and [2].
const THREE: [(f32, f32); 3] = [(40.0, 10.0), (40.0, 20.0), (40.0, 10.0)];

#[test]
fn layout_wrap_01_constructors() {
    let r = RenderWrap::new(Axis::Vertical);
    assert_eq!(r.direction, Axis::Vertical);
    assert_eq!(r.alignment, WrapAlignment::Start);
    assert_eq!(r.run_alignment, WrapAlignment::Start);
    assert_eq!(r.cross_axis_alignment, WrapCrossAlignment::Start);
    assert_eq!((r.spacing, r.run_spacing), (0.0, 0.0));
    assert_eq!(RenderWrap::horizontal(), RenderWrap::new(Axis::Horizontal));
    assert_eq!(RenderWrap::vertical(), RenderWrap::new(Axis::Vertical));
    assert_ne!(RenderWrap::horizontal(), with(|r| r.run_spacing = 1.0));
}

#[test]
fn layout_wrap_02_child_constraints() {
    let mut w = wrap(RenderWrap::horizontal(), &THREE);
    w.layout(loose(100.0, 200.0));
    for kid in &w.kids {
        assert_eq!(
            w.tree.constraints(*kid),
            Some(BoxConstraints::new(0.0, 100.0, 0.0, INF))
        );
    }
}

#[test]
fn layout_wrap_03_runs() {
    let mut w = wrap(with(|r| r.spacing = 10.0), &THREE);
    assert_eq!(w.layout(loose(100.0, 200.0)), s(90.0, 30.0));
    assert_eq!(w.offsets(), [v(0.0, 0.0), v(50.0, 0.0), v(0.0, 20.0)]);
    // Exactly the limit: same run.
    let mut w = wrap(RenderWrap::horizontal(), &[(50.0, 10.0), (50.0, 10.0)]);
    assert_eq!(w.layout(loose(100.0, 200.0)), s(100.0, 10.0));
    // A child as wide as the limit sits alone.
    let mut w = wrap(
        RenderWrap::horizontal(),
        &[(30.0, 10.0), (150.0, 10.0), (30.0, 10.0)],
    );
    assert_eq!(w.layout(loose(100.0, 200.0)), s(100.0, 30.0));
    assert_eq!(w.offsets(), [v(0.0, 0.0), v(0.0, 10.0), v(0.0, 20.0)]);
    // Unbounded limit: one run.
    let mut w = wrap(with(|r| r.spacing = 10.0), &THREE);
    assert_eq!(
        w.layout(BoxConstraints::new(0.0, INF, 0.0, 200.0)),
        s(140.0, 20.0)
    );
}

#[test]
fn layout_wrap_04_size() {
    let mut w = wrap(
        with(|r| {
            r.spacing = 10.0;
            r.run_spacing = 5.0;
        }),
        &THREE,
    );
    assert_eq!(w.layout(loose(100.0, 200.0)), s(90.0, 35.0));
    assert_eq!(w.offsets()[2], v(0.0, 25.0));
    assert_eq!(
        w.layout(BoxConstraints::new(95.0, 100.0, 50.0, 200.0)),
        s(95.0, 50.0)
    );
    let mut w = wrap(RenderWrap::horizontal(), &[]);
    assert_eq!(w.layout(loose(100.0, 200.0)), s(0.0, 0.0));
    assert_eq!(
        w.layout(BoxConstraints::tight(s(30.0, 40.0))),
        s(30.0, 40.0)
    );
}

#[test]
fn layout_wrap_05_run_alignment() {
    // Runs' cross extents 20 and 10 in a 100-tall wrap: 70 free.
    let cases = [
        (WrapAlignment::Start, [0.0, 20.0]),
        (WrapAlignment::End, [70.0, 90.0]),
        (WrapAlignment::Center, [35.0, 55.0]),
        (WrapAlignment::SpaceBetween, [0.0, 90.0]),
        (WrapAlignment::SpaceAround, [17.5, 72.5]),
        (
            WrapAlignment::SpaceEvenly,
            [70.0 / 3.0, 70.0 / 3.0 * 2.0 + 20.0],
        ),
    ];
    for (alignment, ys) in cases {
        let mut w = wrap(
            with(|r| {
                r.spacing = 10.0;
                r.run_alignment = alignment;
            }),
            &THREE,
        );
        w.layout(BoxConstraints::tight(s(100.0, 100.0)));
        let offsets = w.offsets();
        assert!(
            (offsets[0].y - ys[0]).abs() < 1e-3,
            "{alignment:?}: {offsets:?}"
        );
        assert!(
            (offsets[2].y - ys[1]).abs() < 1e-3,
            "{alignment:?}: {offsets:?}"
        );
    }
}

#[test]
fn layout_wrap_06_alignment_within_runs() {
    // Run 1 is 90 wide (10 free), run 2 is 40 wide (60 free).
    let cases = [
        (WrapAlignment::End, [10.0, 60.0, 60.0]),
        (WrapAlignment::Center, [5.0, 55.0, 30.0]),
        (WrapAlignment::SpaceBetween, [0.0, 60.0, 0.0]),
    ];
    for (alignment, xs) in cases {
        let mut w = wrap(
            with(|r| {
                r.spacing = 10.0;
                r.alignment = alignment;
            }),
            &THREE,
        );
        w.layout(BoxConstraints::tight(s(100.0, 100.0)));
        let got: Vec<f32> = w.offsets().iter().map(|o| o.x).collect();
        assert_eq!(got, xs, "{alignment:?}");
    }
    let mut w = wrap(
        with(|r| r.alignment = WrapAlignment::SpaceEvenly),
        &[(20.0, 10.0), (20.0, 10.0)],
    );
    w.layout(BoxConstraints::tight(s(100.0, 100.0)));
    assert_eq!(w.offsets(), [v(20.0, 0.0), v(60.0, 0.0)]);
}

#[test]
fn layout_wrap_07_cross_alignment_and_vertical() {
    for (alignment, y) in [
        (WrapCrossAlignment::Start, 0.0),
        (WrapCrossAlignment::End, 10.0),
        (WrapCrossAlignment::Center, 5.0),
    ] {
        let mut w = wrap(
            with(|r| {
                r.spacing = 10.0;
                r.cross_axis_alignment = alignment;
            }),
            &THREE,
        );
        w.layout(loose(100.0, 200.0));
        assert_eq!(w.offsets()[0], v(0.0, y), "{alignment:?}");
        assert_eq!(w.offsets()[1], v(50.0, 0.0), "{alignment:?}");
    }
    let mut vertical = RenderWrap::vertical();
    vertical.spacing = 10.0;
    let mut w = wrap(vertical, &[(10.0, 40.0), (20.0, 40.0), (10.0, 40.0)]);
    assert_eq!(w.layout(loose(200.0, 100.0)), s(30.0, 90.0));
    assert_eq!(w.offsets(), [v(0.0, 0.0), v(0.0, 50.0), v(20.0, 0.0)]);
    assert_eq!(
        w.tree.constraints(w.kids[0]),
        Some(BoxConstraints::new(0.0, INF, 0.0, 100.0))
    );
}

#[test]
fn layout_wrap_08_intrinsics() {
    let mut w = wrap(with(|r| r.spacing = 10.0), &THREE);
    assert_eq!(w.tree.min_intrinsic_width(w.root, INF), 40.0);
    assert_eq!(w.tree.max_intrinsic_width(w.root, INF), 140.0);
    assert_eq!(w.tree.min_intrinsic_height(w.root, 100.0), 30.0);
    assert_eq!(w.tree.max_intrinsic_height(w.root, 100.0), 30.0);
    assert_eq!(w.tree.max_intrinsic_height(w.root, INF), 20.0);
    let mut w = wrap(
        with(|r| {
            r.spacing = 10.0;
            r.run_spacing = 5.0;
        }),
        &THREE,
    );
    assert_eq!(w.tree.min_intrinsic_height(w.root, 100.0), 35.0);
    // Vertical mirrors it.
    let mut vertical = RenderWrap::vertical();
    vertical.spacing = 10.0;
    let mut w = wrap(vertical, &[(10.0, 40.0), (20.0, 40.0), (10.0, 40.0)]);
    assert_eq!(w.tree.min_intrinsic_height(w.root, INF), 40.0);
    assert_eq!(w.tree.max_intrinsic_height(w.root, INF), 140.0);
    assert_eq!(w.tree.min_intrinsic_width(w.root, 100.0), 30.0);
}

#[test]
fn layout_wrap_09_robustness() {
    let mut w = wrap(RenderWrap::horizontal(), &[]);
    assert_eq!(w.tree.min_intrinsic_width(w.root, INF), 0.0);
    assert_eq!(w.tree.max_intrinsic_height(w.root, 50.0), 0.0);
    // Negative or NaN spacing counts as 0.
    for bad in [-5.0, NAN] {
        let mut w = wrap(
            with(|r| {
                r.spacing = bad;
                r.run_spacing = bad;
            }),
            &[(60.0, 10.0), (60.0, 10.0)],
        );
        assert_eq!(w.layout(loose(100.0, 200.0)), s(60.0, 20.0), "{bad}");
        assert_eq!(w.offsets()[1], v(0.0, 10.0), "{bad}");
    }
    // Nothing panics, every child is placed once.
    for direction in [Axis::Horizontal, Axis::Vertical] {
        for alignment in [WrapAlignment::SpaceAround, WrapAlignment::End] {
            let mut r = RenderWrap::new(direction);
            r.alignment = alignment;
            r.run_alignment = alignment;
            r.spacing = INF;
            let mut w = wrap(r, &[(NAN, INF), (0.0, 0.0), (1e30, 3.0)]);
            for c in [
                BoxConstraints::UNCONSTRAINED,
                BoxConstraints::new(NAN, NAN, NAN, NAN),
                BoxConstraints::new(80.0, 20.0, INF, INF),
                BoxConstraints::tight(Size::ZERO),
            ] {
                w.tree.mark_needs_layout(w.root);
                let _ = w.layout(c);
                for arg in [0.0, 10.0, INF, NAN] {
                    let _ = w.tree.min_intrinsic_height(w.root, arg);
                    let _ = w.tree.max_intrinsic_width(w.root, arg);
                }
            }
        }
    }
}
