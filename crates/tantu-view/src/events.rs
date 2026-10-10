//! Pointer events: hit-testing, capture and bubble phases, pointer capture, hover and cursors.
//! Spec: `docs/specs/view/events.md`.

use tantu_core::{Point, Vec2};
use tantu_scene::ElementId;

use crate::{BuildCx, ViewTree};

/// A mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// The main button (usually left).
    Primary,
    /// The context-menu button (usually right).
    Secondary,
    /// The middle button or wheel press.
    Middle,
    /// Any other button.
    Other(u16),
}

/// What happened to the pointer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointerKind {
    /// A button went down.
    Down(PointerButton),
    /// A button went up.
    Up(PointerButton),
    /// The pointer moved.
    Move,
    /// Wheel or trackpad scroll in logical pixels (positive y scrolls down).
    Scroll(Vec2),
    /// The pointer came over an element (hover; delivered to that element only).
    Enter,
    /// The pointer left the window, or left an element (hover).
    Leave,
}

/// A pointer event in window coordinates (logical pixels).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    /// What happened.
    pub kind: PointerKind,
    /// Where, in window coordinates.
    pub position: Point,
}

/// When a handler sees an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    /// On the way down, root first.
    Capture,
    /// On the way up, target first.
    Bubble,
}

/// What a handler did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Handled {
    /// Let the event go on.
    Continue,
    /// Stop the dispatch here.
    Stop,
}

/// Cursor shapes an element can ask for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CursorIcon {
    /// The platform's default arrow.
    #[default]
    Default,
    /// A hand, for things that can be pressed.
    Pointer,
    /// A text caret.
    Text,
    /// An open hand.
    Grab,
    /// A closed hand.
    Grabbing,
    /// Not allowed.
    NotAllowed,
    /// Horizontal resize.
    ResizeHorizontal,
    /// Vertical resize.
    ResizeVertical,
}

/// What a handler receives.
pub struct PointerCx<'a> {
    /// The event, in window coordinates.
    pub event: &'a PointerEvent,
    /// The element the handler belongs to.
    pub element: ElementId,
    /// The event's position in the element's coordinates.
    pub local: Point,
    /// The phase it is delivered in.
    pub phase: Phase,
}

impl BuildCx<'_> {
    /// Calls `handler` for pointer events in `phase` on paths through `element`.
    pub fn on_pointer(
        &mut self,
        element: ElementId,
        phase: Phase,
        handler: impl Fn(&PointerCx<'_>) -> Handled + 'static,
    ) {
        let _ = (element, phase, handler);
        todo!()
    }

    /// The cursor while the pointer is over `element` (the innermost element with one wins).
    pub fn set_cursor(&mut self, element: ElementId, cursor: CursorIcon) {
        let _ = (element, cursor);
        todo!()
    }
}

impl ViewTree {
    /// The render elements under `position`, innermost (topmost) first.
    pub fn hit_test(&self, position: Point) -> Vec<ElementId> {
        let _ = position;
        todo!()
    }

    /// Delivers `event`: capture phase root to target, then bubble phase target to root, until
    /// a handler returns `Stop`. Returns whether any handler ran.
    pub fn dispatch_pointer(&mut self, event: PointerEvent) -> bool {
        let _ = event;
        todo!()
    }

    /// The cursor for the pointer's last position.
    pub fn cursor(&self) -> CursorIcon {
        todo!()
    }
}
