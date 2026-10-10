//! Layout benchmarks, spec `docs/specs/layout/benchmarks.md`.
//!
//! Run with `cargo bench -p tantu-layout`. Each benchmark times one `step()` of a scenario from
//! `scenarios.rs`; building the tree is not timed.

// `criterion_group!` generates an undocumented `pub fn`.
#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};

mod scenarios;

use scenarios::*;

fn layout(c: &mut Criterion) {
    let mut full = FullLayout::new();
    c.bench_function("full_layout_10k", |b| b.iter(|| full.step()));
    let mut one = RelayoutOneLeaf::new();
    c.bench_function("relayout_one_leaf_10k", |b| b.iter(|| one.step()));
    let mut noop = NoOpLayout::new();
    c.bench_function("no_op_layout_10k", |b| b.iter(|| noop.step()));
    let mut wide = WideFlex::new();
    c.bench_function("wide_flex_10k", |b| b.iter(|| wide.step()));
    let mut text = TextGrid::new();
    c.bench_function("text_grid_10k", |b| b.iter(|| text.step()));
}

criterion_group!(benches, layout);
criterion_main!(benches);
