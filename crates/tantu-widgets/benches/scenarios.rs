//! Benchmark scenarios for `docs/specs/widgets/data-grid.md` (WIDGETS-GRID-08), shared by
//! `benches/grid.rs` (timed) and `tests/bench_scenarios.rs` (checked).

#![allow(dead_code)]

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use tantu_test::{Finder, WidgetTester};
use tantu_view::Keyed;
use tantu_view::core::Vec2;
use tantu_view::layout::RenderParagraph;
use tantu_widgets::{CellValue, DataGrid, FnSource, GridColumn, ScrollController};

/// The grid's rows.
pub const ROWS: usize = 1_000_000;
/// The grid's columns, 120 px each.
pub const COLUMNS: usize = 20;
/// Each row's height.
pub const ROW_HEIGHT: f32 = 32.0;
/// The header's height.
pub const HEADER_HEIGHT: f32 = 36.0;
/// The window.
pub const WINDOW: (f32, f32) = (1280.0, 720.0);

/// A 1 000 000 × 20 grid in a 1280 × 720 window; a step scrolls one row with the wheel
/// (deltas applied at once) and runs the frame.
pub struct GridScroll {
    tester: WidgetTester,
    vertical: ScrollController,
}

impl GridScroll {
    /// Builds the grid and runs its first frame.
    pub fn new() -> Self {
        let out: Rc<Cell<Option<ScrollController>>> = Rc::default();
        let mut tester = {
            let out = out.clone();
            WidgetTester::with_size(WINDOW.0, WINDOW.1, move || {
                let (vertical, horizontal) = (ScrollController::new(), ScrollController::new());
                out.set(Some(vertical));
                Keyed::new(
                    "g",
                    DataGrid::new(
                        (0..COLUMNS).map(|c| GridColumn::new(format!("Column {c}"))),
                        Rc::new(FnSource::new(ROWS, |r, c| {
                            CellValue::Text(format!("r{r}c{c}"))
                        })),
                    )
                    .row_height(ROW_HEIGHT)
                    .header_height(HEADER_HEIGHT)
                    .controllers(vertical, horizontal)
                    .wheel_duration(Duration::ZERO),
                )
            })
        };
        // The cells appear one frame after the first paint records the extents.
        tester.pump();
        GridScroll {
            tester,
            vertical: out.get().expect("built"),
        }
    }

    /// One wheel step of a row, and its frame.
    pub fn step(&mut self) {
        self.tester
            .scroll(&Finder::key("g"), Vec2::new(0.0, ROW_HEIGHT));
    }

    /// The row at the top of the body.
    pub fn first_visible_row(&self) -> usize {
        (self.tester.enter(|| self.vertical.offset()) / ROW_HEIGHT) as usize
    }

    /// How many cells (header and body) are built: one paragraph each.
    pub fn built_cells(&self) -> usize {
        self.tester
            .find_all(&Finder::render::<RenderParagraph>())
            .len()
    }

    /// WIDGETS-GRID-02's bound for this window: (visible rows + 2) × (visible columns + 2)
    /// body cells plus (visible columns + 2) header cells.
    pub fn cell_bound() -> usize {
        let rows = ((WINDOW.1 - HEADER_HEIGHT) / ROW_HEIGHT).ceil() as usize + 2;
        let columns = (WINDOW.0 / 120.0).ceil() as usize + 2;
        rows * columns + columns
    }
}

impl Default for GridScroll {
    fn default() -> Self {
        GridScroll::new()
    }
}
