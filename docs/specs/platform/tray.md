# The system tray (macOS and Windows)

- **Status:** Implemented (2026-10-16; the user said "continue" on the six proposals)
- **Crates:** `tantu-platform` (tray description, app events, fake), `tantu-platform-winit`
  (`tray-icon`, feature `tray`, macOS and Windows only), `tantu-widgets` (menu entries for
  hosts), `tantu` (`SystemTray` view, the runner), `examples/gallery`
- **Plan item:** Phase 4, Dialogs/notifications/tray, part "the system tray" (ADR 0017 part 3)
- **Related:** [ADR 0017](../../adr/0017-native-desktop-integration.md),
  [native menus](native-menus.md), [native dialogs](native-dialogs.md)

## Purpose

Background-friendly apps (sync clients, chat, monitors) put an icon in the Windows
notification area or the macOS menu bar extras, with a tooltip and a small menu. This part
adds one tray icon per app, declared as a view so it lives as long as the view that shows
it and follows signals like any other view.

## Scope

In scope: one tray icon per app (RGBA image, tooltip, a menu of `MenuItem`s with actions,
check marks, dividers and submenus, and a click callback); reactive updates; removal when
the view goes away; app-level platform events; the fake platform; the winit shell with
`tray-icon` 0.24.2 behind `tray` on macOS and Windows.

Out of scope: Linux (no GTK, ADR 0017; a StatusNotifierItem tray through `ksni` is a later
part); several tray icons; apps that run with no window at all (the run still ends when the
last window closes); balloon messages (use notifications); template/dark-mode icon variants.

## Public API

```rust
// tantu-platform
/// The tray icon's image: straight-alpha RGBA8 (like `ClipboardImage`).
pub struct TrayImage { pub width: u32, pub height: u32, pub rgba: Vec<u8> }
/// The app's tray icon.
pub struct Tray { pub image: TrayImage, pub tooltip: Option<String>,
                  pub menu: Vec<NativeMenuItem> }
impl NativeCapabilities { pub tray: bool /* … */ }
pub trait PlatformContext {
    // ...
    /// Shows (or replaces, or with `None` removes) the app's tray icon. False when not
    /// supported (default).
    fn set_tray(&mut self, tray: Option<Tray>) -> bool { false }
}
/// Something that happened to the app rather than a window.
pub enum AppEvent {
    /// A tray menu item was chosen.
    TrayMenuItemActivated { id: u64 },
    /// The tray icon was clicked (primary button).
    TrayClicked,
}
pub trait PlatformHandler {
    // ...
    /// An app-level event (default: ignored).
    fn app_event(&mut self, cx: &mut dyn PlatformContext, event: AppEvent) {}
}

// tantu-widgets
/// The host's view of menu items (what `MenuBar::native` sends), for other hosts.
pub fn native_menu_entries(items: &[MenuItem]) -> Vec<NativeMenuEntry>;

// tantu (facade)
/// The app's tray icon, shown while this view is built (under the runner, on platforms
/// with a tray). Paints nothing and takes no space.
pub struct SystemTray { /* … */ }
impl SystemTray {
    pub fn new(image: ImageData) -> Self;
    pub fn tooltip(self, text: impl IntoProp<String>) -> Self;
    pub fn menu(self, items: impl IntoIterator<Item = MenuItem>) -> Self;
    pub fn on_click(self, f: impl Fn() + 'static) -> Self;
}
```

## Behavior

- **PLATFORM-TRAY-01:** `set_tray` defaults to false and `native_capabilities().tray` to
  false; `PlatformHandler::app_event` defaults to doing nothing. A supported platform shows
  the icon, replaces it on the next call, removes it on `None`, and delivers
  `AppEvent::TrayMenuItemActivated`/`TrayClicked` later, never inside a call.
- **PLATFORM-FAKE-20:** The fake platform supports the tray unless
  `without_native_menus()` (or `without_native_dialogs()`), records every `set_tray` call
  in `FakeLog::trays`, and scripts app events with `FakeStep::App(AppEvent)`.
- **PLATFORM-WINIT-22:** With `tray` on macOS and Windows, the shell builds a `tray-icon`
  icon from the image (`Icon::from_rgba`), tooltip and a `muda` menu (items get
  `tantu:tray:<id>` ids), keeps it alive until replaced or removed, and forwards menu and
  click events through the event-loop proxy as app events. Elsewhere it keeps the default.
- **FACADE-TRAY-01:** A `SystemTray` built in a window the runner opened (with
  `native_capabilities().tray`) sends its tray after the event: the image, the tooltip and
  the menu as `native_menu_entries` would describe it, numbering items; it re-sends when
  the tooltip or a menu state it reads changes. Chosen items call their `on_select`, a
  click calls `on_click`, with the runtime current.
- **FACADE-TRAY-02:** When the `SystemTray`'s scope is disposed (its window closes or a
  `Dyn` drops it) the runner removes the icon; a second `SystemTray` replaces the first
  (the app has one icon). Without support it does nothing.
- **FACADE-TRAY-03:** The facade feature `tray` forwards to `tantu-platform-winit`; it is
  not a default feature.
- **GALLERY-58:** The gallery shows a tray icon (the 32 × 32 gradient, tooltip "Tantu
  gallery") with "Dark theme" (a check item on the shared flag), "New window" and "Close
  gallery" (closes the main window); the `tray` feature is enabled. Under the tester it
  shows nothing (every gallery test and golden unchanged).

## Performance and allocation

The tray is rebuilt when its tooltip or a menu state changes.

## Open questions

Proposals from the agent, for review:

1. **`tray-icon` 0.24.2** (Rust 1.73, the same `muda` 0.19 as the menu bars); 0.25 and
   later need Rust 1.90.
2. **A declarative `SystemTray` view** whose lifetime is the tray's, rather than an
   imperative `Desktop::set_tray`: it follows signals and cleans itself up.
3. **One tray icon per app**; a second `SystemTray` replaces the first.
4. **App-level events** through a new `PlatformHandler::app_event` with a default, so
   other shells keep compiling; tray events aren't tied to a window.
5. **No tray-only apps yet**: the run still ends with the last window (a "keep running in
   the tray" mode needs a window-less lifecycle; later).
6. **Not testable by clicking in CI**; the `tray-icon` mapping is unit-tested where it can
   run (Windows), the rest through the fake platform, and the user checks the gallery's
   tray on a Mac and on Windows.
