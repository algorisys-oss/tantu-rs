//! Gallery and kitchen-sink dialog examples (GALLERY-33..34).
use tantu::prelude::*;
use tantu::view::Keyed;

/// The Buttons section's dialog: "Delete…" opens a confirmation, echoed below.
pub(super) fn samples() -> impl View {
    let result = signal("none");
    let controller = DialogController::new();
    Row::new()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .spacing(12.0)
        .child(Keyed::new(
            "delete-dialog",
            Button::new("Delete…").on_press(move || controller.open()),
        ))
        .child(Keyed::new(
            "dialog-echo",
            Text::new(move || format!("Dialog: {}", result.get())),
        ))
        .child(
            Dialog::new(controller, move || {
                AlertDialog::new()
                    .title("Delete file?")
                    .content(Text::new(
                        "report.pdf will be deleted. This can't be undone.",
                    ))
                    .action(Button::new("Cancel").on_press(move || {
                        controller.close();
                        result.set("cancelled");
                    }))
                    .action(Button::new("Delete").on_press(move || {
                        controller.close();
                        result.set("deleted");
                    }))
            })
            .on_dismiss(move || result.set("dismissed")),
        )
}

/// The dialogs card: a non-dismissible rename dialog with a text field (GALLERY-34).
pub(super) fn sink() -> impl View {
    let name = signal("untitled".to_owned());
    let controller = DialogController::new();
    let body = Row::new()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .spacing(12.0)
        .child(Button::new("Rename…").on_press(move || controller.open()))
        .child(Keyed::new(
            "sink-dialog",
            Dialog::new(controller, move || {
                // Empty, with the current name as the placeholder; OK keeps it if nothing
                // was typed.
                let field = TextEditingController::new("");
                AlertDialog::new()
                    .title("Rename")
                    .content(SizedBox::width(240.0).child(Keyed::new(
                        "sink-rename-field",
                        TextField::new(field).placeholder(name.get_untracked()),
                    )))
                    .action(Button::new("OK").on_press(move || {
                        controller.close();
                        let text = field.text();
                        if !text.is_empty() {
                            name.set(text);
                        }
                    }))
            })
            .dismissible(false),
        ));
    super::card(
        "dialogs",
        "Dialogs",
        body,
        super::echo(move || format!("Name: {}", name.get())),
    )
}
