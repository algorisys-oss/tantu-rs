//! Gallery and kitchen-sink split view examples (GALLERY-24..25).
use tantu::prelude::*;
use tantu::view::Keyed;

/// The section's starting sizes: Folders, Messages and the preview column.
const INITIAL: [f32; 3] = [0.25, 0.35, 0.40];

/// "Sizes: 25 % · 35 % · 40 %" for a fractions signal.
fn sizes(fractions: Signal<Vec<f32>>) -> String {
    let parts: Vec<String> = fractions
        .get()
        .iter()
        .map(|f| format!("{:.0} %", f * 100.0))
        .collect();
    format!("Sizes: {}", parts.join(" · "))
}

/// A list of `count` rows reading `name i`.
fn list(name: &'static str, count: usize) -> impl View {
    ListView::builder(count, 32.0, move |i| {
        Padding::new(EdgeInsets::symmetric(12.0, 0.0))
            .child(Align::new(Alignment::CENTER_LEFT).child(Text::new(format!("{name} {}", i + 1))))
    })
}

pub(super) fn section() -> impl View {
    let fractions = signal(INITIAL.to_vec());
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(Text::new(
            "Drag a divider, or Tab to it and use the arrows, Home and End; Enter collapses Folders.",
        ))
        .child(SizedBox::new(720.0, 360.0).child(Keyed::new(
            "split",
            SplitView::horizontal()
                .fractions(fractions)
                .pane(
                    SplitPane::new(Keyed::new("split-folders", list("Folder", 12)))
                        .min_size(160.0)
                        .collapsible(true)
                        .label("Folders"),
                )
                .pane(
                    SplitPane::new(Keyed::new("split-messages", list("Message", 200)))
                        .min_size(120.0)
                        .label("Messages"),
                )
                .pane(
                    SplitPane::new(
                        SplitView::vertical()
                            .pane(
                                SplitPane::new(Keyed::new(
                                    "split-preview",
                                    Padding::all(12.0)
                                        .child(Text::new("Preview: select a message to read it.")),
                                ))
                                .min_size(80.0)
                                .label("Preview"),
                            )
                            .pane(
                                SplitPane::new(Keyed::new(
                                    "split-output",
                                    Padding::all(12.0).child(Text::new("Output")),
                                ))
                                .min_size(40.0)
                                .collapsible(true)
                                .label("Output"),
                            ),
                    )
                    .label("Preview column"),
                ),
        )))
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(12.0)
                .child(Keyed::new("split-echo", Text::new(move || sizes(fractions))))
                .child(Button::new("Reset").on_press(move || fractions.set(INITIAL.to_vec()))),
        )
}

pub(super) fn sink() -> impl View {
    let fractions = signal(vec![0.5, 0.5]);
    let body = SizedBox::new(600.0, 160.0).child(Keyed::new(
        "sink-split_view",
        SplitView::horizontal()
            .fractions(fractions)
            .pane(
                SplitPane::new(Padding::all(12.0).child(Text::new("Left pane")))
                    .min_size(80.0)
                    .label("Left"),
            )
            .pane(
                SplitPane::new(Padding::all(12.0).child(Text::new("Right pane")))
                    .min_size(80.0)
                    .label("Right"),
            ),
    ));
    super::card(
        "split",
        "Split view",
        body,
        super::echo(move || sizes(fractions)),
    )
}
