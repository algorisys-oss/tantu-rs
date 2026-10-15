//! Tests for `docs/specs/widgets/dock-area.md`, rules LAYOUT-DOCK-01..05.

mod common;

use std::sync::Arc;

use common::*;
use tantu_core::{Rect, Size, Vec2};
use tantu_layout::{
    Axis, BoxConstraints, DockPlan, DockPlanNode, DockSlot, LayoutId, LayoutTree, RenderDock,
};

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

/// A (keys 10, 11) | (B (20) / C (30)): a row (key 100) of A at 0.25 and a column (key 101).
fn ide_plan(row_fractions: Vec<f32>) -> DockPlan {
    DockPlan {
        nodes: vec![
            DockPlanNode::Split {
                key: 100,
                axis: Axis::Horizontal,
                fractions: row_fractions,
                children: vec![1, 2],
            },
            DockPlanNode::Group {
                key: 1,
                panels: vec![10, 11],
            },
            DockPlanNode::Split {
                key: 101,
                axis: Axis::Vertical,
                fractions: vec![0.5, 0.5],
                children: vec![3, 4],
            },
            DockPlanNode::Group {
                key: 2,
                panels: vec![20],
            },
            DockPlanNode::Group {
                key: 3,
                panels: vec![30],
            },
        ],
    }
}

/// A dock with one leaf child per slot (`None`: no parent data).
fn fixture(dock: RenderDock, slots: &[Option<DockSlot>]) -> (LayoutTree, LayoutId, Vec<LayoutId>) {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(dock);
    let children: Vec<LayoutId> = slots
        .iter()
        .map(|slot| {
            let id = tree.insert(leaf(&log, "child", 5.0, 5.0));
            if let Some(slot) = slot {
                tree.set_parent_data(id, Some(*slot));
            }
            id
        })
        .collect();
    tree.set_children(root, &children).expect("fresh nodes");
    (tree, root, children)
}

fn placed(tree: &LayoutTree, id: LayoutId) -> (Vec2, Size) {
    (
        tree.offset(id).expect("laid out"),
        tree.size(id).expect("laid out"),
    )
}

fn rects(tree: &LayoutTree, root: LayoutId) -> Vec<Rect> {
    tree.get::<RenderDock>(root)
        .expect("dock")
        .node_rects()
        .to_vec()
}

#[test]
fn layout_dock_01_groups_share_space_as_split_panes() {
    let slots = [Some(DockSlot::Panel(10)), Some(DockSlot::Panel(30))];
    let (mut tree, root, children) = fixture(
        RenderDock::new(Arc::new(ide_plan(vec![0.25, 0.75]))),
        &slots,
    );
    assert_eq!(
        tree.layout(root, BoxConstraints::loose(s(401.0, 301.0))),
        s(401.0, 301.0)
    );
    // 400 px across: A 100, the column 300; the column's 300 px down: B and C 150 each.
    assert_eq!(placed(&tree, children[0]).1.width, 100.0);
    assert_eq!(placed(&tree, children[1]).1.width, 300.0);
    // The group minimum is the panes' minimum.
    let (mut tree, root, children) =
        fixture(RenderDock::new(Arc::new(ide_plan(vec![0.1, 0.9]))), &slots);
    tree.layout(root, BoxConstraints::tight(s(401.0, 301.0)));
    assert_eq!(placed(&tree, children[0]).1.width, 80.0);
    assert_eq!(placed(&tree, children[1]).1.width, 320.0);
    // An unbounded axis takes 0; an empty plan lays out nothing but the placeholder.
    let (mut tree, root, _) = fixture(RenderDock::new(Arc::new(ide_plan(vec![0.5, 0.5]))), &[]);
    assert_eq!(
        tree.layout(root, BoxConstraints::new(0.0, f32::INFINITY, 0.0, 100.0)),
        s(0.0, 100.0)
    );
    let (mut tree, root, children) = fixture(
        RenderDock::new(Arc::new(DockPlan::default())),
        &[Some(DockSlot::Placeholder), Some(DockSlot::Panel(10))],
    );
    assert_eq!(
        tree.layout(root, BoxConstraints::loose(s(200.0, 100.0))),
        s(200.0, 100.0)
    );
    assert_eq!(placed(&tree, children[0]), (Vec2::ZERO, s(200.0, 100.0)));
    assert_eq!(placed(&tree, children[1]), (Vec2::ZERO, Size::ZERO));
}

#[test]
fn layout_dock_02_strips_panels_and_placeholder() {
    let slots = [
        Some(DockSlot::Strip(1)),
        Some(DockSlot::Panel(10)),
        Some(DockSlot::Panel(11)),
        Some(DockSlot::Strip(3)),
        Some(DockSlot::Panel(30)),
        Some(DockSlot::Placeholder),
    ];
    let (mut tree, root, c) = fixture(
        RenderDock::new(Arc::new(ide_plan(vec![0.25, 0.75]))),
        &slots,
    );
    tree.layout(root, BoxConstraints::loose(s(401.0, 301.0)));
    assert_eq!(placed(&tree, c[0]), (Vec2::ZERO, s(100.0, 36.0)));
    assert_eq!(
        tree.constraints(c[0]),
        Some(BoxConstraints::tight(s(100.0, 36.0)))
    );
    // Every panel the group lists sits below its strip.
    assert_eq!(placed(&tree, c[1]), (Vec2::new(0.0, 36.0), s(100.0, 265.0)));
    assert_eq!(placed(&tree, c[2]), (Vec2::new(0.0, 36.0), s(100.0, 265.0)));
    assert_eq!(
        placed(&tree, c[3]),
        (Vec2::new(101.0, 151.0), s(300.0, 36.0))
    );
    assert_eq!(
        placed(&tree, c[4]),
        (Vec2::new(101.0, 187.0), s(300.0, 114.0))
    );
    assert_eq!(placed(&tree, c[5]), (Vec2::ZERO, s(401.0, 301.0)));
    // A group shorter than the strip: the strip is clamped, the panels get no height.
    let mut dock = RenderDock::new(Arc::new(DockPlan {
        nodes: vec![DockPlanNode::Group {
            key: 1,
            panels: vec![10],
        }],
    }));
    dock.strip_height = 36.0;
    let (mut tree, root, c) = fixture(dock, &[Some(DockSlot::Strip(1)), Some(DockSlot::Panel(10))]);
    tree.layout(root, BoxConstraints::tight(s(120.0, 20.0)));
    assert_eq!(placed(&tree, c[0]), (Vec2::ZERO, s(120.0, 20.0)));
    assert_eq!(placed(&tree, c[1]), (Vec2::new(0.0, 20.0), s(120.0, 0.0)));
}

#[test]
fn layout_dock_03_dividers_are_centred_on_the_gaps() {
    let slots = [
        Some(DockSlot::Divider {
            split: 100,
            index: 0,
        }),
        Some(DockSlot::Divider {
            split: 101,
            index: 0,
        }),
    ];
    let (mut tree, root, c) = fixture(
        RenderDock::new(Arc::new(ide_plan(vec![0.25, 0.75]))),
        &slots,
    );
    tree.layout(root, BoxConstraints::loose(s(401.0, 301.0)));
    // The row's gap is x 100..101; the column's y 150..151 within x 101..401.
    assert_eq!(placed(&tree, c[0]), (Vec2::new(96.5, 0.0), s(8.0, 301.0)));
    assert_eq!(
        placed(&tree, c[1]),
        (Vec2::new(101.0, 146.5), s(300.0, 8.0))
    );
}

#[test]
fn layout_dock_04_unknown_slots_get_nothing() {
    let slots = [
        None,
        Some(DockSlot::Strip(99)),
        Some(DockSlot::Panel(99)),
        Some(DockSlot::Divider {
            split: 100,
            index: 5,
        }),
        Some(DockSlot::Divider { split: 1, index: 0 }),
        Some(DockSlot::Strip(100)),
    ];
    let (mut tree, root, c) = fixture(
        RenderDock::new(Arc::new(ide_plan(vec![0.25, 0.75]))),
        &slots,
    );
    tree.layout(root, BoxConstraints::loose(s(401.0, 301.0)));
    for child in c {
        assert_eq!(placed(&tree, child), (Vec2::ZERO, Size::ZERO));
    }
    // Malformed plans don't panic: children pointing backwards, to themselves or nowhere.
    let plan = DockPlan {
        nodes: vec![
            DockPlanNode::Split {
                key: 1,
                axis: Axis::Horizontal,
                fractions: vec![f32::NAN],
                children: vec![0, 7, 1],
            },
            DockPlanNode::Split {
                key: 2,
                axis: Axis::Vertical,
                fractions: vec![],
                children: vec![0],
            },
        ],
    };
    let (mut tree, root, _) = fixture(RenderDock::new(Arc::new(plan)), &[Some(DockSlot::Strip(1))]);
    tree.layout(root, BoxConstraints::loose(s(100.0, 100.0)));
}

#[test]
fn layout_dock_05_node_rects() {
    let (mut tree, root, _) = fixture(RenderDock::new(Arc::new(ide_plan(vec![0.25, 0.75]))), &[]);
    assert!(rects(&tree, root).is_empty());
    tree.layout(root, BoxConstraints::loose(s(401.0, 301.0)));
    assert_eq!(
        rects(&tree, root),
        [
            Rect::from_ltrb(0.0, 0.0, 401.0, 301.0),
            Rect::from_ltrb(0.0, 0.0, 100.0, 301.0),
            Rect::from_ltrb(101.0, 0.0, 401.0, 301.0),
            Rect::from_ltrb(101.0, 0.0, 401.0, 150.0),
            Rect::from_ltrb(101.0, 151.0, 401.0, 301.0),
        ]
    );
}
