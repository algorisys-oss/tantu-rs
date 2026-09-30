# 0001. Retained tree and fine-grained reactivity

- **Status:** Accepted. The disposed-handle consequence is superseded by
  [ADR 0007](0007-using-disposed-reactive-handles.md).
- **Date:** 2026-09-30

## Context

Tantu has to handle enterprise UIs: data grids with tens of thousands of rows, docking layouts,
large forms and full accessibility. Several things in such UIs need a stable identity that lasts
across frames:

- keyboard focus and IME composition state
- accessibility nodes, which screen readers track by id
- scroll positions and running animations
- incremental relayout and repaint of only what changed

Knots and Clay, which inspired parts of Tantu, are immediate mode: the UI is rebuilt from scratch
every frame. That is simple, but identity has to be reconstructed each frame, and the cost of a
frame grows with the size of the whole UI rather than the size of the change.

Rust has no garbage collector. A tree whose nodes hold closures that refer back into the tree
easily ends up as `Rc<RefCell<…>>` cycles that leak or panic at runtime.

## Decision

We will keep a **retained** UI with three trees, as Flutter does:

1. **View tree.** Short-lived, cheap values that describe the UI (Flutter's `Widget`s).
2. **Element tree.** Long-lived nodes that hold identity and state. The reconciler matches new
   views against existing elements, by key when one is given and by position otherwise.
3. **Render tree.** Objects that do layout, paint and hit-testing.

Elements and render objects live in a **generational arena** and refer to each other by id, not
by pointer.

State lives in **signals** from `tantu-reactive` (`Signal<T>`, `Memo<T>`, `Effect`), with
automatic dependency tracking. Writing a signal marks only the elements that read it as dirty; only
those rebuild, relayout and repaint. There is no whole-subtree `setState` rebuild by default.
Closures capture signal handles, which are `Copy` ids into the reactive runtime, so they don't hold
references into the tree.

## Consequences

- The work per frame depends on how much changed, not on how big the UI is.
- Focus, IME, accessibility, scroll positions and animations keep their identity across rebuilds.
- Ids instead of pointers avoid reference cycles between the tree and the reactive graph.
- We have to build and maintain a reconciler and a reactive runtime. Both are subtle. The
  reactive runtime needs tests for glitch-freedom (no observer sees a half-updated state),
  disposal, and leaks (count live nodes).
- A signal handle can outlive the scope that owns it. What happens when a disposed signal is used
  must be defined in the `tantu-reactive` spec, without undefined behavior or panics in release
  builds. (Superseded by [ADR 0007](0007-using-disposed-reactive-handles.md): reads of a disposed
  handle panic, `try_*` reads return `None`, writes are ignored.)
- Debugging "why did this rebuild?" needs tooling. The Phase 5 inspector shows the signal graph
  for this reason.

## Alternatives considered

- **Immediate mode** (egui, Knots, Clay). Rejected: no stable identity without extra bookkeeping,
  and the per-frame cost scales with the whole UI. That doesn't fit 50k-row grids or AccessKit's
  retained tree.
- **Elm architecture** (Iced): messages go to an `update` function and the whole view is rebuilt
  and diffed. Easy to reason about, but every change rebuilds the view, and passing messages
  through the whole app is heavy for large enterprise screens.
- **Flutter-style `setState`**, rebuilding the subtree under the stateful widget. Rebuilds more
  than needed and relies on `const` widgets and careful splitting to stay fast. Signals give
  finer-grained updates without that discipline.
