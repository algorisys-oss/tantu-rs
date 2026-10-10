# Dynamic content

- **Status:** Agreed (decided by the agent while the user was away, at their request; review)
- **Crate:** `tantu-view`
- **Plan item:** Phase 2, `tantu-view` → "Dynamic content"
- **Related:** [ADR 0011](../../adr/0011-view-layer.md) (point 7), [view tree](tree.md) (regions,
  `remove`), [frame](frame.md) (the queue)

## Purpose

Components run once (ADR 0011), so content that changes shape needs explicit views: `Dyn`
re-runs a view closure when its signals change, `Show` switches on a condition, and `For` keeps
a keyed list in step with a signal-driven `Vec`. Each is a region element, so its children take
its place in the parent's layout (a `For` inside a `Column` gives the column one child per item).
Rebuilds are queued like prop values and done in the next `frame`.

## Scope

In scope:

- `Dyn`, `Show`, `For`: building, rebuilding at frames, ownership of what their content
  creates, removal.

Out of scope (and where it goes):

- Enter/exit animations for list items (`AnimatedList`-style): Phase 3 animation item.
- Virtualization (building only visible items): Phase 3 `ListView`.

## Public API

Crate root `tantu_view`.

```rust
use std::hash::Hash;

/// Content rebuilt from `f` whenever the signals `f` reads change.
pub struct Dyn<V> { /* private */ }
impl<V: View> Dyn<V> {
    pub fn new(f: impl Fn() -> V + 'static) -> Self;
}
impl<V: View> View for Dyn<V> { /* a region */ }

/// `then` while `when` is true, else `fallback` (or nothing).
pub struct Show { /* private */ }
impl Show {
    pub fn new<V: View>(when: impl Fn() -> bool + 'static, then: impl Fn() -> V + 'static) -> Self;
    pub fn fallback<V: View>(self, fallback: impl Fn() -> V + 'static) -> Self;
}
impl View for Show { /* a region */ }

/// One child per item of `each`, matched to existing children by `key`.
pub struct For<T, K, V> { /* private */ }
impl<T: 'static, K: Eq + Hash + Clone + 'static, V: View> For<T, K, V> {
    pub fn new(
        each: impl Fn() -> Vec<T> + 'static,
        key: impl Fn(&T) -> K + 'static,
        view: impl Fn(T) -> V + 'static,
    ) -> Self;
}
impl<T: 'static, K: Eq + Hash + Clone + 'static, V: View> View for For<T, K, V> { /* a region */ }
```

## Behavior

### Dyn

- **VIEW-DYN-01:** `Dyn::new(f)` builds a region and, inside it, the view `f` returns, during the
  build. When a signal `f` read changes, `f` runs again at once (an effect) and the tree needs a
  frame; at the next `frame`, the region's old children are removed (their bindings stop, their
  cleanups run) and the new view is built in their place. Several changes before a frame
  rebuild once, with the latest view. A rebuild counts as one applied update in
  `FrameReport::applied`.
- **VIEW-DYN-02:** Ownership: what `f` creates while running (signals, effects, cleanups in the
  components it calls) belongs to that version of the content, not to the effect: it stays
  alive until the frame that replaces the content, where it is disposed after the old elements
  are removed. So the old content keeps working (its bindings may still run and read its
  signals) until it is replaced. A version replaced before any frame built it is disposed when
  it is replaced.
- **VIEW-DYN-03:** Removing the `Dyn` (or an ancestor) disposes its content, the pending version and
  its effect; a queued rebuild then does nothing.

### Show

- **VIEW-DYN-04:** `Show::new(when, then)` shows `then()`'s view while `when()` is true, and the
  `fallback()` view (or nothing) while false. It rebuilds only when the condition's value
  changes: signals read by `when` that don't flip it, and signals read while running `then` or
  `fallback`, cause no rebuild.

### For

- **VIEW-DYN-05:** `For::new(each, key, view)` builds `view(item)` for each item of `each()`, in
  order, inside a region, so the items are children of the region's render parent in that
  order. `view` runs untracked: signals it reads cause no rebuild.
- **VIEW-DYN-06:** When a signal `each` read changes, the list is reconciled at the next frame by
  key: items whose key was already present keep their element (same id, same scope and state,
  their old item value); items whose key is gone are removed (cleanups run); new keys are built
  with `view(item)`; the children, and the layout children, follow the new order. Several changes
  before a frame reconcile once, against the latest list.
- **VIEW-DYN-07:** Duplicate keys in one list: the first item with a key is used, later ones are
  skipped. An empty list leaves the region empty. Removing the `For` disposes every item.

## Performance and allocation

A rebuild costs what building the new content costs; `For` builds only new keys and moves the
others (one `LayoutTree::set_children` for the new order). Reconciliation is O(n) with a hash map
from key to element.

## Open questions

Resolved (2026-10-10, decided by the agent while the user was away; review):

1. **Content owned by a per-version scope** (VIEW-DYN-02), not by the effect, so old content
   never reads disposed signals between a change and the frame that replaces it.
2. **`For` items keep their first item value**; a kept key doesn't receive the new `T` (as in
   Solid's keyed `For`). Data that changes per item belongs in signals inside `T`.
3. **Rebuilds count in `FrameReport::applied`**, alongside prop values (the field's doc says
   "updates applied").
