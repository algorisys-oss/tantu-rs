# winit platform shell

- **Status:** Agreed
- **Crate:** `tantu-platform-winit`
- **Plan item:** Phase 1, "`tantu-platform-winit`: winit 0.30 shell, event conversion, window
  handles for renderers"
- **Related:** [ADR 0004](../../adr/0004-platform-trait-with-winit-by-default.md),
  [platform](../platform/platform.md) (the trait and event types this implements)

## Purpose

`WinitPlatform` implements `tantu_platform::Platform` on [winit](https://github.com/rust-windowing/winit)
0.30, giving Tantu windows and input on Windows, macOS and Linux (Wayland and X11). It converts
winit's events into Tantu's event types and hands renderers the native window handles. It is the
only crate that sees winit types.

## Scope

In scope:

- `WinitPlatform::new` and `Platform::run` on winit's `ApplicationHandler` model.
- `PlatformContext` on real windows: create (from `WindowAttributes`), close, request redraw,
  size, scale factor, title, surface target (`Arc<winit::window::Window>`), exit.
- Conversion of winit events and values: window events, pointer (mouse) events with logical
  positions, wheel deltas, keys, key locations, modifiers, sizes.

Out of scope: everything ADR 0004 lists beyond Phase 1 (IME, clipboard, drag and drop, cursor
icons, touch, menus, dialogs, tray, AccessKit), and presenting pixels (renderers draw through the
surface target).

## Dependencies

- `winit` 0.30.13 (Apache-2.0, MSRV 1.70) with its default features: X11, Wayland (loaded at
  run time, so no system libraries are needed to build), Wayland client-side decorations, and
  `raw-window-handle` 0.6. Allowed only in `tantu-platform-*` (AGENTS.md). Builds on Rust 1.85
  (checked).
- Note: the Wayland decorations crate (`sctk-adwaita`) pulls in `tiny-skia` 0.11 transitively.
  The dependency rule is about which Tantu crates use tiny-skia; this is winit drawing its own
  title bars, so it is accepted. Without it, GNOME on Wayland shows windows with no title bar.

## Public API

Crate root `tantu_platform_winit`.

```rust
use tantu_platform::{Platform, PlatformError, PlatformHandler};

/// The winit platform shell. Create it on the main thread.
pub struct WinitPlatform { /* winit::event_loop::EventLoop<()> */ }

impl WinitPlatform {
    /// Creates the event loop. Must be called on the main thread (a winit requirement on macOS,
    /// and on Linux). Fails, without panicking, when there is no display or an event loop was
    /// already created in this process.
    pub fn new() -> Result<WinitPlatform, PlatformError>;
}

impl Platform for WinitPlatform { /* run: winit's run_app */ }
impl std::fmt::Debug for WinitPlatform { /* ... */ }
```

Conversion functions are private; the rules below describe them and unit tests in `src/` check
them.

## Behavior

Running

- **PLATFORM-WINIT-01:** `run` calls `started` once, at winit's first `resumed`; later `resumed`
  calls (mobile-style suspend/resume) don't call it again. It calls `window_event` for each
  converted event of a Tantu-created window, and `idle` at each `about_to_wait`. It returns
  `Ok(())` after `exit`, and `Err(PlatformError::Os(_))` if winit's loop fails.
- **PLATFORM-WINIT-02:** `create_window` creates a winit window from the attributes (title,
  logical inner size, logical minimum size, resizable, visible), assigns it the next `WindowId`
  (starting at 1), and requests a redraw for it. Creation failures return
  `Err(PlatformError::Os(_))`.
- **PLATFORM-WINIT-03:** `close_window` hides the window and forgets it: later winit events for
  it are dropped, `inner_size`/`scale_factor`/`surface_target` return `None`. The native window
  is destroyed when the last `SurfaceTarget` clone is dropped.
- **PLATFORM-WINIT-04:** `surface_target` returns the window as `Arc<dyn WindowHandles>`, whose
  window and display handles are valid while it is held. `inner_size` and `scale_factor` read the
  window. `request_redraw` and `set_title` call winit. `exit` makes the loop exit.

Conversions

- **PLATFORM-WINIT-05:** Keys: winit `Key::Named(n)` maps to `Key::Named` with the same name for
  every `NamedKey` Tantu has; `Space` maps to `Key::Character(" ")`; other named keys,
  `Unidentified` and `Dead` map to `Key::Unidentified`; `Character(s)` maps to `Character(s)`.
- **PLATFORM-WINIT-06:** Key locations map one to one (`Standard`, `Left`, `Right`, `Numpad`).
  A keyboard event becomes `WindowEvent::Keyboard` with the converted key and location,
  `Pressed`/`Released`, `repeat`, and `text` as a `String`. Key events marked synthetic by winit
  (sent for keys already held when focus arrives) are dropped.
- **PLATFORM-WINIT-07:** Modifiers: winit's `SHIFT`, `CONTROL`, `ALT` and `SUPER` set `shift`,
  `control`, `alt` and `super_key`.
- **PLATFORM-WINIT-08:** Mouse buttons: `Left` → `Primary`, `Right` → `Secondary`, `Middle`,
  `Back`, `Forward` → the same, `Other(n)` → `Other(n)`.
- **PLATFORM-WINIT-09:** Positions: a physical cursor position `(x, y)` becomes the logical
  `Point(x / s, y / s)` with the window's scale factor `s`. `CursorMoved` becomes `PointerMoved`
  and records the position; `MouseInput` becomes `PointerButton` and `MouseWheel` becomes `Wheel`,
  both at the last recorded position of that window (the origin before any move).
  `CursorEntered`/`CursorLeft` become `PointerEntered`/`PointerLeft`. All carry
  `PointerId::Mouse`.
- **PLATFORM-WINIT-10:** Wheel deltas: winit's sign convention is reversed to Tantu's (positive
  `y` scrolls down): `LineDelta(x, y)` → `Lines { x: -x, y: -y }`, and `PixelDelta(p)` →
  `Pixels(Vec2(-p.x / s, -p.y / s))` in logical pixels.
- **PLATFORM-WINIT-11:** `CloseRequested`, `Resized(size)` (as `PhysicalSize`),
  `ScaleFactorChanged` (as `f32`), `RedrawRequested`, `Focused(b)` and `ModifiersChanged` map to
  the matching `WindowEvent`. Other winit events (IME, touch, file drops, theme, ...) produce no
  event in Phase 1.
- **PLATFORM-WINIT-12:** Attributes: the winit window attributes built from `WindowAttributes`
  have the same title, `LogicalSize` inner size, `LogicalSize` minimum size (or none), resizable
  and visible flags.

## Testing

Conversion rules (05–12) are unit tests in `src/` that build winit values directly; they need no
display. Rules 01–04 need a real event loop on the main thread, which the Rust test harness
doesn't provide, so they are checked by `tests/window.rs`, a test target with `harness = false`.
It runs only when `TANTU_WINDOW_TESTS=1` is set (CI runners have no usable display); otherwise
it prints that it was skipped. With a display it opens a window, checks the first redraw, size,
scale factor and surface handles, requests a redraw, sets the title, closes the window and exits.

## Performance and allocation

Converting an event allocates only for key text and character keys.

## Open questions

Resolved (2026-10-06, decided in autopilot at the user's request; review these):


1. **winit 0.30 vs 0.31.** 0.30.13, the current stable release. 0.31 is in beta; move
   when it is released (its API changes are mostly in the event loop model).
2. **Window tests in CI.** opt-in (`TANTU_WINDOW_TESTS=1`), run by hand on each OS
   before the Phase 1 milestone. Running them in CI would need Xvfb on Linux and a GUI session on
   macOS runners; revisit if regressions slip through.
3. **Synthetic key events dropped.** drop them, so a key held while the window gains
   focus doesn't type. Modifier state still arrives through `ModifiersChanged`.
