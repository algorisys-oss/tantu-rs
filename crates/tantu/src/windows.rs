//! [`Windows`] and [`WindowHandle`]: opening and closing the app's windows from views. Spec:
//! `docs/specs/facade/windows.md`.

use tantu_view::{BuildCx, View};

use crate::Window;

/// Opens and closes the app's windows from views. Provided at the root of every window the
/// runner opens. A `Copy` handle; requests take effect after the current event
/// (FACADE-WIN-03).
#[derive(Clone, Copy)]
pub struct Windows {
    _private: (),
}

impl Windows {
    /// The app's windows, or `None` outside a window opened by the runner (a `WidgetTester`
    /// tree, a bare `ViewTree`).
    pub fn of(cx: &BuildCx<'_>) -> Option<Windows> {
        let _ = cx;
        todo!("FACADE-WIN-03")
    }

    /// Opens `window` showing `content`, built in the new window's own root scope.
    pub fn open<V: View>(
        self,
        window: Window,
        content: impl FnOnce() -> V + 'static,
    ) -> WindowHandle {
        let _ = (window, content);
        todo!("FACADE-WIN-03")
    }

    /// The window this handle was taken from.
    pub fn current(self) -> WindowHandle {
        todo!("FACADE-WIN-03")
    }

    /// How many windows are open (a tracked read).
    pub fn count(self) -> usize {
        todo!("FACADE-WIN-03")
    }
}

/// One window of the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowHandle {
    token: u64,
}

impl WindowHandle {
    /// Closes the window, as the user closing it but without the veto (FACADE-WIN-04).
    pub fn close(self) {
        let _ = self.token;
        todo!("FACADE-WIN-04")
    }

    /// Whether it is still open (a tracked read).
    pub fn is_open(self) -> bool {
        todo!("FACADE-WIN-04")
    }

    /// Changes its title (FACADE-WIN-05); ignored once the window is closed.
    pub fn set_title(self, title: impl Into<String>) {
        let _ = title;
        todo!("FACADE-WIN-05")
    }
}
