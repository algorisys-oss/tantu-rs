//! [`Desktop`]: the OS's dialogs and notifications for views (ADR 0017). Spec:
//! `docs/specs/platform/native-dialogs.md`.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use tantu_platform::{
    FileDialogRequest, MessageButtons, MessageRequest, MessageResult, NativeCapabilities,
    Notification,
};
use tantu_reactive::Signal;
use tantu_view::BuildCx;

/// A file dialog's continuation.
pub(crate) type FileDone = Box<dyn FnOnce(Vec<PathBuf>)>;
/// A message dialog's continuation.
pub(crate) type MessageDone = Box<dyn FnOnce(MessageResult)>;

/// What views asked for, applied by the runner after the event (FACADE-DESK-01).
pub(crate) enum DesktopRequest {
    /// A file dialog over the window `token`.
    File {
        token: u64,
        request: FileDialogRequest,
        done: FileDone,
    },
    /// A message dialog over the window `token`.
    Message {
        token: u64,
        request: MessageRequest,
        done: MessageDone,
    },
    /// A notification.
    Notify(Notification),
}

/// The runner's queue of desktop requests, shared by every [`Desktop`] handle.
#[derive(Default)]
pub(crate) struct DesktopQueue(RefCell<Vec<DesktopRequest>>);

impl DesktopQueue {
    fn push(&self, request: DesktopRequest) {
        self.0.borrow_mut().push(request);
    }

    /// The queued requests, oldest first.
    pub(crate) fn take(&self) -> Vec<DesktopRequest> {
        std::mem::take(&mut self.0.borrow_mut())
    }
}

/// What an unsupported message dialog answers: `No` to a yes/no question, else `Cancel`
/// (FACADE-DESK-02).
pub(crate) fn unanswered(buttons: MessageButtons) -> MessageResult {
    match buttons {
        MessageButtons::YesNo => MessageResult::No,
        MessageButtons::Ok | MessageButtons::OkCancel => MessageResult::Cancel,
    }
}

/// The OS's file and message dialogs and notifications, for views. Inherited at every
/// window's root (`Desktop::of(cx)`, `None` outside the runner). `Copy`; requests are applied
/// after the current event (FACADE-DESK-01).
#[derive(Clone, Copy)]
pub struct Desktop {
    /// The runner's queue (owned by the app scope).
    pub(crate) queue: Signal<Rc<DesktopQueue>>,
    /// The window this handle was provided to.
    pub(crate) window: u64,
    /// What the platform supports (FACADE-DESK-02).
    pub(crate) capabilities: NativeCapabilities,
}

impl Desktop {
    /// The window's desktop services, or `None` outside a window the runner opened.
    pub fn of(cx: &BuildCx<'_>) -> Option<Desktop> {
        cx.inherit::<Desktop>()
    }

    /// Opens a file dialog over this window; `done` gets the chosen paths (empty when
    /// cancelled or unsupported).
    pub fn file_dialog(
        self,
        request: FileDialogRequest,
        done: impl FnOnce(Vec<PathBuf>) + 'static,
    ) {
        self.push(DesktopRequest::File {
            token: self.window,
            request,
            done: Box::new(done),
        });
    }

    /// Asks a question; `done` gets the button (`Cancel`, or `No` for yes/no questions,
    /// when unsupported).
    pub fn message(self, request: MessageRequest, done: impl FnOnce(MessageResult) + 'static) {
        self.push(DesktopRequest::Message {
            token: self.window,
            request,
            done: Box::new(done),
        });
    }

    /// Shows a notification after the event; false when unsupported.
    pub fn notify(self, title: impl Into<String>, body: impl Into<String>) -> bool {
        if !self.capabilities.notifications {
            return false;
        }
        self.push(DesktopRequest::Notify(Notification {
            title: title.into(),
            body: body.into(),
        }));
        true
    }

    /// Whether file and message dialogs are available in this build and platform.
    pub fn has_native_dialogs(self) -> bool {
        self.capabilities.dialogs
    }

    fn push(self, request: DesktopRequest) {
        let _ = self.queue.try_with(|queue| queue.push(request));
    }
}
