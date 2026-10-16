# 0017. Native desktop integration: dialogs, notifications, menus and the tray

- **Status:** Proposed (2026-10-16; drafted by the agent for the user's review)
- **Date:** 2026-10-16
- **Related:** [ADR 0004](0004-platform-trait-with-winit-by-default.md),
  [native dialogs and notifications spec](../specs/platform/native-dialogs.md),
  [menus](../specs/widgets/menu.md), [menu bar](../specs/widgets/menu-bar.md),
  [dialogs](../specs/widgets/dialog.md)

## Context

PLAN.md's last Phase 4 items are native OS menus (the macOS app menu, native menu bars and
context menus), native file dialogs, notifications and the system tray. Tantu draws its own
menus, menu bar and modal dialogs already; what is missing is what only the OS can do:
the system file chooser (with its sandbox and portal permissions), OS notifications, the
macOS menu bar at the top of the screen, and an icon in the tray.

ADR 0004 puts all of this behind the `Platform` trait. Each needs a platform crate. The
Rust GUI ecosystem's choices are:

| Need | Crate | License | Linux backend |
|---|---|---|---|
| File and message dialogs | `rfd` 0.17 | MIT | xdg-desktop-portal over D-Bus (default), or GTK 3 |
| Notifications | `notify-rust` 4.18 | MIT/Apache-2.0 | D-Bus (`zbus`) |
| Native menus | `muda` 0.21 (Tauri) | MIT/Apache-2.0 | GTK 3 |
| Tray icon | `tray-icon` 0.26 (Tauri) | MIT/Apache-2.0 | GTK 3 + libappindicator |

`rfd` and `notify-rust` need nothing beyond D-Bus on Linux. `muda` and `tray-icon` need
GTK 3 on Linux through the gtk3-rs bindings, which are no longer maintained, and a GTK
main loop beside winit's.

## Decision

1. **Platform trait additions, plain data.** `tantu-platform` gains request types (file
   dialogs, message dialogs, notifications, menus, tray) and `PlatformContext` methods
   with defaults that report "unsupported"; results come back as events, never by
   blocking a handler. `FakePlatform` scripts and records them. The dependency rule is
   unchanged: only `tantu-platform-winit` sees the crates.
2. **Every integration is an opt-in Cargo feature** of `tantu-platform-winit`, forwarded
   by the facade: `native-dialogs` (`rfd`), `notifications` (`notify-rust`),
   `native-menus` (`muda`), `tray` (`tray-icon`). None is a default feature, so the
   default build (and the "small utility" binary) doesn't grow; AGENTS.md's default
   feature list is unchanged.
3. **Linux without GTK.** `rfd` uses its default xdg-portal backend. `native-menus` and
   `tray` compile `muda`/`tray-icon` only for macOS and Windows; on Linux the platform
   reports them unsupported and apps keep Tantu's own `MenuBar` (what most Linux toolkits
   do). A Linux tray through the StatusNotifierItem D-Bus protocol (`ksni`, pure Rust) is
   left as a later part.
4. **Apps reach them through a facade handle**, `Desktop` (like `Windows`, ADR 0016),
   inherited at every window's root, because `tantu-widgets` and `tantu-view` can't see
   the platform. Requests are queued and applied by the runner after the event.
5. **Three parts:** native file and message dialogs with notifications (first, the most
   requested, no GTK anywhere); native menus (the macOS app menu and menu bars on macOS
   and Windows); the tray.

## Consequences

- Apps get OS file choosers and notifications by enabling a feature; without it, the
  `Desktop` calls report "unsupported" and apps can fall back to Tantu's own dialogs.
- Linux apps don't get native menu bars (by design) or, for now, a tray.
- Four new dependencies, each opt-in and each already used by Rust GUI toolkits (egui,
  Tauri, Slint, iced examples).
- Async results through events add a request id to each call; the runner maps it back to
  the app's callback.
