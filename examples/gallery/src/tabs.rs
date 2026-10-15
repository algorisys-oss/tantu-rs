//! Gallery and kitchen-sink tab examples (GALLERY-22..23).
use tantu::prelude::*;
use tantu::view::Keyed;

/// The `TabView` section's tab names, in order.
const TABS: [&str; 4] = ["General", "Advanced", "Data", "Archived"];

pub(super) fn section() -> impl View {
    let selected = signal(0usize);
    let name = TextEditingController::new("");
    let on = signal(true);
    let level = signal(40.0);
    let scrolled = signal(0usize);
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(Text::new(
            "Arrows move between tabs; Control+Tab cycles from anywhere inside. Pages keep their state.",
        ))
        .child(SizedBox::new(680.0, 280.0).child(Keyed::new(
            "tabs",
            TabView::new()
                .selected(selected)
                .tab(Tab::new("General"), move || {
                    Padding::all(16.0).child(
                        Column::new()
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .spacing(8.0)
                            .child(Text::new("Name"))
                            .child(Keyed::new(
                                "tabs-field",
                                TextField::new(name).placeholder("Type here"),
                            )),
                    )
                })
                .tab(Tab::new("Advanced"), move || {
                    Padding::all(16.0).child(
                        Column::new()
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .spacing(8.0)
                            .child(
                                Row::new()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .spacing(8.0)
                                    .child(Switch::new(on).on_change(move |v| on.set(v)))
                                    .child(Text::new("Sync automatically")),
                            )
                            .child(SizedBox::width(320.0).child(
                                Slider::new(level)
                                    .range(0.0, 100.0)
                                    .on_change(move |v| level.set(v)),
                            )),
                    )
                })
                .tab(Tab::new("Data"), || {
                    ListView::builder(200usize, 32.0, |i| {
                        Padding::new(EdgeInsets::symmetric(16.0, 0.0)).child(
                            Align::new(Alignment::CENTER_LEFT)
                                .child(Text::new(format!("Record {i}"))),
                        )
                    })
                })
                .tab(Tab::new("Archived").enabled(false), || {
                    Text::new("Nothing archived")
                }),
        )))
        .child(Keyed::new(
            "tab-echo",
            Text::new(move || format!("Tab: {}", TABS[selected.get().min(TABS.len() - 1)])),
        ))
        .child(Text::new("A scrollable bar of twelve tabs:"))
        .child(SizedBox::width(680.0).child(Keyed::new(
            "tabs-scrollable",
            TabBar::new(scrolled)
                .scrollable(true)
                .tabs((1..=12).map(|i| Tab::new(format!("Item {i}")))),
        )))
}

/// The tabs card's tab names.
const SINK_TABS: [&str; 3] = ["Note", "Option", "Input"];
/// The card's standalone bar's tabs.
const RANGES: [&str; 3] = ["Day", "Week", "Month"];

pub(super) fn sink() -> impl View {
    let selected = signal(0usize);
    let range = signal(0usize);
    let checked = signal(false);
    let input = TextEditingController::new("");
    let body = Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(SizedBox::width(360.0).child(Keyed::new(
            "sink-tab_bar",
            TabBar::new(range).tabs(RANGES.map(Tab::new)),
        )))
        .child(
            SizedBox::new(600.0, 200.0).child(Keyed::new(
                "sink-tab_view",
                TabView::new()
                    .selected(selected)
                    .tab(Tab::new("Note"), || {
                        Padding::all(16.0).child(Text::new("Pages are built on first visit."))
                    })
                    .tab(Tab::new("Option"), move || {
                        Align::new(Alignment::TOP_LEFT).child(
                            Padding::all(16.0).child(
                                Row::new()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .spacing(8.0)
                                    .child(
                                        Checkbox::new(checked).on_change(move |v| checked.set(v)),
                                    )
                                    .child(Text::new("Remember this")),
                            ),
                        )
                    })
                    .tab(Tab::new("Input"), move || {
                        Padding::all(16.0)
                            .child(TextField::new(input).placeholder("Kept while hidden"))
                    }),
            )),
        );
    super::card(
        "tabs",
        "Tabs",
        body,
        super::echo(move || {
            format!(
                "Tab: {}, range: {}",
                SINK_TABS[selected.get().min(2)],
                RANGES[range.get().min(2)]
            )
        }),
    )
}
