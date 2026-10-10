//! Tests for `docs/specs/view/shortcuts.md`, rules VIEW-SHORT-01..06.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_layout::{RenderConstrainedBox, RenderFlex};
use tantu_view::{
    Actions, AnyView, BuildCx, ElementId, FocusOptions, Handled, Key, KeyEvent, Keyed, LogicalKey,
    Modifiers, NamedKey, Phase, Shortcuts, SingleActivator, View, ViewTree,
};

type Log = Rc<RefCell<Vec<String>>>;

#[derive(Clone)]
struct Save;
#[derive(Clone)]
struct Close;
#[derive(Clone)]
struct Export;
#[derive(Clone)]
struct Noop;
#[derive(Clone)]
struct SelectAll(&'static str);

const CONTROL: Modifiers = Modifiers {
    shift: false,
    control: true,
    alt: false,
    super_key: false,
};

fn ch(c: &str) -> LogicalKey {
    LogicalKey::Character(c.into())
}

#[test]
fn view_short_01_activators() {
    let save = SingleActivator::character("s").control();
    assert!(save.accepts(&KeyEvent::press(ch("s"), CONTROL)));
    assert!(save.accepts(&KeyEvent::press(ch("S"), CONTROL)));
    assert!(!save.accepts(&KeyEvent::release(ch("s"), CONTROL)));
    assert!(!save.accepts(&KeyEvent::press(ch("s"), Modifiers::default())));
    let ctrl_shift = Modifiers {
        shift: true,
        ..CONTROL
    };
    assert!(!save.accepts(&KeyEvent::press(ch("s"), ctrl_shift)));
    assert!(
        SingleActivator::character("a")
            .shift()
            .accepts(&KeyEvent::press(
                ch("A"),
                Modifiers {
                    shift: true,
                    ..Modifiers::default()
                }
            ))
    );
    let mut repeat = KeyEvent::press(ch("s"), CONTROL);
    repeat.repeat = true;
    assert!(save.accepts(&repeat));
    let escape = SingleActivator::new(LogicalKey::Named(NamedKey::Escape));
    assert!(escape.accepts(&KeyEvent::press(
        LogicalKey::Named(NamedKey::Escape),
        Modifiers::default()
    )));
    assert!(!escape.accepts(&KeyEvent::press(ch("x"), Modifiers::default())));
    // The primary modifier.
    let primary = SingleActivator::character("s").primary();
    let expected = if cfg!(target_os = "macos") {
        SingleActivator::character("s").super_key()
    } else {
        SingleActivator::character("s").control()
    };
    assert_eq!(primary, expected);
    let alt = SingleActivator::character("x").alt();
    assert!(alt.accepts(&KeyEvent::press(
        ch("x"),
        Modifiers {
            alt: true,
            ..Modifiers::default()
        }
    )));
}

/// A 100 × 20 box, focusable.
struct Field;

impl View for Field {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(Some(100.0), Some(20.0)), []);
        cx.focusable(id, FocusOptions::default());
        id
    }
}

struct Column(Vec<AnyView>);

impl View for Column {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderFlex::column(), self.0)
    }
}

/// Registers key loggers on the tree's root (its parent), then builds `child`.
struct RootLogger<V>(V, Log);

impl<V: View> View for RootLogger<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let root = cx.parent();
        for phase in [Phase::Capture, Phase::Bubble] {
            let log = self.1.clone();
            cx.on_key(root, phase, move |k| {
                if k.event.pressed {
                    let tag = if k.phase == Phase::Capture {
                        "capture"
                    } else {
                        "bubble"
                    };
                    log.borrow_mut().push(format!("root {tag}"));
                }
                Handled::Continue
            });
        }
        self.0.build(cx)
    }
}

/// ```text
/// root (key loggers)
/// └─ outer Shortcuts: Ctrl+S → Save, Escape → Close, Ctrl+A → SelectAll("outer"),
///    Ctrl+E → Save, Tab → Noop
///    outer Actions: Save, SelectAll
///    └─ column
///       ├─ inner Shortcuts: Ctrl+A → SelectAll("inner"), Ctrl+E → Export
///       │  inner Actions: SelectAll
///       │  └─ field (focusable)
///       └─ other (focusable)
/// ```
fn fixture() -> (ViewTree, Log) {
    let log = Log::default();
    let l = log.clone();
    let tree = ViewTree::new(move || {
        let (a, b, c) = (l.clone(), l.clone(), l.clone());
        let field =
            Shortcuts::new(Actions::new(Keyed::new("field", Field)).on(
                move |intent: &SelectAll| a.borrow_mut().push(format!("select all {}", intent.0)),
            ))
            .bind(
                SingleActivator::character("a").control(),
                SelectAll("inner"),
            )
            .bind(SingleActivator::character("e").control(), Export);
        let column = Column(vec![
            AnyView::new(field),
            AnyView::new(Keyed::new("other", Field)),
        ]);
        let outer = Shortcuts::new(Keyed::new(
            "outer",
            Actions::new(column)
                .on(move |_: &Save| b.borrow_mut().push("save".to_owned()))
                .on(move |intent: &SelectAll| {
                    c.borrow_mut().push(format!("select all {}", intent.0))
                }),
        ))
        .bind(SingleActivator::character("s").control(), Save)
        .bind(
            SingleActivator::new(LogicalKey::Named(NamedKey::Escape)),
            Close,
        )
        .bind(
            SingleActivator::character("a").control(),
            SelectAll("outer"),
        )
        .bind(SingleActivator::character("e").control(), Save)
        .bind(SingleActivator::new(LogicalKey::Named(NamedKey::Tab)), Noop);
        RootLogger(outer, l)
    });
    (tree, log)
}

fn id(tree: &ViewTree, key: &str) -> ElementId {
    tree.find_key(&Key::from(key))[0]
}

fn take(log: &Log) -> Vec<String> {
    std::mem::take(&mut *log.borrow_mut())
}

fn press(tree: &mut ViewTree, key: LogicalKey, modifiers: Modifiers) -> bool {
    let handled = tree.dispatch_key(KeyEvent::press(key.clone(), modifiers));
    tree.dispatch_key(KeyEvent::release(key, modifiers));
    handled
}

#[test]
fn view_short_02_a_matching_binding_invokes_its_action() {
    let (mut tree, log) = fixture();
    let other = id(&tree, "other");
    tree.focus(other);
    assert!(press(&mut tree, ch("s"), CONTROL));
    // Capture handlers run first; the shortcut stops the bubble phase.
    assert_eq!(take(&log), ["root capture", "save"]);
}

#[test]
fn view_short_03_a_binding_without_an_action_does_nothing() {
    let (mut tree, log) = fixture();
    let (field, other) = (id(&tree, "field"), id(&tree, "other"));
    tree.focus(other);
    press(
        &mut tree,
        LogicalKey::Named(NamedKey::Escape),
        Modifiers::default(),
    );
    assert_eq!(take(&log), ["root capture", "root bubble"]);
    // Tab is bound to an intent nobody handles: it still moves focus.
    tree.focus(field);
    press(
        &mut tree,
        LogicalKey::Named(NamedKey::Tab),
        Modifiers::default(),
    );
    assert_eq!(tree.focused(), Some(other));
}

#[test]
fn view_short_04_the_innermost_binding_wins() {
    let (mut tree, log) = fixture();
    let (field, other) = (id(&tree, "field"), id(&tree, "other"));
    tree.focus(field);
    press(&mut tree, ch("a"), CONTROL);
    assert_eq!(take(&log), ["root capture", "select all inner"]);
    // Not bound inside: the outer binding applies.
    press(&mut tree, ch("s"), CONTROL);
    assert_eq!(take(&log), ["root capture", "save"]);
    // Bound inside to an intent without an action: the outer binding is tried next.
    press(&mut tree, ch("e"), CONTROL);
    assert_eq!(take(&log), ["root capture", "save"]);
    tree.focus(other);
    press(&mut tree, ch("a"), CONTROL);
    assert_eq!(take(&log), ["root capture", "select all outer"]);
}

#[test]
fn view_short_05_invoke() {
    let (mut tree, log) = fixture();
    let (field, other) = (id(&tree, "field"), id(&tree, "other"));
    tree.focus(field);
    assert!(tree.invoke(&SelectAll("from code")));
    assert_eq!(take(&log), ["select all from code"]);
    tree.focus(other);
    assert!(tree.invoke(&Save));
    assert_eq!(take(&log), ["save"]);
    assert!(!tree.invoke(&Close));
    // Nothing focused: only the root's path, which has no actions.
    tree.unfocus();
    assert!(!tree.invoke(&Save));
    assert!(take(&log).is_empty());
}

#[test]
fn view_short_06_removal_drops_bindings_and_actions() {
    let (mut tree, log) = fixture();
    let other = id(&tree, "other");
    tree.focus(other);
    let outer = id(&tree, "outer");
    tree.remove(outer);
    assert!(!tree.invoke(&Save));
    // Ctrl+S now only reaches the root's loggers.
    press(&mut tree, ch("s"), CONTROL);
    assert_eq!(take(&log), ["root capture", "root bubble"]);
}
