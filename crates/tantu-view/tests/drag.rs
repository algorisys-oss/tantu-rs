//! Tests for `docs/specs/widgets/drag-and-drop.md`, rules VIEW-DRAG-01..04.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_core::{Point, Size};
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_reactive::{Signal, signal};
use tantu_scene::Scene;
use tantu_view::{
    AnyView, BuildCx, DragEventKind, DragHandle, DragOutcome, ElementId, Handled, KeyEvent,
    LogicalKey, Modifiers, NamedKey, Phase, PointerButton, PointerEvent, PointerKind, Show, View,
    ViewTree,
};

type Log = Rc<RefCell<Vec<String>>>;

/// Registers handlers on a built element.
type Setup = Box<dyn FnOnce(&mut BuildCx<'_>, ElementId)>;

/// A 50 × 50 box set up by `setup`.
struct Square {
    setup: Setup,
}

impl View for Square {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(50.0), Some(50.0)), []);
        (self.setup)(cx, id);
        id
    }
}

/// A row of children with 50 px between them.
struct Row(Vec<AnyView>);

impl View for Row {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let mut flex = RenderFlex::row();
        flex.spacing = 50.0;
        flex.cross_axis_alignment = tantu_layout::CrossAxisAlignment::Start;
        cx.render(flex, self.0)
    }
}

/// A target logging its events as `"{name}:{kind}"`, accepting on `Enter` when `accepts`
/// and taking drops when `takes`.
fn target(log: &Log, name: &'static str, accepts: bool, takes: bool) -> Square {
    let log = log.clone();
    Square {
        setup: Box::new(move |cx, id| {
            cx.on_drag(id, move |d| {
                let payload = d.payload.downcast_ref::<i32>().copied();
                log.borrow_mut()
                    .push(format!("{name}:{:?}:{payload:?}", d.kind));
                match d.kind {
                    DragEventKind::Enter => accepts,
                    DragEventKind::Drop => takes,
                    _ => false,
                }
            });
        }),
    }
}

struct Fx {
    tree: ViewTree,
    handle: DragHandle,
    log: Log,
    shown: Signal<bool>,
}

/// A source at x 0..50 that starts a drag carrying 7 on its first move and records the
/// outcome it sees on release; target A (accepts and takes) at 100..150; B (refuses) at
/// 200..250; C (accepts, declines the drop) at 300..350, shown while `shown`.
fn fixture() -> Fx {
    let log: Log = Rc::default();
    type Out = Rc<RefCell<Option<(DragHandle, Signal<bool>)>>>;
    let out: Out = Rc::default();
    let mut tree = {
        let (log, out) = (log.clone(), out.clone());
        ViewTree::new(move || {
            let shown = signal(true);
            let source_log = log.clone();
            let handle_out = out.clone();
            let c = {
                let log = log.clone();
                move || target(&log, "C", true, false)
            };
            Row(vec![
                AnyView::new(Square {
                    setup: Box::new(move |cx, id| {
                        let handle = cx.drag();
                        *handle_out.borrow_mut() = Some((handle.clone(), shown));
                        cx.on_pointer(id, Phase::Bubble, move |p| {
                            match p.event.kind {
                                PointerKind::Move if !handle.is_active() => {
                                    handle.start(Rc::new(7i32));
                                }
                                PointerKind::Up(_) => source_log
                                    .borrow_mut()
                                    .push(format!("source:up:{:?}", handle.outcome())),
                                _ => {}
                            }
                            Handled::Stop
                        });
                    }),
                }),
                AnyView::new(target(&log, "A", true, true)),
                AnyView::new(target(&log, "B", false, false)),
                AnyView::new(Show::new(move || shown.get(), c)),
            ])
        })
    };
    frame(&mut tree);
    let (handle, shown) = out.borrow_mut().take().expect("built");
    Fx {
        tree,
        handle,
        log,
        shown,
    }
}

fn frame(tree: &mut ViewTree) {
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::loose(Size::new(500.0, 100.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
}

impl Fx {
    fn pointer(&mut self, kind: PointerKind, x: f32) {
        self.tree.dispatch_pointer(PointerEvent {
            kind,
            position: Point::new(x, 25.0),
            modifiers: Modifiers::default(),
        });
        frame(&mut self.tree);
    }

    fn down(&mut self, x: f32) {
        self.pointer(PointerKind::Down(PointerButton::Primary), x);
    }

    fn mv(&mut self, x: f32) {
        self.pointer(PointerKind::Move, x);
    }

    fn up(&mut self, x: f32) {
        self.pointer(PointerKind::Up(PointerButton::Primary), x);
    }

    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.log.borrow_mut())
    }

    fn active(&self) -> bool {
        let handle = self.handle.clone();
        self.tree.enter(|| handle.is_active())
    }

    fn outcome(&self) -> Option<DragOutcome> {
        self.handle.outcome()
    }
}

#[test]
fn view_drag_01_sessions() {
    let fx = fixture();
    assert!(!fx.active());
    assert_eq!(fx.outcome(), None);
    let handle = fx.handle.clone();
    fx.tree.enter(|| {
        assert!(handle.start(Rc::new(1i32)));
        assert!(handle.is_active());
        assert!(!handle.start(Rc::new(2i32)), "one at a time");
    });
    assert_eq!(fx.outcome(), None);
}

#[test]
fn view_drag_02_enter_over_leave() {
    let mut fx = fixture();
    fx.down(25.0);
    fx.mv(30.0);
    assert!(fx.active());
    assert_eq!(
        fx.take(),
        Vec::<String>::new(),
        "no target under the source"
    );
    fx.mv(110.0);
    fx.mv(120.0);
    assert_eq!(
        fx.take(),
        ["A:Enter:Some(7)", "A:Over:Some(7)", "A:Over:Some(7)"]
    );
    fx.mv(210.0);
    assert_eq!(
        fx.take(),
        ["A:Leave:Some(7)", "B:Enter:Some(7)", "B:Over:Some(7)"]
    );
    fx.mv(175.0);
    assert_eq!(fx.take(), ["B:Leave:Some(7)"]);
}

#[test]
fn view_drag_03_drop_and_cancel() {
    // Released over A: dropped, accepted, known to the source on its release.
    let mut fx = fixture();
    fx.down(25.0);
    fx.mv(30.0);
    fx.mv(120.0);
    fx.take();
    fx.up(120.0);
    assert_eq!(fx.take(), ["A:Drop:Some(7)", "source:up:Some(Accepted)"]);
    assert!(!fx.active());
    assert_eq!(fx.outcome(), Some(DragOutcome::Accepted));
    // Over B (refusing): no drop, rejected.
    fx.down(25.0);
    fx.mv(30.0);
    fx.mv(220.0);
    fx.take();
    fx.up(220.0);
    assert_eq!(fx.take(), ["source:up:Some(Rejected)"]);
    // Over C (declining the drop): rejected.
    fx.down(25.0);
    fx.mv(30.0);
    fx.mv(320.0);
    fx.take();
    fx.up(320.0);
    assert_eq!(fx.take(), ["C:Drop:Some(7)", "source:up:Some(Rejected)"]);
    // Escape cancels and is consumed; leaving the window doesn't end the session.
    fx.down(25.0);
    fx.mv(30.0);
    fx.mv(120.0);
    fx.pointer(PointerKind::Leave, 120.0);
    assert!(fx.active());
    fx.take();
    assert!(fx.tree.dispatch_key(KeyEvent::press(
        LogicalKey::Named(NamedKey::Escape),
        Modifiers::default()
    )));
    assert_eq!(fx.take(), ["A:Leave:Some(7)"]);
    assert!(!fx.active());
    assert_eq!(fx.outcome(), Some(DragOutcome::Cancelled));
}

#[test]
fn view_drag_04_removed_target() {
    let mut fx = fixture();
    fx.down(25.0);
    fx.mv(30.0);
    fx.mv(320.0);
    assert_eq!(fx.take(), ["C:Enter:Some(7)", "C:Over:Some(7)"]);
    let shown = fx.shown;
    fx.tree.enter(|| shown.set(false));
    frame(&mut fx.tree);
    fx.mv(330.0);
    fx.up(330.0);
    assert_eq!(
        fx.take(),
        ["source:up:Some(Rejected)"],
        "no events to the removed target"
    );
}

#[test]
fn view_drag_05_position() {
    let mut fx = fixture();
    fx.down(25.0);
    fx.mv(30.0);
    fx.mv(140.0);
    let handle = fx.handle.clone();
    assert_eq!(fx.tree.enter(|| handle.position()), Point::new(140.0, 25.0));
    // Moves are followed even when nothing captures them.
    fx.tree.dispatch_pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(400.0, 60.0),
        modifiers: Modifiers::default(),
    });
    assert_eq!(fx.tree.enter(|| handle.position()), Point::new(400.0, 60.0));
}
