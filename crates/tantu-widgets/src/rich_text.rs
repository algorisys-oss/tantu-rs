//! [`TextSpan`] and [`RichText`]. Spec: `docs/specs/widgets/rich-text.md`.

use tantu_view::core::Color;
use tantu_view::text::{FontFamily, TextStyle};
use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

/// `RichText`'s default color (Material 3's on-surface).
const ON_SURFACE: Color = Color::from_argb32(0xFF1C_1B1F);

/// A run of text with a style, or overrides of its parent's, and children (Flutter's
/// `TextSpan`).
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    text: String,
    style: Option<TextStyle>,
    size: Option<f32>,
    weight: Option<f32>,
    italic: Option<bool>,
    family: Option<FontFamily>,
    color: Option<Color>,
    children: Vec<TextSpan>,
}

impl TextSpan {
    /// A span of `text` inheriting everything from its parent.
    pub fn new(text: impl Into<String>) -> Self {
        TextSpan {
            text: text.into(),
            style: None,
            size: None,
            weight: None,
            italic: None,
            family: None,
            color: None,
            children: Vec::new(),
        }
    }

    /// A span with no text of its own, only children.
    pub fn group() -> Self {
        TextSpan::new("")
    }

    /// A full style, replacing the inherited one (the overrides below still apply on top).
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// The font size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    /// The font weight (100 to 900).
    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Weight 700.
    pub fn bold(self) -> Self {
        self.weight(700.0)
    }

    /// Italic or upright.
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = Some(italic);
        self
    }

    /// The font family.
    pub fn family(mut self, family: FontFamily) -> Self {
        self.family = Some(family);
        self
    }

    /// The text color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Appends a child span.
    pub fn child(mut self, child: TextSpan) -> Self {
        self.children.push(child);
        self
    }

    /// Appends child spans.
    pub fn children(mut self, children: impl IntoIterator<Item = TextSpan>) -> Self {
        self.children.extend(children);
        self
    }

    /// The concatenated text of the span and its children, in order.
    pub fn to_plain_text(&self) -> String {
        todo!("WIDGETS-RICHTEXT-01")
    }
}

impl IntoProp<TextSpan> for TextSpan {
    fn into_prop(self) -> Prop<TextSpan> {
        Prop::Value(self)
    }
}

/// A paragraph of styled spans (Flutter's `RichText`).
pub struct RichText {
    span: Prop<TextSpan>,
    style: TextStyle,
    color: Prop<Color>,
    max_lines: Option<u32>,
    soft_wrap: bool,
}

impl RichText {
    /// `span` on `TextStyle::body()` in on-surface, wrapping at the available width.
    pub fn new(span: impl IntoProp<TextSpan>) -> Self {
        RichText {
            span: span.into_prop(),
            style: TextStyle::body(),
            color: Prop::Value(ON_SURFACE),
            max_lines: None,
            soft_wrap: true,
        }
    }

    /// The base style, which the root span inherits.
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// The base color.
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

impl View for RichText {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.span,
            self.style,
            self.color,
            self.max_lines,
            self.soft_wrap,
        );
        todo!("WIDGETS-RICHTEXT-02")
    }
}
