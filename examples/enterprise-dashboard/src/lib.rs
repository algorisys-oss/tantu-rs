//! An order dashboard, the Phase 4 milestone: a dockable layout with an order grid, a
//! details form, KPI tiles and a revenue chart, an activity log, a menu bar, a toolbar, a
//! status bar and English or German. Run `cargo run -p enterprise-dashboard`.
//! Spec: `docs/specs/examples/enterprise-dashboard.md`.
//!
//! ```no_run
//! use tantu::prelude::*;
//! App::new()
//!     .window(Window::new("Orders").size(1280.0, 800.0), enterprise_dashboard::dashboard)
//!     .run()?;
//! # Ok::<(), tantu::Error>(())
//! ```

#![forbid(unsafe_code)]

mod messages;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tantu::core::{Rect, Size};
use tantu::layout::{BoxConstraints, RenderConstrainedBox};
use tantu::prelude::*;
use tantu::reactive::untrack;
use tantu::scene::{BorderRadius, RoundedRect};
use tantu::view::{BuildCx, Dyn, ElementId, Keyed, Paint, PaintCx, Prop};
use tantu::widgets::{
    CellValue, DataSource, DockLayout, DockNode, GridColumn, GridSelectionController,
    GridSelectionMode, PanelId,
};

/// A sales region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    /// North.
    North,
    /// South.
    South,
    /// East.
    East,
    /// West.
    West,
}

/// Where an order is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Not shipped yet.
    Open,
    /// On its way.
    Shipped,
    /// Settled.
    Paid,
}

/// One order (generated).
#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    /// The order number, from 1.
    pub id: u32,
    /// Who ordered.
    pub customer: String,
    /// Where.
    pub region: Region,
    /// How much, in currency units with cents.
    pub amount: f64,
    /// When.
    pub date: Date,
    /// Where it is.
    pub status: Status,
}

/// How many orders the dashboard shows.
pub const ORDERS: usize = 10_000;

const CUSTOMERS: [&str; 12] = [
    "Northwind Traders",
    "Contoso Ltd",
    "Fabrikam Inc",
    "Adventure Works",
    "Tailspin Toys",
    "Wide World Importers",
    "Litware Inc",
    "Proseware Inc",
    "Alpine Ski House",
    "Coho Winery",
    "Fourth Coffee",
    "Margie's Travel",
];

const REGIONS: [Region; 4] = [Region::North, Region::South, Region::East, Region::West];
const STATUSES: [Status; 3] = [Status::Open, Status::Shipped, Status::Paid];

impl Region {
    /// The message id part (`north`, …).
    fn key(self) -> &'static str {
        match self {
            Region::North => "north",
            Region::South => "south",
            Region::East => "east",
            Region::West => "west",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

impl Status {
    fn key(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Shipped => "shipped",
            Status::Paid => "paid",
        }
    }
}

/// The `count` orders the app shows, the same on every run (DASH-02).
pub fn orders(count: usize) -> Vec<Order> {
    // A 64-bit linear congruential generator (Knuth's MMIX constants).
    let mut state: u64 = 0x2026_1016;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        state >> 33
    };
    let start = Date::new(2026, 1, 1).expect("2026-01-01 exists");
    (0..count)
        .map(|i| {
            let cents = 1_000 + next() % 499_001;
            Order {
                id: i as u32 + 1,
                customer: CUSTOMERS[(next() % CUSTOMERS.len() as u64) as usize].to_owned(),
                region: REGIONS[i % REGIONS.len()],
                amount: cents as f64 / 100.0,
                date: start.add_days((next() % 365) as i64).unwrap_or(start),
                status: STATUSES[i % STATUSES.len()],
            }
        })
        .collect()
}

/// The orders and the user's edits (the grid reads them; DASH-03).
#[derive(Clone)]
struct Book {
    base: Rc<Vec<Order>>,
    edits: Rc<RefCell<HashMap<usize, Order>>>,
    /// Bumped on every edit, so the grid, the totals and the form follow.
    revision: Signal<u64>,
}

impl Book {
    fn order(&self, row: usize) -> Order {
        self.edits
            .borrow()
            .get(&row)
            .cloned()
            .unwrap_or_else(|| self.base[row].clone())
    }

    fn save(&self, row: usize, order: Order) {
        self.edits.borrow_mut().insert(row, order);
        self.revision.update(|r| *r += 1);
    }

    fn reset(&self) {
        self.edits.borrow_mut().clear();
        self.revision.update(|r| *r += 1);
    }

    /// (orders, total, total per region), tracked through the revision.
    fn totals(&self) -> (usize, f64, [f64; 4]) {
        self.revision.get();
        let mut by_region = [0.0; 4];
        let edits = self.edits.borrow();
        for (row, base) in self.base.iter().enumerate() {
            let order = edits.get(&row).unwrap_or(base);
            by_region[order.region.index()] += order.amount;
        }
        (self.base.len(), by_region.iter().sum(), by_region)
    }
}

impl DataSource for Book {
    fn row_count(&self) -> usize {
        self.base.len()
    }

    fn cell(&self, row: usize, column: usize) -> CellValue {
        let order = self.order(row);
        match column {
            0 => CellValue::Int(i64::from(order.id)),
            1 => CellValue::Text(order.customer),
            2 => CellValue::Int(order.region.index() as i64),
            3 => CellValue::Float(order.amount),
            4 => CellValue::Text(order.date.to_string()),
            _ => CellValue::Int(order.status as i64),
        }
    }
}

/// What the panels share.
#[derive(Clone)]
struct Shared {
    book: Book,
    loc: Localization,
    selection: GridSelectionController,
    /// The selected order's row in the source.
    selected: Signal<Option<usize>>,
    activity: Signal<Vec<String>>,
    layout: Signal<DockLayout>,
}

/// The starting layout (DASH-01).
fn initial_layout() -> DockLayout {
    DockLayout::new(DockNode::row([
        (0.55, DockNode::tabs(["orders", "activity"])),
        (
            0.45,
            DockNode::column([
                (0.55, DockNode::tabs(["details"])),
                (0.45, DockNode::tabs(["revenue"])),
            ]),
        ),
    ]))
}

/// The dashboard's view (DASH-01).
pub fn dashboard() -> impl View {
    Builder::new(|_: &mut BuildCx<'_>| {
        let loc = messages::localization();
        let book = Book {
            base: Rc::new(orders(ORDERS)),
            edits: Rc::default(),
            revision: signal(0),
        };
        let selection = GridSelectionController::new();
        // The order the grid last selected. The grid drops its selection when the model
        // changes (a save bumps the revision), but the form keeps showing the order.
        let selected = signal(None::<usize>);
        {
            let selection = selection.clone();
            effect(move || {
                let row = selection
                    .selection()
                    .and_then(|s| selection.source_cell(s.active))
                    .map(|c| c.row);
                if row.is_some() && row != selected.get_untracked() {
                    selected.set(row);
                }
            });
        }
        let shared = Shared {
            book,
            loc,
            selection,
            selected,
            activity: signal(Vec::new()),
            layout: signal(initial_layout()),
        };
        let window = Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .child(menu_bar(shared.clone()))
            .child(toolbar(shared.clone()))
            .child(Expanded::new(
                SizedBox::expand().child(dock(shared.clone())),
            ))
            .child(status_bar(shared));
        Localizations::new(loc, window)
    })
}

/// File, View and Help, rebuilt in the current language (DASH-01, -05).
fn menu_bar(shared: Shared) -> impl View {
    let (loc, layout, activity) = (shared.loc, shared.layout, shared.activity);
    Dyn::new(move || {
        loc.locale();
        let text = |id: &str| untrack(|| loc.text(id));
        let set = move |id: &'static str| {
            move || {
                if let Some(locale) = Locale::parse(id) {
                    loc.set_locale(locale);
                }
            }
        };
        Keyed::new(
            "menubar",
            MenuBar::new()
                .menu(
                    text("menu-file"),
                    [MenuItem::new(text("reset-layout"))
                        .on_select(move || layout.set(initial_layout()))],
                )
                .menu(
                    text("menu-view"),
                    [
                        MenuItem::new("English").on_select(set("en")),
                        MenuItem::new("Deutsch").on_select(set("de")),
                    ],
                )
                .menu(
                    text("menu-help"),
                    [MenuItem::new(text("about")).on_select(move || {
                        let line = untrack(|| loc.text("about-text"));
                        activity.update(|a| a.push(line));
                    })],
                ),
        )
    })
}

/// Reset edits and the selected order (DASH-01, -05).
fn toolbar(shared: Shared) -> impl View {
    let (loc, selected, book) = (shared.loc, shared.selected, shared.book);
    Toolbar::new()
        .child(Button::new(move || loc.text("reset-edits")).on_press(move || book.reset()))
        .child(Keyed::new(
            "toolbar-order",
            Text::new(move || match selected.get() {
                Some(row) => loc.text_with("order-number", &Args::new().set("id", row + 1)),
                None => String::new(),
            }),
        ))
}

/// The order count and the selection (DASH-06).
fn status_bar(shared: Shared) -> impl View {
    let (loc, selected, book) = (shared.loc, shared.selected, shared.book);
    StatusBar::new()
        .leading(Keyed::new(
            "status-orders",
            Text::new(move || {
                let count = book.base.len();
                loc.text_with("status-orders", &Args::new().set("count", count))
            })
            .style(TextStyle::label()),
        ))
        .trailing(Keyed::new(
            "status-selection",
            Text::new(move || match selected.get() {
                Some(row) => loc.text_with("status-selected", &Args::new().set("id", row + 1)),
                None => String::new(),
            })
            .style(TextStyle::label()),
        ))
}

/// The four panels in a dock (DASH-01).
fn dock(shared: Shared) -> impl View {
    let (loc, layout) = (shared.loc, shared.layout);
    DockArea::new(layout, move |id: &PanelId| -> AnyView {
        match id.as_str() {
            "orders" => AnyView::new(Keyed::new(
                "orders",
                SizedBox::expand().child(orders_panel(shared.clone())),
            )),
            "details" => AnyView::new(Keyed::new("details", details_panel(shared.clone()))),
            "revenue" => AnyView::new(Keyed::new("revenue", revenue_panel(shared.clone()))),
            _ => AnyView::new(Keyed::new("activity", activity_panel(shared.clone()))),
        }
    })
    .title(move |id| loc.text(&format!("panel-{}", id.as_str())))
}

/// The order grid (DASH-02).
fn orders_panel(shared: Shared) -> impl View {
    let loc = shared.loc;
    let column = |id: &str, width: f32| {
        GridColumn::new(untrack(|| loc.text(id)))
            .width(width)
            .sortable(true)
    };
    let columns = [
        column("column-no", 72.0),
        column("column-customer", 190.0),
        column("column-region", 100.0).builder(move |_, value| {
            let region = match value {
                CellValue::Int(i) => REGIONS.get(i as usize).copied(),
                _ => None,
            };
            Text::new(move || {
                region.map_or_else(String::new, |r| loc.text(&format!("region-{}", r.key())))
            })
        }),
        column("column-amount", 110.0).builder(move |_, value| {
            let amount = match value {
                CellValue::Float(v) => v,
                _ => 0.0,
            };
            Text::new(move || loc.format_decimal(amount, 2))
        }),
        column("column-date", 120.0).builder(move |_, value| {
            let date = match value {
                CellValue::Text(t) => Date::parse_iso(&t),
                _ => None,
            };
            Text::new(move || {
                date.map_or_else(String::new, |d| loc.format_date(d, DateLength::Medium))
            })
        }),
        column("column-status", 100.0).builder(move |_, value| {
            let status = match value {
                CellValue::Int(i) => STATUSES.get(i as usize).copied(),
                _ => None,
            };
            Text::new(move || {
                status.map_or_else(String::new, |s| loc.text(&format!("status-{}", s.key())))
            })
        }),
    ];
    Keyed::new(
        "orders-grid",
        DataGrid::new(columns, Rc::new(shared.book.clone()))
            .revision(shared.book.revision)
            .client_sort(true)
            .selection_mode(GridSelectionMode::Row)
            .selection_controller(shared.selection.clone()),
    )
}

/// The selected order's form (DASH-03).
fn details_panel(shared: Shared) -> impl View {
    let loc = shared.loc;
    let customer = TextEditingController::new("");
    let amount = TextEditingController::new("");
    let date = signal(None::<Date>);
    let region = signal(None::<usize>);
    let form = FormController::new();
    let selected = shared.selected;
    // The form shows the selected order (and the saved edits).
    {
        let book = shared.book.clone();
        effect(move || {
            let Some(row) = selected.get() else {
                return;
            };
            book.revision.get();
            let order = untrack(|| book.order(row));
            untrack(|| {
                customer.set_text(order.customer.clone());
                amount.set_text(format!("{:.2}", order.amount));
                date.set(Some(order.date));
                region.set(Some(order.region.index()));
            });
        });
    }
    let has_selection = memo(move || selected.get().is_some());
    let (book, activity) = (shared.book, shared.activity);
    Padding::all(16.0).child(Dyn::new(move || {
        loc.locale();
        let text = |id: &str| untrack(|| loc.text(id));
        if !has_selection.get() {
            return AnyView::new(Text::new(text("select-order")));
        }
        let book = book.clone();
        let save = move || {
            if !form.validate() {
                return;
            }
            let Some(row) = selected.get_untracked() else {
                return;
            };
            let mut order = book.order(row);
            order.customer = customer.text().trim().to_owned();
            order.amount = amount.text().parse().unwrap_or(order.amount);
            if let Some(d) = date.get_untracked() {
                order.date = d;
            }
            if let Some(r) = region.get_untracked().and_then(|r| REGIONS.get(r)) {
                order.region = *r;
            }
            let id = order.id;
            book.save(row, order);
            let line = loc.text_with("activity-updated", &Args::new().set("id", id));
            activity.update(|a| a.push(line));
        };
        AnyView::new(Form::new(
            form,
            SizedBox::width(360.0).child(
                Column::new()
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .spacing(8.0)
                    .child(
                        TextFormField::new(customer)
                            .label(text("field-customer"))
                            .validator(validators::required(text("error-customer"))),
                    )
                    .child(
                        TextFormField::new(amount)
                            .label(text("field-amount"))
                            .input_formatter(FilteringFormatter::allow(|c| {
                                c.is_ascii_digit() || c == '.'
                            }))
                            .validator(validators::custom(
                                |t| t.parse::<f64>().is_ok_and(|v| v > 0.0),
                                text("error-amount"),
                            )),
                    )
                    .child(
                        Row::new()
                            .spacing(12.0)
                            .child(DatePickerField::new(date).label(text("field-date")))
                            .child(
                                ComboBox::new(
                                    region,
                                    REGIONS.map(|r| text(&format!("region-{}", r.key()))),
                                )
                                .width(140.0)
                                .label(text("field-region")),
                            ),
                    )
                    .child(
                        Align::new(Alignment::CENTER_LEFT)
                            .child(Button::new(text("save")).on_press(save)),
                    ),
            ),
        ))
    }))
}

/// The tallest bar (DASH-04).
const CHART_HEIGHT: f32 = 120.0;

/// KPI tiles and the revenue chart (DASH-04).
fn revenue_panel(shared: Shared) -> impl View {
    let (loc, book) = (shared.loc, shared.book);
    let totals = memo(move || book.totals());
    let tile = move |key: &'static str, value: Rc<dyn Fn() -> String>| {
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(4.0)
            .child(Text::new(move || loc.text(key)).style(TextStyle::label()))
            .child(Keyed::new(
                key,
                Text::new(move || value()).style(TextStyle::title()),
            ))
    };
    let tiles = LayoutGrid::new([TrackSize::fr(1.0), TrackSize::fr(1.0), TrackSize::fr(1.0)])
        .column_gap(12.0)
        .child(tile(
            "kpi-orders",
            Rc::new(move || loc.format_integer(totals.get().0 as i64)),
        ))
        .child(tile(
            "kpi-revenue",
            Rc::new(move || loc.format_decimal(totals.get().1, 2)),
        ))
        .child(tile(
            "kpi-average",
            Rc::new(move || {
                let (count, total, _) = totals.get();
                loc.format_decimal(total / count.max(1) as f64, 2)
            }),
        ));
    let mut chart = Row::new()
        .cross_axis_alignment(CrossAxisAlignment::End)
        .spacing(24.0);
    for region in REGIONS {
        let value = move || totals.get().2[region.index()];
        let height = move || {
            let max = totals.get().2.iter().copied().fold(0.0, f64::max);
            if max > 0.0 {
                (value() / max) as f32 * CHART_HEIGHT
            } else {
                0.0
            }
        };
        chart = chart.child(
            Column::new()
                .spacing(4.0)
                .child(Keyed::new(
                    format!("bar-{}", region.key()),
                    Bar {
                        height: height.into_prop(),
                    },
                ))
                .child(Text::new(move || {
                    loc.text(&format!("region-{}", region.key()))
                }))
                .child(Text::new(move || loc.format_decimal(value(), 0)).style(TextStyle::label())),
        );
    }
    Padding::all(16.0).child(
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .spacing(16.0)
            .child(tiles)
            .child(chart),
    )
}

/// One bar of the chart: a 48 px wide box of a reactive height, filled with the theme's
/// primary color.
struct Bar {
    height: Prop<f32>,
}

impl View for Bar {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::new(48.0, 0.0))),
            [],
        );
        let color = Theme::of(cx).colors.primary;
        cx.set_paint(id, BarPaint(color));
        cx.bind(id, self.height, |e, height| {
            e.update_render::<RenderConstrainedBox>(|r| {
                r.additional = BoxConstraints::tight(Size::new(48.0, height.max(0.0)));
            });
        });
        id
    }
}

/// Fills a bar, its top corners rounded.
struct BarPaint(Color);

impl Paint for BarPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene().fill(
            RoundedRect::new(
                Rect::from_ltwh(0.0, 0.0, size.width, size.height),
                BorderRadius::circular(4.0),
            ),
            self.0,
        );
    }
}

/// The activity log, newest first (DASH-03).
fn activity_panel(shared: Shared) -> impl View {
    let activity = shared.activity;
    Padding::all(16.0).child(ScrollView::new(Dyn::new(move || {
        let mut column = Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(6.0);
        for line in activity.get().into_iter().rev() {
            column = column.child(Text::new(line));
        }
        column
    })))
}
