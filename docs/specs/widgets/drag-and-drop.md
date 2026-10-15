# Drag and drop, part 1: inside the app

- **Status:** Agreed (2026-10-16; decided by the agent while the user was away, review)
- **Crates:** `tantu-view` (drag sessions, drag handlers, dispatch), `tantu-widgets`
  (`Draggable`, `DragTarget`), `examples/gallery`
- **Plan item:** Phase 4, "Multi-window, drag & drop (intra-app + OS), clipboard", the
  drag-and-drop part, intra-app (OS drops are part 2)
- **Related:** [event dispatch](../view/events.md), [focus](../view/focus.md),
  [overlay](../view/overlay.md), [kitchen sink](../examples/kitchen-sink.md)

## Purpose

Users move things by dragging them: cards between columns, files into folders, rows
into a trash bin, fields onto a form designer. Flutter offers `Draggable` (the source,
with a feedback widget that follows the pointer) and `DragTarget` (a region that may
accept the dragged data). Tantu's pointer handlers can't see the tree, and while a source
holds the pointer capture no other element receives pointer events, so drag and drop
needs a small protocol in the view layer: the source starts a *drag session* with a
payload, and the tree delivers drag events to targets under the pointer.

## Scope

In scope: drag sessions in `tantu-view` (start, enter/over/leave/drop delivery, cancel
with Escape, the outcome); `Draggable` and `DragTarget` widgets with typed data, a feedback
view following the pointer and a placeholder; a gallery card.

Out of scope (and where it goes): drops from and to the operating system (files, text;
part 2, through the platform); drag-and-drop accessibility (keyboard alternatives are the
app's, as in Flutter); auto-scrolling while dragging near a scroll view's edge; multiple
simultaneous drags (touch); long-press-to-drag.

## Public API

```rust
// tantu-view
/// A drag's data, of any type.
pub type DragPayload = Rc<dyn Any>;

/// How a drag ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragOutcome { Accepted, Rejected, Cancelled }

/// The tree's drag session, for drag sources (cloneable; from `BuildCx::drag`).
#[derive(Clone)]
pub struct DragHandle { /* private */ }
impl DragHandle {
    /// Starts a drag carrying `payload`; false (and nothing happens) if one is active.
    pub fn start(&self, payload: DragPayload) -> bool;
    /// Whether a drag is active (a tracked read).
    pub fn is_active(&self) -> bool;
    /// How the last drag ended, `None` while one is active or before the first.
    pub fn outcome(&self) -> Option<DragOutcome>;
    /// The pointer's last position in the window during a drag (a tracked read).
    pub fn position(&self) -> Point;
}

/// What a drag handler is told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragEventKind { Enter, Over, Leave, Drop }

/// A drag event delivered to a target.
pub struct DragCx<'a> {
    pub kind: DragEventKind,
    pub payload: &'a dyn Any,
    /// The pointer, in window coordinates.
    pub position: Point,
    /// The pointer in the target's coordinates.
    pub local: Point,
}

impl BuildCx<'_> {
    /// The tree's drag session.
    pub fn drag(&self) -> DragHandle;
    /// Makes `element` a drag target: `handler` answers `Enter` (true to accept the
    /// payload) and `Drop` (true when it took it); its answers to `Over` and `Leave` are
    /// ignored.
    pub fn on_drag(&mut self, element: ElementId, handler: impl Fn(&DragCx<'_>) -> bool + 'static);
}

// tantu-widgets
/// A child the user can drag, carrying `data` (Flutter's `Draggable`).
pub struct Draggable<T> { /* private */ }
impl<T: Clone + 'static> Draggable<T> {
    pub fn new(data: T, child: impl View) -> Self;
    /// What follows the pointer while dragging (default: nothing).
    pub fn feedback<V: View>(self, feedback: impl Fn() -> V + 'static) -> Self;
    /// What stays in the child's place while dragging (default: the child).
    pub fn child_when_dragging<V: View>(self, view: impl Fn() -> V + 'static) -> Self;
    pub fn on_drag_started(self, f: impl Fn() + 'static) -> Self;
    /// Called when the drag ends, with whether a target accepted the data.
    pub fn on_drag_end(self, f: impl Fn(bool) + 'static) -> Self;
}

/// A region that may accept dragged data of type `T` (Flutter's `DragTarget`).
pub struct DragTarget<T> { /* private */ }
impl<T: Clone + 'static> DragTarget<T> {
    pub fn new(child: impl View) -> Self;
    /// Whether to accept the data hovering over it (default: always).
    pub fn on_will_accept(self, f: impl Fn(&T) -> bool + 'static) -> Self;
    /// Called with the data dropped on it.
    pub fn on_accept(self, f: impl Fn(T) + 'static) -> Self;
    /// Set to true while acceptable data hovers over it, false otherwise.
    pub fn candidate(self, signal: Signal<bool>) -> Self;
}
```

## Behavior

- **VIEW-DRAG-01:** `DragHandle::start` begins a session with its payload unless one is
  active; `is_active` is reactive; `outcome` is `None` while active and holds how the last
  one ended.
- **VIEW-DRAG-02:** While a session is active, every pointer move (captured or not) finds
  the innermost element on the path under the pointer that has drag handlers. When it
  changes, the old target gets `Leave` and the new one `Enter`, whose answer records
  whether it accepts; then the target gets `Over`. Overlays with pointer passthrough are
  skipped by the hit test as usual, so feedback under the pointer doesn't hide targets.
- **VIEW-DRAG-03:** A primary release ends the session before the release is delivered
  normally: an accepting target gets `Drop`, and the outcome is `Accepted` if it answered
  true, else `Rejected` (also when there was no accepting target). Escape (pressed while
  a session is active) cancels it: the target gets `Leave` and the outcome is
  `Cancelled`; the key is consumed. A pointer leaving the window doesn't end the session.
- **VIEW-DRAG-04:** A target removed during a session is forgotten without events; its
  handlers go with it (VIEW-EVENT-06).
- **VIEW-DRAG-05:** `DragHandle::position` follows the pointer during a session: every
  move updates it before targets get their events. The source needs it because its own
  pointer capture can end mid-drag (a placeholder replacing the pressed child takes the
  child offstage, which releases the capture, VIEW-OFFSTAGE); found while implementing,
  review.
- **WIDGETS-DND-01:** `Draggable`: a primary press, then a move of more than 4 px with the
  button held, starts a session carrying its data (`Rc<T>` as the payload) and calls
  `on_drag_started`; the child is replaced by `child_when_dragging` (if given) and the
  feedback is shown in a pointer-passthrough overlay whose top-left corner keeps the
  offset from the pointer (`DragHandle::position`) that the press had from the child's
  top-left corner. When the
  session ends (release or Escape), the feedback goes, the child comes back and
  `on_drag_end` is called with whether the outcome was `Accepted`. A press and release
  without moving starts nothing (the press still reaches handlers below, so a `Button`
  inside still works).
- **WIDGETS-DND-02:** `DragTarget<T>` is a drag target for payloads holding a `T`; others
  are not accepted. `Enter` accepts when `on_will_accept` (default: true) does, and the
  candidate signal follows (true from an accepting `Enter`, false on `Leave` or `Drop`); a
  `Drop` calls `on_accept` with a clone of the data and answers true.
- **GALLERY-49:** A "dnd" kitchen-sink card after "dashboard" (twenty-six `SINK_CARDS`
  entries, samples `draggable` and `drag_target`): three chips ("Red", "Green", "Blue")
  and two bins ("Warm" accepts only Red; "Any" accepts all), the bins highlighted while a
  candidate hovers, echoed by `sink-dnd-state` ("Warm: none, Any: none", then e.g. "Warm:
  Red, Any: none"). Test: dragging Red onto Warm drops it; dragging Blue onto Warm is
  rejected.

## Performance and allocation

During a drag each move adds one hit test; nothing happens without an active session.

## Open questions

Decided by the agent while the user was away (2026-10-16), for review:

1. **A session in the view layer** (tree-held state reached through a handle, like the
   clipboard) rather than geometry registries in the widgets, so any element can be a
   target and hit-testing stays the tree's.
2. **The release ends the session before normal delivery**, so the source's release
   handler already knows the outcome.
3. **Payloads are `Rc<dyn Any>`**; the widgets wrap `T` and targets downcast, so mismatched
   types are simply not accepted.
4. **Escape cancels**, as on every desktop.
