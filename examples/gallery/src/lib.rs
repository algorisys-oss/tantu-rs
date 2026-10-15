//! The Tantu widget gallery (spec `docs/specs/examples/gallery.md`): a header, a sidebar of
//! sections and the selected section's samples.

mod dock;
mod menus;
mod split;
mod tabs;
mod tree;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use tantu::core::Rect;
use tantu::layout::{BoxConstraints, RenderConstrainedBox};
use tantu::prelude::*;
use tantu::reactive::untrack;
use tantu::scene::{BorderRadius, ImageData, RoundedRect};
use tantu::view::{Anchor, BuildCx, ElementId, Keyed, OverlayPortal, Paint, PaintCx, Prop};

/// The section names, in sidebar order.
pub const SECTIONS: [&str; 13] = [
    "Buttons",
    "Text",
    "Selection",
    "Text field",
    "Lists",
    "Media",
    "Animation",
    "Data grid",
    "Tree view",
    "Docking",
    "Split view",
    "Tabs",
    "Kitchen sink",
];

/// The kitchen-sink cards, in page order: (card key, the widgets it shows, as sample keys
/// `sink-<widget>`) (GALLERY-12).
pub const SINK_CARDS: [(&str, &[&str]); 18] = [
    (
        "buttons",
        &["button", "icon_button", "tooltip", "overlay_portal"],
    ),
    ("toggles", &["checkbox", "radio", "switch"]),
    ("sliders", &["slider"]),
    (
        "typography",
        &["text", "rich_text", "divider", "vertical_divider"],
    ),
    ("fields", &["text_field"]),
    ("media", &["icon", "image"]),
    ("lists", &["list_view"]),
    ("tree", &["tree_view"]),
    ("tabs", &["tab_bar", "tab_view"]),
    ("split", &["split_view"]),
    ("dock", &["dock_area"]),
    ("menus", &["menu_anchor", "context_menu"]),
    ("bars", &["tool_bar", "status_bar"]),
    ("grid", &["data_grid"]),
    (
        "layout",
        &[
            "row",
            "column",
            "expanded",
            "flexible",
            "spacer",
            "stack",
            "positioned",
            "center",
            "align",
            "padding",
            "sized_box",
            "constrained_box",
            "limited_box",
        ],
    ),
    ("scrolling", &["scroll_view"]),
    ("animation", &["opacity", "animated_switcher"]),
    ("focus", &["focus_scope"]),
];

const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);
const RED: Color = Color::from_argb32(0xFFB3_261E);

/// The key of a section's sidebar button.
fn nav_key(section: &str) -> String {
    format!("nav-{}", section.to_lowercase().replace(' ', "-"))
}

/// The gallery's view: a menu bar, the header, the sidebar and the section, and a status
/// bar (GALLERY-01, -31).
pub fn gallery() -> impl View {
    Builder::new(|cx| {
        let theme = Theme::signal(cx).unwrap_or_else(|| signal(Theme::light()));
        let section = signal(SECTIONS[0]);
        let dark = signal(false);
        // A message shown in the status bar until the next navigation (Help → About).
        let note: Signal<Option<String>> = signal(None);
        effect(move || {
            theme.set(if dark.get() {
                Theme::dark()
            } else {
                Theme::light()
            });
        });
        effect(move || {
            section.get();
            untrack(|| {
                if note.get().is_some() {
                    note.set(None);
                }
            });
        });
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .child(Keyed::new("menubar", menu_bar(section, dark, note)))
            .child(Keyed::new("header", header(section, dark)))
            .child(Expanded::new(
                Row::new()
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .child(Keyed::new("sidebar", sidebar(section)))
                    .child(Expanded::new(Keyed::new("content", content(section)))),
            ))
            .child(Keyed::new("statusbar", status_bar(section, dark, note)))
    })
}

/// View (dark theme), Go (every section) and Help (GALLERY-31).
fn menu_bar(
    section: Signal<&'static str>,
    dark: Signal<bool>,
    note: Signal<Option<String>>,
) -> impl View {
    MenuBar::new()
        .menu(
            "View",
            [MenuItem::new("Dark theme")
                .checked(dark)
                .on_select(move || dark.update(|d| *d = !*d))],
        )
        .menu(
            "Go",
            SECTIONS
                .iter()
                .map(move |name| MenuItem::new(*name).on_select(move || section.set(name))),
        )
        .menu(
            "Help",
            [MenuItem::new("About Tantu").on_select(move || {
                note.set(Some("Tantu: compose once, render your way.".to_owned()))
            })],
        )
}

/// The section (or a Help message) and the theme (GALLERY-31).
fn status_bar(
    section: Signal<&'static str>,
    dark: Signal<bool>,
    note: Signal<Option<String>>,
) -> impl View {
    StatusBar::new()
        .leading(Keyed::new(
            "status-section",
            Text::new(move || {
                note.get()
                    .unwrap_or_else(|| format!("Section: {}", section.get()))
            })
            .style(TextStyle::label()),
        ))
        .trailing(Keyed::new(
            "status-theme",
            Text::new(move || if dark.get() { "Dark" } else { "Light" }.to_owned())
                .style(TextStyle::label()),
        ))
}

fn header(section: Signal<&'static str>, dark: Signal<bool>) -> impl View {
    Padding::new(EdgeInsets::symmetric(16.0, 12.0)).child(
        Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(16.0)
            .child(Text::new("Tantu gallery").style(TextStyle::title()))
            .child(Spacer::new())
            .child(Keyed::new(
                "current",
                Text::new(move || section.get().to_string()).style(TextStyle::label()),
            ))
            .child(dark_mode_switch(dark)),
    )
}

/// "Dark" and a switch that sets the window's theme to dark or light (GALLERY-05). The theme
/// signal comes from the runner (or the tester), so the whole window follows.
fn dark_mode_switch(dark: Signal<bool>) -> impl View {
    Row::new()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .spacing(4.0)
        .child(Text::new("Dark").style(TextStyle::label()))
        .child(Keyed::new(
            "dark",
            Switch::new(dark).on_change(move |v| dark.set(v)),
        ))
}

/// The section buttons, scrolling when the window is too short for all of them.
fn sidebar(section: Signal<&'static str>) -> impl View {
    SizedBox::width(200.0).child(ScrollView::new(
        Padding::all(8.0).child(
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Stretch)
                .spacing(4.0)
                .children(SECTIONS.iter().map(|name| {
                    Keyed::new(
                        nav_key(name),
                        Button::new(*name).on_press(move || section.set(name)),
                    )
                })),
        ),
    ))
}

fn content(section: Signal<&'static str>) -> impl View {
    Padding::all(16.0).child(Dyn::new(move || {
        let name = section.get();
        let samples: AnyView = match name {
            "Text" => AnyView::new(text_section()),
            "Selection" => AnyView::new(selection_section()),
            "Text field" => AnyView::new(text_field_section()),
            "Lists" => AnyView::new(lists_section()),
            "Media" => AnyView::new(media_section()),
            "Animation" => AnyView::new(animation_section()),
            "Data grid" => AnyView::new(grid_section()),
            "Kitchen sink" => AnyView::new(kitchen_sink_section()),
            "Tree view" => AnyView::new(tree::section()),
            "Tabs" => AnyView::new(tabs::section()),
            "Split view" => AnyView::new(split::section()),
            "Docking" => AnyView::new(dock::section()),
            _ => AnyView::new(buttons_section()),
        };
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(16.0)
            .child(Keyed::new(
                "heading",
                Text::new(name).style(TextStyle::title()),
            ))
            .child(samples)
    }))
}

fn buttons_section() -> impl View {
    let presses = signal(0u32);
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(12.0)
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(12.0)
                .child(Keyed::new(
                    "press",
                    Button::new("Press me").on_press(move || presses.update(|n| *n += 1)),
                ))
                .child(Keyed::new(
                    "press-count",
                    Text::new(move || format!("Pressed {} times", presses.get())),
                )),
        )
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(12.0)
                .child(Button::new("Disabled").enabled(false).on_press(|| {}))
                .child(Keyed::new(
                    "icon-button",
                    IconButton::new(Icons::FAVORITE).on_press(|| {}),
                ))
                .child(Tooltip::new(
                    "A tooltip",
                    Button::new("Hover me").on_press(|| {}),
                )),
        )
        .child(menus::samples())
}

fn text_section() -> impl View {
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(Text::new("Body text in the default style."))
        .child(Text::new("A title").style(TextStyle::title()))
        .child(Text::new("A label").style(TextStyle::label()))
        .child(Divider::new())
        .child(RichText::new(
            TextSpan::group()
                .child(TextSpan::new("Rich text with a "))
                .child(TextSpan::new("bold red").bold().color(RED))
                .child(TextSpan::new(" span and a "))
                .child(TextSpan::new("large").size(24.0))
                .child(TextSpan::new(" one.")),
        ))
        .child(Divider::new())
}

fn selection_section() -> impl View {
    let (checked, on, choice, value) = (signal(false), signal(false), signal('a'), signal(50.0f64));
    let on_off = |state: bool| if state { "on" } else { "off" };
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(12.0)
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(8.0)
                .child(Keyed::new(
                    "checkbox",
                    Checkbox::new(checked).on_change(move |v| checked.set(v)),
                ))
                .child(Keyed::new(
                    "checkbox-state",
                    Text::new(move || format!("Checkbox: {}", on_off(checked.get()))),
                )),
        )
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(8.0)
                .child(Keyed::new(
                    "switch",
                    Switch::new(on).on_change(move |v| on.set(v)),
                ))
                .child(Keyed::new(
                    "switch-state",
                    Text::new(move || format!("Switch: {}", on_off(on.get()))),
                )),
        )
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(8.0)
                .children(['a', 'b', 'c'].map(|letter| {
                    Keyed::new(
                        format!("radio-{letter}"),
                        Radio::new(letter, choice).on_change(move |v| choice.set(v)),
                    )
                }))
                .child(Keyed::new(
                    "radio-state",
                    Text::new(move || format!("Radio: {}", choice.get())),
                )),
        )
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(8.0)
                .child(
                    SizedBox::width(300.0).child(Keyed::new(
                        "slider",
                        Slider::new(value)
                            .range(0.0, 100.0)
                            .on_change(move |v| value.set(v)),
                    )),
                )
                .child(Keyed::new(
                    "slider-state",
                    Text::new(move || format!("Slider: {:.0}", value.get())),
                )),
        )
}

fn text_field_section() -> impl View {
    let (name, notes) = (
        TextEditingController::new(""),
        TextEditingController::new(""),
    );
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(12.0)
        .child(SizedBox::width(320.0).child(Keyed::new(
            "field",
            TextField::new(name).placeholder("Type here"),
        )))
        .child(Keyed::new(
            "echo",
            Text::new(move || format!("You typed: {}", name.text())),
        ))
        .child(
            SizedBox::width(320.0).child(Keyed::new(
                "notes",
                TextField::new(notes)
                    .placeholder("Notes (multi-line)")
                    .max_lines(None),
            )),
        )
}

fn lists_section() -> impl View {
    let selection = signal(None::<usize>);
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(Text::new("10 000 rows, built as they scroll into view.").style(TextStyle::label()))
        .child(SizedBox::new(400.0, 200.0).child(Keyed::new(
            "list",
            ListView::builder(10_000usize, 32.0, |i| {
                Padding::new(EdgeInsets::symmetric(8.0, 0.0))
                    .child(Align::new(Alignment::CENTER_LEFT).child(Text::new(format!("Row {i}"))))
            }),
        )))
        .child(
            Text::new("Headers and rows of different heights; click or use the arrow keys.")
                .style(TextStyle::label()),
        )
        .child(
            SizedBox::new(400.0, 200.0).child(Keyed::new(
                "contacts",
                ListView::builder_with_extents(
                    200usize,
                    |i| if i % 11 == 0 { 24.0 } else { 32.0 },
                    move |i| contact_row(i, selection),
                )
                .selection(selection),
            )),
        )
        .child(Keyed::new(
            "selected",
            Text::new(move || match selection.get() {
                Some(i) => format!("Selected: {i}"),
                None => "Selected: none".to_owned(),
            }),
        ))
}

/// A row of the contacts list (GALLERY-08): a group header every eleventh row, else a
/// contact, highlighted in primary while selected, with a 1 px line under it.
fn contact_row(i: usize, selection: Signal<Option<usize>>) -> impl View {
    let header = i % 11 == 0;
    let label = if header {
        format!("Group {}", i / 11)
    } else {
        format!("Contact {i}")
    };
    let color = move || {
        if selection.get() == Some(i) {
            PRIMARY
        } else {
            Color::from_argb32(0xFF1C_1B1F)
        }
    };
    let style = if header {
        TextStyle::label()
    } else {
        TextStyle::body()
    };
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .child(Expanded::new(
            Padding::new(EdgeInsets::symmetric(8.0, 0.0)).child(
                Align::new(Alignment::CENTER_LEFT)
                    .child(Text::new(label).style(style).color(color)),
            ),
        ))
        .child(Divider::new().height(1.0))
}

fn media_section() -> impl View {
    let icons = [
        Icons::HOME,
        Icons::SEARCH,
        Icons::SETTINGS,
        Icons::FAVORITE,
        Icons::DELETE,
        Icons::CHECK,
    ];
    let image: AnyView = match gradient(64, 64) {
        Ok(data) => AnyView::new(Keyed::new(
            "image",
            Image::new(data).size(128.0, 128.0).fit(BoxFit::Contain),
        )),
        Err(_) => AnyView::new(SizedBox::shrink()),
    };
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(12.0)
        .child(
            Row::new()
                .spacing(12.0)
                .children(icons.map(|icon| Icon::new(icon).size(32.0).color(PRIMARY))),
        )
        .child(image)
}

/// The Animation section (GALLERY-06): an implicitly animated box and a controller-driven
/// dot.
/// The Data grid section (GALLERY-10): a million generated rows in a `DataGrid`, and the
/// first visible row read from its vertical controller.
/// Generated data with memory proportional to committed edits, not the row count.
struct EditableGridSource {
    generated: FnSource,
    edits: RefCell<HashMap<(usize, usize), CellValue>>,
    error: Signal<String>,
    columns: usize,
    score_column: usize,
}
impl DataSource for EditableGridSource {
    fn row_count(&self) -> usize {
        self.generated.row_count()
    }
    fn cell(&self, row: usize, column: usize) -> CellValue {
        self.edits
            .borrow()
            .get(&(row, column))
            .cloned()
            .unwrap_or_else(|| self.generated.cell(row, column))
    }
    fn set_cell(
        &self,
        row: usize,
        column: usize,
        value: CellValue,
    ) -> std::result::Result<(), GridEditError> {
        if row >= self.row_count() || column == 0 || column >= self.columns {
            return Err(GridEditError::ReadOnly);
        }
        if column == self.score_column && !matches!(value, CellValue::Int(0..=100)) {
            return Err(GridEditError::Rejected(
                "Score must be between 0 and 100".to_owned(),
            ));
        }
        self.edits.borrow_mut().insert((row, column), value);
        self.error.set("none".to_owned());
        Ok(())
    }
}

fn grid_section() -> impl View {
    let (vertical, horizontal) = (ScrollController::new(), ScrollController::new());
    let sort = signal(None::<SortState>);
    let mode = signal(GridSelectionMode::Row);
    let selection = GridSelectionController::new();
    let grid_selection = selection.clone();
    let echo_selection = selection.clone();
    let mode_controls = Row::new().spacing(8.0).children(
        [
            ("none", GridSelectionMode::None),
            ("row", GridSelectionMode::Row),
            ("cell", GridSelectionMode::Cell),
            ("range", GridSelectionMode::Range),
        ]
        .into_iter()
        .map(|(name, next)| {
            let selection = selection.clone();
            Keyed::new(
                format!("grid-mode-{name}"),
                Button::new(format!("{next:?}"))
                    .enabled(move || mode.get() != next)
                    .on_press(move || {
                        selection.set_selection(None);
                        mode.set(next);
                    }),
            )
        }),
    );
    const TITLES: [&str; 6] = ["Id", "Name", "City", "Amount", "Active", "Score"];
    // The widths and the display order are the app's (GALLERY-15), so it could persist them.
    let widths = signal(vec![80.0f32, 200.0, 180.0, 140.0, 90.0, 140.0]);
    let order = signal((0..TITLES.len()).collect::<Vec<_>>());
    const CITIES: [&str; 7] = [
        "Mumbai",
        "Pune",
        "Bengaluru",
        "Chennai",
        "Delhi",
        "Kolkata",
        "Hyderabad",
    ];
    let edit_error = signal("none".to_owned());
    let source = Rc::new(EditableGridSource {
        error: edit_error,
        columns: 6,
        score_column: 5,
        edits: RefCell::default(),
        generated: FnSource::new(1_000_000, |row, column| match column {
            0 => CellValue::Int(row as i64),
            1 => CellValue::Text(format!("Person {row}")),
            2 => CellValue::Text(CITIES[row % CITIES.len()].to_owned()),
            3 => CellValue::Float((row % 1000) as f64 * 1.25),
            4 => CellValue::Bool(row % 3 == 0),
            _ => CellValue::Int((row * 37 % 101) as i64),
        }),
    });
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(
            Text::new(
                "1 000 000 rows and 6 columns; only the visible cells are built. Scroll on both \
                 axes; resize/reorder headers. Double-click or F2 edits; Enter commits, Escape cancels.",
            )
            .style(TextStyle::label()),
        )
        .child(Keyed::new("grid-edit-error", Text::new(move || format!("Edit error: {}", edit_error.get()))))
        .child(Keyed::new("grid-selection-mode", mode_controls))
        .child(SizedBox::new(700.0, 400.0).child(Dyn::new(move || {
            let mode = mode.get();
            Keyed::new(
                "grid",
                DataGrid::new(
                    [
                        GridColumn::new("Id").width(80.0).sortable(true),
                        GridColumn::new("Name").width(200.0).sortable(true).editable(true),
                        GridColumn::new("City").width(180.0).sortable(true).editable(true),
                        GridColumn::new("Amount").width(140.0).sortable(true).editable(true),
                        GridColumn::new("Active").width(90.0).sortable(true).editable(true),
                        GridColumn::new("Score").width(140.0).sortable(true).editable(true),
                    ],
                    source.clone(),
                )
                .on_edit_error(move |_, error| edit_error.set(error.to_string()))
                .selection_mode(mode)
                .selection_controller(grid_selection.clone())
                .controllers(vertical, horizontal)
                .frozen_columns(1)
                .sort_state(sort)
                .column_widths(widths)
                .reorderable(true)
                .column_order(order),
            )
        })))
        .child(Keyed::new(
            "grid-selection",
            Text::new(move || {
                let Some(s) = echo_selection.selection() else {
                    return "Selection: none".to_owned();
                };
                match mode.get() {
                    GridSelectionMode::None => "Selection: none".to_owned(),
                    GridSelectionMode::Row => format!(
                        "Selection: rows {}..{}",
                        s.anchor.row.min(s.active.row),
                        s.anchor.row.max(s.active.row)
                    ),
                    GridSelectionMode::Cell => {
                        format!("Selection: cell ({}, {})", s.active.row, s.active.column)
                    }
                    GridSelectionMode::Range => format!(
                        "Selection: ({}, {})..({}, {})",
                        s.anchor.row, s.anchor.column, s.active.row, s.active.column
                    ),
                }
            }),
        ))
        // The sort state echoed (GALLERY-11); the source stays in its own order.
        .child(Keyed::new(
            "sort",
            Text::new(move || match sort.get() {
                None => "Sort: none".to_owned(),
                Some(state) => format!(
                    "Sort: {} {}",
                    TITLES[state.column],
                    if state.ascending {
                        "ascending"
                    } else {
                        "descending"
                    }
                ),
            }),
        ))
        .child(Keyed::new(
            "first-row",
            Text::new(move || format!("First row: {}", (vertical.offset() / 32.0) as usize)),
        ))
        // The Name column's width and the display order echoed (GALLERY-15).
        .child(Keyed::new(
            "name-width",
            Text::new(move || {
                let width = widths.with(|w| w.get(1).copied().unwrap_or(200.0));
                format!("Name width: {}", width.round() as i64)
            }),
        ))
        .child(Keyed::new(
            "order",
            Text::new(move || {
                let names = order.with(|o| {
                    o.iter()
                        .map(|&c| TITLES.get(c).copied().unwrap_or("?"))
                        .collect::<Vec<_>>()
                        .join(", ")
                });
                format!("Order: {names}")
            }),
        ))
}

fn animation_section() -> impl View {
    let grow = signal(false);
    let transition = Duration::from_millis(300);
    let width = animated(
        move || if grow.get() { 200.0f32 } else { 100.0 },
        transition,
    );
    let color = animated(move || if grow.get() { RED } else { PRIMARY }, transition);
    let controller = AnimationController::new(Duration::from_secs(1));
    controller.repeat(true);
    let card = signal(0usize);
    let x = controller
        .drive(Tween::new(0.0f32, 276.0))
        .curve(Curve::EaseInOut);
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(12.0)
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(8.0)
                .child(Keyed::new(
                    "grow",
                    Switch::new(grow).on_change(move |v| grow.set(v)),
                ))
                .child(Text::new("Grow the box (300 ms, ease in-out)")),
        )
        .child(Keyed::new(
            "box",
            ColorBox {
                width: width.into_prop(),
                color: color.into_prop(),
            },
        ))
        .child(
            Text::new("A controller repeating back and forth every second:")
                .style(TextStyle::label()),
        )
        .child(
            SizedBox::new(300.0, 24.0).child(
                Align::new(Alignment::CENTER_LEFT).child(
                    Padding::new(move || EdgeInsets {
                        left: x.get(),
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                    })
                    .child(Keyed::new(
                        "dot",
                        Icon::new(Icons::CIRCLE).size(24.0).color(PRIMARY),
                    )),
                ),
            ),
        )
        .child(Text::new("A switcher cross-fading between cards:").style(TextStyle::label()))
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(12.0)
                .child(Keyed::new(
                    "next",
                    Button::new("Next").on_press(move || card.update(|c| *c = (*c + 1) % 3)),
                ))
                .child(Keyed::new(
                    "switcher",
                    AnimatedSwitcher::new(card, |c| {
                        AnyView::new(Text::new(CARDS[c]).style(TextStyle::title()).color(PRIMARY))
                    }),
                )),
        )
}

/// The switcher's cards.
const CARDS: [&str; 3] = ["One", "Two", "Three"];

/// The Kitchen sink section (GALLERY-12..14): every widget on one scrolling page. It fills
/// the rest of the content column (`Expanded`) so the page scrolls.
fn kitchen_sink_section() -> impl View {
    let page = SizedBox::width(720.0).child(
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .spacing(24.0)
            .child(sink_buttons())
            .child(sink_toggles())
            .child(sink_slider())
            .child(sink_text())
            .child(sink_fields())
            .child(sink_media())
            .child(sink_lists())
            .child(tree::sink())
            .child(tabs::sink())
            .child(split::sink())
            .child(dock::sink())
            .child(menus::sink())
            .child(menus::bars_sink())
            .child(sink_grid())
            .child(sink_layout())
            .child(sink_scrolling())
            .child(sink_animation())
            .child(sink_focus()),
    );
    Expanded::new(Keyed::new(
        "kitchen-sink",
        Padding::all(0.0).child(Keyed::new("sink", ScrollView::new(page))),
    ))
}

/// A labelled card keyed `sink-<key>`: a title, the body and an optional echo keyed
/// `sink-<key>-state`.
fn card(key: &str, title: &'static str, body: impl View, echo: Option<AnyView>) -> impl View {
    let mut column = Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(Text::new(title).style(TextStyle::title()))
        .child(body);
    if let Some(echo) = echo {
        column = column.child(Keyed::new(format!("sink-{key}-state"), echo));
    }
    Keyed::new(format!("sink-{key}"), column)
}

/// A reactive-text echo for a card.
fn echo(text: impl Fn() -> String + 'static) -> Option<AnyView> {
    Some(AnyView::new(Text::new(text)))
}

/// A filled, rounded `width` × 40 box (the gallery's `ColorBox`).
fn swatch(width: f32, color: Color) -> ColorBox {
    ColorBox {
        width: width.into_prop(),
        color: color.into_prop(),
    }
}

fn on_off(state: bool) -> &'static str {
    if state { "on" } else { "off" }
}

fn sink_buttons() -> impl View {
    let presses = signal(0u32);
    card(
        "buttons",
        "Buttons",
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0)
            .child(
                Row::new()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .spacing(12.0)
                    .child(Keyed::new(
                        "sink-button",
                        Button::new("Press").on_press(move || presses.update(|n| *n += 1)),
                    ))
                    .child(Button::new("Disabled").enabled(false))
                    .child(Keyed::new(
                        "sink-icon_button",
                        IconButton::new(Icons::FAVORITE).on_press(|| {}),
                    ))
                    .child(Keyed::new(
                        "sink-tooltip",
                        Tooltip::new("A tooltip", Button::new("Hover me")),
                    )),
            )
            .child(sink_popup()),
        echo(move || format!("Pressed {}", presses.get())),
    )
}

/// An overlay inheriting an independently controlled local theme (GALLERY-19).
fn sink_popup() -> impl View {
    let (shown, dark) = (signal(false), signal(true));
    let local_theme = signal(Theme::dark());
    Builder::new(move |cx| {
        let label_theme = Theme::signal(cx).unwrap_or_else(|| signal(Theme::light()));
        let target = Keyed::new(
            "sink-popup-target",
            Row::new()
                .main_axis_size(MainAxisSize::Min)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(12.0)
                .child(Keyed::new(
                    "sink-overlay_portal",
                    Button::new("Open popup").on_press(move || shown.set(true)),
                ))
                .child(Keyed::new(
                    "sink-popup-dark",
                    Switch::new(dark).on_change(move |value| {
                        dark.set(value);
                        local_theme.set(if value { Theme::dark() } else { Theme::light() });
                    }),
                ))
                .child(ThemeProvider::new(label_theme, Text::new("Local dark"))),
        );
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0)
            .child(ThemeProvider::new(
                local_theme,
                OverlayPortal::new(target)
                    .visible(shown)
                    .anchor(
                        Anchor::new(Alignment::CENTER_RIGHT, Alignment::CENTER_LEFT)
                            .offset(12.0, 0.0),
                    )
                    .on_outside_press(move || shown.set(false))
                    .overlay(move || {
                        SizedBox::width(220.0).child(Keyed::new(
                            "sink-popup-close",
                            Button::new("Close themed popup").on_press(move || shown.set(false)),
                        ))
                    }),
            ))
            .child(Keyed::new(
                "sink-popup-state",
                Text::new(move || {
                    format!(
                        "Popup: {}, theme: {}",
                        if shown.get() { "visible" } else { "hidden" },
                        if dark.get() { "dark" } else { "light" }
                    )
                }),
            ))
    })
}

fn sink_toggles() -> impl View {
    let (checked, choice, on) = (signal(true), signal('b'), signal(false));
    card(
        "toggles",
        "Toggles",
        Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(12.0)
            .child(Keyed::new(
                "sink-checkbox",
                Checkbox::new(checked).on_change(move |v| checked.set(v)),
            ))
            .child(Checkbox::new(false).indeterminate(true))
            .child(Checkbox::new(true).enabled(false))
            .child(Keyed::new(
                "sink-radio",
                Radio::new('a', choice).on_change(move |v| choice.set(v)),
            ))
            .child(Keyed::new(
                "sink-radio-b",
                Radio::new('b', choice).on_change(move |v| choice.set(v)),
            ))
            .child(Keyed::new(
                "sink-radio-c",
                Radio::new('c', choice).on_change(move |v| choice.set(v)),
            ))
            .child(Keyed::new(
                "sink-switch",
                Switch::new(on).on_change(move |v| on.set(v)),
            )),
        echo(move || {
            format!(
                "Checkbox {}, Radio {}, Switch {}",
                on_off(checked.get()),
                choice.get(),
                on_off(on.get())
            )
        }),
    )
}

fn sink_slider() -> impl View {
    let (a, b) = (signal(50.0f64), signal(25.0f64));
    card(
        "sliders",
        "Slider",
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(4.0)
            .child(
                SizedBox::width(320.0).child(Keyed::new(
                    "sink-slider",
                    Slider::new(a)
                        .range(0.0, 100.0)
                        .on_change(move |v| a.set(v)),
                )),
            )
            .child(
                SizedBox::width(320.0).child(
                    Slider::new(b)
                        .range(0.0, 100.0)
                        .divisions(4)
                        .on_change(move |v| b.set(v)),
                ),
            ),
        echo(move || format!("Slider {} / {}", a.get().round(), b.get().round())),
    )
}

fn sink_text() -> impl View {
    card(
        "typography",
        "Text",
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0)
            .child(Keyed::new("sink-text", Text::new("Body text")))
            .child(Text::new("Title text").style(TextStyle::title()))
            .child(Text::new("Label text").style(TextStyle::label()))
            .child(Keyed::new(
                "sink-rich_text",
                RichText::new(
                    TextSpan::group()
                        .child(TextSpan::new("Rich "))
                        .child(TextSpan::new("bold").bold().color(RED))
                        .child(TextSpan::new(" text")),
                ),
            ))
            .child(Keyed::new("sink-divider", Divider::new()))
            .child(
                SizedBox::height(24.0).child(
                    Row::new()
                        .spacing(8.0)
                        .child(Text::new("Left"))
                        .child(Keyed::new("sink-vertical_divider", VerticalDivider::new()))
                        .child(Text::new("Right")),
                ),
            ),
        None,
    )
}

fn sink_fields() -> impl View {
    let (single, multi, off) = (
        TextEditingController::new(""),
        TextEditingController::new("Many lines\nof text"),
        TextEditingController::new("Disabled"),
    );
    card(
        "fields",
        "Text fields",
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0)
            .child(SizedBox::width(320.0).child(Keyed::new(
                "sink-text_field",
                TextField::new(single).placeholder("Type here"),
            )))
            .child(SizedBox::width(320.0).child(TextField::new(multi).max_lines(None)))
            .child(SizedBox::width(320.0).child(TextField::new(off).enabled(false))),
        echo(move || format!("Typed: {}", single.text())),
    )
}

fn sink_media() -> impl View {
    let images: AnyView = match (gradient(64, 64), gradient(64, 64), gradient(64, 64)) {
        (Ok(a), Ok(b), Ok(c)) => AnyView::new(
            Row::new()
                .spacing(12.0)
                .child(Keyed::new(
                    "sink-image",
                    Image::new(a).size(96.0, 64.0).fit(BoxFit::Contain),
                ))
                .child(Image::new(b).size(96.0, 64.0).fit(BoxFit::Cover))
                .child(Image::new(c).size(96.0, 64.0).fit(BoxFit::Fill)),
        ),
        _ => AnyView::new(SizedBox::shrink()),
    };
    card(
        "media",
        "Media",
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0)
            .child(
                Row::new()
                    .spacing(12.0)
                    .child(Keyed::new(
                        "sink-icon",
                        Icon::new(Icons::HOME).size(32.0).color(PRIMARY),
                    ))
                    .child(Icon::new(Icons::SEARCH).size(32.0).color(PRIMARY))
                    .child(Icon::new(Icons::SETTINGS).size(32.0).color(PRIMARY)),
            )
            .child(images),
        None,
    )
}

fn sink_lists() -> impl View {
    let selection = signal(None::<usize>);
    card(
        "lists",
        "Lists",
        SizedBox::new(400.0, 160.0).child(Keyed::new(
            "sink-list_view",
            ListView::separated(
                1000usize,
                32.0,
                |i| {
                    Padding::new(EdgeInsets::symmetric(8.0, 0.0)).child(
                        Align::new(Alignment::CENTER_LEFT).child(Text::new(format!("Row {i}"))),
                    )
                },
                1.0,
                |_| Divider::new(),
            )
            .selection(selection),
        )),
        echo(move || match selection.get() {
            Some(i) => format!("Selected {i}"),
            None => "Selected none".to_owned(),
        }),
    )
}

fn sink_grid() -> impl View {
    const TITLES: [&str; 3] = ["Id", "Name", "Score"];
    let (sort, filter, even) = (
        signal(None::<SortState>),
        signal(None::<RowFilter>),
        signal(false),
    );
    let edit_error = signal("none".to_owned());
    let selection = GridSelectionController::new();
    let grid_selection = selection.clone();
    let widths = signal(vec![80.0f32, 200.0, 120.0]);
    let order = signal(vec![0usize, 1, 2]);
    let source = Rc::new(EditableGridSource {
        error: edit_error,
        columns: 3,
        score_column: 2,
        edits: RefCell::default(),
        generated: FnSource::new(1000, |row, column| match column {
            0 => CellValue::Int(row as i64),
            1 => CellValue::Text(format!("Item {row}")),
            _ => CellValue::Int((row * 37 % 101) as i64),
        }),
    });
    card(
        "grid",
        "Data grid",
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0)
            .child(
                Row::new()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .spacing(8.0)
                    .child(Keyed::new(
                        "sink-grid-filter",
                        Switch::new(even).on_change(move |v| {
                            even.set(v);
                            let predicate: Option<RowFilter> = v.then(|| {
                                Rc::new(|source: &dyn DataSource, row: usize| {
                                    matches!(source.cell(row, 0), CellValue::Int(n) if n % 2 == 0)
                                }) as RowFilter
                            });
                            filter.set(predicate);
                        }),
                    ))
                    .child(Text::new("Even rows only")),
            )
            .child(
                SizedBox::new(600.0, 240.0).child(Keyed::new(
                    "sink-data_grid",
                    DataGrid::new(
                        [
                            GridColumn::new("Id").width(80.0),
                            GridColumn::new("Name").width(200.0).editable(true),
                            GridColumn::new("Score").width(120.0).editable(true),
                        ],
                        source,
                    )
                    .frozen_columns(1)
                    .column_widths(widths)
                    .column_order(order)
                    .reorderable(true)
                    .selection_mode(GridSelectionMode::Range)
                    .selection_controller(grid_selection)
                    .on_edit_error(move |_, error| edit_error.set(error.to_string()))
                    .client_sort(true)
                    .sort_state(sort)
                    .filter(filter),
                )),
            )
            .child(Text::new("Resize/drag headers; Shift+arrows selects a range. Double-click or F2 edits; Enter commits, Escape cancels.").style(TextStyle::label()))
            .child(Keyed::new("sink-grid-selection", Text::new(move || match selection.selection() {
                None => "Selection: none".to_owned(),
                Some(s) => format!("Selection: ({}, {})..({}, {})", s.anchor.row, s.anchor.column, s.active.row, s.active.column),
            })))
            .child(Keyed::new("sink-grid-edit-error", Text::new(move || format!("Edit error: {}", edit_error.get()))))
            .child(Row::new().spacing(24.0)
                .child(Keyed::new("sink-grid-name-width", Text::new(move || format!("Name width: {}", widths.with(|w| w.get(1).copied().unwrap_or(200.0)).round() as i64))))
                .child(Keyed::new("sink-grid-order", Text::new(move || format!("Order: {}", order.with(|o| o.iter().map(|&c| TITLES.get(c).copied().unwrap_or("?")).collect::<Vec<_>>().join(", ")))))))
            .child(Text::new("Million-row scrolling and cadence trial:").style(TextStyle::label()))
            .child(Keyed::new("sink-grid-stress-command", Text::new("cargo run --release -p data-grid-stress").style(TextStyle::label()))),
        echo(move || {
            let sort = match sort.get() {
                None => "none".to_owned(),
                Some(state) => format!(
                    "{} {}",
                    TITLES[state.column],
                    if state.ascending {
                        "ascending"
                    } else {
                        "descending"
                    }
                ),
            };
            format!("Sort: {sort}, rows {}", if even.get() { 500 } else { 1000 })
        }),
    )
}

fn sink_layout() -> impl View {
    card(
        "layout",
        "Layout",
        Column::new()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .spacing(8.0)
            .child(Keyed::new(
                "sink-row",
                Row::new()
                    .spacing(8.0)
                    .child(Keyed::new(
                        "sink-expanded",
                        Expanded::new(swatch(100.0, PRIMARY)),
                    ))
                    .child(Keyed::new(
                        "sink-flexible",
                        Flexible::new(swatch(100.0, RED)),
                    ))
                    .child(Keyed::new("sink-spacer", Spacer::new()))
                    .child(swatch(60.0, PRIMARY)),
            ))
            .child(Keyed::new(
                "sink-column",
                Column::new()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .spacing(4.0)
                    .child(swatch(100.0, PRIMARY))
                    .child(swatch(60.0, RED)),
            ))
            .child(
                SizedBox::new(200.0, 80.0).child(Keyed::new(
                    "sink-stack",
                    Stack::new()
                        .child(Keyed::new(
                            "sink-positioned",
                            Positioned::new(swatch(80.0, PRIMARY)).left(0.0).top(0.0),
                        ))
                        .child(Positioned::new(swatch(80.0, RED)).right(0.0).bottom(0.0)),
                )),
            )
            .child(
                Row::new()
                    .spacing(8.0)
                    .child(SizedBox::new(100.0, 60.0).child(Keyed::new(
                        "sink-center",
                        Center::new().child(swatch(40.0, PRIMARY)),
                    )))
                    .child(SizedBox::new(100.0, 60.0).child(Keyed::new(
                        "sink-align",
                        Align::new(Alignment::BOTTOM_RIGHT).child(swatch(40.0, RED)),
                    )))
                    .child(SizedBox::new(100.0, 60.0).child(Keyed::new(
                        "sink-padding",
                        Padding::all(8.0).child(swatch(100.0, PRIMARY)),
                    )))
                    .child(Keyed::new(
                        "sink-sized_box",
                        SizedBox::new(50.0, 30.0).child(swatch(100.0, RED)),
                    ))
                    .child(Keyed::new(
                        "sink-constrained_box",
                        ConstrainedBox::new(BoxConstraints::loose(Size::new(60.0, 20.0)))
                            .child(swatch(100.0, PRIMARY)),
                    ))
                    .child(Keyed::new(
                        "sink-limited_box",
                        LimitedBox::new().max_width(60.0).child(swatch(100.0, RED)),
                    )),
            ),
        None,
    )
}

fn sink_scrolling() -> impl View {
    let vertical = ScrollController::new();
    card(
        "scrolling",
        "Scrolling",
        Row::new()
            .spacing(16.0)
            .child(
                SizedBox::new(300.0, 240.0).child(Keyed::new(
                    "sink-scroll_view",
                    ScrollView::new(
                        Column::new()
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .children((0..30).map(|i| {
                                SizedBox::height(28.0).child(
                                    Align::new(Alignment::CENTER_LEFT)
                                        .child(Text::new(format!("Line {i}"))),
                                )
                            })),
                    )
                    .controller(vertical),
                )),
            )
            .child(
                SizedBox::new(300.0, 60.0).child(
                    ScrollView::new(Row::new().spacing(8.0).children(
                        (0..20).map(|i| swatch(60.0, if i % 2 == 0 { PRIMARY } else { RED })),
                    ))
                    .horizontal(),
                ),
            ),
        echo(move || format!("Offset {}", vertical.offset().round())),
    )
}

fn sink_animation() -> impl View {
    let (alpha, on) = (signal(1.0f64), signal(false));
    card(
        "animation",
        "Animation",
        Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(16.0)
            .child(
                SizedBox::width(200.0).child(
                    Slider::new(alpha)
                        .range(0.0, 1.0)
                        .on_change(move |v| alpha.set(v)),
                ),
            )
            .child(Keyed::new(
                "sink-opacity",
                Opacity::new(move || alpha.get() as f32, swatch(100.0, PRIMARY)),
            ))
            .child(Keyed::new(
                "sink-switcher-toggle",
                Switch::new(on).on_change(move |v| on.set(v)),
            ))
            .child(Keyed::new(
                "sink-animated_switcher",
                AnimatedSwitcher::new(on, |on| {
                    AnyView::new(
                        Text::new(if on { "On" } else { "Off" })
                            .style(TextStyle::title())
                            .color(PRIMARY),
                    )
                }),
            )),
        None,
    )
}

fn sink_focus() -> impl View {
    let focus = signal("none");
    let button = move |key: &'static str, name: &'static str| {
        Keyed::new(
            key,
            FocusEcho {
                name,
                focus,
                child: AnyView::new(Button::new(name)),
            },
        )
    };
    card(
        "focus",
        "Focus scope",
        Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(8.0)
            .child(Keyed::new(
                "sink-focus_scope",
                FocusScope::new(
                    Row::new()
                        .spacing(8.0)
                        .child(button("sink-focus-one", "One"))
                        .child(button("sink-focus-two", "Two"))
                        .child(button("sink-focus-three", "Three")),
                ),
            ))
            .child(Text::new("| outside the scope:"))
            .child(button("sink-focus-outside", "Outside")),
        echo(move || format!("Focus: {}", focus.get())),
    )
}

/// Reports its child's focus into a signal (the focus card's echo).
struct FocusEcho {
    name: &'static str,
    focus: Signal<&'static str>,
    child: AnyView,
}

impl View for FocusEcho {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.child.build(cx);
        let (name, focus) = (self.name, self.focus);
        cx.on_focus_change(id, move |focused| {
            if focused {
                focus.set(name);
            }
        });
        id
    }
}

/// A 40 px tall filled box whose width and color are reactive (the animated props' target).
struct ColorBox {
    width: Prop<f32>,
    color: Prop<Color>,
}

impl View for ColorBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::new(100.0, 40.0))),
            [],
        );
        cx.set_paint(id, ColorBoxPaint(PRIMARY));
        cx.bind(id, self.width, |e, width| {
            e.update_render::<RenderConstrainedBox>(|r| {
                r.additional = BoxConstraints::tight(Size::new(width, 40.0));
            });
        });
        cx.bind(id, self.color, |e, color| {
            e.update_paint::<ColorBoxPaint>(|p| p.0 = color);
        });
        id
    }
}

/// Fills the box with its color, rounded.
struct ColorBoxPaint(Color);

impl Paint for ColorBoxPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene().fill(
            RoundedRect::new(
                Rect::from_ltwh(0.0, 0.0, size.width, size.height),
                BorderRadius::circular(8.0),
            ),
            self.0,
        );
    }
}

/// A `width` × `height` image fading from the primary color to white.
fn gradient(
    width: u32,
    height: u32,
) -> std::result::Result<ImageData, tantu::scene::ResourceError> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let t = (x + y) as f32 / (width + height - 2) as f32;
            let mix = |c: f32| ((c * (1.0 - t) + t) * 255.0).round() as u8;
            pixels.extend_from_slice(&[mix(PRIMARY.r), mix(PRIMARY.g), mix(PRIMARY.b), 255]);
        }
    }
    ImageData::rgba8(width, height, pixels)
}
