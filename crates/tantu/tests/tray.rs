//! Tests for `docs/specs/platform/tray.md`, rules FACADE-TRAY-01..03.

use std::cell::RefCell;
use std::rc::Rc;

use tantu::platform::{
    AppEvent, FakeLog, FakePlatform, FakeStep, NativeMenuItem, PlatformContext, Tray, WindowId,
};
use tantu::prelude::*;
use tantu::scene::{ImageData, RenderError, RenderReport, Renderer, Resources, Scene};
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

fn image() -> ImageData {
    ImageData::rgba8(2, 2, vec![255; 16]).expect("a 2 × 2 image")
}

/// A window with a tray: "Mute" (a check item), "Hide" (removes the tray) and a click
/// logger; the tooltip follows the mute flag.
fn app(log: &Log) -> App {
    let log = log.clone();
    App::new().window(Window::new("A").size(300.0, 200.0), move || {
        let (muted, shown) = (signal(false), signal(true));
        let log = log.clone();
        Show::new(
            move || shown.get(),
            move || {
                let (l1, l2) = (log.clone(), log.clone());
                SystemTray::new(image())
                    .tooltip(move || if muted.get() { "Muted" } else { "Live" }.to_owned())
                    .menu([
                        MenuItem::new("Mute").checked(muted).on_select(move || {
                            muted.update(|m| *m = !*m);
                            l1.borrow_mut().push("mute".into());
                        }),
                        MenuItem::divider(),
                        MenuItem::new("Hide").on_select(move || shown.set(false)),
                    ])
                    .on_click(move || l2.borrow_mut().push("click".into()))
            },
        )
    })
}

/// (label, id, checked) of a tray's items.
fn items(tray: &Tray) -> Vec<(String, u64, Option<bool>)> {
    tray.menu
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
fn facade_tray_01_tray_menu_and_click() {
    let log: Log = Rc::default();
    let first = run(app(&log), FakePlatform::new());
    let tray = first
        .trays
        .last()
        .cloned()
        .flatten()
        .expect("the tray was set");
    assert_eq!(tray.tooltip.as_deref(), Some("Live"));
    assert_eq!((tray.image.width, tray.image.height), (2, 2));
    let entries = items(&tray);
    assert_eq!(entries[0].0, "Mute");
    assert_eq!(entries[0].2, Some(false));
    assert!(matches!(tray.menu[1], NativeMenuItem::Separator));
    // Choosing Mute runs its action and re-sends the tray (tooltip and check mark follow);
    // a click calls on_click.
    let log: Log = Rc::default();
    let second = run(
        app(&log),
        FakePlatform::new()
            .step(FakeStep::App(AppEvent::TrayMenuItemActivated {
                id: entries[0].1,
            }))
            .step(FakeStep::App(AppEvent::TrayClicked)),
    );
    assert_eq!(*log.borrow(), ["mute", "click"]);
    let last = second.trays.last().cloned().flatten().expect("a tray");
    assert_eq!(last.tooltip.as_deref(), Some("Muted"));
    assert_eq!(items(&last)[0].2, Some(true));
}

#[test]
fn facade_tray_02_removal_and_replacement() {
    // Hide drops the view, which removes the tray.
    let log: Log = Rc::default();
    let first = run(app(&log), FakePlatform::new());
    let hide = items(
        first
            .trays
            .last()
            .cloned()
            .flatten()
            .as_ref()
            .expect("a tray"),
    )[1]
    .1;
    let removed = run(
        app(&log),
        FakePlatform::new().step(FakeStep::App(AppEvent::TrayMenuItemActivated { id: hide })),
    );
    assert_eq!(removed.trays.last(), Some(&None));
    // A second tray replaces the first.
    let two = App::new().window(Window::new("A").size(300.0, 200.0), || {
        Column::new()
            .child(SystemTray::new(image()).tooltip("first"))
            .child(SystemTray::new(image()).tooltip("second"))
    });
    let log = run(two, FakePlatform::new());
    let last = log.trays.last().cloned().flatten().expect("a tray");
    assert_eq!(last.tooltip.as_deref(), Some("second"));
    // Without a tray, nothing is set.
    let none = run(
        app(&Rc::default()),
        FakePlatform::new().without_native_menus(),
    );
    assert!(none.trays.is_empty());
}

#[test]
fn facade_tray_03_opt_in_feature() {
    let manifest = include_str!("../Cargo.toml");
    let line = |name: &str| {
        manifest
            .lines()
            .find(|l| l.starts_with(&format!("{name} = ")))
            .unwrap_or_else(|| panic!("no {name} line"))
            .to_owned()
    };
    assert!(line("tray").contains("tantu-platform-winit/tray"));
    assert!(!line("default").contains("tray"));
}
