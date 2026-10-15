//! Offstage: retained but hidden subtrees. Spec: `docs/specs/view/offstage.md`.

use tantu_scene::ElementId;

use crate::{BuildCx, ElementMut, ViewTree};

impl BuildCx<'_> {
    /// Marks `element` (render or region) offstage or onstage (VIEW-OFFSTAGE-01). Applies at
    /// once while building; reactive callers bind a `Prop<bool>` and use
    /// [`ElementMut::set_offstage`].
    pub fn set_offstage(&mut self, element: ElementId, offstage: bool) {
        self.tree.set_offstage_flag(element, offstage);
    }
}

impl ElementMut<'_> {
    /// As [`BuildCx::set_offstage`], for updates applied at a frame.
    pub fn set_offstage(&mut self, offstage: bool) {
        let id = self.id;
        self.tree.set_offstage_flag(id, offstage);
    }
}

impl ViewTree {
    /// Whether `element` or any ancestor is offstage (VIEW-OFFSTAGE-06). False for removed
    /// ids.
    pub fn is_offstage(&self, element: ElementId) -> bool {
        let mut next = Some(element);
        while let Some(id) = next {
            let Some(e) = self.element(id) else {
                return false;
            };
            if e.offstage {
                return true;
            }
            next = self.parent(id);
        }
        false
    }

    /// Sets `id`'s own flag; a change re-syncs its render ancestor's layout children
    /// (VIEW-OFFSTAGE-02), releases focus, capture and hover inside a hidden subtree
    /// (VIEW-OFFSTAGE-03, -04) and updates the overlays (VIEW-OFFSTAGE-05). Setting the
    /// current value changes nothing (VIEW-OFFSTAGE-06).
    pub(crate) fn set_offstage_flag(&mut self, id: ElementId, offstage: bool) {
        let Some(element) = self.element_mut(id) else {
            return;
        };
        if element.offstage == offstage {
            return;
        }
        element.offstage = offstage;
        if let Some(parent) = self.parent(id) {
            self.sync_layout_children(parent);
        }
        if offstage {
            self.release_focus_within(id);
            self.release_pointer_offstage();
        }
        self.sync_overlay_offstage();
    }
}
