//! Correctness tests for the benchmark scenarios, spec `docs/specs/layout/benchmarks.md`.

#[path = "../benches/scenarios.rs"]
mod scenarios;

use scenarios::*;

const STEPS: usize = 3;

#[test]
fn layout_bench_01_full_layout_lays_out_everything() {
    let mut s = FullLayout::new();
    let nodes = s.node_count();
    assert!((9_000..=11_000).contains(&nodes), "{nodes} nodes");
    for _ in 0..STEPS {
        let size = s.step();
        assert_eq!(s.leaf_layouts(), 3_200, "every leaf, once");
        assert_eq!(size, s.expected_root_size());
        assert!(s.leaves_inside_rows());
    }
}

#[test]
fn layout_bench_02_relayout_one_leaf() {
    let mut s = RelayoutOneLeaf::new();
    for _ in 0..STEPS {
        s.step();
        assert_eq!(s.leaf_layouts(), 1);
        let (laid_out, given) = s.changed_leaf();
        assert_eq!(laid_out, given);
    }
}

#[test]
fn layout_bench_03_no_op_layout() {
    let mut s = NoOpLayout::new();
    let first = s.step();
    for _ in 0..STEPS {
        assert_eq!(s.step(), first);
        assert_eq!(s.leaf_layouts(), 0);
    }
}

#[test]
fn layout_bench_04_wide_flex() {
    let mut s = WideFlex::new();
    for _ in 0..STEPS {
        s.step();
        assert_eq!(s.child_layouts(), 10_000);
        let (expanded, free) = s.expanded_width_and_free_space();
        assert!(
            (expanded - free).abs() <= free * 0.001,
            "{expanded} vs {free}"
        );
        assert_eq!(s.overflow(), 0.0);
    }
}

#[test]
fn layout_bench_05_text_grid_hits_the_cache() {
    let mut s = TextGrid::new();
    for _ in 0..STEPS {
        s.step();
        assert_eq!(s.paragraph_layouts(), 10_000);
        assert!(s.inner_calls() <= 100, "{} inner calls", s.inner_calls());
    }
}
