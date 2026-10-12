//! A million-row DataGrid desktop workload. Run `cargo run --release -p data-grid-stress`.
//! Spec: `docs/specs/examples/data-grid-stress.md`. Cadence measures UI ticks, not GPU delivery.

#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use tantu::prelude::*;
use tantu::reactive::{Ticker, ticker};

/// Generated source row count.
pub const ROWS: usize = 1_000_000;
/// Generated source column count.
pub const COLUMNS: usize = 20;
const ROW_HEIGHT: f32 = 32.0;
const HEADER_HEIGHT: f32 = 36.0;

/// Statistics for at most the latest 600 positive intervals of one scrolling run.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CadenceSnapshot {
    /// Number of intervals represented, not completed GPU frames.
    pub samples: usize,
    /// Intervals per second; zero when empty.
    pub hz: f64,
    /// Nearest-rank 95th percentile interval in milliseconds; zero when empty.
    pub p95_ms: f64,
    /// Intervals longer than 1/60 second, before rounding milliseconds.
    pub over_budget: usize,
}

/// Fixed-capacity interval collection for redraw-tick cadence.
#[derive(Default)]
pub struct CadenceStats;
impl CadenceStats {
    /// Clears the baseline and samples for a new run.
    pub fn reset(&mut self) {
        todo!("GRID-STRESS-04")
    }
    /// Records an increasing elapsed time; the first observation is a baseline.
    pub fn record(&mut self, _elapsed: Duration) {
        todo!("GRID-STRESS-04")
    }
    /// Computes the current rolling-window statistics without changing collection.
    pub fn snapshot(&self) -> CadenceSnapshot {
        todo!("GRID-STRESS-04")
    }
}

#[derive(Default)]
struct GeneratedSource {
    #[cfg(test)]
    reads: Cell<usize>,
}
impl DataSource for GeneratedSource {
    fn row_count(&self) -> usize {
        0
    }
    fn cell(&self, _row: usize, _column: usize) -> CellValue {
        CellValue::Empty
    }
}

#[derive(Clone)]
struct Workload {
    vertical: ScrollController,
    horizontal: ScrollController,
    frozen: Signal<bool>,
    running: Signal<bool>,
    snapshot: Signal<CadenceSnapshot>,
    stats: Rc<RefCell<CadenceStats>>,
    ticker: Ticker,
    source: Rc<GeneratedSource>,
}
impl Workload {
    fn new() -> Self {
        Self {
            vertical: ScrollController::new(),
            horizontal: ScrollController::new(),
            frozen: signal(false),
            running: signal(false),
            snapshot: signal(CadenceSnapshot::default()),
            stats: Rc::new(RefCell::new(CadenceStats)),
            ticker: ticker(|_| {}),
            source: Rc::default(),
        }
    }
}

/// The million-row workload with scrolling controls and cadence status.
pub fn stress_view() -> impl View {
    workload_view(Workload::new())
}
fn workload_view(_model: Workload) -> impl View {
    Text::new("Stress example pending")
}

#[cfg(test)]
mod tests;
