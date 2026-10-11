//! Tests for `docs/specs/theme/theme.md`, rules THEME-01..03.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_theme::{Density, Shape, Theme, ThemeProvider};
use tantu_view::core::{Color, Size};
use tantu_view::layout::{BoxConstraints, RenderConstrainedBox};
use tantu_view::reactive::{Signal, signal};
use tantu_view::text::TextStyle;
use tantu_view::{BuildCx, Builder, ElementId, View, ViewTree};

/// A fixed-size render element.
struct Leaf;

impl View for Leaf {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(
            RenderConstrainedBox::new(BoxConstraints::tight(Size::new(10.0, 10.0))),
            [],
        )
    }
}

fn argb(value: u32) -> Color {
    Color::from_argb32(value)
}

#[test]
fn theme_01_presets() {
    let light = Theme::light();
    let c = light.colors;
    assert_eq!(c.primary, argb(0xFF67_50A4));
    assert_eq!(c.on_primary, Color::WHITE);
    assert_eq!(c.surface, Color::WHITE);
    assert_eq!(c.on_surface, argb(0xFF1C_1B1F));
    assert_eq!(c.on_surface_variant, argb(0xFF49_454F));
    assert_eq!(c.surface_container_highest, argb(0xFFE6_E0E9));
    assert_eq!(c.outline, argb(0xFF79_747E));
    assert_eq!(c.outline_variant, argb(0xFFCA_C4D0));
    assert_eq!(c.inverse_surface, argb(0xFF31_3033));
    assert_eq!(c.inverse_on_surface, argb(0xFFF4_EFF4));
    assert_eq!(c.error, argb(0xFFB3_261E));

    let dark = Theme::dark();
    let d = dark.colors;
    assert_eq!(d.primary, argb(0xFFD0_BCFF));
    assert_eq!(d.on_primary, argb(0xFF38_1E72));
    assert_eq!(d.surface, argb(0xFF14_1218));
    assert_eq!(d.on_surface, argb(0xFFE6_E0E9));
    assert_eq!(d.on_surface_variant, argb(0xFFCA_C4D0));
    assert_eq!(d.surface_container_highest, argb(0xFF36_343B));
    assert_eq!(d.outline, argb(0xFF93_8F99));
    assert_eq!(d.outline_variant, argb(0xFF49_454F));
    assert_eq!(d.inverse_surface, argb(0xFFE6_E0E9));
    assert_eq!(d.inverse_on_surface, argb(0xFF32_2F35));
    assert_eq!(d.error, argb(0xFFF2_B8B5));

    for theme in [&light, &dark] {
        assert_eq!(theme.text.body, TextStyle::body());
        assert_eq!(theme.text.title, TextStyle::title());
        assert_eq!(theme.text.label, TextStyle::label());
        assert_eq!(
            theme.shape,
            Shape {
                extra_small: 4.0,
                small: 8.0,
                medium: 12.0,
                large: 16.0
            }
        );
        assert_eq!(theme.density, Density(0));
    }
    // The two differ only in colors.
    assert_ne!(light, dark);
    assert_eq!(
        Theme {
            colors: dark.colors,
            ..light.clone()
        },
        dark
    );
}

/// What a build saw: `Theme::of` and `Theme::signal`.
type Seen = Rc<RefCell<Vec<(Theme, Option<Signal<Theme>>)>>>;

fn record(seen: &Seen, cx: &BuildCx<'_>) {
    seen.borrow_mut().push((Theme::of(cx), Theme::signal(cx)));
}

#[test]
fn theme_02_of_signal_and_provider() {
    let seen: Seen = Rc::default();
    let signals: Rc<RefCell<Vec<Signal<Theme>>>> = Rc::default();
    let tree = {
        let (seen, signals) = (seen.clone(), signals.clone());
        ViewTree::new(move || {
            Builder::new(move |cx| {
                // Nothing provided: the light theme, no signal.
                record(&seen, cx);
                let outer = signal(Theme::dark());
                signals.borrow_mut().push(outer);
                let (seen2, signals2) = (seen.clone(), signals.clone());
                ThemeProvider::new(
                    outer,
                    Builder::new(move |cx| {
                        record(&seen2, cx);
                        let inner = signal(Theme::light());
                        signals2.borrow_mut().push(inner);
                        let seen3 = seen2.clone();
                        ThemeProvider::new(
                            inner,
                            Builder::new(move |cx| {
                                record(&seen3, cx);
                                Leaf
                            }),
                        )
                    }),
                )
            })
        })
    };
    let seen = seen.borrow();
    let signals = signals.borrow();
    assert_eq!(seen.len(), 3);
    assert_eq!(seen[0].0, Theme::light());
    assert!(seen[0].1.is_none());
    assert_eq!(seen[1].0, Theme::dark());
    // The nested provider shadows the outer one.
    assert_eq!(seen[2].0, Theme::light());
    // The signals are the provided handles: writing one is seen through the other.
    tree.enter(|| {
        let outer = seen[1].1.expect("provided");
        signals[0].set(Theme::light());
        assert_eq!(outer.get(), Theme::light());
        let inner = seen[2].1.expect("provided");
        signals[1].set(Theme::dark());
        assert_eq!(inner.get(), Theme::dark());
    });
}

#[test]
fn theme_03_density() {
    assert_eq!(Density(0).offset(), 0.0);
    assert_eq!(Density(-1).offset(), -4.0);
    assert_eq!(Density(-5).offset(), -12.0);
    assert_eq!(Density(2).offset(), 8.0);
    assert_eq!(Density(9).offset(), 12.0);
    assert_eq!(Density(-1).apply(40.0), 36.0);
    assert_eq!(Density(1).apply(40.0), 44.0);
    assert_eq!(Density(-3).apply(10.0), 0.0);
}
