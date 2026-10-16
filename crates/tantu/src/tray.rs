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

/// What a `SystemTray` asks the runner to show (FACADE-TRAY-01).
pub(crate) struct TrayRequest {
    pub(crate) image: tantu_platform::TrayImage,
    pub(crate) tooltip: Option<String>,
    pub(crate) entries: Vec<tantu_widgets::NativeMenuEntry>,
    pub(crate) on_click: Option<Rc<dyn Fn()>>,
}

/// The app's tray as the runner tracks it (FACADE-TRAY-01, -02).
#[derive(Default)]
pub(crate) struct TrayState {
    /// A change not shown yet: `Some(None)` removes the tray.
    pub(crate) pending: Option<Option<TrayRequest>>,
    /// The newest `SystemTray`'s number; older ones are replaced and stay quiet.
    pub(crate) latest: u64,
    /// The shown menu's actions, by id, and the click callback.
    pub(crate) actions: std::collections::HashMap<u64, Rc<dyn Fn()>>,
    pub(crate) on_click: Option<Rc<dyn Fn()>>,
}

/// The runner's tray, inherited where the platform has one.
#[derive(Clone)]
pub(crate) struct TrayHandle(pub(crate) Rc<std::cell::RefCell<TrayState>>);

impl View for SystemTray {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(
            tantu_layout::RenderConstrainedBox::new(tantu_layout::BoxConstraints::tight(
                tantu_core::Size::ZERO,
            )),
            [],
        );
        let Some(TrayHandle(state)) = cx.inherit::<TrayHandle>() else {
            return id; // No tray here (FACADE-TRAY-02).
        };
        let number = {
            let mut state = state.borrow_mut();
            state.latest += 1;
            state.latest
        };
        let image = tantu_platform::TrayImage {
            width: self.image.width(),
            height: self.image.height(),
            rgba: self.image.pixels().to_vec(),
        };
        let tooltip: Option<Rc<dyn Fn() -> String>> = self.tooltip.map(|prop| match prop {
            Prop::Value(text) => Rc::new(move || text.clone()) as Rc<dyn Fn() -> String>,
            Prop::Dynamic(f) => Rc::from(f),
        });
        let (menu, on_click) = (self.menu, self.on_click);
        {
            let state = Rc::clone(&state);
            // Sent again whenever the tooltip or a menu state it reads changes.
            tantu_reactive::effect(move || {
                let request = TrayRequest {
                    image: image.clone(),
                    tooltip: tooltip.as_ref().map(|t| t()),
                    entries: tantu_widgets::native_menu_entries(&menu),
                    on_click: on_click.clone(),
                };
                let mut state = state.borrow_mut();
                if state.latest == number {
                    state.pending = Some(Some(request));
                }
            });
        }
        // The tray goes with the view, unless a newer one replaced it (FACADE-TRAY-02).
        tantu_reactive::on_cleanup(move || {
            if let Ok(mut state) = state.try_borrow_mut() {
                if state.latest == number {
                    state.pending = Some(None);
                }
            }
        });
        id
    }
}
