//! Tests for `docs/specs/layout/image.md`, rules LAYOUT-IMAGE-01..02.

use tantu_core::Size;
use tantu_layout::{BoxConstraints, LayoutId, LayoutTree, RenderImage};

const INF: f32 = f32::INFINITY;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn image(width: Option<f32>, height: Option<f32>) -> RenderImage {
    RenderImage {
        width,
        height,
        ..RenderImage::new(s(60.0, 40.0))
    }
}

fn one(render: RenderImage) -> (LayoutTree, LayoutId) {
    let mut tree = LayoutTree::new();
    let id = tree.insert(render);
    (tree, id)
}

#[test]
fn layout_image_01_size() {
    let cases = [
        (
            image(None, None),
            BoxConstraints::loose(s(300.0, 200.0)),
            s(60.0, 40.0),
        ),
        (
            image(Some(120.0), None),
            BoxConstraints::loose(s(300.0, 200.0)),
            s(120.0, 80.0),
        ),
        (
            image(None, Some(10.0)),
            BoxConstraints::loose(s(300.0, 200.0)),
            s(15.0, 10.0),
        ),
        (
            image(Some(120.0), Some(30.0)),
            BoxConstraints::loose(s(300.0, 200.0)),
            s(120.0, 30.0),
        ),
        (
            image(None, None),
            BoxConstraints::tight(s(300.0, 200.0)),
            s(300.0, 200.0),
        ),
        (
            image(None, None),
            BoxConstraints::loose(s(30.0, 100.0)),
            s(30.0, 20.0),
        ),
        (
            image(None, None),
            BoxConstraints::UNCONSTRAINED,
            s(60.0, 40.0),
        ),
        (
            RenderImage::new(Size::ZERO),
            BoxConstraints::loose(s(300.0, 200.0)),
            Size::ZERO,
        ),
    ];
    for (i, (render, constraints, expected)) in cases.into_iter().enumerate() {
        let (mut tree, id) = one(render);
        assert_eq!(tree.layout(id, constraints), expected, "case {i}");
    }
    assert_eq!(image(None, None), RenderImage::new(s(60.0, 40.0)));
}

#[test]
fn layout_image_02_intrinsics() {
    let (mut tree, id) = one(image(None, None));
    assert_eq!(tree.max_intrinsic_width(id, 20.0), 30.0);
    assert_eq!(tree.max_intrinsic_width(id, INF), 60.0);
    assert_eq!(tree.max_intrinsic_height(id, 30.0), 20.0);
    assert_eq!(tree.max_intrinsic_height(id, INF), 40.0);
    assert_eq!(tree.min_intrinsic_width(id, 20.0), 0.0);
    assert_eq!(tree.min_intrinsic_height(id, 30.0), 0.0);
    // With an explicit size the minimums equal the maximums.
    let (mut tree, id) = one(image(Some(120.0), None));
    assert_eq!(tree.min_intrinsic_width(id, INF), 120.0);
    assert_eq!(tree.max_intrinsic_width(id, INF), 120.0);
    assert_eq!(tree.min_intrinsic_height(id, INF), 80.0);
    assert_eq!(tree.max_intrinsic_height(id, INF), 80.0);
    // A tight query width wins over the explicit width (`enforce`), as in Flutter.
    assert_eq!(tree.max_intrinsic_height(id, 30.0), 20.0);
}
