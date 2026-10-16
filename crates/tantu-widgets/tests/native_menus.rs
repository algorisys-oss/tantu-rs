//! Tests for `docs/specs/platform/native-menus.md`, rule WIDGETS-MENUBAR-05.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::reactive::{Signal, signal};
use tantu_view::{Keyed, Provide};
use tantu_widgets::{
    Column, MenuBar, MenuItem, NativeMenuEntry, NativeMenuEntryKind, NativeMenuHost, NativeMenus,
    Text,
};

/// A host that records what it is given, accepting or declining.
struct Host {
    accept: bool,
    calls: RefCell<Vec<Vec<NativeMenuEntry>>>,
}

impl NativeMenuHost for Host {
    fn set_menus(&self, menus: Vec<NativeMenuEntry>) -> bool {
        self.calls.borrow_mut().push(menus);
        self.accept
    }
}

/// A flat description of entries: "label" for items (with [x]/[ ] for check items and
/// "!" when disabled), "-" for separators, "label{...}" for submenus.
fn describe(entries: &[NativeMenuEntry]) -> String {
    entries
        .iter()
        .map(|e| match &e.kind {
            NativeMenuEntryKind::Separator => "-".to_owned(),
            NativeMenuEntryKind::Submenu(items) => format!("{}{{{}}}", e.label, describe(items)),
            NativeMenuEntryKind::Item => format!(
                "{}{}{}{}",
                if e.enabled { "" } else { "!" },
                match e.checked {
                    Some(true) => "[x]",
                    Some(false) => "[ ]",
                    None => "",
                },
                e.label,
                e.shortcut
                    .as_deref()
                    .map(|s| format!("({s})"))
                    .unwrap_or_default()
            ),
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn widgets_menubar_05_native_host() {
    let host = Rc::new(Host {
        accept: true,
        calls: RefCell::default(),
    });
    let log: Rc<RefCell<Vec<&str>>> = Rc::default();
    type Signals = Rc<std::cell::Cell<Option<(Signal<bool>, Signal<bool>)>>>;
    let out: Signals = Rc::default();
    let mut t = {
        let (host, log, out) = (host.clone(), log.clone(), out.clone());
        WidgetTester::with_size(400.0, 200.0, move || {
            let (wrap, can_save) = (signal(false), signal(true));
            out.set(Some((wrap, can_save)));
            let (l1, l2) = (log.clone(), log.clone());
            Provide::new(
                NativeMenus(host as Rc<dyn NativeMenuHost>),
                Column::new()
                    .child(Keyed::new(
                        "bar",
                        MenuBar::new()
                            .native(true)
                            .menu(
                                "File",
                                [
                                    MenuItem::new("Save")
                                        .shortcut("Ctrl+S")
                                        .enabled(move || can_save.get())
                                        .on_select(move || l1.borrow_mut().push("save")),
                                    MenuItem::divider(),
                                    MenuItem::submenu("Recent", [MenuItem::new("a.txt")]),
                                ],
                            )
                            .menu(
                                "View",
                                [MenuItem::new("Wrap")
                                    .checked(wrap)
                                    .on_select(move || l2.borrow_mut().push("wrap"))],
                            ),
                    ))
                    .child(Keyed::new("below", Text::new("below"))),
            )
        })
    };
    t.pump();
    let (wrap, can_save) = out.get().expect("built");
    // The host got the menus; the bar takes no space and has no semantics node.
    let first = describe(&host.calls.borrow()[0]);
    assert_eq!(first, "File{Save(Ctrl+S),-,Recent{a.txt}},View{[ ]Wrap}");
    assert_eq!(t.rect(t.find(&Finder::key("bar"))).height(), 0.0);
    assert!(
        !t.a11y_tree()
            .nodes
            .iter()
            .any(|(_, n)| n.role() == accesskit::Role::MenuBar)
    );
    assert_eq!(t.rect(t.find(&Finder::key("below"))).top, 0.0);
    // State changes re-send the menus.
    t.enter(|| {
        wrap.set(true);
        can_save.set(false);
    });
    t.pump();
    let last = describe(host.calls.borrow().last().expect("a call"));
    assert_eq!(last, "File{!Save(Ctrl+S),-,Recent{a.txt}},View{[x]Wrap}");
    // Activations call the items' actions.
    let entries = host.calls.borrow().last().cloned().expect("a call");
    let NativeMenuEntryKind::Submenu(view) = &entries[1].kind else {
        panic!("View is a submenu");
    };
    (view[0].on_select.as_ref().expect("an action"))();
    assert_eq!(*log.borrow(), ["wrap"]);

    // A host that declines leaves the bar drawn.
    let declining = Rc::new(Host {
        accept: false,
        calls: RefCell::default(),
    });
    let mut t = {
        let declining = declining.clone();
        WidgetTester::with_size(400.0, 200.0, move || {
            Provide::new(
                NativeMenus(declining as Rc<dyn NativeMenuHost>),
                Column::new().child(Keyed::new(
                    "bar",
                    MenuBar::new()
                        .native(true)
                        .menu("File", [MenuItem::new("Save")]),
                )),
            )
        })
    };
    t.pump();
    assert_eq!(declining.calls.borrow().len(), 1);
    assert_eq!(t.rect(t.find(&Finder::key("bar"))).height(), 40.0);
    // Without `native(true)` the host isn't asked.
    let unused = Rc::new(Host {
        accept: true,
        calls: RefCell::default(),
    });
    let mut t = {
        let unused = unused.clone();
        WidgetTester::with_size(400.0, 200.0, move || {
            Provide::new(
                NativeMenus(unused as Rc<dyn NativeMenuHost>),
                MenuBar::new().menu("File", [MenuItem::new("Save")]),
            )
        })
    };
    t.pump();
    assert!(unused.calls.borrow().is_empty());
}
