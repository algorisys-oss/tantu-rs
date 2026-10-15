//! Gallery and kitchen-sink form examples (GALLERY-36..37).
use tantu::prelude::*;
use tantu::view::Keyed;

/// "Ada" is a name; an email needs an "@" and a "." after it.
fn looks_like_email(text: &str) -> bool {
    text.split_once('@')
        .is_some_and(|(user, domain)| !user.is_empty() && domain.contains('.'))
}

/// The Text field section's form: Name and Email, Submit and Reset, and an echo.
pub(super) fn sample() -> impl View {
    let form = FormController::new();
    let (name, email) = (
        TextEditingController::new(""),
        TextEditingController::new(""),
    );
    let result = signal("not submitted".to_owned());
    let submit = move || {
        if form.validate() {
            result.set(format!("submitted {} <{}>", name.text(), email.text()));
        } else {
            let invalid = [
                name.text().trim().is_empty(),
                !looks_like_email(&email.text()),
            ]
            .into_iter()
            .filter(|bad| *bad)
            .count();
            result.set(format!(
                "fix {invalid} field{}",
                if invalid == 1 { "" } else { "s" }
            ));
        }
    };
    Keyed::new(
        "form",
        Form::new(
            form,
            SizedBox::width(320.0).child(
                Column::new()
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .spacing(8.0)
                    .child(
                        TextFormField::new(name)
                            .label("Name")
                            .helper("As it appears on the invoice")
                            .validator(validators::required("Enter your name")),
                    )
                    .child(
                        TextFormField::new(email)
                            .label("Email")
                            .validator(validators::required("Enter an email"))
                            .validator(validators::custom(
                                looks_like_email,
                                "That doesn't look like an email",
                            )),
                    )
                    .child(
                        Row::new()
                            .spacing(8.0)
                            .child(Button::new("Submit").on_press(submit))
                            .child(Button::new("Reset").on_press(move || {
                                form.reset();
                                result.set("not submitted".to_owned());
                            })),
                    )
                    .child(Keyed::new(
                        "form-echo",
                        Text::new(move || format!("Form: {}", result.get())),
                    )),
            ),
        )
        .autovalidate(AutovalidateMode::OnUserInteraction),
    )
}

/// The form card: one field validating as the user types (GALLERY-37).
pub(super) fn sink() -> impl View {
    let form = FormController::new();
    let code = TextEditingController::new("");
    let body = SizedBox::width(320.0).child(Keyed::new(
        "sink-text_form_field",
        Form::new(
            form,
            TextFormField::new(code)
                .label("Code")
                .helper("At least 3 characters")
                .validator(validators::min_length(3, "Too short"))
                .autovalidate(AutovalidateMode::OnUserInteraction),
        ),
    ));
    super::card(
        "form",
        "Form",
        body,
        super::echo(move || if form.is_valid() { "Valid" } else { "Invalid" }.to_owned()),
    )
}
