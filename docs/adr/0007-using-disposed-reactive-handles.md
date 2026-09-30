# 0007. Using disposed reactive handles

- **Status:** Accepted (decided in autopilot while finishing Phase 0; open to review)
- **Date:** 2026-09-30
- **Supersedes:** the disposed-handle consequence of
  [ADR 0001](0001-retained-tree-and-fine-grained-reactivity.md) ("without undefined behavior or
  panics in release builds"). The rest of ADR 0001 stands.

## Context

Signal, memo and effect handles are `Copy` ids into the reactive runtime (ADR 0001). A handle can
outlive the scope that owns its node, for example when a closure that captured it runs after its
element was removed. ADR 0001 asked the `tantu-reactive` spec to define what happens then,
"without undefined behavior or panics in release builds".

That can't be met for reads. `Signal::get(self) -> T` has to return a `T`, and once the node is
disposed there is no value left to return. Keeping values alive after disposal would leak, and
requiring `T: Default` to fabricate one would hide bugs behind plausible-looking data.

## Decision

We will treat the three kinds of use differently:

- **Reads** (`get`, `with`) of a disposed handle panic in every build, with a message that says
  the node was disposed. This follows `RefCell::borrow` and slice indexing: a use-after-dispose is
  a bug in the caller, reported where it happens.
- **Checked reads** (`try_get`, `try_with`) return `None` instead. Code that can legitimately run
  after its scope is gone (timers, async completions, callbacks held elsewhere) uses these.
- **Writes** (`set`, `update`) and `dispose` on a disposed handle do nothing. A late write has no
  one to notify, so dropping it is harmless.

A handle used outside its own runtime is treated as disposed.

There is never undefined behavior: `tantu-reactive` has no `unsafe`, and a stale id can't
resolve to a newer node because the arena is generational (spec `docs/specs/core/arena.md`).

## Consequences

- Ordinary reactive code keeps the plain `count.get()` style from `AGENTS.md`.
- Widgets dispose their effects and callbacks together with their scope, so in normal use nothing
  reads a disposed signal. The panic only surfaces bugs.
- Code that may outlive its scope has to use `try_*`. The API docs say so.

## Alternatives considered

- **`get` returns `Option<T>`.** No panics, but every read in every view closure needs an
  `unwrap` or a default, which is worse for app code than one documented panic.
- **Keep the last value after disposal.** Leaks until the slot is reused, and still has no value
  once it is.
- **`T: Default` and return the default in release builds.** Hides the bug and shows wrong data.
- **Panic only in debug builds.** Release builds would still need a value to return.
