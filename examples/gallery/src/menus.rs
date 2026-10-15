//! Gallery and kitchen-sink menu examples (GALLERY-29..30).
use tantu::prelude::*;
use tantu::view::Keyed;

/// A 220 × 72 box centring `text`, the context menus' target.
fn target(text: &'static str) -> impl View {
    SizedBox::new(220.0, 72.0).child(Center::new().child(Text::new(text)))
}

/// The Buttons section's menus: a "More" anchor and a context-menu box, with an echo.
pub(super) fn samples() -> impl View {
    let chosen = signal("none".to_owned());
    let wrap = signal(true);
    let pick = move |name: &'static str| move || chosen.set(name.to_owned());
    let more = MenuController::new();
    let anchor =
        MenuAnchor::new(more, Button::new("More").on_press(move || more.toggle())).items([
            MenuItem::new("Cut")
                .shortcut("Ctrl+X")
                .icon(Icons::CONTENT_CUT)
                .on_select(pick("Cut")),
            MenuItem::new("Copy")
                .shortcut("Ctrl+C")
                .icon(Icons::CONTENT_COPY)
                .on_select(pick("Copy")),
            MenuItem::new("Paste")
                .shortcut("Ctrl+V")
                .icon(Icons::CONTENT_PASTE)
                .on_select(pick("Paste")),
            MenuItem::divider(),
            MenuItem::new("Word wrap").checked(wrap).on_select(move || {
                wrap.update(|w| *w = !*w);
                let state = if wrap.get_untracked() { "on" } else { "off" };
                chosen.set(format!("Word wrap {state}"));
            }),
            MenuItem::submenu(
                "Open recent",
                [
                    MenuItem::new("notes.md").on_select(pick("notes.md")),
                    MenuItem::new("todo.md").on_select(pick("todo.md")),
                    MenuItem::new("readme.md").on_select(pick("readme.md")),
                ],
            ),
            MenuItem::new("Print")
                .enabled(false)
                .on_select(pick("Print")),
        ]);
    let region = ContextMenuRegion::new(Keyed::new("context-region", target("Right-click here")))
        .items([
            MenuItem::new("Rename").on_select(pick("Rename")),
            MenuItem::new("Duplicate").on_select(pick("Duplicate")),
            MenuItem::divider(),
            MenuItem::new("Delete").on_select(pick("Delete")),
        ]);
    Row::new()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .spacing(12.0)
        .child(anchor)
        .child(region)
        .child(Keyed::new(
            "menu-echo",
            Text::new(move || format!("Menu: {}", chosen.get())),
        ))
}

pub(super) fn sink() -> impl View {
    let chosen = signal("none".to_owned());
    let pick = move |name: &'static str| move || chosen.set(name.to_owned());
    let actions = MenuController::new();
    let body = Row::new()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .spacing(12.0)
        .child(Keyed::new(
            "sink-menu_anchor",
            MenuAnchor::new(
                actions,
                Button::new("Actions").on_press(move || actions.toggle()),
            )
            .items([
                MenuItem::new("Duplicate").on_select(pick("Duplicate")),
                MenuItem::new("Archive").on_select(pick("Archive")),
                MenuItem::divider(),
                MenuItem::new("Delete").on_select(pick("Delete")),
            ]),
        ))
        .child(Keyed::new(
            "sink-context_menu",
            ContextMenuRegion::new(target("Right-click")).items([
                MenuItem::new("Inspect").on_select(pick("Inspect")),
                MenuItem::new("Refresh").on_select(pick("Refresh")),
            ]),
        ));
    super::card(
        "menus",
        "Menus",
        body,
        super::echo(move || format!("Menu: {}", chosen.get())),
    )
}
