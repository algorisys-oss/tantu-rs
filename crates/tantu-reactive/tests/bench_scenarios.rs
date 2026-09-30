//! Correctness tests for the benchmark scenarios, spec `docs/specs/reactive/benchmarks.md`.

#[path = "../benches/scenarios.rs"]
mod scenarios;

use scenarios::*;

const STEPS: usize = 3;

#[test]
fn reactive_bench_01_create_dispose_restores_node_count() {
    let mut s = CreateDispose::new(100);
    let before = s.node_count();
    for k in 1..=STEPS {
        s.step();
        assert_eq!(s.effect_runs(), k * 100);
        assert_eq!(s.node_count(), before);
    }
}

#[test]
fn reactive_bench_02_get_set_round_trips() {
    let mut s = GetSet::new(1000);
    for _ in 0..STEPS {
        s.step();
    }
    assert_eq!(s.value(), (STEPS * 1000) as u64);
}

#[test]
fn reactive_bench_03_fan_out_runs_each_effect_once() {
    let n = 100;
    let mut s = FanOut::new(n);
    let initial = s.effect_runs();
    assert_eq!(initial, n);
    for k in 1..=STEPS {
        s.step();
        assert_eq!(s.effect_runs(), initial + k * n);
        let source = s.source();
        assert!(s.seen().iter().all(|&v| v == source));
    }
}

#[test]
fn reactive_bench_04_fan_in_batch_runs_effect_once() {
    let mut s = FanIn::new(100);
    let initial = s.effect_runs();
    assert_eq!(initial, 1);
    for k in 1..=STEPS {
        s.step();
        assert_eq!(s.effect_runs(), initial + k);
        assert_eq!(s.seen(), s.expected());
        assert_ne!(s.seen(), 0);
    }
}

#[test]
fn reactive_bench_05_deep_chain_recomputes_each_memo_once() {
    let n = 1000;
    let mut s = DeepChain::new(n);
    let (recomputes, runs) = (s.recomputes(), s.effect_runs());
    assert_eq!(recomputes, n);
    for k in 1..=STEPS {
        s.step();
        assert_eq!(s.recomputes(), recomputes + k * n);
        assert_eq!(s.effect_runs(), runs + k);
        assert_eq!(s.seen(), s.head() + n as u64);
    }
}

#[test]
fn reactive_bench_06_diamond_is_glitch_free() {
    let n = 100;
    let mut s = Diamond::new(n);
    let (middle, sum, runs) = (s.middle_recomputes(), s.sum_recomputes(), s.effect_runs());
    assert_eq!((middle, sum, runs), (n, 1, 1));
    for k in 1..=STEPS {
        s.step();
        assert_eq!(s.middle_recomputes(), middle + k * n);
        assert_eq!(s.sum_recomputes(), sum + k);
        assert_eq!(s.effect_runs(), runs + k);
    }
    assert_eq!(s.glitches(), 0);
}

#[test]
fn reactive_bench_07_cutoff_skips_effects() {
    let n = 100;
    let mut s = Cutoff::new(n);
    let (recomputes, runs) = (s.memo_recomputes(), s.effect_runs());
    assert_eq!((recomputes, runs), (1, n));
    for k in 1..=STEPS {
        s.step();
        assert_eq!(s.memo_recomputes(), recomputes + k);
        assert_eq!(s.effect_runs(), runs);
    }
}

#[test]
fn reactive_bench_08_scope_churn_runs_cleanups_and_frees_nodes() {
    let n = 100;
    let mut s = ScopeChurn::new(n);
    let before = s.node_count();
    for k in 1..=STEPS {
        s.step();
        assert_eq!(s.effect_runs(), k * n);
        assert_eq!(s.cleanups(), k * n);
        assert_eq!(s.node_count(), before);
    }
}
