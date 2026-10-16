//! Native dialogs and notifications (ADR 0017, PLATFORM-WINIT-20), behind the
//! `native-dialogs` and `notifications` features.

#[cfg(feature = "native-dialogs")]
use tantu_platform::{MessageButtons, MessageLevel, MessageResult};

/// Runs a future to completion on the current thread (dialogs are awaited on a worker
/// thread, so the event loop never blocks).
#[cfg(feature = "native-dialogs")]
pub(crate) fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let _ = future;
    todo!("PLATFORM-WINIT-20")
}

/// rfd's level for ours.
#[cfg(feature = "native-dialogs")]
pub(crate) fn level(level: MessageLevel) -> rfd::MessageLevel {
    let _ = level;
    todo!("PLATFORM-WINIT-20")
}

/// rfd's buttons for ours.
#[cfg(feature = "native-dialogs")]
pub(crate) fn buttons(buttons: MessageButtons) -> rfd::MessageButtons {
    let _ = buttons;
    todo!("PLATFORM-WINIT-20")
}

/// Our result for rfd's; anything unexpected counts as dismissing the dialog.
#[cfg(feature = "native-dialogs")]
pub(crate) fn message_result(
    result: rfd::MessageDialogResult,
    buttons: MessageButtons,
) -> MessageResult {
    let _ = (result, buttons);
    todo!("PLATFORM-WINIT-20")
}

/// A notify-rust notification for ours, under the app's name.
#[cfg(feature = "notifications")]
pub(crate) fn notification(
    notification: &tantu_platform::Notification,
    app_name: &str,
) -> notify_rust::Notification {
    let _ = (notification, app_name);
    todo!("PLATFORM-WINIT-20")
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
