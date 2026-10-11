//! The Tantu widget gallery (spec `docs/specs/examples/gallery.md`): a header, a sidebar of
//! sections and the selected section's samples.

use tantu::prelude::*;
use tantu::scene::ImageData;
use tantu::view::Keyed;

/// The section names, in sidebar order.
pub const SECTIONS: [&str; 7] = [
    "Buttons",
    "Text",
    "Selection",
    "Text field",
    "Lists",
    "Media",
    "Animation",
];

const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);
const RED: Color = Color::from_argb32(0xFFB3_261E);

/// The key of a section's sidebar button.
fn nav_key(section: &str) -> String {
    format!("nav-{}", section.to_lowercase().replace(' ', "-"))
}

/// The gallery's view.
pub fn gallery() -> impl View {
    let section = signal(SECTIONS[0]);
    Column::new()
        .child(Keyed::new("header", header(section)))
        .child(Expanded::new(
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Stretch)
                .child(Keyed::new("sidebar", sidebar(section)))
                .child(Expanded::new(Keyed::new("content", content(section)))),
        ))
}

fn header(section: Signal<&'static str>) -> impl View {
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
            .child(dark_mode_switch()),
    )
}

/// "Dark" and a switch that sets the window's theme to dark or light (GALLERY-05). The theme
/// signal comes from the runner (or the tester), so the whole window follows.
fn dark_mode_switch() -> impl View {
    Builder::new(|cx| {
        let theme = Theme::signal(cx).unwrap_or_else(|| signal(Theme::light()));
        let dark = signal(false);
        effect(move || {
            theme.set(if dark.get() {
                Theme::dark()
            } else {
                Theme::light()
            });
        });
        Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(4.0)
            .child(Text::new("Dark").style(TextStyle::label()))
            .child(Keyed::new(
                "dark",
                Switch::new(dark).on_change(move |v| dark.set(v)),
            ))
    })
}

fn sidebar(section: Signal<&'static str>) -> impl View {
    SizedBox::width(200.0).child(
        Padding::all(8.0).child(
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Stretch)
                .spacing(8.0)
                .children(SECTIONS.iter().map(|name| {
                    Keyed::new(
                        nav_key(name),
                        Button::new(*name).on_press(move || section.set(name)),
                    )
                })),
        ),
    )
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
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .spacing(8.0)
        .child(Text::new("10 000 rows, built as they scroll into view.").style(TextStyle::label()))
        .child(SizedBox::new(400.0, 320.0).child(Keyed::new(
            "list",
            ListView::builder(10_000usize, 32.0, |i| {
                Padding::new(EdgeInsets::symmetric(8.0, 0.0))
                    .child(Align::new(Alignment::CENTER_LEFT).child(Text::new(format!("Row {i}"))))
            }),
        )))
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

/// The Animation section (GALLERY-06).
fn animation_section() -> impl View {
    SizedBox::shrink()
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
