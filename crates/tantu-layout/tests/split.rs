//! Tests for `docs/specs/widgets/split-view.md`, rules LAYOUT-SPLIT-01..05.

mod common;

use std::sync::Arc;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{Axis, BoxConstraints, LayoutId, LayoutTree, RenderSplit};

const INF: f32 = f32::INFINITY;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn f(values: &[f32]) -> Arc<[f32]> {
    Arc::from(values.to_vec())
}

fn b(values: &[bool]) -> Arc<[bool]> {
    Arc::from(values.to_vec())
}

/// A split of `panes` leaves (each wanting `want`) and `panes - 1` handle leaves, plus
/// `extra` further leaves.
struct Fixture {
    tree: LayoutTree,
    root: LayoutId,
    panes: Vec<LayoutId>,
    handles: Vec<LayoutId>,
    extra: Vec<LayoutId>,
}

fn fixture(render: RenderSplit, wants: &[Size], extra: usize) -> Fixture {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let panes_count = render.panes();
    let root = tree.insert(render);
    let panes: Vec<LayoutId> = wants
        .iter()
        .map(|w| tree.insert(leaf(&log, "pane", w.width, w.height)))
        .collect();
    let handles: Vec<LayoutId> = (1..panes_count)
        .map(|_| tree.insert(leaf(&log, "handle", 0.0, 0.0)))
        .collect();
    let extra: Vec<LayoutId> = (0..extra)
        .map(|_| tree.insert(leaf(&log, "extra", 50.0, 50.0)))
        .collect();
    let children: Vec<LayoutId> = panes
        .iter()
        .chain(&handles)
        .chain(&extra)
        .copied()
        .collect();
    tree.set_children(root, &children).expect("fresh nodes");
    Fixture {
        tree,
        root,
        panes,
        handles,
        extra,
    }
}

fn split(axis: Axis, fractions: &[f32]) -> RenderSplit {
    let panes = fractions.len().max(1);
    let mut r = RenderSplit::new(axis, panes);
    r.fractions = f(fractions);
    r
}

fn sizes(fx: &Fixture) -> Vec<f32> {
    fx.tree
        .get::<RenderSplit>(fx.root)
        .expect("split")
        .sizes()
        .to_vec()
}

fn assert_close(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len(), "{actual:?} vs {expected:?}");
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() < 1e-3, "{actual:?} vs {expected:?}");
    }
}

#[test]
fn layout_split_01_fractions_share_the_space_after_the_gaps() {
    let wants = [s(10.0, 10.0); 3];
    let mut fx = fixture(split(Axis::Horizontal, &[0.5, 0.25, 0.25]), &wants, 0);
    // The whole main extent; 300 - 2 × 1 = 298 for the panes.
    assert_eq!(
        fx.tree
            .layout(fx.root, BoxConstraints::loose(s(300.0, 100.0))),
        s(300.0, 100.0)
    );
    assert_close(&sizes(&fx), &[149.0, 74.5, 74.5]);

    // Cleaning: a missing entry counts as 1/n, negative and NaN ones as 0, all zero as equal.
    let cases: [(&[f32], [f32; 3]); 4] = [
        (&[], [298.0 / 3.0; 3]),
        (&[-1.0, f32::NAN, 2.0], [0.0, 0.0, 298.0]),
        (&[0.0, 0.0, 0.0], [298.0 / 3.0; 3]),
        (
            &[1.0, 1.0],
            [298.0 * 3.0 / 7.0, 298.0 * 3.0 / 7.0, 298.0 / 7.0],
        ),
    ];
    for (fractions, expected) in cases {
        let mut r = RenderSplit::new(Axis::Horizontal, 3);
        r.fractions = f(fractions);
        let mut fx = fixture(r, &wants, 0);
        fx.tree
            .layout(fx.root, BoxConstraints::loose(s(300.0, 100.0)));
        assert_close(&sizes(&fx), &expected);
        // The field itself is never rewritten.
        let kept = fx.tree.get::<RenderSplit>(fx.root).expect("split");
        assert_eq!(kept.fractions.len(), fractions.len());
    }

    // Gaps wider than the extent leave nothing for the panes, without panicking.
    let mut r = split(Axis::Horizontal, &[0.5, 0.5]);
    r.gap = 50.0;
    let mut fx = fixture(r, &[s(10.0, 10.0); 2], 0);
    fx.tree
        .layout(fx.root, BoxConstraints::loose(s(30.0, 10.0)));
    assert_close(&sizes(&fx), &[0.0, 0.0]);
}

#[test]
fn layout_split_02_minimums_and_collapsed_panes() {
    let wants = [s(10.0, 10.0); 3];
    let layout = |fractions: &[f32], mins: &[f32], collapsible: &[bool], extent: f32| {
        let mut r = split(Axis::Horizontal, fractions);
        r.gap = 0.0;
        r.min_sizes = f(mins);
        r.collapsible = b(collapsible);
        let mut fx = fixture(r, &wants[..fractions.len()], 0);
        fx.tree
            .layout(fx.root, BoxConstraints::tight(s(extent, 50.0)));
        sizes(&fx)
    };
    // Raised to its minimum, taking the space from the other pane.
    assert_close(
        &layout(&[0.1, 0.9], &[100.0, 0.0], &[], 300.0),
        &[100.0, 200.0],
    );
    // Taken in proportion to each pane's size above its own minimum (135 and 85 above).
    assert_close(
        &layout(&[0.1, 0.45, 0.45], &[100.0, 0.0, 50.0], &[], 300.0),
        &[
            100.0,
            135.0 - 70.0 * 135.0 / 220.0,
            135.0 - 70.0 * 85.0 / 220.0,
        ],
    );
    // A collapsible pane with fraction 0 gets exactly 0, whatever its minimum.
    assert_close(
        &layout(&[0.0, 1.0], &[80.0, 80.0], &[true, false], 100.0),
        &[0.0, 100.0],
    );
    // A non-collapsible one is raised to its minimum.
    assert_close(
        &layout(&[0.0, 1.0], &[80.0, 0.0], &[false, false], 100.0),
        &[80.0, 20.0],
    );
    // Minimums that don't fit are scaled down together to fill the space exactly.
    assert_close(
        &layout(&[0.5, 0.5], &[80.0, 80.0], &[], 100.0),
        &[50.0, 50.0],
    );
    assert_close(
        &layout(&[0.2, 0.8], &[120.0, 40.0], &[], 80.0),
        &[60.0, 20.0],
    );
}

#[test]
fn layout_split_03_placement_and_cross_axis() {
    let wants = [s(10.0, 40.0), s(10.0, 60.0), s(10.0, 20.0)];
    let mut fx = fixture(split(Axis::Horizontal, &[0.5, 0.25, 0.25]), &wants, 0);
    fx.tree
        .layout(fx.root, BoxConstraints::loose(s(300.0, 100.0)));
    let offsets: Vec<_> = fx.panes.iter().map(|p| fx.tree.offset(*p)).collect();
    assert_eq!(
        offsets,
        [
            Some(Vec2::new(0.0, 0.0)),
            Some(Vec2::new(150.0, 0.0)),
            Some(Vec2::new(225.5, 0.0)),
        ]
    );
    // Tight on both axes: the main size and the split's (maximum) cross extent.
    assert_eq!(
        fx.tree.constraints(fx.panes[1]),
        Some(BoxConstraints::tight(s(74.5, 100.0)))
    );
    assert_eq!(fx.tree.size(fx.panes[0]), Some(s(149.0, 100.0)));

    // An unbounded cross axis takes the largest pane's cross size, tight for every pane.
    let mut fx = fixture(split(Axis::Horizontal, &[0.5, 0.25, 0.25]), &wants, 0);
    assert_eq!(
        fx.tree
            .layout(fx.root, BoxConstraints::new(0.0, 300.0, 0.0, INF)),
        s(300.0, 60.0)
    );
    let heights: Vec<_> = fx
        .panes
        .iter()
        .map(|p| fx.tree.size(*p).map(|s| s.height))
        .collect();
    assert_eq!(heights, [Some(60.0); 3]);

    // Vertical: the same along y.
    let wants = [s(40.0, 10.0); 2];
    let mut fx = fixture(split(Axis::Vertical, &[0.25, 0.75]), &wants, 0);
    assert_eq!(
        fx.tree
            .layout(fx.root, BoxConstraints::loose(s(80.0, 201.0))),
        s(80.0, 201.0)
    );
    assert_close(&sizes(&fx), &[50.0, 150.0]);
    assert_eq!(fx.tree.offset(fx.panes[1]), Some(Vec2::new(0.0, 51.0)));
    assert_eq!(fx.tree.size(fx.panes[1]), Some(s(80.0, 150.0)));
}

#[test]
fn layout_split_04_handles_are_centred_on_the_gaps() {
    let wants = [s(10.0, 10.0); 3];
    let mut fx = fixture(split(Axis::Horizontal, &[0.5, 0.25, 0.25]), &wants, 0);
    fx.tree
        .layout(fx.root, BoxConstraints::loose(s(300.0, 100.0)));
    // The first gap is 149..150 (centre 149.5), the second 224.5..225.5 (centre 225).
    assert_eq!(fx.tree.size(fx.handles[0]), Some(s(8.0, 100.0)));
    assert_eq!(
        fx.tree.constraints(fx.handles[0]),
        Some(BoxConstraints::tight(s(8.0, 100.0)))
    );
    assert_eq!(fx.tree.offset(fx.handles[0]), Some(Vec2::new(145.5, 0.0)));
    assert_eq!(fx.tree.offset(fx.handles[1]), Some(Vec2::new(221.0, 0.0)));

    // A handle thinner than the gap is as thick as the gap.
    let mut r = split(Axis::Vertical, &[0.5, 0.5]);
    r.gap = 10.0;
    r.handle = 4.0;
    let mut fx = fixture(r, &[s(10.0, 10.0); 2], 0);
    fx.tree
        .layout(fx.root, BoxConstraints::tight(s(50.0, 110.0)));
    assert_eq!(fx.tree.size(fx.handles[0]), Some(s(50.0, 10.0)));
    assert_eq!(fx.tree.offset(fx.handles[0]), Some(Vec2::new(0.0, 50.0)));
}

#[test]
fn layout_split_05_unbounded_empty_and_extra_children() {
    // An unbounded main axis: each pane at its minimum (0 when collapsed), plus the gaps.
    let mut r = split(Axis::Horizontal, &[0.2, 0.0, 0.8]);
    r.min_sizes = f(&[50.0, 40.0, 30.0]);
    r.collapsible = b(&[false, true, false]);
    let mut fx = fixture(r, &[s(10.0, 20.0); 3], 0);
    assert_eq!(
        fx.tree
            .layout(fx.root, BoxConstraints::new(0.0, INF, 0.0, 20.0)),
        s(82.0, 20.0)
    );
    assert_close(&sizes(&fx), &[50.0, 0.0, 30.0]);

    // No panes: the smallest size.
    let mut tree = LayoutTree::new();
    let root = tree.insert(RenderSplit::new(Axis::Horizontal, 0));
    assert_eq!(
        tree.layout(root, BoxConstraints::new(10.0, 300.0, 5.0, 100.0)),
        s(10.0, 5.0)
    );

    // Children beyond 2n - 1 get zero size.
    let mut fx = fixture(split(Axis::Horizontal, &[0.5, 0.5]), &[s(10.0, 10.0); 2], 2);
    fx.tree
        .layout(fx.root, BoxConstraints::loose(s(100.0, 50.0)));
    for extra in &fx.extra {
        assert_eq!(fx.tree.size(*extra), Some(Size::ZERO));
    }
}
