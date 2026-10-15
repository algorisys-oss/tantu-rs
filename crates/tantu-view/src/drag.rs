//! Drag sessions: a source starts one with a payload and the tree delivers drag events to
//! targets under the pointer. Spec: `docs/specs/widgets/drag-and-drop.md` (VIEW-DRAG-01..04).

use std::any::Any;
use std::rc::Rc;

use tantu_core::Point;

use crate::{BuildCx, ElementId};

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
    _private: (),
}

impl DragHandle {
    /// Starts a drag carrying `payload`; false (and nothing happens) if one is active.
    pub fn start(&self, payload: DragPayload) -> bool {
        let _ = payload;
        todo!("VIEW-DRAG-01")
    }

    /// Whether a drag is active (a tracked read).
    pub fn is_active(&self) -> bool {
        todo!("VIEW-DRAG-01")
    }

    /// How the last drag ended, `None` while one is active or before the first.
    pub fn outcome(&self) -> Option<DragOutcome> {
        todo!("VIEW-DRAG-01")
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
        DragHandle { _private: () }
    }

    /// Makes `element` a drag target: `handler` answers `Enter` (true to accept the payload)
    /// and `Drop` (true when it took it); its answers to `Over` and `Leave` are ignored.
    pub fn on_drag(&mut self, element: ElementId, handler: impl Fn(&DragCx<'_>) -> bool + 'static) {
        let _ = (element, handler);
        todo!("VIEW-DRAG-02")
    }
}
