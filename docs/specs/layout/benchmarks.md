# Layout benchmarks

- **Status:** Implemented
- **Crate:** `tantu-layout` (benches and tests only; no library API)
- **Plan item:** Phase 2, "Layout benchmark: 10k render objects, target < 1 ms full layout"
- **Related:** [layout tree](tree.md), [single-child](single-child.md), [flex](flex.md),
  [text](text.md), [reactive benchmarks](../reactive/benchmarks.md) (same structure)

## Purpose

Measure the layout pass on trees the size of a busy screen (about 10 000 render objects), check
the Phase 2 target (a full layout under 1 ms in a release build), and give later work
(`tantu-view`, the data grid) baseline numbers. As with the reactive benchmarks, every scenario
also runs as an ordinary test that checks it does what its name says, so a benchmark can't
silently measure the wrong thing.

## Scope

In scope:

- Five scenarios: a full layout of a 10k-node dashboard, the same tree re-laid out after one
  leaf changes, a no-op pass, one flex with 10 000 children, and a grid of 10 000 paragraphs
  through `MeasureCache`.
- A `cargo bench -p tantu-layout` harness (criterion 0.7, as in `tantu-reactive`).
- Correctness tests for every scenario, run by `cargo test`.
- Baseline timings recorded in this spec.

Out of scope:

- Timing assertions in tests or CI (runners are too noisy; numbers are compared by hand).
- Real text shaping (the text scenario uses the fake measurer; `tantu-text` gets its own
  benchmarks).
- Paint and hit-testing (`tantu-view`).

## Public API

None. Scenarios live in `crates/tantu-layout/benches/scenarios.rs`, included as a module by the
bench target (`benches/layout.rs`) and the test target (`tests/bench_scenarios.rs`). Each
scenario owns its `LayoutTree`, is built by `new()` (setup, not timed) and driven by `step()`
(timed), and exposes counters for its test. Leaves are a small counting layout defined in the
scenarios file; every other node is a built-in layout.

```rust
/// A 10k-node dashboard: a column of 100 rows, each a padded row of 32 expanded cells, each
/// cell a padding around a centered sized box around a counting leaf. The step alternates the
/// root width between two values, so every node is laid out again.
pub struct FullLayout { /* ... */ }
/// The same tree; the step changes one leaf's size (through `LayoutTree::set`) and lays out.
pub struct RelayoutOneLeaf { /* ... */ }
/// The same tree; the step lays out again with nothing changed.
pub struct NoOpLayout { /* ... */ }
/// One row with 10 000 children (every other one expanded); the step alternates the width.
pub struct WideFlex { /* ... */ }
/// A column of 100 rows of 100 paragraphs (50 distinct strings) measured through a
/// `MeasureCache` over the fake measurer; the step alternates the width.
pub struct TextGrid { /* ... */ }
```

## Behavior

- **LAYOUT-BENCH-01:** `FullLayout` has between 9 000 and 11 000 nodes. Each step lays out every
  leaf exactly once (counted by the leaves) and gives the root the size of its constraints;
  the leaves end up at offsets inside their rows.
- **LAYOUT-BENCH-02:** Each `RelayoutOneLeaf` step lays out exactly one leaf, and the changed leaf
  has its new size afterwards.
- **LAYOUT-BENCH-03:** Each `NoOpLayout` step lays out no leaf and returns the same root size.
- **LAYOUT-BENCH-04:** Each `WideFlex` step lays out all 10 000 children, the expanded children
  share the free space (their widths add up to it within 0.1 %), and the flex reports no
  overflow.
- **LAYOUT-BENCH-05:** Each `TextGrid` step lays out all 10 000 paragraphs; after the first step,
  the inner measurer is called at most 100 times per step (2 per distinct string), and the rest
  are cache hits.

## Performance and allocation

The target, from `PLAN.md`: `FullLayout::step` under 1 ms (criterion median, release build,
the reference laptop). **Met.** Baselines (criterion median per `step()`, `cargo bench -p
tantu-layout`, Rust 1.87.0, Linux, Intel Core i5-1235U laptop, 2026-10-10):

| Scenario | Nodes | Median per step |
|---|---|---|
| `FullLayout` (every node laid out) | 9 801 | 635 µs (≈ 65 ns per node) |
| `RelayoutOneLeaf` | 9 801 | 7.5 µs |
| `NoOpLayout` | 9 801 | 29 ns |
| `WideFlex` (one row, 10 000 children) | 10 001 | 1.22 ms |
| `TextGrid` (10 000 paragraphs, cache hits) | 10 101 | 3.18 ms |

Notes:

- `RelayoutOneLeaf` touches only the changed path (leaf, its centered box, cell, row, row
  padding, root) and skips the other 9 795 nodes through the constraint cache: about 85× cheaper
  than a full pass.
- `TextGrid` costs about 320 ns per paragraph, almost all of it in `MeasureCache` lookups: the
  text is hashed with the standard library's SipHash, and the key is hashed again by the
  `HashMap`. A faster hasher (or hashing once) is the obvious optimization; it is left for the
  Phase 5 performance work, since real text shaping will dominate misses anyway.

When a number regresses by more than about 20 % on the same machine, find out why before
merging.

## Open questions

Resolved (2026-10-10):

1. **"Full layout"** means every node's constraints change on a tree that already exists (the
   width alternates); building the tree isn't timed. Agreed with the user.
2. **Leaves count their own layouts**; the tree gets no public counters. Decided by the agent
   (the user answered only the first question); review.
