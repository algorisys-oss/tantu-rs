//! Offstage: retained but hidden subtrees. Spec: `docs/specs/view/offstage.md`.

use tantu_scene::ElementId;

use crate::{BuildCx, ElementMut, ViewTree};

impl BuildCx<'_> {
    /// Marks `element` (render or region) offstage or onstage (VIEW-OFFSTAGE-01). Applies at
    /// once while building; reactive callers bind a `Prop<bool>` and use
    /// [`ElementMut::set_offstage`].
    pub fn set_offstage(&mut self, element: ElementId, offstage: bool) {
        let _ = (element, offstage);
        todo!("VIEW-OFFSTAGE-01")
    }
}

impl ElementMut<'_> {
    /// As [`BuildCx::set_offstage`], for updates applied at a frame.
    pub fn set_offstage(&mut self, offstage: bool) {
        let _ = offstage;
        todo!("VIEW-OFFSTAGE-01")
    }
}

impl ViewTree {
    /// Whether `element` or any ancestor is offstage (VIEW-OFFSTAGE-06). False for removed
    /// ids.
    pub fn is_offstage(&self, element: ElementId) -> bool {
        let _ = element;
        todo!("VIEW-OFFSTAGE-06")
    }
}
