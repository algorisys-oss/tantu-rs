//! Gallery and kitchen-sink form examples (GALLERY-36..45).
use tantu::prelude::*;
use tantu::view::Keyed;

/// "Ada" is a name; an email needs an "@" and a "." after it.
fn looks_like_email(text: &str) -> bool {
    text.split_once('@')
        .is_some_and(|(user, domain)| !user.is_empty() && domain.contains('.'))
}

/// "Phone: none", "Phone: incomplete" or the digits (GALLERY-40).
fn masked_echo(
    label: &str,
    mask: &MaskFormatter,
    text: &str,
    show: impl Fn(&str) -> String,
) -> String {
    let shown = if text.is_empty() {
        "none".to_owned()
    } else if mask.is_complete(text) {
        show(text)
    } else {
        "incomplete".to_owned()
    };
    format!("{label}: {shown}")
}

/// The Text field section's form: Name, Email and a masked Phone, Submit and Reset, and
/// echoes.
pub(super) fn sample() -> impl View {
    let form = FormController::new();
    let (name, email, phone) = (
        TextEditingController::new(""),
        TextEditingController::new(""),
        TextEditingController::new(""),
    );
    let mask = MaskFormatter::new("(###) ###-####");
    let result = signal("not submitted".to_owned());
    let submit = {
        let mask = mask.clone();
        move || {
            if form.validate() {
                result.set(format!("submitted {} <{}>", name.text(), email.text()));
            } else {
                let phone = phone.text();
                let invalid = [
                    name.text().trim().is_empty(),
                    !looks_like_email(&email.text()),
                    !phone.is_empty() && !mask.is_complete(&phone),
                ]
                .into_iter()
                .filter(|bad| *bad)
                .count();
                result.set(format!(
                    "fix {invalid} field{}",
                    if invalid == 1 { "" } else { "s" }
                ));
            }
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
                        TextFormField::new(phone)
                            .label("Phone")
                            .placeholder("(555) 123-4567")
                            .input_formatter(mask.clone())
                            .validator(mask.validator("Enter all ten digits")),
                    )
                    .child(Keyed::new(
                        "mask-echo",
                        Text::new(move || {
                            masked_echo("Phone", &mask, &phone.text(), |t| mask.unmask(t))
                        }),
                    ))
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

/// The mask card: a date field (GALLERY-41).
pub(super) fn mask_sink() -> impl View {
    let date = TextEditingController::new("");
    let mask = MaskFormatter::new("##/##/####");
    let body = SizedBox::width(320.0).child(Keyed::new(
        "sink-masked_field",
        TextFormField::new(date)
            .label("Date")
            .placeholder("MM/DD/YYYY")
            .input_formatter(mask.clone())
            .validator(mask.validator("Enter the whole date"))
            .autovalidate(AutovalidateMode::OnUserInteraction),
    ));
    super::card(
        "mask",
        "Masked input",
        body,
        super::echo(move || masked_echo("Date", &mask, &date.text(), str::to_owned)),
    )
}

/// Twelve fruits for the Selection section's combo box (GALLERY-38).
const FRUITS: [&str; 12] = [
    "Apple",
    "Apricot",
    "Banana",
    "Blueberry",
    "Cherry",
    "Grape",
    "Kiwi",
    "Lemon",
    "Mango",
    "Orange",
    "Peach",
    "Pear",
];

/// The Selection section's combo box and its echo.
pub(super) fn combo_sample() -> impl View {
    let fruit = signal(None::<usize>);
    Row::new()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .spacing(12.0)
        .child(ComboBox::new(fruit, FRUITS).placeholder("Fruit"))
        .child(Keyed::new(
            "combo-echo",
            Text::new(move || {
                format!(
                    "Fruit: {}",
                    fruit
                        .get()
                        .and_then(|i| FRUITS.get(i))
                        .copied()
                        .unwrap_or("none")
                )
            }),
        ))
}

/// Currencies for the combo card (GALLERY-39).
const CURRENCIES: [&str; 5] = [
    "EUR — Euro",
    "GBP — Pound sterling",
    "INR — Indian rupee",
    "JPY — Japanese yen",
    "USD — US dollar",
];

/// The combo card: a currency picker echoing the code.
pub(super) fn combo_sink() -> impl View {
    let currency = signal(Some(0usize));
    let body = Keyed::new(
        "sink-combo_box",
        ComboBox::new(currency, CURRENCIES)
            .width(280.0)
            .placeholder("Currency"),
    );
    super::card(
        "combo",
        "Combo box",
        body,
        super::echo(move || {
            let code = currency
                .get()
                .and_then(|i| CURRENCIES.get(i))
                .map_or("none", |c| &c[..3]);
            format!("Currency: {code}")
        }),
    )
}

/// A date the gallery's calendars treat as today (fixed, so goldens don't change).
fn gallery_today() -> Date {
    Date::new(2026, 10, 16).expect("2026-10-16 exists")
}

/// The Selection section's date field and its echo (GALLERY-42).
pub(super) fn date_sample() -> impl View {
    let due = signal(None::<Date>);
    Row::new()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .spacing(12.0)
        .child(
            DatePickerField::new(due)
                .label("Due date")
                .first_date(Date::new(2026, 1, 1).expect("exists"))
                .last_date(Date::new(2027, 12, 31).expect("exists"))
                .today(gallery_today()),
        )
        .child(Keyed::new(
            "date-echo",
            Text::new(move || match due.get() {
                Some(date) => format!("Due: {date}"),
                None => "Due: none".to_owned(),
            }),
        ))
}

/// The date card: a standalone calendar (GALLERY-43).
pub(super) fn date_sink() -> impl View {
    let picked = signal(Some(gallery_today()));
    let body = Keyed::new(
        "sink-calendar_date_picker",
        CalendarDatePicker::new(picked).today(gallery_today()),
    );
    super::card(
        "date",
        "Date picker",
        body,
        super::echo(move || match picked.get() {
            Some(date) => format!("Picked: {date}"),
            None => "Picked: none".to_owned(),
        }),
    )
}

/// The Selection section's time field and its echo (GALLERY-44).
pub(super) fn time_sample() -> impl View {
    let start = signal(None::<Time>);
    Row::new()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .spacing(12.0)
        .child(TimePickerField::new(start).label("Start").minute_step(15))
        .child(Keyed::new(
            "time-echo",
            Text::new(move || match start.get() {
                Some(time) => format!("Start: {time}"),
                None => "Start: none".to_owned(),
            }),
        ))
}

/// The time card: a field starting at noon (GALLERY-45).
pub(super) fn time_sink() -> impl View {
    let time = signal(Time::new(12, 0));
    let body = Keyed::new(
        "sink-time_picker_field",
        TimePickerField::new(time).label("Reminder"),
    );
    super::card(
        "time",
        "Time picker",
        body,
        super::echo(move || match time.get() {
            Some(time) => format!("Time: {time}"),
            None => "Time: none".to_owned(),
        }),
    )
}
