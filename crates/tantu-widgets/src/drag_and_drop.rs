//! [`Draggable`] and [`DragTarget`]: drag and drop inside the app. Spec:
//! `docs/specs/widgets/drag-and-drop.md` (WIDGETS-DND-01..02).

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use tantu_view::core::Point;
use tantu_view::layout::{Alignment, BoxConstraints, RenderConstrainedBox};
use tantu_view::reactive::{Signal, effect, signal, untrack};
use tantu_view::{
    Anchor, AnyView, BuildCx, DragEventKind, DragOutcome, ElementId, Handled, OverlayPointer,
    OverlayPortal, Phase, PointerButton, PointerKind, Show, View,
};

use crate::{Offstage, Stack};

/// How far the pointer moves with the button held before a drag starts (WIDGETS-DND-01).
const SLOP: f32 = 4.0;

/// Decides whether a target accepts hovering data.
type WillAccept<T> = Rc<dyn Fn(&T) -> bool>;

/// Builds a view on demand.
type ViewFn = Rc<dyn Fn() -> AnyView>;

/// A child the user can drag, carrying `data` (Flutter's `Draggable`, WIDGETS-DND-01).
pub struct Draggable<T> {
    data: T,
    child: AnyView,
    feedback: Option<ViewFn>,
    child_when_dragging: Option<ViewFn>,
    on_drag_started: Option<Rc<dyn Fn()>>,
    on_drag_end: Option<Rc<dyn Fn(bool)>>,
}

impl<T: Clone + 'static> Draggable<T> {
    /// `child`, draggable, carrying `data`.
    pub fn new(data: T, child: impl View) -> Self {
        Draggable {
            data,
            child: AnyView::new(child),
            feedback: None,
            child_when_dragging: None,
            on_drag_started: None,
            on_drag_end: None,
        }
    }

    /// What follows the pointer while dragging (default: nothing).
    pub fn feedback<V: View>(mut self, feedback: impl Fn() -> V + 'static) -> Self {
        self.feedback = Some(Rc::new(move || AnyView::new(feedback())));
        self
    }

    /// What stays in the child's place while dragging (default: the child).
    pub fn child_when_dragging<V: View>(mut self, view: impl Fn() -> V + 'static) -> Self {
        self.child_when_dragging = Some(Rc::new(move || AnyView::new(view())));
        self
    }

    /// Called when a drag starts.
    pub fn on_drag_started(mut self, f: impl Fn() + 'static) -> Self {
        self.on_drag_started = Some(Rc::new(f));
        self
    }

    /// Called when the drag ends, with whether a target accepted the data.
    pub fn on_drag_end(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_drag_end = Some(Rc::new(f));
        self
    }
}

impl<T: Clone + 'static> View for Draggable<T> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let handle = cx.drag();
        // Whether this draggable's drag is the active session.
        let dragging = signal(false);
        // The pointer and the press, in window coordinates.
        let pointer = signal(Point::ZERO);
        let press: Rc<Cell<Option<Point>>> = Rc::default();
        let moved = Rc::new(Cell::new(false));
        // The session ending (a release or Escape) ends this drag.
        {
            let (handle, on_drag_end) = (handle.clone(), self.on_drag_end.clone());
            effect(move || {
                let active = handle.is_active();
                if !active && untrack(|| dragging.get()) {
                    dragging.set(false);
                    if let Some(f) = &on_drag_end {
                        f(handle.outcome() == Some(DragOutcome::Accepted));
                    }
                }
            });
        }
        let ghost = self.child_when_dragging;
        let has_ghost = ghost.is_some();
        let mut body = Stack::new().child(
            Offstage::new(move || has_ghost && dragging.get()).child(AnyView::new(self.child)),
        );
        if let Some(ghost) = ghost {
            body = body.child(Show::new(move || dragging.get(), move || ghost()));
        }
        let id = match self.feedback {
            Some(feedback) => {
                let (press, handle) = (Rc::clone(&press), handle.clone());
                OverlayPortal::new(body)
                    .visible(move || dragging.get())
                    .pointer(OverlayPointer::Passthrough)
                    .anchor(move || {
                        // The feedback keeps the press's offset from the pointer.
                        // The session's position: this element's capture may have ended
                        // (VIEW-DRAG-05).
                        let from = press.get().unwrap_or(Point::ZERO);
                        let at = if dragging.get() {
                            handle.position()
                        } else {
                            pointer.get()
                        };
                        Anchor::new(Alignment::TOP_LEFT, Alignment::TOP_LEFT)
                            .offset(at.x - from.x, at.y - from.y)
                    })
                    .overlay(move || feedback())
                    .build(cx)
            }
            None => body.build(cx),
        };
        let (data, on_drag_started) = (self.data, self.on_drag_started);
        cx.on_pointer(id, Phase::Capture, move |p| {
            let at = p.event.position;
            match p.event.kind {
                PointerKind::Down(PointerButton::Primary) => {
                    press.set(Some(at));
                    moved.set(false);
                    Handled::Continue
                }
                PointerKind::Move => {
                    let Some(from) = press.get() else {
                        return Handled::Continue;
                    };
                    if !dragging.get_untracked() {
                        let far = (at.x - from.x).hypot(at.y - from.y) > SLOP;
                        if !far || !handle.start(Rc::new(data.clone())) {
                            return Handled::Continue;
                        }
                        moved.set(true);
                        pointer.set(at);
                        dragging.set(true);
                        if let Some(f) = &on_drag_started {
                            f();
                        }
                        return Handled::Stop;
                    }
                    pointer.set(at);
                    Handled::Stop
                }
                PointerKind::Up(PointerButton::Primary) => {
                    press.set(None);
                    if moved.replace(false) {
                        Handled::Stop
                    } else {
                        Handled::Continue
                    }
                }
                _ => Handled::Continue,
            }
        });
        id
    }
}

/// A region that may accept dragged data of type `T` (Flutter's `DragTarget`,
/// WIDGETS-DND-02).
pub struct DragTarget<T> {
    child: AnyView,
    will_accept: Option<WillAccept<T>>,
    on_accept: Option<Rc<dyn Fn(T)>>,
    candidate: Option<Signal<bool>>,
    _data: PhantomData<T>,
}

impl<T: Clone + 'static> DragTarget<T> {
    /// `child` as a drop target.
    pub fn new(child: impl View) -> Self {
        DragTarget {
            child: AnyView::new(child),
            will_accept: None,
            on_accept: None,
            candidate: None,
            _data: PhantomData,
        }
    }

    /// Whether to accept the data hovering over it (default: always).
    pub fn on_will_accept(mut self, f: impl Fn(&T) -> bool + 'static) -> Self {
        self.will_accept = Some(Rc::new(f));
        self
    }

    /// Called with the data dropped on it.
    pub fn on_accept(mut self, f: impl Fn(T) + 'static) -> Self {
        self.on_accept = Some(Rc::new(f));
        self
    }

    /// Set to true while acceptable data hovers over it, false otherwise.
    pub fn candidate(mut self, signal: Signal<bool>) -> Self {
        self.candidate = Some(signal);
        self
    }
}

impl<T: Clone + 'static> View for DragTarget<T> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        // An element of its own to receive drag events, sized by the child.
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [self.child],
        );
        let candidate = self.candidate.unwrap_or_else(|| signal(false));
        let set = move |on: bool| {
            if candidate.get_untracked() != on {
                candidate.set(on);
            }
        };
        let (will_accept, on_accept) = (self.will_accept, self.on_accept);
        cx.on_drag(id, move |d| {
            let data = d.payload.downcast_ref::<T>();
            match d.kind {
                DragEventKind::Enter => {
                    let accepts =
                        data.is_some_and(|data| will_accept.as_ref().is_none_or(|f| f(data)));
                    set(accepts);
                    accepts
                }
                DragEventKind::Over => false,
                DragEventKind::Leave => {
                    set(false);
                    false
                }
                DragEventKind::Drop => {
                    set(false);
                    match (data, &on_accept) {
                        (Some(data), Some(f)) => {
                            f(data.clone());
                            true
                        }
                        (Some(_), None) => true,
                        (None, _) => false,
                    }
                }
            }
        });
        id
    }
}
