//! Tests for `docs/specs/layout/single-child.md`, rules LAYOUT-SINGLE-01..15.

mod common;

use common::*;
use tantu_core::{EdgeInsets, Size, Vec2};
use tantu_layout::{
    Alignment, BoxConstraints, IntrinsicChildren, LayoutChildren, LayoutId, LayoutTree,
    RenderAspectRatio, RenderBox, RenderConstrainedBox, RenderFractionallySizedBox, RenderPadding,
    RenderPositionedBox,
};

const INF: f32 = f32::INFINITY;
const NAN: f32 = f32::NAN;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// A tree with `render` at the root and, if given, one leaf child of that size.
struct One {
    tree: LayoutTree,
    root: LayoutId,
    child: Option<LayoutId>,
}

fn one(render: impl RenderBox, child: Option<Size>) -> One {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let child = child.map(|size| {
        let id = tree.insert(leaf(&log, "child", size.width, size.height));
        tree.set_children(root, &[id]).expect("valid");
        id
    });
    One { tree, root, child }
}

impl One {
    fn layout(&mut self, c: BoxConstraints) -> Size {
        self.tree.layout(self.root, c)
    }

    fn child_constraints(&self) -> BoxConstraints {
        self.tree
            .constraints(self.child.expect("has a child"))
            .expect("laid out")
    }

    fn child_size(&self) -> Size {
        self.tree
            .size(self.child.expect("has a child"))
            .expect("laid out")
    }

    fn child_offset(&self) -> Vec2 {
        self.tree
            .offset(self.child.expect("has a child"))
            .expect("exists")
    }
}

/// A leaf whose intrinsic sizes echo their argument (to see what a parent passes down).
struct Echo;

impl RenderBox for Echo {
    fn perform_layout(&mut self, c: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        c.smallest()
    }

    fn min_intrinsic_width(&self, height: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        height
    }

    fn max_intrinsic_width(&self, height: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        height
    }

    fn min_intrinsic_height(&self, width: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        width
    }

    fn max_intrinsic_height(&self, width: f32, _: &mut IntrinsicChildren<'_>) -> f32 {
        width
    }
}

fn with_echo(render: impl RenderBox) -> (LayoutTree, LayoutId) {
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let echo = tree.insert(Echo);
    tree.set_children(root, &[echo]).expect("valid");
    (tree, root)
}

#[test]
fn layout_single_01_alignment_constants() {
    let cases = [
        (Alignment::TOP_LEFT, -1.0, -1.0),
        (Alignment::TOP_CENTER, 0.0, -1.0),
        (Alignment::TOP_RIGHT, 1.0, -1.0),
        (Alignment::CENTER_LEFT, -1.0, 0.0),
        (Alignment::CENTER, 0.0, 0.0),
        (Alignment::CENTER_RIGHT, 1.0, 0.0),
        (Alignment::BOTTOM_LEFT, -1.0, 1.0),
        (Alignment::BOTTOM_CENTER, 0.0, 1.0),
        (Alignment::BOTTOM_RIGHT, 1.0, 1.0),
    ];
    for (a, x, y) in cases {
        assert_eq!((a.x, a.y), (x, y));
    }
    assert_eq!(Alignment::default(), Alignment::CENTER);
    let odd = Alignment::new(3.0, NAN);
    assert_eq!(odd.x, 3.0);
    assert!(odd.y.is_nan());
}

#[test]
fn layout_single_02_along_offset() {
    let free = v(100.0, 50.0);
    assert_eq!(Alignment::TOP_LEFT.along_offset(free), v(0.0, 0.0));
    assert_eq!(Alignment::BOTTOM_RIGHT.along_offset(free), free);
    assert_eq!(Alignment::CENTER.along_offset(free), v(50.0, 25.0));
    assert_eq!(Alignment::new(0.5, -1.0).along_offset(free), v(75.0, 0.0));
    assert_eq!(
        Alignment::CENTER.along_offset(v(-20.0, -10.0)),
        v(-10.0, -5.0)
    );
    assert_eq!(
        Alignment::new(2.0, 0.0).along_offset(v(10.0, 10.0)),
        v(15.0, 5.0)
    );
}

#[test]
fn layout_single_03_padding_layout() {
    let padding = EdgeInsets::from_ltrb(5.0, 1.0, 15.0, 9.0); // horizontal 20, vertical 10
    let mut t = one(RenderPadding::new(padding), Some(s(30.0, 20.0)));
    assert_eq!(t.layout(loose(100.0, 100.0)), s(50.0, 30.0));
    assert_eq!(
        t.child_constraints(),
        BoxConstraints::new(0.0, 80.0, 0.0, 90.0)
    );
    assert_eq!(t.child_offset(), v(5.0, 1.0));
    // A child that wants more gets the deflated maximum.
    let mut t = one(RenderPadding::new(padding), Some(s(1000.0, 1000.0)));
    assert_eq!(t.layout(loose(100.0, 100.0)), s(100.0, 100.0));
    assert_eq!(t.child_size(), s(80.0, 90.0));
    // Without a child.
    let mut t = one(RenderPadding::new(padding), None);
    assert_eq!(t.layout(loose(100.0, 100.0)), s(20.0, 10.0));
    assert_eq!(
        t.layout(BoxConstraints::tight(s(200.0, 200.0))),
        s(200.0, 200.0)
    );
    assert_eq!(RenderPadding::new(padding).padding, padding);
}

#[test]
fn layout_single_04_padding_intrinsics() {
    let padding = EdgeInsets::from_ltrb(5.0, 1.0, 15.0, 9.0);
    let mut t = one(RenderPadding::new(padding), Some(s(30.0, 20.0)));
    assert_eq!(t.tree.min_intrinsic_width(t.root, 50.0), 50.0);
    assert_eq!(t.tree.max_intrinsic_width(t.root, 50.0), 50.0);
    assert_eq!(t.tree.min_intrinsic_height(t.root, 100.0), 30.0);
    assert_eq!(t.tree.max_intrinsic_height(t.root, 100.0), 30.0);
    let mut t = one(RenderPadding::new(padding), None);
    assert_eq!(t.tree.min_intrinsic_width(t.root, 50.0), 20.0);
    assert_eq!(t.tree.max_intrinsic_height(t.root, 50.0), 10.0);
    // What reaches the child: the argument minus the other axis' padding, not below 0.
    let (mut tree, root) = with_echo(RenderPadding::new(padding));
    assert_eq!(tree.min_intrinsic_width(root, 50.0), 40.0 + 20.0);
    assert_eq!(tree.max_intrinsic_width(root, 5.0), 0.0 + 20.0);
    assert_eq!(tree.min_intrinsic_height(root, 50.0), 30.0 + 10.0);
    assert_eq!(tree.max_intrinsic_height(root, INF), INF);
}

#[test]
fn layout_single_05_positioned_box() {
    let child = Some(s(30.0, 20.0));
    let mut t = one(RenderPositionedBox::center(), child);
    assert_eq!(t.layout(loose(100.0, 100.0)), s(100.0, 100.0));
    assert_eq!(t.child_constraints(), loose(100.0, 100.0));
    assert_eq!(t.child_offset(), v(35.0, 40.0));
    // Tight incoming constraints are loosened for the child.
    assert_eq!(
        t.layout(BoxConstraints::tight(s(100.0, 100.0))),
        s(100.0, 100.0)
    );
    assert_eq!(t.child_size(), s(30.0, 20.0));
    // A factor shrink-wraps that axis.
    let mut t = one(
        RenderPositionedBox {
            width_factor: Some(2.0),
            ..RenderPositionedBox::center()
        },
        child,
    );
    assert_eq!(t.layout(loose(100.0, 100.0)), s(60.0, 100.0));
    assert_eq!(t.child_offset(), v(15.0, 40.0));
    // Unbounded constraints shrink-wrap too.
    let mut t = one(RenderPositionedBox::center(), child);
    assert_eq!(
        t.layout(BoxConstraints::new(0.0, 100.0, 0.0, INF)),
        s(100.0, 20.0)
    );
    assert_eq!(t.child_offset(), v(35.0, 0.0));
    // Alignment.
    for (a, offset) in [
        (Alignment::TOP_LEFT, v(0.0, 0.0)),
        (Alignment::BOTTOM_RIGHT, v(70.0, 80.0)),
        (Alignment::CENTER_RIGHT, v(70.0, 40.0)),
    ] {
        let mut t = one(RenderPositionedBox::new(a), child);
        t.layout(loose(100.0, 100.0));
        assert_eq!(t.child_offset(), offset, "{a:?}");
    }
    // Without a child.
    let mut t = one(RenderPositionedBox::center(), None);
    assert_eq!(t.layout(loose(100.0, 100.0)), s(100.0, 100.0));
    assert_eq!(t.layout(BoxConstraints::UNCONSTRAINED), s(0.0, 0.0));
    let mut t = one(
        RenderPositionedBox {
            width_factor: Some(2.0),
            ..RenderPositionedBox::center()
        },
        None,
    );
    assert_eq!(t.layout(loose(100.0, 100.0)), s(0.0, 100.0));
    assert_eq!(
        RenderPositionedBox::default(),
        RenderPositionedBox::center()
    );
    assert_eq!(
        RenderPositionedBox::new(Alignment::TOP_LEFT),
        RenderPositionedBox {
            alignment: Alignment::TOP_LEFT,
            width_factor: None,
            height_factor: None
        }
    );
}

#[test]
fn layout_single_06_positioned_box_factors_and_intrinsics() {
    for bad in [-1.0, NAN] {
        let mut t = one(
            RenderPositionedBox {
                width_factor: Some(bad),
                ..RenderPositionedBox::center()
            },
            Some(s(30.0, 20.0)),
        );
        assert_eq!(t.layout(loose(100.0, 100.0)), s(0.0, 100.0), "{bad}");
        assert_eq!(t.child_offset(), v(-15.0, 40.0), "{bad}");
    }
    let mut t = one(
        RenderPositionedBox {
            width_factor: Some(2.0),
            height_factor: Some(3.0),
            ..RenderPositionedBox::center()
        },
        Some(s(30.0, 20.0)),
    );
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), 30.0);
    assert_eq!(t.tree.max_intrinsic_width(t.root, INF), 30.0);
    assert_eq!(t.tree.min_intrinsic_height(t.root, INF), 20.0);
    assert_eq!(t.tree.max_intrinsic_height(t.root, INF), 20.0);
    let mut t = one(RenderPositionedBox::center(), None);
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), 0.0);
}

#[test]
fn layout_single_07_constrained_box() {
    let child = Some(s(30.0, 20.0));
    let mut t = one(RenderConstrainedBox::sized(Some(50.0), None), child);
    assert_eq!(t.layout(loose(100.0, 100.0)), s(50.0, 20.0));
    assert_eq!(
        t.child_constraints(),
        BoxConstraints::new(50.0, 50.0, 0.0, 100.0)
    );
    assert_eq!(t.child_offset(), v(0.0, 0.0));
    // Within the incoming constraints.
    assert_eq!(t.layout(loose(40.0, 100.0)), s(40.0, 20.0));
    // Without a child.
    let mut t = one(RenderConstrainedBox::sized(Some(50.0), Some(40.0)), None);
    assert_eq!(t.layout(loose(100.0, 100.0)), s(50.0, 40.0));
    assert_eq!(
        t.layout(BoxConstraints::tight(s(10.0, 10.0))),
        s(10.0, 10.0)
    );
    let mut t = one(RenderConstrainedBox::expand(), None);
    assert_eq!(t.layout(loose(100.0, 80.0)), s(100.0, 80.0));
    let mut t = one(RenderConstrainedBox::shrink(), child);
    assert_eq!(t.layout(loose(100.0, 80.0)), s(0.0, 0.0));
    assert_eq!(
        t.layout(BoxConstraints::tight(s(10.0, 10.0))),
        s(10.0, 10.0)
    );
    // ConstrainedBox: a minimum the child must meet.
    let mut t = one(
        RenderConstrainedBox::new(BoxConstraints::new(40.0, INF, 0.0, INF)),
        child,
    );
    assert_eq!(t.layout(loose(100.0, 100.0)), s(40.0, 20.0));
}

#[test]
fn layout_single_08_constrained_box_constructors() {
    let c = BoxConstraints::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(RenderConstrainedBox::new(c).additional, c);
    assert_eq!(
        RenderConstrainedBox::sized(Some(5.0), None).additional,
        BoxConstraints::tight_for(Some(5.0), None)
    );
    assert_eq!(
        RenderConstrainedBox::expand().additional,
        BoxConstraints::expand(None, None)
    );
    assert_eq!(
        RenderConstrainedBox::shrink().additional,
        BoxConstraints::tight(Size::ZERO)
    );
}

#[test]
fn layout_single_09_constrained_box_intrinsics() {
    let child = Some(s(30.0, 20.0));
    let mut t = one(RenderConstrainedBox::sized(Some(50.0), None), child);
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), 50.0);
    assert_eq!(t.tree.max_intrinsic_width(t.root, INF), 50.0);
    assert_eq!(t.tree.min_intrinsic_height(t.root, INF), 20.0);
    let mut t = one(
        RenderConstrainedBox::new(BoxConstraints::new(0.0, 25.0, 25.0, INF)),
        child,
    );
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), 25.0);
    assert_eq!(t.tree.max_intrinsic_height(t.root, INF), 25.0);
    // expand(): infinite minimum width, so the child's intrinsics pass through.
    let mut t = one(RenderConstrainedBox::expand(), child);
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), 30.0);
    assert_eq!(t.tree.max_intrinsic_height(t.root, INF), 20.0);
    // No child: 0 through the same rules.
    let mut t = one(
        RenderConstrainedBox::new(BoxConstraints::new(10.0, 25.0, 0.0, INF)),
        None,
    );
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), 10.0);
}

#[test]
fn layout_single_10_fractionally_sized_box() {
    let child = Some(s(30.0, 20.0));
    let mut t = one(RenderFractionallySizedBox::new(Some(0.5), None), child);
    assert_eq!(t.layout(loose(100.0, 80.0)), s(50.0, 20.0));
    assert_eq!(
        t.child_constraints(),
        BoxConstraints::new(50.0, 50.0, 0.0, 80.0)
    );
    assert_eq!(t.child_offset(), v(0.0, 0.0));
    // A factor above 1 overflows, centered.
    let mut t = one(RenderFractionallySizedBox::new(Some(1.5), None), child);
    assert_eq!(t.layout(loose(100.0, 80.0)), s(100.0, 20.0));
    assert_eq!(t.child_size(), s(150.0, 20.0));
    assert_eq!(t.child_offset(), v(-25.0, 0.0));
    // Both axes.
    let mut t = one(RenderFractionallySizedBox::new(Some(0.5), Some(0.5)), child);
    assert_eq!(t.layout(loose(100.0, 80.0)), s(50.0, 40.0));
    // Alignment against a larger box (tight incoming constraints).
    let mut t = one(
        RenderFractionallySizedBox {
            alignment: Alignment::BOTTOM_RIGHT,
            ..RenderFractionallySizedBox::new(Some(0.5), None)
        },
        child,
    );
    assert_eq!(
        t.layout(BoxConstraints::tight(s(100.0, 80.0))),
        s(100.0, 80.0)
    );
    // The child's height comes from the tight incoming height: no vertical free space.
    assert_eq!(t.child_size(), s(50.0, 80.0));
    assert_eq!(t.child_offset(), v(50.0, 0.0));
    // Without a child.
    let mut t = one(RenderFractionallySizedBox::new(Some(0.5), None), None);
    assert_eq!(t.layout(loose(100.0, 80.0)), s(50.0, 0.0));
    // A negative or NaN factor counts as 0.
    for bad in [-2.0, NAN] {
        let mut t = one(RenderFractionallySizedBox::new(Some(bad), None), child);
        t.layout(loose(100.0, 80.0));
        assert_eq!(
            t.child_constraints(),
            BoxConstraints::new(0.0, 0.0, 0.0, 80.0),
            "{bad}"
        );
    }
    assert_eq!(
        RenderFractionallySizedBox::new(None, Some(1.0)),
        RenderFractionallySizedBox {
            alignment: Alignment::CENTER,
            width_factor: None,
            height_factor: Some(1.0)
        }
    );
}

#[test]
fn layout_single_11_fractionally_sized_box_intrinsics() {
    let mut t = one(
        RenderFractionallySizedBox::new(Some(0.5), None),
        Some(s(30.0, 20.0)),
    );
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), 60.0);
    assert_eq!(t.tree.max_intrinsic_width(t.root, INF), 60.0);
    assert_eq!(t.tree.min_intrinsic_height(t.root, INF), 20.0);
    assert_eq!(t.tree.max_intrinsic_height(t.root, INF), 20.0);
    let mut t = one(
        RenderFractionallySizedBox::new(Some(0.0), None),
        Some(s(30.0, 20.0)),
    );
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), INF);
}

#[test]
fn layout_single_12_aspect_ratio() {
    let cases = [
        (
            2.0,
            BoxConstraints::new(0.0, 100.0, 0.0, 100.0),
            s(100.0, 50.0),
        ),
        (
            2.0,
            BoxConstraints::new(0.0, 100.0, 0.0, 30.0),
            s(60.0, 30.0),
        ),
        (2.0, BoxConstraints::new(0.0, INF, 0.0, 40.0), s(80.0, 40.0)),
        (2.0, BoxConstraints::new(10.0, INF, 0.0, INF), s(10.0, 5.0)),
        (
            0.5,
            BoxConstraints::new(0.0, 100.0, 60.0, 100.0),
            s(50.0, 100.0),
        ),
        (
            4.0,
            BoxConstraints::new(90.0, 100.0, 0.0, 20.0),
            s(90.0, 20.0),
        ),
        (
            1.0,
            BoxConstraints::new(0.0, 100.0, 120.0, 200.0),
            s(100.0, 120.0),
        ),
        (3.0, BoxConstraints::tight(s(10.0, 70.0)), s(10.0, 70.0)),
    ];
    for (ratio, c, expected) in cases {
        let mut t = one(RenderAspectRatio::new(ratio), Some(s(5.0, 5.0)));
        assert_eq!(t.layout(c), expected, "{ratio} in {c:?}");
        assert_eq!(
            t.child_constraints(),
            BoxConstraints::tight(expected),
            "{ratio} in {c:?}"
        );
        assert_eq!(t.child_offset(), v(0.0, 0.0));
        let mut t = one(RenderAspectRatio::new(ratio), None);
        assert_eq!(t.layout(c), expected, "no child: {ratio} in {c:?}");
    }
    assert_eq!(RenderAspectRatio::new(1.5).aspect_ratio, 1.5);
}

#[test]
fn layout_single_13_bad_aspect_ratio() {
    let c = BoxConstraints::new(10.0, 100.0, 20.0, 100.0);
    for bad in [0.0, -1.0, NAN, INF, -INF] {
        let mut t = one(RenderAspectRatio::new(bad), Some(s(50.0, 50.0)));
        assert_eq!(t.layout(c), s(10.0, 20.0), "{bad}");
        assert_eq!(
            t.child_constraints(),
            BoxConstraints::tight(s(10.0, 20.0)),
            "{bad}"
        );
    }
}

#[test]
fn layout_single_14_aspect_ratio_intrinsics() {
    let mut t = one(RenderAspectRatio::new(2.0), Some(s(30.0, 20.0)));
    assert_eq!(t.tree.min_intrinsic_width(t.root, 10.0), 20.0);
    assert_eq!(t.tree.max_intrinsic_width(t.root, 10.0), 20.0);
    assert_eq!(t.tree.min_intrinsic_height(t.root, 10.0), 5.0);
    assert_eq!(t.tree.max_intrinsic_height(t.root, 10.0), 5.0);
    assert_eq!(t.tree.min_intrinsic_width(t.root, INF), 30.0);
    assert_eq!(t.tree.max_intrinsic_height(t.root, INF), 20.0);
    let mut t = one(RenderAspectRatio::new(-1.0), Some(s(30.0, 20.0)));
    assert_eq!(t.tree.min_intrinsic_width(t.root, 10.0), 30.0);
    assert_eq!(t.tree.min_intrinsic_height(t.root, 10.0), 20.0);
}

/// All five layouts, with ordinary and malformed properties.
fn every_layout() -> Vec<Box<dyn Fn(&mut LayoutTree) -> LayoutId>> {
    vec![
        Box::new(|t| t.insert(RenderPadding::new(EdgeInsets::all(4.0)))),
        Box::new(|t| {
            t.insert(RenderPadding::new(EdgeInsets::from_ltrb(
                NAN, -5.0, INF, 1.0,
            )))
        }),
        Box::new(|t| t.insert(RenderPositionedBox::center())),
        Box::new(|t| {
            t.insert(RenderPositionedBox {
                alignment: Alignment::new(NAN, INF),
                width_factor: Some(INF),
                height_factor: Some(-3.0),
            })
        }),
        Box::new(|t| t.insert(RenderConstrainedBox::sized(Some(10.0), Some(10.0)))),
        Box::new(|t| {
            t.insert(RenderConstrainedBox::new(BoxConstraints::new(
                NAN, -1.0, INF, 0.0,
            )))
        }),
        Box::new(|t| t.insert(RenderFractionallySizedBox::new(Some(0.5), Some(2.0)))),
        Box::new(|t| t.insert(RenderFractionallySizedBox::new(Some(INF), Some(NAN)))),
        Box::new(|t| t.insert(RenderAspectRatio::new(1.5))),
        Box::new(|t| t.insert(RenderAspectRatio::new(NAN))),
    ]
}

#[test]
fn layout_single_15_shared_rules() {
    let log = Log::default();
    let constraints = [
        loose(100.0, 100.0),
        BoxConstraints::tight(s(30.0, 40.0)),
        BoxConstraints::UNCONSTRAINED,
        BoxConstraints::new(NAN, NAN, NAN, NAN),
        BoxConstraints::new(80.0, 20.0, INF, INF),
    ];
    for make in every_layout() {
        for children in [0, 1, 3] {
            let mut tree = LayoutTree::new();
            let root = make(&mut tree);
            let kids: Vec<LayoutId> = (0..children)
                .map(|i| tree.insert(leaf(&log, "kid", 10.0 * (i + 1) as f32, 5.0)))
                .collect();
            tree.set_children(root, &kids).expect("valid");
            for c in constraints {
                tree.mark_needs_layout(root);
                let _ = tree.layout(root, c);
                for arg in [0.0, 50.0, INF, NAN] {
                    let _ = tree.min_intrinsic_width(root, arg);
                    let _ = tree.max_intrinsic_width(root, arg);
                    let _ = tree.min_intrinsic_height(root, arg);
                    let _ = tree.max_intrinsic_height(root, arg);
                }
            }
            // Only the first child is ever laid out.
            for extra in kids.iter().skip(1) {
                assert_eq!(tree.size(*extra), None);
                assert_eq!(tree.offset(*extra), Some(Vec2::ZERO));
            }
        }
    }
    log.take();
    // Unchanged properties don't re-lay out through `set`.
    let mut tree = LayoutTree::new();
    let root = tree.insert(RenderPadding::new(EdgeInsets::all(4.0)));
    tree.layout(root, loose(100.0, 100.0));
    assert!(!tree.set(root, RenderPadding::new(EdgeInsets::all(4.0))));
    assert!(!tree.needs_layout(root));
    assert!(tree.set(root, RenderPadding::new(EdgeInsets::all(5.0))));
    let root2 = tree.insert(RenderAspectRatio::new(2.0));
    tree.layout(root2, loose(100.0, 100.0));
    assert!(!tree.set(root2, RenderAspectRatio::new(2.0)));
    assert!(!tree.set(root2, RenderAspectRatio::new(2.0)) && !tree.needs_layout(root2));
}
