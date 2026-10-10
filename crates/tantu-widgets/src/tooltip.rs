//! [`Tooltip`]: a short message shown near a widget while hovered or focused. Spec:
//! `docs/specs/widgets/tooltip.md`.

use std::rc::Rc;

use tantu_view::core::{Color, EdgeInsets, Rect};
use tantu_view::layout::{Alignment, BoxConstraints};
use tantu_view::reactive::signal;
use tantu_view::scene::{BorderRadius, RoundedRect};
use tantu_view::text::TextStyle;
use tantu_view::{
    Anchor, BuildCx, ElementId, ElementKind, Handled, IntoProp, OverlayPointer, OverlayPortal,
    Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, View,
};

use crate::{Align, ConstrainedBox, Padding, Text};

/// The tooltip's background (Material 3 inverse surface).
const INVERSE_SURFACE: Color = Color::from_argb32(0xFF31_3033);
/// The message color (Material 3 inverse on-surface).
const INVERSE_ON_SURFACE: Color = Color::from_argb32(0xFFF4_EFF4);
/// Space between the child and the tooltip.
const GAP: f32 = 4.0;

/// A short message shown near `child` while hovered or focused (Material 3 plain tooltip).
pub struct Tooltip<V> {
    message: Prop<String>,
    child: V,
    above: bool,
}

impl<V: View> Tooltip<V> {
    /// A tooltip showing `message` (a value, a closure or a signal) for `child`.
    pub fn new(message: impl IntoProp<String>, child: V) -> Self {
        Tooltip {
            message: message.into_prop(),
            child,
            above: false,
        }
    }

    /// Show above the child instead of below (it still flips when there's no room).
    pub fn prefer_above(mut self) -> Self {
        self.above = true;
        self
    }
}

impl<V: View> View for Tooltip<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (hovered, focused) = (signal(false), signal(false));
        let message: Rc<dyn Fn() -> String> = match self.message {
            Prop::Value(text) => Rc::new(move || text.clone()),
            Prop::Dynamic(f) => Rc::from(f),
        };
        let anchor = if self.above {
            Anchor::new(Alignment::TOP_CENTER, Alignment::BOTTOM_CENTER).offset(0.0, -GAP)
        } else {
            Anchor::new(Alignment::BOTTOM_CENTER, Alignment::TOP_CENTER).offset(0.0, GAP)
        };
        let portal = OverlayPortal::new(self.child)
            .visible(move || hovered.get() || focused.get())
            .anchor(anchor)
            .pointer(OverlayPointer::Passthrough)
            .overlay(move || TooltipBox(Rc::clone(&message)));
        let id = portal.build(cx);
        if cx.tree().kind(id) == Some(ElementKind::Region) {
            tracing::warn!(
                "Tooltip around a dynamic region gets no pointer events; it never shows"
            );
            return id;
        }
        cx.on_pointer(id, Phase::Bubble, move |p| {
            match p.event.kind {
                PointerKind::Enter => hovered.set(true),
                PointerKind::Leave | PointerKind::Down(PointerButton::Primary) => {
                    hovered.set(false)
                }
                _ => {}
            }
            Handled::Continue
        });
        cx.on_focus_within(id, move |inside| focused.set(inside));
        id
    }
}

/// The tooltip's box: at least 24 px tall, 8 × 4 px padding, the message centered in 12 px.
struct TooltipBox(Rc<dyn Fn() -> String>);

impl View for TooltipBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let message = self.0;
        let style = TextStyle {
            size: 12.0,
            ..TextStyle::body()
        };
        let content =
            ConstrainedBox::new(BoxConstraints::new(0.0, f32::INFINITY, 24.0, f32::INFINITY))
                .child(
                    Padding::new(EdgeInsets::symmetric(8.0, 4.0)).child(
                        Align::new(Alignment::CENTER)
                            .width_factor(1.0)
                            .height_factor(1.0)
                            .child(
                                Text::new(move || message())
                                    .style(style)
                                    .color(INVERSE_ON_SURFACE),
                            ),
                    ),
                );
        let id = content.build(cx);
        cx.set_paint(id, TooltipPaint);
        id
    }
}

/// Fills the box with a 4 px rounded rect.
struct TooltipPaint;

impl Paint for TooltipPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let rect = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        cx.scene().fill(
            RoundedRect::new(rect, BorderRadius::circular(4.0)),
            INVERSE_SURFACE,
        );
    }
}
