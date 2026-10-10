//! The Phase 2 layout demo: a header, a fixed sidebar, proportional bands, a stack with a
//! badge and a footer, composed from Flutter-style layout widgets. Spec:
//! `docs/specs/examples/layout-demo.md`.

use tantu::core::Rect;
use tantu::layout::{BoxConstraints, RenderConstrainedBox};
use tantu::prelude::*;
use tantu::view::{BuildCx, ElementId, Keyed, Paint, PaintCx};

/// The sidebar's sections: (key, name).
const SECTIONS: [(&str, &str); 3] = [
    ("section-overview", "Overview"),
    ("section-reports", "Reports"),
    ("section-settings", "Settings"),
];

/// The demo's view.
pub fn layout_demo() -> impl View {
    let section = signal(SECTIONS[0].1.to_owned());
    Column::new()
        .child(Keyed::new("header", header()))
        .child(Expanded::new(Keyed::new(
            "body",
            Row::new()
                .cross_axis_alignment(CrossAxisAlignment::Stretch)
                .child(Keyed::new("sidebar", sidebar(section)))
                .child(Expanded::new(Keyed::new("content", content()))),
        )))
        .child(Keyed::new(
            "footer",
            Padding::all(12.0).child(
                Center::new().child(Text::new(move || format!("Section: {}", section.get()))),
            ),
        ))
}

/// A title, a spacer and two buttons.
fn header() -> impl View {
    Padding::all(12.0).child(
        Row::new()
            .spacing(8.0)
            .child(Text::new("Layout demo").style(TextStyle::title()))
            .child(Spacer::new())
            .child(Button::new("Help"))
            .child(Button::new("About")),
    )
}

/// A fixed-width column of section buttons.
fn sidebar(section: Signal<String>) -> impl View {
    let buttons = SECTIONS.map(|(key, name)| {
        Keyed::new(
            key,
            Button::new(name).on_press(move || section.set(name.to_owned())),
        )
    });
    SizedBox::width(160.0).child(
        Padding::all(12.0).child(
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Stretch)
                .spacing(8.0)
                .children(buttons),
        ),
    )
}

/// Three bands in ratio 1 : 2 : 1 beside a stack with a badge in its top-right corner.
fn content() -> impl View {
    let band = |key: &'static str, flex: u32, color: u32| {
        Expanded::new(Keyed::new(key, Swatch::new(Color::from_argb32(color)))).flex(flex)
    };
    Padding::all(12.0).child(
        Row::new()
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .spacing(12.0)
            .child(Expanded::new(
                Column::new()
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .spacing(12.0)
                    .child(band("band1", 1, 0xFFD0BCFF))
                    .child(band("band2", 2, 0xFFB69DF8))
                    .child(band("band3", 1, 0xFFD0BCFF)),
            ))
            .child(Keyed::new(
                "stack",
                SizedBox::width(200.0).child(
                    Stack::new()
                        .fit(StackFit::Expand)
                        .child(Swatch::new(Color::from_argb32(0xFFE8DEF8)))
                        .child(
                            Positioned::new(Keyed::new(
                                "badge",
                                Swatch::new(Color::from_argb32(0xFFB3261E)),
                            ))
                            .top(8.0)
                            .right(8.0)
                            .width(40.0)
                            .height(24.0),
                        ),
                ),
            )),
    )
}

/// A box filled with one color (an example of a custom `Paint`).
pub struct Swatch {
    color: Color,
}

impl Swatch {
    /// A box filling its constraints with `color`.
    pub fn new(color: Color) -> Self {
        Swatch { color }
    }
}

/// Fills the element's bounds.
struct SwatchPaint(Color);

impl Paint for SwatchPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene()
            .fill_rect(Rect::from_ltwh(0.0, 0.0, size.width, size.height), self.0);
    }
}

impl View for Swatch {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::new(BoxConstraints::default()), []);
        cx.set_paint(id, SwatchPaint(self.color));
        id
    }
}
