//! Gallery and kitchen-sink docking examples (GALLERY-26..27).
use tantu::prelude::*;
use tantu::view::Keyed;

/// "Explorer" for "explorer".
fn title(id: &PanelId) -> String {
    let mut chars = id.as_str().chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

/// The IDE layout: Explorer, Search | Editor, Readme / Terminal, Problems.
fn ide() -> DockLayout {
    DockLayout::new(DockNode::row([
        (0.25, DockNode::tabs(["explorer", "search"])),
        (
            0.75,
            DockNode::column([
                (0.6, DockNode::tabs(["editor", "readme"])),
                (0.4, DockNode::tabs(["terminal", "problems"])),
            ]),
        ),
    ]))
}

/// "Active: <title> · Panels: <count>", the active group's active panel.
fn echo(layout: Signal<DockLayout>, active: Signal<Option<DockGroupId>>) -> String {
    layout.with(|l| {
        let shown = active
            .get()
            .and_then(|g| l.active(g))
            .map(title)
            .unwrap_or_else(|| "none".to_owned());
        format!("Active: {shown} · Panels: {}", l.panels().len())
    })
}

/// Padded text filling a panel.
fn note(text: &'static str) -> impl View {
    Padding::all(12.0).child(Text::new(text))
}

pub(super) fn section() -> impl View {
    let layout = signal(ide());
    let active = signal(None);
    let saved: Signal<Option<String>> = signal(None);
    let editor = TextEditingController::new("");
    let known = [
        "explorer", "search", "editor", "readme", "terminal", "problems",
    ];
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(Text::new(
            "Drag tabs onto strips, group edges or the dock's edges; close them (middle click too); drag dividers. Control+Tab cycles, Control+W closes.",
        ))
        .child(SizedBox::new(720.0, 340.0).child(Keyed::new(
            "dock",
            DockArea::new(layout, move |id: &PanelId| -> AnyView {
                match id.as_str() {
                    "explorer" => AnyView::new(note("src/\n  main.rs\n  lib.rs\nCargo.toml")),
                    "search" => AnyView::new(note("Search across files")),
                    "editor" => AnyView::new(Align::new(Alignment::TOP_LEFT).child(
                        Padding::all(12.0).child(Keyed::new(
                            "dock-editor",
                            TextField::new(editor).placeholder("fn main() {}"),
                        )),
                    )),
                    "readme" => AnyView::new(note("A docked readme.")),
                    "terminal" => AnyView::new(note("$ cargo run -p gallery")),
                    _ => AnyView::new(note("No problems")),
                }
            })
            .title(title)
            .closable(|id| id.as_str() != "explorer")
            .active_group(active)
            .placeholder(|| note("Everything is closed; Reset layout brings it back.")),
        )))
        .child(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .spacing(12.0)
                .child(Keyed::new("dock-echo", Text::new(move || echo(layout, active))))
                .child(Button::new("Reset layout").on_press(move || layout.set(ide())))
                .child(Button::new("Save").on_press(move || saved.set(Some(layout.get().save()))))
                .child(Button::new("Restore").on_press(move || {
                    if let Some(text) = saved.get() {
                        if let Ok(restored) =
                            DockLayout::load(&text, |id| known.contains(&id.as_str()))
                        {
                            layout.set(restored);
                        }
                    }
                })),
        )
}

pub(super) fn sink() -> impl View {
    let layout = signal(DockLayout::new(DockNode::row([
        (0.5, DockNode::tabs(["notes"])),
        (0.5, DockNode::tabs(["tasks", "log"])),
    ])));
    let active = signal(None);
    let body = SizedBox::new(600.0, 220.0).child(Keyed::new(
        "sink-dock_area",
        DockArea::new(layout, |id: &PanelId| -> AnyView {
            match id.as_str() {
                "notes" => AnyView::new(note("Notes")),
                "tasks" => AnyView::new(note("Tasks")),
                _ => AnyView::new(note("Log")),
            }
        })
        .title(title)
        .active_group(active),
    ));
    super::card(
        "dock",
        "Docking",
        body,
        super::echo(move || echo(layout, active)),
    )
}
