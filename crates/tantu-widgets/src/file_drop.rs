//! [`FileDropTarget`]: accepts files dropped from the OS. Spec:
//! `docs/specs/platform/file-drop.md`.

use std::path::PathBuf;
use std::rc::Rc;

use tantu_view::reactive::Signal;
use tantu_view::{AnyView, BuildCx, ElementId, View};

/// Accepts files dropped from the OS onto `child` (the `desktop_drop` package's
/// `DropTarget` is the model). Paints nothing itself; highlight with
/// [`hovering`](Self::hovering) (WIDGETS-FILEDROP-01).
pub struct FileDropTarget {
    child: AnyView,
    on_drop: Option<Rc<dyn Fn(&[PathBuf])>>,
    hovering: Option<Signal<bool>>,
}

impl FileDropTarget {
    /// A target around `child`.
    pub fn new(child: impl View) -> Self {
        FileDropTarget {
            child: AnyView::new(child),
            on_drop: None,
            hovering: None,
        }
    }

    /// Called with the dropped paths.
    pub fn on_drop(self, f: impl Fn(&[PathBuf]) + 'static) -> Self {
        FileDropTarget {
            on_drop: Some(Rc::new(f)),
            ..self
        }
    }

    /// Set true while files hover over this target.
    pub fn hovering(self, signal: Signal<bool>) -> Self {
        FileDropTarget {
            hovering: Some(signal),
            ..self
        }
    }
}

impl View for FileDropTarget {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = cx;
        todo!("WIDGETS-FILEDROP-01")
    }
}
