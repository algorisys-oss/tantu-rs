//! Tests for `docs/specs/widgets/icon.md`, rules WIDGETS-ICON-01..04.

use std::cell::Cell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_view::core::{Color, Rect, Size};
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::{Command, GlyphRun};
use tantu_view::{Keyed, LogicalKey, Modifiers, View};
use tantu_widgets::{
    Column, Icon, IconButton, IconData, Icons, MATERIAL_ICONS_FAMILY, MATERIAL_ICONS_FONT, Row,
};

const ON_SURFACE: Color = Color::from_argb32(0xFF1C_1B1F);
const ON_SURFACE_VARIANT: Color = Color::from_argb32(0xFF49_454F);
const ON_SURFACE_38: Color = Color::from_argb32(0x611C_1B1F);
const PRIMARY: Color = Color::from_argb32(0xFF67_50A4);
const CODEPOINTS: &str = include_str!("../fonts/material-icons/MaterialIcons-Regular.codepoints");

/// The glyph run the element keyed `key` painted, with its glyph ids.
fn glyph_run(tester: &WidgetTester, key: &str) -> Option<(GlyphRun, Vec<u32>)> {
    let element = tester.find(&Finder::key(key));
    let frame = tester.frame();
    frame.entries_for(element).find_map(|e| match &e.command {
        Command::GlyphRun(run) => Some((
            run.clone(),
            frame.scene.glyphs(run).iter().map(|g| g.id).collect(),
        )),
        _ => None,
    })
}

fn window(view: impl View + 'static) -> WidgetTester {
    let view = Cell::new(Some(view));
    WidgetTester::with_size(300.0, 200.0, move || {
        Column::new().child(view.take().expect("built once"))
    })
}

#[test]
fn widgets_icon_01_icon_data_and_the_table() {
    assert_eq!(
        Icons::CHECK,
        IconData::new('\u{e5ca}', MATERIAL_ICONS_FAMILY).with_font(MATERIAL_ICONS_FONT)
    );
    assert_eq!(IconData::new('a', "X").font, None);
    assert_eq!(MATERIAL_ICONS_FAMILY, "Material Icons");
    assert_eq!(&MATERIAL_ICONS_FONT[..4], &[0, 1, 0, 0]);
    // Names and code points come from the codepoints file; the first duplicate wins.
    let first = |name: &str| {
        CODEPOINTS
            .lines()
            .find_map(|l| l.strip_prefix(name).and_then(|rest| rest.strip_prefix(' ')))
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .and_then(char::from_u32)
            .expect("in the file")
    };
    assert_eq!(Icons::N3D_ROTATION.code_point, first("3d_rotation"));
    assert_eq!(Icons::FLOURESCENT.code_point, first("flourescent"));
    assert_eq!(Icons::ARROW_BACK.code_point, first("arrow_back"));
    assert_eq!(Icons::ZOOM_OUT_MAP.code_point, first("zoom_out_map"));
}

#[test]
fn widgets_icon_02_paints_a_glyph() {
    let tester = window(Keyed::new("i", Icon::new(Icons::CHECK)));
    assert_eq!(
        tester.rect(tester.find(&Finder::key("i"))).size(),
        Size::new(24.0, 24.0)
    );
    let (run, glyphs) = glyph_run(&tester, "i").expect("a glyph run");
    assert_eq!((run.font_size, run.color), (24.0, ON_SURFACE));
    assert_eq!(glyphs.len(), 1);
    assert_ne!(glyphs[0], 0, "the icon font's glyph, not the missing glyph");
    let tester = window(Keyed::new(
        "i",
        Icon::new(Icons::HOME).size(48.0).color(PRIMARY),
    ));
    assert_eq!(
        tester.rect(tester.find(&Finder::key("i"))).size(),
        Size::new(48.0, 48.0)
    );
    let (run, glyphs) = glyph_run(&tester, "i").expect("a glyph run");
    assert_eq!((run.font_size, run.color), (48.0, PRIMARY));
    assert_eq!(glyphs.len(), 1);
    let mut tester = WidgetTester::with_size(120.0, 48.0, || {
        Row::new()
            .child(Icon::new(Icons::CHECK))
            .child(Icon::new(Icons::HOME).size(32.0).color(PRIMARY))
            .child(Icon::new(Icons::SETTINGS).size(48.0))
    });
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/icon.png"
    ));
}

#[test]
fn widgets_icon_03_reactive() {
    type Signals = (Signal<IconData>, Signal<Color>);
    let out: Rc<Cell<Option<Signals>>> = Rc::default();
    let mut tester = {
        let out = out.clone();
        WidgetTester::with_size(300.0, 200.0, move || {
            let (icon, color) = (signal(Icons::CHECK), signal(ON_SURFACE));
            out.set(Some((icon, color)));
            Column::new().child(Keyed::new("i", Icon::new(icon).color(color)))
        })
    };
    let (icon, color) = out.get().expect("built");
    let (_, before) = glyph_run(&tester, "i").expect("a glyph run");
    tester.enter(|| icon.set(Icons::HOME));
    assert!(tester.pump());
    let (run, after) = glyph_run(&tester, "i").expect("a glyph run");
    assert_ne!(before, after);
    assert_eq!(run.color, ON_SURFACE);
    tester.enter(|| color.set(PRIMARY));
    assert!(tester.pump());
    let (run, same) = glyph_run(&tester, "i").expect("a glyph run");
    assert_eq!((run.color, same), (PRIMARY, after));
    assert!(!tester.pump());
}

#[test]
fn widgets_icon_04_icon_button() {
    let presses = Rc::new(Cell::new(0));
    let count = presses.clone();
    let mut tester = window(Keyed::new(
        "b",
        IconButton::new(Icons::CHECK).on_press(move || count.set(count.get() + 1)),
    ));
    let rect = tester.rect(tester.find(&Finder::key("b")));
    assert_eq!(rect.size(), Size::new(40.0, 40.0));
    // The icon is centered: the 24 px glyph run sits 8 px in.
    let button = tester.find(&Finder::key("b"));
    let icon = tester.find_all(&Finder::render::<tantu_view::layout::RenderConstrainedBox>());
    let icon = *icon
        .iter()
        .find(|id| **id != button)
        .expect("the icon element");
    assert_eq!(
        tester.rect(icon),
        Rect::from_ltrb(rect.left + 8.0, 8.0, rect.left + 32.0, 32.0)
    );
    let run = tester
        .frame()
        .entries_for(icon)
        .find_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(run.clone()),
            _ => None,
        })
        .expect("the icon's glyph run");
    assert_eq!((run.font_size, run.color), (24.0, ON_SURFACE_VARIANT));
    tester.tap(&Finder::key("b"));
    assert_eq!(presses.get(), 1);
    tester.focus(&Finder::key("b"));
    tester.press_key(LogicalKey::Character(" ".into()), Modifiers::default());
    assert_eq!(presses.get(), 2);
    // The state layer (hover from the tap, focus): a 40 px circle in on-surface at 10 %.
    let layer = tester
        .frame()
        .entries_for(button)
        .find_map(|e| match e.command {
            Command::Fill { shape, color } if shape.rect.width() == 40.0 => Some(color),
            _ => None,
        });
    assert_eq!(layer, Some(ON_SURFACE.with_alpha(0.10)));
    // Disabled: no `on_press`, or `enabled(false)`.
    for button in [
        IconButton::new(Icons::CHECK),
        IconButton::new(Icons::CHECK).enabled(false).on_press(|| {}),
    ] {
        let mut tester = window(Keyed::new("b", button));
        let b = tester.find(&Finder::key("b"));
        let icon = tester.find_all(&Finder::render::<tantu_view::layout::RenderConstrainedBox>());
        let icon = *icon.iter().find(|id| **id != b).expect("the icon element");
        let color = tester
            .frame()
            .entries_for(icon)
            .find_map(|e| match &e.command {
                Command::GlyphRun(run) => Some(run.color),
                _ => None,
            });
        assert_eq!(color, Some(ON_SURFACE_38));
        tester.hover(&Finder::key("b"));
        tester.tap(&Finder::key("b"));
        assert!(
            tester
                .frame()
                .entries_for(b)
                .all(|e| !matches!(e.command, Command::Fill { .. }))
        );
    }
    let mut tester = WidgetTester::with_size(80.0, 40.0, || {
        Row::new()
            .child(IconButton::new(Icons::CHECK).on_press(|| {}))
            .child(IconButton::new(Icons::CHECK))
    });
    tester.matches_golden(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/goldens/icon-button.png"
    ));
}
