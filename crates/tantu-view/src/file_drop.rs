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
        if self.tree.contains(element) {
            let handlers = self.tree.events.file_drops.entry(element).or_default();
            crate::events::register(handlers, Rc::new(handler) as FileDropHandler);
        }
    }
}

impl ViewTree {
    /// Delivers a platform file drop to its target (VIEW-FILEDROP-01, -02).
    pub fn dispatch_file_drop(&mut self, event: FileDrop) {
        let current = self.live_file_target();
        match event {
            FileDrop::Hovered { paths, position } => {
                let target = self.file_target_at(position);
                if current != target {
                    if let Some(old) = current {
                        self.file_event(old, &FileDropEvent::Cancelled);
                    }
                }
                self.events.file_target = target;
                if let Some(id) = target {
                    self.file_event(id, &FileDropEvent::Hovered(&paths));
                }
            }
            FileDrop::Cancelled => {
                self.events.file_target = None;
                if let Some(old) = current {
                    self.file_event(old, &FileDropEvent::Cancelled);
                }
            }
            FileDrop::Dropped { paths, position } => {
                self.events.file_target = None;
                // The hovered target takes the drop; without a hover, pick one as for a
                // hover (VIEW-FILEDROP-02).
                if let Some(id) = current.or_else(|| self.file_target_at(position)) {
                    self.file_event(id, &FileDropEvent::Dropped(&paths));
                }
            }
        }
    }

    /// Whether `id` has a live file-drop handler (VIEW-EVENT-10).
    fn is_file_target(&self, id: ElementId) -> bool {
        self.events
            .file_drops
            .get(&id)
            .is_some_and(|list| list.iter().any(|(_, alive)| alive.get()))
    }

    /// The hovered target, if it is still a live, onstage target.
    fn live_file_target(&self) -> Option<ElementId> {
        self.events
            .file_target
            .filter(|id| self.contains(*id) && self.is_file_target(*id) && !self.is_offstage(*id))
    }

    /// The innermost target under `position`, else the outermost target in tree order
    /// (VIEW-FILEDROP-01).
    fn file_target_at(&self, position: Option<Point>) -> Option<ElementId> {
        position
            .and_then(|p| {
                self.hit_test(p)
                    .into_iter()
                    .find(|id| self.is_file_target(*id))
            })
            .or_else(|| self.outermost_file_target(self.root()))
    }

    /// The first target in a preorder walk from `id`, skipping offstage subtrees.
    fn outermost_file_target(&self, id: ElementId) -> Option<ElementId> {
        if self.element(id).is_none_or(|e| e.offstage) {
            return None;
        }
        if self.is_file_target(id) {
            return Some(id);
        }
        self.children(id)
            .iter()
            .find_map(|child| self.outermost_file_target(*child))
    }

    /// Calls `target`'s handlers with `event`, the runtime current.
    fn file_event(&self, target: ElementId, event: &FileDropEvent<'_>) {
        let handlers = crate::events::live(self.events.file_drops.get(&target));
        self.enter(|| {
            for handler in &handlers {
                handler(event);
            }
        });
    }
}
