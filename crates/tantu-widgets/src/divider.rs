//! [`Divider`] and [`VerticalDivider`]. Spec: `docs/specs/widgets/divider.md`.

use tantu_view::core::{Color, Rect};
use tantu_view::layout::{BoxConstraints, RenderConstrainedBox, RenderLimitedBox};
use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Paint, PaintCx, Prop, View};

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

    /// Builds the divider: a `LimitedBox` (0 on the unbounded axis) around an element that
    /// expands across the bounded axis, takes `extent` on the other, and paints the line
    /// (WIDGETS-DIVIDER-01, -03).
    fn build(self, cx: &mut BuildCx<'_>, axis: Axis, extent: Prop<f32>) -> ElementId {
        let inner = LineElement {
            axis,
            extent,
            line: self,
        };
        let limited = match axis {
            Axis::Horizontal => RenderLimitedBox::new(0.0, f32::INFINITY),
            Axis::Vertical => RenderLimitedBox::new(f32::INFINITY, 0.0),
        };
        cx.render(limited, [AnyView::new(inner)])
    }
}

/// Which way the line runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    Horizontal,
    Vertical,
}

/// The element that takes the divider's extent and paints its line.
struct LineElement {
    axis: Axis,
    extent: Prop<f32>,
    line: Line,
}

impl View for LineElement {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let axis = self.axis;
        let id = cx.render(RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED), []);
        cx.set_paint(
            id,
            DividerPaint {
                axis,
                thickness: 0.0,
                indent: 0.0,
                end_indent: 0.0,
                color: OUTLINE_VARIANT,
            },
        );
        cx.bind(id, self.extent, move |e, extent| {
            e.update_render::<RenderConstrainedBox>(|r| {
                r.additional = match axis {
                    Axis::Horizontal => BoxConstraints::expand(None, Some(extent)),
                    Axis::Vertical => BoxConstraints::expand(Some(extent), None),
                };
            });
        });
        cx.bind(id, self.line.thickness, |e, v| {
            e.update_paint::<DividerPaint>(|p| p.thickness = v);
        });
        cx.bind(id, self.line.indent, |e, v| {
            e.update_paint::<DividerPaint>(|p| p.indent = v);
        });
        cx.bind(id, self.line.end_indent, |e, v| {
            e.update_paint::<DividerPaint>(|p| p.end_indent = v);
        });
        cx.bind(id, self.line.color, |e, v| {
            e.update_paint::<DividerPaint>(|p| p.color = v);
        });
        id
    }
}

/// Paints the line (WIDGETS-DIVIDER-02).
struct DividerPaint {
    axis: Axis,
    thickness: f32,
    indent: f32,
    end_indent: f32,
    color: Color,
}

/// A length, or 0 for a negative or NaN one.
fn non_negative(value: f32) -> f32 {
    if value.is_nan() || value < 0.0 {
        0.0
    } else {
        value
    }
}

impl Paint for DividerPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let thickness = non_negative(self.thickness);
        let (start, end_indent) = (non_negative(self.indent), non_negative(self.end_indent));
        let (length, breadth) = match self.axis {
            Axis::Horizontal => (size.width, size.height),
            Axis::Vertical => (size.height, size.width),
        };
        let end = length - end_indent;
        if thickness <= 0.0 || end <= start {
            return;
        }
        let across = (breadth - thickness) / 2.0;
        let rect = match self.axis {
            Axis::Horizontal => Rect::from_ltrb(start, across, end, across + thickness),
            Axis::Vertical => Rect::from_ltrb(across, start, across + thickness, end),
        };
        cx.scene().fill_rect(rect, self.color);
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
        self.line.build(cx, Axis::Horizontal, self.height)
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
        self.line.build(cx, Axis::Vertical, self.width)
    }
}
