//! [`Provide`]: a region providing a value to the views under it. Spec:
//! `docs/specs/view/inherited.md`.

use crate::{BuildCx, ElementId, View};

/// A region providing `value` to the views built under it (VIEW-INHERIT-02).
pub struct Provide<T, V> {
    value: T,
    child: V,
}

impl<T: 'static, V: View> Provide<T, V> {
    /// Provides `value` to `child` and everything under it.
    pub fn new(value: T, child: V) -> Self {
        Provide { value, child }
    }
}

impl<T: 'static, V: View> View for Provide<T, V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let (value, child) = (self.value, self.child);
        cx.region(move |cx| {
            cx.provide(value);
            child.build(cx);
        })
    }
}
