//! Tests for `docs/specs/layout/viewport.md`, rules LAYOUT-VIEWPORT-01..02.

mod common;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{BoxConstraints, LayoutId, LayoutTree, RenderViewport, ScrollAxis};

const INF: f32 = f32::INFINITY;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn one(
    render: RenderViewport,
    child: Option<Size>,
) -> (LayoutTree, LayoutId, Option<LayoutId>, Log) {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let child = child.map(|size| {
        let id = tree.insert(leaf(&log, "child", size.width, size.height));
        tree.set_children(root, &[id]).expect("valid");
        id
    });
    (tree, root, child, log)
}

#[test]
fn layout_viewport_01_layout() {
    let loose = BoxConstraints::loose(s(300.0, 200.0));
    let (mut tree, root, child, _) = one(
        RenderViewport::new(ScrollAxis::Vertical),
        Some(s(300.0, 1000.0)),
    );
    let child = child.expect("child");
    assert_eq!(tree.layout(root, loose), s(300.0, 200.0));
    assert_eq!(
        tree.constraints(child),
        Some(BoxConstraints::new(0.0, 300.0, 0.0, INF))
    );
    assert_eq!(tree.size(child), Some(s(300.0, 1000.0)));
    assert_eq!(tree.offset(child), Some(Vec2::ZERO));
    assert_eq!(
        tree.get::<RenderViewport>(root).map(|v| v.content()),
        Some(s(300.0, 1000.0))
    );
    // A scrolled offset moves the child up; a short child makes the viewport short.
    let mut scrolled = RenderViewport::new(ScrollAxis::Vertical);
    scrolled.offset = 150.0;
    tree.set(root, scrolled);
    tree.layout(root, loose);
    assert_eq!(tree.offset(child), Some(Vec2::new(0.0, -150.0)));
    let (mut tree, root, _, _) = one(
        RenderViewport::new(ScrollAxis::Vertical),
        Some(s(300.0, 50.0)),
    );
    assert_eq!(tree.layout(root, loose), s(300.0, 50.0));
    // Horizontal: the width is unbounded for the child.
    let mut horizontal = RenderViewport::new(ScrollAxis::Horizontal);
    horizontal.offset = 40.0;
    let (mut tree, root, child, _) = one(horizontal, Some(s(1000.0, 100.0)));
    let child = child.expect("child");
    assert_eq!(tree.layout(root, loose), s(300.0, 100.0));
    assert_eq!(
        tree.constraints(child),
        Some(BoxConstraints::new(0.0, INF, 0.0, 200.0))
    );
    assert_eq!(tree.offset(child), Some(Vec2::new(-40.0, 0.0)));
    // Non-finite offsets count as 0; no child gives the smallest size.
    let mut odd = RenderViewport::new(ScrollAxis::Vertical);
    odd.offset = f32::NAN;
    let (mut tree, root, child, _) = one(odd, Some(s(300.0, 1000.0)));
    tree.layout(root, loose);
    assert_eq!(tree.offset(child.expect("child")), Some(Vec2::ZERO));
    let (mut tree, root, _, _) = one(RenderViewport::new(ScrollAxis::Vertical), None);
    assert_eq!(
        tree.layout(root, BoxConstraints::new(10.0, 300.0, 20.0, 200.0)),
        s(10.0, 20.0)
    );
    assert_eq!(
        tree.get::<RenderViewport>(root).map(|v| v.content()),
        Some(Size::ZERO)
    );
}

#[test]
fn layout_viewport_02_intrinsics_and_equality() {
    let (mut tree, root, _, _) = one(
        RenderViewport::new(ScrollAxis::Vertical),
        Some(s(300.0, 1000.0)),
    );
    assert_eq!(tree.min_intrinsic_width(root, INF), 300.0);
    assert_eq!(tree.max_intrinsic_width(root, INF), 300.0);
    assert_eq!(tree.min_intrinsic_height(root, INF), 1000.0);
    assert_eq!(tree.max_intrinsic_height(root, INF), 1000.0);
    let (mut tree, root, _, _) = one(RenderViewport::new(ScrollAxis::Vertical), None);
    assert_eq!(tree.max_intrinsic_height(root, INF), 0.0);
    // Equality ignores the content size: `set` with the same offset doesn't re-lay out.
    let (mut tree, root, _, log) = one(
        RenderViewport::new(ScrollAxis::Vertical),
        Some(s(300.0, 1000.0)),
    );
    tree.layout(root, BoxConstraints::loose(s(300.0, 200.0)));
    log.take();
    assert!(!tree.set(root, RenderViewport::new(ScrollAxis::Vertical)));
    tree.layout(root, BoxConstraints::loose(s(300.0, 200.0)));
    assert!(log.take().is_empty());
    let mut moved = RenderViewport::new(ScrollAxis::Vertical);
    moved.offset = 10.0;
    assert!(tree.set(root, moved));
    assert!(tree.needs_layout(root));
}
