//! DataGrid part 5: selection, navigation, reveal and mapped semantics.

use std::cell::RefCell;
use std::rc::Rc;
use tantu_test::accesskit::{Action, Role};
use tantu_test::{Finder, WidgetTester};
use tantu_view::reactive::{Signal, signal};
use tantu_view::{Keyed, LogicalKey, Modifiers, NamedKey, SemanticsAction};
use tantu_widgets::{
    CellValue, DataGrid, FnSource, GridCell, GridColumn, GridSelection, GridSelectionController,
    GridSelectionMode as Mode, ScrollController, SortState,
};

fn cell(row: usize, column: usize) -> GridCell {
    GridCell { row, column }
}
fn single(row: usize, column: usize) -> GridSelection {
    GridSelection::single(cell(row, column))
}
fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Default::default()
    }
}
fn primary() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers {
            super_key: true,
            ..Default::default()
        }
    } else {
        Modifiers {
            control: true,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
struct Handles {
    selection: GridSelectionController,
    vertical: ScrollController,
    horizontal: ScrollController,
    order: Signal<Vec<usize>>,
    widths: Signal<Vec<f32>>,
    sort: Signal<Option<SortState>>,
    revision: Signal<u64>,
}
struct Fixture {
    t: WidgetTester,
    h: Handles,
}
impl Fixture {
    fn new(mode: Mode) -> Self {
        let out = Rc::new(RefCell::new(None));
        let capture = out.clone();
        let mut t = WidgetTester::with_size(360.0, 200.0, move || {
            let h = Handles {
                selection: GridSelectionController::new(),
                vertical: ScrollController::new(),
                horizontal: ScrollController::new(),
                order: signal((0..8).collect()),
                widths: signal(vec![120.0; 8]),
                sort: signal(None),
                revision: signal(0),
            };
            *capture.borrow_mut() = Some(h.clone());
            Keyed::new(
                "grid",
                DataGrid::new(
                    (0..8).map(|c| GridColumn::new(format!("C{c}"))),
                    Rc::new(FnSource::new(1_000_000, |r, c| {
                        CellValue::Text(format!("r{r}c{c}"))
                    })),
                )
                .selection_mode(mode)
                .selection_controller(h.selection)
                .controllers(h.vertical, h.horizontal)
                .column_order(h.order)
                .column_widths(h.widths)
                .sort_state(h.sort)
                .revision(h.revision)
                .frozen_columns(1)
                .client_sort(false)
                .reorderable(true),
            )
        });
        t.pump();
        let h = out.borrow_mut().take().expect("built");
        Self { t, h }
    }
    fn selection(&self) -> Option<GridSelection> {
        self.t.enter(|| self.h.selection.selection())
    }
    fn set(&mut self, selection: Option<GridSelection>) {
        self.t.enter(|| self.h.selection.set_selection(selection));
        self.t.pump();
        self.t.pump();
    }
    fn key(&mut self, key: NamedKey, modifiers: Modifiers) {
        self.t.press_key(LogicalKey::Named(key), modifiers);
        self.t.pump();
    }
}

#[test]
fn widgets_grid_16_models_bounds_and_membership() {
    let reversed = GridSelection {
        anchor: cell(usize::MAX, 9),
        active: cell(2, 1),
    };
    assert!(reversed.contains(cell(8, 3), Mode::Range));
    assert!(reversed.contains(cell(8, 500), Mode::Row));
    assert!(!reversed.contains(cell(8, 3), Mode::Cell));
    assert!(!reversed.contains(cell(8, 3), Mode::None));
    let mut f = Fixture::new(Mode::Cell);
    f.set(Some(GridSelection {
        anchor: cell(1, 0),
        active: cell(2, 1),
    }));
    assert_eq!(f.selection(), Some(single(2, 1)));
    f.set(Some(single(1_000_000, 0)));
    assert_eq!(f.selection(), None);
    f.set(Some(single(0, 8)));
    assert_eq!(f.selection(), None);
    let mut none = Fixture::new(Mode::None);
    none.set(Some(single(1, 1)));
    assert_eq!(none.selection(), None);
}

#[test]
fn widgets_grid_17_click_and_shift_ranges() {
    for mode in [Mode::Row, Mode::Cell, Mode::Range] {
        let mut f = Fixture::new(mode);
        f.t.tap(&Finder::text("r1c1"));
        assert_eq!(f.selection(), Some(single(1, 1)));
        f.t.tap_with_modifiers(&Finder::text("r3c2"), shift());
        assert_eq!(
            f.selection(),
            Some(if mode == Mode::Cell {
                single(3, 2)
            } else {
                GridSelection {
                    anchor: cell(1, 1),
                    active: cell(3, 2),
                }
            })
        );
        f.t.tap(&Finder::text("r2c0"));
        assert_eq!(f.selection(), Some(single(2, 0)));
    }
}

#[test]
fn widgets_grid_18_keys_and_tab() {
    let mut f = Fixture::new(Mode::Range);
    f.t.focus(&Finder::key("grid"));
    assert_eq!(f.selection(), None);
    f.key(NamedKey::ArrowDown, Modifiers::default());
    assert_eq!(f.selection(), Some(single(0, 0)));
    f.key(NamedKey::ArrowDown, shift());
    f.key(NamedKey::ArrowRight, shift());
    assert_eq!(
        f.selection(),
        Some(GridSelection {
            anchor: cell(0, 0),
            active: cell(1, 1)
        })
    );
    f.key(NamedKey::End, Modifiers::default());
    assert_eq!(f.selection(), Some(single(1, 7)));
    f.key(NamedKey::Home, Modifiers::default());
    assert_eq!(f.selection(), Some(single(1, 0)));
    f.key(NamedKey::PageDown, Modifiers::default());
    assert_eq!(f.selection(), Some(single(6, 0)));
    f.key(NamedKey::End, primary());
    assert_eq!(f.selection(), Some(single(999_999, 7)));
    f.key(NamedKey::Home, primary());
    assert_eq!(f.selection(), Some(single(0, 0)));
    f.key(NamedKey::Escape, Modifiers::default());
    assert_eq!(f.selection(), None);
    f.key(NamedKey::Tab, Modifiers::default());
    let tree = f.t.a11y_tree();
    let grid = f.t.find(&Finder::key("grid"));
    assert_eq!(tree.focus.0, grid.to_raw()); // only Tab stop; no per-cell stops
}

#[test]
fn widgets_grid_19_header_commands() {
    let mut f = Fixture::new(Mode::Cell);
    f.set(Some(single(1, 1)));
    f.t.focus(&Finder::key("grid"));
    f.key(NamedKey::F(6), Modifiers::default());
    f.key(
        NamedKey::ArrowRight,
        Modifiers {
            alt: true,
            ..Default::default()
        },
    );
    assert_eq!(f.t.enter(|| f.h.widths.get())[1], 130.0);
    let mut reorder = primary();
    reorder.shift = true;
    f.key(NamedKey::ArrowRight, reorder);
    assert_eq!(f.t.enter(|| f.h.order.get()), vec![0, 2, 1, 3, 4, 5, 6, 7]);
    assert_eq!(f.selection(), None);
    f.key(NamedKey::F(6), Modifiers::default());
    f.key(NamedKey::ArrowDown, Modifiers::default());
    assert_eq!(f.selection(), Some(single(0, 0)));
}

#[test]
fn widgets_grid_20_reveal_and_virtualization() {
    let mut f = Fixture::new(Mode::Range);
    f.set(Some(GridSelection {
        anchor: cell(0, 0),
        active: cell(999_999, 7),
    }));
    assert_eq!(f.t.enter(|| f.h.vertical.offset()), 32_000_000.0 - 164.0);
    assert_eq!(f.t.enter(|| f.h.horizontal.offset()), 600.0);
    let nodes = f.t.a11y_tree();
    assert!(
        nodes
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Cell)
            .count()
            < 60
    );
    let x = f.t.enter(|| f.h.horizontal.offset());
    f.set(Some(single(500, 0)));
    assert_eq!(f.t.enter(|| f.h.horizontal.offset()), x);
}

#[test]
fn widgets_grid_20_selection_paint() {
    let mut f=Fixture::new(Mode::Row);
    f.set(Some(single(2,1)));
    let colors=f.t.enter(||f.t.theme().get().colors);
    let mix=|base:f32,accent:f32|base*0.88+accent*0.12;
    let expected=tantu_view::core::Color {r:mix(colors.surface.r,colors.primary.r),
        g:mix(colors.surface.g,colors.primary.g),b:mix(colors.surface.b,colors.primary.b),a:colors.surface.a};
    assert!(f.t.frame().scene.entries().iter().any(|entry|matches!(entry.command,
        tantu_view::scene::Command::Fill {color,..} if color==expected)));
}

#[test]
fn widgets_grid_21_model_reset_and_mapping() {
    let mut f = Fixture::new(Mode::Row);
    assert_eq!(
        f.t.enter(|| f.h.selection.source_cell(cell(20, 2))),
        Some(cell(20, 2))
    );
    f.set(Some(single(2, 1)));
    f.t.enter(|| f.h.widths.set(vec![140.0; 8]));
    f.t.pump();
    assert_eq!(f.selection(), Some(single(2, 1)));
    f.t.enter(|| f.h.order.set(vec![2, 0, 1]));
    f.t.pump();
    f.t.pump();
    assert_eq!(f.selection(), None);
    assert_eq!(
        f.t.enter(|| f.h.selection.source_cell(cell(20, 0))),
        Some(cell(20, 2))
    );
    f.set(Some(single(2, 1)));
    f.t.enter(|| f.h.revision.update(|r| *r += 1));
    f.t.pump();
    assert_eq!(f.selection(), None);
    assert_eq!(
        f.t.enter(|| f.h.selection.source_cell(cell(1_000_000, 0))),
        None
    );
}

#[test]
fn widgets_grid_22_accessibility_values_and_active_cell() {
    let mut f = Fixture::new(Mode::Range);
    f.set(Some(single(2, 1)));
    f.t.focus(&Finder::key("grid"));
    let update = f.t.a11y_tree();
    let (_, grid) = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Grid)
        .expect("grid");
    assert_eq!(grid.row_count(), Some(1_000_000));
    assert_eq!(grid.column_count(), Some(8));
    assert!(grid.is_multiselectable());
    let (id, node) = update
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Cell && n.value() == Some("r2c1"))
        .expect("cell");
    assert_eq!(node.row_index(), Some(2));
    assert_eq!(node.column_index(), Some(1));
    assert_eq!(node.is_selected(), Some(true));
    assert!(node.supports_action(Action::Click));
    assert_eq!(grid.active_descendant(), Some(*id));
    assert!(f.t.perform_action(&Finder::key("grid"), SemanticsAction::Focus));
}
