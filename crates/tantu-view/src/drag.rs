//! Drag sessions: a source starts one with a payload and the tree delivers drag events to
//! targets under the pointer. Spec: `docs/specs/widgets/drag-and-drop.md` (VIEW-DRAG-01..04).

use std::any::Any;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use tantu_core::Point;
use tantu_reactive::Signal;

use crate::{BuildCx, ElementId, PointerButton, PointerEvent, PointerKind, ViewTree};

/// A target's drag handler.
pub(crate) type DragHandler = Rc<dyn Fn(&DragCx<'_>) -> bool>;

/// The tree's drag session, shared with its handles (VIEW-DRAG-01).
#[derive(Default)]
pub(crate) struct DragSlot {
    payload: RefCell<Option<DragPayload>>,
    outcome: Cell<Option<DragOutcome>>,
    /// Whether a session is active, for reactive readers (created with the tree).
    pub(crate) active: OnceCell<Signal<bool>>,
    /// The pointer during a session (VIEW-DRAG-05; created with the tree).
    pub(crate) position: OnceCell<Signal<Point>>,
}

impl DragSlot {
    fn set_active(&self, on: bool) {
        if let Some(active) = self.active.get() {
            if active.get_untracked() != on {
                active.set(on);
            }
        }
    }

    /// Ends the session with `outcome`.
    fn end(&self, outcome: DragOutcome) {
        self.payload.borrow_mut().take();
        self.outcome.set(Some(outcome));
        self.set_active(false);
    }

    fn payload(&self) -> Option<DragPayload> {
        self.payload.borrow().clone()
    }
}

/// A drag's data, of any type.
pub type DragPayload = Rc<dyn Any>;

/// How a drag ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragOutcome {
    /// A target took the payload.
    Accepted,
    /// The pointer was released over no accepting target, or the target declined the drop.
    Rejected,
    /// Escape ended it.
    Cancelled,
}

/// The tree's drag session, for drag sources (cloneable; from [`BuildCx::drag`]).
#[derive(Clone)]
pub struct DragHandle {
    slot: Rc<DragSlot>,
}

impl DragHandle {
    /// Starts a drag carrying `payload`; false (and nothing happens) if one is active.
    pub fn start(&self, payload: DragPayload) -> bool {
        if self.slot.payload.borrow().is_some() {
            return false;
        }
        *self.slot.payload.borrow_mut() = Some(payload);
        self.slot.outcome.set(None);
        self.slot.set_active(true);
        true
    }

    /// Whether a drag is active (a tracked read).
    pub fn is_active(&self) -> bool {
        match self.slot.active.get() {
            Some(active) => active.get(),
            None => self.slot.payload.borrow().is_some(),
        }
    }

    /// How the last drag ended, `None` while one is active or before the first.
    pub fn outcome(&self) -> Option<DragOutcome> {
        self.slot.outcome.get()
    }

    /// The pointer's last position in the window during a drag (a tracked read).
    pub fn position(&self) -> Point {
        self.slot
            .position
            .get()
            .map_or(Point::ZERO, |position| position.get())
    }
}

/// What a drag handler is told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragEventKind {
    /// The drag came over the target; answer whether it accepts the payload.
    Enter,
    /// The drag moved over the target.
    Over,
    /// The drag left the target (or was cancelled over it).
    Leave,
    /// The payload was dropped on the accepting target; answer whether it took it.
    Drop,
}

/// A drag event delivered to a target.
pub struct DragCx<'a> {
    /// What happened.
    pub kind: DragEventKind,
    /// The dragged data.
    pub payload: &'a dyn Any,
    /// The pointer, in window coordinates.
    pub position: Point,
    /// The pointer in the target's coordinates.
    pub local: Point,
}

impl BuildCx<'_> {
    /// The tree's drag session.
    pub fn drag(&self) -> DragHandle {
        DragHandle {
            slot: Rc::clone(&self.tree.drag),
        }
    }

    /// Makes `element` a drag target: `handler` answers `Enter` (true to accept the payload)
    /// and `Drop` (true when it took it); its answers to `Over` and `Leave` are ignored.
    pub fn on_drag(&mut self, element: ElementId, handler: impl Fn(&DragCx<'_>) -> bool + 'static) {
        if self.tree.contains(element) {
            let handlers = self.tree.events.drags.entry(element).or_default();
            crate::events::register(handlers, Rc::new(handler) as DragHandler);
        }
    }
}

impl ViewTree {
    /// Whether a drag session is active.
    pub(crate) fn dragging(&self) -> bool {
        self.drag.payload.borrow().is_some()
    }

    /// Calls `target`'s drag handlers with `kind`; true if one answered true.
    fn drag_event(&self, target: ElementId, kind: DragEventKind, position: Point) -> bool {
        let handlers = crate::events::live(self.events.drags.get(&target));
        let Some(payload) = self.drag.payload() else {
            return false;
        };
        let origin = self.origin(target);
        let cx = DragCx {
            kind,
            payload: payload.as_ref(),
            position,
            local: Point::new(position.x - origin.x, position.y - origin.y),
        };
        let handlers = handlers.clone();
        self.enter(|| handlers.iter().fold(false, |any, h| h(&cx) || any))
    }

    /// Whether `id` has a live drag handler (VIEW-DRAG-02, VIEW-EVENT-10).
    fn is_drag_target(&self, id: ElementId) -> bool {
        self.events
            .drags
            .get(&id)
            .is_some_and(|list| list.iter().any(|(_, alive)| alive.get()))
    }

    /// The current target, if it is still in the tree (VIEW-DRAG-04).
    fn live_target(&self) -> Option<(ElementId, bool)> {
        self.events
            .drag_target
            .filter(|(id, _)| self.contains(*id) && self.is_drag_target(*id))
    }

    /// Delivers enter, leave and over for a pointer at `position` (VIEW-DRAG-02).
    pub(crate) fn drag_over(&mut self, position: Point) {
        let path = self.hit_test(position);
        let target = path.iter().copied().find(|id| self.is_drag_target(*id));
        let current = self.live_target();
        if current.map(|(id, _)| id) != target {
            if let Some((old, _)) = current {
                self.drag_event(old, DragEventKind::Leave, position);
            }
            self.events.drag_target = target.map(|id| {
                let accepts = self.drag_event(id, DragEventKind::Enter, position);
                (id, accepts)
            });
        }
        if let Some(id) = target {
            self.drag_event(id, DragEventKind::Over, position);
        }
    }

    /// Ends the session on a primary release, dropping on an accepting target
    /// (VIEW-DRAG-03).
    pub(crate) fn drop_drag(&mut self, position: Point) {
        let outcome = match self.live_target() {
            Some((id, true)) if self.drag_event(id, DragEventKind::Drop, position) => {
                DragOutcome::Accepted
            }
            _ => DragOutcome::Rejected,
        };
        self.events.drag_target = None;
        let drag = Rc::clone(&self.drag);
        self.enter(|| drag.end(outcome));
    }

    /// Cancels the session (Escape, VIEW-DRAG-03).
    pub(crate) fn cancel_drag(&mut self) {
        if let Some((id, _)) = self.live_target() {
            let position = self.events.pointer_position().unwrap_or(Point::ZERO);
            self.drag_event(id, DragEventKind::Leave, position);
        }
        self.events.drag_target = None;
        let drag = Rc::clone(&self.drag);
        self.enter(|| drag.end(DragOutcome::Cancelled));
    }

    /// Runs the drag protocol around a pointer event's normal delivery.
    pub(crate) fn drag_before(&mut self, event: &PointerEvent) {
        if self.dragging() && event.kind == PointerKind::Up(PointerButton::Primary) {
            self.drop_drag(event.position);
        }
    }

    /// Runs the drag protocol after a pointer event's normal delivery.
    pub(crate) fn drag_after(&mut self, event: &PointerEvent) {
        if self.dragging() && event.kind == PointerKind::Move {
            if let Some(position) = self.drag.position.get().copied() {
                self.enter(|| position.set(event.position));
            }
            self.drag_over(event.position);
        }
    }
}
