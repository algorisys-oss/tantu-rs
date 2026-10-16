# Native file and message dialogs, and notifications

- **Status:** Agreed (2026-10-16; the user said "continue" on the six proposals)
- **Crates:** `tantu-platform` (requests, events, fake), `tantu-platform-winit` (`rfd`,
  `notify-rust`, behind features), `tantu` (`Desktop`, the runner, features),
  `examples/gallery`
- **Plan item:** Phase 4, "Dialogs, native file dialogs, notifications, system tray", part
  "native file and message dialogs, notifications" (part 1 of ADR 0017)
- **Related:** [ADR 0017](../../adr/0017-native-desktop-integration.md),
  [modal dialogs](../widgets/dialog.md), [multiple windows](../facade/windows.md)

## Purpose

Apps open and save files through the OS chooser (it knows the user's places, handles
sandbox permissions and looks native), ask the occasional native yes/no question, and tell
the user something happened while the window is in the background. This part adds those
three through the `Platform` trait and a `Desktop` handle.

## Scope

In scope: open (one or many files), save and pick-folder dialogs with a title, a starting
directory, a file name and extension filters; message dialogs (info, warning, error) with
Ok, Ok/Cancel or Yes/No buttons; notifications with a title and body; the fake platform;
the winit shell behind `native-dialogs` and `notifications`; `Desktop` in the facade; a
gallery sample.

Out of scope: native menus and the tray (later parts of ADR 0017); notification actions,
icons, sounds and click callbacks; custom dialog contents; synchronous (blocking) APIs.

## Public API

```rust
// tantu-platform
/// What kind of file dialog.
pub enum FileDialogKind { Open, OpenMany, Save, PickFolder }
/// A file dialog request.
pub struct FileDialogRequest {
    pub kind: FileDialogKind,
    pub title: Option<String>,
    pub directory: Option<PathBuf>,
    pub file_name: Option<String>,
    /// (name, extensions without dots), e.g. ("Images", ["png", "jpg"]).
    pub filters: Vec<(String, Vec<String>)>,
}
pub enum MessageLevel { Info, Warning, Error }
pub enum MessageButtons { Ok, OkCancel, YesNo }
pub struct MessageRequest { pub level: MessageLevel, pub title: String,
                            pub description: String, pub buttons: MessageButtons }
/// Which button closed a message dialog.
pub enum MessageResult { Ok, Cancel, Yes, No }
pub struct Notification { pub title: String, pub body: String }
/// Identifies one dialog request.
pub struct DialogId(pub u64);

pub trait PlatformContext {
    // ...
    /// Opens a file dialog for `window`; the result arrives as
    /// `WindowEvent::FileDialogClosed`. `None`: not supported (default).
    fn file_dialog(&mut self, window: WindowId, request: FileDialogRequest) -> Option<DialogId> { None }
    /// Opens a message dialog; the result arrives as `WindowEvent::MessageDialogClosed`.
    fn message_dialog(&mut self, window: WindowId, request: MessageRequest) -> Option<DialogId> { None }
    /// Shows a notification; false when not supported or it failed (default false).
    fn notify(&mut self, notification: &Notification) -> bool { false }
    /// What this platform supports (default: nothing). Added while implementing (review):
    /// `Desktop` needs it for `has_native_dialogs` and `notify`'s answer, which are
    /// synchronous while requests are applied after the event.
    fn native_capabilities(&self) -> NativeCapabilities { NativeCapabilities::default() }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeCapabilities { pub dialogs: bool, pub notifications: bool }
pub enum WindowEvent {
    // ...
    /// A file dialog closed: the chosen paths, empty when cancelled.
    FileDialogClosed { id: DialogId, paths: Vec<PathBuf> },
    /// A message dialog closed.
    MessageDialogClosed { id: DialogId, result: MessageResult },
}

// tantu (facade)
/// The OS's dialogs and notifications for views; inherited at every window's root
/// (`Desktop::of(cx)`, `None` outside the runner). `Copy`.
pub struct Desktop { /* … */ }
impl Desktop {
    pub fn of(cx: &BuildCx<'_>) -> Option<Desktop>;
    /// Opens a file dialog over this window; `done` gets the paths (empty when cancelled
    /// or unsupported).
    pub fn file_dialog(self, request: FileDialogRequest, done: impl FnOnce(Vec<PathBuf>) + 'static);
    /// Asks a question; `done` gets the button (`Cancel`/`No` when unsupported).
    pub fn message(self, request: MessageRequest, done: impl FnOnce(MessageResult) + 'static);
    /// Shows a notification; false when unsupported.
    pub fn notify(self, title: impl Into<String>, body: impl Into<String>) -> bool;
    /// Whether file and message dialogs are available in this build and platform.
    pub fn has_native_dialogs(self) -> bool;
}
```

## Behavior

- **PLATFORM-NATIVE-01:** `PlatformContext`'s new methods default to unsupported (`None`,
  `false`); a supported request returns a fresh `DialogId` at once, and exactly one
  `FileDialogClosed`/`MessageDialogClosed` with that id arrives later for the window,
  after the call returns (never inside it).
- **PLATFORM-FAKE-18:** The fake platform records every request and notification in the
  log and answers each dialog from a script (`FakePlatform::file_dialog_answer(paths)`,
  `message_answer(result)`, in order; unanswered dialogs get an empty path list or
  `Cancel`), delivering each result after the next script step's own event (a
  `FakeStep::Idle` suffices), in request order; a window closed by then gets nothing
  (clarified while writing the tests, review). `without_native_dialogs()` gives the
  trait's defaults.
- **PLATFORM-WINIT-20:** With `native-dialogs`, the shell runs `rfd`'s async dialogs
  (parented to the window) without blocking the event loop and delivers the result
  through the event-loop proxy; with `notifications`, `notify` shows a `notify-rust`
  notification titled with the app's name. Without the features they keep the defaults.
- **FACADE-DESK-01:** `Desktop::of` is `Some` in every window the runner opens. Requests
  are applied after the event; each result calls its `done` once, with the app's runtime
  current, then the window redraws as after any event. A window closed before its result
  arrives drops the callback without calling it.
- **FACADE-DESK-02:** `Desktop` reads the platform's `native_capabilities()` when the
  window opens: `has_native_dialogs` is its `dialogs`, and `notify` returns its
  `notifications` (the notification itself is shown after the event). Without support, `file_dialog` calls `done(vec![])` and `message`
  calls `done(Cancel)` (for `YesNo`: `No`) after the event, and `has_native_dialogs` is
  false, so apps can fall back to Tantu's own `Dialog`.
- **FACADE-DESK-03:** The facade features `native-dialogs` and `notifications` forward to
  `tantu-platform-winit`; neither is a default feature.
- **GALLERY-56:** A "desktop" kitchen-sink card (after "clipboard"; 30 cards) has "Open
  file…", "Ask…" and "Notify" buttons and echoes the outcome in `sink-desktop-state`
  ("Opened: a.txt", "Open cancelled", "Answer: Yes", "Notified", "Not supported in this
  build"). The gallery enables both features. The buttons are keyed `sink-open_file`,
  `sink-ask` and `sink-notify` (the card's `SINK_CARDS` samples; added while writing the
  tests, review).

## Performance and allocation

Requests allocate their strings once; nothing per frame.

## Open questions

Proposals from the agent, for review:

1. **Accept ADR 0017**: opt-in features, platform-trait requests with event results, no
   GTK on Linux, three parts in this order.
2. **Non-blocking only.** Results arrive as events, so windows keep repainting while the
   OS dialog is open; no `pick_file() -> Option<PathBuf>` that stalls the loop.
3. **`Desktop` handle in the facade**, beside `Windows`, rather than widgets: the widgets
   can't see the platform, and apps choose when to call it.
4. **Unsupported answers like a cancel** (FACADE-DESK-02), so app code has one path;
   `has_native_dialogs` lets it offer Tantu's own dialog instead.
5. **Notifications are fire-and-forget** in this part (no actions or click callbacks).
6. **The gallery enables both features**; CI builds them on all three OSes but can't
   click OS dialogs, so the winit side is covered by an opt-in window test.
