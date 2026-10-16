//! [`FakePlatform`]: a scripted platform for tests without a display.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_core::{Point, Size};

use crate::Clipboard as _;
use crate::platform::{Platform, PlatformContext, PlatformError, PlatformHandler, SurfaceTarget};
use crate::types::{CursorIcon, PhysicalSize, WindowAttributes, WindowEvent, WindowId};

/// One step of a [`FakePlatform`] script.
#[derive(Clone, Debug, PartialEq)]
pub enum FakeStep {
    /// Deliver `event` to the `window`-th window created (0 = the first).
    Event {
        /// Index of the window in creation order.
        window: usize,
        /// The event to deliver.
        event: WindowEvent,
    },
    /// Deliver nothing; just let pending redraws and `idle` run.
    Idle,
    /// Moves the fake clock forward, then runs a redraw round and `idle` (PLATFORM-FAKE-12).
    Advance(std::time::Duration),
}

/// What the handler asked a [`FakePlatform`] for.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FakeLog {
    /// Every window created, in creation order.
    pub windows: Vec<FakeWindow>,
    /// Whether the handler called `exit`.
    pub exited: bool,
    /// How many script steps were delivered (skipped steps not counted).
    pub steps_delivered: usize,
    /// Every `wake_at` call, as offsets from the run's start (`None` for a cancel)
    /// (PLATFORM-FAKE-12).
    pub wakes: Vec<Option<std::time::Duration>>,
    /// The clipboard's text when the run ended (PLATFORM-FAKE-15).
    pub clipboard_text: Option<String>,
    /// The fake clipboard's image when the run ended (PLATFORM-FAKE-17).
    pub clipboard_image: Option<crate::ClipboardImage>,
    /// The fake clipboard's HTML when the run ended (PLATFORM-FAKE-17).
    pub clipboard_html: Option<String>,
    /// The fake primary selection when the run ended (PLATFORM-FAKE-17).
    pub primary_selection: Option<String>,
    /// Every file dialog requested, with its window (PLATFORM-FAKE-18).
    pub file_dialogs: Vec<(WindowId, crate::FileDialogRequest)>,
    /// Every message dialog requested, with its window.
    pub message_dialogs: Vec<(WindowId, crate::MessageRequest)>,
    /// Every notification shown.
    pub notifications: Vec<crate::Notification>,
}

/// A window of a [`FakePlatform`], as last seen.
#[derive(Clone, Debug, PartialEq)]
pub struct FakeWindow {
    /// Its id.
    pub id: WindowId,
    /// The attributes, with the title as last set.
    pub attributes: WindowAttributes,
    /// Its inner size.
    pub size: PhysicalSize,
    /// Its scale factor.
    pub scale_factor: f32,
    /// Whether the handler closed it.
    pub closed: bool,
    /// `RedrawRequested` events delivered to it.
    pub redraws: u32,
    /// What the handler last passed to `set_ime_allowed` (PLATFORM-FAKE-11).
    pub ime_allowed: bool,
    /// What the handler last passed to `set_ime_cursor_area`.
    pub ime_area: Option<(Point, Size)>,
    /// The updates sent with `update_accessibility`, in order (PLATFORM-FAKE-13).
    pub accessibility_updates: Vec<accesskit::TreeUpdate>,
    /// Every cursor set with `set_cursor`, in order, repeats included (PLATFORM-FAKE-14).
    pub cursors: Vec<CursorIcon>,
    /// Every `set_menu_bar` call for the window, in order (PLATFORM-FAKE-19).
    pub menu_bars: Vec<Option<crate::NativeMenu>>,
}

/// A platform without a display: runs a handler against a script of events.
///
/// After `started` and after each delivered step, every window with a pending redraw gets one
/// `RedrawRequested` (requests made meanwhile wait for the next round), then `idle` is called
/// (not after `started`). One more redraw round runs after the last step. The clock starts
/// at `run_logged` and moves only with [`FakeStep::Advance`] (PLATFORM-FAKE-12).
#[derive(Clone, Debug)]
pub struct FakePlatform {
    scale_factor: f32,
    steps: Vec<FakeStep>,
    clipboard: Option<String>,
    /// Answers for file and message dialogs, in order (PLATFORM-FAKE-18).
    file_answers: Vec<Vec<std::path::PathBuf>>,
    message_answers: Vec<crate::MessageResult>,
    /// Whether dialogs and notifications are supported.
    native: bool,
}

impl FakePlatform {
    /// Answers the next unanswered file dialog with `paths` (PLATFORM-FAKE-18).
    pub fn file_dialog_answer(mut self, paths: Vec<std::path::PathBuf>) -> Self {
        self.file_answers.push(paths);
        self
    }

    /// Answers the next unanswered message dialog with `result`.
    pub fn message_answer(mut self, result: crate::MessageResult) -> Self {
        self.message_answers.push(result);
        self
    }

    /// A platform without native menus (PLATFORM-FAKE-19).
    pub fn without_native_menus(self) -> Self {
        todo!("PLATFORM-FAKE-19")
    }

    /// A platform without native dialogs or notifications (the trait's defaults).
    pub fn without_native_dialogs(self) -> Self {
        FakePlatform {
            native: false,
            ..self
        }
    }

    /// An empty script; new windows get scale factor 1.
    pub fn new() -> FakePlatform {
        FakePlatform {
            scale_factor: 1.0,
            steps: Vec::new(),
            clipboard: None,
            file_answers: Vec::new(),
            message_answers: Vec::new(),
            native: true,
        }
    }

    /// The clipboard's text when the run starts (PLATFORM-FAKE-15).
    pub fn clipboard_text(self, text: impl Into<String>) -> Self {
        let text = text.into();
        FakePlatform {
            clipboard: (!text.is_empty()).then_some(text),
            ..self
        }
    }

    /// The scale factor new windows get.
    pub fn scale_factor(self, scale_factor: f32) -> Self {
        FakePlatform {
            scale_factor,
            ..self
        }
    }

    /// Appends a step.
    pub fn step(mut self, step: FakeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Appends `FakeStep::Event { window, event }`.
    pub fn event(self, window: usize, event: WindowEvent) -> Self {
        self.step(FakeStep::Event { window, event })
    }

    /// Runs like [`Platform::run`] and returns the log.
    pub fn run_logged(self, handler: &mut dyn PlatformHandler) -> FakeLog {
        let start = std::time::Instant::now();
        let clipboard = Rc::new(FakeClipboard {
            text: RefCell::new(self.clipboard),
            ..FakeClipboard::default()
        });
        let mut cx = FakeContext {
            log: FakeLog::default(),
            scale_factor: self.scale_factor,
            pending: Vec::new(),
            start,
            now: start,
            clipboard: Rc::clone(&clipboard),
            native: self.native,
            file_answers: self.file_answers.into(),
            message_answers: self.message_answers.into(),
            next_dialog: 0,
            dialog_results: Vec::new(),
        };
        handler.started(&mut cx);
        if !cx.log.exited {
            cx.redraw_round(handler);
        }
        for step in self.steps {
            if cx.log.exited {
                break;
            }
            // Dialog answers due after this step's own event (PLATFORM-FAKE-18).
            let due = std::mem::take(&mut cx.dialog_results);
            match step {
                FakeStep::Event { window, mut event } => {
                    let Some(w) = cx.log.windows.get_mut(window).filter(|w| !w.closed) else {
                        cx.deliver(handler, due);
                        continue;
                    };
                    match &mut event {
                        WindowEvent::Resized(size) => w.size = *size,
                        WindowEvent::ScaleFactorChanged(scale) => w.scale_factor = *scale,
                        _ => {}
                    }
                    let id = w.id;
                    cx.log.steps_delivered += 1;
                    handler.window_event(&mut cx, id, event);
                }
                FakeStep::Idle => cx.log.steps_delivered += 1,
                FakeStep::Advance(by) => {
                    cx.now += by;
                    cx.log.steps_delivered += 1;
                }
            }
            cx.deliver(handler, due);
            if cx.log.exited {
                break;
            }
            cx.redraw_round(handler);
            if cx.log.exited {
                break;
            }
            handler.idle(&mut cx);
        }
        if !cx.log.exited {
            cx.redraw_round(handler);
        }
        cx.log.clipboard_text = clipboard.read_text();
        cx.log.clipboard_image = clipboard.read_image();
        cx.log.clipboard_html = clipboard.read_html();
        cx.log.primary_selection = clipboard.read_primary();
        cx.log
    }
}

impl Default for FakePlatform {
    fn default() -> Self {
        FakePlatform::new()
    }
}

impl Platform for FakePlatform {
    fn run(self, handler: &mut dyn PlatformHandler) -> Result<(), PlatformError> {
        self.run_logged(handler);
        Ok(())
    }
}

/// The `PlatformContext` a `FakePlatform` hands its handler.
struct FakeContext {
    log: FakeLog,
    scale_factor: f32,
    /// Indices of windows with a redraw pending, in order of first request.
    pending: Vec<usize>,
    /// When the run started: the clock's origin (PLATFORM-FAKE-12).
    start: std::time::Instant,
    /// The fake clock, moved only by `Advance`.
    now: std::time::Instant,
    /// The in-memory clipboard (PLATFORM-FAKE-15).
    clipboard: Rc<FakeClipboard>,
    /// Whether dialogs and notifications are supported (PLATFORM-FAKE-18).
    native: bool,
    /// Scripted answers, in order.
    file_answers: std::collections::VecDeque<Vec<std::path::PathBuf>>,
    message_answers: std::collections::VecDeque<crate::MessageResult>,
    /// The last dialog id handed out.
    next_dialog: u64,
    /// Answers waiting for the next step, with their window.
    dialog_results: Vec<(WindowId, WindowEvent)>,
}

/// The fake platform's clipboard, in memory: text, an image, HTML and the primary
/// selection side by side; writing HTML also sets the text to its alternative
/// (PLATFORM-FAKE-15, -17).
#[derive(Default)]
struct FakeClipboard {
    text: RefCell<Option<String>>,
    image: RefCell<Option<crate::ClipboardImage>>,
    html: RefCell<Option<String>>,
    primary: RefCell<Option<String>>,
}

/// Empty text counts as none.
fn non_empty(text: &str) -> Option<String> {
    (!text.is_empty()).then(|| text.to_owned())
}

impl crate::Clipboard for FakeClipboard {
    fn read_text(&self) -> Option<String> {
        self.text.borrow().clone()
    }

    fn write_text(&self, text: &str) -> bool {
        *self.text.borrow_mut() = non_empty(text);
        true
    }

    fn read_image(&self) -> Option<crate::ClipboardImage> {
        self.image.borrow().clone()
    }

    fn write_image(&self, image: &crate::ClipboardImage) -> bool {
        *self.image.borrow_mut() = Some(image.clone());
        true
    }

    fn write_html(&self, html: &str, alt_text: &str) -> bool {
        *self.html.borrow_mut() = non_empty(html);
        *self.text.borrow_mut() = non_empty(alt_text);
        true
    }

    fn read_html(&self) -> Option<String> {
        self.html.borrow().clone()
    }

    fn read_primary(&self) -> Option<String> {
        self.primary.borrow().clone()
    }

    fn write_primary(&self, text: &str) -> bool {
        *self.primary.borrow_mut() = non_empty(text);
        true
    }
}

impl FakeContext {
    /// Delivers dialog answers to the windows still open (PLATFORM-FAKE-18).
    fn deliver(&mut self, handler: &mut dyn PlatformHandler, due: Vec<(WindowId, WindowEvent)>) {
        for (window, event) in due {
            if !self.log.exited && self.open_index(window).is_some() {
                handler.window_event(self, window, event);
            }
        }
    }

    /// A fresh dialog id.
    fn dialog_id(&mut self) -> crate::DialogId {
        self.next_dialog += 1;
        crate::DialogId(self.next_dialog)
    }

    /// The index of an open window.
    fn open_index(&self, window: WindowId) -> Option<usize> {
        self.log
            .windows
            .iter()
            .position(|w| w.id == window && !w.closed)
    }

    /// Delivers one `RedrawRequested` to each window with a redraw pending.
    fn redraw_round(&mut self, handler: &mut dyn PlatformHandler) {
        let due = std::mem::take(&mut self.pending);
        for index in due {
            if self.log.exited {
                return;
            }
            let Some(w) = self.log.windows.get_mut(index).filter(|w| !w.closed) else {
                continue;
            };
            w.redraws += 1;
            let id = w.id;
            handler.window_event(self, id, WindowEvent::RedrawRequested);
        }
    }

    fn mark_pending(&mut self, index: usize) {
        if !self.pending.contains(&index) {
            self.pending.push(index);
        }
    }
}

impl PlatformContext for FakeContext {
    fn create_window(&mut self, attributes: &WindowAttributes) -> Result<WindowId, PlatformError> {
        let index = self.log.windows.len();
        let id = WindowId::from_raw(index as u64 + 1);
        self.log.windows.push(FakeWindow {
            id,
            attributes: attributes.clone(),
            size: PhysicalSize::from_logical(attributes.size, self.scale_factor),
            scale_factor: self.scale_factor,
            closed: false,
            redraws: 0,
            ime_allowed: false,
            ime_area: None,
            accessibility_updates: Vec::new(),
            cursors: Vec::new(),
            menu_bars: Vec::new(),
        });
        self.mark_pending(index);
        Ok(id)
    }

    fn close_window(&mut self, window: WindowId) {
        if let Some(index) = self.open_index(window) {
            self.log.windows[index].closed = true;
            self.pending.retain(|&i| i != index);
        }
    }

    fn request_redraw(&mut self, window: WindowId) {
        if let Some(index) = self.open_index(window) {
            self.mark_pending(index);
        }
    }

    fn inner_size(&self, window: WindowId) -> Option<PhysicalSize> {
        self.open_index(window).map(|i| self.log.windows[i].size)
    }

    fn scale_factor(&self, window: WindowId) -> Option<f32> {
        self.open_index(window)
            .map(|i| self.log.windows[i].scale_factor)
    }

    fn set_title(&mut self, window: WindowId, title: &str) {
        if let Some(index) = self.open_index(window) {
            self.log.windows[index].attributes.title = title.to_owned();
        }
    }

    fn surface_target(&self, _window: WindowId) -> Option<SurfaceTarget> {
        None
    }

    fn set_ime_allowed(&mut self, window: WindowId, allowed: bool) {
        if let Some(index) = self.open_index(window) {
            self.log.windows[index].ime_allowed = allowed;
        }
    }

    fn set_ime_cursor_area(&mut self, window: WindowId, origin: Point, size: Size) {
        if let Some(index) = self.open_index(window) {
            self.log.windows[index].ime_area = Some((origin, size));
        }
    }

    fn file_dialog(
        &mut self,
        window: WindowId,
        request: crate::FileDialogRequest,
    ) -> Option<crate::DialogId> {
        if !self.native || self.open_index(window).is_none() {
            return None;
        }
        self.log.file_dialogs.push((window, request));
        let id = self.dialog_id();
        let paths = self.file_answers.pop_front().unwrap_or_default();
        self.dialog_results
            .push((window, WindowEvent::FileDialogClosed { id, paths }));
        Some(id)
    }

    fn message_dialog(
        &mut self,
        window: WindowId,
        request: crate::MessageRequest,
    ) -> Option<crate::DialogId> {
        if !self.native || self.open_index(window).is_none() {
            return None;
        }
        self.log.message_dialogs.push((window, request));
        let id = self.dialog_id();
        let result = self
            .message_answers
            .pop_front()
            .unwrap_or(crate::MessageResult::Cancel);
        self.dialog_results
            .push((window, WindowEvent::MessageDialogClosed { id, result }));
        Some(id)
    }

    fn notify(&mut self, notification: &crate::Notification) -> bool {
        if self.native {
            self.log.notifications.push(notification.clone());
        }
        self.native
    }

    fn native_capabilities(&self) -> crate::NativeCapabilities {
        crate::NativeCapabilities {
            dialogs: self.native,
            notifications: self.native,
            menus: self.native,
        }
    }

    fn clipboard(&mut self) -> Option<std::rc::Rc<dyn crate::Clipboard>> {
        Some(Rc::clone(&self.clipboard) as Rc<dyn crate::Clipboard>)
    }

    fn set_cursor(&mut self, window: WindowId, cursor: CursorIcon) {
        if let Some(index) = self.open_index(window) {
            self.log.windows[index].cursors.push(cursor);
        }
    }

    fn exit(&mut self) {
        self.log.exited = true;
    }

    fn update_accessibility(&mut self, window: WindowId, update: accesskit::TreeUpdate) {
        if let Some(index) = self.open_index(window) {
            self.log.windows[index].accessibility_updates.push(update);
        }
    }

    fn now(&self) -> std::time::Instant {
        self.now
    }

    fn wake_at(&mut self, deadline: Option<std::time::Instant>) {
        self.log
            .wakes
            .push(deadline.map(|d| d.saturating_duration_since(self.start)));
    }
}
