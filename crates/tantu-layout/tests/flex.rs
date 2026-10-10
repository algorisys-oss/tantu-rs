//! Tests for `docs/specs/layout/flex.md`, rules LAYOUT-FLEX-01..14.

mod common;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{
    Axis, BoxConstraints, CrossAxisAlignment, FlexFit, FlexParentData, IntrinsicChildren,
    LayoutChildren, LayoutId, LayoutTree, MainAxisAlignment, MainAxisSize, RenderBox, RenderFlex,
};

const INF: f32 = f32::INFINITY;
const NAN: f32 = f32::NAN;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// One child: its natural size and optional flex parent data.
type Kid = (f32, f32, Option<FlexParentData>);

struct Flex {
    tree: LayoutTree,
    root: LayoutId,
    kids: Vec<LayoutId>,
}

fn flex(render: RenderFlex, kids: &[Kid]) -> Flex {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let ids: Vec<LayoutId> = kids
        .iter()
        .map(|&(w, h, data)| {
            let id = tree.insert(leaf(&log, "kid", w, h));
            if let Some(data) = data {
                tree.set_parent_data(id, Some(data));
            }
            id
        })
        .collect();
    tree.set_children(root, &ids).expect("valid");
    Flex {
        tree,
        root,
        kids: ids,
    }
}

impl Flex {
    fn layout(&mut self, c: BoxConstraints) -> Size {
        self.tree.layout(self.root, c)
    }

    fn offsets(&self) -> Vec<Vec2> {
        self.kids
            .iter()
            .map(|id| self.tree.offset(*id).expect("exists"))
            .collect()
    }

    fn sizes(&self) -> Vec<Size> {
        self.kids
            .iter()
            .map(|id| self.tree.size(*id).expect("laid out"))
            .collect()
    }

    fn constraints(&self, i: usize) -> BoxConstraints {
        self.tree.constraints(self.kids[i]).expect("laid out")
    }

    fn render(&self) -> &RenderFlex {
        self.tree.get::<RenderFlex>(self.root).expect("a flex")
    }
}

fn row() -> RenderFlex {
    RenderFlex::row()
}

fn with(f: impl FnOnce(&mut RenderFlex)) -> RenderFlex {
    let mut r = RenderFlex::row();
    f(&mut r);
    r
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[track_caller]
fn assert_offsets_close(actual: &[Vec2], expected: &[Vec2]) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!(
            close(a.x, e.x) && close(a.y, e.y),
            "{actual:?} vs {expected:?}"
        );
    }
}

/// `Expanded(1)` parent data.
fn e1() -> Option<FlexParentData> {
    Some(FlexParentData::expanded(1))
}

#[test]
fn layout_flex_01_constructors() {
    let r = RenderFlex::new(Axis::Vertical);
    assert_eq!(r.direction, Axis::Vertical);
    assert_eq!(r.main_axis_alignment, MainAxisAlignment::Start);
    assert_eq!(r.main_axis_size, MainAxisSize::Max);
    assert_eq!(r.cross_axis_alignment, CrossAxisAlignment::Center);
    assert_eq!(r.spacing, 0.0);
    assert_eq!(r.overflow(), 0.0);
    assert_eq!(RenderFlex::row(), RenderFlex::new(Axis::Horizontal));
    assert_eq!(RenderFlex::column(), RenderFlex::new(Axis::Vertical));
    assert_eq!(Axis::Horizontal.flip(), Axis::Vertical);
    assert_eq!(Axis::Vertical.flip(), Axis::Horizontal);
    assert_eq!(
        FlexParentData::expanded(2),
        FlexParentData {
            flex: 2,
            fit: FlexFit::Tight
        }
    );
    assert_eq!(
        FlexParentData::flexible(3),
        FlexParentData {
            flex: 3,
            fit: FlexFit::Loose
        }
    );
}

#[test]
fn layout_flex_02_equality_ignores_overflow() {
    let mut f = flex(row(), &[(60.0, 10.0, None), (70.0, 10.0, None)]);
    f.layout(loose(100.0, 100.0));
    assert_eq!(f.render().overflow(), 30.0);
    assert_eq!(*f.render(), RenderFlex::row());
    assert!(!f.tree.set(f.root, RenderFlex::row()));
    assert!(!f.tree.needs_layout(f.root));
    assert_ne!(row(), RenderFlex::column());
    assert_ne!(row(), with(|r| r.spacing = 1.0));
    assert_ne!(
        row(),
        with(|r| r.main_axis_alignment = MainAxisAlignment::End)
    );
    assert_ne!(row(), with(|r| r.main_axis_size = MainAxisSize::Min));
    assert_ne!(
        row(),
        with(|r| r.cross_axis_alignment = CrossAxisAlignment::End)
    );
}

#[test]
fn layout_flex_03_inflexible_children() {
    let mut f = flex(row(), &[(30.0, 20.0, None), (50.0, 40.0, None)]);
    assert_eq!(f.layout(loose(200.0, 100.0)), s(200.0, 40.0));
    assert_eq!(f.constraints(0), BoxConstraints::new(0.0, INF, 0.0, 100.0));
    assert_eq!(f.sizes(), [s(30.0, 20.0), s(50.0, 40.0)]);
    // Stretch: tight cross at the maximum.
    let mut f = flex(
        with(|r| r.cross_axis_alignment = CrossAxisAlignment::Stretch),
        &[(30.0, 20.0, None), (50.0, 40.0, None)],
    );
    assert_eq!(f.layout(loose(200.0, 100.0)), s(200.0, 100.0));
    assert_eq!(
        f.constraints(0),
        BoxConstraints::new(0.0, INF, 100.0, 100.0)
    );
    assert_eq!(f.sizes(), [s(30.0, 100.0), s(50.0, 100.0)]);
    // Spacing counts towards the allocated space.
    let mut f = flex(
        with(|r| {
            r.spacing = 10.0;
            r.main_axis_size = MainAxisSize::Min;
        }),
        &[(30.0, 20.0, None), (50.0, 40.0, None), (20.0, 10.0, None)],
    );
    assert_eq!(f.layout(loose(200.0, 100.0)), s(120.0, 40.0));
}

#[test]
fn layout_flex_04_flexible_children_share_free_space() {
    let mut f = flex(
        row(),
        &[
            (40.0, 20.0, None),
            (10.0, 10.0, e1()),
            (10.0, 10.0, Some(FlexParentData::expanded(3))),
        ],
    );
    assert_eq!(f.layout(loose(200.0, 100.0)), s(200.0, 20.0));
    assert_eq!(
        f.constraints(1),
        BoxConstraints::new(40.0, 40.0, 0.0, 100.0)
    );
    assert_eq!(
        f.constraints(2),
        BoxConstraints::new(120.0, 120.0, 0.0, 100.0)
    );
    assert_eq!(f.offsets()[1].x, 40.0);
    assert_eq!(f.offsets()[2].x, 80.0);
    // Uneven shares: the last flexible child takes the remainder, so they fill exactly.
    let mut f = flex(
        row(),
        &[
            (30.0, 20.0, None),
            (10.0, 10.0, e1()),
            (10.0, 10.0, e1()),
            (10.0, 10.0, e1()),
        ],
    );
    f.layout(loose(200.0, 100.0));
    let sizes = f.sizes();
    let flex_total: f32 = sizes[1..].iter().map(|s| s.width).sum();
    assert!(close(flex_total, 170.0), "{sizes:?}");
    let end = f.offsets()[3].x + sizes[3].width;
    assert!(close(end, 200.0), "{end}");
    // Loose: at most the share.
    let mut f = flex(
        row(),
        &[
            (40.0, 20.0, None),
            (10.0, 10.0, Some(FlexParentData::flexible(1))),
        ],
    );
    f.layout(loose(200.0, 100.0));
    assert_eq!(
        f.constraints(1),
        BoxConstraints::new(0.0, 160.0, 0.0, 100.0)
    );
    assert_eq!(f.sizes()[1], s(10.0, 10.0));
    // Spacing is taken before the shares.
    let mut f = flex(
        with(|r| r.spacing = 20.0),
        &[(40.0, 20.0, None), (10.0, 10.0, e1())],
    );
    f.layout(loose(200.0, 100.0));
    assert_eq!(
        f.constraints(1),
        BoxConstraints::new(140.0, 140.0, 0.0, 100.0)
    );
    assert_eq!(f.offsets()[1].x, 60.0);
}

#[test]
fn layout_flex_05_unbounded_main_axis() {
    let mut f = flex(row(), &[(40.0, 20.0, None), (10.0, 10.0, e1())]);
    let size = f.layout(BoxConstraints::new(0.0, INF, 0.0, 100.0));
    assert_eq!(f.constraints(1), BoxConstraints::new(0.0, INF, 0.0, 100.0));
    assert_eq!(size, s(50.0, 20.0));
}

#[test]
fn layout_flex_06_flex_size() {
    let kids = [(30.0, 20.0, None), (50.0, 40.0, None)];
    let mut f = flex(row(), &kids);
    assert_eq!(f.layout(loose(200.0, 100.0)), s(200.0, 40.0));
    let mut f = flex(with(|r| r.main_axis_size = MainAxisSize::Min), &kids);
    assert_eq!(f.layout(loose(200.0, 100.0)), s(80.0, 40.0));
    assert_eq!(
        f.layout(BoxConstraints::new(100.0, 200.0, 70.0, 100.0)),
        s(100.0, 70.0)
    );
    // Max in an unbounded main axis: the allocated space.
    let mut f = flex(row(), &kids);
    assert_eq!(
        f.layout(BoxConstraints::new(0.0, INF, 0.0, 100.0)),
        s(80.0, 40.0)
    );
    // Flexible children's actual sizes count in Min.
    let mut f = flex(
        with(|r| r.main_axis_size = MainAxisSize::Min),
        &[
            (30.0, 20.0, None),
            (10.0, 10.0, Some(FlexParentData::flexible(1))),
        ],
    );
    assert_eq!(f.layout(loose(200.0, 100.0)), s(40.0, 20.0));
}

#[test]
fn layout_flex_07_overflow() {
    let mut f = flex(row(), &[(60.0, 10.0, None), (70.0, 10.0, None)]);
    assert_eq!(f.layout(loose(100.0, 100.0)), s(100.0, 10.0));
    assert_eq!(f.render().overflow(), 30.0);
    assert_eq!(f.offsets(), [v(0.0, 0.0), v(60.0, 0.0)]);
    let mut f = flex(
        with(|r| r.spacing = 20.0),
        &[(45.0, 10.0, None), (45.0, 10.0, None)],
    );
    f.layout(loose(100.0, 100.0));
    assert_eq!(f.render().overflow(), 10.0);
    // Fitting resets it.
    f.layout(loose(200.0, 100.0));
    assert_eq!(f.render().overflow(), 0.0);
}

#[test]
fn layout_flex_08_main_axis_alignment() {
    let kids = [(20.0, 10.0, None), (20.0, 10.0, None), (20.0, 10.0, None)];
    let cases = [
        (MainAxisAlignment::Start, [0.0, 20.0, 40.0]),
        (MainAxisAlignment::End, [140.0, 160.0, 180.0]),
        (MainAxisAlignment::Center, [70.0, 90.0, 110.0]),
        (MainAxisAlignment::SpaceBetween, [0.0, 90.0, 180.0]),
        (
            MainAxisAlignment::SpaceAround,
            [140.0 / 6.0, 90.0, 180.0 - 140.0 / 6.0],
        ),
        (MainAxisAlignment::SpaceEvenly, [35.0, 90.0, 145.0]),
    ];
    for (alignment, xs) in cases {
        let mut f = flex(with(|r| r.main_axis_alignment = alignment), &kids);
        f.layout(loose(200.0, 10.0));
        let expected: Vec<Vec2> = xs.iter().map(|x| v(*x, 0.0)).collect();
        assert_offsets_close(&f.offsets(), &expected);
    }
    // SpaceBetween with one child: at the start.
    let mut f = flex(
        with(|r| r.main_axis_alignment = MainAxisAlignment::SpaceBetween),
        &[(20.0, 10.0, None)],
    );
    f.layout(loose(200.0, 10.0));
    assert_eq!(f.offsets(), [v(0.0, 0.0)]);
    // Spacing adds to the alignment's between space.
    let mut f = flex(
        with(|r| {
            r.main_axis_alignment = MainAxisAlignment::SpaceBetween;
            r.spacing = 10.0;
        }),
        &kids,
    );
    f.layout(loose(200.0, 10.0));
    assert_offsets_close(&f.offsets(), &[v(0.0, 0.0), v(90.0, 0.0), v(180.0, 0.0)]);
    // No remaining space when overflowing: alignment can't move children back.
    let mut f = flex(
        with(|r| r.main_axis_alignment = MainAxisAlignment::End),
        &[(60.0, 10.0, None), (70.0, 10.0, None)],
    );
    f.layout(loose(100.0, 10.0));
    assert_eq!(f.offsets(), [v(0.0, 0.0), v(60.0, 0.0)]);
}

#[test]
fn layout_flex_09_children_in_order() {
    let mut f = flex(
        with(|r| r.spacing = 5.0),
        &[(10.0, 10.0, None), (20.0, 10.0, None), (30.0, 10.0, None)],
    );
    f.layout(loose(200.0, 10.0));
    assert_eq!(f.offsets(), [v(0.0, 0.0), v(15.0, 0.0), v(40.0, 0.0)]);
}

#[test]
fn layout_flex_10_cross_axis_alignment() {
    let kids = [(30.0, 20.0, None), (30.0, 60.0, None)];
    let cases = [
        (CrossAxisAlignment::Start, [0.0, 0.0]),
        (CrossAxisAlignment::End, [40.0, 0.0]),
        (CrossAxisAlignment::Center, [20.0, 0.0]),
        (CrossAxisAlignment::Stretch, [0.0, 0.0]),
    ];
    for (alignment, ys) in cases {
        let mut f = flex(with(|r| r.cross_axis_alignment = alignment), &kids);
        f.layout(loose(200.0, 100.0));
        assert_eq!(
            f.offsets(),
            [v(0.0, ys[0]), v(30.0, ys[1])],
            "{alignment:?}"
        );
    }
    // A larger cross size than the children (tight constraints): offsets against it.
    let mut f = flex(
        with(|r| r.cross_axis_alignment = CrossAxisAlignment::End),
        &kids,
    );
    f.layout(BoxConstraints::tight(s(200.0, 100.0)));
    assert_eq!(f.offsets(), [v(0.0, 80.0), v(30.0, 40.0)]);
}

#[test]
fn layout_flex_11_column() {
    let kids = [(20.0, 20.0, None), (40.0, 20.0, None), (20.0, 20.0, None)];
    let mut f = flex(
        {
            let mut r = RenderFlex::column();
            r.main_axis_alignment = MainAxisAlignment::SpaceBetween;
            r
        },
        &kids,
    );
    assert_eq!(f.layout(loose(100.0, 200.0)), s(40.0, 200.0));
    assert_eq!(f.offsets(), [v(10.0, 0.0), v(0.0, 90.0), v(10.0, 180.0)]);
    assert_eq!(f.constraints(0), BoxConstraints::new(0.0, 100.0, 0.0, INF));
    let mut f = flex(
        RenderFlex::column(),
        &[(20.0, 20.0, None), (10.0, 10.0, e1())],
    );
    f.layout(loose(100.0, 200.0));
    assert_eq!(
        f.constraints(1),
        BoxConstraints::new(0.0, 100.0, 180.0, 180.0)
    );
    // The cross size is the widest child (20), so the 10-wide child centers at 5.
    assert_eq!(f.offsets()[1], v(5.0, 20.0));
}

#[test]
fn layout_flex_12_main_axis_intrinsics() {
    let kids = [
        (30.0, 20.0, None),
        (50.0, 40.0, None),
        (10.0, 10.0, e1()),
        (30.0, 15.0, Some(FlexParentData::expanded(2))),
    ];
    let mut f = flex(with(|r| r.spacing = 5.0), &kids);
    // 30 + 50 + max(10 / 1, 30 / 2) · 3 + 5 · 3
    assert_eq!(f.tree.min_intrinsic_width(f.root, INF), 140.0);
    assert_eq!(f.tree.max_intrinsic_width(f.root, INF), 140.0);
    let mut col = RenderFlex::column();
    col.spacing = 5.0;
    let mut f = flex(col, &kids);
    // 20 + 40 + max(10 / 1, 15 / 2) · 3 + 5 · 3
    assert_eq!(f.tree.min_intrinsic_height(f.root, INF), 105.0);
    assert_eq!(f.tree.max_intrinsic_height(f.root, INF), 105.0);
    let mut f = flex(row(), &[]);
    assert_eq!(f.tree.min_intrinsic_width(f.root, INF), 0.0);
}

/// A child with a fixed main (width) intrinsic whose cross (height) intrinsics echo the width
/// they are asked at.
struct Echo {
    width: f32,
}

impl RenderBox for Echo {
    fn perform_layout(&mut self, c: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        c.smallest()
    }

    fn min_intrinsic_width(&self, _: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.width
    }

    fn max_intrinsic_width(&self, _: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        self.width
    }

    fn min_intrinsic_height(&self, width: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        width
    }

    fn max_intrinsic_height(&self, width: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        width
    }
}

#[test]
fn layout_flex_13_cross_axis_intrinsics() {
    let build = |spacing: f32| {
        let mut tree = LayoutTree::new();
        let root = tree.insert(with(|r| r.spacing = spacing));
        let kids: Vec<LayoutId> = [
            (30.0, None),
            (50.0, None),
            (5.0, e1()),
            (5.0, Some(FlexParentData::flexible(3))),
        ]
        .into_iter()
        .map(|(width, data)| {
            let id = tree.insert(Echo { width });
            if let Some(data) = data {
                tree.set_parent_data(id, Some(data));
            }
            id
        })
        .collect();
        tree.set_children(root, &kids).expect("valid");
        (tree, root)
    };
    // Inflexible children at their widths (30, 50); flexible at (200 − 80) / 4 · flex.
    let (mut tree, root) = build(0.0);
    assert_eq!(tree.min_intrinsic_height(root, 200.0), 90.0);
    assert_eq!(tree.max_intrinsic_height(root, 200.0), 90.0);
    assert_eq!(tree.min_intrinsic_height(root, INF), INF);
    // Spacing (3 · 10) comes off the extent first: (200 − 80 − 30) / 4 · 3.
    let (mut tree, root) = build(10.0);
    assert_eq!(tree.max_intrinsic_height(root, 200.0), 67.5);
    // Too little room: the shares are 0, the inflexible children decide.
    let (mut tree, root) = build(0.0);
    assert_eq!(tree.min_intrinsic_height(root, 40.0), 50.0);
}

#[test]
fn layout_flex_14_robustness() {
    let mut f = flex(row(), &[]);
    assert_eq!(f.layout(loose(200.0, 100.0)), s(200.0, 0.0));
    assert_eq!(f.render().overflow(), 0.0);
    let mut f = flex(with(|r| r.main_axis_size = MainAxisSize::Min), &[]);
    assert_eq!(f.layout(loose(200.0, 100.0)), s(0.0, 0.0));
    assert_eq!(
        f.layout(BoxConstraints::tight(s(50.0, 60.0))),
        s(50.0, 60.0)
    );
    // Parent data of another type: inflexible.
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(row());
    let kid = tree.insert(leaf(&log, "kid", 10.0, 10.0));
    tree.set_parent_data(kid, Some(5u32));
    tree.set_children(root, &[kid]).expect("valid");
    tree.layout(root, loose(200.0, 100.0));
    assert_eq!(tree.size(kid), Some(s(10.0, 10.0)));
    // Huge flex factors don't overflow: two u32::MAX shares split the space evenly.
    let mut f = flex(
        row(),
        &[
            (1.0, 1.0, Some(FlexParentData::expanded(u32::MAX))),
            (1.0, 1.0, Some(FlexParentData::expanded(u32::MAX))),
        ],
    );
    f.layout(loose(200.0, 100.0));
    assert_eq!(f.sizes(), [s(100.0, 1.0), s(100.0, 1.0)]);
    // A zero flex is inflexible.
    let mut f = flex(row(), &[(10.0, 10.0, Some(FlexParentData::expanded(0)))]);
    f.layout(loose(200.0, 100.0));
    assert_eq!(f.sizes(), [s(10.0, 10.0)]);
    // Negative or NaN spacing counts as 0.
    for bad in [-10.0, NAN] {
        let mut f = flex(
            with(|r| r.spacing = bad),
            &[(10.0, 10.0, None), (10.0, 10.0, None)],
        );
        f.layout(loose(200.0, 100.0));
        assert_eq!(f.offsets()[1], v(10.0, 0.0), "{bad}");
    }
    // Nothing panics.
    let constraints = [
        BoxConstraints::UNCONSTRAINED,
        BoxConstraints::new(NAN, NAN, NAN, NAN),
        BoxConstraints::new(80.0, 20.0, INF, INF),
        BoxConstraints::tight(Size::ZERO),
    ];
    for alignment in [
        MainAxisAlignment::Start,
        MainAxisAlignment::SpaceAround,
        MainAxisAlignment::SpaceEvenly,
    ] {
        for cross in [CrossAxisAlignment::Stretch, CrossAxisAlignment::Center] {
            for direction in [Axis::Horizontal, Axis::Vertical] {
                let mut r = RenderFlex::new(direction);
                r.main_axis_alignment = alignment;
                r.cross_axis_alignment = cross;
                r.spacing = INF;
                let mut f = flex(
                    r,
                    &[
                        (INF, NAN, e1()),
                        (-5.0, 3.0, None),
                        (2.0, 2.0, Some(FlexParentData::flexible(7))),
                    ],
                );
                for c in constraints {
                    f.tree.mark_needs_layout(f.root);
                    let _ = f.layout(c);
                    for arg in [0.0, 50.0, INF, NAN] {
                        let _ = f.tree.min_intrinsic_width(f.root, arg);
                        let _ = f.tree.max_intrinsic_height(f.root, arg);
                    }
                }
            }
        }
    }
}
