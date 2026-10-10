//! Tests for `docs/specs/text/scripts.md`, rules TEXT-SCRIPT-01..04.

use tantu_core::{Color, Point, Size};
use tantu_layout::{TextMeasure, TextStyleKey};
use tantu_scene::{Command, Resources, Scene};
use tantu_text::{FontFamily, TextStyle, TextSystem};

const LIBERATION: &[u8] = include_bytes!("fonts/LiberationSans-Regular.ttf");
const NOTO_HEBREW: &[u8] = include_bytes!("fonts/NotoSansHebrew-Regular.ttf");

/// One drawn glyph: id, absolute x, and the size of the font file it came from.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Drawn {
    id: u32,
    x: f32,
    font_len: usize,
}

fn draw(text: &mut TextSystem, s: &str, style: TextStyleKey) -> Vec<Drawn> {
    let mut resources = Resources::new();
    let mut scene = Scene::new();
    let mut b = scene.begin(Size::new(1000.0, 200.0));
    text.paint(
        &mut b,
        &mut resources,
        s,
        style,
        f32::INFINITY,
        None,
        Color::BLACK,
        Point::ZERO,
    );
    b.finish().expect("balanced");
    let mut out = Vec::new();
    for entry in scene.entries() {
        if let Command::GlyphRun(run) = &entry.command {
            let font_len = resources.font(run.font).map_or(0, |f| f.bytes().len());
            for g in scene.glyphs(run) {
                out.push(Drawn {
                    id: g.id,
                    x: run.origin.x + g.x,
                    font_len,
                });
            }
        }
    }
    out
}

fn ids(drawn: &[Drawn]) -> Vec<u32> {
    drawn.iter().map(|d| d.id).collect()
}

/// Liberation Sans only, as the default family.
fn liberation() -> (TextSystem, TextStyleKey) {
    let mut text = TextSystem::without_system_fonts();
    text.register_font(LIBERATION.to_vec());
    text.set_default_family(FontFamily::Named("Liberation Sans".into()));
    let style = text.style(TextStyle::default());
    (text, style)
}

#[test]
fn text_script_01_rtl_in_visual_order() {
    let (mut text, style) = liberation();
    let word = draw(&mut text, "שלום", style);
    assert_eq!(word.len(), 4);
    assert!(word.windows(2).all(|w| w[0].x < w[1].x), "{word:?}");
    let last_letter = draw(&mut text, "ם", style);
    assert_eq!(word[0].id, last_letter[0].id);
    assert!(word.iter().all(|d| d.id != 0));
}

#[test]
fn text_script_02_mixed_paragraph() {
    let (mut text, style) = liberation();
    let mixed = draw(&mut text, "ab שלום cd", style);
    assert!(mixed.windows(2).all(|w| w[0].x <= w[1].x), "{mixed:?}");
    let all = ids(&mixed);
    assert_eq!(&all[..2], &ids(&draw(&mut text, "ab", style))[..]);
    assert_eq!(
        &all[all.len() - 2..],
        &ids(&draw(&mut text, "cd", style))[..]
    );
    let hebrew = ids(&draw(&mut text, "שלום", style));
    assert!(all.windows(4).any(|w| w == hebrew.as_slice()), "{all:?}");
    // The width is the runs' advances: "ab", "שלום", "cd" and two spaces.
    let width =
        |text: &mut TextSystem, s: &str| text.measure(s, style, f32::INFINITY, None).size.width;
    let space = width(&mut text, "a b") - width(&mut text, "ab");
    let expected =
        width(&mut text, "ab") + width(&mut text, "שלום") + width(&mut text, "cd") + 2.0 * space;
    let measured = width(&mut text, "ab שלום cd");
    assert!(
        (measured - expected).abs() < 0.5,
        "{measured} vs {expected}"
    );
}

/// Liberation Sans registered first, then Noto Sans Hebrew as the default family.
fn hebrew_default() -> (TextSystem, TextStyleKey) {
    let mut text = TextSystem::without_system_fonts();
    text.register_font(LIBERATION.to_vec());
    text.register_font(NOTO_HEBREW.to_vec());
    text.set_default_family(FontFamily::Named("Noto Sans Hebrew".into()));
    let style = text.style(TextStyle::default());
    (text, style)
}

#[test]
fn text_script_03_fallback_through_registered_families() {
    let (mut text, style) = hebrew_default();
    let latin = draw(&mut text, "abc", style);
    let (mut reference, ref_style) = liberation();
    assert_eq!(ids(&latin), ids(&draw(&mut reference, "abc", ref_style)));
    assert!(
        latin.iter().all(|d| d.font_len == LIBERATION.len()),
        "{latin:?}"
    );
    let hebrew = draw(&mut text, "שלום", style);
    assert!(
        hebrew
            .iter()
            .all(|d| d.font_len == NOTO_HEBREW.len() && d.id != 0)
    );
    let mixed = draw(&mut text, "ab שלום", style);
    let fonts: std::collections::BTreeSet<usize> = mixed.iter().map(|d| d.font_len).collect();
    assert_eq!(fonts.len(), 2, "{mixed:?}");
    assert!(mixed.iter().all(|d| d.id != 0));
}

#[test]
fn text_script_04_missing_everywhere_is_still_measured() {
    let (mut text, style) = liberation();
    let drawn = draw(&mut text, "م", style);
    assert_eq!(ids(&drawn), [0]);
    assert!(text.measure("م", style, f32::INFINITY, None).size.width > 0.0);
}
