//! Modal dialogs: [`Dialog`], [`DialogController`] and [`AlertDialog`]. Spec:
//! `docs/specs/widgets/dialog.md`.

use std::rc::Rc;

use tantu_view::reactive::{Signal, signal};
use tantu_view::{AnyView, BuildCx, ElementId, View};

/// Opens and closes a dialog; `Copy`, owned by the scope that created it.
#[derive(Clone, Copy)]
pub struct DialogController {
    open: Signal<bool>,
}

impl Default for DialogController {
    fn default() -> Self {
        DialogController::new()
    }
}

impl DialogController {
    /// A closed dialog.
    pub fn new() -> Self {
        DialogController {
            open: signal(false),
        }
    }

    /// Opens the dialog.
    pub fn open(&self) {
        if !self.open.get_untracked() {
            self.open.set(true);
        }
    }

    /// Closes the dialog.
    pub fn close(&self) {
        if self.open.get_untracked() {
            self.open.set(false);
        }
    }

    /// Whether the dialog is open (tracked).
    pub fn is_open(&self) -> bool {
        self.open.get()
    }
}

/// A modal dialog shown over the window while its controller is open. Place it anywhere in
/// the tree; it takes no space (WIDGETS-DIALOG-01..05).
pub struct Dialog {
    controller: DialogController,
    content: Rc<dyn Fn() -> AnyView>,
    dismissible: bool,
    on_dismiss: Option<Rc<dyn Fn()>>,
    label: Option<String>,
}

impl Dialog {
    /// A dialog showing `content`, built each time it opens.
    pub fn new<V: View>(controller: DialogController, content: impl Fn() -> V + 'static) -> Self {
        Dialog {
            controller,
            content: Rc::new(move || AnyView::new(content())),
            dismissible: true,
            on_dismiss: None,
            label: None,
        }
    }

    /// Whether a press on the barrier and Escape close the dialog (default true).
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Called when the user dismisses the dialog (barrier or Escape), after it closes.
    pub fn on_dismiss(mut self, f: impl Fn() + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }

    /// The accessible name (an `AlertDialog` names itself after its title).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl View for Dialog {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (
            cx,
            self.controller,
            self.content,
            self.dismissible,
            self.on_dismiss,
            self.label,
        );
        todo!("WIDGETS-DIALOG-01..05")
    }
}

/// Material's basic dialog layout: a title, a message and buttons (WIDGETS-DIALOG-04).
pub struct AlertDialog {
    title: Option<String>,
    content: Option<AnyView>,
    actions: Vec<AnyView>,
}

impl Default for AlertDialog {
    fn default() -> Self {
        AlertDialog::new()
    }
}

impl AlertDialog {
    /// An empty alert dialog.
    pub fn new() -> Self {
        AlertDialog {
            title: None,
            content: None,
            actions: Vec::new(),
        }
    }

    /// The title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The message or other content under the title.
    pub fn content(mut self, content: impl View) -> Self {
        self.content = Some(AnyView::new(content));
        self
    }

    /// Adds a button (or any view) to the actions row, in order, aligned to the end.
    pub fn action(mut self, action: impl View) -> Self {
        self.actions.push(AnyView::new(action));
        self
    }
}

impl View for AlertDialog {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.title, self.content, self.actions);
        todo!("WIDGETS-DIALOG-04")
    }
}
