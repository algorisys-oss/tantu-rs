//! [`Desktop`]: the OS's dialogs and notifications for views (ADR 0017). Spec:
//! `docs/specs/platform/native-dialogs.md`.

use std::path::PathBuf;

use tantu_platform::{FileDialogRequest, MessageRequest, MessageResult};
use tantu_view::BuildCx;

/// The OS's file and message dialogs and notifications, for views. Inherited at every
/// window's root (`Desktop::of(cx)`, `None` outside the runner). `Copy`; requests are applied
/// after the current event (FACADE-DESK-01).
#[derive(Clone, Copy)]
pub struct Desktop {
    _private: (),
}

impl Desktop {
    /// The window's desktop services, or `None` outside a window the runner opened.
    pub fn of(cx: &BuildCx<'_>) -> Option<Desktop> {
        let _ = cx;
        todo!("FACADE-DESK-01")
    }

    /// Opens a file dialog over this window; `done` gets the chosen paths (empty when
    /// cancelled or unsupported).
    pub fn file_dialog(
        self,
        request: FileDialogRequest,
        done: impl FnOnce(Vec<PathBuf>) + 'static,
    ) {
        let _ = (request, done);
        todo!("FACADE-DESK-01")
    }

    /// Asks a question; `done` gets the button (`Cancel`, or `No` for yes/no questions,
    /// when unsupported).
    pub fn message(self, request: MessageRequest, done: impl FnOnce(MessageResult) + 'static) {
        let _ = (request, done);
        todo!("FACADE-DESK-01")
    }

    /// Shows a notification; false when unsupported.
    pub fn notify(self, title: impl Into<String>, body: impl Into<String>) -> bool {
        let _ = (title.into(), body.into());
        todo!("FACADE-DESK-01")
    }

    /// Whether file and message dialogs are available in this build and platform.
    pub fn has_native_dialogs(self) -> bool {
        todo!("FACADE-DESK-02")
    }
}
