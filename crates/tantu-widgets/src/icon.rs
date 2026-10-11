//! [`IconData`], [`Icon`] and [`IconButton`]. Spec: `docs/specs/widgets/icon.md`.

use std::rc::Rc;

use tantu_view::core::Color;
use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

/// Material Icons Regular, as a TTF (Apache 2.0, see `fonts/material-icons/LICENSE`).
#[cfg(feature = "material-icons")]
pub const MATERIAL_ICONS_FONT: &[u8] =
    include_bytes!("../fonts/material-icons/MaterialIcons-Regular.ttf");

/// The Material Icons family name.
#[cfg(feature = "material-icons")]
pub const MATERIAL_ICONS_FAMILY: &str = "Material Icons";

/// A glyph of an icon font.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IconData {
    /// The glyph's code point.
    pub code_point: char,
    /// The font family holding it.
    pub family: &'static str,
    /// The font file to register before use, if the widget must bring it.
    pub font: Option<&'static [u8]>,
}

impl IconData {
    /// The glyph `code_point` of `family`, with no font to register.
    pub const fn new(code_point: char, family: &'static str) -> Self {
        IconData {
            code_point,
            family,
            font: None,
        }
    }

    /// The same icon, registering `font` before use.
    pub const fn with_font(self, font: &'static [u8]) -> Self {
        IconData {
            font: Some(font),
            ..self
        }
    }
}

impl IntoProp<IconData> for IconData {
    fn into_prop(self) -> Prop<IconData> {
        Prop::Value(self)
    }
}

/// Draws an icon glyph (Flutter's `Icon`).
pub struct Icon {
    icon: Prop<IconData>,
    size: f32,
    color: Prop<Color>,
}

impl Icon {
    /// `icon` at 24 px in on-surface (`0xFF1C1B1F`).
    pub fn new(icon: impl IntoProp<IconData>) -> Self {
        Icon {
            icon: icon.into_prop(),
            size: 24.0,
            color: Prop::Value(Color::from_argb32(0xFF1C_1B1F)),
        }
    }

    /// The icon's size, a square (default 24).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// The icon's color.
    pub fn color(mut self, color: impl IntoProp<Color>) -> Self {
        self.color = color.into_prop();
        self
    }
}

impl View for Icon {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.icon, self.size, self.color);
        todo!("WIDGETS-ICON-02")
    }
}

/// A pressable icon (Flutter's `IconButton`, Material 3 standard icon button).
pub struct IconButton {
    icon: Prop<IconData>,
    on_press: Option<Rc<dyn Fn()>>,
    enabled: Prop<bool>,
}

impl IconButton {
    /// An icon button, enabled, reporting nothing (so drawn disabled until `on_press` is
    /// set).
    pub fn new(icon: impl IntoProp<IconData>) -> Self {
        IconButton {
            icon: icon.into_prop(),
            on_press: None,
            enabled: Prop::Value(true),
        }
    }

    /// Called when the button is activated (a primary release inside it, or Space/Enter).
    pub fn on_press(mut self, f: impl Fn() + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }

    /// A disabled button ignores input and draws dimmed. No `on_press` disables it too.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }
}

impl View for IconButton {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.icon, self.on_press, self.enabled);
        todo!("WIDGETS-ICON-04")
    }
}
