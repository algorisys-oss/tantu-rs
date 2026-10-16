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

/// What a native menu event names (PLATFORM-WINIT-21).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "native-menus"), allow(dead_code))]
pub(crate) enum MenuTarget {
    /// The item `id` of the window's bar.
    Item(tantu_platform::WindowId, u64),
    /// The macOS application menu's Quit.
    Quit,
}

/// The `muda` id of the window's item `id`.
#[cfg_attr(not(feature = "native-menus"), allow(dead_code))]
pub(crate) fn menu_id(window: tantu_platform::WindowId, id: u64) -> String {
    let _ = (window, id);
    todo!("PLATFORM-WINIT-21")
}

/// The target a `muda` id names, if it is one of ours.
#[cfg_attr(not(feature = "native-menus"), allow(dead_code))]
pub(crate) fn parse_menu_id(id: &str) -> Option<MenuTarget> {
    let _ = id;
    todo!("PLATFORM-WINIT-21")
}

/// The `muda` menu for a window's bar; on macOS with the application menu first.
#[cfg(all(
    feature = "native-menus",
    any(target_os = "macos", target_os = "windows")
))]
pub(crate) fn build_menu(
    menu: &tantu_platform::NativeMenu,
    window: tantu_platform::WindowId,
    app_name: &str,
) -> muda::Menu {
    let _ = (menu, window, app_name);
    todo!("PLATFORM-WINIT-21")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_winit_21_menu_ids_and_mapping() {
        use tantu_platform::WindowId;
        let w = WindowId::from_raw(3);
        assert_eq!(parse_menu_id(&menu_id(w, 7)), Some(MenuTarget::Item(w, 7)));
        assert_eq!(parse_menu_id("tantu:quit"), Some(MenuTarget::Quit));
        assert_eq!(parse_menu_id("other"), None);
        assert_eq!(parse_menu_id("tantu:x:1"), None);
        // Building a real menu needs the main thread on macOS; Windows checks it here.
        #[cfg(all(feature = "native-menus", target_os = "windows"))]
        {
            use tantu_platform::{NativeMenu, NativeMenuItem};
            let menu = NativeMenu {
                menus: vec![NativeMenuItem::Submenu {
                    label: "File".into(),
                    enabled: true,
                    items: vec![
                        NativeMenuItem::Item {
                            id: 1,
                            label: "Save".into(),
                            enabled: true,
                            checked: None,
                            shortcut: Some("Ctrl+S".into()),
                        },
                        NativeMenuItem::Separator,
                        NativeMenuItem::Item {
                            id: 2,
                            label: "Wrap".into(),
                            enabled: false,
                            checked: Some(true),
                            shortcut: Some("not a shortcut".into()),
                        },
                    ],
                }],
            };
            let built = build_menu(&menu, w, "App");
            let items = built.items();
            assert_eq!(items.len(), 1, "no application menu on Windows");
            let file = items[0].as_submenu().expect("a submenu");
            assert_eq!(file.text(), "File");
            let entries = file.items();
            assert_eq!(entries.len(), 3);
            let save = entries[0].as_menuitem().expect("an item");
            assert_eq!(save.text(), "Save");
            assert_eq!(save.id().0, menu_id(w, 1));
            let wrap = entries[2].as_check_menuitem().expect("a check item");
            assert!(wrap.is_checked() && !wrap.is_enabled());
        }
    }

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
