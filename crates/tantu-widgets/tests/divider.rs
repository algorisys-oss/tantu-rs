//! Tests for `docs/specs/widgets/divider.md`, rules WIDGETS-DIVIDER-01..04.

use std::cell::Cell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Color, Rect};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::{Keyed, View};
use tantu_widgets::{Column, Divider, Padding, Row, SizedBox, VerticalDivider};

const OUTLINE_VARIANT: Color = Color::from_argb32(0xFFCA_C4D0);
const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);

/// Every filled rectangle in the last frame (the dividers are the only painters here).
fn fills(tester: &WidgetTester) -> Vec<(Rect, Color)> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { shape, color } => Some((shape.rect, color)),
            _ => None,
        })
        .collect()
}

/// A 300 × 200 window showing `view` directly.
fn window(view: impl View + 'static) -> WidgetTester {
    let view = std::cell::Cell::new(Some(view));
    WidgetTester::with_size(300.0, 200.0, move || view.take().expect("built once"))
}

#[test]
fn widgets_divider_01_height_and_width() {
    let tester = window(Column::new().child(Keyed::new("d", Divider::new())));
    assert_eq!(
        tester.rect(tester.find(&Finder::key("d"))),
        Rect::from_ltrb(0.0, 0.0, 300.0, 16.0)
    );
    let tester = window(Column::new().child(Keyed::new("d", Divider::new().height(24.0))));
    assert_eq!(
        tester.rect(tester.find(&Finder::key("d"))),
        Rect::from_ltrb(0.0, 0.0, 300.0, 24.0)
    );
    // In an unbounded width (a row) it is 0 wide.
    let tester = window(Row::new().child(Keyed::new("d", Divider::new())));
    let rect = tester.rect(tester.find(&Finder::key("d")));
    assert_eq!((rect.width(), rect.height()), (0.0, 16.0));
}

#[test]
fn widgets_divider_02_paints_the_line() {
    let tester = window(Column::new().child(Divider::new()));
    assert_eq!(
        fills(&tester),
        vec![(Rect::from_ltrb(0.0, 7.5, 300.0, 8.5), OUTLINE_VARIANT)]
    );
    let tester = window(
        Column::new().child(
            Divider::new()
                .thickness(2.0)
                .indent(10.0)
                .end_indent(20.0)
                .color(PRIMARY),
        ),
    );
    assert_eq!(
        fills(&tester),
        vec![(Rect::from_ltrb(10.0, 7.0, 280.0, 9.0), PRIMARY)]
    );
    // Bad values count as 0; an empty line isn't painted.
    for bad in [-1.0, f32::NAN] {
        let tester = window(Column::new().child(Divider::new().indent(bad).end_indent(bad)));
        assert_eq!(
            fills(&tester),
            vec![(Rect::from_ltrb(0.0, 7.5, 300.0, 8.5), OUTLINE_VARIANT)],
            "{bad}"
        );
        let tester = window(Column::new().child(Divider::new().thickness(bad)));
        assert!(fills(&tester).is_empty(), "{bad}");
    }
    let tester = window(Column::new().child(Divider::new().indent(200.0).end_indent(100.0)));
    assert!(fills(&tester).is_empty());
    let tester = window(Column::new().child(Divider::new().thickness(0.0)));
    assert!(fills(&tester).is_empty());
    // The look.
    let mut tester = WidgetTester::with_size(200.0, 120.0, || {
        Padding::all(8.0).child(
            Column::new()
                .spacing(8.0)
                .child(Divider::new())
                .child(
                    Divider::new()
                        .thickness(4.0)
                        .indent(20.0)
                        .end_indent(20.0)
                        .color(PRIMARY),
                )
                .child(
                    SizedBox::height(40.0).child(
                        Row::new()
                            .child(SizedBox::new(40.0, 40.0))
                            .child(VerticalDivider::new())
                            .child(
                                VerticalDivider::new()
                                    .thickness(3.0)
                                    .indent(8.0)
                                    .end_indent(8.0)
                                    .color(PRIMARY),
                            ),
                    ),
                ),
        )
    });
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/divider.png"
    ));
}

#[test]
fn widgets_divider_03_vertical() {
    let tester = window(Row::new().child(Keyed::new("d", VerticalDivider::new())));
    assert_eq!(
        tester.rect(tester.find(&Finder::key("d"))),
        Rect::from_ltrb(0.0, 0.0, 16.0, 200.0)
    );
    assert_eq!(
        fills(&tester),
        vec![(Rect::from_ltrb(7.5, 0.0, 8.5, 200.0), OUTLINE_VARIANT)]
    );
    let tester = window(
        Row::new().child(Keyed::new(
            "d",
            VerticalDivider::new()
                .width(24.0)
                .thickness(2.0)
                .indent(10.0)
                .end_indent(20.0)
                .color(PRIMARY),
        )),
    );
    assert_eq!(
        tester.rect(tester.find(&Finder::key("d"))),
        Rect::from_ltrb(0.0, 0.0, 24.0, 200.0)
    );
    assert_eq!(
        fills(&tester),
        vec![(Rect::from_ltrb(11.0, 10.0, 13.0, 180.0), PRIMARY)]
    );
    // In an unbounded height (a column) it is 0 tall.
    let tester = window(Column::new().child(Keyed::new("d", VerticalDivider::new())));
    let rect = tester.rect(tester.find(&Finder::key("d")));
    assert_eq!((rect.width(), rect.height()), (16.0, 0.0));
    assert!(fills(&tester).is_empty());
}

#[test]
fn widgets_divider_04_props_are_reactive() {
    type Signals = (Signal<f32>, Signal<f32>, Signal<f32>, Signal<Color>);
    let signals: Rc<Cell<Option<Signals>>> = Rc::new(Cell::new(None));
    let out = Rc::clone(&signals);
    let mut tester = WidgetTester::with_size(300.0, 200.0, move || {
        let (height, thickness, indent, color) = (
            signal(16.0f32),
            signal(1.0f32),
            signal(0.0f32),
            signal(OUTLINE_VARIANT),
        );
        out.set(Some((height, thickness, indent, color)));
        Column::new().child(Keyed::new(
            "d",
            Divider::new()
                .height(height)
                .thickness(thickness)
                .indent(indent)
                .end_indent(indent)
                .color(color),
        ))
    });
    let (height, thickness, indent, color) = signals.get().expect("built");
    assert_eq!(
        fills(&tester),
        vec![(Rect::from_ltrb(0.0, 7.5, 300.0, 8.5), OUTLINE_VARIANT)]
    );
    tester.enter(|| height.set(32.0));
    assert!(tester.pump());
    assert_eq!(tester.rect(tester.find(&Finder::key("d"))).height(), 32.0);
    assert_eq!(
        fills(&tester),
        vec![(Rect::from_ltrb(0.0, 15.5, 300.0, 16.5), OUTLINE_VARIANT)]
    );
    tester.enter(|| {
        thickness.set(2.0);
        indent.set(10.0);
        color.set(PRIMARY);
    });
    assert!(tester.pump());
    assert_eq!(
        fills(&tester),
        vec![(Rect::from_ltrb(10.0, 15.0, 290.0, 17.0), PRIMARY)]
    );
    assert!(!tester.pump());
}
