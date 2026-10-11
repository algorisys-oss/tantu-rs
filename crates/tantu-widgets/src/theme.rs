//! How the widgets read the theme (WIDGETS-THEME-01..04): a handle to the provided
//! `Signal<Theme>` (or the light theme), read inside the reactive closures driving paints and
//! sizes, so a theme switch updates exactly the widgets that read it.

use tantu_theme::{ColorScheme, Density, Shape, Theme};
use tantu_view::core::Color;
use tantu_view::reactive::Signal;
use tantu_view::{BuildCx, Prop};

/// The theme signal in effect where a widget builds, or the light theme when none is provided.
#[derive(Clone, Copy)]
pub(crate) struct ThemeHandle(Option<Signal<Theme>>);

impl ThemeHandle {
    /// The theme provided above the element being built.
    pub(crate) fn of(cx: &BuildCx<'_>) -> Self {
        ThemeHandle(Theme::signal(cx))
    }

    /// The theme now, for build-time choices such as text presets (an untracked read).
    pub(crate) fn snapshot(&self) -> Theme {
        self.0.map_or_else(Theme::light, Signal::get_untracked)
    }

    /// The colors (a tracked read inside a reactive closure).
    pub(crate) fn colors(&self) -> ColorScheme {
        self.read(|t| t.colors)
    }

    /// The shape scale (a tracked read).
    pub(crate) fn shape(&self) -> Shape {
        self.read(|t| t.shape)
    }

    fn read<T>(&self, f: impl FnOnce(&Theme) -> T) -> T {
        match self.0 {
            Some(signal) => signal.with(f),
            None => f(&Theme::light()),
        }
    }

    /// A prop of the colors: dynamic when a theme is provided, a value otherwise.
    pub(crate) fn colors_prop(self) -> Prop<ColorScheme> {
        self.prop(|t| t.colors)
    }

    /// A prop of the density.
    pub(crate) fn density_prop(self) -> Prop<Density> {
        self.prop(|t| t.density)
    }

    /// A prop of one color role.
    pub(crate) fn color_prop(self, role: impl Fn(&ColorScheme) -> Color + 'static) -> Prop<Color> {
        self.prop(move |t| role(&t.colors))
    }

    fn prop<T: 'static>(self, f: impl Fn(&Theme) -> T + 'static) -> Prop<T> {
        match self.0 {
            Some(signal) => Prop::Dynamic(Box::new(move || signal.with(&f))),
            None => Prop::Value(f(&Theme::light())),
        }
    }
}

/// `color` at an 8-bit alpha (so `at(c, 0x61)` equals `Color::from_argb32(0x61……)`).
pub(crate) fn at(color: Color, alpha: u8) -> Color {
    color.with_alpha(f32::from(alpha) / 255.0)
}

/// Material's 38 % alpha (disabled content).
pub(crate) const ALPHA_38: u8 = 0x61;
/// Material's 12 % alpha (disabled containers).
pub(crate) const ALPHA_12: u8 = 0x1F;

/// `over` blended over `base` at `alpha` (source-over in sRGB): a state layer on a filled
/// control.
pub(crate) fn blend(base: Color, over: Color, alpha: f32) -> Color {
    let mix = |b: f32, o: f32| b * (1.0 - alpha) + o * alpha;
    Color {
        r: mix(base.r, over.r),
        g: mix(base.g, over.g),
        b: mix(base.b, over.b),
        a: base.a,
    }
}
