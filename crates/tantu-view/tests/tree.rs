//! Tests for `docs/specs/view/tree.md`, rules VIEW-TREE-01..11.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_core::{Size, Vec2};
use tantu_layout::{
    BoxConstraints, FlexParentData, NoTextMeasure, RenderConstrainedBox, RenderFlex,
};
use tantu_reactive::{Signal, effect, on_cleanup, signal};
use tantu_view::{AnyView, BuildCx, ElementId, ElementKind, View, ViewTree};

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

/// A fixed-size render element; records its id in `out` if given.
struct Leaf {
    size: Size,
    out: Option<Rc<Cell<Option<ElementId>>>>,
}

fn leaf(w: f32, h: f32) -> AnyView {
    AnyView::new(Leaf {
        size: s(w, h),
        out: None,
    })
}

fn leaf_into(w: f32, h: f32, out: &Rc<Cell<Option<ElementId>>>) -> AnyView {
    AnyView::new(Leaf {
        size: s(w, h),
        out: Some(out.clone()),
    })
}

impl View for Leaf {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::sized(Some(self.size.width), Some(self.size.height)),
            [],
        );
        if let Some(out) = self.out {
            out.set(Some(id));
        }
        id
    }
}

/// A column render element.
struct Col(Vec<AnyView>);

impl View for Col {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderFlex::column(), self.0)
    }
}

/// Code run inside a region's build.
type Hook = Box<dyn FnOnce(&mut BuildCx<'_>)>;

/// A region with children and an optional hook run inside its build.
struct Region {
    children: Vec<AnyView>,
    hook: Option<Hook>,
}

fn region(children: Vec<AnyView>) -> AnyView {
    AnyView::new(Region {
        children,
        hook: None,
    })
}

fn region_with(children: Vec<AnyView>, hook: impl FnOnce(&mut BuildCx<'_>) + 'static) -> AnyView {
    AnyView::new(Region {
        children,
        hook: Some(Box::new(hook)),
    })
}

impl View for Region {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|cx| {
            if let Some(hook) = self.hook {
                hook(cx);
            }
            for child in self.children {
                child.build(cx);
            }
        })
    }
}

/// `child` with `FlexParentData::expanded(1)`; records whether it was accepted.
struct Expanded(AnyView, Rc<Cell<Option<bool>>>);

impl View for Expanded {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.0.build(cx);
        self.1
            .set(Some(cx.set_parent_data(id, FlexParentData::expanded(1))));
        id
    }
}

/// The single child of the root (the app's element).
fn app_element(tree: &ViewTree) -> ElementId {
    tree.children(tree.root())[0]
}

fn loose(w: f32, h: f32) -> BoxConstraints {
    BoxConstraints::loose(s(w, h))
}

#[test]
fn view_tree_01_new_runs_app_once() {
    let calls = Rc::new(Cell::new(0));
    let sig: Rc<Cell<Option<Signal<i32>>>> = Rc::default();
    let tree = {
        let (calls, sig) = (calls.clone(), sig.clone());
        ViewTree::new(move || {
            calls.set(calls.get() + 1);
            sig.set(Some(signal(41)));
            Col(vec![leaf(1.0, 1.0), leaf(2.0, 2.0)])
        })
    };
    assert_eq!(calls.get(), 1);
    let root = tree.root();
    assert_eq!(tree.kind(root), Some(ElementKind::Render));
    assert_eq!(tree.parent(root), None);
    assert_eq!(tree.len(), 4);
    assert!(!tree.is_empty());
    // The signal belongs to the tree's runtime.
    let s = sig.get().expect("created");
    assert_eq!(tree.enter(|| s.get()), 41);
}

#[test]
fn view_tree_02_render_elements() {
    let (a, b) = (Rc::default(), Rc::default());
    let tree = {
        let (a, b) = (Rc::clone(&a), Rc::clone(&b));
        ViewTree::new(move || Col(vec![leaf_into(1.0, 1.0, &a), leaf_into(2.0, 2.0, &b)]))
    };
    let col = app_element(&tree);
    let (a, b): (ElementId, ElementId) = (a.get().expect("built"), b.get().expect("built"));
    assert_eq!(tree.children(col), &[a, b]);
    assert_eq!(tree.parent(a), Some(col));
    assert_eq!(tree.parent(col), Some(tree.root()));
    assert_eq!(tree.kind(a), Some(ElementKind::Render));
    let ids = [tree.root(), col, a, b];
    for (i, x) in ids.iter().enumerate() {
        assert!(tree.layout_id(*x).is_some());
        for y in &ids[i + 1..] {
            assert_ne!(x, y);
        }
    }
    let la = tree.layout_id(a).expect("render");
    assert!(tree.layout_tree().get::<RenderConstrainedBox>(la).is_some());
}

#[test]
fn view_tree_03_region_elements() {
    let seen: Rc<RefCell<Vec<ElementId>>> = Rc::default();
    let tree = {
        let seen = seen.clone();
        ViewTree::new(move || {
            Col(vec![region_with(vec![leaf(1.0, 1.0)], move |cx| {
                seen.borrow_mut().push(cx.parent());
            })])
        })
    };
    let col = app_element(&tree);
    let region = tree.children(col)[0];
    assert_eq!(tree.kind(region), Some(ElementKind::Region));
    assert_eq!(tree.layout_id(region), None);
    assert_eq!(*seen.borrow(), [region]);
    assert_eq!(tree.children(region).len(), 1);
}

#[test]
fn view_tree_04_regions_flatten_into_layout_children() {
    let ids: Vec<Rc<Cell<Option<ElementId>>>> = (0..4).map(|_| Rc::default()).collect();
    let mut tree = {
        let ids = ids.clone();
        ViewTree::new(move || {
            Col(vec![
                leaf_into(10.0, 1.0, &ids[0]),
                region(vec![
                    leaf_into(10.0, 2.0, &ids[1]),
                    region(vec![leaf_into(10.0, 3.0, &ids[2])]),
                ]),
                leaf_into(10.0, 4.0, &ids[3]),
            ])
        })
    };
    let col = app_element(&tree);
    let layout = |id: &Rc<Cell<Option<ElementId>>>, t: &ViewTree| {
        t.layout_id(id.get().expect("built")).expect("render")
    };
    let expected: Vec<_> = ids.iter().map(|id| layout(id, &tree)).collect();
    let col_layout = tree.layout_id(col).expect("render");
    assert_eq!(tree.layout_tree().children(col_layout), expected.as_slice());
    tree.layout(loose(100.0, 100.0), &mut NoTextMeasure);
    let ys: Vec<f32> = expected
        .iter()
        .map(|l| tree.layout_tree().offset(*l).expect("exists").y)
        .collect();
    assert_eq!(ys, [0.0, 1.0, 3.0, 6.0]);
}

#[test]
fn view_tree_05_element_scopes() {
    let runs = Rc::new(Cell::new(0));
    let cleanups = Rc::new(Cell::new(0));
    let sig: Rc<Cell<Option<Signal<i32>>>> = Rc::default();
    let mut tree = {
        let (runs, cleanups, sig) = (runs.clone(), cleanups.clone(), sig.clone());
        ViewTree::new(move || {
            Col(vec![region_with(vec![], move |_| {
                let s = signal(0);
                sig.set(Some(s));
                effect(move || {
                    s.get();
                    runs.set(runs.get() + 1);
                });
                on_cleanup(move || cleanups.set(cleanups.get() + 1));
            })])
        })
    };
    let s = sig.get().expect("created");
    assert_eq!(runs.get(), 1);
    tree.enter(|| s.set(1));
    assert_eq!(runs.get(), 2);
    // Removing the column disposes the region's scope, which it owns.
    let col = app_element(&tree);
    let region = tree.children(col)[0];
    assert!(tree.scope(region).is_some());
    assert_eq!(tree.remove(col), 2);
    assert_eq!(cleanups.get(), 1);
    tree.enter(|| assert!(s.is_disposed()));
    assert_eq!(runs.get(), 2);
}

#[test]
fn view_tree_06_parent_data() {
    let (on_render, on_region) = (Rc::new(Cell::new(None)), Rc::new(Cell::new(None)));
    let out = Rc::new(Cell::new(None));
    let tree = {
        let (on_render, on_region, out) = (on_render.clone(), on_region.clone(), out.clone());
        ViewTree::new(move || {
            Col(vec![
                AnyView::new(Expanded(leaf_into(1.0, 1.0, &out), on_render)),
                AnyView::new(Expanded(region(vec![leaf(1.0, 1.0)]), on_region)),
            ])
        })
    };
    assert_eq!(on_render.get(), Some(true));
    assert_eq!(on_region.get(), Some(false));
    let leaf_layout = tree.layout_id(out.get().expect("built")).expect("render");
    assert_eq!(
        tree.layout_tree()
            .parent_data::<FlexParentData>(leaf_layout),
        Some(&FlexParentData::expanded(1))
    );
}

#[test]
fn view_tree_07_any_view() {
    let direct = ViewTree::new(|| Col(vec![leaf(3.0, 4.0)]));
    let erased = ViewTree::new(|| AnyView::new(Col(vec![leaf(3.0, 4.0)])));
    assert_eq!(direct.len(), erased.len());
    let el = app_element(&erased);
    assert_eq!(erased.kind(el), Some(ElementKind::Render));
    assert_eq!(erased.children(el).len(), 1);
}

#[test]
fn view_tree_08_root_and_layout() {
    let out = Rc::new(Cell::new(None));
    let mut tree = {
        let out = out.clone();
        ViewTree::new(move || leaf_into(30.0, 20.0, &out))
    };
    let leaf_layout = tree.layout_id(out.get().expect("built")).expect("render");
    let window = BoxConstraints::loose(s(800.0, 600.0));
    assert_eq!(tree.layout(window, &mut NoTextMeasure), s(800.0, 600.0));
    assert_eq!(tree.layout_tree().constraints(leaf_layout), Some(window));
    assert_eq!(tree.layout_tree().offset(leaf_layout), Some(Vec2::ZERO));
    assert_eq!(tree.layout_tree().size(leaf_layout), Some(s(30.0, 20.0)));
    // Unbounded: the root takes its largest child's size.
    let size = tree.layout(BoxConstraints::UNCONSTRAINED, &mut NoTextMeasure);
    assert_eq!(size, s(30.0, 20.0));
    // A region at the top works too.
    let mut tree = ViewTree::new(|| region(vec![leaf(5.0, 6.0), leaf(7.0, 2.0)]));
    let size = tree.layout(BoxConstraints::UNCONSTRAINED, &mut NoTextMeasure);
    assert_eq!(size, s(7.0, 6.0));
}

#[test]
fn view_tree_09_remove() {
    let cleanups = Rc::new(Cell::new(0));
    let ids: Vec<Rc<Cell<Option<ElementId>>>> = (0..3).map(|_| Rc::default()).collect();
    let mut tree = {
        let (cleanups, ids) = (cleanups.clone(), ids.clone());
        ViewTree::new(move || {
            Col(vec![
                leaf_into(10.0, 1.0, &ids[0]),
                region_with(
                    vec![leaf_into(10.0, 2.0, &ids[1]), leaf(10.0, 2.0)],
                    move |_| on_cleanup(move || cleanups.set(cleanups.get() + 1)),
                ),
                leaf_into(10.0, 3.0, &ids[2]),
            ])
        })
    };
    let col = app_element(&tree);
    let region = tree.children(col)[1];
    let inner = ids[1].get().expect("built");
    let inner_layout = tree.layout_id(inner).expect("render");
    let len = tree.len();
    assert_eq!(tree.remove(region), 3);
    assert_eq!(tree.len(), len - 3);
    assert_eq!(cleanups.get(), 1);
    assert!(!tree.contains(region) && !tree.contains(inner));
    assert!(!tree.layout_tree().contains(inner_layout));
    let first = ids[0].get().expect("built");
    let last = ids[2].get().expect("built");
    assert_eq!(tree.children(col), &[first, last]);
    let col_layout = tree.layout_id(col).expect("render");
    assert_eq!(
        tree.layout_tree().children(col_layout),
        &[
            tree.layout_id(first).expect("render"),
            tree.layout_id(last).expect("render")
        ]
    );
    tree.layout(loose(100.0, 100.0), &mut NoTextMeasure);
    let last_layout = tree.layout_id(last).expect("render");
    assert_eq!(
        tree.layout_tree().offset(last_layout),
        Some(Vec2::new(0.0, 1.0))
    );
    // The root and unknown ids.
    let n = tree.len();
    assert_eq!(tree.remove(tree.root()), 0);
    assert_eq!(tree.remove(region), 0);
    assert_eq!(tree.len(), n);
}

#[test]
fn view_tree_10_drop_runs_cleanups() {
    let cleanups = Rc::new(Cell::new(0));
    {
        let c = cleanups.clone();
        let _tree = ViewTree::new(move || {
            let c2 = c.clone();
            on_cleanup(move || c2.set(c2.get() + 1));
            Col(vec![region_with(vec![], move |_| {
                on_cleanup(move || c.set(c.get() + 1));
            })])
        });
        assert_eq!(cleanups.get(), 0);
    }
    assert_eq!(cleanups.get(), 2);
}

#[test]
fn view_tree_11_robustness() {
    let out = Rc::new(Cell::new(None));
    let sig: Rc<Cell<Option<Signal<i32>>>> = Rc::default();
    let mut tree = {
        let (out, sig) = (out.clone(), sig.clone());
        ViewTree::new(move || {
            sig.set(Some(signal(1)));
            Col(vec![leaf_into(1.0, 1.0, &out)])
        })
    };
    let gone = out.get().expect("built");
    tree.remove(gone);
    assert!(!tree.contains(gone));
    assert_eq!(tree.kind(gone), None);
    assert_eq!(tree.parent(gone), None);
    assert!(tree.children(gone).is_empty());
    assert_eq!(tree.layout_id(gone), None);
    assert!(tree.scope(gone).is_none());
    let foreign = ElementId::from_raw(u64::MAX).expect("non-zero");
    assert!(!tree.contains(foreign));
    assert_eq!(tree.remove(foreign), 0);
    // enter makes the runtime current only inside.
    let s = sig.get().expect("created");
    assert_eq!(tree.enter(|| s.get()), 1);
    assert_eq!(s.try_get(), None);
}
