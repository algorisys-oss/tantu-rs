//! [`FileDropTarget`]: accepts files dropped from the OS. Spec:
//! `docs/specs/platform/file-drop.md`.

use std::path::PathBuf;
use std::rc::Rc;

use tantu_view::layout::{BoxConstraints, RenderConstrainedBox};
use tantu_view::reactive::{Signal, untrack};
use tantu_view::{AnyView, BuildCx, ElementId, FileDropEvent, View};

/// Called with the dropped paths.
type OnDrop = Rc<dyn Fn(&[PathBuf])>;

/// Accepts files dropped from the OS onto `child` (the `desktop_drop` package's
/// `DropTarget` is the model). Paints nothing itself; highlight with
/// [`hovering`](Self::hovering) (WIDGETS-FILEDROP-01).
pub struct FileDropTarget {
    child: AnyView,
    on_drop: Option<OnDrop>,
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
        // A box that takes its child's size; the handlers live on it.
        let id = cx.render(
            RenderConstrainedBox::new(BoxConstraints::default()),
            [self.child],
        );
        let (on_drop, hovering) = (self.on_drop, self.hovering);
        let set_hovering = move |value: bool| {
            if let Some(signal) = hovering {
                if untrack(|| signal.try_get()) != Some(value) {
                    signal.set(value);
                }
            }
        };
        cx.on_file_drop(id, move |event| match event {
            FileDropEvent::Hovered(_) => set_hovering(true),
            FileDropEvent::Cancelled => set_hovering(false),
            FileDropEvent::Dropped(paths) => {
                set_hovering(false);
                if let Some(on_drop) = &on_drop {
                    on_drop(paths);
                }
            }
        });
        id
    }
}
