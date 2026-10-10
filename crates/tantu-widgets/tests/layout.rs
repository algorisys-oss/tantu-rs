//! Tests for `docs/specs/widgets/layout.md`, rules WIDGETS-LAYOUT-01..06.

use std::cell::Cell;
use std::rc::Rc;

use tantu_view::core::{Color, EdgeInsets, Size, Vec2};
use tantu_view::layout::{
    Alignment, BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex, RenderPadding,
    RenderPositionedBox, RenderStack,
};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{Command, Scene};
use tantu_view::{BuildCx, Dyn, ElementId, Paint, PaintCx, View, ViewTree};
use tantu_widgets::{
    Align, Center, Column, ConstrainedBox, Expanded, Flexible, Padding, Positioned, Row, SizedBox,
    Spacer, Stack,
};

fn loose(w: f32, h: f32) -> BoxConstraints {
    BoxConstraints::loose(Size::new(w, h))
}

fn tight(w: f32, h: f32) -> BoxConstraints {
    BoxConstraints::tight(Size::new(w, h))
}

/// The app's top element (the root's only child).
fn top(tree: &ViewTree) -> ElementId {
    tree.children(tree.root())[0]
}

fn child(tree: &ViewTree, id: ElementId, i: usize) -> ElementId {
    tree.children(id)[i]
}

fn offset(tree: &ViewTree, id: ElementId) -> Vec2 {
    let layout = tree.layout_id(id).expect("a render element");
    tree.layout_tree().offset(layout).expect("laid out")
}

fn size(tree: &ViewTree, id: ElementId) -> Size {
    let layout = tree.layout_id(id).expect("a render element");
    tree.layout_tree().size(layout).expect("laid out")
}

fn frame(tree: &mut ViewTree, constraints: BoxConstraints) -> Scene {
    let mut scene = Scene::new();
    tree.frame(constraints, &mut NoTextMeasure, &mut scene);
    scene
}

#[test]
fn widgets_layout_01_one_render_element_with_defaults() {
    let tree = ViewTree::new(|| {
        Column::new()
            .child(Padding::all(4.0))
            .child(Center::new().child(SizedBox::shrink()))
            .child(Row::new())
            .child(Stack::new())
    });
    let column = top(&tree);
    assert_eq!(tree.children(column).len(), 4);
    let get = |id| tree.layout_id(id).expect("render element");
    let layout = tree.layout_tree();
    assert_eq!(
        layout.get::<RenderFlex>(get(column)),
        Some(&RenderFlex::column())
    );
    let padding = child(&tree, column, 0);
    assert!(tree.children(padding).is_empty());
    assert_eq!(
        layout.get::<RenderPadding>(get(padding)),
        Some(&RenderPadding::new(EdgeInsets::all(4.0)))
    );
    let center = child(&tree, column, 1);
    assert_eq!(
        layout.get::<RenderPositionedBox>(get(center)),
        Some(&RenderPositionedBox::center())
    );
    assert_eq!(tree.children(center).len(), 1);
    assert_eq!(
        layout.get::<RenderFlex>(get(child(&tree, column, 2))),
        Some(&RenderFlex::row())
    );
    assert_eq!(
        layout.get::<RenderStack>(get(child(&tree, column, 3))),
        Some(&RenderStack::new())
    );
}

#[test]
fn widgets_layout_02_layout_matches_the_render_objects() {
    let mut tree = ViewTree::new(|| {
        Column::new()
            .spacing(8.0)
            .child(SizedBox::new(50.0, 50.0))
            .child(SizedBox::new(50.0, 50.0))
    });
    tree.layout(loose(200.0, 200.0), &mut NoTextMeasure);
    let column = top(&tree);
    assert_eq!(offset(&tree, child(&tree, column, 1)).y, 58.0);

    let mut tree = ViewTree::new(|| Padding::all(16.0).child(SizedBox::new(10.0, 10.0)));
    tree.layout(loose(200.0, 200.0), &mut NoTextMeasure);
    let padding = top(&tree);
    assert_eq!(
        offset(&tree, child(&tree, padding, 0)),
        Vec2::new(16.0, 16.0)
    );
    assert_eq!(size(&tree, padding), Size::new(42.0, 42.0));

    let mut tree = ViewTree::new(|| Center::new().child(SizedBox::new(50.0, 50.0)));
    tree.layout(tight(200.0, 200.0), &mut NoTextMeasure);
    assert_eq!(
        offset(&tree, child(&tree, top(&tree), 0)),
        Vec2::new(75.0, 75.0)
    );

    let mut tree = ViewTree::new(|| {
        Align::new(Alignment::BOTTOM_RIGHT)
            .width_factor(2.0)
            .child(SizedBox::new(20.0, 10.0))
    });
    tree.layout(loose(200.0, 200.0), &mut NoTextMeasure);
    assert_eq!(size(&tree, top(&tree)), Size::new(40.0, 200.0));
    assert_eq!(
        offset(&tree, child(&tree, top(&tree), 0)),
        Vec2::new(20.0, 190.0)
    );

    let mut tree = ViewTree::new(|| {
        ConstrainedBox::new(BoxConstraints::new(30.0, 60.0, 30.0, 60.0))
            .child(SizedBox::new(100.0, 10.0))
    });
    tree.layout(loose(200.0, 200.0), &mut NoTextMeasure);
    assert_eq!(size(&tree, top(&tree)), Size::new(60.0, 30.0));

    let mut tree = ViewTree::new(|| SizedBox::width(70.0).child(SizedBox::new(10.0, 25.0)));
    tree.layout(loose(200.0, 200.0), &mut NoTextMeasure);
    assert_eq!(size(&tree, top(&tree)), Size::new(70.0, 25.0));
    let mut tree = ViewTree::new(|| SizedBox::height(70.0).child(SizedBox::new(10.0, 25.0)));
    tree.layout(loose(200.0, 200.0), &mut NoTextMeasure);
    assert_eq!(size(&tree, top(&tree)), Size::new(10.0, 70.0));
}

#[test]
fn widgets_layout_03_dynamic_props() {
    let gap: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let mut tree = {
        let gap = gap.clone();
        ViewTree::new(move || {
            let spacing = signal(8.0f32);
            gap.set(Some(spacing));
            Column::new()
                .spacing(spacing)
                .child(SizedBox::new(50.0, 50.0))
                .child(SizedBox::new(50.0, 50.0))
        })
    };
    let spacing = gap.get().expect("built");
    let constraints = loose(200.0, 200.0);
    frame(&mut tree, constraints);
    let column = top(&tree);
    let second = child(&tree, column, 1);
    assert_eq!(offset(&tree, second).y, 58.0);
    tree.enter(|| spacing.set(20.0));
    // Not before the frame.
    assert_eq!(offset(&tree, second).y, 58.0);
    frame(&mut tree, constraints);
    assert_eq!(offset(&tree, second).y, 70.0);
    // A closure prop works too.
    let mut tree = ViewTree::new(|| {
        let pad = signal(1.0f32);
        Padding::new(move || EdgeInsets::all(pad.get()))
    });
    frame(&mut tree, constraints);
    assert_eq!(size(&tree, top(&tree)), Size::new(2.0, 2.0));
}

#[test]
fn widgets_layout_03_equal_value_keeps_layout_clean() {
    let gap: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let mut tree = {
        let gap = gap.clone();
        ViewTree::new(move || {
            let spacing = signal(8.0f32);
            gap.set(Some(spacing));
            Column::new().spacing(move || spacing.get().max(10.0))
        })
    };
    let spacing = gap.get().expect("built");
    let constraints = loose(200.0, 200.0);
    frame(&mut tree, constraints);
    let layout = tree.layout_id(top(&tree)).expect("render element");
    // 9 → max(9, 10) = 10: the same spacing as before, so applying it marks nothing.
    tree.enter(|| spacing.set(9.0));
    let mut scene = Scene::new();
    let report = tree.frame(constraints, &mut NoTextMeasure, &mut scene);
    assert_eq!(report.applied, 1);
    assert!(!tree.layout_tree().needs_layout(layout));
    assert_eq!(
        tree.layout_tree()
            .get::<RenderFlex>(layout)
            .map(|f| f.spacing),
        Some(10.0)
    );
}

#[test]
fn widgets_layout_04_parent_data() {
    let mut tree = ViewTree::new(|| {
        Row::new()
            .child(SizedBox::width(50.0))
            .child(Expanded::new(SizedBox::height(10.0)))
            .child(Expanded::new(SizedBox::height(10.0)).flex(2))
    });
    tree.layout(tight(350.0, 100.0), &mut NoTextMeasure);
    let row = top(&tree);
    assert_eq!(size(&tree, child(&tree, row, 1)).width, 100.0);
    assert_eq!(size(&tree, child(&tree, row, 2)).width, 200.0);

    // Flexible: loose, so the child keeps its own width.
    let mut tree = ViewTree::new(|| {
        Row::new()
            .child(Flexible::new(SizedBox::new(30.0, 10.0)))
            .child(Expanded::new(SizedBox::height(10.0)))
    });
    tree.layout(tight(300.0, 100.0), &mut NoTextMeasure);
    let row = top(&tree);
    assert_eq!(size(&tree, child(&tree, row, 0)).width, 30.0);
    assert_eq!(size(&tree, child(&tree, row, 1)).width, 150.0);

    let mut tree = ViewTree::new(|| {
        Stack::new()
            .child(SizedBox::new(100.0, 100.0))
            .child(
                Positioned::new(SizedBox::shrink())
                    .left(10.0)
                    .top(20.0)
                    .width(30.0)
                    .height(40.0),
            )
            .child(Positioned::fill(SizedBox::shrink()))
    });
    tree.layout(loose(200.0, 200.0), &mut NoTextMeasure);
    let stack = top(&tree);
    let positioned = child(&tree, stack, 1);
    assert_eq!(offset(&tree, positioned), Vec2::new(10.0, 20.0));
    assert_eq!(size(&tree, positioned), Size::new(30.0, 40.0));
    assert_eq!(size(&tree, child(&tree, stack, 2)), Size::new(100.0, 100.0));

    // A region child: no parent data to set; it is built unwrapped, without panicking.
    let mut tree = ViewTree::new(|| {
        Row::new()
            .child(Expanded::new(Dyn::new(|| SizedBox::new(20.0, 10.0))))
            .child(Expanded::new(SizedBox::height(10.0)))
    });
    tree.layout(tight(300.0, 100.0), &mut NoTextMeasure);
    let row = top(&tree);
    let region = child(&tree, row, 0);
    assert_eq!(size(&tree, child(&tree, region, 0)).width, 20.0);
    assert_eq!(size(&tree, child(&tree, row, 1)).width, 280.0);
}

#[test]
fn widgets_layout_05_spacer_and_sized_boxes() {
    let mut tree = ViewTree::new(|| {
        Row::new()
            .child(SizedBox::width(100.0))
            .child(Spacer::new())
            .child(SizedBox::width(50.0))
    });
    tree.layout(tight(300.0, 100.0), &mut NoTextMeasure);
    let row = top(&tree);
    assert_eq!(offset(&tree, child(&tree, row, 2)).x, 250.0);
    assert_eq!(size(&tree, child(&tree, row, 1)).width, 150.0);

    let mut tree = ViewTree::new(|| Row::new().child(Spacer::new()).child(Spacer::new().flex(3)));
    tree.layout(tight(400.0, 100.0), &mut NoTextMeasure);
    let row = top(&tree);
    assert_eq!(size(&tree, child(&tree, row, 1)).width, 300.0);

    let mut tree = ViewTree::new(|| Center::new().child(SizedBox::shrink()));
    tree.layout(tight(200.0, 200.0), &mut NoTextMeasure);
    assert_eq!(size(&tree, child(&tree, top(&tree), 0)), Size::ZERO);
    let mut tree = ViewTree::new(|| Center::new().child(SizedBox::expand()));
    tree.layout(tight(200.0, 120.0), &mut NoTextMeasure);
    assert_eq!(
        size(&tree, child(&tree, top(&tree), 0)),
        Size::new(200.0, 120.0)
    );
    let _ = RenderConstrainedBox::expand();
}

/// A box that fills itself with a color.
struct Swatch(Color);

struct SwatchPaint(Color);

impl Paint for SwatchPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene().fill_rect(
            tantu_view::core::Rect::from_ltwh(0.0, 0.0, size.width, size.height),
            self.0,
        );
    }
}

impl View for Swatch {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(20.0), Some(20.0)), []);
        cx.set_paint(id, SwatchPaint(self.0));
        id
    }
}

#[test]
fn widgets_layout_06_layout_widgets_paint_nothing() {
    let mut tree = ViewTree::new(|| {
        Padding::all(8.0).child(
            Column::new()
                .child(SizedBox::new(10.0, 10.0))
                .child(Row::new().child(Center::new())),
        )
    });
    let scene = frame(&mut tree, loose(200.0, 200.0));
    assert!(
        scene
            .entries()
            .iter()
            .all(|e| !matches!(e.command, Command::Fill { .. } | Command::Stroke { .. }))
    );

    let mut tree = ViewTree::new(|| {
        Stack::new()
            .child(Swatch(Color::from_argb32(0xFFFF0000)))
            .child(Swatch(Color::from_argb32(0xFF0000FF)))
    });
    let scene = frame(&mut tree, loose(200.0, 200.0));
    let fills: Vec<Color> = scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { color, .. } => Some(color),
            _ => None,
        })
        .collect();
    assert_eq!(
        fills,
        [
            Color::from_argb32(0xFFFF0000),
            Color::from_argb32(0xFF0000FF)
        ]
    );
}
