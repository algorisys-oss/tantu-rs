//! Tests for `docs/specs/layout/grid.md`, rules LAYOUT-GRID-01..04.

mod common;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{BoxConstraints, GridSlot, LayoutId, LayoutTree, ListExtents, RenderGridBody};

const INF: f32 = f32::INFINITY;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn slot(row: usize, column: usize) -> Option<GridSlot> {
    Some(GridSlot { row, column })
}

/// A million 32 px rows by twenty 120 px columns.
fn big() -> RenderGridBody {
    RenderGridBody::new(
        ListExtents::Fixed(32.0),
        1_000_000,
        ListExtents::Fixed(120.0),
        20,
    )
}

/// Rows of 10, 20, 30 px and columns of 50 and 60 px.
fn tables() -> RenderGridBody {
    RenderGridBody::new(
        ListExtents::table([10.0, 20.0, 30.0]),
        3,
        ListExtents::table([50.0, 60.0]),
        2,
    )
}

/// A grid body with one 10 × 10 leaf per slot (a `None` slot carries no parent data).
fn grid(
    render: RenderGridBody,
    slots: &[Option<GridSlot>],
) -> (LayoutTree, LayoutId, Vec<LayoutId>, Log) {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let children: Vec<LayoutId> = slots
        .iter()
        .map(|slot| {
            let id = tree.insert(leaf(&log, "cell", 10.0, 10.0));
            if let Some(slot) = slot {
                tree.set_parent_data(id, Some(*slot));
            }
            id
        })
        .collect();
    tree.set_children(root, &children).expect("valid");
    (tree, root, children, log)
}

#[test]
fn layout_grid_01_size() {
    for slots in [vec![slot(0, 0)], vec![]] {
        let (mut tree, root, _, _) = grid(big(), &slots);
        assert_eq!(
            tree.layout(root, BoxConstraints::loose(s(600.0, 400.0))),
            s(600.0, 400.0)
        );
        assert_eq!(
            tree.layout(root, BoxConstraints::new(0.0, 600.0, 0.0, INF)),
            s(600.0, 32_000_000.0)
        );
        assert_eq!(
            tree.layout(root, BoxConstraints::UNCONSTRAINED),
            s(2400.0, 32_000_000.0)
        );
        // A tight constraint wins.
        assert_eq!(
            tree.layout(root, BoxConstraints::tight(s(100.0, 100.0))),
            s(100.0, 100.0)
        );
    }
}

#[test]
fn layout_grid_02_placement() {
    let mut body = big();
    body.offset = Vec2::new(100.0, 50.0);
    let (mut tree, root, cells, log) = grid(body, &[slot(3, 2), None, slot(1000, 10)]);
    tree.layout(root, BoxConstraints::loose(s(600.0, 400.0)));
    assert_eq!(
        tree.constraints(cells[0]),
        Some(BoxConstraints::tight(s(120.0, 32.0)))
    );
    assert_eq!(tree.size(cells[0]), Some(s(120.0, 32.0)));
    assert_eq!(tree.offset(cells[0]), Some(Vec2::new(140.0, 46.0)));
    // Without parent data: slot (0, 0).
    assert_eq!(tree.offset(cells[1]), Some(Vec2::new(-100.0, -50.0)));
    assert_eq!(tree.size(cells[1]), Some(s(120.0, 32.0)));
    // Far outside the bounds, still laid out and placed.
    assert_eq!(tree.offset(cells[2]), Some(Vec2::new(1100.0, 31_950.0)));
    assert_eq!(log.take().len(), 3);
    // Table extents, and a slot past the counts (0 extent, placed at the end).
    let (mut tree, root, cells, _) = grid(tables(), &[slot(2, 1), slot(5, 1)]);
    tree.layout(root, BoxConstraints::loose(s(600.0, 400.0)));
    assert_eq!(
        tree.constraints(cells[0]),
        Some(BoxConstraints::tight(s(60.0, 30.0)))
    );
    assert_eq!(tree.offset(cells[0]), Some(Vec2::new(50.0, 30.0)));
    assert_eq!(
        tree.constraints(cells[1]),
        Some(BoxConstraints::tight(s(60.0, 0.0)))
    );
    assert_eq!(tree.offset(cells[1]), Some(Vec2::new(50.0, 60.0)));
    // A non-finite offset component counts as 0.
    let mut body = tables();
    body.offset = Vec2::new(f32::NAN, INF);
    let (mut tree, root, cells, _) = grid(body, &[slot(1, 1)]);
    tree.layout(root, BoxConstraints::loose(s(600.0, 400.0)));
    assert_eq!(tree.offset(cells[0]), Some(Vec2::new(50.0, 10.0)));
}

#[test]
fn layout_grid_03_content_intrinsics_and_equality() {
    assert_eq!(big().content(), s(2400.0, 32_000_000.0));
    assert_eq!(tables().content(), s(110.0, 60.0));
    let (mut tree, root, cells, log) = grid(tables(), &[slot(0, 0)]);
    assert_eq!(tree.min_intrinsic_width(root, INF), 110.0);
    assert_eq!(tree.max_intrinsic_width(root, 20.0), 110.0);
    assert_eq!(tree.min_intrinsic_height(root, INF), 60.0);
    assert_eq!(tree.max_intrinsic_height(root, 20.0), 60.0);
    tree.layout(root, BoxConstraints::loose(s(300.0, 300.0)));
    log.take();
    assert!(!tree.set(root, tables()));
    tree.layout(root, BoxConstraints::loose(s(300.0, 300.0)));
    assert!(log.take().is_empty());
    // A changed offset re-lays out the body: the cell moves (its own constraints are
    // unchanged, so the layout cache keeps its size).
    let mut moved = tables();
    moved.offset = Vec2::new(0.0, 5.0);
    assert!(tree.set(root, moved));
    tree.layout(root, BoxConstraints::loose(s(300.0, 300.0)));
    assert_eq!(tree.offset(cells[0]), Some(Vec2::new(0.0, -5.0)));
}

#[test]
fn layout_grid_04_frozen_columns() {
    let mut body = big();
    body.offset = Vec2::new(100.0, 50.0);
    body.frozen_columns = 2;
    let (mut tree, root, cells, _) = grid(body, &[slot(0, 1), slot(0, 2), slot(3, 0)]);
    tree.layout(root, BoxConstraints::loose(s(600.0, 400.0)));
    // Frozen: the column's start, the row offset still applied.
    assert_eq!(tree.offset(cells[0]), Some(Vec2::new(120.0, -50.0)));
    assert_eq!(tree.offset(cells[2]), Some(Vec2::new(0.0, 46.0)));
    // Not frozen: scrolled.
    assert_eq!(tree.offset(cells[1]), Some(Vec2::new(140.0, -50.0)));
}
