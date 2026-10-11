//! Tests for `docs/specs/widgets/rich-text.md`, rules WIDGETS-RICHTEXT-01..03.

use std::cell::Cell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::Color;
use tantu_view::layout::{RenderParagraph, StyleSpan};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::text::{FontFamily, TextStyle, TextSystem};
use tantu_view::{ElementId, Keyed, ViewTree};
use tantu_widgets::{Column, RichText, TextSpan};

const ON_SURFACE: Color = Color::from_argb32(0xFF1C_1B1F);
const RED: Color = Color::from_argb32(0xFFC8_0000);

/// The paragraph of the tree's top element.
fn paragraph(tree: &ViewTree) -> RenderParagraph {
    let top = tree.children(tree.root())[0];
    let layout = tree.layout_id(top).expect("render element");
    tree.layout_tree()
        .get::<RenderParagraph>(layout)
        .expect("a paragraph")
        .clone()
}

/// A span with a bold red word and a bigger exclamation mark.
fn sample() -> TextSpan {
    TextSpan::group()
        .child(TextSpan::new("Hello "))
        .child(TextSpan::new("bold").bold().color(RED))
        .child(TextSpan::new(" world").child(TextSpan::new("!").size(30.0)))
}

/// The glyph runs of the element: (glyph count, color).
fn runs(tester: &WidgetTester, element: ElementId) -> Vec<(usize, Color)> {
    let frame = tester.frame();
    frame
        .entries_for(element)
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some((frame.scene.glyphs(run).len(), run.color)),
            _ => None,
        })
        .collect()
}

#[test]
fn widgets_richtext_01_flatten() {
    assert_eq!(sample().to_plain_text(), "Hello bold world!");
    assert_eq!(
        TextSpan::new("a").child(TextSpan::new("b")).to_plain_text(),
        "ab"
    );
    let system = TextSystem::without_system_fonts();
    let styles = system.styles();
    let tree = ViewTree::with_text_styles(styles.clone(), || RichText::new(sample()));
    let p = paragraph(&tree);
    assert_eq!(&*p.text, "Hello bold world!");
    let bold = styles.key(TextStyle {
        weight: 700.0,
        ..TextStyle::body()
    });
    let big = styles.key(TextStyle {
        size: 30.0,
        ..TextStyle::body()
    });
    assert_eq!(
        &*p.spans,
        [
            StyleSpan {
                start: 6,
                end: 10,
                style: bold,
                color: Some(RED)
            },
            StyleSpan {
                start: 16,
                end: 17,
                style: big,
                color: None
            },
        ]
    );
    // A full style replaces the inherited one, and overrides apply on top of it; colors
    // inherit through the tree; the base style and color come from `RichText`.
    let span = TextSpan::new("a")
        .color(RED)
        .child(
            TextSpan::new("b")
                .style(TextStyle::title())
                .size(10.0)
                .child(TextSpan::new("c").italic(true)),
        )
        .child(TextSpan::new("d").family(FontFamily::Monospace));
    let tree = ViewTree::with_text_styles(styles.clone(), move || {
        RichText::new(span)
            .style(TextStyle::label())
            .color(Color::BLACK)
    });
    let p = paragraph(&tree);
    assert_eq!(p.style, styles.key(TextStyle::label()));
    let title10 = TextStyle {
        size: 10.0,
        ..TextStyle::title()
    };
    assert_eq!(
        &*p.spans,
        [
            StyleSpan {
                start: 0,
                end: 1,
                style: styles.key(TextStyle::label()),
                color: Some(RED)
            },
            StyleSpan {
                start: 1,
                end: 2,
                style: styles.key(title10.clone()),
                color: Some(RED)
            },
            StyleSpan {
                start: 2,
                end: 3,
                style: styles.key(TextStyle {
                    italic: true,
                    ..title10
                }),
                color: Some(RED)
            },
            StyleSpan {
                start: 3,
                end: 4,
                style: styles.key(TextStyle {
                    family: FontFamily::Monospace,
                    ..TextStyle::label()
                }),
                color: Some(RED)
            },
        ]
    );
}

#[test]
fn widgets_richtext_02_paragraph_and_paint() {
    let mut tester = WidgetTester::with_size(400.0, 100.0, || {
        Column::new().child(Keyed::new(
            "r",
            RichText::new(sample()).max_lines(2).soft_wrap(false),
        ))
    });
    let element = tester.find(&Finder::text("Hello bold world!"));
    assert_eq!(element, tester.find(&Finder::key("r")));
    // One run per parley run: the bigger "!" is a run of its own.
    assert_eq!(
        runs(&tester, element),
        [(6, ON_SURFACE), (4, RED), (6, ON_SURFACE), (1, ON_SURFACE)]
    );
    assert!(tester.rect(element).height() > 0.0);
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/rich-text.png"
    ));
}

#[test]
fn widgets_richtext_03_reactive() {
    type Signals = (Signal<TextSpan>, Signal<Color>);
    let out: Rc<Cell<Option<Signals>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(400.0, 100.0, move || {
            let (span, color) = (signal(sample()), signal(ON_SURFACE));
            out.set(Some((span, color)));
            Column::new().child(Keyed::new("r", RichText::new(span).color(color)))
        })
    };
    let (span, color) = out.get().expect("built");
    let element = tester.find(&Finder::key("r"));
    tester.enter(|| span.set(TextSpan::new("Plain ").child(TextSpan::new("red").color(RED))));
    assert!(tester.pump());
    tester.find(&Finder::text("Plain red"));
    assert_eq!(runs(&tester, element), [(6, ON_SURFACE), (3, RED)]);
    tester.enter(|| color.set(Color::BLACK));
    assert!(tester.pump());
    assert_eq!(runs(&tester, element), [(6, Color::BLACK), (3, RED)]);
    assert!(!tester.pump());
}
