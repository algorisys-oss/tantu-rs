//! Pointer events: hit-testing, capture and bubble phases, pointer capture, hover and cursors.
//! Spec: `docs/specs/view/events.md`.

use std::collections::HashMap;
use std::rc::Rc;

use tantu_core::{Point, Size, Vec2};
use tantu_scene::ElementId;

use crate::tree::Kind;
use crate::{BuildCx, ViewTree};

/// A registered pointer handler.
type Handler = Rc<dyn Fn(&PointerCx<'_>) -> Handled>;

/// The tree's pointer state: handlers, cursors, capture and hover.
#[derive(Default)]
pub(crate) struct EventState {
    handlers: HashMap<ElementId, Vec<(Phase, Handler)>>,
    cursors: HashMap<ElementId, CursorIcon>,
    /// The path (innermost first) holding the pointer, and the button that pressed it.
    captured: Option<(Vec<ElementId>, PointerButton)>,
    /// The path (innermost first) under the pointer at the last move.
    hovered: Vec<ElementId>,
    /// The last pointer position, `None` after the pointer left the window.
    position: Option<Point>,
}

impl EventState {
    /// Forgets everything registered on `id` (VIEW-EVENT-06).
    pub(crate) fn forget(&mut self, id: ElementId) {
        self.handlers.remove(&id);
        self.cursors.remove(&id);
    }
}

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
    /// The element's size from the last layout (`Size::ZERO` if never laid out).
    pub size: Size,
}

impl BuildCx<'_> {
    /// Calls `handler` for pointer events in `phase` on paths through `element`.
    pub fn on_pointer(
        &mut self,
        element: ElementId,
        phase: Phase,
        handler: impl Fn(&PointerCx<'_>) -> Handled + 'static,
    ) {
        if self.tree.contains(element) {
            let handlers = self.tree.events.handlers.entry(element).or_default();
            handlers.push((phase, Rc::new(handler)));
        }
    }

    /// The cursor while the pointer is over `element` (the innermost element with one wins).
    pub fn set_cursor(&mut self, element: ElementId, cursor: CursorIcon) {
        if self.tree.contains(element) {
            self.tree.events.cursors.insert(element, cursor);
        }
    }
}

impl ViewTree {
    /// The render elements under `position`, innermost (topmost) first.
    pub fn hit_test(&self, position: Point) -> Vec<ElementId> {
        let mut path = Vec::new();
        self.hit_into(self.root(), position, Vec2::ZERO, &mut path);
        path
    }

    /// Delivers `event`: capture phase root to target, then bubble phase target to root, until
    /// a handler returns `Stop`. Returns whether any handler ran.
    pub fn dispatch_pointer(&mut self, event: PointerEvent) -> bool {
        // A capture whose holder was removed is released (VIEW-EVENT-06).
        if let Some((path, _)) = &self.events.captured {
            if !path.first().is_some_and(|target| self.contains(*target)) {
                self.events.captured = None;
            }
        }
        if event.kind == PointerKind::Leave {
            self.events.position = None;
            self.update_hover(Vec::new(), event.position);
            return false;
        }
        self.events.position = Some(event.position);
        if let Some((path, button)) = self.events.captured.clone() {
            // Captured: everything goes to the holder's path (VIEW-EVENT-03).
            let ran = self.deliver(&path, &event);
            if event.kind == PointerKind::Up(button) {
                self.events.captured = None;
                let under = self.hit_test(event.position);
                self.update_hover(under, event.position);
            }
            return ran;
        }
        let path = self.hit_test(event.position);
        if event.kind == PointerKind::Move {
            self.update_hover(path.clone(), event.position);
        }
        let ran = self.deliver(&path, &event);
        if let PointerKind::Down(button) = event.kind {
            if ran && !path.is_empty() {
                self.events.captured = Some((path, button));
            }
        }
        ran
    }

    /// The cursor for the pointer's last position.
    pub fn cursor(&self) -> CursorIcon {
        let Some(position) = self.events.position else {
            return CursorIcon::Default;
        };
        self.hit_test(position)
            .iter()
            .find_map(|id| self.events.cursors.get(id).copied())
            .unwrap_or_default()
    }

    /// Hit-tests `id` (whose render parent's top-left is at `parent_origin` in the window),
    /// pushing the hit path innermost first; true if anything was hit (VIEW-EVENT-01).
    fn hit_into(
        &self,
        id: ElementId,
        position: Point,
        parent_origin: Vec2,
        path: &mut Vec<ElementId>,
    ) -> bool {
        let children = self.children(id);
        let layout = match self.element_kind(id) {
            Some(Kind::Region) => {
                // Transparent: the children are tested in its place, topmost first.
                return children
                    .iter()
                    .rev()
                    .any(|child| self.hit_into(*child, position, parent_origin, path));
            }
            Some(Kind::Render(layout, _)) => *layout,
            None => return false,
        };
        let tree = self.layout_tree();
        let (Some(size), Some(offset)) = (tree.size(layout), tree.offset(layout)) else {
            return false; // never laid out
        };
        let origin = parent_origin + offset;
        let (x, y) = (position.x - origin.x, position.y - origin.y);
        if !(x >= 0.0 && x < size.width && y >= 0.0 && y < size.height) {
            return false;
        }
        for child in children.iter().rev() {
            if self.hit_into(*child, position, origin, path) {
                break;
            }
        }
        path.push(id);
        true
    }

    /// The window position of a render element's top-left corner (sum of layout offsets).
    fn origin(&self, id: ElementId) -> Vec2 {
        let mut origin = Vec2::ZERO;
        let mut next = Some(id);
        while let Some(current) = next {
            if let Some(layout) = self.layout_id(current) {
                origin += self.layout_tree().offset(layout).unwrap_or(Vec2::ZERO);
            }
            next = self.parent(current);
        }
        origin
    }

    /// Runs capture handlers root to target, then bubble handlers target to root, until one
    /// stops (VIEW-EVENT-02). True if any ran.
    fn deliver(&self, path: &[ElementId], event: &PointerEvent) -> bool {
        let capture = path.iter().rev().map(|id| (*id, Phase::Capture));
        let bubble = path.iter().map(|id| (*id, Phase::Bubble));
        let mut ran = false;
        self.enter(|| {
            for (id, phase) in capture.chain(bubble) {
                if self.call(id, phase, event, &mut ran) == Handled::Stop {
                    return;
                }
            }
        });
        ran
    }

    /// Calls `id`'s handlers for `phase`; `Stop` if one stopped.
    fn call(&self, id: ElementId, phase: Phase, event: &PointerEvent, ran: &mut bool) -> Handled {
        let Some(handlers) = self.events.handlers.get(&id) else {
            return Handled::Continue;
        };
        let origin = self.origin(id);
        let cx = PointerCx {
            event,
            element: id,
            local: Point::new(event.position.x - origin.x, event.position.y - origin.y),
            phase,
            size: self
                .layout_id(id)
                .and_then(|layout| self.layout_tree().size(layout))
                .unwrap_or(Size::ZERO),
        };
        for (_, handler) in handlers.iter().filter(|(p, _)| *p == phase) {
            *ran = true;
            if handler(&cx) == Handled::Stop {
                return Handled::Stop;
            }
        }
        Handled::Continue
    }

    /// Sends `Leave` to elements no longer under the pointer (innermost first) and `Enter` to
    /// new ones (outermost first), each to that element only (VIEW-EVENT-04).
    fn update_hover(&mut self, under: Vec<ElementId>, position: Point) {
        let old = std::mem::replace(&mut self.events.hovered, under);
        let mut ran = false;
        let send = |id: ElementId, kind: PointerKind, ran: &mut bool| {
            let event = PointerEvent { kind, position };
            self.enter(|| self.call(id, Phase::Bubble, &event, ran));
        };
        let new = &self.events.hovered;
        for id in old
            .iter()
            .filter(|id| !new.contains(id) && self.contains(**id))
        {
            send(*id, PointerKind::Leave, &mut ran);
        }
        for id in new.iter().rev().filter(|id| !old.contains(id)) {
            send(*id, PointerKind::Enter, &mut ran);
        }
    }
}
