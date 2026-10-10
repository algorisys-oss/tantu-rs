//! Tests for `docs/specs/text/system.md`, rules TEXT-SYS-01..09. They use Liberation Sans
//! (SIL OFL 1.1, `tests/fonts/`) in a system without system fonts, so results don't depend on
//! the machine.

use tantu_core::{Color, Point, Size};
use tantu_layout::{TextMeasure, TextMetrics, TextStyleKey};
use tantu_scene::{Command, Resources, Scene};
use tantu_text::{FontFamily, TextStyle, TextSystem};

const INF: f32 = f32::INFINITY;
const FONT: &[u8] = include_bytes!("fonts/LiberationSans-Regular.ttf");

/// A system with only Liberation Sans, which is also the default family.
fn system() -> TextSystem {
    let mut text = TextSystem::without_system_fonts();
    assert_eq!(text.register_font(FONT.to_vec()), ["Liberation Sans"]);
    text.set_default_family(FontFamily::Named("Liberation Sans".into()));
    text
}

fn body(text: &mut TextSystem) -> TextStyleKey {
    text.style(TextStyle {
        family: FontFamily::Named("Liberation Sans".into()),
        ..TextStyle::default()
    })
}

#[test]
fn text_sys_01_fonts() {
    let mut empty = TextSystem::without_system_fonts();
    let key = empty.style(TextStyle::default());
    assert_eq!(empty.measure("Hello", key, INF, None).size.width, 0.0);
    assert!(empty.register_font(vec![1, 2, 3, 4]).is_empty());
    assert!(empty.register_font(Vec::new()).is_empty());
    assert_eq!(empty.measure("Hello", key, INF, None).size.width, 0.0);
    let mut text = system();
    let key = body(&mut text);
    assert!(text.measure("Hello", key, INF, None).size.width > 0.0);
}

#[test]
fn text_sys_02_styles() {
    let mut text = system();
    let a = text.style(TextStyle::default());
    let b = text.style(TextStyle::default());
    let c = text.style(TextStyle {
        size: 20.0,
        ..TextStyle::default()
    });
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(text.text_style(c).map(|s| s.size), Some(20.0));
    assert_eq!(text.text_style(TextStyleKey(9_999)), None);
    let defaults = TextStyle::default();
    assert_eq!(defaults.family, FontFamily::SansSerif);
    assert_eq!(
        (defaults.size, defaults.weight, defaults.italic),
        (16.0, 400.0, false)
    );
    assert_eq!(defaults.line_height, None);
    // Sanitizing.
    let odd = text.style(TextStyle {
        size: f32::NAN,
        weight: 5000.0,
        line_height: Some(-1.0),
        ..TextStyle::default()
    });
    let style = text.text_style(odd).expect("interned").clone();
    assert_eq!(
        (style.size, style.weight, style.line_height),
        (16.0, 1000.0, None)
    );
    let also_default = text.style(TextStyle {
        size: -3.0,
        weight: f32::NAN,
        ..TextStyle::default()
    });
    assert_eq!(also_default, a);
}

#[test]
fn text_sys_03_fallback() {
    let mut text = system();
    let named = body(&mut text);
    let missing = text.style(TextStyle {
        family: FontFamily::Named("No Such Family".into()),
        ..TextStyle::default()
    });
    let generic = text.style(TextStyle {
        family: FontFamily::Monospace,
        ..TextStyle::default()
    });
    let expected = text.measure("Hello", named, INF, None);
    assert_eq!(text.measure("Hello", missing, INF, None), expected);
    assert_eq!(text.measure("Hello", generic, INF, None), expected);
    // An unknown key measures with the default style.
    let default_key = text.style(TextStyle::default());
    let default = text.measure("Hello", default_key, INF, None);
    assert_eq!(
        text.measure("Hello", TextStyleKey(424_242), INF, None),
        default
    );
}

#[test]
fn text_sys_04_metrics() {
    let mut text = system();
    let key = body(&mut text);
    assert_eq!(text.measure("", key, 100.0, None), TextMetrics::default());
    let one = text.measure("Hello", key, INF, None);
    assert_eq!(one.line_count, 1);
    assert!(one.size.width > 20.0 && one.size.width < 60.0, "{one:?}");
    assert!(one.size.height > 0.0);
    assert!(one.first_baseline > 0.0 && one.first_baseline < one.size.height);
    assert_eq!(one.first_baseline, one.last_baseline);
}

#[test]
fn text_sys_05_lines() {
    let mut text = system();
    let key = body(&mut text);
    let short = text.measure("Hello", key, INF, None);
    let long = text.measure("Hello world", key, INF, None);
    assert!(long.size.width > short.size.width);
    let wrapped = text.measure("Hello world foo", key, 60.0, None);
    assert!(wrapped.line_count >= 2, "{wrapped:?}");
    assert!(wrapped.size.width <= 60.0, "{wrapped:?}");
    // A word wider than the width stays on its own line, wider than the width.
    let wide = text.measure("Incomprehensibilities", key, 30.0, None);
    assert_eq!(wide.line_count, 1);
    assert!(wide.size.width > 30.0);
    // Hard breaks.
    let two = text.measure("a\nb", key, INF, None);
    assert_eq!(two.line_count, 2);
    // Uniform line heights.
    let line = short.size.height;
    let three = text.measure("a\nb\nc", key, INF, None);
    assert_eq!(three.size.height, 3.0 * line);
    assert_eq!(three.last_baseline - three.first_baseline, 2.0 * line);
}

#[test]
fn text_sys_06_max_lines() {
    let mut text = system();
    let key = body(&mut text);
    let all = text.measure("a\nb\nc\nd", key, INF, None);
    let two = text.measure("a\nb\nc\nd", key, INF, Some(2));
    assert_eq!(all.line_count, 4);
    assert_eq!(two.line_count, 2);
    assert_eq!(two.size.height, all.size.height / 2.0);
    assert_eq!(two.first_baseline, all.first_baseline);
    assert_eq!(
        two.last_baseline - two.first_baseline,
        all.size.height / 4.0
    );
    assert_eq!(text.measure("a\nb", key, INF, Some(9)).line_count, 2);
}

#[test]
fn text_sys_07_min_intrinsic_width() {
    let mut text = system();
    let key = body(&mut text);
    let min = text.min_intrinsic_width("a wonderful day", key);
    let word = text.measure("wonderful", key, INF, None).size.width;
    assert!((min - word).abs() <= 0.5, "{min} vs {word}");
    assert_eq!(text.min_intrinsic_width("", key), 0.0);
}

#[test]
fn text_sys_08_odd_widths() {
    let mut text = system();
    let key = body(&mut text);
    let one_line = text.measure("one two three", key, INF, None);
    assert_eq!(one_line.line_count, 1);
    assert_eq!(text.measure("one two three", key, f32::NAN, None), one_line);
    assert_eq!(text.measure("one two three", key, 0.0, None).line_count, 3);
    assert_eq!(text.measure("one two three", key, -5.0, None).line_count, 3);
    for width in [INF, -INF, f32::NAN, 0.0, 1e-30, 1e30] {
        for max_lines in [None, Some(0), Some(1), Some(u32::MAX)] {
            let _ = text.measure("x y\nz", key, width, max_lines);
        }
    }
}

#[test]
fn text_sys_09_paint() {
    let mut text = system();
    let key = body(&mut text);
    let metrics = text.measure("Hello", key, INF, None);
    let mut resources = Resources::new();
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(200.0, 100.0));
    let color = Color::from_rgb8(10, 20, 30);
    text.paint(
        &mut b,
        &mut resources,
        "Hello",
        key,
        INF,
        None,
        color,
        Point::new(5.0, 7.0),
    );
    text.paint(
        &mut b,
        &mut resources,
        "Hello",
        key,
        INF,
        None,
        color,
        Point::new(5.0, 40.0),
    );
    text.paint(
        &mut b,
        &mut resources,
        "",
        key,
        INF,
        None,
        color,
        Point::ZERO,
    );
    b.finish().expect("balanced");
    let runs: Vec<_> = scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(run.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(runs.len(), 2);
    let glyphs = scene.glyphs(&runs[0]);
    assert_eq!(glyphs.len(), 5);
    assert_eq!(runs[0].color, color);
    assert_eq!(runs[0].font_size, 16.0);
    assert_eq!(runs[0].origin, Point::new(5.0, 7.0));
    // The first line's baseline sits at origin.y + first_baseline.
    assert_eq!(glyphs[0].y, metrics.first_baseline);
    assert_eq!(glyphs[0].x, 0.0);
    assert!(glyphs[1].x > 0.0);
    // The same font is registered once.
    assert_eq!(runs[0].font, runs[1].font);
    let font = resources.font(runs[0].font).expect("registered");
    assert_eq!(font.bytes().len(), FONT.len());
    // Lines cut by max_lines paint nothing.
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(200.0, 100.0));
    text.paint(
        &mut b,
        &mut resources,
        "ab\ncd",
        key,
        INF,
        Some(1),
        color,
        Point::ZERO,
    );
    b.finish().expect("balanced");
    let total: usize = scene
        .entries()
        .iter()
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(scene.glyphs(run).len()),
            _ => None,
        })
        .sum();
    assert_eq!(total, 2);
}
