//! Cell-editor behavior from WIDGETS-GRID-23..28.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use tantu_test::accesskit::{Node, NodeId, Role};
use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Point, Vec2};
use tantu_view::reactive::{Signal, signal};
use tantu_view::{
    ImeEvent, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
};
use tantu_widgets::{
    Button, CellValue, Column, DataGrid, DataSource, FnSource, GridCell, GridColumn, GridEditError,
    GridSelection, GridSelectionController, GridSelectionMode, RowFilter, ScrollController,
    SizedBox, SortState, Text,
};

#[derive(Default)]
struct Store {
    values: RefCell<HashMap<(usize, usize), CellValue>>,
    writes: RefCell<Vec<(usize, usize, CellValue)>>,
    reject: Cell<bool>,
    reads: Cell<usize>,
}
impl DataSource for Store {
    fn row_count(&self) -> usize {
        1_000
    }
    fn cell(&self, row: usize, column: usize) -> CellValue {
        self.reads.set(self.reads.get() + 1);
        self.values
            .borrow()
            .get(&(row, column))
            .cloned()
            .unwrap_or_else(|| match column {
                0 => CellValue::Int(row as i64),
                1 => CellValue::Text(format!("row {row}")),
                2 => CellValue::Int(row as i64 + 10),
                3 => CellValue::Float(1.5),
                4 => CellValue::Bool(false),
                _ => CellValue::Empty,
            })
    }
    fn set_cell(&self, row: usize, column: usize, value: CellValue) -> Result<(), GridEditError> {
        self.writes.borrow_mut().push((row, column, value.clone()));
        if self.reject.get() {
            return Err(GridEditError::Rejected("Try again".into()));
        }
        self.values.borrow_mut().insert((row, column), value);
        Ok(())
    }
}
#[derive(Clone)]
struct Handles {
    selection: GridSelectionController,
    vertical: ScrollController,
    horizontal: ScrollController,
    order: Signal<Vec<usize>>,
    widths: Signal<Vec<f32>>,
    revision: Signal<u64>,
    sort: Signal<Option<SortState>>,
    filter: Signal<Option<RowFilter>>,
    show: Signal<bool>,
}
struct Fixture {
    t: WidgetTester,
    h: Handles,
    store: Rc<Store>,
    errors: Rc<RefCell<Vec<(GridCell, GridEditError)>>>,
}
impl Fixture {
    fn new() -> Self {
        Self::with(|grid| grid)
    }
    fn with(configure: impl Fn(DataGrid) -> DataGrid + 'static) -> Self {
        let store = Rc::new(Store::default());
        let source = store.clone();
        let errors = Rc::new(RefCell::new(Vec::new()));
        let report = errors.clone();
        let out = Rc::new(RefCell::new(None));
        let capture = out.clone();
        let mut t = WidgetTester::with_size(420.0, 260.0, move || {
            let h = Handles {
                selection: GridSelectionController::new(),
                vertical: ScrollController::new(),
                horizontal: ScrollController::new(),
                order: signal((0..6).collect()),
                widths: signal(vec![120.0; 6]),
                revision: signal(0),
                sort: signal(None),
                filter: signal(None),
                show: signal(true),
            };
            *capture.borrow_mut() = Some(h.clone());
            let (source, report, build_h) = (source.clone(), report.clone(), h.clone());
            Column::new()
                .child(Keyed::new("before", Button::new("Before")))
                .child(SizedBox::new(420.0, 180.0).child(tantu_view::Show::new(
                    move || h.show.get(),
                    move || {
                        let columns = (0..6).map(|c| {
                            let column = GridColumn::new(format!("C{c}")).editable(c != 0);
                            if c == 1 {
                                column.builder(|_, v| Text::new(v.to_string()))
                            } else {
                                column
                            }
                        });
                        let report = report.clone();
                        Keyed::new(
                            "grid",
                            configure(
                                DataGrid::new(columns, source.clone())
                                    .selection_mode(GridSelectionMode::Cell)
                                    .selection_controller(build_h.selection.clone())
                                    .controllers(build_h.vertical, build_h.horizontal)
                                    .column_order(build_h.order)
                                    .column_widths(build_h.widths)
                                    .revision(build_h.revision)
                                    .sort_state(build_h.sort)
                                    .filter(build_h.filter)
                                    .frozen_columns(1)
                                    .wheel_duration(Duration::ZERO)
                                    .on_edit_error(move |cell, error| {
                                        report.borrow_mut().push((cell, error))
                                    }),
                            ),
                        )
                    },
                )))
                .child(Keyed::new("after", Button::new("After")))
        });
        t.pump();
        let h = out.borrow_mut().take().expect("built");
        Self {
            t,
            h,
            store,
            errors,
        }
    }
    fn key(&mut self, key: NamedKey) {
        self.t
            .press_key(LogicalKey::Named(key), Modifiers::default());
        self.t.pump();
    }
    fn select(&mut self, row: usize, column: usize) {
        self.t.enter(|| {
            self.h
                .selection
                .set_selection(Some(GridSelection::single(GridCell { row, column })))
        });
        self.t.pump();
        self.t.pump();
    }
    fn open(&mut self, row: usize, column: usize) {
        self.select(row, column);
        self.t.focus(&Finder::key("grid"));
        self.key(NamedKey::F(2));
        assert_eq!(self.fields().len(), 1, "F2 opens one editor");
    }
    fn fields(&mut self) -> Vec<(NodeId, Node)> {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .filter(|(_, n)| n.role() == Role::TextInput)
            .collect()
    }
    fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.t.press_key(
                LogicalKey::Character(c.to_string().into()),
                Modifiers::default(),
            );
        }
    }
    fn assert_grid_focus(&mut self) {
        let grid = self.t.find(&Finder::key("grid"));
        assert_eq!(self.t.a11y_tree().focus.0, grid.to_raw());
    }
}

#[test]
fn widgets_grid_23_read_only_contract_and_opt_in() {
    let source = FnSource::new(1, |_, _| CellValue::Empty);
    assert_eq!(
        source.set_cell(0, 0, CellValue::Int(1)),
        Err(GridEditError::ReadOnly)
    );
    assert_eq!(GridEditError::ReadOnly.to_string(), "Cell is read-only");
    assert_eq!(
        GridEditError::InvalidValue.to_string(),
        "Invalid cell value"
    );
    assert_eq!(
        GridEditError::Rejected("reason".into()).to_string(),
        "reason"
    );
    let _: &dyn std::error::Error = &GridEditError::ReadOnly;
    let mut f = Fixture::new();
    f.select(0, 0);
    f.t.focus(&Finder::key("grid"));
    f.key(NamedKey::F(2));
    assert!(f.fields().is_empty());
    assert!(f.store.writes.borrow().is_empty());
}

#[test]
fn widgets_grid_24_pointer_entry_none_and_expiry() {
    let mut f = Fixture::with(|g| g.selection_mode(GridSelectionMode::None));
    f.t.tap(&Finder::text("row 1"));
    f.t.tap(&Finder::text("row 1"));
    f.t.pump();
    assert_eq!(f.fields().len(), 1);
    assert_eq!(f.t.enter(|| f.h.selection.selection()), None);
    f.key(NamedKey::Escape);
    f.t.tap(&Finder::text("row 1"));
    f.t.advance(Duration::from_millis(501));
    f.t.tap(&Finder::text("row 1"));
    assert!(f.fields().is_empty());
    f.t.tap_with_modifiers(
        &Finder::text("row 1"),
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert!(f.fields().is_empty());
    assert!(f.store.writes.borrow().is_empty());
}

#[test]
fn widgets_grid_24_sorted_filtered_reordered_source_coordinates() {
    let mut f = Fixture::with(|g| g.client_sort(true));
    f.t.enter(|| {
        f.h.order.set(vec![1, 2, 0, 3, 4, 5]);
        f.h.filter.set(Some(Rc::new(|_, r| r < 4 && r != 1)));
        f.h.sort.set(Some(SortState {
            column: 2,
            ascending: false,
        }));
    });
    f.t.pump();
    f.t.pump();
    f.open(0, 0);
    f.type_text("mapped");
    f.key(NamedKey::Enter);
    assert_eq!(
        &*f.store.writes.borrow(),
        &[(3, 1, CellValue::Text("mapped".into()))]
    );
    assert!(f.fields().is_empty());
}

#[test]
fn widgets_grid_25_field_focus_size_selection_and_semantics() {
    let mut f = Fixture::new();
    f.open(2, 1);
    let (id, node) = f.fields().pop().expect("field");
    assert_eq!(node.label(), Some("C1, row 2"));
    assert_eq!(node.value(), Some("row 2"));
    let bounds = node.bounds().expect("bounds");
    assert_eq!(bounds.width(), 120.0);
    assert_eq!(bounds.height(), 32.0);
    let tree = f.t.a11y_tree();
    assert_eq!(tree.focus, id);
    let grid = tree
        .nodes
        .iter()
        .find(|(_, n)| n.role() == Role::Grid)
        .expect("grid");
    assert!(grid.1.active_descendant().is_none());
    f.type_text("new");
    assert_eq!(f.fields()[0].1.value(), Some("new")); // entry selected all
    let selection = f.t.enter(|| f.h.selection.selection());
    f.key(NamedKey::PageDown);
    f.key(NamedKey::F(6));
    assert_eq!(f.t.enter(|| f.h.selection.selection()), selection);
    assert_eq!(f.fields().len(), 1);
    assert!(f.t.find_all(&Finder::text("row 2")).is_empty()); // custom builder replaced
}

#[test]
fn widgets_grid_26_typed_conversion_and_invalid_drafts() {
    for (column, input, expected) in [
        (1, "  words  ", CellValue::Text("  words  ".into())),
        (2, " -42 ", CellValue::Int(-42)),
        (3, " 2.25 ", CellValue::Float(2.25)),
        (4, " TRUE ", CellValue::Bool(true)),
        (5, "text", CellValue::Text("text".into())),
    ] {
        let mut f = Fixture::new();
        f.open(0, column);
        f.type_text(input);
        f.key(NamedKey::Enter);
        assert_eq!(f.store.cell(0, column), expected);
        assert!(f.fields().is_empty());
    }
    for (column, input) in [
        (2, "9223372036854775808"),
        (2, "2.5"),
        (3, "NaN"),
        (3, "inf"),
        (3, "-inf"),
        (3, "1e999"),
        (4, "yes"),
    ] {
        let mut f = Fixture::new();
        f.open(0, column);
        f.type_text(input);
        f.key(NamedKey::Enter);
        assert!(f.store.writes.borrow().is_empty());
        assert_eq!(
            f.errors.borrow().last().expect("error").1,
            GridEditError::InvalidValue
        );
        assert_eq!(f.fields()[0].1.value(), Some(input));
    }
    for column in [1, 2, 3, 4, 5] {
        let mut f = Fixture::new();
        f.open(0, column);
        f.key(NamedKey::Backspace);
        f.key(NamedKey::Enter);
        if column == 1 {
            assert_eq!(f.store.cell(0, 1), CellValue::Text(String::new()));
        } else if column == 5 {
            assert!(f.fields().is_empty());
            assert!(f.store.writes.borrow().is_empty());
        } else {
            assert_eq!(f.errors.borrow()[0].1, GridEditError::InvalidValue);
        }
    }
}

#[test]
fn widgets_grid_27_rejection_retry_local_refresh_and_unchanged() {
    let mut f = Fixture::new();
    f.open(2, 1);
    f.type_text("accepted");
    f.store.reject.set(true);
    f.key(NamedKey::Enter);
    assert_eq!(f.store.cell(2, 1), CellValue::Text("row 2".into()));
    assert_eq!(f.fields()[0].1.value(), Some("accepted"));
    assert_eq!(
        &*f.errors.borrow(),
        &[(
            GridCell { row: 2, column: 1 },
            GridEditError::Rejected("Try again".into())
        )]
    );
    f.store.reject.set(false);
    f.key(NamedKey::Enter);
    f.assert_grid_focus();
    assert!(f.fields().is_empty());
    f.t.find(&Finder::text("accepted"));
    assert_eq!(f.t.enter(|| f.h.revision.get()), 0);
    assert_eq!(
        f.t.enter(|| f.h.selection.selection())
            .expect("preserved")
            .active,
        GridCell { row: 2, column: 1 }
    );
    let count = f.store.writes.borrow().len();
    f.key(NamedKey::Enter);
    assert_eq!(f.store.writes.borrow().len(), count);
    f.open(2, 1);
    f.key(NamedKey::Enter);
    assert_eq!(f.store.writes.borrow().len(), count);
}

#[test]
fn widgets_grid_27_ime_and_uncommitted_keystrokes() {
    let mut f = Fixture::new();
    f.open(1, 1);
    let reads = f.store.reads.get();
    f.type_text("draft");
    assert_eq!(f.store.reads.get(), reads);
    assert!(f.store.writes.borrow().is_empty());
    f.t.ime(ImeEvent::Preedit {
        text: "語".into(),
        cursor: None,
    });
    f.key(NamedKey::Enter);
    assert!(f.store.writes.borrow().is_empty());
    assert_eq!(f.fields().len(), 1);
    f.t.ime(ImeEvent::Commit("語".into()));
    f.key(NamedKey::Enter);
    assert_eq!(f.store.writes.borrow().len(), 1);
    assert!(f.fields().is_empty());
}

#[test]
fn widgets_grid_27_refresh_sort_filter_and_clear_selection() {
    let mut f = Fixture::with(|g| g.client_sort(true));
    f.t.enter(|| {
        f.h.filter.set(Some(Rc::new(|source, r| {
            r < 3 && source.cell(r, 2) != CellValue::Int(99)
        })));
        f.h.sort.set(Some(SortState {
            column: 2,
            ascending: true,
        }));
    });
    f.t.pump();
    f.open(0, 2);
    f.type_text("99");
    f.key(NamedKey::Enter);
    assert_eq!(f.store.cell(0, 2), CellValue::Int(99));
    assert_eq!(f.t.enter(|| f.h.selection.selection()), None);
    assert!(f.t.find_all(&Finder::text("99")).is_empty());
    f.t.find(&Finder::text("row 1"));
}

#[test]
fn widgets_grid_28_escape_blur_and_tab() {
    let mut f = Fixture::new();
    f.open(1, 1);
    f.type_text("discard");
    f.key(NamedKey::Escape);
    assert!(f.fields().is_empty());
    f.assert_grid_focus();
    assert_eq!(f.store.cell(1, 1), CellValue::Text("row 1".into()));
    for shift in [false, true] {
        f.open(1, 1);
        f.t.press_key(
            LogicalKey::Named(NamedKey::Tab),
            Modifiers {
                shift,
                ..Default::default()
            },
        );
        f.t.pump();
        let target =
            f.t.find(&Finder::key(if shift { "before" } else { "after" }));
        assert_eq!(f.t.a11y_tree().focus.0, target.to_raw());
        assert!(f.fields().is_empty());
    }
    f.open(1, 1);
    f.t.focus(&Finder::key("after"));
    f.t.pump();
    let after = f.t.find(&Finder::key("after"));
    assert_eq!(f.t.a11y_tree().focus.0, after.to_raw());
    assert!(f.fields().is_empty());
    assert!(f.store.writes.borrow().is_empty());
}

#[test]
fn widgets_grid_28_model_and_selection_changes_cancel() {
    for change in 0..5 {
        let mut f = Fixture::new();
        f.open(1, 1);
        f.type_text("discard");
        f.t.enter(|| match change {
            0 => f.h.revision.update(|r| *r += 1),
            1 => f.h.order.set(vec![1, 0, 2, 3, 4, 5]),
            2 => f.h.sort.set(Some(SortState {
                column: 1,
                ascending: true,
            })),
            3 => f.h.filter.set(Some(Rc::new(|_, r| r != 0))),
            _ => f.h.selection.set_selection(None),
        });
        f.t.pump();
        f.t.pump();
        assert!(f.fields().is_empty(), "change {change}");
        f.assert_grid_focus();
        f.key(NamedKey::Enter);
        assert!(f.store.writes.borrow().is_empty());
    }
}

#[test]
fn widgets_grid_28_viewport_width_and_removal() {
    let mut f = Fixture::new();
    f.open(1, 1);
    f.type_text("draft");
    f.t.enter(|| {
        f.h.widths.update(|w| w[1] = 140.0);
        f.h.vertical.jump_to(40.0);
    });
    f.t.pump();
    assert_eq!(f.fields()[0].1.value(), Some("draft"));
    // row 1 (y=32..64) remains built by the buffer at offset 65 but is no longer visible.
    f.t.enter(|| f.h.vertical.jump_to(65.0));
    f.t.pump();
    f.t.pump();
    assert!(f.fields().is_empty());
    f.assert_grid_focus();
    f.open(3, 1);
    f.t.enter(|| f.h.horizontal.jump_to(200.0));
    f.t.pump();
    f.t.pump();
    assert!(f.fields().is_empty());
    f.open(3, 1);
    f.t.enter(|| f.h.show.set(false));
    f.t.pump();
    f.t.advance(Duration::from_secs(1));
    assert!(f.fields().is_empty());
    assert!(f.store.writes.borrow().is_empty());
    f.t.enter(|| f.h.show.set(true));
    f.t.pump();
    f.t.pump();
    f.open(0, 1);
    assert_eq!(f.fields().len(), 1);
}

#[test]
fn widgets_grid_24_drag_resets_double_click_candidate() {
    let mut f = Fixture::new();
    let rect = f.t.rect(f.t.find(&Finder::text("row 1")));
    let position = Point::new(
        (rect.left + rect.right) / 2.0,
        (rect.top + rect.bottom) / 2.0,
    );
    for (kind, p) in [
        (PointerKind::Down(PointerButton::Primary), position),
        (PointerKind::Move, position + Vec2::new(8.0, 0.0)),
        (PointerKind::Up(PointerButton::Primary), position),
    ] {
        f.t.pointer(PointerEvent {
            kind,
            position: p,
            modifiers: Modifiers::default(),
        });
    }
    f.t.pump();
    f.t.tap(&Finder::text("row 1"));
    assert!(f.fields().is_empty());
    f.t.tap(&Finder::text("row 1"));
    f.t.pump();
    assert_eq!(f.fields().len(), 1);
}

#[test]
fn widgets_grid_24_double_click_includes_500_ms_boundary() {
    let mut f = Fixture::new();
    f.t.tap(&Finder::text("row 1"));
    f.t.advance(Duration::from_millis(500));
    f.t.tap(&Finder::text("row 1"));
    f.t.pump();
    assert_eq!(f.fields().len(), 1);
}

#[test]
fn widgets_grid_24_model_reset_clears_pending_double_click() {
    let mut f = Fixture::new();
    f.t.tap(&Finder::text("row 1"));
    f.t.enter(|| f.h.revision.update(|r| *r += 1));
    f.t.pump();
    f.t.tap(&Finder::text("row 1"));
    assert!(f.fields().is_empty());
    f.t.tap(&Finder::text("row 1"));
    f.t.pump();
    assert_eq!(f.fields().len(), 1);
}

#[test]
fn widgets_grid_27_unchanged_nonfinite_value_and_frozen_edit() {
    let mut f = Fixture::new();
    f.store
        .values
        .borrow_mut()
        .insert((0, 3), CellValue::Float(f64::NAN));
    f.t.enter(|| f.h.revision.update(|r| *r += 1));
    f.t.pump();
    f.open(0, 3);
    f.key(NamedKey::Enter);
    assert!(f.store.writes.borrow().is_empty());
    assert!(f.fields().is_empty());
    f.t.enter(|| f.h.order.set(vec![1, 0, 2, 3, 4, 5]));
    f.t.pump();
    f.open(1, 0);
    f.type_text("pinned");
    f.t.enter(|| f.h.horizontal.jump_to(200.0));
    f.t.pump();
    assert_eq!(f.fields()[0].1.value(), Some("pinned"));
    f.key(NamedKey::Enter);
    assert_eq!(
        &*f.store.writes.borrow(),
        &[(1, 1, CellValue::Text("pinned".into()))]
    );
}

#[test]
fn widgets_grid_28_raw_submit_after_revision_does_not_write_stale_record() {
    let mut f = Fixture::new();
    f.open(1, 1);
    f.type_text("stale");
    // Change the model and send Enter before any frame applies cancellation bindings.
    f.t.enter(|| f.h.revision.update(|r| *r += 1));
    f.key(NamedKey::Enter);
    assert!(f.store.writes.borrow().is_empty());
    assert!(f.fields().is_empty());
    f.assert_grid_focus();
}
