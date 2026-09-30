//! Benchmark scenarios, spec `docs/specs/reactive/benchmarks.md`.
//!
//! Shared by the bench target (`benches/reactive.rs`) and the test target
//! (`tests/bench_scenarios.rs`) through `#[path]`, so every scenario that is timed is also checked.
//! `new(n)` builds the graph (not timed); `step()` is the timed part.

#![allow(dead_code)]

use std::cell::Cell;
use std::rc::Rc;

use tantu_reactive::{Memo, Runtime, Scope, Signal, batch, effect, memo, on_cleanup, signal};

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
        CreateDispose {
            rt: Runtime::new(),
            n,
            effect_runs: Rc::default(),
        }
    }

    /// One create-and-dispose round.
    pub fn step(&mut self) {
        let (n, runs) = (self.n, &self.effect_runs);
        self.rt.enter(|| {
            let scope = Scope::new();
            scope.run(|| {
                for i in 0..n {
                    let s = signal(i);
                    let m = memo(move || s.get() + 1);
                    let runs = runs.clone();
                    effect(move || {
                        m.get();
                        bump(&runs);
                    });
                }
            });
            scope.dispose();
        });
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
        let rt = Runtime::new();
        let value = rt.enter(|| signal(0));
        GetSet { rt, n, value }
    }

    /// `n` round trips.
    pub fn step(&mut self) {
        let (n, value) = (self.n, self.value);
        self.rt.enter(|| {
            for _ in 0..n {
                value.set(value.get() + 1);
            }
        });
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
        let rt = Runtime::new();
        let effect_runs = Rc::new(Cell::new(0));
        let seen: Rc<Vec<Cell<u64>>> = Rc::new((0..n).map(|_| Cell::new(0)).collect());
        let source = rt.enter(|| {
            let source = signal(0);
            for i in 0..n {
                let (runs, seen) = (effect_runs.clone(), seen.clone());
                effect(move || {
                    seen[i].set(source.get());
                    bump(&runs);
                });
            }
            source
        });
        FanOut {
            rt,
            source,
            next: 1,
            effect_runs,
            seen,
        }
    }

    /// Writes the source once.
    pub fn step(&mut self) {
        let (source, next) = (self.source, self.next);
        self.rt.enter(|| source.set(next));
        self.next += 1;
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
        let rt = Runtime::new();
        let effect_runs = Rc::new(Cell::new(0));
        let seen = Rc::new(Cell::new(0));
        let (signals, sum) = rt.enter(|| {
            let signals: Vec<Signal<u64>> = (0..n).map(|_| signal(0)).collect();
            let inputs = signals.clone();
            let sum = memo(move || inputs.iter().map(|s| s.get()).sum::<u64>());
            let (runs, seen) = (effect_runs.clone(), seen.clone());
            effect(move || {
                seen.set(sum.get());
                bump(&runs);
            });
            (signals, sum)
        });
        FanIn {
            rt,
            signals,
            sum,
            next: 1,
            effect_runs,
            seen,
        }
    }

    /// Writes every signal inside one batch.
    pub fn step(&mut self) {
        let (signals, next) = (&self.signals, self.next);
        self.rt.enter(|| {
            batch(|| {
                for s in signals {
                    s.set(next);
                }
            });
        });
        self.next += 1;
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
        let rt = Runtime::new();
        let recomputes = Rc::new(Cell::new(0));
        let effect_runs = Rc::new(Cell::new(0));
        let seen = Rc::new(Cell::new(0));
        let head = rt.enter(|| {
            let head = signal(0);
            let first = {
                let recomputes = recomputes.clone();
                memo(move || {
                    bump(&recomputes);
                    head.get() + 1
                })
            };
            let last = (1..n).fold(first, |prev, _| {
                let recomputes = recomputes.clone();
                memo(move || {
                    bump(&recomputes);
                    prev.get() + 1
                })
            });
            let (runs, seen) = (effect_runs.clone(), seen.clone());
            effect(move || {
                seen.set(last.get());
                bump(&runs);
            });
            head
        });
        DeepChain {
            rt,
            n,
            head,
            next: 1,
            recomputes,
            effect_runs,
            seen,
        }
    }

    /// Writes the head of the chain.
    pub fn step(&mut self) {
        let (head, next) = (self.head, self.next);
        self.rt.enter(|| head.set(next));
        self.next += 1;
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
        let rt = Runtime::new();
        let middle_recomputes = Rc::new(Cell::new(0));
        let sum_recomputes = Rc::new(Cell::new(0));
        let effect_runs = Rc::new(Cell::new(0));
        let glitches = Rc::new(Cell::new(0));
        let source = rt.enter(|| {
            let source = signal(0u64);
            let middle: Vec<Memo<u64>> = (0..n as u64)
                .map(|i| {
                    let recomputes = middle_recomputes.clone();
                    memo(move || {
                        bump(&recomputes);
                        source.get() + i
                    })
                })
                .collect();
            let sum = {
                let recomputes = sum_recomputes.clone();
                memo(move || {
                    bump(&recomputes);
                    middle.iter().map(|m| m.get()).sum::<u64>()
                })
            };
            // Sum of `source + i` for i in 0..n.
            let n = n as u64;
            let expected = move |source: u64| n * source + n * n.saturating_sub(1) / 2;
            let (runs, glitches) = (effect_runs.clone(), glitches.clone());
            effect(move || {
                if sum.get() != expected(source.get()) {
                    bump(&glitches);
                }
                bump(&runs);
            });
            source
        });
        Diamond {
            rt,
            n,
            source,
            next: 1,
            middle_recomputes,
            sum_recomputes,
            effect_runs,
            glitches,
        }
    }

    /// Writes the source once.
    pub fn step(&mut self) {
        let (source, next) = (self.source, self.next);
        self.rt.enter(|| source.set(next));
        self.next += 1;
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
        let rt = Runtime::new();
        let memo_recomputes = Rc::new(Cell::new(0));
        let effect_runs = Rc::new(Cell::new(0));
        let source = rt.enter(|| {
            let source = signal(0u64);
            let even = {
                let recomputes = memo_recomputes.clone();
                memo(move || {
                    bump(&recomputes);
                    source.get() % 2 == 0
                })
            };
            for _ in 0..n {
                let runs = effect_runs.clone();
                effect(move || {
                    even.get();
                    bump(&runs);
                });
            }
            source
        });
        Cutoff {
            rt,
            source,
            next: 2,
            memo_recomputes,
            effect_runs,
        }
    }

    /// Writes the source with a value the memo maps to its current output.
    pub fn step(&mut self) {
        let (source, next) = (self.source, self.next);
        self.rt.enter(|| source.set(next));
        self.next += 2;
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
        ScopeChurn {
            rt: Runtime::new(),
            n,
            effect_runs: Rc::default(),
            cleanups: Rc::default(),
        }
    }

    /// Mounts `n` child scopes under a parent, then disposes the parent.
    pub fn step(&mut self) {
        let (n, runs, cleanups) = (self.n, &self.effect_runs, &self.cleanups);
        self.rt.enter(|| {
            let parent = Scope::new();
            parent.run(|| {
                for i in 0..n {
                    Scope::new().run(|| {
                        let s = signal(i);
                        let m = memo(move || s.get() * 2);
                        let runs = runs.clone();
                        effect(move || {
                            m.get();
                            bump(&runs);
                        });
                        let cleanups = cleanups.clone();
                        on_cleanup(move || bump(&cleanups));
                    });
                }
            });
            parent.dispose();
        });
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
