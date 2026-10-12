# Million-row DataGrid workload

Run `cargo run --release -p data-grid-stress`. The window opens at 1280 × 720 logical pixels
with 1,000,000 generated rows and 20 text columns. There is no million-row array or scan.

Start scrolls one row per delivered UI tick. Stop freezes the last statistics. Beginning,
Middle and End stop/reset the run and jump to the requested position. Frozen columns toggles
two pinned columns and resets the trial. Manual scrolling works on both axes.

The status measures **redraw-tick cadence**, including ordinary grid work and status updates,
not GPU completion or compositor delivery. It excludes a two-second warmup, retains at most
600 positive intervals, and refreshes at most twice per second. Rate is interval count divided
by total time. p95 uses nearest rank. The over-budget count compares durations to the exact
1/60-second budget before rounding displayed milliseconds. Stop publishes the final snapshot.

For a repeatable native trial:

1. Use a release build and keep the 1280 × 720 window fully visible. Record the commit, OS,
   display backend, CPU, GPU/driver, refresh rate, scale factor and power mode. Use a 60 Hz
   display mode for a 60 Hz trial.
2. Press Beginning, then Start. After the two-second warmup, observe at least 30 seconds.
   Record steady-state Hz, p95 interval and over-budget fraction. Repeat from Middle and
   with two frozen columns. Repeat each trial three times.
3. Inspect for stalls, blank cells and incorrect frozen/header positions. Record delivered
   frames with a platform presentation profiler when available, including its measurement
   boundary. Below 59 Hz is a failed scheduling check on a 60 Hz trial; above that, inspect
   the interval distribution and delivery. A rounded mean alone does not certify 60 fps.
   The renderer/frame-work budget is 16.667 ms.
4. Separately run `cargo bench -p tantu-widgets --bench grid` for CPU grid work. Its target
   is below 8 ms per step; it does not include native GPU presentation.

Deterministic checks: `cargo test -p data-grid-stress`. No host-speed assertion runs in CI.
The software golden checks the initial stopped view. The spec is
[`docs/specs/examples/data-grid-stress.md`](../../docs/specs/examples/data-grid-stress.md).

Native 60 fps delivery on representative hardware is **not yet verified**. Record results
and presentation-profiler evidence in HANDOFF before closing the project's hardware criterion.
