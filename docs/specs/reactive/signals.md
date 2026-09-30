# Signals, memos and effects

- **Status:** Implemented
- **Crate:** `tantu-reactive`
- **Plan item:** Phase 0, "`tantu-reactive`: `Signal`, `Memo`, `Effect`, batch, scoped disposal,
  and tests for glitch-freedom"
- **Related:** [ADR 0001](../../adr/0001-retained-tree-and-fine-grained-reactivity.md) (retained
  tree + fine-grained signals), [ADR 0007](../../adr/0007-using-disposed-reactive-handles.md)
  (disposed handles), [arena](../core/arena.md) (node storage)

## Purpose

State in a Tantu app lives in signals. Views read signals inside closures
(`Text::new(move || format!("Count: {}", count.get()))`), and the runtime records which
computation read which signal. Writing a signal re-runs only the computations that depend on it.
That is what lets `tantu-view` rebuild, relayout and repaint just the elements that changed.

App developers use `signal`, `memo`, `effect` and `batch`. `tantu-view` uses `Runtime` (one per
app), `Scope` (one per element, so removing an element disposes everything it created) and
`on_cleanup`.

## Scope

In scope:

- `Runtime`: owns the reactive graph. Entered explicitly; one per app or window.
- `Signal<T>`: a readable and writable value.
- `Memo<T>`: a lazily computed, cached value derived from other signals and memos, with equality
  cut-off.
- `Effect`: a side-effecting computation that re-runs when what it read changes.
- Automatic, dynamic dependency tracking; `untrack` and `*_untracked` reads.
- `batch`: group writes so effects run once, at the end.
- Ownership and disposal: `Scope`, nodes owned by the computation that created them,
  `on_cleanup`, what happens to disposed handles.
- Glitch-freedom: no computation ever sees a mix of old and new values.

Out of scope:

- Deferring effects to the next frame. Effects run synchronously when the write (or the outermost
  batch) completes. `tantu-view` may add a frame scheduler on top later.
- Multi-threading. A `Runtime` and its nodes live on one thread; `Runtime` is not `Send`.
- Async resources, stores/nested signals, split read/write handles, `set_if_changed`. Later, if
  needed.
- Recovering from a panic inside a user closure. The panic propagates; the runtime is not
  poisoned (no internal borrow is held while user code runs), but the node that panicked may be
  left half-updated.
- The order in which several effects triggered by the same write run. It is deterministic but not
  specified.

## Public API

Crate root of `tantu-reactive`.

```rust
/// The reactive graph: every signal, memo, effect and scope of one app or window.
///
/// Not `Send`. Handles resolve against the runtime that is current on this thread, set by
/// [`Runtime::enter`]. Dropping the runtime disposes every node it still holds.
pub struct Runtime { /* private */ }

impl Runtime {
    /// An empty runtime.
    pub fn new() -> Runtime;
    /// Runs `f` with this runtime as the current one on this thread, then restores the previous
    /// current runtime (also if `f` panics). Calls may nest.
    pub fn enter<R>(&self, f: impl FnOnce() -> R) -> R;
    /// Number of live nodes (signals, memos, effects and scopes). For leak tests.
    pub fn node_count(&self) -> usize;
}
impl Default for Runtime { /* Runtime::new() */ }

/// A readable, writable value. `Copy`, whatever `T` is.
pub struct Signal<T> { /* private */ }

/// Creates a signal in the current runtime, owned by the current owner.
/// Panics if no runtime is current.
pub fn signal<T: 'static>(value: T) -> Signal<T>;

impl<T: 'static> Signal<T> {
    /// Same as [`signal`].
    pub fn new(value: T) -> Signal<T>;
    /// A clone of the value. Subscribes the running computation. Panics if disposed.
    pub fn get(self) -> T where T: Clone;
    /// Calls `f` with a reference to the value. Subscribes. Panics if disposed.
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R;
    /// `get` without subscribing.
    pub fn get_untracked(self) -> T where T: Clone;
    /// `with` without subscribing.
    pub fn with_untracked<R>(self, f: impl FnOnce(&T) -> R) -> R;
    /// `get`, or `None` if disposed.
    pub fn try_get(self) -> Option<T> where T: Clone;
    /// `with`, or `None` if disposed.
    pub fn try_with<R>(self, f: impl FnOnce(&T) -> R) -> Option<R>;
    /// Replaces the value and notifies dependents. Does nothing if disposed.
    pub fn set(self, value: T);
    /// Mutates the value in place and notifies dependents. Does nothing if disposed.
    pub fn update(self, f: impl FnOnce(&mut T));
    /// Disposes the signal and drops its value. Does nothing if already disposed.
    pub fn dispose(self);
    /// True once disposed (or when used outside its runtime).
    pub fn is_disposed(self) -> bool;
}

/// A cached value computed from other signals and memos. `Copy`, whatever `T` is.
pub struct Memo<T> { /* private */ }

/// Creates a memo. `f` runs lazily: on the first read, then on reads after a dependency changed.
/// Panics if no runtime is current.
pub fn memo<T: PartialEq + 'static>(f: impl FnMut() -> T + 'static) -> Memo<T>;

impl<T: PartialEq + 'static> Memo<T> {
    /// Same as [`memo`].
    pub fn new(f: impl FnMut() -> T + 'static) -> Memo<T>;
    /// Same as the `Signal` methods of the same name.
    pub fn get(self) -> T where T: Clone;
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R;
    pub fn get_untracked(self) -> T where T: Clone;
    pub fn with_untracked<R>(self, f: impl FnOnce(&T) -> R) -> R;
    pub fn try_get(self) -> Option<T> where T: Clone;
    pub fn try_with<R>(self, f: impl FnOnce(&T) -> R) -> Option<R>;
    /// Disposes the memo, its computation and what it owns.
    pub fn dispose(self);
    pub fn is_disposed(self) -> bool;
}

/// A computation run for its side effects. `Copy`.
pub struct Effect { /* private */ }

/// Creates an effect and runs it once, right away. It re-runs whenever something it read in its
/// latest run changes. Panics if no runtime is current.
pub fn effect(f: impl FnMut() + 'static) -> Effect;

impl Effect {
    /// Same as [`effect`].
    pub fn new(f: impl FnMut() + 'static) -> Effect;
    /// Stops the effect for good, disposes what it owns and runs its cleanups.
    pub fn dispose(self);
    pub fn is_disposed(self) -> bool;
}

/// An owner for nodes, disposed as a unit (for example one per element). `Copy`.
pub struct Scope { /* private */ }

impl Scope {
    /// A new scope owned by the current owner. Panics if no runtime is current.
    pub fn new() -> Scope;
    /// Runs `f` with this scope as the owner of every node created inside it.
    /// Panics if the scope is disposed.
    pub fn run<R>(self, f: impl FnOnce() -> R) -> R;
    /// Disposes everything the scope owns, runs its cleanups, then the scope itself.
    pub fn dispose(self);
    pub fn is_disposed(self) -> bool;
}

/// Runs `f`; effects triggered by writes inside it run once, when the outermost `batch` returns.
pub fn batch<R>(f: impl FnOnce() -> R) -> R;
/// Runs `f` without subscribing the running computation to anything `f` reads.
pub fn untrack<R>(f: impl FnOnce() -> R) -> R;
/// Registers `f` to run when the current owner is disposed or, for an effect or memo, before it
/// re-runs. Panics if no runtime is current.
pub fn on_cleanup(f: impl FnOnce() + 'static);
```

The **current owner** is the innermost of: the scope in `Scope::run`, the effect or memo whose
computation is running, or else the runtime itself. The **running computation** is the effect or
memo whose closure is executing, if any.

## Behavior

Runtime and handles

- **REACTIVE-SIG-01:** `Signal<T>`, `Memo<T>`, `Effect` and `Scope` are `Copy + 'static` for any
  `T: 'static` (no `T: Copy` bound) and at most 16 bytes.
- **REACTIVE-SIG-02:** `signal`, `memo`, `effect`, `Scope::new` and `on_cleanup` panic with a
  message containing "no current Runtime" when no runtime is current. `batch` and `untrack` just
  call `f` then.
- **REACTIVE-SIG-03:** `Runtime::enter` makes the runtime current for `f` and restores the
  previously current runtime (or none) afterwards, also when `f` panics. Calls nest, with the same
  or with other runtimes.
- **REACTIVE-SIG-04:** A handle used while its own runtime is not current behaves as disposed
  (REACTIVE-SIG-21): `is_disposed` is true, reads panic, `try_*` return `None`, writes are
  ignored. It never reaches a node of another runtime.
- **REACTIVE-SIG-05:** `node_count` is the number of live signals, memos, effects and scopes.
  Dropping a `Runtime` disposes all its nodes as `Scope::dispose` would: cleanups run, values and
  closures are dropped.

Signals

- **REACTIVE-SIG-06:** `get` and `with` see the latest value. `set` replaces it, `update` changes
  it in place.
- **REACTIVE-SIG-07:** Every `set` and `update` notifies dependents, even when the new value
  equals the old one (signals need no `PartialEq`; memos provide the cut-off).
- **REACTIVE-SIG-08:** A read with `get`, `with`, `try_get` or `try_with` inside a running
  computation subscribes that computation. `get_untracked`, `with_untracked` and any read inside
  `untrack` don't subscribe.

Memos

- **REACTIVE-SIG-09:** A memo is lazy. Its closure doesn't run at creation; it runs on the first
  read, and later only on a read after a dependency changed. Repeated reads with no change in
  between run it once.
- **REACTIVE-SIG-10:** When a memo recomputes to a value equal (`PartialEq`) to its previous one,
  it keeps the previous value and its dependents don't re-run.
- **REACTIVE-SIG-11:** A memo that reads itself, directly or through other memos, panics with a
  message containing "cycle".

Effects

- **REACTIVE-SIG-12:** `effect` runs its closure once, synchronously, before returning, even
  inside a `batch`.
- **REACTIVE-SIG-13:** An effect re-runs synchronously, before the write returns, when a signal
  or memo it read in its latest run changes (outside a batch). Writes to anything it didn't read
  don't run it.
- **REACTIVE-SIG-14:** Dependencies are dynamic: after each run, an effect or memo depends on
  exactly what that run read. A dependency read only in an earlier run no longer triggers it.
- **REACTIVE-SIG-15:** Glitch-free. After a write, every affected memo and effect runs at most
  once, after all its affected dependencies are up to date, so it never sees a mix of old and new
  values. (Diamond: `a = s + 1`, `b = s * 2`, an effect reading `a` and `b` runs once per write
  of `s` and only ever sees pairs that belong together.)
- **REACTIVE-SIG-16:** A write made inside an effect doesn't run other effects inside it. Effects
  it triggers run after the current effect returns, before the outer write returns. An effect
  that writes a signal it reads runs again after its current run, once per such write.

Batching

- **REACTIVE-SIG-17:** Inside `batch`, writes take effect at once (reads of signals and memos see
  the new values), but effects wait until the outermost `batch` returns and then each affected
  effect runs once. `batch` returns `f`'s result.

Ownership and disposal

- **REACTIVE-SIG-18:** Every node is owned by the owner that was current when it was created.
  Disposing a scope, effect or memo disposes every node it owns, recursively (nested scopes,
  their signals, memos and effects), before the owner itself.
- **REACTIVE-SIG-19:** Before an effect or memo re-runs, the nodes it created in its previous run
  are disposed and its cleanups run.
- **REACTIVE-SIG-20:** An `on_cleanup` callback runs exactly once: when its owner is disposed,
  or before its owner (an effect or memo) re-runs. An owner's cleanups run after its owned nodes
  are disposed, in registration order. Callbacks registered at the top level run when the
  `Runtime` is dropped.
- **REACTIVE-SIG-21:** On a disposed handle, `is_disposed` is true; `get`, `with` and the
  `_untracked` reads panic with a message containing "disposed"; `try_get` and `try_with` return
  `None`; `set`, `update` and `dispose` do nothing (a value passed to `set` is dropped).
  `Scope::run` on a disposed scope panics with a message containing "disposed". A disposed effect
  never runs again.
- **REACTIVE-SIG-22:** No leaks. A node's value, closure and cleanups are dropped exactly once,
  at disposal. After creating and disposing any number of nodes in a scope, `node_count` returns
  to what it was before.

Misuse

- **REACTIVE-SIG-23:** Writing a signal inside its own `with` (or `update`) closure panics with a
  message containing "borrowed", as `RefCell` does. It never deadlocks or corrupts the value. (A
  memo forced to recompute inside its own `with` panics the same way.)

## Performance and allocation

- A write visits only the nodes downstream of the written signal. Unrelated nodes are not
  touched.
- Each affected memo and effect runs at most once per write or batch (REACTIVE-SIG-15, -17).
- Reads are O(1) plus a subscription check that is linear in the reader's dependency count
  (small in practice).
- Timings are measured by the reactive micro-benchmarks (spec `benchmarks.md`), not asserted here.

## Design notes

- **Algorithm.** Push-pull with three node states (clean, check, dirty), as in Reactively and
  Leptos: a write marks direct dependents dirty and everything further down "check" and queues the
  effects it reaches. Effects are then brought up to date by pulling: a "check" node first updates
  its memo sources and recomputes only if one of them actually changed. This is what makes
  updates glitch-free and memo cut-off work.
- **Storage.** Nodes live in a `tantu_core::Arena` inside the runtime; handles are an arena `Id`
  plus the runtime's id. Values sit behind their own `RefCell`, so no borrow of the graph is held
  while user code runs.
- **Current runtime.** A thread-local stack records which runtime is entered, so view code can
  write `signal(0)` without passing a context around. This is not hidden global state: every
  runtime is an explicit value, several can exist (one per window, one per test), and nothing
  works outside `Runtime::enter`.

## Open questions

Resolved (2026-09-30, decided in autopilot while finishing Phase 0):

1. **Implicit context or explicit `cx` parameter?** Implicit, through `Runtime::enter`, to keep
   the `AGENTS.md` authoring style (`let count = signal(0);`).
2. **Disposed handles.** Reads panic, `try_*` reads return `None`, writes are ignored; see ADR
   0007.
3. **Synchronous or frame-deferred effects?** Synchronous for now. A frame scheduler belongs to
   `tantu-view` when it exists.
4. **Equality check on `Signal::set`?** No. It would force `PartialEq` on every signal; memos do
   the cut-off.
