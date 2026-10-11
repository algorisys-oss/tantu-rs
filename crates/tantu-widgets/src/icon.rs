//! [`IconData`], [`Icon`] and [`IconButton`]. Spec: `docs/specs/widgets/icon.md`.

use std::rc::Rc;

use tantu_view::core::{Color, Point, Size};
use tantu_view::layout::{BoxConstraints, RenderConstrainedBox, TextStyleKey};
use tantu_view::text::{FontFamily, TextStyle};
use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Paint, PaintCx, Prop, View};

use crate::Center;
use crate::toggles::{
    ON_SURFACE_38, ON_SURFACE_VARIANT, State, TogglePaint, build_toggle, enabled_fn,
};
use crate::util::dynamic;

/// The icon button's element (Material 3's standard icon button, desktop tap target).
const ICON_BUTTON_SIZE: Size = Size::new(40.0, 40.0);

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
        let icon = dynamic(self.icon);
        // The family and its font come from the first value (WIDGETS-ICON-03).
        let first = icon();
        if let Some(font) = first.font {
            cx.ensure_font(font);
        }
        let style = cx.text_style(TextStyle {
            family: FontFamily::Named(first.family.into()),
            size: self.size,
            line_height: Some(1.0),
            ..TextStyle::default()
        });
        let size = Size::new(self.size, self.size);
        let id = cx.render(RenderConstrainedBox::new(BoxConstraints::tight(size)), []);
        cx.set_paint(
            id,
            IconPaint {
                text: String::new(),
                style,
                color: Color::TRANSPARENT,
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || icon().code_point)),
            |e, code_point| {
                e.update_paint::<IconPaint>(|p| p.text = code_point.to_string());
            },
        );
        cx.bind(id, self.color, |e, color| {
            e.update_paint::<IconPaint>(|p| p.color = color);
        });
        id
    }
}

/// Paints the icon's code point as a one-line paragraph at the origin (WIDGETS-ICON-02).
struct IconPaint {
    text: String,
    style: TextStyleKey,
    color: Color,
}

impl Paint for IconPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        cx.paint_text(
            &self.text,
            self.style,
            f32::INFINITY,
            Some(1),
            self.color,
            Point::ZERO,
        );
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
        let enabled = enabled_fn(self.enabled, self.on_press.is_some());
        let activate: Rc<dyn Fn()> = {
            let on_press = self.on_press;
            Rc::new(move || {
                if let Some(on_press) = &on_press {
                    on_press();
                }
            })
        };
        let color = {
            let enabled = Rc::clone(&enabled);
            move || {
                if enabled() {
                    ON_SURFACE_VARIANT
                } else {
                    ON_SURFACE_38
                }
            }
        };
        let icon = Center::new().child(Icon::new(self.icon).color(color));
        build_toggle(
            cx,
            ICON_BUTTON_SIZE,
            enabled,
            activate,
            IconButtonPaint {
                state: State::default(),
            },
            [AnyView::new(icon)],
        )
    }
}

/// Paints the icon button's state layer (WIDGETS-ICON-04); the icon is a child.
struct IconButtonPaint {
    state: State,
}

impl TogglePaint for IconButtonPaint {
    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

impl Paint for IconButtonPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        self.state.paint_layer(cx.scene(), center, false);
    }
}
