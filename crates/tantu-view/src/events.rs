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

/// Cleared when the reactive scope current at a handler's registration is disposed, so the
/// handler is dropped with the component that registered it (VIEW-EVENT-10).
#[derive(Clone)]
pub(crate) struct Alive(Rc<std::cell::Cell<bool>>);

impl Alive {
    /// A flag owned by the current reactive scope.
    pub(crate) fn here() -> Self {
        let flag = Rc::new(std::cell::Cell::new(true));
        let cleared = Rc::clone(&flag);
        tantu_reactive::on_cleanup(move || cleared.set(false));
        Alive(flag)
    }

    pub(crate) fn get(&self) -> bool {
        self.0.get()
    }
}

/// A handler with the flag that says whether its registering scope still exists.
pub(crate) type Live<H> = (H, Alive);

/// Appends `handler` to `list`, first dropping the handlers whose scopes are gone.
pub(crate) fn register<H>(list: &mut Vec<(H, Alive)>, handler: H) {
    list.retain(|(_, alive)| alive.get());
    list.push((handler, Alive::here()));
}

/// The live handlers of `list`, cloned.
pub(crate) fn live<H: Clone>(list: Option<&Vec<(H, Alive)>>) -> Vec<H> {
    list.map_or_else(Vec::new, |list| {
        list.iter()
            .filter(|(_, alive)| alive.get())
            .map(|(handler, _)| handler.clone())
            .collect()
    })
}
/// An accessibility action handler (VIEW-SEM-03).
pub(crate) type ActionHandler = Rc<dyn Fn(&crate::ActionCx<'_>) -> bool>;

/// The tree's pointer state: handlers, cursors, capture and hover.
#[derive(Default)]
pub(crate) struct EventState {
    handlers: HashMap<ElementId, Vec<Live<(Phase, Handler)>>>,
    cursors: HashMap<ElementId, CursorIcon>,
    /// The path (innermost first) holding the pointer, and the button that pressed it.
    captured: Option<(Vec<ElementId>, PointerButton)>,
    /// The path (innermost first) under the pointer at the last move.
    hovered: Vec<ElementId>,
    /// The last pointer position, `None` after the pointer left the window.
    position: Option<Point>,
    /// Accessibility action handlers (VIEW-SEM-03).
    pub(crate) actions: HashMap<ElementId, Vec<(ActionHandler, Alive)>>,
    /// Drag targets' handlers (VIEW-DRAG-02).
    pub(crate) drags: HashMap<ElementId, Vec<(crate::drag::DragHandler, Alive)>>,
    /// The drag target under the pointer and whether it accepts the payload.
    pub(crate) drag_target: Option<(ElementId, bool)>,
    /// File-drop targets' handlers (VIEW-FILEDROP-01).
    pub(crate) file_drops: HashMap<ElementId, Vec<(crate::file_drop::FileDropHandler, Alive)>>,
    /// The target files from the OS are hovering over (VIEW-FILEDROP-02).
    pub(crate) file_target: Option<ElementId>,
}

impl EventState {
    /// Forgets everything registered on `id` (VIEW-EVENT-06).
    pub(crate) fn forget(&mut self, id: ElementId) {
        self.handlers.remove(&id);
        self.cursors.remove(&id);
        self.actions.remove(&id);
        self.drags.remove(&id);
        self.file_drops.remove(&id);
    }

    /// The last pointer position in the window, if the pointer is in it.
    pub(crate) fn pointer_position(&self) -> Option<Point> {
        self.position
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
    /// The modifiers held when this event occurred.
    pub modifiers: crate::Modifiers,
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
            register(handlers, (phase, Rc::new(handler) as Handler));
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
        // A release ends a drag before its normal delivery; moves update the drag target
        // after it (VIEW-DRAG-02, -03).
        self.drag_before(&event);
        let ran = self.dispatch_pointer_normally(event);
        self.drag_after(&event);
        ran
    }

    fn dispatch_pointer_normally(&mut self, event: PointerEvent) -> bool {
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
        if event.kind == PointerKind::Down(PointerButton::Primary) {
            self.outside_presses(&path);
            self.focus_on_press(&path);
        }
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

    /// The cursor for the pointer's last position; while a capture is held, the cursor of
    /// the captured path wherever the pointer is (VIEW-EVENT-09).
    pub fn cursor(&self) -> CursorIcon {
        if let Some((path, _)) = &self.events.captured {
            return path
                .iter()
                .find_map(|id| self.events.cursors.get(id).copied())
                .unwrap_or_default();
        }
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
        if self.element(id).is_none_or(|e| e.offstage) {
            return false; // Offstage subtrees aren't hit (VIEW-OFFSTAGE-03).
        }
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
        if self.is_overlay_layer(id) {
            // Transparent itself; passthrough overlays are skipped (VIEW-OVERLAY-04).
            return children
                .iter()
                .rev()
                .filter(|child| !self.is_passthrough(**child))
                .any(|child| self.hit_into(*child, position, origin, path));
        }
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
    pub(crate) fn origin(&self, id: ElementId) -> Vec2 {
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
        let current = handlers
            .iter()
            .filter(|((p, _), alive)| *p == phase && alive.get());
        for ((_, handler), _) in current {
            *ran = true;
            if handler(&cx) == Handled::Stop {
                return Handled::Stop;
            }
        }
        Handled::Continue
    }

    /// After a subtree went offstage: hovered elements in it get `Leave` and a capture whose
    /// target is in it is released without an `Up` (VIEW-OFFSTAGE-03).
    pub(crate) fn release_pointer_offstage(&mut self) {
        if let Some((path, _)) = &self.events.captured {
            if path.first().is_some_and(|target| self.is_offstage(*target)) {
                self.events.captured = None;
            }
        }
        if !self.events.hovered.iter().any(|id| self.is_offstage(*id)) {
            return;
        }
        let position = self.events.position.unwrap_or(Point::ZERO);
        let kept = self
            .events
            .hovered
            .iter()
            .copied()
            .filter(|id| !self.is_offstage(*id))
            .collect();
        self.update_hover(kept, position);
    }

    /// Sends `Leave` to elements no longer under the pointer (innermost first) and `Enter` to
    /// new ones (outermost first), each to that element only (VIEW-EVENT-04).
    fn update_hover(&mut self, under: Vec<ElementId>, position: Point) {
        let old = std::mem::replace(&mut self.events.hovered, under);
        let mut ran = false;
        let send = |id: ElementId, kind: PointerKind, ran: &mut bool| {
            let event = PointerEvent {
                kind,
                position,
                modifiers: Default::default(),
            };
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
