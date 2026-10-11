//! [`Builder`]: a view built from a closure that gets the build context (VIEW-TREE-12).

use crate::{BuildCx, ElementId, View};

/// Builds the view `f` returns, giving `f` the build context (Flutter's `Builder`), so `f`
/// can read inherited values (`cx.inherit`) and the text styles where a plain view can't.
pub struct Builder<F> {
    f: F,
}

impl<V: View, F: FnOnce(&mut BuildCx<'_>) -> V + 'static> Builder<F> {
    /// A view that builds whatever `f(cx)` returns, in place.
    pub fn new(f: F) -> Self {
        Builder { f }
    }
}

impl<V: View, F: FnOnce(&mut BuildCx<'_>) -> V + 'static> View for Builder<F> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        (self.f)(cx).build(cx)
    }
}
