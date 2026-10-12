//! Tests for `docs/specs/widgets/data-grid.md`, rules WIDGETS-GRID-01..07 and 09 (08 is in
//! `bench_scenarios.rs`), and `docs/specs/widgets/data-grid-sort.md`, WIDGETS-GRID-10..12.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use tantu_test::accesskit::{NodeId, Role};
use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Color, Point, Rect, Vec2};
use tantu_view::layout::RenderParagraph;
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{Clip, Command};
use tantu_view::{
    Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
};
use tantu_widgets::{
    CellValue, Column, DataGrid, DataSource, FnSource, GridColumn, RowFilter, ScrollController,
    ScrollView, SizedBox, SortState, Text,
};

const THUMB: Color = Color::from_argb32(0x661C_1B1F);

/// `rows` rows whose cell (r, c) reads "r{r}c{c}".
fn source(rows: usize) -> Rc<dyn DataSource> {
    Rc::new(FnSource::new(rows, |r, c| {
        CellValue::Text(format!("r{r}c{c}"))
    }))
}

/// `n` columns titled "Col 0".., 120 px each.
fn columns(n: usize) -> Vec<GridColumn> {
    (0..n)
        .map(|c| GridColumn::new(format!("Col {c}")))
        .collect()
}

type Two = Rc<Cell<Option<(ScrollController, ScrollController)>>>;

/// A `w` × `h` window holding the grid `make` returns, keyed "g", with wheel steps applied
/// at once; returns its vertical and horizontal controllers.
fn grid(
    w: f32,
    h: f32,
    make: impl FnOnce() -> DataGrid + 'static,
) -> (WidgetTester, ScrollController, ScrollController) {
    let out: Two = Rc::default();
    let make = Cell::new(Some(make));
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(w, h, move || {
            let (vertical, horizontal) = (ScrollController::new(), ScrollController::new());
            out.set(Some((vertical, horizontal)));
            let make = make.take().expect("built once");
            Keyed::new(
                "g",
                make()
                    .controllers(vertical, horizontal)
                    .wheel_duration(Duration::ZERO),
            )
        })
    };
    let (vertical, horizontal) = out.get().expect("built");
    // The cells appear one frame after the first paint records the extents, as list items.
    tester.pump();
    (tester, vertical, horizontal)
}

fn offset(tester: &WidgetTester, c: ScrollController) -> f32 {
    tester.enter(|| c.offset())
}

/// Every paragraph's text, in tree order.
fn texts(tester: &WidgetTester) -> Vec<String> {
    tester
        .find_all(&Finder::render::<RenderParagraph>())
        .into_iter()
        .filter_map(|e| tester.text(e))
        .collect()
}

fn has(tester: &WidgetTester, text: &str) -> bool {
    !tester.find_all(&Finder::text(text)).is_empty()
}

fn rect(tester: &WidgetTester, text: &str) -> Rect {
    tester.rect(tester.find(&Finder::text(text)))
}

fn fills(tester: &WidgetTester, color: Color) -> Vec<Rect> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { shape, color: c } if c == color => Some(shape.rect),
            _ => None,
        })
        .collect()
}

#[test]
fn widgets_grid_01_structure_and_sizing() {
    let (tester, _, _) = grid(600.0, 400.0, || {
        DataGrid::new(
            [
                GridColumn::new("Id"),
                GridColumn::new("Name"),
                GridColumn::new("City"),
            ],
            source(1000),
        )
    });
    assert_eq!(
        tester.rect(tester.find(&Finder::key("g"))),
        Rect::from_ltwh(0.0, 0.0, 600.0, 400.0)
    );
    let id = rect(&tester, "Id");
    assert!(
        id.left >= 0.0 && id.right <= 120.0 && id.bottom <= 36.0,
        "{id:?}"
    );
    let name = rect(&tester, "Name");
    assert!(
        name.left >= 120.0 && name.right <= 240.0 && name.bottom <= 36.0,
        "{name:?}"
    );
    let cell = rect(&tester, "r0c0");
    assert!(
        cell.top >= 36.0 && cell.bottom <= 68.0 && cell.right <= 120.0,
        "{cell:?}"
    );
    let next = rect(&tester, "r1c1");
    assert!(next.top >= 68.0 && next.left >= 120.0, "{next:?}");
    // No header.
    let (tester, _, _) = grid(600.0, 400.0, || {
        DataGrid::new(columns(3), source(1000)).header_height(0.0)
    });
    assert!(!has(&tester, "Col 0"));
    assert!(rect(&tester, "r0c0").bottom <= 32.0);
    // The default texts.
    assert_eq!(CellValue::Empty.to_string(), "");
    assert_eq!(CellValue::Text("a".into()).to_string(), "a");
    assert_eq!(CellValue::Int(-3).to_string(), "-3");
    assert_eq!(CellValue::Float(2.5).to_string(), "2.5");
    assert_eq!(CellValue::Bool(true).to_string(), "true");
    assert_eq!(CellValue::Bool(false).to_string(), "false");
}

#[test]
fn widgets_grid_02_virtualization() {
    let calls = Rc::new(Cell::new(0usize));
    let (mut tester, vertical, _) = grid(600.0, 400.0, {
        let calls = calls.clone();
        move || {
            DataGrid::new(
                columns(20),
                Rc::new(FnSource::new(1_000_000, move |r, c| {
                    calls.set(calls.get() + 1);
                    CellValue::Text(format!("r{r}c{c}"))
                })),
            )
        }
    });
    let body = |t: &WidgetTester| texts(t).iter().filter(|s| s.starts_with('r')).count();
    let header = |t: &WidgetTester| texts(t).iter().filter(|s| s.starts_with("Col ")).count();
    // 12 visible rows + 2, 5 visible columns + 2.
    assert!(body(&tester) <= 14 * 7, "{} body cells", body(&tester));
    assert!(header(&tester) <= 7, "{} header cells", header(&tester));
    assert!(calls.get() <= 14 * 7, "{} source calls", calls.get());
    assert!(has(&tester, "r12c0") && !has(&tester, "r13c0"));
    assert!(has(&tester, "r0c5") && !has(&tester, "r0c6"));
    assert!(has(&tester, "Col 5") && !has(&tester, "Col 6"));
    let r5 = tester.find(&Finder::text("r5c0"));
    // One row down: one row leaves the range at the top (after the buffer), one enters.
    tester.scroll(&Finder::key("g"), Vec2::new(0.0, 32.0));
    assert_eq!(offset(&tester, vertical), 32.0);
    assert!(has(&tester, "r0c0") && has(&tester, "r13c0") && !has(&tester, "r14c0"));
    tester.scroll(&Finder::key("g"), Vec2::new(0.0, 32.0));
    assert!(!has(&tester, "r0c0") && has(&tester, "r14c0"));
    assert!(body(&tester) <= 14 * 7);
    // Cells that stayed in the range kept their elements.
    assert_eq!(tester.find(&Finder::text("r5c0")), r5);
}

#[test]
fn widgets_grid_03_column_builder_and_clipping() {
    let (tester, _, _) = grid(600.0, 400.0, || {
        DataGrid::new(
            [
                GridColumn::new("Narrow").width(40.0).builder(|_, _| {
                    // A builder decides its own wrapping; one unwrapped line overflows.
                    Text::new("a cell text much wider than its forty pixel column")
                        .soft_wrap(false)
                        .max_lines(1)
                }),
                GridColumn::new("B").builder(|row, value| Text::new(format!("B{row}:{value}"))),
                GridColumn::new("C"),
            ],
            source(10),
        )
    });
    assert!(has(&tester, "B0:r0c1") && has(&tester, "B9:r9c1"));
    assert!(!has(&tester, "r0c1"));
    assert!(has(&tester, "r0c2"));
    // The header keeps its title.
    assert!(has(&tester, "B") && has(&tester, "Narrow"));
    // The wide content overflows its 40 px cell and is clipped to it.
    // (The paragraph's rect is constrained to the cell, so the overflow shows only in the
    // glyphs; what is checkable is the clip.)
    assert!(has(
        &tester,
        "a cell text much wider than its forty pixel column"
    ));
    let clipped = tester
        .frame()
        .scene
        .entries()
        .iter()
        .any(|e| matches!(e.command, Command::PushClip(Clip::Rect(r)) if r.width() == 40.0 && r.height() == 32.0));
    assert!(clipped, "no 40 × 32 clip");
}

#[test]
fn widgets_grid_04_scrolling() {
    let (mut tester, vertical, horizontal) =
        grid(600.0, 400.0, || DataGrid::new(columns(20), source(1000)));
    assert_eq!(
        tester.enter(|| (vertical.viewport_extent(), vertical.max_scroll_extent())),
        (364.0, 32_000.0 - 364.0)
    );
    assert_eq!(
        tester.enter(|| (horizontal.viewport_extent(), horizontal.max_scroll_extent())),
        (600.0, 1800.0)
    );
    let header = rect(&tester, "Col 0");
    let cell = rect(&tester, "r3c0");
    // y scrolls the rows (row 0 leaves the built range); the header stays.
    tester.scroll(&Finder::key("g"), Vec2::new(0.0, 100.0));
    assert_eq!(offset(&tester, vertical), 100.0);
    assert_eq!(rect(&tester, "Col 0"), header);
    assert_eq!(rect(&tester, "r3c0").top, cell.top - 100.0);
    assert!(!has(&tester, "r0c0"));
    // x scrolls the columns, header included.
    tester.scroll(&Finder::key("g"), Vec2::new(240.0, 0.0));
    assert_eq!(offset(&tester, horizontal), 240.0);
    assert_eq!(rect(&tester, "Col 2").left, header.left);
    assert_eq!(rect(&tester, "r3c2").left, cell.left);
    assert!(!has(&tester, "Col 0"));
    // Clamped.
    tester.scroll(&Finder::key("g"), Vec2::new(0.0, 1e6));
    assert_eq!(offset(&tester, vertical), 32_000.0 - 364.0);
    tester.scroll(&Finder::key("g"), Vec2::new(1e6, 0.0));
    assert_eq!(offset(&tester, horizontal), 1800.0);
    tester.scroll(&Finder::key("g"), Vec2::new(-1e6, -1e6));
    assert_eq!(
        (offset(&tester, vertical), offset(&tester, horizontal)),
        (0.0, 0.0)
    );
    // Two scrollbars when both axes overflow; the vertical thumb is dragged.
    assert_eq!(fills(&tester, THUMB).len(), 2);
    let thumb = fills(&tester, THUMB)
        .into_iter()
        .find(|r| r.height() > r.width())
        .expect("a vertical thumb");
    assert!(thumb.left >= 590.0 && thumb.top >= 36.0, "{thumb:?}");
    let start = thumb.center();
    tester.pointer(PointerEvent {
        kind: PointerKind::Down(PointerButton::Primary),
        position: start,
    });
    tester.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(start.x, start.y + 50.0),
    });
    tester.pointer(PointerEvent {
        kind: PointerKind::Up(PointerButton::Primary),
        position: Point::new(start.x, start.y + 50.0),
    });
    tester.pump();
    // track 360, thumb 24 (the minimum): 50 px of track are 50 · max / 336 of content.
    let expected = 50.0 * (32_000.0 - 364.0) / 336.0;
    assert!((offset(&tester, vertical) - expected).abs() < 1.0);
    // No scrollbar without overflow.
    let (tester, _, _) = grid(600.0, 400.0, || DataGrid::new(columns(3), source(5)));
    assert!(fills(&tester, THUMB).is_empty());
    // A wheel the grid can't use continues to an outer scroll view.
    let outer: Rc<Cell<Option<ScrollController>>> = Rc::default();
    let mut tester = {
        let outer = outer.clone();
        WidgetTester::with_size(600.0, 400.0, move || {
            let controller = ScrollController::new();
            outer.set(Some(controller));
            ScrollView::new(
                Column::new()
                    .child(SizedBox::new(600.0, 200.0).child(Keyed::new(
                        "g",
                        DataGrid::new(columns(3), source(5)).wheel_duration(Duration::ZERO),
                    )))
                    .child(SizedBox::new(600.0, 1000.0)),
            )
            .controller(controller)
            .wheel_duration(Duration::ZERO)
        })
    };
    let controller = outer.get().expect("built");
    tester.scroll(&Finder::key("g"), Vec2::new(0.0, 50.0));
    assert_eq!(offset(&tester, controller), 50.0);
}

#[test]
fn widgets_grid_05_keys() {
    let (mut tester, vertical, horizontal) =
        grid(600.0, 400.0, || DataGrid::new(columns(20), source(1000)));
    let key =
        |t: &mut WidgetTester, k: NamedKey| t.press_key(LogicalKey::Named(k), Modifiers::default());
    // A press focuses the grid.
    tester.tap(&Finder::text("r2c1"));
    key(&mut tester, NamedKey::ArrowDown);
    assert_eq!(offset(&tester, vertical), 32.0);
    key(&mut tester, NamedKey::ArrowRight);
    assert_eq!(offset(&tester, horizontal), 40.0);
    key(&mut tester, NamedKey::PageDown);
    assert_eq!(offset(&tester, vertical), 32.0 + 364.0);
    key(&mut tester, NamedKey::End);
    assert_eq!(offset(&tester, vertical), 32_000.0 - 364.0);
    key(&mut tester, NamedKey::PageUp);
    assert_eq!(offset(&tester, vertical), 32_000.0 - 364.0 - 364.0);
    key(&mut tester, NamedKey::Home);
    assert_eq!(offset(&tester, vertical), 0.0);
    key(&mut tester, NamedKey::ArrowUp);
    assert_eq!(offset(&tester, vertical), 0.0);
    key(&mut tester, NamedKey::ArrowLeft);
    assert_eq!(offset(&tester, horizontal), 0.0);
}

/// A source over shared rows of text, for change tests.
struct VecSource(Rc<RefCell<Vec<Vec<String>>>>);

impl DataSource for VecSource {
    fn row_count(&self) -> usize {
        self.0.borrow().len()
    }

    fn cell(&self, row: usize, column: usize) -> CellValue {
        match self.0.borrow().get(row).and_then(|r| r.get(column)) {
            Some(text) => CellValue::Text(text.clone()),
            None => CellValue::Empty,
        }
    }
}

fn rows(n: usize) -> Rc<RefCell<Vec<Vec<String>>>> {
    Rc::new(RefCell::new(
        (0..n)
            .map(|r| (0..2).map(|c| format!("r{r}c{c}")).collect())
            .collect(),
    ))
}

#[test]
fn widgets_grid_06_revision() {
    let data = rows(20);
    let rev: Rc<Cell<Option<Signal<u64>>>> = Rc::default();
    let (mut tester, vertical, _) = grid(600.0, 400.0, {
        let (data, rev) = (data.clone(), rev.clone());
        move || {
            let revision = signal(0u64);
            rev.set(Some(revision));
            DataGrid::new(
                [
                    GridColumn::new("A"),
                    GridColumn::new("B").builder(|_, value| Text::new(format!("B:{value}"))),
                ],
                Rc::new(VecSource(data)),
            )
            .revision(revision)
        }
    });
    let revision = rev.get().expect("built");
    let a0 = tester.find(&Finder::text("r0c0"));
    let b0 = tester.find(&Finder::text("B:r0c1"));
    assert_eq!(tester.enter(|| vertical.max_scroll_extent()), 640.0 - 364.0);
    {
        let mut data = data.borrow_mut();
        data[0][0] = "changed".into();
        data[0][1] = "new".into();
        for r in 20..25 {
            data.push((0..2).map(|c| format!("r{r}c{c}")).collect());
        }
    }
    // Nothing moves until the revision changes.
    tester.pump();
    assert!(has(&tester, "r0c0") && !has(&tester, "changed"));
    tester.enter(|| revision.update(|n| *n += 1));
    tester.pump();
    // The default cell re-read its value in place; the built cell was rebuilt.
    assert_eq!(tester.find(&Finder::text("changed")), a0);
    assert!(!has(&tester, "r0c0"));
    assert_ne!(tester.find(&Finder::text("B:new")), b0);
    // The row count followed.
    assert_eq!(tester.enter(|| vertical.max_scroll_extent()), 800.0 - 364.0);
    // Without a revision signal the source is read once per cell build.
    let data = rows(20);
    let (mut tester, _, _) = grid(600.0, 400.0, {
        let data = data.clone();
        move || DataGrid::new(columns(2), Rc::new(VecSource(data)))
    });
    data.borrow_mut()[0][0] = "changed".into();
    tester.pump();
    assert!(has(&tester, "r0c0") && !has(&tester, "changed"));
}

#[test]
fn widgets_grid_07_semantics() {
    let (mut tester, _, _) = grid(600.0, 400.0, || {
        DataGrid::new([GridColumn::new("Id"), GridColumn::new("Name")], source(10))
    });
    let grid_id = tester.find(&Finder::key("g"));
    let update = tester.a11y_tree();
    let role_of = |id: NodeId| {
        update
            .nodes
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, n)| n.role())
    };
    assert_eq!(role_of(NodeId(grid_id.to_raw())), Some(Role::Grid));
    let has_role = |role: Role, label: Option<&str>| {
        update
            .nodes
            .iter()
            .any(|(_, n)| n.role() == role && label.is_none_or(|l| n.label() == Some(l)))
    };
    assert!(has_role(Role::ColumnHeader, Some("Name")));
    assert!(has_role(Role::ColumnHeader, Some("Id")));
    assert!(has_role(Role::Cell, None));
    assert!(has_role(Role::Label, Some("r0c0")));
}

const SURFACE: Color = Color::from_argb32(0xFFFF_FFFF);
const OUTLINE: Color = Color::from_argb32(0xFF79_747E);

#[test]
fn widgets_grid_09_frozen_columns() {
    let (mut tester, _, horizontal) = grid(600.0, 400.0, || {
        DataGrid::new(columns(20), source(1000)).frozen_columns(2)
    });
    let (c0, c1, h0) = (
        rect(&tester, "r0c0"),
        rect(&tester, "r0c1"),
        rect(&tester, "Col 0"),
    );
    assert_eq!(
        tester.enter(|| horizontal.max_scroll_extent()),
        1800.0,
        "the extents are unchanged"
    );
    tester.scroll(&Finder::key("g"), Vec2::new(240.0, 0.0));
    assert_eq!(offset(&tester, horizontal), 240.0);
    // Frozen cells and headers stay; the scrolling ones moved under them.
    assert_eq!(rect(&tester, "r0c0"), c0);
    assert_eq!(rect(&tester, "r0c1"), c1);
    assert_eq!(rect(&tester, "Col 0"), h0);
    assert_eq!(rect(&tester, "r0c4").left, 240.0 + 8.0);
    assert_eq!(rect(&tester, "Col 4").left, 240.0 + 8.0);
    assert!(has(&tester, "r0c3") && !has(&tester, "r0c2") && !has(&tester, "r0c8"));
    assert!(has(&tester, "Col 3") && !has(&tester, "Col 2"));
    // Frozen cells are opaque (a surface fill the size of a cell) and painted after the
    // scrolling cells; the last frozen column's right line is the outline color.
    let entries = tester.frame().scene.entries();
    let cell_fills = |color: Color| {
        entries
            .iter()
            .filter(|e| {
                matches!(e.command, Command::Fill { shape, color: c } if c == color
                    && shape.rect.width() == 120.0)
            })
            .count()
    };
    // Rows painted (the buffer row below the window is culled), plus the header.
    let painted_rows = tester
        .find_all(&Finder::render::<RenderParagraph>())
        .into_iter()
        .filter(|e| tester.text(*e).is_some_and(|t| t.ends_with("c0")))
        .filter(|e| tester.rect(*e).top < 400.0)
        .count();
    assert_eq!(
        cell_fills(SURFACE),
        2 * painted_rows + 2,
        "{painted_rows} painted frozen rows"
    );
    let edge_lines = entries
        .iter()
        .filter(|e| matches!(e.command, Command::Fill { shape, color } if color == OUTLINE && shape.rect.width() == 1.0))
        .count();
    assert_eq!(edge_lines, painted_rows + 1);
    let position = |text: &str| {
        let id = tester.find(&Finder::text(text));
        entries
            .iter()
            .position(|e| e.element == Some(id))
            .expect("painted")
    };
    assert!(
        position("r0c0") > position("r0c4"),
        "frozen cells paint last"
    );
    // More frozen columns than columns: all frozen, nothing scrolls.
    let (mut tester, _, horizontal) = grid(600.0, 400.0, || {
        DataGrid::new(columns(3), source(10)).frozen_columns(5)
    });
    tester.scroll(&Finder::key("g"), Vec2::new(100.0, 0.0));
    assert_eq!(offset(&tester, horizontal), 0.0);
    assert_eq!(rect(&tester, "r0c2").left, 240.0 + 8.0);
}

const ON_SURFACE_VARIANT: Color = Color::from_argb32(0xFF49_454F);

/// The chevron's bars: 2 px fills in on-surface-variant.
fn chevron_bars(tester: &WidgetTester) -> usize {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter(|e| matches!(e.command, Command::Fill { shape, color } if color == ON_SURFACE_VARIANT && shape.rect.height() == 2.0))
        .count()
}

/// The display order of column `c`'s texts, top to bottom.
fn column_texts(tester: &WidgetTester, prefix: &str) -> Vec<String> {
    let mut cells: Vec<(f32, String)> = tester
        .find_all(&Finder::render::<RenderParagraph>())
        .into_iter()
        .filter_map(|e| tester.text(e).map(|t| (tester.rect(e).top, t)))
        .filter(|(_, t)| t.starts_with(prefix))
        .collect();
    cells.sort_by(|a, b| a.0.total_cmp(&b.0));
    cells.into_iter().map(|(_, t)| t).collect()
}

#[test]
fn widgets_grid_10_sort_state_and_header_clicks() {
    let state: Rc<Cell<Option<Signal<Option<SortState>>>>> = Rc::default();
    let (mut tester, _, _) = grid(600.0, 400.0, {
        let state = state.clone();
        move || {
            let sort = signal(None::<SortState>);
            state.set(Some(sort));
            DataGrid::new(
                [
                    GridColumn::new("Col 0"),
                    GridColumn::new("Col 1").sortable(true),
                    GridColumn::new("Col 2").sortable(true),
                ],
                source(5),
            )
            .sort_state(sort)
        }
    });
    let sort = state.get().expect("built");
    let read = |t: &WidgetTester| t.enter(|| sort.get());
    assert_eq!(read(&tester), None);
    assert_eq!(chevron_bars(&tester), 0);
    // Not sortable: nothing.
    tester.tap(&Finder::text("Col 0"));
    assert_eq!(read(&tester), None);
    // Ascending, descending, none.
    tester.tap(&Finder::text("Col 1"));
    assert_eq!(
        read(&tester),
        Some(SortState {
            column: 1,
            ascending: true
        })
    );
    assert_eq!(chevron_bars(&tester), 2);
    tester.tap(&Finder::text("Col 1"));
    assert_eq!(
        read(&tester),
        Some(SortState {
            column: 1,
            ascending: false
        })
    );
    assert_eq!(chevron_bars(&tester), 2);
    tester.tap(&Finder::text("Col 1"));
    assert_eq!(read(&tester), None);
    assert_eq!(chevron_bars(&tester), 0);
    // Another sortable column starts ascending; the app can set the state too.
    tester.tap(&Finder::text("Col 1"));
    tester.tap(&Finder::text("Col 2"));
    assert_eq!(
        read(&tester),
        Some(SortState {
            column: 2,
            ascending: true
        })
    );
    tester.enter(|| {
        sort.set(Some(SortState {
            column: 0,
            ascending: false,
        }))
    });
    tester.pump();
    assert_eq!(chevron_bars(&tester), 2);
    // The state alone reorders nothing.
    assert_eq!(column_texts(&tester, "r")[0], "r0c0");
    // Without a shared signal the grid owns one; with client_sort every column is sortable.
    let (mut tester, _, _) = grid(600.0, 400.0, || {
        DataGrid::new(columns(3), source(5)).client_sort(true)
    });
    tester.tap(&Finder::text("Col 0"));
    assert_eq!(chevron_bars(&tester), 2);
}

#[test]
fn widgets_grid_11_client_sort() {
    use std::cmp::Ordering::Equal;
    // The value order.
    let order = [
        CellValue::Empty,
        CellValue::Bool(false),
        CellValue::Bool(true),
        CellValue::Float(f64::NAN),
        CellValue::Int(2),
        CellValue::Float(2.5),
        CellValue::Int(3),
        CellValue::Text("B".into()),
        CellValue::Text("a".into()),
    ];
    for (i, a) in order.iter().enumerate() {
        for (j, b) in order.iter().enumerate() {
            assert_eq!(a.compare(b), i.cmp(&j), "{a:?} vs {b:?}");
        }
    }
    assert_eq!(CellValue::Int(2).compare(&CellValue::Float(2.0)), Equal);
    assert_eq!(
        CellValue::Float(f64::NAN).compare(&CellValue::Float(f64::NAN)),
        Equal
    );
    // A grid over rows whose column 0 is "b", "a", "c", "a" and column 1 "x0".."x3".
    let data = Rc::new(RefCell::new(
        ["b", "a", "c", "a"]
            .iter()
            .enumerate()
            .map(|(i, t)| vec![(*t).to_owned(), format!("x{i}")])
            .collect::<Vec<_>>(),
    ));
    let state: Rc<Cell<Option<Signal<Option<SortState>>>>> = Rc::default();
    let rev: Rc<Cell<Option<Signal<u64>>>> = Rc::default();
    let (mut tester, _, _) = grid(600.0, 400.0, {
        let (data, state, rev) = (data.clone(), state.clone(), rev.clone());
        move || {
            let (sort, revision) = (signal(None::<SortState>), signal(0u64));
            state.set(Some(sort));
            rev.set(Some(revision));
            DataGrid::new(
                [
                    GridColumn::new("K"),
                    GridColumn::new("V").builder(|row, value| Text::new(format!("B{row}:{value}"))),
                ],
                Rc::new(VecSource(data)),
            )
            .client_sort(true)
            .sort_state(sort)
            .revision(revision)
        }
    });
    let (sort, revision) = (state.get().expect("built"), rev.get().expect("built"));
    let keys = |t: &WidgetTester| column_texts(t, "");
    // Ascending: a (source 1), a (source 3), b, c; stable, and builders see source rows.
    tester.tap(&Finder::text("K"));
    let texts = keys(&tester);
    let col0: Vec<&str> = texts
        .iter()
        .filter(|t| t.len() == 1)
        .map(String::as_str)
        .collect();
    assert_eq!(col0, ["a", "a", "b", "c"]);
    let col1: Vec<&str> = texts
        .iter()
        .filter(|t| t.starts_with('B'))
        .map(String::as_str)
        .collect();
    assert_eq!(col1, ["B1:x1", "B3:x3", "B0:x0", "B2:x2"]);
    // Descending reverses the comparison, not the stable order.
    tester.tap(&Finder::text("K"));
    let texts = keys(&tester);
    let col0: Vec<&str> = texts
        .iter()
        .filter(|t| t.len() == 1)
        .map(String::as_str)
        .collect();
    assert_eq!(col0, ["c", "b", "a", "a"]);
    let col1: Vec<&str> = texts
        .iter()
        .filter(|t| t.starts_with('B'))
        .map(String::as_str)
        .collect();
    assert_eq!(col1, ["B2:x2", "B0:x0", "B1:x1", "B3:x3"]);
    // None: the source order.
    tester.tap(&Finder::text("K"));
    let texts = keys(&tester);
    let col0: Vec<&str> = texts
        .iter()
        .filter(|t| t.len() == 1)
        .map(String::as_str)
        .collect();
    assert_eq!(col0, ["b", "a", "c", "a"]);
    // A revision change re-sorts.
    tester.enter(|| {
        sort.set(Some(SortState {
            column: 0,
            ascending: true,
        }))
    });
    tester.pump();
    data.borrow_mut()[2][0] = "0".into();
    tester.enter(|| revision.update(|n| *n += 1));
    tester.pump();
    let texts = keys(&tester);
    let col0: Vec<&str> = texts
        .iter()
        .filter(|t| t.len() == 1)
        .map(String::as_str)
        .collect();
    assert_eq!(col0, ["0", "a", "a", "b"]);
}

#[test]
fn widgets_grid_12_filter() {
    let calls = Rc::new(Cell::new(0usize));
    let filt: Rc<Cell<Option<Signal<Option<RowFilter>>>>> = Rc::default();
    let state: Rc<Cell<Option<Signal<Option<SortState>>>>> = Rc::default();
    let (mut tester, vertical, _) = grid(600.0, 400.0, {
        let (filt, state) = (filt.clone(), state.clone());
        move || {
            let (filter, sort) = (signal(None::<RowFilter>), signal(None::<SortState>));
            filt.set(Some(filter));
            state.set(Some(sort));
            DataGrid::new(
                [GridColumn::new("Id"), GridColumn::new("Name")],
                Rc::new(FnSource::new(20, |r, c| match c {
                    0 => CellValue::Int(r as i64),
                    _ => CellValue::Text(format!("n{r}")),
                })),
            )
            .filter(filter)
            .client_sort(true)
            .sort_state(sort)
        }
    });
    let (filter, sort) = (filt.get().expect("built"), state.get().expect("built"));
    assert_eq!(tester.enter(|| vertical.max_scroll_extent()), 640.0 - 364.0);
    // Even ids only: 10 rows, once per source row.
    let even: RowFilter = {
        let calls = calls.clone();
        Rc::new(move |source, row| {
            calls.set(calls.get() + 1);
            matches!(source.cell(row, 0), CellValue::Int(n) if n % 2 == 0)
        })
    };
    tester.enter(|| filter.set(Some(even)));
    tester.pump();
    assert_eq!(calls.get(), 20);
    assert_eq!(tester.enter(|| vertical.max_scroll_extent()), 0.0);
    assert_eq!(
        column_texts(&tester, "n"),
        (0..10).map(|i| format!("n{}", 2 * i)).collect::<Vec<_>>()
    );
    // Sorted after filtering.
    tester.enter(|| {
        sort.set(Some(SortState {
            column: 0,
            ascending: false,
        }))
    });
    tester.pump();
    assert_eq!(column_texts(&tester, "n")[0], "n18");
    assert_eq!(
        calls.get(),
        20,
        "the filter wasn't re-run for a sort change"
    );
    // Back to every row.
    tester.enter(|| {
        filter.set(None);
        sort.set(None);
    });
    tester.pump();
    assert_eq!(tester.enter(|| vertical.max_scroll_extent()), 640.0 - 364.0);
    assert!(has(&tester, "n1"));
}
