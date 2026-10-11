//! [`FocusScope`]. Spec: `docs/specs/view/focus-scope.md`.

use tantu_view::{AnyView, BuildCx, ElementId, View};

/// A subtree whose Tab traversal stays inside it and which remembers its last focused
/// descendant (Flutter's `FocusScope`). Paints nothing (WIDGETS-FOCUSSCOPE-01).
pub struct FocusScope {
    child: AnyView,
}

impl FocusScope {
    /// A scope around `child`.
    pub fn new(child: impl View) -> Self {
        FocusScope {
            child: AnyView::new(child),
        }
    }
}

impl View for FocusScope {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let child = self.child;
        let id = cx.region(|cx| {
            child.build(cx);
        });
        cx.focus_scope(id);
        id
    }
}
