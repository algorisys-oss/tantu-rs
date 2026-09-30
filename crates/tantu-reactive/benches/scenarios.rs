//! Benchmark scenarios, spec `docs/specs/reactive/benchmarks.md`.
//!
//! Shared by the bench target (`benches/reactive.rs`) and the test target
//! (`tests/bench_scenarios.rs`) through `#[path]`, so every scenario that is timed is also checked.
//! `new(n)` builds the graph (not timed); `step()` is the timed part.

#![allow(dead_code)]

use std::cell::Cell;
use std::rc::Rc;

use tantu_reactive::{Memo, Runtime, Signal};

/// Adds one to a shared counter.
fn bump(counter: &Cell<usize>) {
    counter.set(counter.get() + 1);
}

/// REACTIVE-BENCH-01: creates `n` signals, memos and effects in a scope, then disposes it.
pub struct CreateDispose {
    rt: Runtime,
    n: usize,
    effect_runs: Rc<Cell<usize>>,
}

impl CreateDispose {
    /// Builds the scenario.
    pub fn new(n: usize) -> Self {
        todo!("CreateDispose::new({n})")
    }

    /// One create-and-dispose round.
    pub fn step(&mut self) {
        todo!()
    }

    /// Total effect runs so far.
    pub fn effect_runs(&self) -> usize {
        self.effect_runs.get()
    }

    /// Live nodes in the runtime.
    pub fn node_count(&self) -> usize {
        self.rt.node_count()
    }
}

/// REACTIVE-BENCH-02: `n` get/set round trips on a signal with no subscribers.
pub struct GetSet {
    rt: Runtime,
    n: usize,
    value: Signal<u64>,
}

impl GetSet {
    /// Builds the scenario.
    pub fn new(n: usize) -> Self {
        todo!("GetSet::new({n})")
    }

    /// `n` round trips.
    pub fn step(&mut self) {
        todo!()
    }

    /// The signal's value.
    pub fn value(&self) -> u64 {
        self.rt.enter(|| self.value.get_untracked())
    }
}

/// REACTIVE-BENCH-03: one signal read by `n` effects.
pub struct FanOut {
    rt: Runtime,
    source: Signal<u64>,
    next: u64,
    effect_runs: Rc<Cell<usize>>,
    seen: Rc<Vec<Cell<u64>>>,
}

impl FanOut {
    /// Builds the scenario.
    pub fn new(n: usize) -> Self {
        todo!("FanOut::new({n})")
    }

    /// Writes the source once.
    pub fn step(&mut self) {
        todo!()
    }

    /// Total effect runs so far.
    pub fn effect_runs(&self) -> usize {
        self.effect_runs.get()
    }

    /// The source's value.
    pub fn source(&self) -> u64 {
        self.rt.enter(|| self.source.get_untracked())
    }

    /// The value each effect saw last.
    pub fn seen(&self) -> Vec<u64> {
        self.seen.iter().map(Cell::get).collect()
    }
}

/// REACTIVE-BENCH-04: `n` signals summed by one memo, read by one effect; batched writes.
pub struct FanIn {
    rt: Runtime,
    signals: Vec<Signal<u64>>,
    sum: Memo<u64>,
    next: u64,
    effect_runs: Rc<Cell<usize>>,
    seen: Rc<Cell<u64>>,
}

impl FanIn {
    /// Builds the scenario.
    pub fn new(n: usize) -> Self {
        todo!("FanIn::new({n})")
    }

    /// Writes every signal inside one batch.
    pub fn step(&mut self) {
        todo!()
    }

    /// Total effect runs so far.
    pub fn effect_runs(&self) -> usize {
        self.effect_runs.get()
    }

    /// The sum the effect saw last.
    pub fn seen(&self) -> u64 {
        self.seen.get()
    }

    /// The sum of the signals, computed directly.
    pub fn expected(&self) -> u64 {
        self.rt
            .enter(|| self.signals.iter().map(|s| s.get_untracked()).sum())
    }
}

/// REACTIVE-BENCH-05: a chain of `n` memos with an effect on the last one.
pub struct DeepChain {
    rt: Runtime,
    n: usize,
    head: Signal<u64>,
    next: u64,
    recomputes: Rc<Cell<usize>>,
    effect_runs: Rc<Cell<usize>>,
    seen: Rc<Cell<u64>>,
}

impl DeepChain {
    /// Builds the scenario.
    pub fn new(n: usize) -> Self {
        todo!("DeepChain::new({n})")
    }

    /// Writes the head of the chain.
    pub fn step(&mut self) {
        todo!()
    }

    /// Total memo recomputations so far.
    pub fn recomputes(&self) -> usize {
        self.recomputes.get()
    }

    /// Total effect runs so far.
    pub fn effect_runs(&self) -> usize {
        self.effect_runs.get()
    }

    /// The value the effect saw last.
    pub fn seen(&self) -> u64 {
        self.seen.get()
    }

    /// The head's value.
    pub fn head(&self) -> u64 {
        self.rt.enter(|| self.head.get_untracked())
    }
}

/// REACTIVE-BENCH-06: one signal, `n` memos on it, one memo summing them, one effect.
pub struct Diamond {
    rt: Runtime,
    n: usize,
    source: Signal<u64>,
    next: u64,
    middle_recomputes: Rc<Cell<usize>>,
    sum_recomputes: Rc<Cell<usize>>,
    effect_runs: Rc<Cell<usize>>,
    glitches: Rc<Cell<usize>>,
}

impl Diamond {
    /// Builds the scenario.
    pub fn new(n: usize) -> Self {
        todo!("Diamond::new({n})")
    }

    /// Writes the source once.
    pub fn step(&mut self) {
        todo!()
    }

    /// Total recomputations of the middle memos so far.
    pub fn middle_recomputes(&self) -> usize {
        self.middle_recomputes.get()
    }

    /// Total recomputations of the sum memo so far.
    pub fn sum_recomputes(&self) -> usize {
        self.sum_recomputes.get()
    }

    /// Total effect runs so far.
    pub fn effect_runs(&self) -> usize {
        self.effect_runs.get()
    }

    /// Effect runs that saw a sum inconsistent with the source.
    pub fn glitches(&self) -> usize {
        self.glitches.get()
    }
}

/// REACTIVE-BENCH-07: one signal, one memo that ignores most changes, `n` effects on the memo.
pub struct Cutoff {
    rt: Runtime,
    source: Signal<u64>,
    next: u64,
    memo_recomputes: Rc<Cell<usize>>,
    effect_runs: Rc<Cell<usize>>,
}

impl Cutoff {
    /// Builds the scenario.
    pub fn new(n: usize) -> Self {
        todo!("Cutoff::new({n})")
    }

    /// Writes the source with a value the memo maps to its current output.
    pub fn step(&mut self) {
        todo!()
    }

    /// Total memo recomputations so far.
    pub fn memo_recomputes(&self) -> usize {
        self.memo_recomputes.get()
    }

    /// Total effect runs so far.
    pub fn effect_runs(&self) -> usize {
        self.effect_runs.get()
    }
}

/// REACTIVE-BENCH-08: mounts and unmounts `n` child scopes with a signal, memo, effect and cleanup.
pub struct ScopeChurn {
    rt: Runtime,
    n: usize,
    effect_runs: Rc<Cell<usize>>,
    cleanups: Rc<Cell<usize>>,
}

impl ScopeChurn {
    /// Builds the scenario.
    pub fn new(n: usize) -> Self {
        todo!("ScopeChurn::new({n})")
    }

    /// Mounts `n` child scopes under a parent, then disposes the parent.
    pub fn step(&mut self) {
        todo!()
    }

    /// Total effect runs so far.
    pub fn effect_runs(&self) -> usize {
        self.effect_runs.get()
    }

    /// Total cleanups run so far.
    pub fn cleanups(&self) -> usize {
        self.cleanups.get()
    }

    /// Live nodes in the runtime.
    pub fn node_count(&self) -> usize {
        self.rt.node_count()
    }
}
