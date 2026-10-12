# DataGrid: million-row stress example

- **Status:** Implemented (the user said "continue", 2026-10-12)
- **Crate:** `examples/data-grid-stress`
- **Plan item:** Phase 4, DataGrid part 7, the 1M-row stress test against the 60 fps criterion
- **Related:** [DataGrid](../widgets/data-grid.md), [frame clock](../reactive/timers.md),
  [gallery](gallery.md), [App runner](../facade/app.md)

## Purpose

A standalone desktop workload for measuring and inspecting scrolling a million-row grid.
The existing `grid_scroll_1m` Criterion benchmark measures CPU widget work with a headless
tester; its roughly 0.8 ms result does not establish native presentation at 60 fps. This
example supplies repeatable scrolling, bounded redraw-cadence statistics and a documented
hardware measurement procedure, without adding the Phase 5 profiling overlay to the library.

## Scope

In scope: a `data-grid-stress` workspace package, a library containing the workload and a
deterministic cadence accumulator, a binary opening a 1280 × 720 logical-pixel window through
`App::run`, behavior tests and one software golden. Runtime dependency: only `tantu`;
`tantu-test` is a dev dependency. An example README records commands and the manual protocol.

Out of scope: library instrumentation APIs, GPU timestamp queries, a new renderer or shell,
sorting/filtering a million rows, editors, a second benchmark implementation, and claiming
the hardware criterion from headless timing. Those grid interactions already have gallery
samples. This example measures redraw tick cadence, not GPU completion or compositor delivery.

## Public API

```rust
use std::time::Duration;
use tantu::prelude::*;

/// Generated source dimensions, independent of viewport size.
pub const ROWS: usize = 1_000_000;
pub const COLUMNS: usize = 20;

/// The million-row workload, including scrolling controls and cadence status.
pub fn stress_view() -> impl View;

/// Statistics for at most the latest 600 positive intervals of one scrolling run.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CadenceSnapshot {
    /// Number of intervals represented (not ticks or completed GPU frames).
    pub samples: usize,
    /// Intervals per second: samples divided by their total duration; zero when empty.
    pub hz: f64,
    /// Nearest-rank 95th percentile interval in milliseconds; zero when empty.
    pub p95_ms: f64,
    /// Intervals longer than 1/60 second, compared without rounding milliseconds.
    pub over_budget: usize,
}

/// Fixed-capacity interval collection; no per-tick allocation after construction.
#[derive(Default)]
pub struct CadenceStats { /* private fields */ }
impl CadenceStats {
    /// Clears the baseline and samples for a new run.
    pub fn reset(&mut self);
    /// Records an increasing elapsed time from the run's ticker. The first observation
    /// establishes a baseline. Equal or earlier times are ignored, retaining the baseline.
    pub fn record(&mut self, elapsed: Duration);
    /// Computes the current rolling-window statistics without changing the collection.
    pub fn snapshot(&self) -> CadenceSnapshot;
}
```

Private generated-source and workload helpers can be unit-tested directly; they are not
additional library APIs. No new facade or widget API is needed.

## Behavior

- **GRID-STRESS-01:** The source reports exactly 1,000,000 rows and 20 columns, generating
  `CellValue::Text("r{row}c{column}")` on demand, including the first and last cell. It stores
  no row collection. The grid (`stress-grid`) has 120 px columns labelled `Column 0` through
  `Column 19`, 32 px rows, a 36 px header and independent vertical/horizontal controllers.
  Smooth wheel animation is disabled, matching the existing CPU benchmark workload.
- **GRID-STRESS-02:** A toolbar precedes an `Expanded` grid. The initial first row is zero;
  automatic scrolling is initially stopped. `Start` (`start`), `Stop` (`stop`), `Beginning`
  (`beginning`), `Middle` (`middle`) and `End` (`end`) are present. Starting resets statistics
  and starts a scoped ticker; every delivered tick advances one row (32 px), wrapping to
  zero at the vertical end. Stopping stops the ticker and preserves the last statistics.
  Start while running and Stop while stopped are no-ops. Removing the view disposes its ticker.
- **GRID-STRESS-03:** Beginning, Middle and End stop automatic scrolling and reset cadence
  samples; they jump to zero, row 500,000 and the maximum vertical offset respectively,
  clamped to the current viewport. A `Frozen columns` checkbox (`frozen`) toggles between
  zero and two pinned columns; it stops and resets the run, preserving offsets subject to
  the grid's usual clamping. Manual scrolling works on both axes. These controls permit
  repeatable trials without collecting or sorting rows.
- **GRID-STRESS-04:** For increasing elapsed observations, the accumulator retains the
  latest 600 intervals only. Empty and single-observation snapshots have zero samples and
  zero numeric statistics. Duplicate/decreasing observations do not alter the baseline or
  samples. `reset` restores the empty state. For `n` intervals, p95 is sorted interval
  `ceil(0.95*n)-1`; rate is `n/sum(intervals in seconds)`. `over_budget` counts intervals
  whose exact duration exceeds 1/60 second (compare integer nanoseconds × 60 to 1 billion).
  Tests cover known distributions, ring rollover, restart and exact budget boundaries.
- **GRID-STRESS-05:** Status texts show the first visible row (`first-row`), dimensions,
  running/stopped state, and `Redraw cadence` (`cadence`) with sample count, Hz, p95 ms and
  over-budget count. Empty samples display a waiting indication. Cadence status refreshes
  at most twice per elapsed second while running, and immediately on Stop/reset; it does
  not add a rebuild for every scrolling tick. Measurements exclude the first two seconds
  of each run: collect the first baseline at/after two seconds, then intervals thereafter.
  The UI and README explicitly identify this as redraw-tick cadence, not GPU timing.
- **GRID-STRESS-06:** At 1280 × 720, after initialization, a one-row wheel step, a jump to
  the middle/end, horizontal scrolling and the frozen-column toggle, generated-cell reads
  per settled frame and live body cells are bounded by the visible row/column ranges plus
  the grid's one-cell buffers (and two frozen columns), independent of source row count.
  Count source reads through a private test instrument; inspect cells outside timed work.
  A software golden of the stopped initial view verifies readable controls/status, clipped
  grid content and the full-size window layout.

## Performance and allocation

- Constructing the data source uses constant space; no million-row allocation or scan.
- The ticker, controllers and statistics belong to the view's reactive owner. Collection
  uses fixed-capacity storage for 600 durations. Snapshot calculation may sort a fixed-size
  scratch buffer at the limited status-update cadence; it does not allocate per tick.
- Cadence includes ordinary grid work and limited status updates. It is an end-to-end
  scheduling observation at the UI ticker, rather than a duration of layout/paint alone.
- Retain the existing CPU benchmark (`cargo bench -p tantu-widgets --bench grid`), target
  below 8 ms per scroll step. Do not add Finder traversal to its measured section.

## Hardware validation procedure

Performance benchmarks and native stress-app trials run locally only (the user, 2026-10-12).
GitHub Actions may run the bounded deterministic correctness tests; no benchmark or native
performance workload is added there.

1. Run `cargo run --release -p data-grid-stress` at 1280 × 720 logical pixels with the
   window fully visible. Record commit, OS/display backend, CPU, GPU/driver, display refresh
   rate, scale factor and power mode. Run with a 60 Hz display mode for a 60 Hz trial.
2. Start at Beginning, press Start, allow the two-second warmup and observe for at least
   30 seconds. Record steady-state rate, p95 interval and over-budget fraction. Repeat from
   Middle and with two frozen columns; repeat each trial three times.
3. Inspect for visible stalls, blank cells and incorrect frozen/header positioning. Use a
   platform presentation profiler when available to verify delivered frames; record the
   tool and its measurement boundary alongside results. Redraw cadence alone cannot
   certify compositor delivery. A rate below 59 Hz on a 60 Hz trial is a failed scheduling
   check; above that, assess interval distribution and delivery rather than asserting
   success from a rounded mean. The renderer/frame-work budget remains 16.667 ms.
4. Record the existing CPU benchmark separately. No performance assertion depending on
   host timing is made in CI. If native hardware/profiler access is unavailable, record
   that the 0.1 hardware success criterion remains unverified; the example can be complete
   while that project-wide criterion remains open.

## Resolved proposals

1. **Workload:** match the existing 1M × 20 text-cell benchmark, with an optional two-column
   frozen trial. Keep editors/sorting/filtering in the gallery rather than changing the
   measured workload. Agreed on the user's "continue" (2026-10-12).
2. **Measurement boundary:** use the existing scoped reactive ticker for repeatable
   scrolling and label its cadence honestly. Keep GPU delivery verification manual; avoid
   a new library profiler API during Phase 4. Agreed on the user's "continue" (2026-10-12).
3. **Run protocol:** one row per delivered tick, two-second warmup, 600-interval rolling
   window, status updates at most twice per second, 30-second hardware trials. Agreed on the user's "continue" (2026-10-12).
4. **Completion:** complete part 7 when the example, deterministic tests, golden and
   measurement instructions pass checks; retain an explicit open hardware-validation item
   in HANDOFF/PLAN until native 60 fps has been measured. Agreed on the user's "continue" (2026-10-12).
