//! [`SystemTray`]: the app's tray icon as a view (ADR 0017 part 3). Spec:
//! `docs/specs/platform/tray.md`.

use std::rc::Rc;

use tantu_scene::ImageData;
use tantu_view::{BuildCx, ElementId, IntoProp, Prop, View};
use tantu_widgets::MenuItem;

/// The app's tray icon, shown while this view is built (under the runner, on platforms with
/// a tray: macOS and Windows with the `tray` feature). Paints nothing and takes no space
/// (FACADE-TRAY-01, -02).
pub struct SystemTray {
    image: ImageData,
    tooltip: Option<Prop<String>>,
    menu: Vec<MenuItem>,
    on_click: Option<Rc<dyn Fn()>>,
}

impl SystemTray {
    /// A tray icon showing `image`.
    pub fn new(image: ImageData) -> Self {
        SystemTray {
            image,
            tooltip: None,
            menu: Vec::new(),
            on_click: None,
        }
    }

    /// The text shown on hover.
    pub fn tooltip(self, text: impl IntoProp<String>) -> Self {
        SystemTray {
            tooltip: Some(text.into_prop()),
            ..self
        }
    }

    /// The icon's menu.
    pub fn menu(self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        SystemTray {
            menu: items.into_iter().collect(),
            ..self
        }
    }

    /// Called when the icon is clicked.
    pub fn on_click(self, f: impl Fn() + 'static) -> Self {
        SystemTray {
            on_click: Some(Rc::new(f)),
            ..self
        }
    }
}

impl View for SystemTray {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let _ = (cx, self.image, self.tooltip, self.menu, self.on_click);
        todo!("FACADE-TRAY-01")
    }
}
