//! [`Opacity`]: paints its child through a Scene layer with group opacity. Spec:
//! `docs/specs/widgets/transitions.md` (WIDGETS-OPACITY-01..02).

use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

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
        let _ = (self.opacity, self.child, cx);
        todo!()
    }
}
