//! Tests for `docs/specs/view/events.md`, rules VIEW-EVENT-01..06.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_core::{Point, Size};
use tantu_layout::{
    BoxConstraints, CrossAxisAlignment, NoTextMeasure, RenderConstrainedBox, RenderFlex,
    RenderStack,
};
use tantu_reactive::{Signal, signal};
use tantu_view::{
    AnyView, BuildCx, CursorIcon, ElementId, Handled, Phase, PointerButton, PointerCx,
    PointerEvent, PointerKind, View, ViewTree,
};

type Log = Rc<RefCell<Vec<String>>>;
type Ids = Rc<RefCell<Vec<(&'static str, ElementId)>>>;

/// What an element registers.
#[derive(Clone, Copy, Default)]
struct Wiring {
    handlers: bool,
    stop_capture: bool,
    cursor: Option<CursorIcon>,
}

fn kind_name(kind: PointerKind) -> &'static str {
    match kind {
        PointerKind::Down(_) => "down",
        PointerKind::Up(_) => "up",
        PointerKind::Move => "move",
        PointerKind::Scroll(_) => "scroll",
        PointerKind::Enter => "enter",
        PointerKind::Leave => "leave",
    }
}

/// Registers handlers and a cursor on `id`, logging "name phase kind x,y".
fn wire(cx: &mut BuildCx<'_>, id: ElementId, name: &'static str, w: Wiring, log: &Log, ids: &Ids) {
    ids.borrow_mut().push((name, id));
    if w.handlers {
        for phase in [Phase::Capture, Phase::Bubble] {
            let log = log.clone();
            let stop = w.stop_capture && phase == Phase::Capture;
            cx.on_pointer(id, phase, move |p: &PointerCx<'_>| {
                let tag = if p.phase == Phase::Capture {
                    "capture"
                } else {
                    "bubble"
                };
                log.borrow_mut().push(format!(
                    "{name} {tag} {} {},{}",
                    kind_name(p.event.kind),
                    p.local.x,
                    p.local.y
                ));
                if stop {
                    Handled::Stop
                } else {
                    Handled::Continue
                }
            });
        }
    }
    if let Some(cursor) = w.cursor {
        cx.set_cursor(id, cursor);
    }
}

/// A box of `w × h`.
struct Boxed {
    name: &'static str,
    size: Size,
    wiring: Wiring,
    log: Log,
    ids: Ids,
}

impl View for Boxed {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::sized(Some(self.size.width), Some(self.size.height)),
            [],
        );
        wire(cx, id, self.name, self.wiring, &self.log, &self.ids);
        id
    }
}

/// A container: a column (cross axis at the start) or a stack.
struct Container {
    name: &'static str,
    stack: bool,
    children: Vec<AnyView>,
    wiring: Wiring,
    log: Log,
    ids: Ids,
}

impl View for Container {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = if self.stack {
            cx.render(RenderStack::new(), self.children)
        } else {
            let mut column = RenderFlex::column();
            column.cross_axis_alignment = CrossAxisAlignment::Start;
            cx.render(column, self.children)
        };
        wire(cx, id, self.name, self.wiring, &self.log, &self.ids);
        id
    }
}

struct Region(Vec<AnyView>);

impl View for Region {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|cx| {
            for child in self.0 {
                child.build(cx);
            }
        })
    }
}

/// The test tree, laid out in a 200 × 200 window:
///
/// ```text
/// col (column)                     (0, 0)   100 × 170
/// ├─ a     100 × 50                (0, 0)
/// ├─ b     100 × 50                (0, 50)
/// ├─ stack                          (0, 100)  60 × 60
/// │  ├─ c1  60 × 60                 (0, 100)
/// │  └─ c2  40 × 40                 (0, 100)  (on top)
/// └─ region
///    └─ d  30 × 10                  (0, 160)
/// ```
struct Fixture {
    tree: ViewTree,
    log: Log,
    ids: Ids,
}

impl Fixture {
    fn id(&self, name: &str) -> ElementId {
        self.ids
            .borrow()
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, id)| *id)
            .unwrap_or_else(|| panic!("no element {name}"))
    }

    fn send(&mut self, kind: PointerKind, x: f32, y: f32) -> bool {
        self.tree.dispatch_pointer(PointerEvent {
            kind,
            position: Point::new(x, y),
        })
    }

    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.log.borrow_mut())
    }
}

fn fixture(wirings: &[(&'static str, Wiring)]) -> Fixture {
    let (log, ids) = (Log::default(), Ids::default());
    let wiring = |name: &str| {
        wirings
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, w)| *w)
            .unwrap_or_default()
    };
    let boxed = |name, w: f32, h: f32| {
        AnyView::new(Boxed {
            name,
            size: Size::new(w, h),
            wiring: wiring(name),
            log: log.clone(),
            ids: ids.clone(),
        })
    };
    let stack = AnyView::new(Container {
        name: "stack",
        stack: true,
        children: vec![boxed("c1", 60.0, 60.0), boxed("c2", 40.0, 40.0)],
        wiring: wiring("stack"),
        log: log.clone(),
        ids: ids.clone(),
    });
    let col = Container {
        name: "col",
        stack: false,
        children: vec![
            boxed("a", 100.0, 50.0),
            boxed("b", 100.0, 50.0),
            stack,
            AnyView::new(Region(vec![boxed("d", 30.0, 10.0)])),
        ],
        wiring: wiring("col"),
        log: log.clone(),
        ids: ids.clone(),
    };
    let mut tree = ViewTree::new(move || col);
    tree.layout(
        BoxConstraints::loose(Size::new(200.0, 200.0)),
        &mut NoTextMeasure,
    );
    Fixture { tree, log, ids }
}

const ON: Wiring = Wiring {
    handlers: true,
    stop_capture: false,
    cursor: None,
};

#[test]
fn view_event_01_hit_test() {
    let f = fixture(&[]);
    let root = f.tree.root();
    let (col, b) = (f.id("col"), f.id("b"));
    assert_eq!(f.tree.hit_test(Point::new(30.0, 70.0)), [b, col, root]);
    // Overlapping siblings: the later one wins.
    let (stack, c1, c2) = (f.id("stack"), f.id("c1"), f.id("c2"));
    assert_eq!(
        f.tree.hit_test(Point::new(10.0, 110.0)),
        [c2, stack, col, root]
    );
    assert_eq!(
        f.tree.hit_test(Point::new(50.0, 150.0)),
        [c1, stack, col, root]
    );
    // Regions are transparent.
    assert_eq!(
        f.tree.hit_test(Point::new(5.0, 165.0)),
        [f.id("d"), col, root]
    );
    // In the column but on no child; outside the column.
    assert_eq!(f.tree.hit_test(Point::new(80.0, 150.0)), [col, root]);
    assert_eq!(f.tree.hit_test(Point::new(150.0, 150.0)), [root]);
    assert!(f.tree.hit_test(Point::new(-1.0, 5.0)).is_empty());
    // Never laid out: nothing is hit.
    let unlaid = ViewTree::new(|| Region(vec![]));
    assert!(unlaid.hit_test(Point::new(1.0, 1.0)).is_empty());
}

#[test]
fn view_event_02_capture_then_bubble() {
    let mut f = fixture(&[("col", ON), ("b", ON)]);
    assert!(f.send(PointerKind::Down(PointerButton::Primary), 30.0, 70.0));
    assert_eq!(
        f.take(),
        [
            "col capture down 30,70",
            "b capture down 30,20",
            "b bubble down 30,20",
            "col bubble down 30,70",
        ]
    );
    // Stop ends the dispatch.
    let mut f = fixture(&[
        ("col", ON),
        (
            "b",
            Wiring {
                stop_capture: true,
                ..ON
            },
        ),
    ]);
    f.send(
        PointerKind::Scroll(tantu_core::Vec2::new(0.0, 3.0)),
        30.0,
        70.0,
    );
    assert_eq!(
        f.take(),
        ["col capture scroll 30,70", "b capture scroll 30,20"]
    );
    // No handlers on the path: nothing ran.
    let mut f = fixture(&[("b", ON)]);
    assert!(!f.send(PointerKind::Up(PointerButton::Primary), 150.0, 150.0));
}

/// A box whose handler writes a signal.
struct Counter(Signal<u32>);

impl View for Counter {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(50.0), Some(50.0)), []);
        let count = self.0;
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if matches!(p.event.kind, PointerKind::Down(_)) {
                count.update(|c| *c += 1);
            }
            Handled::Continue
        });
        id
    }
}

#[test]
fn view_event_02_handlers_write_signals() {
    let sig: Rc<Cell<Option<Signal<u32>>>> = Rc::default();
    let mut tree = {
        let sig = sig.clone();
        ViewTree::new(move || {
            let count = signal(0u32);
            sig.set(Some(count));
            Counter(count)
        })
    };
    tree.layout(
        BoxConstraints::loose(Size::new(100.0, 100.0)),
        &mut NoTextMeasure,
    );
    let count = sig.get().expect("created");
    for _ in 0..3 {
        tree.dispatch_pointer(PointerEvent {
            kind: PointerKind::Down(PointerButton::Primary),
            position: Point::new(10.0, 10.0),
        });
    }
    assert_eq!(tree.enter(|| count.get()), 3);
}

#[test]
fn view_event_03_pointer_capture() {
    let mut f = fixture(&[("a", ON), ("b", ON)]);
    f.send(PointerKind::Down(PointerButton::Primary), 30.0, 70.0);
    f.take();
    // Over a, but b holds the pointer.
    f.send(PointerKind::Move, 30.0, 10.0);
    f.send(PointerKind::Up(PointerButton::Primary), 30.0, 10.0);
    let log = f.take();
    assert!(log.contains(&"b bubble move 30,-40".to_string()), "{log:?}");
    assert!(log.contains(&"b bubble up 30,-40".to_string()), "{log:?}");
    assert!(
        log.iter()
            .all(|e| !e.starts_with("a ") || e.contains("enter")),
        "{log:?}"
    );
    // Released: a gets the next move.
    f.send(PointerKind::Move, 30.0, 10.0);
    assert!(f.take().iter().any(|e| e == "a bubble move 30,10"));
}

#[test]
fn view_event_04_hover() {
    let mut f = fixture(&[("col", ON), ("a", ON), ("b", ON)]);
    let hover = |f: &Fixture| -> Vec<String> {
        f.take()
            .into_iter()
            .filter(|e| e.contains("enter") || e.contains("leave"))
            .collect()
    };
    f.send(PointerKind::Move, 30.0, 10.0);
    assert_eq!(
        hover(&f),
        ["col bubble enter 30,10", "a bubble enter 30,10"]
    );
    f.send(PointerKind::Move, 30.0, 60.0);
    assert_eq!(hover(&f), ["a bubble leave 30,60", "b bubble enter 30,10"]);
    f.send(PointerKind::Leave, 30.0, 60.0);
    assert_eq!(
        hover(&f),
        ["b bubble leave 30,10", "col bubble leave 30,60"]
    );
}

#[test]
fn view_event_05_cursor() {
    let mut f = fixture(&[
        (
            "col",
            Wiring {
                cursor: Some(CursorIcon::Grab),
                ..Wiring::default()
            },
        ),
        (
            "b",
            Wiring {
                cursor: Some(CursorIcon::Text),
                ..Wiring::default()
            },
        ),
    ]);
    assert_eq!(f.tree.cursor(), CursorIcon::Default);
    f.send(PointerKind::Move, 30.0, 70.0);
    assert_eq!(f.tree.cursor(), CursorIcon::Text);
    f.send(PointerKind::Move, 30.0, 10.0);
    assert_eq!(f.tree.cursor(), CursorIcon::Grab);
    f.send(PointerKind::Move, 150.0, 150.0);
    assert_eq!(f.tree.cursor(), CursorIcon::Default);
}

#[test]
fn view_event_06_removal() {
    let mut f = fixture(&[("a", ON), ("b", ON)]);
    let b = f.id("b");
    // b holds the pointer, then goes away: the capture is released.
    f.send(PointerKind::Down(PointerButton::Primary), 30.0, 70.0);
    f.tree.remove(b);
    f.take();
    f.send(PointerKind::Move, 30.0, 10.0);
    let log = f.take();
    assert!(log.iter().all(|e| !e.starts_with("b ")), "{log:?}");
    assert!(log.iter().any(|e| e.starts_with("a ")), "{log:?}");
    // Odd events don't panic.
    for (x, y) in [(f32::NAN, 0.0), (f32::INFINITY, 1e30), (-1e30, -5.0)] {
        f.send(PointerKind::Move, x, y);
        f.send(PointerKind::Up(PointerButton::Other(9)), x, y);
    }
}

#[test]
fn view_event_07_handler_sees_the_size() {
    let sizes: Rc<RefCell<Vec<Size>>> = Rc::default();
    struct Sized(Rc<RefCell<Vec<Size>>>);
    impl View for Sized {
        fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
            let id = cx.render(RenderConstrainedBox::sized(Some(100.0), Some(50.0)), []);
            let sizes = self.0;
            cx.on_pointer(id, Phase::Bubble, move |p| {
                sizes.borrow_mut().push(p.size);
                Handled::Continue
            });
            id
        }
    }
    let mut tree = {
        let sizes = sizes.clone();
        ViewTree::new(move || Sized(sizes))
    };
    tree.layout(
        BoxConstraints::loose(Size::new(200.0, 200.0)),
        &mut NoTextMeasure,
    );
    tree.dispatch_pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: Point::new(10.0, 10.0),
    });
    assert_eq!(*sizes.borrow(), [Size::new(100.0, 50.0)]);
}
