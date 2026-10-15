//! Tests for `docs/specs/widgets/dialog.md`, rules WIDGETS-DIALOG-01..05.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::{Color, Point, Rect};
use tantu_view::layout::CrossAxisAlignment;
use tantu_view::scene::Command;
use tantu_view::{
    ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
};
use tantu_widgets::{AlertDialog, Button, Column, Dialog, DialogController, SizedBox, Text};

const W: f32 = 400.0;
const H: f32 = 300.0;

type Log = Rc<RefCell<Vec<String>>>;

struct Fx {
    t: WidgetTester,
    controller: DialogController,
    log: Log,
}

/// "Open" and "Behind" buttons at the top-left, and a "Delete file?" alert with Cancel and
/// OK (each closing it).
fn fixture(dismissible: bool) -> Fx {
    let log: Log = Rc::default();
    let out: Rc<RefCell<Option<DialogController>>> = Rc::default();
    let t = {
        let (log, out) = (log.clone(), out.clone());
        WidgetTester::with_size(W, H, move || {
            let controller = DialogController::new();
            *out.borrow_mut() = Some(controller);
            let push = |log: &Log, s: &str| {
                let (log, s) = (log.clone(), s.to_owned());
                move || log.borrow_mut().push(s.clone())
            };
            let behind = push(&log, "behind");
            let dismissed = push(&log, "dismissed");
            let dialog_log = log.clone();
            Column::new()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .child(Keyed::new(
                    "open",
                    Button::new("Open").on_press(move || controller.open()),
                ))
                .child(Keyed::new("behind", Button::new("Behind").on_press(behind)))
                .child(
                    Dialog::new(controller, move || {
                        let (cancel, ok) = (push(&dialog_log, "cancel"), push(&dialog_log, "ok"));
                        AlertDialog::new()
                            .title("Delete file?")
                            .content(Text::new("It can't be undone."))
                            .action(Keyed::new(
                                "cancel",
                                Button::new("Cancel").on_press(move || {
                                    controller.close();
                                    cancel();
                                }),
                            ))
                            .action(Keyed::new(
                                "ok",
                                Button::new("OK").on_press(move || {
                                    controller.close();
                                    ok();
                                }),
                            ))
                    })
                    .dismissible(dismissible)
                    .on_dismiss(dismissed),
                )
        })
    };
    let controller = out.borrow_mut().take().expect("built");
    let mut fx = Fx { t, controller, log };
    fx.pump();
    fx
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("element id")
}

impl Fx {
    fn pump(&mut self) {
        self.t.pump();
        self.t.pump();
    }

    fn open(&mut self) {
        let c = self.controller;
        self.t.enter(|| c.open());
        self.pump();
    }

    fn is_open(&self) -> bool {
        self.t.enter(|| self.controller.is_open())
    }

    fn dialog(&mut self) -> Option<(ElementId, accesskit::Node)> {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .find(|(_, n)| {
                matches!(
                    n.role(),
                    accesskit::Role::Dialog | accesskit::Role::AlertDialog
                )
            })
            .map(|(id, n)| (element(id), n))
    }

    fn click(&mut self, at: Point) {
        for kind in [
            PointerKind::Down(PointerButton::Primary),
            PointerKind::Up(PointerButton::Primary),
        ] {
            self.t.pointer(PointerEvent {
                kind,
                position: at,
                modifiers: Modifiers::default(),
            });
        }
        self.pump();
    }

    fn key(&mut self, key: NamedKey, modifiers: Modifiers) {
        self.t.press_key(LogicalKey::Named(key), modifiers);
        self.pump();
    }

    fn focused(&mut self) -> Option<ElementId> {
        ElementId::from_raw(self.t.a11y_tree().focus.0)
    }

    fn rect(&self, key: &str) -> Rect {
        self.t.rect(self.t.find(&Finder::key(key)))
    }

    fn log(&self) -> Vec<String> {
        self.log.borrow().clone()
    }
}

#[test]
fn widgets_dialog_01_barrier_and_centred_content() {
    let mut fx = fixture(true);
    // Closed: nothing, and the dialog takes no space.
    assert!(fx.dialog().is_none());
    assert_eq!(fx.rect("behind").top, fx.rect("open").bottom);
    fx.open();
    let (id, _) = fx.dialog().expect("open");
    let dialog = fx.t.rect(id);
    assert!((dialog.center().x - W / 2.0).abs() < 0.5, "{dialog:?}");
    assert!((dialog.center().y - H / 2.0).abs() < 0.5, "{dialog:?}");
    // A barrier over the whole window.
    let barrier = Color::BLACK.with_alpha(0.32);
    let full = Rect::from_ltwh(0.0, 0.0, W, H);
    assert!(fx.t.frame().scene.entries().iter().any(|e| matches!(
        &e.command,
        Command::Fill { shape, color } if *color == barrier && shape.rect == full
    )));
    let c = fx.controller;
    fx.t.enter(|| c.close());
    fx.pump();
    assert!(fx.dialog().is_none());
}

#[test]
fn widgets_dialog_02_modality_and_barrier_presses() {
    let mut fx = fixture(true);
    fx.open();
    // The content behind can't be pressed; a barrier press dismisses.
    let behind = fx.rect("behind").center();
    fx.click(behind);
    assert_eq!(fx.log(), ["dismissed"]);
    assert!(!fx.is_open());
    // A press inside the dialog doesn't dismiss.
    fx.open();
    let (id, _) = fx.dialog().expect("open");
    let inside = Point::new(fx.t.rect(id).left + 5.0, fx.t.rect(id).top + 5.0);
    fx.click(inside);
    assert!(fx.is_open());
    // A non-dismissible dialog ignores barrier presses.
    let mut fx = fixture(false);
    fx.open();
    let behind = fx.rect("behind").center();
    fx.click(behind);
    assert!(fx.is_open());
    assert!(fx.log().is_empty());
}

#[test]
fn widgets_dialog_03_focus() {
    let mut fx = fixture(true);
    fx.t.focus(&Finder::key("open"));
    let open = fx.focused();
    fx.key(NamedKey::Enter, Modifiers::default());
    assert!(fx.is_open());
    let (dialog, _) = fx.dialog().expect("open");
    assert_eq!(fx.focused(), Some(dialog));
    // Tab cycles inside the dialog.
    let (cancel, ok) = (
        fx.t.find(&Finder::key("cancel")),
        fx.t.find(&Finder::key("ok")),
    );
    fx.key(NamedKey::Tab, Modifiers::default());
    assert_eq!(fx.focused(), Some(cancel));
    fx.key(NamedKey::Tab, Modifiers::default());
    assert_eq!(fx.focused(), Some(ok));
    fx.key(NamedKey::Tab, Modifiers::default());
    assert_eq!(fx.focused(), Some(cancel));
    // Escape dismisses; the focus comes back.
    fx.key(NamedKey::Escape, Modifiers::default());
    assert!(!fx.is_open());
    assert_eq!(fx.log(), ["dismissed"]);
    assert_eq!(fx.focused(), open);
    // A non-dismissible dialog ignores Escape.
    let mut fx = fixture(false);
    fx.open();
    fx.key(NamedKey::Escape, Modifiers::default());
    assert!(fx.is_open());
}

#[test]
fn widgets_dialog_04_alert_layout() {
    let mut fx = fixture(true);
    fx.open();
    let (id, _) = fx.dialog().expect("open");
    let dialog = fx.t.rect(id);
    assert!((280.0..=560.0).contains(&dialog.width()), "{dialog:?}");
    let title = fx.t.rect(fx.t.find(&Finder::text("Delete file?")));
    let message = fx.t.rect(fx.t.find(&Finder::text("It can't be undone.")));
    let (cancel, ok) = (fx.rect("cancel"), fx.rect("ok"));
    assert_eq!(
        (title.left, title.top),
        (dialog.left + 24.0, dialog.top + 24.0)
    );
    assert_eq!(message.top, title.bottom + 16.0);
    assert_eq!(ok.right, dialog.right - 24.0);
    assert_eq!(cancel.right + 8.0, ok.left);
    assert_eq!(ok.bottom, dialog.bottom - 24.0);
    assert_eq!(ok.top, message.bottom + 24.0);
}

#[test]
fn widgets_dialog_05_semantics() {
    let mut fx = fixture(true);
    fx.open();
    let (_, node) = fx.dialog().expect("open");
    assert_eq!(node.role(), accesskit::Role::AlertDialog);
    assert!(node.is_modal());
    assert_eq!(node.label(), Some("Delete file?"));
    // Other content: a Dialog named by its label.
    let mut t = WidgetTester::with_size(W, H, || {
        let controller = DialogController::new();
        controller.open();
        Dialog::new(controller, || SizedBox::new(100.0, 50.0)).label("Settings")
    });
    t.pump();
    t.pump();
    let node = t
        .a11y_tree()
        .nodes
        .into_iter()
        .find(|(_, n)| n.role() == accesskit::Role::Dialog)
        .map(|(_, n)| n)
        .expect("a dialog");
    assert_eq!(node.label(), Some("Settings"));
    assert!(node.is_modal());
}
