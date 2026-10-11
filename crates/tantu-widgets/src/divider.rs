//! [`Divider`] and [`VerticalDivider`]. Spec: `docs/specs/widgets/divider.md`.

use tantu_view::core::Color;
use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

/// The default divider color (Material 3's outline-variant).
const OUTLINE_VARIANT: Color = Color::from_argb32(0xFFCA_C4D0);

/// What both dividers share: the line's thickness, indents and color.
struct Line {
    thickness: Prop<f32>,
    indent: Prop<f32>,
    end_indent: Prop<f32>,
    color: Prop<Color>,
}

impl Line {
    fn new() -> Self {
        Line {
            thickness: Prop::Value(1.0),
            indent: Prop::Value(0.0),
            end_indent: Prop::Value(0.0),
            color: Prop::Value(OUTLINE_VARIANT),
        }
    }
}

/// A horizontal line separating content (Flutter's `Divider`, Material 3 look).
pub struct Divider {
    height: Prop<f32>,
    line: Line,
}

impl Divider {
    /// 16 px of height with a 1 px line in `0xFFCAC4D0` across the full width.
    pub fn new() -> Self {
        Divider {
            height: Prop::Value(16.0),
            line: Line::new(),
        }
    }

    /// The height the widget takes; the line is centered in it (default 16).
    pub fn height(mut self, height: impl IntoProp<f32>) -> Self {
        self.height = height.into_prop();
        self
    }

    /// The line's thickness (default 1).
    pub fn thickness(mut self, thickness: impl IntoProp<f32>) -> Self {
        self.line.thickness = thickness.into_prop();
        self
    }

    /// Space before the line's start (default 0).
    pub fn indent(mut self, indent: impl IntoProp<f32>) -> Self {
        self.line.indent = indent.into_prop();
        self
    }

    /// Space after the line's end (default 0).
    pub fn end_indent(mut self, end_indent: impl IntoProp<f32>) -> Self {
        self.line.end_indent = end_indent.into_prop();
        self
    }

    /// The line's color.
    pub fn color(mut self, color: impl IntoProp<Color>) -> Self {
        self.line.color = color.into_prop();
        self
    }
}

impl Default for Divider {
    fn default() -> Self {
        Divider::new()
    }
}

impl View for Divider {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.height, self.line.thickness);
        todo!("WIDGETS-DIVIDER-01")
    }
}

/// A vertical line separating content in a row (Flutter's `VerticalDivider`).
pub struct VerticalDivider {
    width: Prop<f32>,
    line: Line,
}

impl VerticalDivider {
    /// 16 px of width with a 1 px line in `0xFFCAC4D0` across the full height.
    pub fn new() -> Self {
        VerticalDivider {
            width: Prop::Value(16.0),
            line: Line::new(),
        }
    }

    /// The width the widget takes; the line is centered in it (default 16).
    pub fn width(mut self, width: impl IntoProp<f32>) -> Self {
        self.width = width.into_prop();
        self
    }

    /// The line's thickness (default 1).
    pub fn thickness(mut self, thickness: impl IntoProp<f32>) -> Self {
        self.line.thickness = thickness.into_prop();
        self
    }

    /// Space above the line's start (default 0).
    pub fn indent(mut self, indent: impl IntoProp<f32>) -> Self {
        self.line.indent = indent.into_prop();
        self
    }

    /// Space below the line's end (default 0).
    pub fn end_indent(mut self, end_indent: impl IntoProp<f32>) -> Self {
        self.line.end_indent = end_indent.into_prop();
        self
    }

    /// The line's color.
    pub fn color(mut self, color: impl IntoProp<Color>) -> Self {
        self.line.color = color.into_prop();
        self
    }
}

impl Default for VerticalDivider {
    fn default() -> Self {
        VerticalDivider::new()
    }
}

impl View for VerticalDivider {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.width, self.line.thickness);
        todo!("WIDGETS-DIVIDER-03")
    }
}
