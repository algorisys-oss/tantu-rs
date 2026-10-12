# Inherited values

- **Status:** Implemented (decided by the agent while the user was away, 2026-10-12; review)
- **Crate:** `tantu-view`
- **Plan item:** Phase 3, "`tantu-theme`" (groundwork: how a theme reaches widgets)
- **Related:** [view tree](tree.md), [dynamic content](dynamic.md), Flutter's
  `InheritedWidget` / `Theme.of(context)`

## Purpose

Some values belong to a subtree rather than to one widget: the theme, text direction, a
form's state. Flutter passes them down with `InheritedWidget`. Tantu's version is a value
attached to an element, found by walking up the element tree while building. Reactivity comes
from the value itself: provide a `Signal<Theme>` (a `Copy` handle) and consumers read it in
their reactive closures, so a theme switch repaints exactly what reads it.

## Public API

```rust
impl BuildCx<'_> {
    /// Attaches `value` to the element being built (the current parent), for the views built
    /// under it; a second value of the same type replaces the first.
    pub fn provide<T: 'static>(&mut self, value: T);
    /// The nearest `T` provided on the element being built or one of its ancestors.
    pub fn inherit<T: Clone + 'static>(&self) -> Option<T>;
}

impl ViewTree {
    /// The `T` provided on `element` itself, if any (tests and inspectors).
    pub fn inherited_on<T: Clone + 'static>(&self, element: ElementId) -> Option<T>;
}

/// A region providing `value` to the views built under it.
pub struct Provide<T, V> { /* value, child */ }
impl<T: 'static, V: View> Provide<T, V> { pub fn new(value: T, child: V) -> Self; }
impl<T: 'static, V: View> View for Provide<T, V> {}
```

## Behavior

- **VIEW-INHERIT-01:** `inherit::<T>()` returns a clone of the nearest provided `T`, starting
  at the element being built and going up to the root; `None` when no ancestor provides one.
  Values of different types are independent, and a value provided deeper shadows the same
  type provided higher. Overlay wrappers continue lookup at their portal target rather than
  the root layer (VIEW-OVERLAY-08..10, [overlay](overlay.md)); structural ancestry stays unchanged.
- **VIEW-INHERIT-02:** `Provide::new(value, child)` builds a region holding `value`, so
  `child` and everything under it inherit it, siblings of the `Provide` don't, and content a
  `Dyn` under it rebuilds later still inherits it.
- **VIEW-INHERIT-03:** `ViewTree::inherited_on(element)` reads a value provided on that element
  alone; removing an element drops its values (`inherited_on` then returns `None`).

## Open questions

Decided by the agent (review): values are looked up at build time (not reactively), so a
provider must be above its consumers when they build; reactive values are signals provided
as values. A `Copy` signal handle is the recommended form for anything that changes.
