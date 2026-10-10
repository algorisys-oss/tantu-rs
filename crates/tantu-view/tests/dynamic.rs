//! Tests for `docs/specs/view/dynamic.md`, rules VIEW-DYN-01..07.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_core::Size;
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_reactive::{Signal, on_cleanup, signal};
use tantu_scene::Scene;
use tantu_view::{
    AnyView, BuildCx, Dyn, ElementId, For, FrameReport, IntoProp, Show, View, ViewTree,
};

type Log = Rc<RefCell<Vec<String>>>;

/// A box of `width` × 10 that logs its build and, when it is removed, its cleanup.
struct Leaf {
    name: String,
    width: f32,
    log: Log,
}

impl View for Leaf {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::sized(Some(self.width), Some(10.0)),
            [],
        );
        self.log.borrow_mut().push(format!("build {}", self.name));
        if let Some(scope) = cx.tree().scope(id) {
            let (log, name) = (self.log.clone(), self.name.clone());
            scope.run(|| on_cleanup(move || log.borrow_mut().push(format!("drop {name}"))));
        }
        id
    }
}

fn leaf(log: &Log, name: impl Into<String>, width: f32) -> Leaf {
    Leaf {
        name: name.into(),
        width,
        log: log.clone(),
    }
}

struct Col(Vec<AnyView>);

impl View for Col {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderFlex::column(), self.0)
    }
}

fn frame(tree: &mut ViewTree) -> FrameReport {
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(800.0, 600.0)),
        &mut NoTextMeasure,
        &mut scene,
    )
}

fn app(tree: &ViewTree) -> ElementId {
    tree.children(tree.root())[0]
}

/// The widths of the column's layout children, in order.
fn widths(tree: &ViewTree) -> Vec<f32> {
    let col = tree.layout_id(app(tree)).expect("render");
    tree.layout_tree()
        .children(col)
        .iter()
        .map(|l| tree.layout_tree().size(*l).map_or(-1.0, |s| s.width))
        .collect()
}

/// Hands a value out of a tree's build.
fn slot<T: Copy>() -> Rc<Cell<Option<T>>> {
    Rc::new(Cell::new(None))
}

#[test]
fn view_dyn_01_dyn_rebuilds_at_frames() {
    let log = Log::default();
    let n = slot::<Signal<u32>>();
    let mut tree = {
        let (log, n) = (log.clone(), n.clone());
        ViewTree::new(move || {
            let count = signal(1u32);
            n.set(Some(count));
            Col(vec![AnyView::new(Dyn::new(move || {
                let c = count.get();
                leaf(&log, format!("v{c}"), 10.0 * c as f32)
            }))])
        })
    };
    let n = n.get().expect("created");
    frame(&mut tree);
    assert_eq!(widths(&tree), [10.0]);
    assert_eq!(*log.borrow(), ["build v1"]);
    let region = tree.children(app(&tree))[0];
    let first = tree.children(region)[0];
    tree.enter(|| n.set(2));
    assert!(tree.needs_frame());
    assert_eq!(tree.children(region), &[first]);
    let report = frame(&mut tree);
    assert_eq!(report.applied, 1);
    assert!(!tree.contains(first));
    assert_eq!(widths(&tree), [20.0]);
    assert_eq!(*log.borrow(), ["build v1", "drop v1", "build v2"]);
    // Coalesced.
    tree.enter(|| {
        n.set(3);
        n.set(4);
    });
    assert_eq!(frame(&mut tree).applied, 1);
    assert_eq!(widths(&tree), [40.0]);
    assert_eq!(log.borrow().last().map(String::as_str), Some("build v4"));
}

/// A box whose width is `local + global`, read by a binding.
struct Summed {
    local: Signal<f32>,
    global: Signal<f32>,
}

impl View for Summed {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(0.0), Some(10.0)), []);
        let (local, global) = (self.local, self.global);
        cx.bind(
            id,
            (move || local.get() + global.get()).into_prop(),
            |el, w| {
                el.update_render(|r: &mut RenderConstrainedBox| {
                    r.additional.min_width = w;
                    r.additional.max_width = w;
                });
            },
        );
        id
    }
}

#[test]
fn view_dyn_02_content_outlives_the_effect_rerun() {
    let log = Log::default();
    let sigs = slot::<(Signal<u32>, Signal<f32>)>();
    let mut tree = {
        let (log, sigs) = (log.clone(), sigs.clone());
        ViewTree::new(move || {
            let (version, global) = (signal(1u32), signal(0.0f32));
            sigs.set(Some((version, global)));
            Col(vec![AnyView::new(Dyn::new(move || {
                let v = version.get();
                // Component code: a local signal and a cleanup owned by this version.
                let local = signal(v as f32);
                let log = log.clone();
                on_cleanup(move || log.borrow_mut().push(format!("cleanup v{v}")));
                Summed { local, global }
            }))])
        })
    };
    let (version, global) = sigs.get().expect("created");
    frame(&mut tree);
    assert_eq!(widths(&tree), [1.0]);
    // The Dyn re-runs; the old content is still there, and still works.
    tree.enter(|| version.set(2));
    assert!(log.borrow().is_empty());
    tree.enter(|| global.set(5.0));
    frame(&mut tree);
    assert_eq!(*log.borrow(), ["cleanup v1"]);
    assert_eq!(widths(&tree), [7.0]);
    // A version replaced before any frame built it is disposed when it is replaced.
    tree.enter(|| version.set(3));
    tree.enter(|| version.set(4));
    assert_eq!(*log.borrow(), ["cleanup v1", "cleanup v3"]);
    frame(&mut tree);
    assert_eq!(*log.borrow(), ["cleanup v1", "cleanup v3", "cleanup v2"]);
    assert_eq!(widths(&tree), [9.0]);
}

#[test]
fn view_dyn_03_removing_a_dyn() {
    let log = Log::default();
    let n = slot::<Signal<u32>>();
    let mut tree = {
        let (log, n) = (log.clone(), n.clone());
        ViewTree::new(move || {
            let count = signal(1u32);
            n.set(Some(count));
            Col(vec![AnyView::new(Col(vec![AnyView::new(Dyn::new(
                move || leaf(&log, format!("v{}", count.get()), 5.0),
            ))]))])
        })
    };
    let n = n.get().expect("created");
    frame(&mut tree);
    tree.enter(|| n.set(2));
    let inner = tree.children(app(&tree))[0];
    tree.remove(inner);
    assert_eq!(frame(&mut tree).applied, 0);
    assert_eq!(*log.borrow(), ["build v1", "drop v1"]);
    tree.enter(|| n.set(3));
    assert!(!tree.needs_frame());
}

#[test]
fn view_dyn_04_show() {
    let log = Log::default();
    let sigs = slot::<(Signal<u32>, Signal<u32>)>();
    let mut tree = {
        let (log, sigs) = (log.clone(), sigs.clone());
        ViewTree::new(move || {
            let (x, y) = (signal(1u32), signal(0u32));
            sigs.set(Some((x, y)));
            let (log_a, log_b) = (log.clone(), log.clone());
            Col(vec![AnyView::new(
                Show::new(
                    move || x.get() > 5,
                    move || {
                        // Read at component level: must not cause rebuilds.
                        let _ = y.get();
                        leaf(&log_a, "then", 30.0)
                    },
                )
                .fallback(move || leaf(&log_b, "else", 10.0)),
            )])
        })
    };
    let (x, y) = sigs.get().expect("created");
    frame(&mut tree);
    assert_eq!(widths(&tree), [10.0]);
    // A change that doesn't flip the condition: nothing to do.
    tree.enter(|| x.set(2));
    assert!(!tree.needs_frame());
    tree.enter(|| x.set(9));
    assert_eq!(frame(&mut tree).applied, 1);
    assert_eq!(widths(&tree), [30.0]);
    tree.enter(|| y.set(7));
    assert!(!tree.needs_frame());
    tree.enter(|| x.set(1));
    frame(&mut tree);
    assert_eq!(widths(&tree), [10.0]);
    assert_eq!(
        *log.borrow(),
        [
            "build else",
            "drop else",
            "build then",
            "drop then",
            "build else"
        ]
    );
    // Without a fallback: nothing while false.
    let mut tree =
        ViewTree::new(move || Col(vec![AnyView::new(Show::new(|| false, || Col(vec![])))]));
    frame(&mut tree);
    assert!(widths(&tree).is_empty());
}

type Items = Signal<Vec<(u32, f32)>>;

/// A tree with a column holding a `For` over `items`.
fn list(log: &Log, initial: Vec<(u32, f32)>) -> (ViewTree, Items, Signal<u32>) {
    let sigs = Rc::new(RefCell::new(None));
    let tree = {
        let (log, sigs) = (log.clone(), sigs.clone());
        ViewTree::new(move || {
            let items = signal(initial);
            let z = signal(0u32);
            *sigs.borrow_mut() = Some((items, z));
            Col(vec![AnyView::new(For::new(
                move || items.get(),
                |item: &(u32, f32)| item.0,
                move |item: (u32, f32)| {
                    let _ = z.get(); // untracked: no rebuilds from this
                    leaf(&log, format!("k{}", item.0), item.1)
                },
            ))])
        })
    };
    let (items, z) = sigs.borrow_mut().take().expect("created");
    (tree, items, z)
}

#[test]
fn view_dyn_05_for_builds_items_in_order() {
    let log = Log::default();
    let (mut tree, _, z) = list(&log, vec![(1, 10.0), (2, 20.0), (3, 30.0)]);
    frame(&mut tree);
    assert_eq!(widths(&tree), [10.0, 20.0, 30.0]);
    assert_eq!(*log.borrow(), ["build k1", "build k2", "build k3"]);
    tree.enter(|| z.set(1));
    assert!(!tree.needs_frame());
}

#[test]
fn view_dyn_06_for_reconciles_by_key() {
    let log = Log::default();
    let (mut tree, items, _) = list(&log, vec![(1, 10.0), (2, 20.0), (3, 30.0)]);
    frame(&mut tree);
    let region = tree.children(app(&tree))[0];
    let before: Vec<ElementId> = tree.children(region).to_vec();
    log.borrow_mut().clear();
    // Key 1 comes back with a new width: it keeps its element and its old value.
    tree.enter(|| items.set(vec![(3, 30.0), (1, 99.0), (4, 40.0)]));
    assert_eq!(frame(&mut tree).applied, 1);
    let after = tree.children(region).to_vec();
    assert_eq!(after.len(), 3);
    assert_eq!(after[0], before[2]);
    assert_eq!(after[1], before[0]);
    assert!(!tree.contains(before[1]));
    assert_eq!(widths(&tree), [30.0, 10.0, 40.0]);
    assert_eq!(*log.borrow(), ["drop k2", "build k4"]);
    // Coalesced.
    log.borrow_mut().clear();
    tree.enter(|| {
        items.set(vec![(5, 50.0)]);
        items.set(vec![(4, 40.0), (6, 60.0)]);
    });
    assert_eq!(frame(&mut tree).applied, 1);
    assert_eq!(widths(&tree), [40.0, 60.0]);
    let mut events = log.borrow().clone();
    events.sort();
    assert_eq!(events, ["build k6", "drop k1", "drop k3"]);
}

#[test]
fn view_dyn_07_for_edge_cases() {
    let log = Log::default();
    let (mut tree, items, _) = list(&log, vec![(1, 10.0), (1, 11.0), (2, 20.0)]);
    frame(&mut tree);
    assert_eq!(widths(&tree), [10.0, 20.0]);
    tree.enter(|| items.set(vec![]));
    frame(&mut tree);
    assert!(widths(&tree).is_empty());
    let region = tree.children(app(&tree))[0];
    assert!(tree.children(region).is_empty());
    // Removing the For disposes every item.
    let (mut tree, _, _) = list(&log, vec![(7, 1.0), (8, 1.0)]);
    frame(&mut tree);
    log.borrow_mut().clear();
    let col = app(&tree);
    tree.remove(col);
    let mut events = log.borrow().clone();
    events.sort();
    assert_eq!(events, ["drop k7", "drop k8"]);
}
