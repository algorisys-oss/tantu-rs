# Reactive props and frames

- **Status:** Implemented (decided by the agent while the user was away, at their request; review)
- **Crate:** `tantu-view`
- **Plan item:** Phase 2, `tantu-view` → "Reactive props and frames"
- **Related:** [ADR 0011](../../adr/0011-view-layer.md) (points 1 and 6), [view tree](tree.md),
  [paint](paint.md), [signals](../reactive/signals.md), [layout tree](../layout/tree.md) (`set`)

## Purpose

How signals reach elements, and what a frame does (ADR 0011). A widget property is a `Prop<T>`:
a fixed value, or a closure (or signal) read again when its signals change. `BuildCx::bind`
connects a prop to an element: the value is applied once while building, and for a dynamic
prop an effect recomputes it on change and queues the new value. `ViewTree::frame` applies the
queued values (updating layout objects through `LayoutTree::set`, so layout is redone only for
real changes), runs layout and repaints the window's Scene. The app runner calls `frame` when
the tree says it needs one.

## Scope

In scope:

- `Prop`, `IntoProp` (values, closures, `Signal`, `Memo`).
- `BuildCx::bind`, `ElementMut` (updating an element's layout object or paint).
- The update queue, coalescing, `needs_frame`, the frame requester, `ViewTree::frame`,
  `FrameReport`.

Out of scope (and where it goes):

- Dynamic content (`Dyn`, `Show`, `For`): dynamic-content spec; their rebuilds use the same
  queue.
- Damage regions and partial repaint: Phase 5 (every frame repaints the whole Scene).
- Animation tickers and vsync: Phase 3 animation item (they will request frames the same way).

## Public API

Crate root `tantu_view`.

```rust
use std::sync::Arc;
use tantu_core::Size;
use tantu_layout::{BoxConstraints, RenderBox, TextMeasure};
use tantu_reactive::{Memo, Signal};
use tantu_scene::{ElementId, Scene};

/// A property value: fixed, or computed (re-read when the signals it reads change).
pub enum Prop<T> {
    Value(T),
    Dynamic(Box<dyn Fn() -> T>),
}
impl<T> Prop<T> {
    /// The current value (calls the closure for `Dynamic`, tracking its signals if inside an
    /// effect).
    pub fn get(&self) -> T where T: Clone;
    /// True for `Dynamic`.
    pub fn is_dynamic(&self) -> bool;
}

/// What a widget setter accepts: a value, a closure returning one, or a signal/memo.
pub trait IntoProp<T> {
    fn into_prop(self) -> Prop<T>;
}
impl<T, F: Fn() -> T + 'static> IntoProp<T> for F { /* Dynamic */ }
impl<T: Clone + 'static> IntoProp<T> for Signal<T> { /* Dynamic, reading the signal */ }
impl<T: Clone + PartialEq + 'static> IntoProp<T> for Memo<T> { /* Dynamic, reading the memo */ }
impl<T> IntoProp<T> for Prop<T> { /* itself */ }
// Values: Value(self) for bool, the integer and float types, char, String, Arc<str>,
// tantu_core::{Color, Size, Point, Vec2, Rect, EdgeInsets}, tantu_layout::{Alignment, Axis,
// MainAxisAlignment, MainAxisSize, CrossAxisAlignment, FlexFit, StackFit, WrapAlignment,
// WrapCrossAlignment, TextWidthBasis, TextStyleKey}; and `&'static str` as `Prop<Arc<str>>`
// and `Prop<String>`. Other types: `Prop::Value(x)` or a closure.

/// Mutable access to one element while applying a prop.
pub struct ElementMut<'a> { /* private */ }
impl ElementMut<'_> {
    pub fn id(&self) -> ElementId;
    /// The element's layout object, if it is an `R`.
    pub fn render<R: RenderBox>(&self) -> Option<&R>;
    /// Changes a copy of the layout object with `f` and stores it with `LayoutTree::set`, so the
    /// node is re-laid out only if it changed. Returns whether it changed (false if the element
    /// isn't a render element with an `R`).
    pub fn update_render<R: RenderBox + Clone + PartialEq>(&mut self, f: impl FnOnce(&mut R)) -> bool;
    /// The element's paint behavior, if it is a `P`.
    pub fn paint<P: Paint>(&self) -> Option<&P>;
    /// Changes the paint behavior in place. Returns false if it isn't a `P`.
    pub fn update_paint<P: Paint>(&mut self, f: impl FnOnce(&mut P)) -> bool;
}

impl BuildCx<'_> {
    /// Connects `prop` to `element`: calls `apply` with its value now, and for a dynamic prop,
    /// again at the next frame after the signals it reads change (with the latest value).
    pub fn bind<T: 'static>(&mut self, element: ElementId, prop: Prop<T>, apply: impl Fn(&mut ElementMut<'_>, T) + 'static);
}

/// What a frame did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameReport {
    /// Prop values applied.
    pub applied: usize,
}

impl ViewTree {
    /// True when something changed since the last frame.
    pub fn needs_frame(&self) -> bool;
    /// Called once each time the tree goes from not needing a frame to needing one (the app
    /// runner asks the platform for a redraw).
    pub fn set_frame_requester(&mut self, requester: impl Fn() + 'static);
    /// Applies the queued prop values, lays out with `constraints` (the window) and `text`, and
    /// repaints `scene` from scratch, sized to the window (or to the root's size when the
    /// constraints are unbounded).
    pub fn frame(&mut self, constraints: BoxConstraints, text: &mut dyn TextMeasure, scene: &mut Scene) -> FrameReport;
}
```

## Behavior

- **VIEW-FRAME-01:** `IntoProp`: a value of a listed type gives `Prop::Value` of itself
  (`&'static str` gives the string); a closure gives `Prop::Dynamic`; `Signal`/`Memo` give
  `Dynamic` props reading them; a `Prop` gives itself. `get` returns the value (calling the
  closure), `is_dynamic` says which kind it is.
- **VIEW-FRAME-02:** `bind` with a `Value` calls `apply` exactly once, during the build, and never
  again.
- **VIEW-FRAME-03:** `bind` with a `Dynamic` calls `apply` once during the build with the closure's
  value (before the tree's first frame). When a signal the closure read changes, the closure
  runs again right away (it is an effect in the element's scope) but `apply` doesn't: the value
  is kept and applied at the next `frame`. Several changes before a frame are coalesced: `apply`
  is called once, with the latest value.
- **VIEW-FRAME-04:** After a change, `needs_frame()` is true and the frame requester has been called
  once; further changes before the frame don't call it again. `frame` clears `needs_frame`.
  A new tree doesn't need a frame for its build-time values (they're already applied) but its
  first `frame` still lays out and paints.
- **VIEW-FRAME-05:** `frame` applies the pending values (in the order their bindings first queued
  them), then runs `ViewTree::layout(constraints, text)`, then repaints `scene` from scratch
  (`begin` with the window size, `ViewTree::paint`, `finish`), and returns how many values it
  applied. A frame with nothing pending still lays out and paints.
- **VIEW-FRAME-06:** `ElementMut::update_render` with the element's layout-object type changes a copy
  and stores it through `LayoutTree::set`: equal → nothing marked, false; different → the node
  needs layout, true. With another type or on a region it returns false. `update_paint` changes
  the paint behavior in place when the type matches (true), else false. `render` and `paint`
  return the current objects when the types match.
- **VIEW-FRAME-07:** Removing an element stops its bindings (their effects are disposed with its
  scope) and drops their pending values: the next frame applies nothing for it and nothing
  panics.
- **VIEW-FRAME-08:** Robustness: `frame` with unbounded constraints sizes the Scene to the root's
  size; a closure that reads no signal is applied once and never again; `apply` may itself
  write signals (values queued by it are applied in the next frame, not the current one).

## Performance and allocation

A change costs one closure run and, the first time a binding changes before a frame, one queue
entry (a boxed closure). The frame's apply loop is O(pending bindings). Static props (`Value`)
create no effect and allocate nothing beyond the build.

## Open questions

Resolved (2026-10-10, decided by the agent while the user was away; review):

1. **`IntoProp` by a list of value types plus a blanket closure impl.** A blanket "any `T`" impl
   would conflict with the closure impl in Rust's coherence rules; listing the common types (with
   `Prop::Value` for others) is what Leptos-style APIs do. `Fn` being a fundamental trait makes
   the per-type impls coexist with the closure impl.
2. **Coalescing per binding** (only the latest value is applied per frame), instead of applying
   every intermediate value.
3. **Every frame repaints the whole Scene**; damage tracking is Phase 5.
4. **Values applied inside `frame`, not inside effects**, as ADR 0011 requires (no re-entrancy;
   one place where the tree changes).
