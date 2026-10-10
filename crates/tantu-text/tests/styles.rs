//! Tests for `docs/specs/text/styles.md`, rules TEXT-STYLES-01..04.

use tantu_layout::{TextMeasure, TextStyleKey};
use tantu_text::{FontFamily, TextStyle, TextStyles, TextSystem};

const FONT: &[u8] = include_bytes!("fonts/LiberationSans-Regular.ttf");

fn sized(size: f32) -> TextStyle {
    TextStyle {
        size,
        ..TextStyle::default()
    }
}

#[test]
fn text_styles_01_keys() {
    let styles = TextStyles::new();
    assert!(styles.is_empty());
    let a = styles.key(sized(20.0));
    let b = styles.key(sized(20.0));
    let c = styles.key(sized(12.0));
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(styles.len(), 2);
    assert_eq!(styles.get(c).map(|s| s.size), Some(12.0));
    // Sanitized: a NaN size is the default size, so it is the same style as 16 px.
    let odd = styles.key(sized(f32::NAN));
    assert_eq!(odd, styles.key(sized(16.0)));
    assert_eq!(styles.get(odd).map(|s| s.size), Some(16.0));
    assert_eq!(styles.get(TextStyleKey(9_999)), None);
}

#[test]
fn text_styles_02_clones_share() {
    let styles = TextStyles::new();
    let other = styles.clone();
    let key = other.key(sized(30.0));
    assert_eq!(styles.get(key).map(|s| s.size), Some(30.0));
    assert_eq!(styles.key(sized(30.0)), key);
    assert_eq!((styles.len(), other.len()), (1, 1));
}

#[test]
fn text_styles_03_system_uses_the_table() {
    let styles = TextStyles::new();
    let before = styles.key(sized(24.0));
    let mut text = TextSystem::without_system_fonts().with_styles(styles.clone());
    text.register_font(FONT.to_vec());
    text.set_default_family(FontFamily::Named("Liberation Sans".into()));
    let after = styles.key(sized(10.0));
    for (key, size) in [(before, 24.0), (after, 10.0)] {
        let own = text.style(sized(size));
        assert_eq!(own, key);
        let m = text.measure("Hello", key, f32::INFINITY, None);
        assert!(m.size.width > 0.0);
        assert_eq!(m, text.measure("Hello", own, f32::INFINITY, None));
    }
    assert_eq!(text.styles().len(), styles.len());
    text.styles().key(sized(99.0));
    assert_eq!(styles.len(), 3);
}

#[test]
fn text_styles_04_presets() {
    for (style, size, weight) in [
        (TextStyle::body(), 14.0, 400.0),
        (TextStyle::title(), 22.0, 400.0),
        (TextStyle::label(), 14.0, 500.0),
    ] {
        assert_eq!(style.family, FontFamily::SansSerif);
        assert_eq!((style.size, style.weight), (size, weight));
        assert!(!style.italic);
        assert_eq!(style.line_height, None);
    }
}
