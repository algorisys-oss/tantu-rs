//! Correctness test for the benchmark scenario, spec `docs/specs/widgets/data-grid.md`
//! (WIDGETS-GRID-08).

#[path = "../benches/scenarios.rs"]
mod scenarios;

use scenarios::*;

#[test]
fn widgets_grid_08_grid_scroll_scenario() {
    let mut s = GridScroll::new();
    let bound = GridScroll::cell_bound();
    assert_eq!(s.first_visible_row(), 0);
    assert!(
        s.built_cells() <= bound,
        "{} cells, bound {bound}",
        s.built_cells()
    );
    for _ in 0..3 {
        s.step();
    }
    assert_eq!(s.first_visible_row(), 3);
    assert!(
        s.built_cells() <= bound,
        "{} cells, bound {bound}",
        s.built_cells()
    );
}
