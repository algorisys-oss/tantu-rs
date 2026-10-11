//! # Tantu: Theme
//!
//! Design tokens and the default themes. A [`Theme`] holds the color roles, text presets,
//! shape scale and density the widgets read; the runner (or a [`ThemeProvider`]) provides a
//! `Signal<Theme>` as an inherited value, so switching it repaints exactly the widgets that
//! read it. Spec: `docs/specs/theme/theme.md`, [ADR 0012](https://github.com/algorisys-oss/tantu-rs/blob/main/docs/adr/0012-theme-tokens-through-inherited-signals.md).
//!
//! ```
//! use tantu_theme::{Theme, ThemeProvider};
//! use tantu_view::reactive::signal;
//! use tantu_view::{Builder, ViewTree};
//! use tantu_view::layout::{BoxConstraints, RenderConstrainedBox};
//!
//! let tree = ViewTree::new(|| {
//!     let theme = signal(Theme::dark());
//!     ThemeProvider::new(
//!         theme,
//!         Builder::new(|cx| {
//!             assert_eq!(Theme::of(cx).colors.primary, Theme::dark().colors.primary);
//!             tantu_view::AnyView::new(Leaf)
//!         }),
//!     )
//! });
//! # struct Leaf;
//! # impl tantu_view::View for Leaf {
//! #     fn build(self, cx: &mut tantu_view::BuildCx<'_>) -> tantu_view::ElementId {
//! #         cx.render(RenderConstrainedBox::new(BoxConstraints::tight(tantu_view::core::Size::ZERO)), [])
//! #     }
//! # }
//! ```

#![forbid(unsafe_code)]

use tantu_view::core::Color;
use tantu_view::reactive::Signal;
use tantu_view::text::TextStyle;
use tantu_view::{BuildCx, ElementId, Provide, View};

/// Material 3 color roles (the subset the widgets use), as sRGB colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorScheme {
    /// Filled buttons, selected toggles, the slider's active track, focus and the caret.
    pub primary: Color,
    /// Text and marks on `primary`.
    pub on_primary: Color,
    /// The window background.
    pub surface: Color,
    /// Text and icons on `surface`; state layers and disabled parts at reduced alpha.
    pub on_surface: Color,
    /// Secondary text and unselected outlines.
    pub on_surface_variant: Color,
    /// Inactive tracks.
    pub surface_container_highest: Color,
    /// Outlines of fields and unselected switches.
    pub outline: Color,
    /// Dividers.
    pub outline_variant: Color,
    /// The tooltip background.
    pub inverse_surface: Color,
    /// Text on `inverse_surface`.
    pub inverse_on_surface: Color,
    /// Errors (reserved for forms).
    pub error: Color,
}

/// The three text presets the widgets use.
#[derive(Clone, Debug, PartialEq)]
pub struct TextTheme {
    /// Body text (`Text`'s default, text fields).
    pub body: TextStyle,
    /// Titles.
    pub title: TextStyle,
    /// Labels on controls (button labels).
    pub label: TextStyle,
}

/// Corner radii (Material 3's shape scale).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    /// Text fields, tooltips (4 px).
    pub extra_small: f32,
    /// 8 px.
    pub small: f32,
    /// 12 px.
    pub medium: f32,
    /// 16 px.
    pub large: f32,
}

/// How tight the controls are: 0 is Material's default, negative is denser (−1, −2, −3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Density(pub i8);

impl Density {
    /// Pixels added to a control's minimum extent: 4 per step, the step clamped to −3..=3
    /// (THEME-03).
    pub fn offset(self) -> f32 {
        todo!()
    }

    /// `extent` plus the offset, never below 0 (THEME-03).
    pub fn apply(self, extent: f32) -> f32 {
        let _ = extent;
        todo!()
    }
}

/// The tokens the widgets read: colors, text presets, shape and density.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// The color roles.
    pub colors: ColorScheme,
    /// The text presets.
    pub text: TextTheme,
    /// The shape scale.
    pub shape: Shape,
    /// The density.
    pub density: Density,
}

impl Theme {
    /// Material 3's baseline light scheme: the widgets' original constants (THEME-01).
    pub fn light() -> Self {
        todo!()
    }

    /// Material 3's baseline dark scheme (THEME-01).
    pub fn dark() -> Self {
        todo!()
    }

    /// The theme in effect while building: the provided signal's value (a tracked read), or
    /// [`Theme::light`] (THEME-02).
    pub fn of(cx: &BuildCx<'_>) -> Theme {
        let _ = cx;
        todo!()
    }

    /// The provided signal, if any: for reactive closures, and for apps switching the theme
    /// (THEME-02).
    pub fn signal(cx: &BuildCx<'_>) -> Option<Signal<Theme>> {
        let _ = cx;
        todo!()
    }
}

/// Provides `theme` to its child and everything under it (THEME-02).
pub struct ThemeProvider<V> {
    inner: Provide<Signal<Theme>, V>,
}

impl<V: View> ThemeProvider<V> {
    /// Provides `theme` to `child`.
    pub fn new(theme: Signal<Theme>, child: V) -> Self {
        ThemeProvider {
            inner: Provide::new(theme, child),
        }
    }
}

impl<V: View> View for ThemeProvider<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.inner.build(cx)
    }
}
