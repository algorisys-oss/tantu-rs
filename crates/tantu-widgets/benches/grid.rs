//! DataGrid benchmarks, spec `docs/specs/widgets/data-grid.md` (WIDGETS-GRID-08).
//!
//! Run with `cargo bench -p tantu-widgets`. The benchmark times one `step()` of the
//! `GridScroll` scenario from `scenarios.rs` (a wheel scroll of one row and its frame);
//! building the grid is not timed.

// `criterion_group!` generates an undocumented `pub fn`.
#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};

mod scenarios;

use scenarios::*;

fn grid(c: &mut Criterion) {
    let mut scroll = GridScroll::new();
    c.bench_function("grid_scroll_1m", |b| b.iter(|| scroll.step()));
}

criterion_group!(benches, grid);
criterion_main!(benches);
