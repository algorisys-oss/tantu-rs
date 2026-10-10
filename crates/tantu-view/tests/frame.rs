//! Tests for `docs/specs/view/frame.md`, rules VIEW-FRAME-01..08.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use tantu_core::{Color, Rect, Size};
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_reactive::{Signal, batch, memo, signal};
use tantu_scene::{Command, Scene};
use tantu_view::{
    AnyView, BuildCx, ElementId, ElementMut, FrameReport, IntoProp, Paint, PaintCx, Prop, View,
    ViewTree,
};

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

/// A box whose width follows a prop; records each value applied.
struct SizedBox {
    width: Prop<f32>,
    applied: Rc<RefCell<Vec<f32>>>,
    out: Rc<Cell<Option<ElementId>>>,
}

impl View for SizedBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(0.0), Some(10.0)), []);
        let applied = self.applied;
        cx.bind(id, self.width, move |el, width| {
            applied.borrow_mut().push(width);
            el.update_render(|r: &mut RenderConstrainedBox| {
                r.additional.min_width = width;
                r.additional.max_width = width;
            });
        });
        self.out.set(Some(id));
        id
    }
}

struct Fixture {
    tree: ViewTree,
    width: Signal<f32>,
    applied: Rc<RefCell<Vec<f32>>>,
    element: ElementId,
}

/// A tree with one box whose width is `width * 10` (a dynamic prop).
fn fixture() -> Fixture {
    let applied: Rc<RefCell<Vec<f32>>> = Rc::default();
    let out = Rc::new(Cell::new(None));
    let sig: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let tree = {
        let (applied, out, sig) = (applied.clone(), out.clone(), sig.clone());
        ViewTree::new(move || {
            let width = signal(1.0f32);
            sig.set(Some(width));
            SizedBox {
                width: (move || width.get() * 10.0).into_prop(),
                applied,
                out,
            }
        })
    };
    Fixture {
        tree,
        width: sig.get().expect("created"),
        applied,
        element: out.get().expect("built"),
    }
}

fn frame(tree: &mut ViewTree) -> (FrameReport, Scene) {
    let mut scene = Scene::new();
    let report = tree.frame(
        BoxConstraints::loose(s(800.0, 600.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    (report, scene)
}

fn width_of(tree: &ViewTree, element: ElementId) -> Option<f32> {
    let layout = tree.layout_id(element)?;
    tree.layout_tree().size(layout).map(|s| s.width)
}

#[test]
fn view_frame_01_into_prop() {
    let rt_tree = ViewTree::new(|| Empty);
    rt_tree.enter(|| {
        let v: Prop<f32> = 2.5f32.into_prop();
        assert!(!v.is_dynamic());
        assert_eq!(v.get(), 2.5);
        let c: Prop<Color> = Color::WHITE.into_prop();
        assert_eq!(c.get(), Color::WHITE);
        let text: Prop<Arc<str>> = "hi".into_prop();
        assert_eq!(&*text.get(), "hi");
        let owned: Prop<String> = "hi".into_prop();
        assert_eq!(owned.get(), "hi");
        let d: Prop<i32> = (|| 7).into_prop();
        assert!(d.is_dynamic());
        assert_eq!(d.get(), 7);
        let sig = signal(3u32);
        let from_signal: Prop<u32> = sig.into_prop();
        assert!(from_signal.is_dynamic());
        sig.set(4);
        assert_eq!(from_signal.get(), 4);
        let m = memo(move || sig.get() * 2);
        let from_memo: Prop<u32> = m.into_prop();
        assert_eq!(from_memo.get(), 8);
        let same: Prop<bool> = Prop::Value(true).into_prop();
        assert!(same.get());
    });
}

/// An empty region.
struct Empty;

impl View for Empty {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|_| {})
    }
}

#[test]
fn view_frame_02_value_applied_once() {
    let applied: Rc<RefCell<Vec<f32>>> = Rc::default();
    let out = Rc::new(Cell::new(None));
    let mut tree = {
        let (applied, out) = (applied.clone(), out.clone());
        ViewTree::new(move || SizedBox {
            width: 42.0f32.into_prop(),
            applied,
            out,
        })
    };
    assert_eq!(*applied.borrow(), [42.0]);
    assert!(!tree.needs_frame());
    let (report, _) = frame(&mut tree);
    assert_eq!(report.applied, 0);
    assert_eq!(*applied.borrow(), [42.0]);
    assert_eq!(width_of(&tree, out.get().expect("built")), Some(42.0));
}

#[test]
fn view_frame_03_dynamic_deferred_and_coalesced() {
    let mut f = fixture();
    assert_eq!(*f.applied.borrow(), [10.0]);
    frame(&mut f.tree);
    assert_eq!(width_of(&f.tree, f.element), Some(10.0));
    // A change: not applied until the next frame.
    f.tree.enter(|| f.width.set(2.0));
    assert_eq!(*f.applied.borrow(), [10.0]);
    let (report, _) = frame(&mut f.tree);
    assert_eq!(report.applied, 1);
    assert_eq!(*f.applied.borrow(), [10.0, 20.0]);
    assert_eq!(width_of(&f.tree, f.element), Some(20.0));
    // Several changes: one application, with the latest value.
    f.tree.enter(|| {
        f.width.set(3.0);
        f.width.set(4.0);
        f.width.set(5.0);
    });
    let (report, _) = frame(&mut f.tree);
    assert_eq!(report.applied, 1);
    assert_eq!(*f.applied.borrow(), [10.0, 20.0, 50.0]);
    // Inside a batch too.
    f.tree.enter(|| {
        batch(|| {
            f.width.set(6.0);
            f.width.set(7.0);
        })
    });
    frame(&mut f.tree);
    assert_eq!(f.applied.borrow().last(), Some(&70.0));
}

#[test]
fn view_frame_04_needs_frame_and_requester() {
    let mut f = fixture();
    let requests = Rc::new(Cell::new(0));
    {
        let requests = requests.clone();
        f.tree
            .set_frame_requester(move || requests.set(requests.get() + 1));
    }
    assert!(!f.tree.needs_frame());
    f.tree.enter(|| f.width.set(2.0));
    assert!(f.tree.needs_frame());
    assert_eq!(requests.get(), 1);
    f.tree.enter(|| f.width.set(3.0));
    assert_eq!(requests.get(), 1);
    frame(&mut f.tree);
    assert!(!f.tree.needs_frame());
    f.tree.enter(|| f.width.set(4.0));
    assert_eq!(requests.get(), 2);
}

/// A full-size fill, to see the Scene repainted.
struct Fill;

impl Paint for Fill {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene().fill_rect(
            Rect::from_ltwh(0.0, 0.0, size.width, size.height),
            Color::BLACK,
        );
    }
}

struct Painted;

impl View for Painted {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(30.0), Some(20.0)), []);
        cx.set_paint(id, Fill);
        id
    }
}

#[test]
fn view_frame_05_frame_lays_out_and_paints() {
    let mut tree = ViewTree::new(|| Painted);
    let mut scene = Scene::new();
    let report = tree.frame(
        BoxConstraints::loose(s(800.0, 600.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    assert_eq!(report, FrameReport::default());
    assert_eq!(scene.size(), s(800.0, 600.0));
    let fills = scene
        .entries()
        .iter()
        .filter(|e| matches!(e.command, Command::Fill { .. }))
        .count();
    assert_eq!(fills, 1);
    // Painting again starts from scratch (the Scene isn't appended to).
    let report = tree.frame(
        BoxConstraints::loose(s(800.0, 600.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
    assert_eq!(report.applied, 0);
    let fills = scene
        .entries()
        .iter()
        .filter(|e| matches!(e.command, Command::Fill { .. }))
        .count();
    assert_eq!(fills, 1);
}

/// A box with a recolorable paint, to test `ElementMut`.
struct Tint(Color);

impl Paint for Tint {}

struct Probe(Rc<RefCell<Vec<String>>>, Signal<u32>);

impl View for Probe {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(5.0), Some(5.0)), []);
        cx.set_paint(id, Tint(Color::BLACK));
        let log = self.0;
        let sig = self.1;
        cx.bind(id, (move || sig.get()).into_prop(), move |el: &mut ElementMut<'_>, v| {
            let same = el.update_render(|r: &mut RenderConstrainedBox| {
                r.additional.max_width = 5.0 + (v / 2) as f32;
                r.additional.min_width = 5.0 + (v / 2) as f32;
            });
            let wrong_type = el.update_render(|_: &mut RenderFlex| {});
            let recolored = el.update_paint(|p: &mut Tint| p.0 = Color::WHITE);
            let wrong_paint = el.update_paint(|_: &mut Fill| {});
            log.borrow_mut().push(format!(
                "{v}: changed {same}, flex {wrong_type}, tint {recolored}, fill {wrong_paint}, render {}, paint {}",
                el.render::<RenderConstrainedBox>().is_some(),
                el.paint::<Tint>().map(|t| t.0 == Color::WHITE).unwrap_or(false),
            ));
            assert_eq!(el.id(), el.id());
        });
        id
    }
}

#[test]
fn view_frame_06_element_mut() {
    let log: Rc<RefCell<Vec<String>>> = Rc::default();
    let sig: Rc<Cell<Option<Signal<u32>>>> = Rc::default();
    let mut tree = {
        let (log, sig) = (log.clone(), sig.clone());
        ViewTree::new(move || {
            let s = signal(0u32);
            sig.set(Some(s));
            Probe(log, s)
        })
    };
    let s = sig.get().expect("created");
    frame(&mut tree);
    // 0 → 1: width 5 + 0 both times, so no change.
    tree.enter(|| s.set(1));
    frame(&mut tree);
    // 1 → 2: width 6, a change.
    tree.enter(|| s.set(2));
    frame(&mut tree);
    assert_eq!(
        *log.borrow(),
        [
            "0: changed false, flex false, tint true, fill false, render true, paint true",
            "1: changed false, flex false, tint true, fill false, render true, paint true",
            "2: changed true, flex false, tint true, fill false, render true, paint true",
        ]
    );
}

/// A region holding a SizedBox, so the box can be removed alone.
struct Holder(SizedBox);

impl View for Holder {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderFlex::column(), [AnyView::new(self.0)])
    }
}

#[test]
fn view_frame_07_removed_elements_stop() {
    let applied: Rc<RefCell<Vec<f32>>> = Rc::default();
    let out = Rc::new(Cell::new(None));
    let sig: Rc<Cell<Option<Signal<f32>>>> = Rc::default();
    let mut tree = {
        let (applied, out, sig) = (applied.clone(), out.clone(), sig.clone());
        ViewTree::new(move || {
            let w = signal(1.0f32);
            sig.set(Some(w));
            Holder(SizedBox {
                width: (move || w.get()).into_prop(),
                applied,
                out,
            })
        })
    };
    let w = sig.get().expect("created");
    let element = out.get().expect("built");
    tree.enter(|| w.set(2.0));
    assert!(tree.needs_frame());
    assert_eq!(tree.remove(element), 1);
    let (report, _) = frame(&mut tree);
    assert_eq!(report.applied, 0);
    tree.enter(|| w.set(3.0));
    frame(&mut tree);
    assert_eq!(*applied.borrow(), [1.0]);
}

/// Applies a value by writing another signal (which another binding reads).
struct Chain(Signal<u32>, Signal<u32>, Rc<RefCell<Vec<String>>>);

impl View for Chain {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(1.0), Some(1.0)), []);
        let (a, b, log) = (self.0, self.1, self.2);
        let log2 = log.clone();
        cx.bind(id, (move || a.get()).into_prop(), move |_, v| {
            log.borrow_mut().push(format!("a {v}"));
            b.set(v * 10);
        });
        cx.bind(id, (move || b.get()).into_prop(), move |_, v| {
            log2.borrow_mut().push(format!("b {v}"));
        });
        // A closure reading nothing.
        let constant: Prop<u32> = (|| 1).into_prop();
        let log3 = Rc::new(Cell::new(0));
        let l3 = log3.clone();
        cx.bind(id, constant, move |_, _| l3.set(l3.get() + 1));
        assert_eq!(log3.get(), 1);
        id
    }
}

/// Two signals handed out of a tree's build.
type SignalPair = (Signal<u32>, Signal<u32>);

#[test]
fn view_frame_08_robustness() {
    // Unbounded constraints: the Scene takes the root's size.
    let mut tree = ViewTree::new(|| Painted);
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::UNCONSTRAINED,
        &mut NoTextMeasure,
        &mut scene,
    );
    assert_eq!(scene.size(), s(30.0, 20.0));
    // apply may write signals: their values are applied in the next frame.
    let log: Rc<RefCell<Vec<String>>> = Rc::default();
    let sigs: Rc<Cell<Option<SignalPair>>> = Rc::default();
    let mut tree = {
        let (log, sigs) = (log.clone(), sigs.clone());
        ViewTree::new(move || {
            let (a, b) = (signal(1u32), signal(0u32));
            sigs.set(Some((a, b)));
            Chain(a, b, log)
        })
    };
    let (a, _) = sigs.get().expect("created");
    // Build: bind applies initial values at once, so a 1 sets b to 10 before b is bound,
    // and b starts at 10. Nothing is left for the first frame.
    assert_eq!(*log.borrow(), ["a 1", "b 10"]);
    let (report, _) = frame(&mut tree);
    assert_eq!(report.applied, 0);
    tree.enter(|| a.set(2));
    let (report, _) = frame(&mut tree);
    assert_eq!(report.applied, 1);
    assert!(tree.needs_frame());
    let (report, _) = frame(&mut tree);
    assert_eq!(report.applied, 1);
    assert_eq!(log.borrow().last().map(String::as_str), Some("b 20"));
}
