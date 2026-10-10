//! [`Text`] and [`Button`]. Spec: `docs/specs/widgets/basic.md`.

use std::rc::Rc;

use tantu_view::core::Color;
use tantu_view::text::TextStyle;
use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

/// Shows a string (a value, a closure or a signal) in a style and color (Flutter's `Text`).
pub struct Text {
    text: Prop<String>,
    style: TextStyle,
    color: Prop<Color>,
    max_lines: Option<u32>,
    soft_wrap: bool,
}

impl Text {
    /// `text` in `TextStyle::body()` and near-black, wrapping at the available width.
    pub fn new(text: impl IntoProp<String>) -> Self {
        Text {
            text: text.into_prop(),
            style: TextStyle::body(),
            color: Prop::Value(Color::from_argb32(0xFF1C_1B1F)),
            max_lines: None,
            soft_wrap: true,
        }
    }

    /// The text style.
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// The text color.
    pub fn color(mut self, color: impl IntoProp<Color>) -> Self {
        self.color = color.into_prop();
        self
    }

    /// Keep at most `max_lines` lines.
    pub fn max_lines(mut self, max_lines: u32) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Wrap at the available width (`true`, the default) or only at hard line breaks.
    pub fn soft_wrap(mut self, soft_wrap: bool) -> Self {
        self.soft_wrap = soft_wrap;
        self
    }
}

impl View for Text {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            self.text,
            self.style,
            self.color,
            self.max_lines,
            self.soft_wrap,
            cx,
        );
        todo!()
    }
}

/// A pressable label (Material 3's filled button, with a fixed look until `tantu-theme`).
pub struct Button {
    label: Prop<String>,
    on_press: Option<Rc<dyn Fn()>>,
    enabled: Prop<bool>,
}

impl Button {
    /// A button labelled `label`, enabled, doing nothing when pressed.
    pub fn new(label: impl IntoProp<String>) -> Self {
        Button {
            label: label.into_prop(),
            on_press: None,
            enabled: Prop::Value(true),
        }
    }

    /// Called when a primary-button press that started on the button is released over it.
    pub fn on_press(mut self, f: impl Fn() + 'static) -> Self {
        self.on_press = Some(Rc::new(f));
        self
    }

    /// A disabled button ignores the pointer and draws dimmed.
    pub fn enabled(mut self, enabled: impl IntoProp<bool>) -> Self {
        self.enabled = enabled.into_prop();
        self
    }
}

impl View for Button {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.label, self.on_press, self.enabled, cx);
        todo!()
    }
}
