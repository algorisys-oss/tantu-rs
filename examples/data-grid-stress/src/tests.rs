//! Deterministic workload and cadence checks; no host-speed assertion.
use super::*;
use tantu::core::Vec2;
use tantu_test::{Finder, WidgetTester};

struct Fixture {
    tester: WidgetTester,
    model: Workload,
    visible: Signal<bool>,
}
impl Fixture {
    fn new() -> Self {
        let output = Rc::new(RefCell::new(None));
        let visible = Rc::new(Cell::new(None));
        let mut tester = {
            let output = output.clone();
            let visible = visible.clone();
            WidgetTester::with_size(1280.0, 720.0, move || {
                let show = signal(true);
                visible.set(Some(show));
                Show::new(
                    move || show.get(),
                    move || {
                        let model = Workload::new();
                        *output.borrow_mut() = Some(model.clone());
                        workload_view(model)
                    },
                )
            })
        };
        tester.pump();
        let model = output.borrow_mut().take().expect("workload built");
        Self {
            tester,
            model,
            visible: visible.get().expect("visibility built"),
        }
    }
    fn text(&self, key: &str) -> String {
        self.tester
            .text(self.tester.find(&Finder::key(key)))
            .expect("text")
    }
    fn tap(&mut self, key: &str) {
        self.tester.tap(&Finder::key(key));
    }
    fn row(&self) -> usize {
        self.tester
            .enter(|| (self.model.vertical.offset() / ROW_HEIGHT) as usize)
    }
}

#[test]
fn grid_stress_01_source_and_dimensions() {
    let source = GeneratedSource::default();
    assert_eq!(source.row_count(), ROWS);
    assert_eq!(COLUMNS, 20);
    assert_eq!(source.cell(0, 0), CellValue::Text("r0c0".into()));
    assert_eq!(
        source.cell(ROWS - 1, COLUMNS - 1),
        CellValue::Text("r999999c19".into())
    );
    // Only instrumentation storage, never a row buffer.
    assert_eq!(
        std::mem::size_of::<GeneratedSource>(),
        std::mem::size_of::<Cell<usize>>()
    );
    let f = Fixture::new();
    assert_eq!(f.row(), 0);
    let grid = f.tester.rect(f.tester.find(&Finder::key("stress-grid")));
    assert_eq!(grid.width(), 1256.0);
    assert!(grid.bottom <= 720.0 && grid.height() > 500.0);
    let c0 = f.tester.rect(f.tester.find(&Finder::text("Column 0")));
    let c1 = f.tester.rect(f.tester.find(&Finder::text("Column 1")));
    assert_eq!(c1.left - c0.left, 120.0);
    let r0 = f.tester.rect(f.tester.find(&Finder::text("r0c0")));
    let r1 = f.tester.rect(f.tester.find(&Finder::text("r1c0")));
    assert_eq!(r1.top - r0.top, ROW_HEIGHT);
    // Text is centered in differently sized header/body cells.
    assert_eq!(c0.center().y - grid.top, HEADER_HEIGHT / 2.0);
    assert_eq!(r0.center().y - grid.top, HEADER_HEIGHT + ROW_HEIGHT / 2.0);
}

#[test]
fn grid_stress_02_start_stop_wrap_and_dispose() {
    let mut f = Fixture::new();
    assert!(!f.tester.enter(|| f.model.ticker.is_active()));
    f.tap("start");
    assert!(f.tester.enter(|| f.model.running.get()));
    assert!(f.tester.enter(|| f.model.ticker.is_active()));
    let before = f.row();
    f.tester.advance(Duration::from_millis(16));
    assert_eq!(f.row(), before + 1);
    // Start again does not restart the measurement clock.
    f.tester.advance(Duration::from_secs(2));
    f.tester.advance(Duration::from_millis(16));
    let samples = f.model.stats.borrow().snapshot().samples;
    f.tap("start");
    assert_eq!(f.model.stats.borrow().snapshot().samples, samples);
    f.tap("stop");
    let snapshot = f.tester.enter(|| f.model.snapshot.get());
    let stopped_row = f.row();
    f.tester.advance(Duration::from_secs(1));
    f.tap("stop");
    assert_eq!(f.row(), stopped_row);
    assert_eq!(f.tester.enter(|| f.model.snapshot.get()), snapshot);
    f.tap("end");
    f.tap("start");
    assert_eq!(f.row(), 0, "first delivered tick wraps at the end");
    let ticker = f.model.ticker;
    f.tester.enter(|| f.visible.set(false));
    f.tester.pump();
    assert!(
        !f.tester.enter(|| ticker.is_active()),
        "view removal disposes ticker"
    );
    f.tester.advance(Duration::from_secs(1));
}

#[test]
fn grid_stress_03_jumps_manual_scroll_and_frozen() {
    let mut f = Fixture::new();
    f.tap("middle");
    f.tester.pump();
    assert_eq!(f.row(), 500_000);
    f.tester.find(&Finder::text("r500000c0"));
    f.tap("end");
    f.tester.pump();
    assert_eq!(
        f.tester.enter(|| f.model.vertical.offset()),
        f.tester.enter(|| f.model.vertical.max_scroll_extent())
    );
    f.tap("beginning");
    assert_eq!(f.row(), 0);
    f.tester
        .scroll(&Finder::key("stress-grid"), Vec2::new(360.0, 96.0));
    f.tester.pump();
    assert_eq!(f.row(), 3);
    assert_eq!(f.tester.enter(|| f.model.horizontal.offset()), 360.0);
    f.tap("start");
    let before = f.row();
    f.tap("frozen");
    f.tester.pump();
    assert!(f.tester.enter(|| f.model.frozen.get()));
    assert!(!f.tester.enter(|| f.model.running.get()));
    assert_eq!(f.row(), before);
    assert_eq!(f.tester.enter(|| f.model.horizontal.offset()), 360.0);
    let grid = f.tester.rect(f.tester.find(&Finder::key("stress-grid")));
    let c0 = f.tester.rect(f.tester.find(&Finder::text("Column 0")));
    let c1 = f.tester.rect(f.tester.find(&Finder::text("Column 1")));
    assert_eq!(c0.left - grid.left, 8.0);
    assert_eq!(c1.left - c0.left, 120.0);
    assert_eq!(
        f.tester.enter(|| f.model.snapshot.get()),
        CadenceSnapshot::default()
    );
}

#[test]
fn grid_stress_04_cadence_distribution_and_invalid_times() {
    let mut stats = CadenceStats::default();
    assert_eq!(stats.snapshot(), CadenceSnapshot::default());
    stats.record(Duration::from_secs(2));
    assert_eq!(stats.snapshot(), CadenceSnapshot::default());
    let mut elapsed = Duration::from_secs(2);
    for ms in 1..=20 {
        elapsed += Duration::from_millis(ms);
        stats.record(elapsed);
    }
    let s = stats.snapshot();
    assert_eq!(s.samples, 20);
    assert!((s.hz - 20.0 / 0.210).abs() < 1e-9);
    assert_eq!(s.p95_ms, 19.0);
    assert_eq!(s.over_budget, 4);
    stats.record(elapsed);
    stats.record(Duration::ZERO);
    assert_eq!(stats.snapshot(), s);
    stats.record(elapsed + Duration::from_millis(10));
    assert_eq!(stats.snapshot().samples, 21);
    stats.reset();
    assert_eq!(stats.snapshot(), CadenceSnapshot::default());
    stats.record(Duration::ZERO);
    stats.record(Duration::from_nanos(16_666_666));
    stats.record(Duration::from_nanos(33_333_333));
    assert_eq!(stats.snapshot().over_budget, 1);
}

#[test]
fn grid_stress_04_ring_rollover_retains_latest_intervals() {
    let mut stats = CadenceStats::default();
    let mut elapsed = Duration::ZERO;
    stats.record(elapsed);
    for _ in 0..30 {
        elapsed += Duration::from_millis(100);
        stats.record(elapsed);
    }
    for _ in 0..600 {
        elapsed += Duration::from_millis(10);
        stats.record(elapsed);
    }
    let s = stats.snapshot();
    assert_eq!(s.samples, 600);
    assert_eq!(s.hz, 100.0);
    assert_eq!(s.p95_ms, 10.0);
    assert_eq!(s.over_budget, 0);
}

#[test]
fn grid_stress_05_warmup_throttle_and_status() {
    let mut f = Fixture::new();
    assert!(f.text("cadence").contains("Redraw cadence"));
    assert!(f.text("cadence").contains("waiting"));
    f.tap("start");
    f.tester.advance(Duration::from_millis(1999));
    assert_eq!(f.model.stats.borrow().snapshot().samples, 0);
    f.tester.advance(Duration::from_millis(1));
    assert_eq!(f.model.stats.borrow().snapshot().samples, 0);
    let waiting = f.text("cadence");
    for _ in 0..24 {
        f.tester.advance(Duration::from_millis(20));
    }
    assert_eq!(f.text("cadence"), waiting);
    f.tester.advance(Duration::from_millis(20));
    assert!(f.text("cadence").contains("25 intervals"));
    assert!(f.text("cadence").contains("50.0 Hz"));
    assert!(f.text("cadence").contains("20.00 ms"));
    assert!(f.text("cadence").contains("25 over"));
    assert!(f.text("cadence").contains("not GPU"));
    assert!(
        f.text("first-row")
            .contains(&format!("First row: {}", f.row()))
    );
    f.tester.advance(Duration::from_millis(20));
    f.tap("stop");
    assert!(f.text("cadence").contains("26 intervals"));
    f.tap("middle");
    assert!(f.text("cadence").contains("waiting"));
}

#[test]
fn grid_stress_06_bounded_work_and_golden() {
    let mut f = Fixture::new();
    f.tester.find(&Finder::key("stress-grid"));
    f.tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/stress-initial.png"
    ));
    fn bounded(f: &mut Fixture) {
        f.model.source.reads.set(0);
        f.tester.pump();
        let rect = f.tester.rect(f.tester.find(&Finder::key("stress-grid")));
        let rows = ((rect.height() - HEADER_HEIGHT) / ROW_HEIGHT).ceil() as usize + 2;
        let columns = (rect.width() / 120.0).ceil() as usize + 2 + 2;
        let bound = rows * columns;
        assert!(f.model.source.reads.get() <= bound);
        // Semantic cell count excludes headers/status and is inspected outside timing.
        let cells = f
            .tester
            .a11y_tree()
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == tantu_test::accesskit::Role::Cell)
            .count();
        assert!(cells > 0 && cells <= bound, "live {cells}, bound {bound}");
    }
    bounded(&mut f);
    f.tester
        .scroll(&Finder::key("stress-grid"), Vec2::new(0.0, 32.0));
    bounded(&mut f);
    for key in ["middle", "end", "frozen"] {
        f.tap(key);
        bounded(&mut f);
    }
    f.tester
        .scroll(&Finder::key("stress-grid"), Vec2::new(480.0, 0.0));
    bounded(&mut f);
}
