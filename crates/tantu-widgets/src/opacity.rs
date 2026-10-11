//! [`Opacity`]: paints its child through a Scene layer with group opacity. Spec:
//! `docs/specs/widgets/transitions.md` (WIDGETS-OPACITY-01..02).

use tantu_view::layout::{BoxConstraints, RenderConstrainedBox};
use tantu_view::scene::Layer;
use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Paint, PaintCx, Prop, View};

/// Paints its child through a Scene layer with group opacity (Flutter's `Opacity`). At 1 no
/// layer is pushed; at 0 nothing is painted; hit-testing is unchanged.
pub struct Opacity<V> {
    opacity: Prop<f32>,
    child: V,
}

impl<V: View> Opacity<V> {
    /// `opacity` in `0..=1` (clamped; NaN counts as 1): a value, closure, signal or animation.
    pub fn new(opacity: impl IntoProp<f32>, child: V) -> Self {
        Opacity {
            opacity: opacity.into_prop(),
            child,
        }
    }
}

impl<V: View> View for Opacity<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        // A pass-through box: the child gets the constraints, the box takes its size.
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::UNCONSTRAINED),
            [AnyView::new(self.child)],
        );
        cx.set_paint(id, OpacityPaint(1.0));
        cx.bind(id, self.opacity, |e, opacity| {
            e.update_paint::<OpacityPaint>(|p| p.0 = clamp(opacity));
        });
        id
    }
}

/// `opacity` in `0..=1`, NaN as 1 (WIDGETS-OPACITY-01).
fn clamp(opacity: f32) -> f32 {
    if opacity.is_nan() {
        1.0
    } else {
        opacity.clamp(0.0, 1.0)
    }
}

/// Wraps the children in a layer (WIDGETS-OPACITY-01).
struct OpacityPaint(f32);

impl Paint for OpacityPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        if self.0 >= 1.0 {
            return; // The children paint as without the widget.
        }
        if self.0 <= 0.0 {
            cx.skip_children();
            return;
        }
        cx.scene().push_layer(Layer {
            opacity: self.0,
            overlay_color: None,
        });
        cx.paint_children();
        cx.scene().pop();
    }
}
