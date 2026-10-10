//! [`LayoutBuilder`]: content built from the constraints its parent gives it. Spec:
//! `docs/specs/view/layout-builder.md`.

use tantu_layout::BoxConstraints;
use tantu_scene::ElementId;

use crate::{AnyView, BuildCx, View};

/// Content built from the constraints its parent gives it.
pub struct LayoutBuilder {
    builder: Box<dyn Fn(BoxConstraints) -> AnyView>,
}

impl LayoutBuilder {
    /// Content from `builder`, called with the element's constraints.
    pub fn new<V: View>(builder: impl Fn(BoxConstraints) -> V + 'static) -> Self {
        let _ = builder;
        todo!()
    }
}

impl View for LayoutBuilder {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.builder);
        todo!()
    }
}
