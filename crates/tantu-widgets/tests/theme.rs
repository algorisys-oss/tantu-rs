//! Tests for `docs/specs/theme/theme.md`, rules WIDGETS-THEME-01..04.

use std::cell::Cell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester};
use tantu_theme::{Density, Theme, ThemeProvider};
use tantu_view::core::{Color, Rect};
use tantu_view::layout::CrossAxisAlignment;
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::text::TextStyle;
use tantu_view::{Keyed, View};
use tantu_widgets::{
    Button, Checkbox, Column, Divider, IconButton, Icons, Radio, Slider, Switch, Text,
    TextEditingController, TextField, Tooltip,
};

/// The fill colors of the element keyed `key`, in paint order.
fn fills(tester: &WidgetTester, key: &str) -> Vec<Color> {
    let element = tester.find(&Finder::key(key));
    tester
        .frame()
        .entries_for(element)
        .filter_map(|e| match e.command {
            Command::Fill { color, .. } => Some(color),
            _ => None,
        })
        .collect()
}

/// The stroke colors of the element keyed `key`.
fn strokes(tester: &WidgetTester, key: &str) -> Vec<Color> {
    let element = tester.find(&Finder::key(key));
    tester
        .frame()
        .entries_for(element)
        .filter_map(|e| match e.command {
            Command::Stroke { color, .. } => Some(color),
            _ => None,
        })
        .collect()
}

/// The glyph-run colors painted inside the bounds of the element keyed `key`.
fn glyph_colors(tester: &WidgetTester, key: &str) -> Vec<Color> {
    let bounds = rect(tester, key);
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter(|e| {
            e.element.is_some_and(|id| {
                let r = tester.rect(id);
                r.left >= bounds.left
                    && r.top >= bounds.top
                    && r.right <= bounds.right
                    && r.bottom <= bounds.bottom
            })
        })
        .filter_map(|e| match &e.command {
            Command::GlyphRun(run) => Some(run.color),
            _ => None,
        })
        .collect()
}

/// Every fill color in the frame.
fn all_fills(tester: &WidgetTester) -> Vec<Color> {
    tester
        .frame()
        .scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::Fill { color, .. } => Some(color),
            _ => None,
        })
        .collect()
}

fn rect(tester: &WidgetTester, key: &str) -> Rect {
    tester.rect(tester.find(&Finder::key(key)))
}

/// Every widget, keyed, in a column.
fn widgets() -> impl View {
    Column::new()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .child(Keyed::new("button", Button::new("A").on_press(|| {})))
        .child(Keyed::new("text", Text::new("x")))
        .child(Keyed::new("divider", Divider::new()))
        .child(Keyed::new("switch", Switch::new(false).on_change(|_| {})))
        .child(Keyed::new(
            "checkbox",
            Checkbox::new(false).on_change(|_| {}),
        ))
        .child(Keyed::new("radio", Radio::new(1, 2).on_change(|_| {})))
        .child(Keyed::new(
            "field",
            TextField::new(TextEditingController::new("")),
        ))
        .child(Keyed::new("slider", Slider::new(0.5).on_change(|_| {})))
        .child(Keyed::new(
            "icon-button",
            IconButton::new(Icons::HOME).on_press(|| {}),
        ))
        .child(Tooltip::new(
            "tip",
            Keyed::new("tip-target", Button::new("B").on_press(|| {})),
        ))
}

#[test]
fn widgets_theme_01_colors_from_the_theme() {
    let mut tester = WidgetTester::new(|| ThemeProvider::new(signal(Theme::dark()), widgets()));
    let dark = Theme::dark().colors;
    assert_eq!(fills(&tester, "button")[0], dark.primary);
    assert_eq!(glyph_colors(&tester, "button"), vec![dark.on_primary]);
    assert_eq!(glyph_colors(&tester, "text"), vec![dark.on_surface]);
    assert_eq!(fills(&tester, "divider"), vec![dark.outline_variant]);
    assert_eq!(fills(&tester, "switch")[0], dark.surface_container_highest);
    assert_eq!(strokes(&tester, "switch"), vec![dark.outline]);
    assert_eq!(strokes(&tester, "checkbox"), vec![dark.on_surface_variant]);
    assert_eq!(strokes(&tester, "radio"), vec![dark.on_surface_variant]);
    assert_eq!(strokes(&tester, "field"), vec![dark.outline]);
    assert_eq!(
        fills(&tester, "slider")[..2],
        [dark.primary, dark.surface_container_highest]
    );
    assert_eq!(
        glyph_colors(&tester, "icon-button"),
        vec![dark.on_surface_variant]
    );
    tester.hover(&Finder::key("tip-target"));
    assert!(all_fills(&tester).contains(&dark.inverse_surface));
    let tip_text = tester.frame().scene.entries().iter().any(
        |e| matches!(&e.command, Command::GlyphRun(run) if run.color == dark.inverse_on_surface),
    );
    assert!(tip_text);
}

#[test]
fn widgets_theme_02_live_switch_is_scoped() {
    let red = Color::from_argb32(0xFFFF_0000);
    let theme_out: Rc<Cell<Option<Signal<Theme>>>> = Rc::default();
    let mut tester = {
        let theme_out = theme_out.clone();
        WidgetTester::new(move || {
            let theme = signal(Theme::light());
            theme_out.set(Some(theme));
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .child(Keyed::new("outside", Button::new("A").on_press(|| {})))
                .child(ThemeProvider::new(
                    theme,
                    Column::new()
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .child(Keyed::new("inside", Button::new("A").on_press(|| {})))
                        .child(Keyed::new("explicit", Text::new("x").color(red))),
                ))
        })
    };
    let theme = theme_out.get().expect("built");
    let (light, dark) = (Theme::light().colors, Theme::dark().colors);
    assert_eq!(fills(&tester, "inside")[0], light.primary);
    tester.enter(|| theme.set(Theme::dark()));
    assert!(tester.pump());
    assert_eq!(fills(&tester, "inside")[0], dark.primary);
    assert_eq!(fills(&tester, "outside")[0], light.primary);
    assert_eq!(glyph_colors(&tester, "explicit"), vec![red]);
    tester.enter(|| theme.set(Theme::light()));
    tester.pump();
    assert_eq!(fills(&tester, "inside")[0], light.primary);
}

#[test]
fn widgets_theme_03_density() {
    let theme_out: Rc<Cell<Option<Signal<Theme>>>> = Rc::default();
    let mut tester = {
        let theme_out = theme_out.clone();
        WidgetTester::new(move || {
            let theme = signal(Theme {
                density: Density(-1),
                ..Theme::light()
            });
            theme_out.set(Some(theme));
            ThemeProvider::new(theme, widgets())
        })
    };
    let theme = theme_out.get().expect("built");
    let size = |tester: &WidgetTester, key: &str| {
        let r = rect(tester, key);
        (r.width(), r.height())
    };
    assert_eq!(size(&tester, "button"), (64.0, 36.0));
    assert_eq!(size(&tester, "checkbox"), (36.0, 36.0));
    assert_eq!(size(&tester, "radio"), (36.0, 36.0));
    assert_eq!(size(&tester, "icon-button"), (36.0, 36.0));
    assert_eq!(size(&tester, "switch"), (56.0, 36.0));
    assert_eq!(size(&tester, "field").1, 36.0);
    assert_eq!(size(&tester, "slider").1, 48.0);

    tester.enter(|| {
        theme.set(Theme {
            density: Density(-2),
            ..Theme::light()
        })
    });
    tester.pump();
    assert_eq!(size(&tester, "button"), (64.0, 32.0));
    assert_eq!(size(&tester, "checkbox"), (32.0, 32.0));
    assert_eq!(size(&tester, "switch"), (52.0, 32.0));
    let field = size(&tester, "field").1;
    assert!((32.0..36.0).contains(&field), "{field}");

    tester.enter(|| theme.set(Theme::light()));
    tester.pump();
    assert_eq!(size(&tester, "button"), (64.0, 40.0));
    assert_eq!(size(&tester, "checkbox"), (40.0, 40.0));
}

#[test]
fn widgets_theme_04_text_presets() {
    let big = TextStyle {
        size: 20.0,
        ..TextStyle::body()
    };
    let tester = {
        let big = big.clone();
        WidgetTester::new(move || {
            let mut theme = Theme::light();
            theme.text.body = big.clone();
            theme.text.label = TextStyle {
                size: 30.0,
                ..TextStyle::label()
            };
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .child(Keyed::new("plain", Text::new("Hello")))
                .child(ThemeProvider::new(
                    signal(theme),
                    Column::new()
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .child(Keyed::new("themed", Text::new("Hello")))
                        .child(Keyed::new("explicit", Text::new("Hello").style(big)))
                        .child(Keyed::new("button", Button::new("A").on_press(|| {}))),
                ))
        })
    };
    let themed = rect(&tester, "themed");
    assert_eq!(themed.height(), rect(&tester, "explicit").height());
    assert!(themed.height() > rect(&tester, "plain").height());
    // The button's label is the theme's label preset: 30 px text makes it taller than 40.
    assert!(rect(&tester, "button").height() > 40.0);
}
