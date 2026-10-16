//! The [`Platform`], [`PlatformHandler`] and [`PlatformContext`] traits.

use std::fmt;
use std::sync::Arc;

use tantu_core::{Point, Size};

use crate::types::{CursorIcon, PhysicalSize, WindowAttributes, WindowEvent, WindowId};

/// Why a platform operation failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum PlatformError {
    /// The platform can't do this (e.g. no display, unsupported feature).
    Unsupported(String),
    /// The OS or windowing library reported an error.
    Os(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::Unsupported(what) => write!(f, "not supported by this platform: {what}"),
            PlatformError::Os(e) => write!(f, "platform error: {e}"),
        }
    }
}

impl std::error::Error for PlatformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PlatformError::Os(e) => Some(e.as_ref()),
            PlatformError::Unsupported(_) => None,
        }
    }
}

/// Window and display handles a GPU renderer can create a surface from.
pub trait WindowHandles:
    raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync
{
}

impl<T> WindowHandles for T where
    T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync
{
}

/// A shared handle to a window's handles. Keeps the native window alive while held.
pub type SurfaceTarget = Arc<dyn WindowHandles>;

/// A clipboard image: straight-alpha RGBA8 rows, top to bottom (PLATFORM-CLIP-02).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipboardImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width · height · 4` bytes.
    pub rgba: Vec<u8>,
}

/// The OS clipboard: plain text (PLATFORM-CLIP-01), and images, HTML and the primary
/// selection (PLATFORM-CLIP-02), whose methods have defaults so implementations that only
/// handle text keep working.
pub trait Clipboard {
    /// The clipboard's text, or `None` when it is empty, holds no text or can't be read.
    fn read_text(&self) -> Option<String>;
    /// Replaces the clipboard's content with `text`; false when it couldn't be written.
    fn write_text(&self, text: &str) -> bool;
    /// The clipboard's image, if it holds one. Default: none.
    fn read_image(&self) -> Option<ClipboardImage> {
        None
    }
    /// Puts an image on the clipboard. Default: unsupported (false).
    fn write_image(&self, image: &ClipboardImage) -> bool {
        let _ = image;
        false
    }
    /// Puts HTML on the clipboard, with `alt_text` for apps that take plain text. Default:
    /// writes `alt_text` only.
    fn write_html(&self, html: &str, alt_text: &str) -> bool {
        let _ = html;
        self.write_text(alt_text)
    }
    /// The clipboard's HTML, if it holds any. Default: none.
    fn read_html(&self) -> Option<String> {
        None
    }
    /// The primary selection (X11 and Wayland); `None` where there is none. Default: none.
    fn read_primary(&self) -> Option<String> {
        None
    }
    /// Sets the primary selection; false where there is none. Default: false.
    fn write_primary(&self, text: &str) -> bool {
        let _ = text;
        false
    }
}

/// What a handler can ask the platform to do while it is called.
pub trait PlatformContext {
    /// Creates a window. A `RedrawRequested` for it follows; read its initial size and scale
    /// factor with [`inner_size`](Self::inner_size) and [`scale_factor`](Self::scale_factor)
    /// (`Resized` and `ScaleFactorChanged` only report changes).
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
    /// windows such as [`FakePlatform`](crate::FakePlatform)).
    fn surface_target(&self, window: WindowId) -> Option<SurfaceTarget>;

    /// Lets the window receive IME events (off by default) (PLATFORM-IME-01).
    fn set_ime_allowed(&mut self, window: WindowId, allowed: bool);

    /// Where the caret is, in logical pixels, so the IME can place its candidate window.
    fn set_ime_cursor_area(&mut self, window: WindowId, origin: Point, size: Size);

    /// Shows `cursor` while the pointer is over `window`, until the next call. A new window
    /// starts with [`CursorIcon::Default`]; unknown windows are ignored (PLATFORM-CURSOR-01).
    fn set_cursor(&mut self, window: WindowId, cursor: CursorIcon);

    /// The clipboard, or `None` on a platform without one; the same handle for every call
    /// (PLATFORM-CLIP-01).
    fn clipboard(&mut self) -> Option<std::rc::Rc<dyn Clipboard>>;

    /// Opens a file dialog for `window`; the result arrives later as
    /// `WindowEvent::FileDialogClosed` (PLATFORM-NATIVE-01). `None`: not supported (the
    /// default).
    fn file_dialog(
        &mut self,
        window: WindowId,
        request: crate::FileDialogRequest,
    ) -> Option<crate::DialogId> {
        let _ = (window, request);
        None
    }

    /// Opens a message dialog for `window`; the result arrives later as
    /// `WindowEvent::MessageDialogClosed`. `None`: not supported (the default).
    fn message_dialog(
        &mut self,
        window: WindowId,
        request: crate::MessageRequest,
    ) -> Option<crate::DialogId> {
        let _ = (window, request);
        None
    }

    /// Shows a notification; false when not supported (the default) or it failed.
    fn notify(&mut self, notification: &crate::Notification) -> bool {
        let _ = notification;
        false
    }

    /// Shows `menu` as `window`'s menu bar (`None` removes it); macOS has one bar for the
    /// app, owned by the last window to set it. False when not supported (the default)
    /// (PLATFORM-MENU-01).
    fn set_menu_bar(&mut self, window: WindowId, menu: Option<crate::NativeMenu>) -> bool {
        let _ = (window, menu);
        false
    }

    /// Shows (or replaces, or with `None` removes) the app's tray icon; false when not
    /// supported (the default) (PLATFORM-TRAY-01).
    fn set_tray(&mut self, tray: Option<crate::Tray>) -> bool {
        let _ = tray;
        todo!("PLATFORM-TRAY-01")
    }

    /// What this platform supports (default: nothing).
    fn native_capabilities(&self) -> crate::NativeCapabilities {
        crate::NativeCapabilities::default()
    }

    /// Ends the event loop after the current callback returns. [`Platform::run`] then returns.
    fn exit(&mut self);

    /// Sends a tree update to the window's assistive technology (ignored while inactive or
    /// for an unknown window) (PLATFORM-A11Y-01).
    fn update_accessibility(&mut self, window: WindowId, update: accesskit::TreeUpdate);

    /// The platform's monotonic clock (PLATFORM-TIME-01).
    fn now(&self) -> std::time::Instant;

    /// Asks for an `idle` call at `deadline` (or earlier) even if no event arrives; `None`
    /// cancels. Each call replaces the previous deadline (PLATFORM-TIME-01).
    fn wake_at(&mut self, deadline: Option<std::time::Instant>);
}

/// The app side of the event loop.
pub trait PlatformHandler {
    /// Called once, first, when windows can be created.
    fn started(&mut self, cx: &mut dyn PlatformContext);

    /// Called for each event of each window.
    fn window_event(&mut self, cx: &mut dyn PlatformContext, window: WindowId, event: WindowEvent);

    /// Called when pending events have been handled and the platform is about to wait.
    fn idle(&mut self, cx: &mut dyn PlatformContext) {
        let _ = cx;
    }

    /// Called for an app-level event, such as the tray's (PLATFORM-TRAY-01). Default:
    /// ignored.
    fn app_event(&mut self, cx: &mut dyn PlatformContext, event: crate::AppEvent) {
        let _ = (cx, event);
    }
}

/// A platform shell: owns the event loop and calls the handler.
pub trait Platform {
    /// Runs the event loop until [`PlatformContext::exit`], calling `handler`. Blocks.
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError>
    where
        Self: Sized;
}
