//! Files dropped from the OS: [`FileDrop`], [`FileDropEvent`] and their delivery. Spec:
//! `docs/specs/platform/file-drop.md`.

use std::path::PathBuf;
use std::rc::Rc;

use tantu_core::Point;

use crate::{BuildCx, ElementId, ViewTree};

/// A file drop from the platform, as the view tree receives it (VIEW-FILEDROP-01).
#[derive(Clone, Debug, PartialEq)]
pub enum FileDrop {
    /// Files are over the window, at `position` when known.
    Hovered {
        /// The dragged files.
        paths: Vec<PathBuf>,
        /// Where, in window coordinates, when known.
        position: Option<Point>,
    },
    /// The drag left the window or was cancelled.
    Cancelled,
    /// Files were dropped, at `position` when known.
    Dropped {
        /// The dropped files.
        paths: Vec<PathBuf>,
        /// Where, in window coordinates, when known.
        position: Option<Point>,
    },
}

/// What a file-drop target's handler receives (VIEW-FILEDROP-02).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FileDropEvent<'a> {
    /// Files are over this target.
    Hovered(&'a [PathBuf]),
    /// They left it, or the drag was cancelled.
    Cancelled,
    /// They were dropped on it.
    Dropped(&'a [PathBuf]),
}

/// A file-drop target's handler.
pub(crate) type FileDropHandler = Rc<dyn Fn(&FileDropEvent<'_>)>;

impl BuildCx<'_> {
    /// Registers `element` as a target for files dropped from the OS (VIEW-FILEDROP-01).
    pub fn on_file_drop(
        &mut self,
        element: ElementId,
        handler: impl Fn(&FileDropEvent<'_>) + 'static,
    ) {
        let _ = (element, handler);
        todo!("VIEW-FILEDROP-01")
    }
}

impl ViewTree {
    /// Delivers a platform file drop to its target (VIEW-FILEDROP-01, -02).
    pub fn dispatch_file_drop(&mut self, event: FileDrop) {
        let _ = event;
        todo!("VIEW-FILEDROP-01")
    }
}
