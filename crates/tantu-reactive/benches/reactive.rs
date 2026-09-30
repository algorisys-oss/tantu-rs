//! Reactive micro-benchmarks, spec `docs/specs/reactive/benchmarks.md`.
//!
//! Run with `cargo bench -p tantu-reactive`. Each benchmark times one `step()` of a scenario from
//! `scenarios.rs`; building the graph is not timed.

// `criterion_group!` generates an undocumented `pub fn`.
#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

mod scenarios;

use scenarios::*;

const SIZES: [usize; 3] = [10, 100, 1000];

/// Times `step` of the scenario built by `new`, at each size.
macro_rules! bench_scenario {
    ($c:expr, $name:literal, $ty:ty, $sizes:expr) => {{
        let mut group = $c.benchmark_group($name);
        for n in $sizes {
            let mut scenario = <$ty>::new(n);
            group.bench_function(BenchmarkId::from_parameter(n), |b| {
                b.iter(|| scenario.step())
            });
        }
        group.finish();
    }};
}

fn reactive(c: &mut Criterion) {
    bench_scenario!(c, "create_dispose", CreateDispose, SIZES);
    bench_scenario!(c, "get_set", GetSet, [100_000]);
    bench_scenario!(c, "fan_out", FanOut, SIZES);
    bench_scenario!(c, "fan_in", FanIn, SIZES);
    bench_scenario!(c, "deep_chain", DeepChain, SIZES);
    bench_scenario!(c, "diamond", Diamond, SIZES);
    bench_scenario!(c, "cutoff", Cutoff, SIZES);
    bench_scenario!(c, "scope_churn", ScopeChurn, SIZES);
}

criterion_group!(benches, reactive);
criterion_main!(benches);
