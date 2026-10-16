//! Native dialogs and notifications (ADR 0017, PLATFORM-WINIT-20), behind the
//! `native-dialogs` and `notifications` features.

#[cfg(feature = "native-dialogs")]
use tantu_platform::{MessageButtons, MessageLevel, MessageResult};

/// Runs a future to completion on the current thread (dialogs are awaited on a worker
/// thread, so the event loop never blocks).
#[cfg(feature = "native-dialogs")]
pub(crate) fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};
    /// Wakes the blocked thread.
    struct Unpark(std::thread::Thread);
    impl Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
        std::thread::park();
    }
}

/// rfd's level for ours.
#[cfg(feature = "native-dialogs")]
pub(crate) fn level(level: MessageLevel) -> rfd::MessageLevel {
    match level {
        MessageLevel::Info => rfd::MessageLevel::Info,
        MessageLevel::Warning => rfd::MessageLevel::Warning,
        MessageLevel::Error => rfd::MessageLevel::Error,
    }
}

/// rfd's buttons for ours.
#[cfg(feature = "native-dialogs")]
pub(crate) fn buttons(buttons: MessageButtons) -> rfd::MessageButtons {
    match buttons {
        MessageButtons::Ok => rfd::MessageButtons::Ok,
        MessageButtons::OkCancel => rfd::MessageButtons::OkCancel,
        MessageButtons::YesNo => rfd::MessageButtons::YesNo,
    }
}

/// Our result for rfd's; anything unexpected counts as dismissing the dialog.
#[cfg(feature = "native-dialogs")]
pub(crate) fn message_result(
    result: rfd::MessageDialogResult,
    buttons: MessageButtons,
) -> MessageResult {
    use rfd::MessageDialogResult as R;
    match result {
        R::Ok => MessageResult::Ok,
        R::Yes => MessageResult::Yes,
        R::No => MessageResult::No,
        // A dismissed dialog: a yes/no question counts as No.
        _ if buttons == MessageButtons::YesNo => MessageResult::No,
        _ => MessageResult::Cancel,
    }
}

/// A notify-rust notification for ours, under the app's name.
#[cfg(feature = "notifications")]
pub(crate) fn notification(
    notification: &tantu_platform::Notification,
    app_name: &str,
) -> notify_rust::Notification {
    let mut n = notify_rust::Notification::new();
    n.summary(&notification.title)
        .body(&notification.body)
        .appname(app_name);
    n
}

#[cfg(test)]
mod tests {
    #[cfg(any(feature = "native-dialogs", feature = "notifications"))]
    use super::*;

    #[test]
    fn platform_winit_20_dialog_and_notification_mapping() {
        #[cfg(feature = "native-dialogs")]
        {
            assert_eq!(block_on(async { 7 }), 7);
            let debug = |v: &dyn std::fmt::Debug| format!("{v:?}");
            assert_eq!(debug(&level(MessageLevel::Warning)), "Warning");
            assert_eq!(debug(&buttons(MessageButtons::YesNo)), "YesNo");
            assert_eq!(debug(&buttons(MessageButtons::OkCancel)), "OkCancel");
            use rfd::MessageDialogResult as R;
            assert_eq!(
                message_result(R::Yes, MessageButtons::YesNo),
                MessageResult::Yes
            );
            assert_eq!(
                message_result(R::No, MessageButtons::YesNo),
                MessageResult::No
            );
            assert_eq!(message_result(R::Ok, MessageButtons::Ok), MessageResult::Ok);
            assert_eq!(
                message_result(R::Cancel, MessageButtons::OkCancel),
                MessageResult::Cancel
            );
            // A dismissed yes/no question counts as No.
            assert_eq!(
                message_result(R::Cancel, MessageButtons::YesNo),
                MessageResult::No
            );
        }
        #[cfg(feature = "notifications")]
        {
            let n = notification(
                &tantu_platform::Notification {
                    title: "Saved".into(),
                    body: "All files".into(),
                },
                "Tantu",
            );
            assert_eq!(n.summary, "Saved");
            assert_eq!(n.body, "All files");
            assert_eq!(n.appname, "Tantu");
        }
    }
}
