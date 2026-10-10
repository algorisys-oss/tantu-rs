//! [`Tooltip`]: a short message shown near a widget while hovered or focused. Spec:
//! `docs/specs/widgets/tooltip.md`.

use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};

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
        let _ = (self.message, self.child, self.above, cx);
        todo!()
    }
}
