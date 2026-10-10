# The `tantu` facade and app runner

- **Status:** Implemented (the user said "continue" on the draft, taking its proposals)
- **Crate:** `tantu`
- **Plan item:** Phase 2, `tantu` facade → "`tantu` facade"
- **Related:** [ADR 0004](../../adr/0004-platform-trait.md), [ADR 0011](../../adr/0011-view-layer.md),
  [platform](../platform/platform.md), [frame](../view/frame.md), [events](../view/events.md),
  [shared text styles](../text/styles.md), [renderer](../scene/renderer.md)

## Purpose

The crate apps depend on. It re-exports what app code needs through `tantu::prelude` and
provides `App`, which opens windows, runs one `ViewTree` per window and connects the platform,
the text system and a renderer:

- platform pointer events go to the tree's event dispatch;
- signal changes request a redraw;
- each redraw runs a frame (apply updates, layout, paint) and renders the Scene.

Running the AGENTS.md counter snippet is the goal.

## Scope

In scope:

- `App` (builder, `run`, `run_with`), `Window` (builder), `Error`, `Result`, `prelude`.
- The runner loop: window creation, redraw scheduling, resize and scale changes, pointer event
  conversion, closing, rendering and render errors.

Out of scope:

- Keyboard input to widgets: Phase 3 focus.
- Cursor shapes: the platform has no cursor API yet (decision 28). `ViewTree::cursor` exists
  but isn't applied.
- Accessibility: `tantu-a11y`, Phase 3.
- A windowed software renderer (it needs a way to present pixels, e.g. `softbuffer`). Here
  `soft` only serves `run_with`.
- Animation frames: a frame is drawn only when something changed, or the platform asks.

## Public API

```rust
/// Errors from running an app.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The platform failed (event loop or window creation).
    Platform(tantu_platform::PlatformError),
    /// Creating a renderer failed.
    Renderer(Box<dyn std::error::Error + Send + Sync>),
    /// Rendering a frame failed (other than a lost target, which is retried).
    Render(tantu_scene::RenderError),
    /// `run` was called without a platform or GPU renderer compiled in (features `winit`, `wgpu`).
    Unsupported(&'static str),
}

/// `std::result::Result<T, tantu::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// A window to open: title and initial logical size (default 800 × 600).
pub struct Window { /* WindowAttributes */ }

impl Window {
    pub fn new(title: impl Into<String>) -> Self;
    pub fn size(self, width: f32, height: f32) -> Self;
    pub fn min_size(self, width: f32, height: f32) -> Self;
    pub fn resizable(self, resizable: bool) -> Self;
}

/// An app: fonts and windows, then `run`.
pub struct App { /* ... */ }

impl App {
    /// An app with no windows; the text system uses system fonts.
    pub fn new() -> Self;
    /// Opens `window` showing the view `content` builds. `content` runs once, when the window
    /// opens, with the window's reactive runtime current (ADR 0011).
    pub fn window<V: View>(self, window: Window, content: impl FnOnce() -> V + 'static) -> Self;
    /// Registers a font file (e.g. bundled with the app); the first one's family becomes the
    /// default family (FACADE-APP-09).
    pub fn font(self, data: Vec<u8>) -> Self;
    /// Uses only registered fonts, not the system's (tests, reproducible output).
    pub fn without_system_fonts(self) -> Self;
    /// Runs on winit with the wgpu renderer until the last window closes.
    pub fn run(self) -> Result<()>;
    /// Runs on `platform`, with `renderer` called once per window to create its renderer
    /// (for tests: `FakePlatform` and the headless renderer; or another shell).
    pub fn run_with(
        self,
        platform: impl Platform,
        renderer: impl FnMut(&mut dyn PlatformContext, WindowId) -> Result<Box<dyn Renderer>> + 'static,
    ) -> Result<()>;
    /// The runner as a platform handler, for shells that drive handlers themselves (and for
    /// `FakePlatform::run_logged` in tests). `run_with` is `platform.run(&mut handler)`
    /// followed by `handler.finish()`.
    pub fn handler(
        self,
        renderer: impl FnMut(&mut dyn PlatformContext, WindowId) -> Result<Box<dyn Renderer>> + 'static,
    ) -> AppHandler;
}

/// The app runner (a `PlatformHandler`).
pub struct AppHandler { /* ... */ }

impl PlatformHandler for AppHandler { /* started, window_event, idle */ }

impl AppHandler {
    /// How the run ended: the first error, or `Ok(())`.
    pub fn finish(self) -> Result<()>;
}

/// What app code imports: `use tantu::prelude::*;`
pub mod prelude {
    pub use crate::{App, Window, Result};
    pub use tantu_core::{Color, EdgeInsets, Point, Size};
    pub use tantu_reactive::{signal, memo, effect, batch, Signal, Memo};
    pub use tantu_layout::{Alignment, MainAxisAlignment, MainAxisSize, CrossAxisAlignment, StackFit};
    pub use tantu_text::TextStyle;
    pub use tantu_view::{View, AnyView, IntoProp, Dyn, Show, For};
    pub use tantu_widgets::*; // Text, Button, Row, Column, Padding, ...
}
```

The crate also re-exports the member crates as modules (`tantu::core`, `tantu::reactive`,
`tantu::scene`, `tantu::text`, `tantu::layout`, `tantu::view`, `tantu::widgets`,
`tantu::platform`), so app code depends on `tantu` alone.

## Behavior

Tests drive `App::handler` with `FakePlatform::run_logged` and the headless renderer.
FACADE-APP-08's test runs only without the `winit` or `wgpu` feature, so CI runs
`cargo test -p tantu --no-default-features` too.

- **FACADE-APP-01:** At `started`, each window is created in the order given, with its title and
  size. Its view tree is built (sharing the app text system's style table), and its renderer
  is created and resized to the window's physical size and scale factor. Then a redraw is
  requested. An app with no windows exits at once. A failing window or renderer creation ends
  the run with `Error::Platform` or the renderer's error.
- **FACADE-APP-02:** On `RedrawRequested`, the runner runs one frame, with constraints tight to
  the window's logical size, the app's text system and the window's `Resources`. It then
  renders the Scene. The first redraw shows the content: the headless frame holds its glyph
  runs and shapes.
- **FACADE-APP-03:** After each event and at `idle`, every window whose tree `needs_frame()` gets
  `request_redraw`. A signal written by a handler therefore leads to one more frame showing the
  change, and a window with nothing changed isn't redrawn.
- **FACADE-APP-04:** `Resized` and `ScaleFactorChanged` resize the renderer and request a redraw.
  The next frame lays out at the new logical size.
- **FACADE-APP-05:** Pointer events are converted and dispatched to the window's tree:
  - `PointerMoved` becomes `Move`, and `PointerLeft` becomes `Leave`.
  - `PointerButton` becomes `Down` or `Up`, with the same button. Back and Forward become
    `Other(3)` and `Other(4)`.
  - `Wheel` becomes `Scroll`. Pixel deltas pass through, and line deltas count 40 logical px
    per line.
  - Positions stay logical.
  - A press on a `Button` followed by a release over it calls `on_press`.
- **FACADE-APP-06:** `CloseRequested` closes that window and drops its tree, which disposes its
  scope. When the last window closes, the runner exits and `run_with` returns `Ok(())`.
- **FACADE-APP-07:** A render result of `Err(TargetLost)` requests another redraw. Any other render
  error ends the run with `Error::Render`. Report counts (missing fonts and so on) are logged
  with `tracing`, never errors.
- **FACADE-APP-08:** `run` without both the `winit` and `wgpu` features returns
  `Error::Unsupported` and opens nothing.
- **FACADE-APP-09:** The family of the first font registered with `App::font` becomes the text
  system's default family. Registered fonts are then used without being named, and apps
  that turn off system fonts still show text. With no registered font, the default stays
  `SansSerif`. (Added while agreeing: without it, generic families find no registered font.)

## Performance and allocation

One `Scene` per window, reused across frames. Frames happen only on request.

## Open questions

Resolved (2026-10-10; the user said "continue" on the draft):

1. **Redraws are scheduled by polling `needs_frame()`** after events and at idle.
2. **40 px per wheel line.**
3. **`run` is GPU-only in Phase 2.**
4. **One `TextSystem` and one `Resources` per app.**

Added while agreeing (review):

- `App::handler` and `AppHandler`, so tests can read `FakePlatform`'s log and other shells can
  drive the runner. `run_with` is built on them.
- FACADE-APP-09: the first registered font is the default family.
