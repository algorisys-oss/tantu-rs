//! [`AnimatedSwitcher`]: cross-fades between children keyed by a value. Spec:
//! `docs/specs/widgets/transitions.md` (WIDGETS-SWITCHER-01..03).

use std::rc::Rc;
use std::time::Duration;

use tantu_view::{AnyView, BuildCx, Curve, ElementId, IntoProp, Prop, View};

/// Shows the child built for the current key, cross-fading when the key changes (Flutter's
/// `AnimatedSwitcher`). With a `bool` key and an empty view for `false`, an animated `Show`.
pub struct AnimatedSwitcher<K> {
    key: Prop<K>,
    build: Rc<dyn Fn(K) -> AnyView>,
    duration: Duration,
    curve: Curve,
}

impl<K: Clone + PartialEq + 'static> AnimatedSwitcher<K> {
    /// `build(key)` for each new key; default 300 ms, `EaseInOut`.
    pub fn new(key: impl IntoProp<K>, build: impl Fn(K) -> AnyView + 'static) -> Self {
        AnimatedSwitcher {
            key: key.into_prop(),
            build: Rc::new(build),
            duration: Duration::from_millis(300),
            curve: Curve::EaseInOut,
        }
    }

    /// How long a cross-fade takes (default 300 ms).
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// The fades' curve (default `EaseInOut`).
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
}

impl<K: Clone + PartialEq + 'static> View for AnimatedSwitcher<K> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (self.key, self.build, self.duration, self.curve, cx);
        todo!()
    }
}
