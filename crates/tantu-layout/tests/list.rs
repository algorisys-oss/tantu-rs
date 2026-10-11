//! Tests for `docs/specs/layout/list.md`, rules LAYOUT-LIST-01..03.

mod common;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{
    BoxConstraints, LayoutId, LayoutTree, ListExtents, ListIndex, RenderListContent, ScrollAxis,
};

const INF: f32 = f32::INFINITY;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

/// A list content with leaves of `sizes`, the `i`-th carrying `ListIndex(indices[i])` when given.
fn list(
    render: RenderListContent,
    sizes: &[Size],
    indices: &[Option<usize>],
) -> (LayoutTree, LayoutId, Vec<LayoutId>, Log) {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let children: Vec<LayoutId> = sizes
        .iter()
        .enumerate()
        .map(|(i, size)| {
            let id = tree.insert(leaf(&log, "item", size.width, size.height));
            if let Some(Some(index)) = indices.get(i) {
                tree.set_parent_data(id, Some(ListIndex(*index)));
            }
            id
        })
        .collect();
    tree.set_children(root, &children).expect("valid");
    (tree, root, children, log)
}

#[test]
fn layout_list_01_layout() {
    let vertical = RenderListContent::new(ScrollAxis::Vertical, 50, 40.0);
    let (mut tree, root, items, _) = list(
        vertical.clone(),
        &[s(100.0, 10.0), s(100.0, 10.0)],
        &[Some(3), Some(7)],
    );
    let c = BoxConstraints::new(0.0, 300.0, 0.0, INF);
    assert_eq!(tree.layout(root, c), s(300.0, 2000.0));
    assert_eq!(
        tree.constraints(items[0]),
        Some(BoxConstraints::tight(s(300.0, 40.0)))
    );
    assert_eq!(tree.size(items[0]), Some(s(300.0, 40.0)));
    assert_eq!(tree.offset(items[0]), Some(Vec2::new(0.0, 120.0)));
    assert_eq!(tree.offset(items[1]), Some(Vec2::new(0.0, 280.0)));
    // Without an index a child takes its position; the main size is constrained.
    let (mut tree, root, items, _) = list(
        vertical.clone(),
        &[s(10.0, 10.0), s(10.0, 10.0)],
        &[None, None],
    );
    assert_eq!(
        tree.layout(root, BoxConstraints::loose(s(300.0, 500.0))),
        s(300.0, 500.0)
    );
    assert_eq!(tree.offset(items[1]), Some(Vec2::new(0.0, 40.0)));
    // An unbounded cross axis takes the widest child, and children get loose cross
    // constraints.
    let (mut tree, root, items, _) = list(
        vertical.clone(),
        &[s(120.0, 10.0), s(80.0, 10.0)],
        &[Some(0), Some(1)],
    );
    assert_eq!(
        tree.layout(root, BoxConstraints::UNCONSTRAINED),
        s(120.0, 2000.0)
    );
    assert_eq!(
        tree.constraints(items[0]),
        Some(BoxConstraints::new(0.0, INF, 40.0, 40.0))
    );
    let (mut tree, root, _, _) = list(vertical.clone(), &[], &[]);
    assert_eq!(
        tree.layout(root, BoxConstraints::UNCONSTRAINED),
        s(0.0, 2000.0)
    );
    // Horizontal, and a bad extent.
    let horizontal = RenderListContent::new(ScrollAxis::Horizontal, 10, 25.0);
    let (mut tree, root, items, _) = list(horizontal, &[s(10.0, 10.0)], &[Some(4)]);
    assert_eq!(
        tree.layout(root, BoxConstraints::new(0.0, INF, 0.0, 100.0)),
        s(250.0, 100.0)
    );
    assert_eq!(tree.offset(items[0]), Some(Vec2::new(100.0, 0.0)));
    assert_eq!(tree.size(items[0]), Some(s(25.0, 100.0)));
    let bad = RenderListContent::new(ScrollAxis::Vertical, 10, f32::NAN);
    let (mut tree, root, _, _) = list(bad, &[s(10.0, 10.0)], &[Some(0)]);
    assert_eq!(
        tree.layout(root, BoxConstraints::loose(s(300.0, 300.0))),
        s(300.0, 0.0)
    );
}

#[test]
fn layout_list_02_intrinsics_and_equality() {
    let vertical = RenderListContent::new(ScrollAxis::Vertical, 50, 40.0);
    let (mut tree, root, _, log) = list(
        vertical.clone(),
        &[s(120.0, 10.0), s(80.0, 10.0)],
        &[Some(0), Some(1)],
    );
    assert_eq!(tree.min_intrinsic_height(root, INF), 2000.0);
    assert_eq!(tree.max_intrinsic_height(root, 100.0), 2000.0);
    assert_eq!(tree.max_intrinsic_width(root, INF), 120.0);
    assert_eq!(tree.min_intrinsic_width(root, INF), 120.0);
    let (mut tree2, root2, _, _) = list(vertical.clone(), &[], &[]);
    assert_eq!(tree2.max_intrinsic_width(root2, INF), 0.0);
    tree.layout(root, BoxConstraints::loose(s(300.0, 300.0)));
    log.take();
    assert!(!tree.set(root, RenderListContent::new(ScrollAxis::Vertical, 50, 40.0)));
    tree.layout(root, BoxConstraints::loose(s(300.0, 300.0)));
    assert!(log.take().is_empty());
    assert!(tree.set(root, RenderListContent::new(ScrollAxis::Vertical, 51, 40.0)));
}

#[test]
fn layout_list_03_extent_table() {
    let table = ListExtents::table([10.0, 20.0, 30.0]);
    assert_eq!(
        (0..3).map(|i| table.start(i)).collect::<Vec<_>>(),
        [0.0, 10.0, 30.0]
    );
    assert_eq!(
        (0..3).map(|i| table.extent(i)).collect::<Vec<_>>(),
        [10.0, 20.0, 30.0]
    );
    assert_eq!(table.total(3), 60.0);
    assert_eq!(table.total(2), 30.0);
    assert_eq!(table.total(5), 60.0, "missing items count as 0");
    for (position, expected) in [
        (-5.0, 0),
        (0.0, 0),
        (9.9, 0),
        (10.0, 1),
        (29.9, 1),
        (30.0, 2),
        (100.0, 2),
    ] {
        assert_eq!(table.index_at(position, 3), Some(expected), "{position}");
    }
    assert_eq!(table.index_at(15.0, 2), Some(1));
    assert_eq!(table.index_at(15.0, 0), None);
    // Negative and NaN extents count as 0.
    let dirty = ListExtents::table([5.0, -1.0, f32::NAN, 5.0]);
    assert_eq!(dirty.total(4), 10.0);
    assert_eq!(dirty.extent(1), 0.0);
    // Fixed behaves as before.
    let fixed = ListExtents::Fixed(40.0);
    assert_eq!(fixed.start(3), 120.0);
    assert_eq!(fixed.extent(3), 40.0);
    assert_eq!(fixed.total(50), 2000.0);
    assert_eq!(fixed.index_at(85.0, 50), Some(2));
    assert_eq!(fixed.index_at(1e9, 50), Some(49));
    assert_eq!(fixed.index_at(3.0, 0), None);
    // The layout object places children by the table.
    let mut render = RenderListContent::new(ScrollAxis::Vertical, 3, 0.0);
    render.extents = table.clone();
    let (mut tree, root, items, _) =
        list(render, &[s(100.0, 5.0), s(100.0, 5.0)], &[Some(1), Some(2)]);
    let c = BoxConstraints::new(0.0, 300.0, 0.0, INF);
    assert_eq!(tree.layout(root, c), s(300.0, 60.0));
    assert_eq!(tree.size(items[0]), Some(s(300.0, 20.0)));
    assert_eq!(tree.offset(items[0]), Some(Vec2::new(0.0, 10.0)));
    assert_eq!(tree.size(items[1]), Some(s(300.0, 30.0)));
    assert_eq!(tree.offset(items[1]), Some(Vec2::new(0.0, 30.0)));
    // Horizontal, and a table longer than the count ignores the rest.
    let mut render = RenderListContent::new(ScrollAxis::Horizontal, 2, 0.0);
    render.extents = table;
    let (mut tree, root, items, _) = list(render, &[s(5.0, 50.0)], &[Some(1)]);
    assert_eq!(
        tree.layout(root, BoxConstraints::new(0.0, INF, 0.0, 100.0)),
        s(30.0, 100.0)
    );
    assert_eq!(tree.offset(items[0]), Some(Vec2::new(10.0, 0.0)));
}
