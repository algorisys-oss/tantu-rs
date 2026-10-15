//! [`Offstage`]. Spec: `docs/specs/view/offstage.md` (WIDGETS-OFFSTAGE-01).

use tantu_view::{AnyView, BuildCx, ElementId, IntoProp, Prop, View};

/// Flutter's `Offstage`: builds its child once and keeps it, hiding it while `offstage`
/// (no layout, paint, hit-testing, focus or semantics; VIEW-OFFSTAGE-01..06).
pub struct Offstage {
    offstage: Prop<bool>,
    child: Option<AnyView>,
}

impl Offstage {
    /// A hidden-or-shown region; `offstage` is a value, closure or signal (true hides).
    pub fn new(offstage: impl IntoProp<bool>) -> Self {
        Offstage {
            offstage: offstage.into_prop(),
            child: None,
        }
    }

    /// The kept child.
    pub fn child(mut self, child: impl View) -> Self {
        self.child = Some(AnyView::new(child));
        self
    }
}

impl View for Offstage {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let child = self.child;
        let id = cx.region(|cx| {
            if let Some(child) = child {
                child.build(cx);
            }
        });
        cx.bind(id, self.offstage, |e, offstage| e.set_offstage(offstage));
        id
    }
}
