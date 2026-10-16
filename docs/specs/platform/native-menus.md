# Native menu bars (macOS and Windows)

- **Status:** Agreed (2026-10-16; the user said "continue" on the six proposals, pinning `muda` 0.19.3)
- **Crates:** `tantu-platform` (menu description, events, fake), `tantu-platform-winit`
  (`muda`, feature `native-menus`, macOS and Windows only), `tantu-widgets` (`MenuBar`
  hands its menus to a host), `tantu` (the runner is the host), `examples/gallery`
- **Plan item:** Phase 4, Menus, part "native OS menus" (ADR 0017 part 2)
- **Related:** [ADR 0017](../../adr/0017-native-desktop-integration.md),
  [menu bar](../widgets/menu-bar.md), [menus](../widgets/menu.md),
  [native dialogs](native-dialogs.md)

## Purpose

On macOS every app's menus live in the bar at the top of the screen, with an application
menu (About, Hide, Quit) first; a menu drawn inside the window looks foreign. On Windows a
native menu bar is the platform's own look and works with Alt and the system's keyboard
handling. This part lets the existing `MenuBar` widget show its menus natively there,
without changing app code beyond one opt-in, and keeps drawing it everywhere else.

## Scope

In scope: a plain-data menu description through the platform; `MenuBar::native(true)`
handing its menus (labels, dividers, submenus, enabled and checked states, shortcut text,
actions) to a host when the platform supports native menus, and drawing nothing then;
reactive updates; activations calling the items' `on_select`; the macOS application menu;
following the focused window; the fake platform; the winit shell with `muda` behind
`native-menus` on macOS and Windows.

Out of scope: native context menus and popup menus (Tantu's are used; a later addition);
menu icons; the Windows accelerator table (shortcuts still reach Tantu's own `Shortcuts`
there); Linux (no GTK, ADR 0017; the bar keeps drawing); the tray (part 3).

## Public API

```rust
// tantu-platform
/// One entry of a native menu.
pub enum NativeMenuItem {
    /// A command. `id` comes back in `WindowEvent::MenuItemActivated`.
    Item { id: u64, label: String, enabled: bool, checked: Option<bool>,
           shortcut: Option<String> },
    Submenu { label: String, enabled: bool, items: Vec<NativeMenuItem> },
    Separator,
}
/// A menu bar: its top-level menus, in order.
pub struct NativeMenu { pub menus: Vec<NativeMenuItem> /* Submenus */ }
impl NativeCapabilities { pub menus: bool, /* … */ }
pub trait PlatformContext {
    // ...
    /// Shows `menu` as `window`'s menu bar (`None` removes it). macOS has one bar for the
    /// app: the last window to set it (or to be focused, see FACADE-APP-21) owns it.
    /// False when not supported (default).
    fn set_menu_bar(&mut self, window: WindowId, menu: Option<NativeMenu>) -> bool { false }
}
pub enum WindowEvent { /* … */ MenuItemActivated { id: u64 } }

// tantu-widgets
impl MenuBar {
    /// Shows the menus natively where the runner can (macOS, Windows with the feature);
    /// elsewhere the bar is drawn as before.
    pub fn native(self, native: bool) -> Self;
}
/// Where a native `MenuBar` sends its menus (the runner provides one; inherited).
pub trait NativeMenuHost {
    /// Replaces the window's menu bar with `menus`; false: not supported, draw it.
    fn set_menus(&self, menus: Vec<NativeMenuEntry>) -> bool;
}
/// A menu bar entry as the host sees it: the platform's data plus the actions.
pub struct NativeMenuEntry { /* label, kind, enabled, checked, shortcut, on_select */ }
```

## Behavior

- **PLATFORM-MENU-01:** `set_menu_bar` defaults to false and `native_capabilities().menus`
  to false. A supported platform shows the menus and later delivers
  `MenuItemActivated { id }` to the window whose bar was activated, never inside the call.
- **PLATFORM-FAKE-19:** The fake platform supports menus unless
  `without_native_dialogs()`/`without_native_menus()` says otherwise, records every
  `set_menu_bar` call per window (`FakeWindow::menu_bars`), and scripts activations as
  ordinary `MenuItemActivated` events.
- **PLATFORM-WINIT-21:** With `native-menus` on macOS and Windows, the shell builds a
  `muda` menu (items, check items, submenus, separators; shortcut text parsed as a `muda`
  accelerator, ignored when it doesn't parse), installs it with `init_for_nsapp` (macOS) or
  `init_for_hwnd` (Windows), and routes `muda`'s menu events through the event-loop proxy.
  On macOS it prepends the application menu (About <app>, Hide, Hide Others, Show All,
  Quit); Quit sends `CloseRequested` to every window rather than ending the process, so
  close vetoes apply. Elsewhere it keeps the default.
- **WIDGETS-MENUBAR-05:** A `MenuBar` with `native(true)` under an inherited
  `NativeMenuHost` gives the host its menus at build and again whenever a label, enabled or
  checked state it reads changes; when the host accepts, the bar takes no space, draws
  nothing and has no semantics node (the OS menu is accessible itself). An activation calls
  that item's `on_select`. Without a host, or when it declines, the bar is drawn as before.
- **FACADE-APP-21:** The runner provides a `NativeMenuHost` at every window's root when
  `native_capabilities().menus`; it assigns ids to entries, calls `set_menu_bar`, and maps
  `MenuItemActivated` back to the entry's action (with the runtime current). When a
  window gains focus it re-sends that window's bar (macOS has one bar for the app).
- **GALLERY-57:** The gallery's menu bar uses `native(true)` (the `native-menus` feature
  is enabled), so on macOS and Windows its View, Go and Help menus are native; on Linux and
  under the tester it is drawn as before (every gallery test and golden unchanged).

## Performance and allocation

The menu description is rebuilt when a state it shows changes (menus are small).

## Open questions

Proposals from the agent, for review:

1. **Opt-in per bar (`native(true)`)** rather than automatic: Windows apps often prefer a
   drawn bar that matches the app's theme, and macOS apps opt in with one line.
2. **Same `MenuBar`, two presentations**: app code keeps one description; the host trait
   lives in `tantu-widgets` (which can't see the platform) and the runner implements it.
3. **No Windows accelerator table**: `muda`'s accelerators need the Win32 loop to call
   `TranslateAcceleratorW`, which winit doesn't do; shortcuts keep going through Tantu's
   `Shortcuts` there and the menu shows their text. On macOS key equivalents work natively.
4. **Quit closes windows** (through `CloseRequested`) instead of terminating, so
   `on_close_request` vetoes keep working.
5. **`muda` 0.19.3, not 0.21:** `muda` 0.20 and later need Rust 1.90, above the MSRV
   (1.88). Pin 0.19.3 (Rust 1.73) now, or raise the MSRV to 1.90 for the newest `muda`
   and `tray-icon` (0.26 also needs 1.90). Recommendation: pin, and raise the MSRV
   separately when another reason appears.
6. **Not testable in CI by clicking**; the `muda` mapping is unit-tested on the macOS and
   Windows runners (which build the feature), and the user checks the gallery on a Mac
   and on Windows.
