# Platform trait and window/input events

- **Status:** Implemented
- **Crate:** `tantu-platform`
- **Plan item:** Phase 1, "`tantu-platform`: `Platform` trait, window/input event types,
  `FakePlatform` for tests"
- **Related:** [ADR 0004](../../adr/0004-platform-trait-with-winit-by-default.md) (Platform trait,
  winit by default), [geometry](../core/geometry.md), [winit shell](../platform-winit/shell.md)

## Purpose

The boundary between Tantu and the operating system. A platform shell (winit by default) owns the
event loop, creates windows and turns OS input into Tantu's own event types; the app runner
(`tantu` facade, Phase 2) implements `PlatformHandler` and reacts. Nothing above this crate sees
winit types, so other shells (web, mobile, embedding) can be added later without touching the UI
crates.

This spec covers Phase 1: windows, resize, DPI, redraw, pointer (mouse), wheel and keyboard. It
also defines `FakePlatform`, a scripted shell for testing the app runner without a display.

## Scope

In scope:

- `Platform`, `PlatformHandler` and `PlatformContext`: run the loop, create and close windows,
  request redraws, read size and scale factor, set the title, get a handle renderers can draw
  into, exit.
- Event types: `WindowEvent` (close requested, resized, scale factor changed, redraw requested,
  focus, pointer entered/left/moved/button, wheel, keyboard, modifiers).
- Value types: `WindowId`, `PhysicalSize`, `WindowAttributes`, `PointerId`, `PointerButton`,
  `ButtonState`, `ScrollDelta`, `Modifiers`, `Key`, `NamedKey`, `KeyLocation`, `KeyEvent`.
- `SurfaceTarget`: the window and display handles (`raw-window-handle` 0.6) a GPU renderer needs.
- `FakePlatform`: runs a handler against a script of events, records what the handler asked for.

Out of scope (and where it goes):

- IME, clipboard, drag and drop, cursor icons, touch and pen, physical key codes, menus,
  dialogs, tray, AccessKit adapter: later items (ADR 0004 lists them; Phase 2–4).
- Frame scheduling and dispatch to widgets: the app runner and `tantu-view` (Phase 2).
- Presenting software-rendered pixels to a window: decided with the app runner.

## Dependencies

- `tantu-core` (geometry).
- `raw-window-handle` 0.6 (MIT OR Apache-2.0 OR Zlib, MSRV 1.64): the standard handle traits
  wgpu, softbuffer and AccessKit accept. Re-exported as `tantu_platform::raw_window_handle`.

## Public API

Crate root `tantu_platform`.

```rust
use tantu_core::{Point, Size, Vec2};

/// Identifies a window within one running platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(/* u64 */);
impl WindowId {
    pub const fn from_raw(raw: u64) -> WindowId;
    pub const fn to_raw(self) -> u64;
}

/// A size in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PhysicalSize { pub width: u32, pub height: u32 }
impl PhysicalSize {
    pub const fn new(width: u32, height: u32) -> PhysicalSize;
    /// `width / scale_factor` × `height / scale_factor` logical pixels.
    pub fn to_logical(self, scale_factor: f32) -> Size;
    /// `size × scale_factor`, rounded to whole pixels.
    pub fn from_logical(size: Size, scale_factor: f32) -> PhysicalSize;
}

/// How to create a window. Builder methods take `self` and return `Self`.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowAttributes {
    pub title: String,
    /// Initial inner size in logical pixels.
    pub size: Size,
    /// Smallest inner size in logical pixels, if limited.
    pub min_size: Option<Size>,
    pub resizable: bool,
    pub visible: bool,
}
impl WindowAttributes {
    /// Title `title`, size 800 × 600, no minimum size, resizable, visible.
    pub fn new(title: impl Into<String>) -> WindowAttributes;
    pub fn size(self, width: f32, height: f32) -> Self;
    pub fn min_size(self, width: f32, height: f32) -> Self;
    pub fn resizable(self, resizable: bool) -> Self;
    pub fn visible(self, visible: bool) -> Self;
}
impl Default for WindowAttributes { /* new("Tantu") */ }

/// Which pointer an event comes from. Only the mouse in Phase 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PointerId { Mouse }

/// A pointer button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton { Primary, Secondary, Middle, Back, Forward, Other(u16) }

/// Pressed or released (pointer buttons and keys).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonState { Pressed, Released }

/// How far a wheel or trackpad scrolled. Positive `y` scrolls down (towards later content),
/// positive `x` scrolls right, as in Flutter and the DOM.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDelta {
    /// In lines (mouse wheels with notches).
    Lines { x: f32, y: f32 },
    /// In logical pixels (trackpads, smooth wheels).
    Pixels(Vec2),
}

/// Modifier keys held down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers { pub shift: bool, pub control: bool, pub alt: bool, pub super_key: bool }
impl Modifiers {
    /// No modifier held.
    pub const NONE: Modifiers;
    /// True if no modifier is held.
    pub fn is_empty(self) -> bool;
}

/// What a key means, after the keyboard layout (W3C `KeyboardEvent.key`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// A key with a name (Enter, arrows, F1, Shift, ...).
    Named(NamedKey),
    /// A key that produces a character, as the layout gives it (`"a"`, `"A"`, `"é"`, `" "`).
    Character(String),
    /// The platform couldn't tell.
    Unidentified,
}

/// Named keys (a subset of W3C named key values; more are added as needed). The space bar is
/// `Key::Character(" ")`, as in W3C.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NamedKey {
    Enter, Tab, Backspace, Delete, Escape, Insert,
    ArrowLeft, ArrowRight, ArrowUp, ArrowDown, Home, End, PageUp, PageDown,
    Shift, Control, Alt, Super, CapsLock,
    ContextMenu, PrintScreen, Pause, NumLock, ScrollLock,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
}

/// Where on the keyboard a key is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeyLocation { #[default] Standard, Left, Right, Numpad }

/// A key press or release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub location: KeyLocation,
    pub state: ButtonState,
    /// True for automatic repeats while the key is held.
    pub repeat: bool,
    /// Text the key produced, if any (for text input until IME arrives).
    pub text: Option<String>,
}

/// Something that happened to a window. Positions are in logical pixels relative to the
/// window's inner top-left corner.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum WindowEvent {
    /// The user asked to close the window. Nothing closes until the handler calls
    /// `PlatformContext::close_window`.
    CloseRequested,
    /// The inner size changed (physical pixels).
    Resized(PhysicalSize),
    /// The scale factor (physical pixels per logical pixel) changed.
    ScaleFactorChanged(f32),
    /// Time to draw a frame (after `request_redraw`, or when the OS needs one).
    RedrawRequested,
    /// The window gained (true) or lost (false) keyboard focus.
    Focused(bool),
    PointerEntered { pointer: PointerId },
    PointerLeft { pointer: PointerId },
    PointerMoved { pointer: PointerId, position: Point },
    PointerButton { pointer: PointerId, button: PointerButton, state: ButtonState, position: Point },
    Wheel { pointer: PointerId, delta: ScrollDelta, position: Point },
    Keyboard(KeyEvent),
    ModifiersChanged(Modifiers),
}

/// Why a platform operation failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum PlatformError {
    /// The platform can't do this (e.g. no display, unsupported feature).
    Unsupported(String),
    /// The OS or windowing library reported an error.
    Os(Box<dyn std::error::Error + Send + Sync>),
}
impl std::fmt::Display for PlatformError { /* ... */ }
impl std::error::Error for PlatformError { /* source() is the Os error */ }

/// Window and display handles a GPU renderer can create a surface from.
pub trait WindowHandles: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync {}
impl<T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync> WindowHandles for T {}

/// A shared handle to a window's handles. Keeps the native window alive while held.
pub type SurfaceTarget = std::sync::Arc<dyn WindowHandles>;

/// What a handler can ask the platform to do while it is called.
pub trait PlatformContext {
    /// Creates a window. A `RedrawRequested` for it follows; read its initial size and scale factor
    /// with `inner_size` and `scale_factor` (`Resized` and `ScaleFactorChanged` only report
    /// changes).
    fn create_window(&mut self, attributes: &WindowAttributes) -> Result<WindowId, PlatformError>;
    /// Closes a window. Unknown or already closed ids are ignored.
    fn close_window(&mut self, window: WindowId);
    /// Asks for a `RedrawRequested` event for `window`, delivered later. Several requests before
    /// it arrives produce one event.
    fn request_redraw(&mut self, window: WindowId);
    /// Current inner size, or `None` for an unknown window.
    fn inner_size(&self, window: WindowId) -> Option<PhysicalSize>;
    /// Current scale factor, or `None` for an unknown window.
    fn scale_factor(&self, window: WindowId) -> Option<f32>;
    /// Sets the window title. Unknown windows are ignored.
    fn set_title(&mut self, window: WindowId, title: &str);
    /// Handles a renderer can draw into, or `None` (unknown window, or a platform without native
    /// windows such as `FakePlatform`).
    fn surface_target(&self, window: WindowId) -> Option<SurfaceTarget>;
    /// Ends the event loop after the current callback returns. `Platform::run` then returns.
    fn exit(&mut self);
}

/// The app side of the event loop.
pub trait PlatformHandler {
    /// Called once, first, when windows can be created.
    fn started(&mut self, cx: &mut dyn PlatformContext);
    /// Called for each event of each window.
    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent);
    /// Called when pending events have been handled and the platform is about to wait.
    fn idle(&mut self, cx: &mut dyn PlatformContext) { let _ = cx; }
}

/// A platform shell: owns the event loop and calls the handler.
pub trait Platform {
    /// Runs the event loop until `exit`, calling `handler`. Blocks.
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError> where Self: Sized;
}

/// One step of a `FakePlatform` script.
#[derive(Clone, Debug, PartialEq)]
pub enum FakeStep {
    /// Deliver `event` to the `window`-th window created (0 = the first).
    Event { window: usize, event: WindowEvent },
    /// Deliver nothing; just let pending redraws and `idle` run.
    Idle,
}

/// What the handler asked a `FakePlatform` for.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FakeLog {
    /// Every window created, in creation order.
    pub windows: Vec<FakeWindow>,
    /// Whether the handler called `exit`.
    pub exited: bool,
    /// How many script steps were delivered (skipped steps not counted).
    pub steps_delivered: usize,
}

/// A window of a `FakePlatform`, as last seen.
#[derive(Clone, Debug, PartialEq)]
pub struct FakeWindow {
    pub id: WindowId,
    /// The attributes, with the title as last set.
    pub attributes: WindowAttributes,
    pub size: PhysicalSize,
    pub scale_factor: f32,
    pub closed: bool,
    /// `RedrawRequested` events delivered to it.
    pub redraws: u32,
}

/// A platform without a display: runs a handler against a script of events.
#[derive(Clone, Debug, Default)]
pub struct FakePlatform { /* private */ }
impl FakePlatform {
    /// An empty script; new windows get scale factor 1.
    pub fn new() -> FakePlatform;
    /// The scale factor new windows get.
    pub fn scale_factor(self, scale_factor: f32) -> Self;
    /// Appends a step.
    pub fn step(self, step: FakeStep) -> Self;
    /// Appends `Event { window, event }`.
    pub fn event(self, window: usize, event: WindowEvent) -> Self;
    /// Runs like `Platform::run` and returns the log.
    pub fn run_logged(self, handler: &mut dyn PlatformHandler) -> FakeLog;
}
impl Platform for FakePlatform { /* run_logged, then Ok(()) */ }
```

## Behavior

Value types

- **PLATFORM-TYPES-01:** `WindowId::from_raw(x).to_raw() == x` for every `u64`.
- **PLATFORM-TYPES-02:** `PhysicalSize::to_logical(s)` is `(width / s, height / s)`;
  `from_logical(size, s)` rounds `size × s` to the nearest integer, clamped to `0..=u32::MAX`
  (negative and NaN give 0). A scale factor that is not finite or not positive counts as 1.
- **PLATFORM-TYPES-03:** `WindowAttributes::new(t)` has title `t`, size 800 × 600, no minimum
  size, resizable and visible; `default()` is `new("Tantu")`. Each builder method sets exactly
  its field.
- **PLATFORM-TYPES-04:** `Modifiers::NONE` and `Modifiers::default()` have every flag false and
  `is_empty()` true; any flag set makes `is_empty()` false.
- **PLATFORM-TYPES-05:** `PlatformError` has a non-empty `Display`; `Os(e)` returns `e` from
  `source()`, `Unsupported` returns `None`.
- **PLATFORM-TYPES-06:** `Platform` is implementable outside this crate and `PlatformHandler` /
  `PlatformContext` are object-safe (`&mut dyn`), so the app runner can be written once against
  them. Event and value types are `Send + Sync`.

FakePlatform

- **PLATFORM-FAKE-01:** `run_logged` calls `started` exactly once, before anything else, then
  delivers the script steps in order. `run` does the same and returns `Ok(())`.
- **PLATFORM-FAKE-02:** `create_window` returns a new, distinct `WindowId` each call and records
  the window with its attributes, scale factor (the platform's), size
  `PhysicalSize::from_logical(attributes.size, scale)`, not closed, 0 redraws. `inner_size` and
  `scale_factor` return those values. A new window has a redraw pending (as if
  `request_redraw` had been called).
- **PLATFORM-FAKE-03:** An `Event { window: i, .. }` step goes to the `i`-th created window. If
  there is no such window yet, or it was closed, the step is skipped (not delivered, not counted).
- **PLATFORM-FAKE-04:** Delivering `Resized(s)` sets the window's size to `s` before the handler
  sees it; `ScaleFactorChanged(f)` likewise sets its scale factor.
- **PLATFORM-FAKE-05:** After each step (and after `started`), every window with a pending
  `request_redraw` gets one `RedrawRequested`, in the order of first request, and its `redraws`
  count goes up. Requests made while these are delivered wait for the next step. After the last
  step there is one more such round. Requests for unknown or closed windows are ignored.
- **PLATFORM-FAKE-06:** After each step's redraw round, `idle` is called once.
- **PLATFORM-FAKE-07:** `close_window` marks the window closed: later steps for it are skipped,
  its pending redraw is dropped, `inner_size` and `scale_factor` return `None`. A scripted
  `CloseRequested` closes nothing by itself.
- **PLATFORM-FAKE-08:** After `exit`, no further callbacks happen (no steps, redraws or `idle`);
  `run_logged` returns with `exited: true`.
- **PLATFORM-FAKE-09:** `set_title` changes the recorded title; `surface_target` is always `None`.
- **PLATFORM-FAKE-10:** `steps_delivered` counts delivered `Event` steps and `Idle` steps.

## Performance and allocation

None beyond the general rules in AGENTS.md. Events are small values; `KeyEvent` may allocate for
its strings, once per key press.

## Open questions

Resolved (2026-10-06, decided in autopilot at the user's request; review these):


1. **Our own event types vs. re-exporting winit's or `keyboard-types`.** our own, small
   and Flutter/W3C-shaped, so no UI crate depends on a windowing library and other shells can
   produce them. `NamedKey` starts with the keys widgets need and is `#[non_exhaustive]`.
2. **Logical positions in events, physical sizes in `Resized`.** pointer positions are
   logical (layout and hit-testing work in logical pixels); `Resized` is physical because the
   renderer's target is.
3. **`CloseRequested` doesn't close.** the handler decides (unsaved-changes prompts),
   as in winit.
4. **`FakePlatform` in this crate** rather than in `tantu-test`. here, so the facade's
   app runner can be tested in Phase 2 without depending on `tantu-test`.
5. **Scroll direction.** positive `y` scrolls down, as in Flutter and the DOM
   (winit's line deltas use the opposite sign; the shell converts).
