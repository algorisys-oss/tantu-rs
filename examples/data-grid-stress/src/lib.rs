//! A million-row DataGrid desktop workload. Run `cargo run --release -p data-grid-stress`.
//! Spec: `docs/specs/examples/data-grid-stress.md`. Cadence measures UI ticks, not GPU delivery.
//!
//! ```no_run
//! use tantu::prelude::*;
//! App::new()
//!     .window(Window::new("DataGrid stress").size(1280.0, 720.0), data_grid_stress::stress_view)
//!     .run()?;
//! # Ok::<(), tantu::Error>(())
//! ```

#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use tantu::prelude::*;
use tantu::reactive::{Ticker, ticker};
use tantu::view::Keyed;

/// Generated source row count.
pub const ROWS: usize = 1_000_000;
/// Generated source column count.
pub const COLUMNS: usize = 20;
const ROW_HEIGHT: f32 = 32.0;
const HEADER_HEIGHT: f32 = 36.0;
const CAPACITY: usize = 600;
const WARMUP: Duration = Duration::from_secs(2);
const STATUS_INTERVAL: Duration = Duration::from_millis(500);

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

struct Samples([Duration; CAPACITY]);
impl Default for Samples {
    fn default() -> Self {
        Self([Duration::ZERO; CAPACITY])
    }
}

/// Fixed-capacity interval collection for redraw-tick cadence.
#[derive(Default)]
pub struct CadenceStats {
    intervals: Samples,
    len: usize,
    next: usize,
    baseline: Option<Duration>,
}
impl CadenceStats {
    /// Clears the baseline and samples for a new run.
    pub fn reset(&mut self) {
        self.len = 0;
        self.next = 0;
        self.baseline = None;
    }
    /// Records an increasing elapsed time; the first observation is a baseline.
    /// Equal or earlier observations are ignored, retaining the baseline.
    pub fn record(&mut self, elapsed: Duration) {
        if let Some(previous) = self.baseline {
            if elapsed <= previous {
                return;
            }
            self.intervals.0[self.next] = elapsed - previous;
            self.next = (self.next + 1) % CAPACITY;
            self.len = (self.len + 1).min(CAPACITY);
        }
        self.baseline = Some(elapsed);
    }
    /// Computes the current rolling-window statistics without changing collection.
    pub fn snapshot(&self) -> CadenceSnapshot {
        if self.len == 0 {
            return CadenceSnapshot::default();
        }
        let intervals = &self.intervals.0[..self.len];
        let total_nanos: u128 = intervals.iter().map(Duration::as_nanos).sum();
        let over_budget = intervals
            .iter()
            .filter(|d| d.as_nanos() * 60 > 1_000_000_000)
            .count();
        let mut sorted = self.intervals.0;
        sorted[..self.len].sort_unstable();
        let index = (self.len * 95).div_ceil(100) - 1;
        CadenceSnapshot {
            samples: self.len,
            hz: self.len as f64 * 1_000_000_000.0 / total_nanos as f64,
            p95_ms: sorted[index].as_secs_f64() * 1000.0,
            over_budget,
        }
    }
}

#[derive(Default)]
struct GeneratedSource {
    #[cfg(test)]
    reads: Cell<usize>,
}
impl DataSource for GeneratedSource {
    fn row_count(&self) -> usize {
        ROWS
    }
    fn cell(&self, row: usize, column: usize) -> CellValue {
        #[cfg(test)]
        self.reads.set(self.reads.get() + 1);
        CellValue::Text(format!("r{row}c{column}"))
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
    last_status: Rc<Cell<Duration>>,
    ticker: Ticker,
    source: Rc<GeneratedSource>,
}
impl Workload {
    fn new() -> Self {
        let vertical = ScrollController::new();
        let stats = Rc::new(RefCell::new(CadenceStats::default()));
        let snapshot = signal(CadenceSnapshot::default());
        let last_status = Rc::new(Cell::new(Duration::ZERO));
        let ticker = {
            let stats = stats.clone();
            let last_status = last_status.clone();
            ticker(move |elapsed| {
                let offset = vertical.offset();
                let max = vertical.max_scroll_extent();
                vertical.jump_to(if offset >= max {
                    0.0
                } else {
                    (offset + ROW_HEIGHT).min(max)
                });
                if elapsed >= WARMUP {
                    stats.borrow_mut().record(elapsed);
                }
                if elapsed.saturating_sub(last_status.get()) >= STATUS_INTERVAL {
                    snapshot.set(stats.borrow().snapshot());
                    last_status.set(elapsed);
                }
            })
        };
        Self {
            vertical,
            horizontal: ScrollController::new(),
            frozen: signal(false),
            running: signal(false),
            snapshot,
            stats,
            last_status,
            ticker,
            source: Rc::default(),
        }
    }
    fn start(&self) {
        if self.running.get_untracked() {
            return;
        }
        self.reset_stats();
        self.running.set(true);
        self.ticker.start();
    }
    fn stop(&self) {
        if !self.running.get_untracked() {
            return;
        }
        self.ticker.stop();
        self.running.set(false);
        self.snapshot.set(self.stats.borrow().snapshot());
    }
    fn reset_stats(&self) {
        self.stats.borrow_mut().reset();
        self.snapshot.set(CadenceSnapshot::default());
        self.last_status.set(Duration::ZERO);
    }
    fn jump(&self, position: Jump) {
        self.stop();
        self.reset_stats();
        let max = self.vertical.max_scroll_extent();
        let offset = match position {
            Jump::Beginning => 0.0,
            Jump::Middle => (ROWS / 2) as f32 * ROW_HEIGHT,
            Jump::End => max,
        };
        self.vertical.jump_to(offset.min(max));
    }
    fn set_frozen(&self, frozen: bool) {
        self.stop();
        self.reset_stats();
        self.frozen.set(frozen);
    }
}
#[derive(Clone, Copy)]
enum Jump {
    Beginning,
    Middle,
    End,
}

/// The million-row workload with scrolling controls and cadence status.
pub fn stress_view() -> impl View {
    workload_view(Workload::new())
}
fn workload_view(model: Workload) -> impl View {
    let (vertical, horizontal, frozen, running, snapshot) = (
        model.vertical,
        model.horizontal,
        model.frozen,
        model.running,
        model.snapshot,
    );
    let start = model.clone();
    let stop = model.clone();
    let freeze = model.clone();
    let mut toolbar = Row::new()
        .spacing(8.0)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .child(Keyed::new(
            "start",
            Button::new("Start").on_press(move || start.start()),
        ))
        .child(Keyed::new(
            "stop",
            Button::new("Stop").on_press(move || stop.stop()),
        ));
    for (key, label, jump) in [
        ("beginning", "Beginning", Jump::Beginning),
        ("middle", "Middle", Jump::Middle),
        ("end", "End", Jump::End),
    ] {
        let model = model.clone();
        toolbar = toolbar.child(Keyed::new(
            key,
            Button::new(label).on_press(move || model.jump(jump)),
        ));
    }
    toolbar = toolbar
        .child(Keyed::new(
            "frozen",
            Checkbox::new(move || frozen.get()).on_change(move |v| freeze.set_frozen(v)),
        ))
        .child(Text::new("Frozen columns"));
    let source = model.source;
    Padding::all(12.0).child(Column::new().spacing(8.0).cross_axis_alignment(CrossAxisAlignment::Stretch)
        .child(Text::new("DataGrid stress · 1,000,000 rows × 20 columns").style(TextStyle::title()))
        .child(toolbar)
        .child(Keyed::new("first-row", Text::new(move || format!("First row: {} · {} · one row per redraw tick", (vertical.offset() / ROW_HEIGHT) as usize, if running.get() { "running" } else { "stopped" }))))
        .child(Keyed::new("cadence", Text::new(move || {
            let s = snapshot.get();
            if s.samples == 0 {
                "Redraw cadence: waiting (2 s warmup) · UI ticks, not GPU delivery".to_owned()
            } else {
                format!("Redraw cadence: {} intervals · {:.1} Hz · p95 {:.2} ms · {} over 16.667 ms · UI ticks, not GPU delivery", s.samples, s.hz, s.p95_ms, s.over_budget)
            }
        }).style(TextStyle::label())))
        .child(Expanded::new(SizedBox::expand().child(Dyn::new(move || {
            let count = if frozen.get() { 2 } else { 0 };
            Keyed::new("stress-grid", DataGrid::new(
                (0..COLUMNS).map(|c| GridColumn::new(format!("Column {c}")).width(120.0)), source.clone())
                .row_height(ROW_HEIGHT).header_height(HEADER_HEIGHT)
                .controllers(vertical, horizontal).frozen_columns(count)
                .wheel_duration(Duration::ZERO))
        })))))
}

#[cfg(test)]
mod tests;
