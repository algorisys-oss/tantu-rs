//! [`Tooltip`]: a short message shown near a widget while hovered or focused. Spec:
//! `docs/specs/widgets/tooltip.md`.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use tantu_view::core::{Color, EdgeInsets, Rect};
use tantu_view::layout::{Alignment, BoxConstraints};
use tantu_view::reactive::{Timer, signal, timer};
use tantu_view::scene::{BorderRadius, RoundedRect};
use tantu_view::text::TextStyle;
use tantu_view::{
    Anchor, BuildCx, ElementId, ElementKind, Handled, IntoProp, OverlayPointer, OverlayPortal,
    Paint, PaintCx, Phase, PointerButton, PointerKind, Prop, Role, Semantics, View,
};

use crate::theme::ThemeHandle;
use crate::{Align, ConstrainedBox, Padding, Text};

/// Space between the child and the tooltip.
const GAP: f32 = 4.0;

/// A short message shown near `child` while hovered or focused (Material 3 plain tooltip).
pub struct Tooltip<V> {
    message: Prop<String>,
    child: V,
    above: bool,
    wait: Duration,
}

impl<V: View> Tooltip<V> {
    /// A tooltip showing `message` (a value, a closure or a signal) for `child`.
    pub fn new(message: impl IntoProp<String>, child: V) -> Self {
        Tooltip {
            message: message.into_prop(),
            child,
            above: false,
            wait: Duration::from_millis(500),
        }
    }

    /// How long the pointer must rest on the child before the tooltip shows (default 500 ms;
    /// zero shows at once) (WIDGETS-TOOLTIP-07).
    pub fn wait_duration(mut self, duration: Duration) -> Self {
        self.wait = duration;
        self
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
        let wait = self.wait;
        let message: Rc<dyn Fn() -> String> = match self.message {
            Prop::Value(text) => Rc::new(move || text.clone()),
            Prop::Dynamic(f) => Rc::from(f),
        };
        let anchor = if self.above {
            Anchor::new(Alignment::TOP_CENTER, Alignment::BOTTOM_CENTER).offset(0.0, -GAP)
        } else {
            Anchor::new(Alignment::BOTTOM_CENTER, Alignment::TOP_CENTER).offset(0.0, GAP)
        };
        // The overlay layer hangs off the root, so the theme is captured here, where the
        // portal is, and passed to the box (WIDGETS-THEME-01).
        let theme = ThemeHandle::of(cx);
        let portal = OverlayPortal::new(self.child)
            .visible(move || hovered.get() || focused.get())
            .anchor(anchor)
            .pointer(OverlayPointer::Passthrough)
            .overlay(move || TooltipBox {
                message: Rc::clone(&message),
                theme,
            });
        let id = portal.build(cx);
        if cx.tree().kind(id) == Some(ElementKind::Region) {
            tracing::warn!(
                "Tooltip around a dynamic region gets no pointer events; it never shows"
            );
            return id;
        }
        // The show delay (WIDGETS-TOOLTIP-07): a timer owned by the portal's scope, so it
        // dies with the tooltip; leaving or pressing cancels it.
        let scope = cx.tree().scope(id);
        let pending: Rc<Cell<Option<Timer>>> = Rc::default();
        cx.on_pointer(id, Phase::Bubble, move |p| {
            match p.event.kind {
                PointerKind::Enter => {
                    if wait.is_zero() {
                        hovered.set(true);
                    } else if let Some(scope) = scope.filter(|s| !s.is_disposed()) {
                        if let Some(timer) = pending.take() {
                            timer.cancel();
                        }
                        let timer = scope.run(|| timer(wait, move || hovered.set(true)));
                        pending.set(Some(timer));
                    }
                }
                PointerKind::Leave | PointerKind::Down(PointerButton::Primary) => {
                    if let Some(timer) = pending.take() {
                        timer.cancel();
                    }
                    hovered.set(false);
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
struct TooltipBox {
    message: Rc<dyn Fn() -> String>,
    theme: ThemeHandle,
}

impl View for TooltipBox {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (message, theme) = (self.message, self.theme);
        let message: Rc<dyn Fn() -> String> = message;
        let style = TextStyle {
            size: 12.0,
            ..theme.snapshot().text.body
        };
        let content =
            ConstrainedBox::new(BoxConstraints::new(0.0, f32::INFINITY, 24.0, f32::INFINITY))
                .child(
                    Padding::new(EdgeInsets::symmetric(8.0, 4.0)).child(
                        Align::new(Alignment::CENTER)
                            .width_factor(1.0)
                            .height_factor(1.0)
                            .child(
                                Text::new({
                                    let message = Rc::clone(&message);
                                    move || message()
                                })
                                .style(style)
                                .color(move || theme.colors().inverse_on_surface),
                            ),
                    ),
                );
        let id = content.build(cx);
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Tooltip,
                ..Default::default()
            },
        );
        {
            let message = Rc::clone(&message);
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || message())),
                |e, text: String| {
                    e.update_semantics(|s| s.name = Some(text));
                },
            );
        }
        cx.set_paint(
            id,
            TooltipPaint {
                color: Color::TRANSPARENT,
                radius: 0.0,
            },
        );
        cx.bind(
            id,
            Prop::Dynamic(Box::new(move || {
                (theme.colors().inverse_surface, theme.shape().extra_small)
            })),
            |e, (color, radius)| {
                e.update_paint::<TooltipPaint>(|p| {
                    p.color = color;
                    p.radius = radius;
                });
            },
        );
        id
    }
}

/// Fills the box with a rounded rect in the theme's inverse surface.
struct TooltipPaint {
    color: Color,
    radius: f32,
}

impl Paint for TooltipPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        let rect = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        cx.scene().fill(
            RoundedRect::new(rect, BorderRadius::circular(self.radius)),
            self.color,
        );
    }
}
