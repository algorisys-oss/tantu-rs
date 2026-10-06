//! Window and input value types.

use tantu_core::{Point, Size, Vec2};

/// Identifies a window within one running platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(u64);

impl WindowId {
    /// From a raw value.
    pub const fn from_raw(raw: u64) -> WindowId {
        todo!()
    }

    /// The raw value.
    pub const fn to_raw(self) -> u64 {
        todo!()
    }
}

/// A size in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PhysicalSize {
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
}

impl PhysicalSize {
    /// A `width × height` size.
    pub const fn new(width: u32, height: u32) -> PhysicalSize {
        PhysicalSize { width, height }
    }

    /// `width / scale_factor` × `height / scale_factor` logical pixels. A scale factor that is
    /// not finite or not positive counts as 1.
    pub fn to_logical(self, scale_factor: f32) -> Size {
        todo!()
    }

    /// `size × scale_factor`, rounded to whole pixels and clamped to `0..=u32::MAX` (negative
    /// and NaN give 0). A scale factor that is not finite or not positive counts as 1.
    pub fn from_logical(size: Size, scale_factor: f32) -> PhysicalSize {
        todo!()
    }
}

/// How to create a window. Builder methods take `self` and return `Self`.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowAttributes {
    /// The title bar text.
    pub title: String,
    /// Initial inner size in logical pixels.
    pub size: Size,
    /// Smallest inner size in logical pixels, if limited.
    pub min_size: Option<Size>,
    /// Whether the user can resize the window.
    pub resizable: bool,
    /// Whether the window is shown when created.
    pub visible: bool,
}

impl WindowAttributes {
    /// Title `title`, size 800 × 600, no minimum size, resizable, visible.
    pub fn new(title: impl Into<String>) -> WindowAttributes {
        todo!()
    }

    /// Sets the initial inner size in logical pixels.
    pub fn size(self, width: f32, height: f32) -> Self {
        todo!()
    }

    /// Sets the smallest inner size in logical pixels.
    pub fn min_size(self, width: f32, height: f32) -> Self {
        todo!()
    }

    /// Sets whether the user can resize the window.
    pub fn resizable(self, resizable: bool) -> Self {
        todo!()
    }

    /// Sets whether the window is shown when created.
    pub fn visible(self, visible: bool) -> Self {
        todo!()
    }
}

impl Default for WindowAttributes {
    fn default() -> Self {
        WindowAttributes::new("Tantu")
    }
}

/// Which pointer an event comes from. Only the mouse in Phase 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PointerId {
    /// The mouse (or a trackpad driving the cursor).
    Mouse,
}

/// A pointer button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// The main button (usually left).
    Primary,
    /// The secondary button (usually right).
    Secondary,
    /// The middle button (often the wheel).
    Middle,
    /// The "back" side button.
    Back,
    /// The "forward" side button.
    Forward,
    /// Any other button, by the platform's number.
    Other(u16),
}

/// Pressed or released (pointer buttons and keys).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonState {
    /// Went down.
    Pressed,
    /// Went up.
    Released,
}

/// How far a wheel or trackpad scrolled. Positive `y` scrolls down (towards later content),
/// positive `x` scrolls right, as in Flutter and the DOM.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDelta {
    /// In lines (mouse wheels with notches).
    Lines {
        /// Horizontal lines.
        x: f32,
        /// Vertical lines.
        y: f32,
    },
    /// In logical pixels (trackpads, smooth wheels).
    Pixels(Vec2),
}

/// Modifier keys held down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// Shift.
    pub shift: bool,
    /// Control (Ctrl).
    pub control: bool,
    /// Alt (Option on macOS).
    pub alt: bool,
    /// Super (Windows key, Command on macOS).
    pub super_key: bool,
}

impl Modifiers {
    /// No modifier held.
    pub const NONE: Modifiers = Modifiers {
        shift: false,
        control: false,
        alt: false,
        super_key: false,
    };

    /// True if no modifier is held.
    pub fn is_empty(self) -> bool {
        todo!()
    }
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
#[allow(missing_docs)] // The names are the W3C key names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NamedKey {
    Enter,
    Tab,
    Backspace,
    Delete,
    Escape,
    Insert,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    Shift,
    Control,
    Alt,
    Super,
    CapsLock,
    ContextMenu,
    PrintScreen,
    Pause,
    NumLock,
    ScrollLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
}

/// Where on the keyboard a key is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KeyLocation {
    /// The only key of its kind, or the main one.
    #[default]
    Standard,
    /// The left one of a pair (left Shift).
    Left,
    /// The right one of a pair (right Shift).
    Right,
    /// On the numeric keypad.
    Numpad,
}

/// A key press or release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// What the key means.
    pub key: Key,
    /// Where it is.
    pub location: KeyLocation,
    /// Pressed or released.
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
    /// A pointer entered the window.
    PointerEntered {
        /// Which pointer.
        pointer: PointerId,
    },
    /// A pointer left the window.
    PointerLeft {
        /// Which pointer.
        pointer: PointerId,
    },
    /// A pointer moved.
    PointerMoved {
        /// Which pointer.
        pointer: PointerId,
        /// Where it is now.
        position: Point,
    },
    /// A pointer button was pressed or released.
    PointerButton {
        /// Which pointer.
        pointer: PointerId,
        /// Which button.
        button: PointerButton,
        /// Pressed or released.
        state: ButtonState,
        /// Where the pointer is.
        position: Point,
    },
    /// A wheel or trackpad scrolled.
    Wheel {
        /// Which pointer.
        pointer: PointerId,
        /// How far.
        delta: ScrollDelta,
        /// Where the pointer is.
        position: Point,
    },
    /// A key was pressed or released.
    Keyboard(KeyEvent),
    /// The held modifier keys changed.
    ModifiersChanged(Modifiers),
}
