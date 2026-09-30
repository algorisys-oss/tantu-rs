# Reactive micro-benchmarks

- **Status:** Implemented
- **Crate:** `tantu-reactive` (benches and tests only; no library API)
- **Plan item:** Phase 0, "Reactive micro-benchmarks"
- **Related:** [signals](signals.md) (the API under test),
  [ADR 0001](../../adr/0001-retained-tree-and-fine-grained-reactivity.md)

## Purpose

Measure the cost of the reactive runtime on the graph shapes a UI produces, so regressions show up
and later work (the view layer, the data grid) has baseline numbers to design against. The
benchmarks are for maintainers; apps never see them.

Each benchmark runs a **scenario**: a small reactive graph plus one repeatable step. The same
scenarios are also run by ordinary tests that check they do what their name says. A benchmark
that silently measures the wrong thing (an effect that never runs, a batch that isn't one) is
worse than none.

## Scope

In scope:

- Eight scenarios covering node churn, plain reads and writes, fan-out, fan-in, deep chains,
  diamonds, memo cut-off and scope churn.
- A `cargo bench -p tantu-reactive` harness (criterion) that times each scenario's step at a
  few sizes.
- Correctness tests for every scenario, run by `cargo test`.
- Baseline timings recorded in this spec.

Out of scope:

- Timing assertions in tests or CI. Shared CI runners are too noisy; numbers are compared by hand.
- Comparisons with other reactive libraries.
- Layout benchmarks (Phase 2, `tantu-layout`).

## Public API

None. Scenarios live in `crates/tantu-reactive/benches/scenarios.rs` and are included as a module
by both the bench target (`benches/reactive.rs`) and the test target (`tests/bench_scenarios.rs`).
Each scenario is a struct that owns its `Runtime`, built by `new(n)` (setup, not timed) and
driven by `step()` (timed). Scenarios expose counters so tests can check them.

```rust
/// Creates `n` signals, `n` memos and `n` effects in a scope, then disposes the scope.
pub struct CreateDispose { /* ... */ }
/// `n` get/set round trips on one signal with no subscribers.
pub struct GetSet { /* ... */ }
/// One signal read by `n` effects.
pub struct FanOut { /* ... */ }
/// `n` signals summed by one memo, read by one effect; the step writes all of them in a batch.
pub struct FanIn { /* ... */ }
/// A chain of `n` memos, each adding 1 to the previous, with an effect on the last one.
pub struct DeepChain { /* ... */ }
/// One signal, `n` memos reading it, one memo summing those, one effect on the sum.
pub struct Diamond { /* ... */ }
/// One signal, one memo that ignores most changes, `n` effects on the memo.
pub struct Cutoff { /* ... */ }
/// Mounts and unmounts `n` child scopes, each with a signal, memo, effect and cleanup.
pub struct ScopeChurn { /* ... */ }
```

## Behavior

Each rule describes one scenario and what its test checks after `new(n)` and some `step()`s.

- **REACTIVE-BENCH-01:** `CreateDispose::step` creates `n` signals, `n` memos (each reading its
  signal) and `n` effects (each reading its memo) inside a fresh scope, then disposes the scope.
  Every effect runs once per step, and after each step `Runtime::node_count` is back to its value
  before the step.
- **REACTIVE-BENCH-02:** `GetSet::step` does `n` round trips of `set(get() + 1)` on a signal with
  no subscribers. After `k` steps the signal holds `k * n`.
- **REACTIVE-BENCH-03:** `FanOut::step` writes the source once. Each of the `n` effects runs
  exactly once per step and sees the new value.
- **REACTIVE-BENCH-04:** `FanIn::step` writes all `n` signals inside one `batch`. The effect runs
  exactly once per step, and the memo equals the sum of the signals.
- **REACTIVE-BENCH-05:** `DeepChain::step` writes the head of the chain. Every memo recomputes
  exactly once per step, and the effect sees `head + n`. Chains of at least 500 memos work on the
  default test thread stack (2 MiB) in a debug build.
- **REACTIVE-BENCH-06:** `Diamond::step` writes the source once. Each of the `n` middle memos and
  the sum memo recompute exactly once per step, and the effect runs exactly once and never sees a
  mix of old and new values (glitch-freedom).
- **REACTIVE-BENCH-07:** `Cutoff::step` writes the source with a value the memo maps to its
  current output. The memo recomputes once and none of the `n` effects run.
- **REACTIVE-BENCH-08:** `ScopeChurn::step` mounts `n` child scopes under a parent, then disposes
  the parent. Every child's effect runs once and every cleanup runs once per step, and after each
  step `Runtime::node_count` is back to its value before the step.

## Performance and allocation

The sizes benchmarked are `n` = 10, 100 and 1 000 (100 000 for `GetSet`). Baselines (criterion
median per `step()`, `cargo bench`, Rust 1.85.1, Linux, Intel Core i5-1235U laptop, 2026-09-30):

| Scenario | n = 10 | n = 100 | n = 1 000 |
|---|---|---|---|
| `CreateDispose` (n signals + memos + effects) | 6.1 µs | 68 µs | 881 µs |
| `GetSet` (n = 100 000 round trips) | | | 2.9 ms (29 ns each) |
| `FanOut` | 0.77 µs | 7.2 µs | 72 µs |
| `FanIn` | 0.50 µs | 3.5 µs | 34 µs |
| `DeepChain` | 1.1 µs | 10.8 µs | 108 µs |
| `Diamond` | 1.2 µs | 10.1 µs | 102 µs |
| `Cutoff` | 0.25 µs | 1.6 µs | 16 µs |
| `ScopeChurn` | 8.2 µs | 86 µs | 853 µs |

Every scenario scales linearly in `n`. The first run of these benchmarks found `FanOut`, `FanIn`
and `Diamond` growing about 33× from 100 to 1 000 (for example `FanOut` at 1 000: 454 µs), because
every re-run unsubscribed from and re-subscribed to all of its sources. Dependency tracking was
changed to leave subscriptions alone when a re-run reads the same sources (spec `signals.md`,
Performance). That made node creation about 10 % slower, which is the accepted trade-off.

No targets are enforced in Phase 0. When a number regresses by more than about 20 % on the same
machine, find out why before merging.

## Open questions

- Nested memos recurse, so very deep chains are limited by the thread stack. Measured on a 2 MiB
  stack: about 1 200 levels in a debug build (~1.7 KB per level) and about 4 000 in release
  (~450 B per level); the 8 MiB main thread allows four times that. REACTIVE-BENCH-05 makes 500
  levels in debug the floor (it first said 1 000, which debug builds miss). Real memo chains are a
  few dozen levels deep, so an iterative update is deferred until a real widget tree needs more.
