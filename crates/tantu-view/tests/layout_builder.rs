//! Tests for `docs/specs/view/layout-builder.md`, rules VIEW-LB-01..06.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_core::Size;
use tantu_layout::{
    BoxConstraints, LayoutChildren, LayoutId, NoTextMeasure, RenderBox, RenderConstrainedBox,
};
use tantu_reactive::{Signal, on_cleanup, signal};
use tantu_scene::Scene;
use tantu_view::{AnyView, BuildCx, ElementId, LayoutBuilder, View, ViewTree};

type Log = Rc<RefCell<Vec<String>>>;

/// A box of `width` × 10 that logs its build and cleanup.
struct Leaf(f32, Log);

impl View for Leaf {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(self.0), Some(10.0)), []);
        self.1.borrow_mut().push(format!("build {}", self.0));
        if let Some(scope) = cx.tree().scope(id) {
            let (log, w) = (self.1.clone(), self.0);
            scope.run(|| on_cleanup(move || log.borrow_mut().push(format!("drop {w}"))));
        }
        id
    }
}

fn loose(w: f32, h: f32) -> BoxConstraints {
    BoxConstraints::loose(Size::new(w, h))
}

/// The layout nodes under `id`'s render element, depth first (to find the leaves).
fn leaf_widths(tree: &ViewTree) -> Vec<f32> {
    fn walk(tree: &ViewTree, id: LayoutId, out: &mut Vec<f32>) {
        let lt = tree.layout_tree();
        if lt.get::<RenderConstrainedBox>(id).is_some() {
            out.push(lt.size(id).map_or(-1.0, |s| s.width));
        }
        for child in lt.children(id) {
            walk(tree, *child, out);
        }
    }
    let mut out = Vec::new();
    if let Some(root) = tree.layout_id(tree.root()) {
        walk(tree, root, &mut out);
    }
    out
}

fn half_width(log: &Log) -> LayoutBuilder {
    let log = log.clone();
    LayoutBuilder::new(move |c| Leaf((c.max_width / 2.0).floor(), log.clone()))
}

/// Lays out its child with its own constraints and records the child's intrinsics.
struct IntrinsicProbe(Rc<Cell<Option<(f32, f32)>>>);

impl RenderBox for IntrinsicProbe {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let min_w = children.min_intrinsic_width(0, f32::INFINITY);
        let max_h = children.max_intrinsic_height(0, 100.0);
        self.0.set(Some((min_w, max_h)));
        children.layout(0, c);
        c.smallest()
    }
}

struct Probe(Rc<Cell<Option<(f32, f32)>>>, AnyView);

impl View for Probe {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(IntrinsicProbe(self.0), [self.1])
    }
}

#[test]
fn view_lb_01_render_element_and_size() {
    let log = Log::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || half_width(&log))
    };
    let lb = tree.children(tree.root())[0];
    let lb_layout = tree.layout_id(lb).expect("a render element");
    tree.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    assert_eq!(
        tree.layout_tree().size(lb_layout),
        Some(Size::new(400.0, 10.0))
    );
    // Intrinsics are 0, even with content.
    let seen = Rc::new(Cell::new(None));
    let mut probed = {
        let (log, seen) = (log.clone(), seen.clone());
        ViewTree::new(move || Probe(seen, AnyView::new(half_width(&log))))
    };
    probed.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    assert_eq!(seen.get(), Some((0.0, 0.0)));
    // No content: the smallest size.
    let mut empty = ViewTree::new(|| LayoutBuilder::new(|_| Nothing));
    empty.layout(
        BoxConstraints::new(5.0, 100.0, 7.0, 100.0),
        &mut NoTextMeasure,
    );
    let lb = empty.children(empty.root())[0];
    let lb_layout = empty.layout_id(lb).expect("render");
    assert_eq!(
        empty.layout_tree().size(lb_layout),
        Some(Size::new(5.0, 7.0))
    );
}

/// An empty region.
struct Nothing;

impl View for Nothing {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|_| {})
    }
}

#[test]
fn view_lb_02_built_during_the_first_layout() {
    let log = Log::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || half_width(&log))
    };
    assert!(log.borrow().is_empty(), "nothing built before layout");
    tree.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    assert_eq!(*log.borrow(), ["build 400"]);
    assert_eq!(leaf_widths(&tree), [400.0]);
}

#[test]
fn view_lb_03_rebuilt_when_constraints_change() {
    let log = Log::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || half_width(&log))
    };
    tree.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    tree.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    assert_eq!(*log.borrow(), ["build 400"]);
    tree.layout(loose(600.0, 600.0), &mut NoTextMeasure);
    assert_eq!(*log.borrow(), ["build 400", "drop 400", "build 300"]);
    assert_eq!(leaf_widths(&tree), [300.0]);
    // NaN constraints are compared bit for bit: one rebuild, not one per layout.
    let nan = BoxConstraints::new(f32::NAN, f32::NAN, 0.0, 100.0);
    tree.layout(nan, &mut NoTextMeasure);
    let after_first = log.borrow().len();
    tree.layout(nan, &mut NoTextMeasure);
    assert_eq!(log.borrow().len(), after_first);
}

#[test]
fn view_lb_04_signals_rebuild_at_frames() {
    let log = Log::default();
    let sig: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let mut tree = {
        let (log, sig) = (log.clone(), sig.clone());
        ViewTree::new(move || {
            let extra = signal(0.0f32);
            sig.set(Some(extra));
            let log = log.clone();
            LayoutBuilder::new(move |c| {
                Leaf((c.max_width / 2.0).floor() + extra.get(), log.clone())
            })
        })
    };
    let extra = sig.get().expect("created");
    let mut scene = Scene::new();
    tree.frame(loose(800.0, 600.0), &mut NoTextMeasure, &mut scene);
    tree.enter(|| extra.set(5.0));
    assert!(tree.needs_frame());
    tree.frame(loose(800.0, 600.0), &mut NoTextMeasure, &mut scene);
    assert_eq!(leaf_widths(&tree), [405.0]);
    assert_eq!(*log.borrow(), ["build 400", "drop 400", "build 405"]);
}

#[test]
fn view_lb_05_nested_builders_settle_in_one_call() {
    let log = Log::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || {
            let log = log.clone();
            LayoutBuilder::new(move |outer| {
                let log = log.clone();
                let quarter = outer.max_width / 4.0;
                Padded(
                    quarter,
                    AnyView::new(LayoutBuilder::new(move |inner| {
                        Leaf(inner.max_width.floor(), log.clone())
                    })),
                )
            })
        })
    };
    tree.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    // The inner builder sees 800 minus 2 · 200 of padding.
    assert_eq!(leaf_widths(&tree), [400.0]);
}

/// A sized-from-padding wrapper: lays its child out with `max_width - 2 * pad`.
struct Padded(f32, AnyView);

impl View for Padded {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(
            tantu_layout::RenderPadding::new(tantu_core::EdgeInsets::symmetric(self.0, 0.0)),
            [self.1],
        )
    }
}

/// `depth` LayoutBuilders nested inside each other, counting builder calls.
struct Nest(u32, Rc<Cell<u32>>);

impl View for Nest {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (depth, calls) = (self.0, self.1);
        LayoutBuilder::new(move |c| {
            calls.set(calls.get() + 1);
            if depth == 0 {
                AnyView::new(Fill(c.max_width))
            } else {
                AnyView::new(Nest(depth - 1, calls.clone()))
            }
        })
        .build(cx)
    }
}

#[test]
fn view_lb_05_round_limit() {
    // Each nesting level is built one round after its parent, so 20 levels need 20 rounds;
    // one layout call allows 16.
    let calls = Rc::new(Cell::new(0));
    let mut tree = {
        let calls = calls.clone();
        ViewTree::new(move || Nest(20, calls))
    };
    tree.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    assert_eq!(calls.get(), 16);
    // The next call carries on.
    tree.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    assert!(calls.get() > 16, "{}", calls.get());
}

/// A box exactly `width` wide.
struct Fill(f32);

impl View for Fill {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderConstrainedBox::sized(Some(self.0), Some(1.0)), [])
    }
}

#[test]
fn view_lb_06_removal() {
    let log = Log::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || Holder(AnyView::new(half_width(&log))))
    };
    tree.layout(loose(800.0, 600.0), &mut NoTextMeasure);
    let holder = tree.children(tree.root())[0];
    let lb = tree.children(holder)[0];
    tree.remove(lb);
    assert_eq!(*log.borrow(), ["build 400", "drop 400"]);
    tree.layout(loose(600.0, 600.0), &mut NoTextMeasure);
    assert_eq!(*log.borrow(), ["build 400", "drop 400"]);
}

struct Holder(AnyView);

impl View for Holder {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(tantu_layout::RenderFlex::column(), [self.0])
    }
}
