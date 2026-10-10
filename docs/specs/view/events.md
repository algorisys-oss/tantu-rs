# Event dispatch

- **Status:** Agreed (the user said "continue" on the draft, taking its proposals)
- **Crate:** `tantu-view`
- **Plan item:** Phase 2, "Event dispatch: hit-testing, bubbling/capture, pointer capture,
  cursor icons"
- **Related:** [ADR 0011](../../adr/0011-view-layer.md), [view tree](tree.md), [frame](frame.md),
  [platform](../platform/platform.md) (the event types the app runner receives)

## Purpose

How pointer input reaches the elements that handle it. The app runner receives platform events
and passes pointer events to `ViewTree::dispatch_pointer`. The tree hit-tests them against the
last layout's geometry, then delivers them to handlers along the path from the root to the
element under the pointer: a capture phase (root first) and a bubble phase (target first), as
in the DOM and Flutter's gesture arena, simplified. A handler can stop propagation. A press
captures the pointer for the element that handled it, until release. Handlers run with the
tree's runtime current and usually write signals; effects queue updates for the next frame
(ADR 0011).

## Scope

In scope:

- `PointerEvent` (tantu-view's own type; the runner converts platform events).
- `BuildCx::on_pointer` (a handler per element and phase), `BuildCx::set_cursor`.
- `ViewTree::hit_test`, `ViewTree::dispatch_pointer`, pointer capture, hover enter/leave,
  `ViewTree::cursor`.

Out of scope (and where it goes):

- Keyboard input, focus, shortcuts: Phase 3 "Focus system".
- Gestures (tap, drag, long press, scale) built on these events: Phase 3 widgets
  (`Button` uses press/release directly for the counter).
- Touch and pen specifics, multi-pointer: later (one pointer for now).
- Hit-testing through paint transforms set by a `Paint` (only layout offsets are used): later,
  with transformed widgets.

## Public API

```rust
use tantu_core::{Point, Vec2};

/// A mouse button.
pub enum PointerButton { Primary, Secondary, Middle, Other(u16) }

/// What happened to the pointer.
pub enum PointerKind {
    Down(PointerButton),
    Up(PointerButton),
    Move,
    /// Wheel or trackpad scroll in logical pixels (positive y scrolls down).
    Scroll(Vec2),
    /// The pointer came over an element (hover, VIEW-EVENT-04; delivered to that element only).
    Enter,
    /// The pointer left the window, or left an element (hover).
    Leave,
}

/// A pointer event in window coordinates (logical pixels).
pub struct PointerEvent { pub kind: PointerKind, pub position: Point }

/// When a handler sees an event.
pub enum Phase { Capture, Bubble }

/// What a handler did.
pub enum Handled { Continue, Stop }

/// Cursor shapes an element can ask for.
pub enum CursorIcon { Default, Pointer, Text, Grab, Grabbing, NotAllowed, ResizeHorizontal, ResizeVertical }

/// What a handler receives: the event, the element, and its position in the element's
/// coordinates.
pub struct PointerCx<'a> { /* event, element, local position, phase */ }

impl BuildCx<'_> {
    /// Calls `handler` for pointer events in `phase` on paths through `element`.
    pub fn on_pointer(&mut self, element: ElementId, phase: Phase, handler: impl Fn(&PointerCx<'_>) -> Handled + 'static);
    /// The cursor while the pointer is over `element` (the innermost element with one wins).
    pub fn set_cursor(&mut self, element: ElementId, cursor: CursorIcon);
}

impl ViewTree {
    /// The render elements under `position`, innermost (topmost) first.
    pub fn hit_test(&self, position: Point) -> Vec<ElementId>;
    /// Delivers `event`: capture phase root to target, then bubble phase target to root, until
    /// a handler returns `Stop`. Returns whether any handler ran.
    pub fn dispatch_pointer(&mut self, event: PointerEvent) -> bool;
    /// The cursor for the pointer's last position.
    pub fn cursor(&self) -> CursorIcon;
}
```

## Behavior

- **VIEW-EVENT-01:** Hit-testing uses the last layout: an element is under a point when the point
  lies in its bounds (offset accumulated from the root, size from layout). Among overlapping
  siblings the later one (painted on top) wins; the path is that element and its ancestors.
  Regions are transparent (their children are tested in their place). Elements never laid out
  are not hit.
- **VIEW-EVENT-02:** `dispatch_pointer` runs capture handlers from the root down to the target, then
  bubble handlers from the target up to the root, each with the event's position in its own
  coordinates; a handler returning `Stop` ends the dispatch. Handlers run with the tree's
  runtime current, so they can read and write signals.
- **VIEW-EVENT-03:** Pointer capture: after a `Down` that some handler on the path handled, every
  later event goes to the same path (even outside its bounds) until the matching `Up`, after
  which hit-testing resumes.
- **VIEW-EVENT-04:** Hover: when the element under the pointer changes on `Move` (or `Leave`), the
  elements that are no longer under it receive a `Leave` event and new ones an `Enter` event
  (`PointerKind` gains `Enter`), outermost first for enter and innermost first for leave.
- **VIEW-EVENT-05:** `cursor()` is the cursor of the innermost element under the pointer that set
  one, else `Default`.
- **VIEW-EVENT-06:** Removing an element removes its handlers and cursor; a capture held by a removed
  element is released. Nothing panics for any event, including during dispatch to an element
  that a handler removes (the removal takes effect at the next frame, ADR 0011).

## Open questions

Resolved (2026-10-10; the user said "continue" on the draft, taking the proposals):

1. **tantu-view's own `PointerEvent`**; the runner converts platform events.
2. **Capture and bubble with `Stop`**; a gesture arena comes with Phase 3 gestures.
3. **Hit-testing by layout bounds only** for Phase 2. An element's subtree is searched only when
   the point lies in the element's bounds (as Flutter does).
