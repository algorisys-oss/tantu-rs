//! [`TextSpan`] and [`RichText`]. Spec: `docs/specs/widgets/rich-text.md`.

use std::sync::Arc;

use tantu_view::core::Color;
use tantu_view::layout::{RenderParagraph, StyleSpan, TextStyleKey};
use tantu_view::text::{FontFamily, TextStyle, TextStyles};
use tantu_view::{BuildCx, ElementId, IntoProp, ParagraphPaint, Prop, Role, Semantics, View};

use crate::theme::ThemeHandle;

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
        let mut text = String::new();
        self.collect_text(&mut text);
        text
    }

    fn collect_text(&self, out: &mut String) {
        out.push_str(&self.text);
        for child in &self.children {
            child.collect_text(out);
        }
    }

    /// This span's effective style: `inherited`, replaced by `style`, with the overrides on
    /// top (WIDGETS-RICHTEXT-01).
    fn effective_style(&self, inherited: &TextStyle) -> TextStyle {
        let mut style = self.style.clone().unwrap_or_else(|| inherited.clone());
        if let Some(size) = self.size {
            style.size = size;
        }
        if let Some(weight) = self.weight {
            style.weight = weight;
        }
        if let Some(italic) = self.italic {
            style.italic = italic;
        }
        if let Some(family) = &self.family {
            style.family = family.clone();
        }
        style
    }

    /// Flattens the tree into `text` and `spans` (WIDGETS-RICHTEXT-01).
    fn flatten_into(
        &self,
        styles: &TextStyles,
        base_key: TextStyleKey,
        inherited: &TextStyle,
        inherited_color: Option<Color>,
        text: &mut String,
        spans: &mut Vec<StyleSpan>,
    ) {
        let style = self.effective_style(inherited);
        let color = self.color.or(inherited_color);
        if !self.text.is_empty() {
            let key = styles.key(style.clone());
            let start = text.len();
            text.push_str(&self.text);
            if key != base_key || color.is_some() {
                spans.push(StyleSpan {
                    start,
                    end: text.len(),
                    style: key,
                    color,
                });
            }
        }
        for child in &self.children {
            child.flatten_into(styles, base_key, &style, color, text, spans);
        }
    }

    /// The paragraph's text and style spans for a base style.
    fn flatten(&self, styles: &TextStyles, base: &TextStyle) -> (String, Vec<StyleSpan>) {
        let base_key = styles.key(base.clone());
        let (mut text, mut spans) = (String::new(), Vec::new());
        self.flatten_into(styles, base_key, base, None, &mut text, &mut spans);
        (text, spans)
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
    style: Option<TextStyle>,
    color: Option<Prop<Color>>,
    max_lines: Option<u32>,
    soft_wrap: bool,
}

impl RichText {
    /// `span` on the theme's body style in on-surface, wrapping at the available width.
    pub fn new(span: impl IntoProp<TextSpan>) -> Self {
        RichText {
            span: span.into_prop(),
            style: None,
            color: None,
            max_lines: None,
            soft_wrap: true,
        }
    }

    /// The base style, which the root span inherits (default: the theme's `text.body`).
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// The base color (default: the theme's `on_surface`).
    pub fn color(mut self, color: impl IntoProp<Color>) -> Self {
        self.color = Some(color.into_prop());
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
        let theme = ThemeHandle::of(cx);
        let styles = cx.text_styles();
        let base = self.style.unwrap_or_else(|| theme.snapshot().text.body);
        let color = self
            .color
            .unwrap_or_else(|| theme.color_prop(|c| c.on_surface));
        let base_key = styles.key(base.clone());
        let mut paragraph = RenderParagraph::new("", base_key);
        paragraph.max_lines = self.max_lines;
        paragraph.soft_wrap = self.soft_wrap;
        let id = cx.render(paragraph, []);
        cx.set_paint(
            id,
            ParagraphPaint {
                color: Color::TRANSPARENT,
            },
        );
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Label,
                ..Default::default()
            },
        );
        cx.bind(id, self.span, move |e, span: TextSpan| {
            let (text, spans) = span.flatten(&styles, &base);
            e.update_semantics(|s| s.name = Some(text.clone()));
            e.update_render::<RenderParagraph>(|p| {
                p.text = Arc::from(text);
                p.spans = Arc::from(spans);
            });
        });
        cx.bind(id, color, |e, color| {
            e.update_paint::<ParagraphPaint>(|p| p.color = color);
        });
        id
    }
}
