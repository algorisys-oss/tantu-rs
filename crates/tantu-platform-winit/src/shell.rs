//! [`WinitPlatform`]: the event loop, windows and the `PlatformContext` on top of winit.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use tantu_core::{Point, Size};
use tantu_platform::accesskit::TreeUpdate;
use tantu_platform::{
    AccessibilityEvent, PhysicalSize, Platform, PlatformContext, PlatformError, PlatformHandler,
    SurfaceTarget, WindowAttributes, WindowEvent, WindowId,
};
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::Window;

/// The event loop's user event: what the AccessKit adapters send (PLATFORM-WINIT-15) and
/// native dialog results from their worker threads (PLATFORM-WINIT-20).
enum UserEvent {
    Access(accesskit_winit::Event),
    #[cfg_attr(not(feature = "native-dialogs"), allow(dead_code))]
    Window(WindowId, WindowEvent),
    /// A tray event (PLATFORM-WINIT-22).
    #[cfg_attr(
        not(all(feature = "tray", any(target_os = "macos", target_os = "windows"))),
        allow(dead_code)
    )]
    App(tantu_platform::AppEvent),
    /// A native menu item's `muda` id (PLATFORM-WINIT-21).
    #[cfg_attr(
        not(all(
            feature = "native-menus",
            any(target_os = "macos", target_os = "windows")
        )),
        allow(dead_code)
    )]
    Menu(String),
}

impl From<accesskit_winit::Event> for UserEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        UserEvent::Access(event)
    }
}

use crate::conv;

/// The winit platform shell. Create it on the main thread.
pub struct WinitPlatform {
    event_loop: EventLoop<UserEvent>,
}

impl WinitPlatform {
    /// Creates the event loop. Must be called on the main thread (a winit requirement on macOS,
    /// and on Linux). Fails, without panicking, when there is no display or an event loop was
    /// already created in this process.
    pub fn new() -> Result<WinitPlatform, PlatformError> {
        let event_loop = EventLoop::with_user_event()
            .build()
            .map_err(|e| PlatformError::Os(Box::new(e)))?;
        Ok(WinitPlatform { event_loop })
    }
}

impl Platform for WinitPlatform {
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError> {
        let proxy = self.event_loop.create_proxy();
        let mut shell = Shell {
            handler,
            windows: Windows::default(),
            started: false,
            proxy,
        };
        self.event_loop
            .run_app(&mut shell)
            .map_err(|e| PlatformError::Os(Box::new(e)))
    }
}

/// winit's application handler: forwards to the Tantu handler.
struct Shell<'h> {
    handler: &'h mut dyn PlatformHandler,
    windows: Windows,
    started: bool,
    /// For the AccessKit adapters, which send their events through the loop.
    proxy: EventLoopProxy<UserEvent>,
}

impl ApplicationHandler<UserEvent> for Shell<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Desktop platforms resume once; later resumes (mobile) don't start the app again.
        if self.started {
            return;
        }
        self.started = true;
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
            proxy: &self.proxy,
        };
        self.handler.started(&mut cx);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let event = match event {
            UserEvent::Access(event) => event,
            UserEvent::App(event) => {
                let mut cx = Context {
                    event_loop,
                    windows: &mut self.windows,
                    proxy: &self.proxy,
                };
                self.handler.app_event(&mut cx, event);
                return;
            }
            // A native menu item (PLATFORM-WINIT-21).
            UserEvent::Menu(id) => {
                let targets: Vec<(WindowId, WindowEvent)> = match crate::native::parse_menu_id(&id)
                {
                    Some(crate::native::MenuTarget::Item(window, id)) => {
                        vec![(window, WindowEvent::MenuItemActivated { id })]
                    }
                    // Quit asks every window to close, so vetoes apply.
                    Some(crate::native::MenuTarget::Quit) => self
                        .windows
                        .entries
                        .keys()
                        .map(|w| (*w, WindowEvent::CloseRequested))
                        .collect(),
                    // Tray items are app events (PLATFORM-WINIT-22).
                    Some(crate::native::MenuTarget::Tray(id)) => {
                        let mut cx = Context {
                            event_loop,
                            windows: &mut self.windows,
                            proxy: &self.proxy,
                        };
                        self.handler.app_event(
                            &mut cx,
                            tantu_platform::AppEvent::TrayMenuItemActivated { id },
                        );
                        return;
                    }
                    None => Vec::new(),
                };
                for (window, event) in targets {
                    if self.windows.entries.contains_key(&window) {
                        let mut cx = Context {
                            event_loop,
                            windows: &mut self.windows,
                            proxy: &self.proxy,
                        };
                        self.handler.window_event(&mut cx, window, event);
                    }
                }
                return;
            }
            // A dialog result, for a window still open (PLATFORM-WINIT-20).
            UserEvent::Window(id, event) => {
                if self.windows.entries.contains_key(&id) {
                    let mut cx = Context {
                        event_loop,
                        windows: &mut self.windows,
                        proxy: &self.proxy,
                    };
                    self.handler.window_event(&mut cx, id, event);
                }
                return;
            }
        };
        // The adapters' events become accessibility window events (PLATFORM-WINIT-15).
        let Some(&id) = self.windows.by_winit.get(&event.window_id) else {
            return;
        };
        let event = match event.window_event {
            accesskit_winit::WindowEvent::InitialTreeRequested => AccessibilityEvent::Activated,
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                AccessibilityEvent::Action(request)
            }
            accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                AccessibilityEvent::Deactivated
            }
        };
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
            proxy: &self.proxy,
        };
        self.handler
            .window_event(&mut cx, id, WindowEvent::Accessibility(event));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(&id) = self.windows.by_winit.get(&window_id) else {
            return;
        };
        let Some(entry) = self.windows.entries.get_mut(&id) else {
            return;
        };
        // The adapter sees every event first (PLATFORM-WINIT-15).
        entry.adapter.process_event(&entry.window, &event);
        match event {
            winit::event::WindowEvent::CursorEntered { .. } => entry.inside = true,
            winit::event::WindowEvent::CursorLeft { .. } => entry.inside = false,
            _ => {}
        }
        // File events are batched per gesture (PLATFORM-WINIT-18).
        let position = entry.inside.then_some(entry.pointer);
        let (mut events, absorbed) = entry.drops.feed(&event, position);
        if !absorbed {
            let scale = entry.window.scale_factor() as f32;
            events.extend(conv::window_event(event, scale, &mut entry.pointer));
        }
        for event in events {
            let mut cx = Context {
                event_loop,
                windows: &mut self.windows,
                proxy: &self.proxy,
            };
            self.handler.window_event(&mut cx, id, event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // A file batch ends with the event batch (PLATFORM-WINIT-18).
        let pending: Vec<(WindowId, WindowEvent)> = self
            .windows
            .entries
            .iter_mut()
            .filter_map(|(id, entry)| entry.drops.flush().map(|event| (*id, event)))
            .collect();
        for (id, event) in pending {
            let mut cx = Context {
                event_loop,
                windows: &mut self.windows,
                proxy: &self.proxy,
            };
            self.handler.window_event(&mut cx, id, event);
        }
        let mut cx = Context {
            event_loop,
            windows: &mut self.windows,
            proxy: &self.proxy,
        };
        self.handler.idle(&mut cx);
    }
}

impl fmt::Debug for WinitPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WinitPlatform").finish_non_exhaustive()
    }
}

/// A Tantu-created window.
struct Entry {
    window: Arc<Window>,
    /// Last cursor position, in logical pixels.
    pointer: Point,
    /// Whether the cursor is inside the window (for file-drop positions).
    inside: bool,
    /// File events of the current gesture (PLATFORM-WINIT-18).
    drops: conv::DropBatch,
    /// The window's AccessKit adapter (PLATFORM-WINIT-15).
    adapter: accesskit_winit::Adapter,
    /// The window's native menu bar, kept alive while shown (PLATFORM-WINIT-21).
    #[cfg(all(
        feature = "native-menus",
        any(target_os = "macos", target_os = "windows")
    ))]
    menu: Option<muda::Menu>,
}

/// The windows Tantu created, by Tantu id and by winit id.
#[derive(Default)]
struct Windows {
    entries: HashMap<WindowId, Entry>,
    by_winit: HashMap<winit::window::WindowId, WindowId>,
    next: u64,
    /// The last native dialog id handed out (PLATFORM-WINIT-20).
    #[cfg_attr(not(feature = "native-dialogs"), allow(dead_code))]
    dialogs: u64,
    /// Whether `muda`'s event handler forwards to the loop yet (PLATFORM-WINIT-21).
    #[cfg_attr(
        not(all(
            feature = "native-menus",
            any(target_os = "macos", target_os = "windows")
        )),
        allow(dead_code)
    )]
    menu_events: bool,
    /// The app's tray icon, kept alive while shown (PLATFORM-WINIT-22).
    #[cfg(all(feature = "tray", any(target_os = "macos", target_os = "windows")))]
    tray: Option<tray_icon::TrayIcon>,
    /// Whether tray clicks forward to the loop yet.
    #[cfg_attr(
        not(all(feature = "tray", any(target_os = "macos", target_os = "windows"))),
        allow(dead_code)
    )]
    tray_events: bool,
    /// The first window's title, the app's name in notifications.
    #[cfg_attr(not(feature = "notifications"), allow(dead_code))]
    app_name: Option<String>,
    /// The OS clipboard, opened on first use; `Some(None)` after a failed open.
    clipboard: Option<Option<std::rc::Rc<dyn tantu_platform::Clipboard>>>,
}

/// The OS clipboard through `arboard` (PLATFORM-WINIT-17).
struct OsClipboard(std::cell::RefCell<arboard::Clipboard>);

impl tantu_platform::Clipboard for OsClipboard {
    fn read_text(&self) -> Option<String> {
        match self.0.borrow_mut().get_text() {
            Ok(text) if !text.is_empty() => Some(text),
            Ok(_) => None,
            Err(error) => {
                tracing::debug!(%error, "couldn't read the clipboard");
                None
            }
        }
    }

    fn write_text(&self, text: &str) -> bool {
        let mut clipboard = self.0.borrow_mut();
        let result = if text.is_empty() {
            clipboard.clear()
        } else {
            clipboard.set_text(text)
        };
        match result {
            Ok(()) => true,
            Err(error) => {
                tracing::debug!(%error, "couldn't write the clipboard");
                false
            }
        }
    }

    fn read_image(&self) -> Option<tantu_platform::ClipboardImage> {
        match self.0.borrow_mut().get_image() {
            Ok(image) => Some(tantu_platform::ClipboardImage {
                width: u32::try_from(image.width).ok()?,
                height: u32::try_from(image.height).ok()?,
                rgba: image.bytes.into_owned(),
            }),
            Err(error) => {
                tracing::debug!(%error, "couldn't read an image from the clipboard");
                None
            }
        }
    }

    fn write_image(&self, image: &tantu_platform::ClipboardImage) -> bool {
        let data = arboard::ImageData {
            width: image.width as usize,
            height: image.height as usize,
            bytes: std::borrow::Cow::Borrowed(&image.rgba),
        };
        logged(self.0.borrow_mut().set_image(data), "an image")
    }

    fn write_html(&self, html: &str, alt_text: &str) -> bool {
        logged(self.0.borrow_mut().set_html(html, Some(alt_text)), "HTML")
    }

    fn read_html(&self) -> Option<String> {
        match self.0.borrow_mut().get().html() {
            Ok(html) if !html.is_empty() => Some(html),
            Ok(_) => None,
            Err(error) => {
                tracing::debug!(%error, "couldn't read HTML from the clipboard");
                None
            }
        }
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    fn read_primary(&self) -> Option<String> {
        use arboard::{GetExtLinux, LinuxClipboardKind};
        match self
            .0
            .borrow_mut()
            .get()
            .clipboard(LinuxClipboardKind::Primary)
            .text()
        {
            Ok(text) if !text.is_empty() => Some(text),
            Ok(_) => None,
            Err(error) => {
                tracing::debug!(%error, "couldn't read the primary selection");
                None
            }
        }
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    fn write_primary(&self, text: &str) -> bool {
        use arboard::{LinuxClipboardKind, SetExtLinux};
        logged(
            self.0
                .borrow_mut()
                .set()
                .clipboard(LinuxClipboardKind::Primary)
                .text(text),
            "the primary selection",
        )
    }
}

/// `true` when a clipboard write worked; failures are logged (PLATFORM-WINIT-19).
fn logged(result: Result<(), arboard::Error>, what: &str) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => {
            tracing::debug!(%error, "couldn't write {what} to the clipboard");
            false
        }
    }
}

/// `PlatformContext` for one callback.
struct Context<'a> {
    event_loop: &'a ActiveEventLoop,
    windows: &'a mut Windows,
    proxy: &'a EventLoopProxy<UserEvent>,
}

impl Context<'_> {
    /// Forwards `muda`'s menu events (menu bars and the tray menu) to the loop, once
    /// (PLATFORM-WINIT-21, -22).
    #[cfg(all(
        any(feature = "native-menus", feature = "tray"),
        any(target_os = "macos", target_os = "windows")
    ))]
    fn forward_menu_events(&mut self) {
        if self.windows.menu_events {
            return;
        }
        let proxy = std::sync::Mutex::new(self.proxy.clone());
        muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            if let Ok(proxy) = proxy.lock() {
                let _ = proxy.send_event(UserEvent::Menu(event.id.0));
            }
        }));
        self.windows.menu_events = true;
    }

    /// A fresh dialog id (PLATFORM-WINIT-20).
    #[cfg(feature = "native-dialogs")]
    fn next_dialog(&mut self) -> tantu_platform::DialogId {
        self.windows.dialogs += 1;
        tantu_platform::DialogId(self.windows.dialogs)
    }
}

impl PlatformContext for Context<'_> {
    fn create_window(&mut self, attributes: &WindowAttributes) -> Result<WindowId, PlatformError> {
        // The adapter must exist before the window is shown (PLATFORM-WINIT-15).
        let window = self
            .event_loop
            .create_window(conv::attributes(attributes).with_visible(false))
            .map_err(|e| PlatformError::Os(Box::new(e)))?;
        let adapter = accesskit_winit::Adapter::with_event_loop_proxy(
            self.event_loop,
            &window,
            self.proxy.clone(),
        );
        window.set_visible(attributes.visible);
        if self.windows.app_name.is_none() {
            self.windows.app_name = Some(attributes.title.clone());
        }
        self.windows.next += 1;
        let id = WindowId::from_raw(self.windows.next);
        self.windows.by_winit.insert(window.id(), id);
        // Every new window gets a first redraw (spec: platform.md, `create_window`).
        window.request_redraw();
        self.windows.entries.insert(
            id,
            Entry {
                window: Arc::new(window),
                pointer: Point::ZERO,
                inside: false,
                drops: conv::DropBatch::default(),
                adapter,
                #[cfg(all(
                    feature = "native-menus",
                    any(target_os = "macos", target_os = "windows")
                ))]
                menu: None,
            },
        );
        Ok(id)
    }

    fn close_window(&mut self, window: WindowId) {
        if let Some(entry) = self.windows.entries.remove(&window) {
            self.windows.by_winit.remove(&entry.window.id());
            // The native window goes away when the last `SurfaceTarget` clone is dropped.
            entry.window.set_visible(false);
        }
    }

    fn request_redraw(&mut self, window: WindowId) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.request_redraw();
        }
    }

    fn inner_size(&self, window: WindowId) -> Option<PhysicalSize> {
        let entry = self.windows.entries.get(&window)?;
        Some(conv::size(entry.window.inner_size()))
    }

    fn scale_factor(&self, window: WindowId) -> Option<f32> {
        let entry = self.windows.entries.get(&window)?;
        Some(entry.window.scale_factor() as f32)
    }

    fn set_title(&mut self, window: WindowId, title: &str) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.set_title(title);
        }
    }

    #[cfg(feature = "native-dialogs")]
    fn file_dialog(
        &mut self,
        window: WindowId,
        request: tantu_platform::FileDialogRequest,
    ) -> Option<tantu_platform::DialogId> {
        use tantu_platform::FileDialogKind;
        let entry = self.windows.entries.get(&window)?;
        let mut dialog = rfd::AsyncFileDialog::new().set_parent(&*entry.window);
        if let Some(title) = &request.title {
            dialog = dialog.set_title(title);
        }
        if let Some(directory) = &request.directory {
            dialog = dialog.set_directory(directory);
        }
        if let Some(name) = &request.file_name {
            dialog = dialog.set_file_name(name);
        }
        for (name, extensions) in &request.filters {
            dialog = dialog.add_filter(name, extensions);
        }
        let id = self.next_dialog();
        let paths = |handles: Vec<rfd::FileHandle>| {
            handles
                .into_iter()
                .map(|h| h.path().to_path_buf())
                .collect::<Vec<_>>()
        };
        // Started here, on the main thread, and awaited on a worker (rfd's advice), so the
        // event loop keeps running (PLATFORM-WINIT-20).
        let result: std::pin::Pin<
            Box<dyn std::future::Future<Output = Vec<std::path::PathBuf>> + Send>,
        > = match request.kind {
            FileDialogKind::Open => {
                let f = dialog.pick_file();
                Box::pin(async move { f.await.map(|h| paths(vec![h])).unwrap_or_default() })
            }
            FileDialogKind::OpenMany => {
                let f = dialog.pick_files();
                Box::pin(async move { f.await.map(paths).unwrap_or_default() })
            }
            FileDialogKind::Save => {
                let f = dialog.save_file();
                Box::pin(async move { f.await.map(|h| paths(vec![h])).unwrap_or_default() })
            }
            FileDialogKind::PickFolder => {
                let f = dialog.pick_folder();
                Box::pin(async move { f.await.map(|h| paths(vec![h])).unwrap_or_default() })
            }
        };
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let paths = crate::native::block_on(result);
            let event = WindowEvent::FileDialogClosed { id, paths };
            let _ = proxy.send_event(UserEvent::Window(window, event));
        });
        Some(id)
    }

    #[cfg(feature = "native-dialogs")]
    fn message_dialog(
        &mut self,
        window: WindowId,
        request: tantu_platform::MessageRequest,
    ) -> Option<tantu_platform::DialogId> {
        let entry = self.windows.entries.get(&window)?;
        let buttons = request.buttons;
        let future = rfd::AsyncMessageDialog::new()
            .set_parent(&*entry.window)
            .set_level(crate::native::level(request.level))
            .set_title(&request.title)
            .set_description(&request.description)
            .set_buttons(crate::native::buttons(buttons))
            .show();
        let id = self.next_dialog();
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let result = crate::native::message_result(crate::native::block_on(future), buttons);
            let event = WindowEvent::MessageDialogClosed { id, result };
            let _ = proxy.send_event(UserEvent::Window(window, event));
        });
        Some(id)
    }

    #[cfg(feature = "notifications")]
    fn notify(&mut self, notification: &tantu_platform::Notification) -> bool {
        let app_name = self.windows.app_name.as_deref().unwrap_or("Tantu");
        match crate::native::notification(notification, app_name).show() {
            Ok(_) => true,
            Err(error) => {
                tracing::debug!(%error, "couldn't show a notification");
                false
            }
        }
    }

    #[cfg(all(
        feature = "native-menus",
        any(target_os = "macos", target_os = "windows")
    ))]
    fn set_menu_bar(&mut self, window: WindowId, menu: Option<tantu_platform::NativeMenu>) -> bool {
        self.forward_menu_events();
        let app_name = self
            .windows
            .app_name
            .clone()
            .unwrap_or_else(|| "Tantu".to_owned());
        let Some(entry) = self.windows.entries.get_mut(&window) else {
            return false;
        };
        let built = menu.map(|m| crate::native::build_menu(&m, window, &app_name));
        install_menu(entry, built)
    }

    #[cfg(all(feature = "tray", any(target_os = "macos", target_os = "windows")))]
    fn set_tray(&mut self, tray: Option<tantu_platform::Tray>) -> bool {
        self.forward_menu_events();
        if !self.windows.tray_events {
            let proxy = std::sync::Mutex::new(self.proxy.clone());
            tray_icon::TrayIconEvent::set_event_handler(Some(
                move |event: tray_icon::TrayIconEvent| {
                    use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Ok(proxy) = proxy.lock() {
                            let _ = proxy
                                .send_event(UserEvent::App(tantu_platform::AppEvent::TrayClicked));
                        }
                    }
                },
            ));
            self.windows.tray_events = true;
        }
        // Replacing drops the old icon first.
        self.windows.tray = None;
        let Some(tray) = tray else {
            return true;
        };
        let mut builder = tray_icon::TrayIconBuilder::new()
            .with_menu(Box::new(crate::native::build_tray_menu(&tray.menu)));
        if let Some(icon) = crate::native::tray_image(&tray.image) {
            builder = builder.with_icon(icon);
        }
        if let Some(tooltip) = &tray.tooltip {
            builder = builder.with_tooltip(tooltip);
        }
        match builder.build() {
            Ok(icon) => {
                self.windows.tray = Some(icon);
                true
            }
            Err(error) => {
                tracing::debug!(%error, "couldn't show the tray icon");
                false
            }
        }
    }

    fn native_capabilities(&self) -> tantu_platform::NativeCapabilities {
        tantu_platform::NativeCapabilities {
            dialogs: cfg!(feature = "native-dialogs"),
            notifications: cfg!(feature = "notifications"),
            menus: cfg!(all(
                feature = "native-menus",
                any(target_os = "macos", target_os = "windows")
            )),
            tray: cfg!(all(
                feature = "tray",
                any(target_os = "macos", target_os = "windows")
            )),
        }
    }

    fn surface_target(&self, window: WindowId) -> Option<SurfaceTarget> {
        let entry = self.windows.entries.get(&window)?;
        Some(entry.window.clone())
    }

    fn set_ime_allowed(&mut self, window: WindowId, allowed: bool) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.set_ime_allowed(allowed);
        }
    }

    fn set_ime_cursor_area(&mut self, window: WindowId, origin: Point, size: Size) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.set_ime_cursor_area(
                winit::dpi::LogicalPosition::new(origin.x, origin.y),
                winit::dpi::LogicalSize::new(size.width, size.height),
            );
        }
    }

    fn clipboard(&mut self) -> Option<std::rc::Rc<dyn tantu_platform::Clipboard>> {
        self.windows
            .clipboard
            .get_or_insert_with(|| match arboard::Clipboard::new() {
                Ok(clipboard) => Some(std::rc::Rc::new(OsClipboard(std::cell::RefCell::new(
                    clipboard,
                )))
                    as std::rc::Rc<dyn tantu_platform::Clipboard>),
                Err(error) => {
                    tracing::debug!(%error, "couldn't open the clipboard");
                    None
                }
            })
            .clone()
    }

    fn set_cursor(&mut self, window: WindowId, cursor: tantu_platform::CursorIcon) {
        if let Some(entry) = self.windows.entries.get(&window) {
            entry.window.set_cursor(conv::cursor(cursor));
        }
    }

    fn exit(&mut self) {
        self.event_loop.exit();
    }

    fn update_accessibility(&mut self, window: WindowId, update: TreeUpdate) {
        if let Some(entry) = self.windows.entries.get_mut(&window) {
            entry.adapter.update_if_active(|| update);
        }
    }

    fn now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }

    fn wake_at(&mut self, deadline: Option<std::time::Instant>) {
        // PLATFORM-WINIT-14: `about_to_wait` (the handler's `idle`) runs when the deadline
        // passes. A passed deadline would fire again at once, so the runner clears it.
        self.event_loop.set_control_flow(match deadline {
            Some(deadline) => winit::event_loop::ControlFlow::WaitUntil(deadline),
            None => winit::event_loop::ControlFlow::Wait,
        });
    }
}

/// Shows `menu` as the window's bar (macOS: the app's bar), replacing the previous one
/// (PLATFORM-WINIT-21).
#[cfg(all(feature = "native-menus", target_os = "macos"))]
fn install_menu(entry: &mut Entry, menu: Option<muda::Menu>) -> bool {
    match &menu {
        Some(menu) => menu.init_for_nsapp(),
        None => {
            if let Some(old) = &entry.menu {
                old.remove_for_nsapp();
            }
        }
    }
    entry.menu = menu;
    true
}

/// Shows `menu` as the window's bar, replacing the previous one (PLATFORM-WINIT-21).
#[cfg(all(feature = "native-menus", target_os = "windows"))]
#[allow(unsafe_code)]
fn install_menu(entry: &mut Entry, menu: Option<muda::Menu>) -> bool {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let hwnd = match entry.window.window_handle().map(|h| h.as_raw()) {
        Ok(RawWindowHandle::Win32(handle)) => handle.hwnd.get(),
        _ => return false,
    };
    if let Some(old) = entry.menu.take() {
        // SAFETY: `hwnd` is this live winit window's handle, which the old menu was
        // installed on.
        let _ = unsafe { old.remove_for_hwnd(hwnd) };
    }
    let Some(menu) = menu else {
        return true;
    };
    // SAFETY: `hwnd` is this live winit window's handle; the menu is kept in the entry,
    // so it outlives its use by the window until replaced or the window closes.
    let installed = unsafe { menu.init_for_hwnd(hwnd) }.is_ok();
    entry.menu = Some(menu);
    installed
}
