//! [`Windows`] and [`WindowHandle`]: opening and closing the app's windows from views. Spec:
//! `docs/specs/facade/windows.md`.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use tantu_reactive::Signal;
use tantu_view::{AnyView, BuildCx, View};

use crate::Window;

/// A window's content, built with the window's root scope current.
pub(crate) type Content = Box<dyn FnOnce() -> AnyView>;

/// What views asked the runner to do, applied after the current event (FACADE-WIN-03).
pub(crate) enum Request {
    /// Open `window` showing `content`, known to the app as `token`.
    Open {
        token: u64,
        window: Window,
        content: Content,
    },
    /// Close the window `token` (no veto).
    Close(u64),
    /// Retitle the window `token`.
    Title(u64, String),
}

/// The runner's request queue and token counter, shared by every [`Windows`] handle.
#[derive(Default)]
pub(crate) struct Queue {
    next: Cell<u64>,
    requests: RefCell<Vec<Request>>,
}

impl Queue {
    /// A fresh window token (tokens are never reused).
    pub(crate) fn token(&self) -> u64 {
        let token = self.next.get() + 1;
        self.next.set(token);
        token
    }

    pub(crate) fn push(&self, request: Request) {
        self.requests.borrow_mut().push(request);
    }

    /// The queued requests, oldest first.
    pub(crate) fn take(&self) -> Vec<Request> {
        std::mem::take(&mut self.requests.borrow_mut())
    }
}

/// Opens and closes the app's windows from views. Provided at the root of every window the
/// runner opens. A `Copy` handle; requests take effect after the current event
/// (FACADE-WIN-03).
#[derive(Clone, Copy)]
pub struct Windows {
    /// The runner's queue (owned by the app scope).
    pub(crate) queue: Signal<Rc<Queue>>,
    /// The tokens of the open windows, in opening order.
    pub(crate) open: Signal<Vec<u64>>,
    /// The window this handle was provided to.
    pub(crate) current: u64,
}

impl Windows {
    /// The app's windows, or `None` outside a window opened by the runner (a `WidgetTester`
    /// tree, a bare `ViewTree`).
    pub fn of(cx: &BuildCx<'_>) -> Option<Windows> {
        cx.inherit::<Windows>()
    }

    /// Opens `window` showing `content`, built in the new window's own root scope. The
    /// returned handle is open once the runner has created the window.
    pub fn open<V: View>(
        self,
        window: Window,
        content: impl FnOnce() -> V + 'static,
    ) -> WindowHandle {
        let token = self
            .queue
            .try_with(|queue| {
                let token = queue.token();
                queue.push(Request::Open {
                    token,
                    window,
                    content: Box::new(move || AnyView::new(content())),
                });
                token
            })
            .unwrap_or(0);
        self.handle(token)
    }

    /// The window this handle was taken from.
    pub fn current(self) -> WindowHandle {
        self.handle(self.current)
    }

    /// How many windows are open (a tracked read).
    pub fn count(self) -> usize {
        self.open.try_with(Vec::len).unwrap_or(0)
    }

    fn handle(self, token: u64) -> WindowHandle {
        WindowHandle {
            token,
            windows: self,
        }
    }
}

/// One window of the app.
#[derive(Clone, Copy)]
pub struct WindowHandle {
    token: u64,
    windows: Windows,
}

impl WindowHandle {
    /// Closes the window, as the user closing it but without the veto (FACADE-WIN-04).
    pub fn close(self) {
        if self.is_open_untracked() {
            self.request(Request::Close(self.token));
        }
    }

    /// Whether it is still open (a tracked read).
    pub fn is_open(self) -> bool {
        self.windows
            .open
            .try_with(|open| open.contains(&self.token))
            .unwrap_or(false)
    }

    /// Changes its title (FACADE-WIN-05); ignored once the window is closed.
    pub fn set_title(self, title: impl Into<String>) {
        if self.is_open_untracked() {
            self.request(Request::Title(self.token, title.into()));
        }
    }

    /// Whether it is open, or queued to open (so a just-opened handle can be retitled).
    fn is_open_untracked(self) -> bool {
        tantu_reactive::untrack(|| {
            let queued = self
                .windows
                .queue
                .try_with(|queue| {
                    queue.requests.borrow().iter().any(|request| {
                        matches!(request, Request::Open { token, .. } if *token == self.token)
                    })
                })
                .unwrap_or(false);
            queued || self.is_open()
        })
    }

    fn request(self, request: Request) {
        let _ = self.windows.queue.try_with(|queue| queue.push(request));
    }
}

impl PartialEq for WindowHandle {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

impl Eq for WindowHandle {}

impl Hash for WindowHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.token.hash(state);
    }
}

impl fmt::Debug for WindowHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WindowHandle").field(&self.token).finish()
    }
}
