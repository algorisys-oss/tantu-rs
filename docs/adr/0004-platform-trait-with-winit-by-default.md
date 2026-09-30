# 0004. Platform trait, with winit by default

- **Status:** Accepted
- **Date:** 2026-09-30

## Context

Tantu targets Windows, macOS and Linux, including both Wayland and X11, because enterprise Linux
desktops still run X11. Knots supports only Wayland on Linux. Later targets may include the web,
mobile and embedding in a host application.

Beyond windows and input, desktop apps need IME, clipboard, drag and drop, native dialogs, menus,
a tray icon and a connection to the platform's accessibility API.

## Decision

`tantu-platform` defines a **`Platform` trait** that covers:

- windows and the event loop
- input: pointer, keyboard, IME, resize, DPI changes
- clipboard, drag and drop
- dialogs, menus and tray
- the AccessKit adapter
- the window surface a renderer draws into

**`tantu-platform-winit`** is the default implementation. Only `tantu-platform-*` crates may depend
on winit. Where winit doesn't cover a feature (native menus, dialogs, tray, rich clipboard), the
winit shell may use other focused crates. Those choices are made in the platform spec, one at a
time.

## Consequences

- winit gives us Windows, macOS, Wayland and X11 from one maintained codebase.
- Other shells (web canvas, Android/iOS, embedding) can be added later without changing the UI.
- The trait has to fit winit's event-loop model, where winit owns the loop and calls into the app.
  The `App` runner in the facade crate is designed around that.
- Some features will need platform-specific code inside the winit shell, so the shell will be the
  crate with the most `unsafe` and `cfg` code. It is exempt from `#![forbid(unsafe_code)]` for that
  reason.

## Alternatives considered

- **Using winit directly in the facade.** Simpler at first, but it ties every crate that handles
  input to winit and blocks other shells.
- **Our own native shells per OS.** Maximum control, but years of work that winit already does.
- **SDL.** A C dependency, with weaker IME and accessibility integration than winit + AccessKit.
