//! Tests for `docs/specs/platform/native-menus.md`, rule FACADE-APP-21.

use std::cell::RefCell;
use std::rc::Rc;

use tantu::platform::{
    FakeLog, FakePlatform, NativeMenu, NativeMenuItem, PlatformContext, WindowEvent, WindowId,
};
use tantu::prelude::*;
use tantu::scene::{RenderError, RenderReport, Renderer, Resources, Scene};
use tantu_render_headless::HeadlessRenderer;

const FONT: &[u8] = include_bytes!("../../tantu-text/tests/fonts/LiberationSans-Regular.ttf");

struct Recorder(HeadlessRenderer);

impl Renderer for Recorder {
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.0.resize(width, height, scale_factor);
    }

    fn render(
        &mut self,
        scene: &Scene,
        resources: &Resources,
    ) -> std::result::Result<RenderReport, RenderError> {
        self.0.render(scene, resources)
    }
}

fn run(app: App, platform: FakePlatform) -> FakeLog {
    let mut handler = app.without_system_fonts().font(FONT.to_vec()).handler(
        |_: &mut dyn PlatformContext, _: WindowId| {
            Ok(Box::new(Recorder(HeadlessRenderer::new(1, 1))) as Box<dyn Renderer>)
        },
    );
    platform.run_logged(&mut handler)
}

type Log = Rc<RefCell<Vec<String>>>;

/// Window A has a native File menu (Save, Wrap); window B a drawn bar.
fn app(log: &Log) -> App {
    let (l1, l2) = (log.clone(), log.clone());
    App::new()
        .window(Window::new("A").size(300.0, 200.0), move || {
            let wrap = signal(false);
            let l2 = l2.clone();
            MenuBar::new().native(true).menu(
                "File",
                [
                    MenuItem::new("Save")
                        .shortcut("Ctrl+S")
                        .on_select(move || l1.borrow_mut().push("save".into())),
                    MenuItem::new("Wrap").checked(wrap).on_select(move || {
                        wrap.update(|w| *w = !*w);
                        l2.borrow_mut().push("wrap".into());
                    }),
                ],
            )
        })
        .window(Window::new("B").size(300.0, 200.0), || {
            MenuBar::new().menu("Edit", [MenuItem::new("Copy")])
        })
}

/// The ids of the File menu's items in a recorded bar.
fn ids(menu: &NativeMenu) -> Vec<(String, u64, Option<bool>)> {
    let NativeMenuItem::Submenu { label, items, .. } = &menu.menus[0] else {
        panic!("a submenu");
    };
    assert_eq!(label, "File");
    items
        .iter()
        .filter_map(|i| match i {
            NativeMenuItem::Item {
                id, label, checked, ..
            } => Some((label.clone(), *id, *checked)),
            _ => None,
        })
        .collect()
}

#[test]
fn facade_app_21_native_menu_bars() {
    let log: Log = Rc::default();
    let first = run(app(&log), FakePlatform::new());
    let bar = first.windows[0]
        .menu_bars
        .last()
        .cloned()
        .flatten()
        .expect("A's bar was installed");
    let items = ids(&bar);
    assert_eq!(items[0].0, "Save");
    assert_eq!(items[1].2, Some(false));
    // B's bar is drawn: nothing installed for it.
    assert!(first.windows[1].menu_bars.is_empty());
    // Activations reach the items; a checked state change re-sends the bar; focusing the
    // window re-sends it too (macOS has one bar for the app).
    let calls = first.windows[0].menu_bars.len();
    let platform = FakePlatform::new()
        .event(0, WindowEvent::MenuItemActivated { id: items[0].1 })
        .event(0, WindowEvent::MenuItemActivated { id: items[1].1 })
        .event(0, WindowEvent::Focused(true));
    let log: Log = Rc::default();
    let second = run(app(&log), platform);
    assert_eq!(*log.borrow(), ["save", "wrap"]);
    let bars = &second.windows[0].menu_bars;
    assert!(bars.len() >= calls + 2, "{} calls", bars.len());
    let last = bars.last().cloned().flatten().expect("a bar");
    assert_eq!(ids(&last)[1].2, Some(true), "Wrap is checked now");
    // Without native menus the bar is drawn and nothing is installed.
    let third = run(
        app(&Rc::default()),
        FakePlatform::new().without_native_menus(),
    );
    assert!(third.windows[0].menu_bars.is_empty());
}
