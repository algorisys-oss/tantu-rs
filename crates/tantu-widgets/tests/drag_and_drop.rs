//! Tests for `docs/specs/widgets/drag-and-drop.md`, rules WIDGETS-DND-01..02.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::Point;
use tantu_view::layout::CrossAxisAlignment;
use tantu_view::reactive::{Signal, signal};
use tantu_view::{
    Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
};
use tantu_widgets::{Button, Column, DragTarget, Draggable, Row, SizedBox};

type Log = Rc<RefCell<Vec<String>>>;

struct Fx {
    t: WidgetTester,
    log: Log,
    candidate: Signal<bool>,
}

/// A 40 × 40 chip carrying 5 at the top left, with a 30 × 30 feedback and a ghost, and a
/// 60 × 60 bin of `T` at x 140..200 accepting when `accept` does.
fn fixture<T: Clone + std::fmt::Debug + 'static>(accept: fn(&T) -> bool) -> Fx {
    let log: Log = Rc::default();
    let out: Rc<Cell<Option<Signal<bool>>>> = Rc::default();
    let mut t = {
        let (log, out) = (log.clone(), out.clone());
        WidgetTester::with_size(400.0, 200.0, move || {
            let candidate = signal(false);
            out.set(Some(candidate));
            let (started, ended, accepted) = (log.clone(), log.clone(), log.clone());
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .child(
                    Row::new()
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .spacing(100.0)
                        .child(
                            Draggable::new(5u32, Keyed::new("chip", SizedBox::new(40.0, 40.0)))
                                .feedback(|| Keyed::new("feedback", SizedBox::new(30.0, 30.0)))
                                .child_when_dragging(|| {
                                    Keyed::new("ghost", SizedBox::new(40.0, 40.0))
                                })
                                .on_drag_started(move || {
                                    started.borrow_mut().push("started".to_owned())
                                })
                                .on_drag_end(move |ok| {
                                    ended.borrow_mut().push(format!("ended:{ok}"))
                                }),
                        )
                        .child(
                            DragTarget::<T>::new(Keyed::new("bin", SizedBox::new(60.0, 60.0)))
                                .on_will_accept(accept)
                                .on_accept(move |data| {
                                    accepted.borrow_mut().push(format!("accepted:{data:?}"))
                                })
                                .candidate(candidate),
                        ),
                )
        })
    };
    t.pump();
    let candidate = out.get().expect("built");
    Fx { t, log, candidate }
}

impl Fx {
    fn pointer(&mut self, kind: PointerKind, x: f32, y: f32) {
        self.t.pointer(PointerEvent {
            kind,
            position: Point::new(x, y),
            modifiers: Modifiers::default(),
        });
        self.t.pump();
        self.t.pump();
    }

    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.log.borrow_mut())
    }

    fn has(&self, key: &str) -> bool {
        !self.t.find_all(&Finder::key(key)).is_empty()
    }

    fn candidate(&mut self) -> bool {
        let candidate = self.candidate;
        self.t.enter(|| candidate.get_untracked())
    }
}

#[test]
fn widgets_dnd_01_draggable() {
    let mut fx = fixture::<u32>(|_| true);
    // Within the 4 px slop nothing starts.
    fx.pointer(PointerKind::Down(PointerButton::Primary), 10.0, 15.0);
    fx.pointer(PointerKind::Move, 13.0, 16.0);
    assert!(fx.take().is_empty());
    assert!(!fx.has("feedback"));
    // Past it: started, the ghost in place, the feedback at the press's offset.
    fx.pointer(PointerKind::Move, 160.0, 25.0);
    assert_eq!(fx.take(), ["started"]);
    assert!(fx.has("ghost") && !fx.has("chip"));
    let feedback = fx.t.rect(fx.t.find(&Finder::key("feedback")));
    assert_eq!((feedback.left, feedback.top), (150.0, 10.0));
    assert!(fx.candidate());
    fx.pointer(PointerKind::Move, 170.0, 30.0);
    let feedback = fx.t.rect(fx.t.find(&Finder::key("feedback")));
    assert_eq!((feedback.left, feedback.top), (160.0, 15.0));
    // Dropped on the bin.
    fx.pointer(PointerKind::Up(PointerButton::Primary), 170.0, 30.0);
    assert_eq!(fx.take(), ["accepted:5", "ended:true"]);
    assert!(!fx.has("feedback") && !fx.has("ghost") && fx.has("chip"));
    assert!(!fx.candidate());
    // Released elsewhere: not accepted. Escape cancels too.
    fx.pointer(PointerKind::Down(PointerButton::Primary), 10.0, 15.0);
    fx.pointer(PointerKind::Move, 300.0, 150.0);
    fx.pointer(PointerKind::Up(PointerButton::Primary), 300.0, 150.0);
    assert_eq!(fx.take(), ["started", "ended:false"]);
    fx.pointer(PointerKind::Down(PointerButton::Primary), 10.0, 15.0);
    fx.pointer(PointerKind::Move, 160.0, 25.0);
    fx.t.press_key(LogicalKey::Named(NamedKey::Escape), Modifiers::default());
    fx.t.pump();
    fx.t.pump();
    assert_eq!(fx.take(), ["started", "ended:false"]);
    assert!(!fx.has("feedback") && fx.has("chip"));
    assert!(!fx.candidate());

    // A press and release without moving reaches a button inside.
    let presses = Rc::new(Cell::new(0));
    let mut t = {
        let presses = presses.clone();
        WidgetTester::with_size(300.0, 100.0, move || {
            let presses = presses.clone();
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .child(Draggable::new(
                    1u32,
                    Keyed::new(
                        "button",
                        Button::new("Press").on_press(move || presses.set(presses.get() + 1)),
                    ),
                ))
        })
    };
    t.pump();
    t.tap(&Finder::key("button"));
    assert_eq!(presses.get(), 1);
}

#[test]
fn widgets_dnd_02_drag_target() {
    // A bin of another type ignores the drag.
    let mut fx = fixture::<String>(|_| true);
    fx.pointer(PointerKind::Down(PointerButton::Primary), 10.0, 15.0);
    fx.pointer(PointerKind::Move, 160.0, 25.0);
    assert!(!fx.candidate());
    fx.pointer(PointerKind::Up(PointerButton::Primary), 160.0, 25.0);
    assert_eq!(fx.take(), ["started", "ended:false"]);
    // A bin whose will-accept refuses.
    let mut fx = fixture::<u32>(|n| *n > 10);
    fx.pointer(PointerKind::Down(PointerButton::Primary), 10.0, 15.0);
    fx.pointer(PointerKind::Move, 160.0, 25.0);
    assert!(!fx.candidate());
    fx.pointer(PointerKind::Up(PointerButton::Primary), 160.0, 25.0);
    assert_eq!(fx.take(), ["started", "ended:false"]);
}
