//! Tests for `docs/specs/widgets/dock-area.md`, rules WIDGETS-DOCKAREA-01..09.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::{Point, Rect, Vec2};
use tantu_view::layout::RenderConstrainedBox;
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::{
    BuildCx, CursorIcon, ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton,
    PointerEvent, PointerKind, View,
};
use tantu_widgets::{
    DockArea, DockGroupId, DockLayout, DockNode, DockSide, DockTarget, DockView, PanelId,
    TextEditingController, TextField,
};

const W: f32 = 600.0;
const H: f32 = 400.0;

fn p(id: &str) -> PanelId {
    PanelId::new(id)
}

/// "Explorer" for "explorer".
fn title(id: &PanelId) -> String {
    let mut chars = id.as_str().chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

/// explorer+search | (editor / terminal+problems, problems active).
fn ide() -> DockLayout {
    DockLayout::new(DockNode::row([
        (0.25, DockNode::tabs(["explorer", "search"])),
        (
            0.75,
            DockNode::column([
                (0.6, DockNode::tabs(["editor"])),
                (
                    0.4,
                    DockNode::Tabs {
                        panels: vec![p("terminal"), p("problems")],
                        active: 1,
                    },
                ),
            ]),
        ),
    ]))
}

/// A box filling its panel that counts its builds.
struct Counted {
    name: String,
    builds: Rc<RefCell<HashMap<String, usize>>>,
}

impl View for Counted {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        *self.builds.borrow_mut().entry(self.name).or_default() += 1;
        cx.render(RenderConstrainedBox::expand(), [])
    }
}

struct Fx {
    t: WidgetTester,
    layout: Signal<DockLayout>,
    active: Signal<Option<DockGroupId>>,
    builds: Rc<RefCell<HashMap<String, usize>>>,
    veto: Rc<Cell<bool>>,
    asked: Rc<RefCell<Vec<String>>>,
    editor: TextEditingController,
}

fn fixture_with(initial: DockLayout, placeholder: bool) -> Fx {
    let builds: Rc<RefCell<HashMap<String, usize>>> = Rc::default();
    let veto = Rc::new(Cell::new(false));
    let asked: Rc<RefCell<Vec<String>>> = Rc::default();
    type Out = Rc<
        RefCell<
            Option<(
                Signal<DockLayout>,
                Signal<Option<DockGroupId>>,
                TextEditingController,
            )>,
        >,
    >;
    let out: Out = Rc::default();
    let t = {
        let (builds, veto, asked, out) = (builds.clone(), veto.clone(), asked.clone(), out.clone());
        WidgetTester::with_size(W, H, move || {
            let layout = signal(initial);
            let active = signal(None);
            let editor = TextEditingController::new("");
            *out.borrow_mut() = Some((layout, active, editor));
            let mut area = DockArea::new(layout, move |id: &PanelId| {
                if id.as_str() == "editor" {
                    tantu_view::AnyView::new(Keyed::new("editor-field", TextField::new(editor)))
                } else {
                    tantu_view::AnyView::new(Keyed::new(
                        format!("content-{}", id.as_str()),
                        Counted {
                            name: id.as_str().to_owned(),
                            builds: builds.clone(),
                        },
                    ))
                }
            })
            .title(title)
            .closable(|id| id.as_str() != "explorer")
            .on_close_request(move |id| {
                asked.borrow_mut().push(id.as_str().to_owned());
                !veto.get()
            })
            .active_group(active);
            if placeholder {
                area = area.placeholder(|| Keyed::new("empty", RenderBoxed));
            }
            area
        })
    };
    let (layout, active, editor) = out.borrow_mut().take().expect("built");
    let mut fx = Fx {
        t,
        layout,
        active,
        builds,
        veto,
        asked,
        editor,
    };
    fx.pump();
    fx
}

/// A box filling its constraints (the placeholder).
struct RenderBoxed;
impl View for RenderBoxed {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderConstrainedBox::expand(), [])
    }
}

fn fixture() -> Fx {
    fixture_with(ide(), true)
}

fn element(id: accesskit::NodeId) -> ElementId {
    ElementId::from_raw(id.0).expect("node ids are element ids")
}

impl Fx {
    fn pump(&mut self) {
        self.t.pump();
        self.t.pump();
    }

    fn update(&mut self, f: impl FnOnce(&mut DockLayout)) {
        let layout = self.layout;
        self.t.enter(|| layout.update(f));
        self.pump();
    }

    fn get(&self) -> DockLayout {
        let layout = self.layout;
        self.t.enter(|| layout.get_untracked())
    }

    fn group(&self, panel: &str) -> DockGroupId {
        self.get().group_of(&p(panel)).expect("in layout")
    }

    fn nodes(&mut self, role: accesskit::Role) -> Vec<(ElementId, accesskit::Node)> {
        self.t
            .a11y_tree()
            .nodes
            .into_iter()
            .filter(|(_, n)| n.role() == role)
            .map(|(id, n)| (element(id), n))
            .collect()
    }

    fn names(&mut self, role: accesskit::Role) -> Vec<String> {
        let mut names: Vec<String> = self
            .nodes(role)
            .iter()
            .map(|(_, n)| n.label().unwrap_or_default().to_owned())
            .collect();
        names.sort();
        names
    }

    /// The titles of the onstage panels, sorted.
    fn shown(&mut self) -> Vec<String> {
        self.names(accesskit::Role::TabPanel)
    }

    /// The tab titled `title` (its panel and label carry the same name).
    fn tab(&mut self, title: &str) -> ElementId {
        self.nodes(accesskit::Role::Tab)
            .into_iter()
            .find(|(_, n)| n.label() == Some(title))
            .map(|(id, _)| id)
            .unwrap_or_else(|| panic!("no tab {title}"))
    }

    fn center(&mut self, title: &str) -> Point {
        let id = self.tab(title);
        self.t.rect(id).center()
    }

    fn click(&mut self, at: Point, button: PointerButton) {
        for kind in [PointerKind::Down(button), PointerKind::Up(button)] {
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

    fn focused_role(&mut self) -> Option<accesskit::Role> {
        let tree = self.t.a11y_tree();
        let focus = tree.focus;
        tree.nodes
            .iter()
            .find(|(id, _)| *id == focus)
            .map(|(_, n)| n.role())
    }

    fn content(&self, panel: &str) -> Vec<ElementId> {
        self.t.find_all(&Finder::key(format!("content-{panel}")))
    }

    fn builds(&self, panel: &str) -> usize {
        self.builds.borrow().get(panel).copied().unwrap_or(0)
    }
}

fn control() -> Modifiers {
    Modifiers {
        control: true,
        ..Default::default()
    }
}

#[test]
fn widgets_dockarea_01_panels_follow_the_layout_and_keep_their_state() {
    let mut fx = fixture();
    assert_eq!(fx.shown(), ["Editor", "Explorer", "Problems"]);
    // Every panel in the layout is built once, the inactive ones too.
    for panel in ["explorer", "search", "terminal", "problems"] {
        assert_eq!(fx.builds(panel), 1, "{panel}");
    }
    // Activating shows another panel; nothing is rebuilt.
    fx.update(|l| l.activate(&p("search")).expect("known"));
    assert_eq!(fx.shown(), ["Editor", "Problems", "Search"]);
    assert_eq!(fx.builds("explorer"), 1);
    // Moving a panel to another group keeps its elements.
    let terminal = fx.content("terminal");
    let editor_group = fx.group("editor");
    fx.update(|l| {
        l.move_panel(
            &p("terminal"),
            DockTarget::Tabs {
                group: editor_group,
                index: 1,
            },
        )
        .expect("moves")
    });
    assert_eq!(fx.content("terminal"), terminal);
    assert_eq!(fx.builds("terminal"), 1);
    assert_eq!(fx.shown(), ["Problems", "Search", "Terminal"]);
    // Text typed in the editor survives switching away and back.
    fx.update(|l| l.activate(&p("editor")).expect("known"));
    fx.t.enter_text(&Finder::key("editor-field"), "hello");
    let field = fx.t.find(&Finder::key("editor-field"));
    fx.update(|l| l.activate(&p("terminal")).expect("known"));
    fx.update(|l| l.activate(&p("editor")).expect("known"));
    assert_eq!(fx.t.find(&Finder::key("editor-field")), field);
    assert_eq!(fx.t.enter(|| fx.editor.text()), "hello");
    // A panel leaving the layout is disposed; coming back, it is built again.
    fx.update(|l| l.close(&p("search")).expect("known"));
    assert!(fx.content("search").is_empty());
    fx.update(|l| {
        l.add("search", DockTarget::Edge(DockSide::Right))
            .expect("adds")
    });
    assert_eq!(fx.builds("search"), 2);
    assert_eq!(fx.builds("explorer"), 1);
}

/// The fills of `element`'s own paint, in window coordinates.
fn fills(t: &WidgetTester, element: ElementId) -> Vec<(Rect, tantu_view::core::Color)> {
    let origin = t.rect(element);
    t.frame()
        .scene
        .entries()
        .iter()
        .filter(|e| e.element == Some(element))
        .filter_map(|e| match &e.command {
            Command::Fill { shape, color } => Some((
                Rect::from_ltrb(
                    shape.rect.left + origin.left,
                    shape.rect.top + origin.top,
                    shape.rect.right + origin.left,
                    shape.rect.bottom + origin.top,
                ),
                *color,
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn widgets_dockarea_02_look() {
    let mut fx = fixture();
    let colors = fx.t.enter(|| fx.t.theme().get().colors);
    // The strips have the container background.
    let strips = fx.nodes(accesskit::Role::TabList);
    assert_eq!(strips.len(), 3);
    for (strip, _) in &strips {
        let rect = fx.t.rect(*strip);
        assert!(
            fills(&fx.t, *strip)
                .iter()
                .any(|(r, c)| *r == rect && *c == colors.surface_container_highest)
        );
    }
    // The active tab: surface background, a 2 px indicator, primary in the active group
    // (the first one at the start) and outline elsewhere.
    let explorer = fx.tab("Explorer");
    let rect = fx.t.rect(explorer);
    let tab_fills = fills(&fx.t, explorer);
    assert!(tab_fills.contains(&(rect, colors.surface)));
    assert!(tab_fills.contains(&(
        Rect::from_ltrb(rect.left, rect.bottom - 2.0, rect.right, rect.bottom),
        colors.primary
    )));
    let editor = fx.tab("Editor");
    let rect = fx.t.rect(editor);
    assert!(fills(&fx.t, editor).contains(&(
        Rect::from_ltrb(rect.left, rect.bottom - 2.0, rect.right, rect.bottom),
        colors.outline
    )));
    // An inactive tab has no background until hovered.
    let search = fx.tab("Search");
    assert!(fills(&fx.t, search).is_empty());
    let center = fx.center("Search");
    fx.t.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: center,
        modifiers: Modifiers::default(),
    });
    fx.pump();
    let rect = fx.t.rect(search);
    assert_eq!(
        fills(&fx.t, search),
        [(rect, colors.on_surface.with_alpha(0.08))]
    );
    // An overflowing strip scrolls to reveal the active tab.
    let group = fx.group("explorer");
    fx.update(|l| {
        for i in 0..8 {
            l.add(format!("panel{i}"), DockTarget::Tabs { group, index: 99 })
                .expect("adds");
        }
    });
    let strip = fx.nodes(accesskit::Role::TabList)[0].0;
    let strip_rect = fx.t.rect(strip);
    let last = fx.tab("Panel7");
    let last = fx.t.rect(last);
    assert!(
        last.right <= strip_rect.right + 0.5,
        "{last:?} in {strip_rect:?}"
    );
    assert!(last.left >= strip_rect.left - 0.5);
}

#[test]
fn widgets_dockarea_03_pointer() {
    let mut fx = fixture();
    // A tab press activates its panel and focuses its strip.
    let at = fx.center("Search");
    fx.click(at, PointerButton::Primary);
    assert_eq!(fx.shown(), ["Editor", "Problems", "Search"]);
    assert_eq!(fx.focused_role(), Some(accesskit::Role::TabList));
    // The close button closes, after asking.
    fx.t.tap(&Finder::semantics_label("Close Problems"));
    fx.pump();
    assert!(!fx.get().contains(&p("problems")));
    assert_eq!(*fx.asked.borrow(), ["problems"]);
    // A middle click closes a closable tab.
    let at = fx.center("Terminal");
    fx.click(at, PointerButton::Middle);
    assert!(!fx.get().contains(&p("terminal")));
    // Not when the app says no, and never a tab that isn't closable.
    fx.veto.set(true);
    let at = fx.center("Editor");
    fx.click(at, PointerButton::Middle);
    assert!(fx.get().contains(&p("editor")));
    let at = fx.center("Explorer");
    fx.click(at, PointerButton::Middle);
    assert!(fx.get().contains(&p("explorer")));
    assert_eq!(*fx.asked.borrow(), ["problems", "terminal", "editor"]);
    assert!(
        fx.t.find_all(&Finder::semantics_label("Close Explorer"))
            .is_empty()
    );
}

#[test]
fn widgets_dockarea_04_active_group() {
    let mut fx = fixture();
    let first = fx.group("explorer");
    assert_eq!(fx.t.enter(|| fx.active.get_untracked()), Some(first));
    // A press in a group's content makes it active.
    let problems = fx.content("problems")[0];
    let at = fx.t.rect(problems).center();
    fx.click(at, PointerButton::Primary);
    let group = fx.group("problems");
    assert_eq!(fx.t.enter(|| fx.active.get_untracked()), Some(group));
    // Focus moving into a group makes it active too.
    fx.t.focus(&Finder::key("editor-field"));
    fx.pump();
    assert_eq!(
        fx.t.enter(|| fx.active.get_untracked()),
        Some(fx.group("editor"))
    );
    // When the active group leaves, the first group takes over; empty: none.
    fx.update(|l| l.close(&p("editor")).expect("known"));
    assert_eq!(fx.t.enter(|| fx.active.get_untracked()), Some(first));
    fx.update(|l| *l = DockLayout::empty());
    assert_eq!(fx.t.enter(|| fx.active.get_untracked()), None);
}

#[test]
fn widgets_dockarea_05_keys() {
    let mut fx = fixture();
    let at = fx.center("Explorer");
    fx.click(at, PointerButton::Primary);
    // Arrows on a focused strip, wrapping; Home and End.
    fx.key(NamedKey::ArrowRight, Modifiers::default());
    assert!(fx.shown().contains(&"Search".to_owned()));
    fx.key(NamedKey::ArrowRight, Modifiers::default());
    assert!(fx.shown().contains(&"Explorer".to_owned()));
    fx.key(NamedKey::End, Modifiers::default());
    assert!(fx.shown().contains(&"Search".to_owned()));
    fx.key(NamedKey::Home, Modifiers::default());
    assert!(fx.shown().contains(&"Explorer".to_owned()));
    // From inside a panel: Control+Tab and Control+PageUp cycle the active group's tabs.
    let at = fx.t.rect(fx.content("problems")[0]).center();
    fx.click(at, PointerButton::Primary);
    fx.key(NamedKey::Tab, control());
    assert!(fx.shown().contains(&"Terminal".to_owned()));
    fx.key(NamedKey::PageUp, control());
    assert!(fx.shown().contains(&"Problems".to_owned()));
    // Control+W and Control+F4 close the active group's active panel.
    fx.key(NamedKey::Tab, control());
    fx.t.press_key(LogicalKey::Character("w".into()), control());
    fx.pump();
    assert!(!fx.get().contains(&p("terminal")));
    fx.key(NamedKey::F(4), control());
    assert!(!fx.get().contains(&p("problems")));
}

#[test]
fn widgets_dockarea_06_dividers() {
    let mut fx = fixture();
    let root = |l: &DockLayout| match l.root() {
        Some(DockView::Split { fractions, .. }) => fractions.to_vec(),
        other => panic!("{other:?}"),
    };
    // 600 px with a 1 px gap: the explorer group is 149.75 px.
    fx.t.hover(&Finder::semantics_label("Resize Explorer"));
    assert_eq!(fx.t.cursor(), CursorIcon::ResizeHorizontal);
    fx.t.drag(
        &Finder::semantics_label("Resize Explorer"),
        Vec2::new(50.0, 0.0),
    );
    fx.pump();
    let f = root(&fx.get());
    assert!((f[0] * 599.0 - 199.75).abs() < 0.01, "{f:?}");
    // Clamped at the 80 px group minimum.
    fx.t.drag(
        &Finder::semantics_label("Resize Explorer"),
        Vec2::new(-300.0, 0.0),
    );
    fx.pump();
    let f = root(&fx.get());
    assert!((f[0] * 599.0 - 80.0).abs() < 0.01, "{f:?}");
    // Keys on a focused divider move it 10 px.
    fx.t.focus(&Finder::semantics_label("Resize Explorer"));
    fx.key(NamedKey::ArrowRight, Modifiers::default());
    let f = root(&fx.get());
    assert!((f[0] * 599.0 - 90.0).abs() < 0.01, "{f:?}");
    // The column's divider is vertical.
    fx.t.hover(&Finder::semantics_label("Resize Editor"));
    assert_eq!(fx.t.cursor(), CursorIcon::ResizeVertical);
}

#[test]
fn widgets_dockarea_07_focus_leaving_hidden_or_closed_panels() {
    let mut fx = fixture();
    let editor_group = fx.group("editor");
    fx.update(|l| {
        l.move_panel(
            &p("terminal"),
            DockTarget::Tabs {
                group: editor_group,
                index: 1,
            },
        )
        .expect("moves")
    });
    fx.update(|l| l.activate(&p("editor")).expect("known"));
    // The editor's field has focus; its panel becomes inactive: the strip takes it.
    fx.t.focus(&Finder::key("editor-field"));
    fx.pump();
    fx.update(|l| l.activate(&p("terminal")).expect("known"));
    assert_eq!(fx.focused_role(), Some(accesskit::Role::TabList));
    // Closing the panel holding focus: its group's strip takes it.
    fx.update(|l| l.activate(&p("editor")).expect("known"));
    fx.t.focus(&Finder::key("editor-field"));
    fx.pump();
    fx.update(|l| l.close(&p("editor")).expect("known"));
    assert_eq!(fx.focused_role(), Some(accesskit::Role::TabList));
    // Closing a group's last panel holding focus: nothing keeps it.
    fx.update(|l| {
        l.add("editor", DockTarget::Edge(DockSide::Bottom))
            .expect("adds")
    });
    fx.t.focus(&Finder::key("editor-field"));
    fx.pump();
    assert!(fx.focused().is_some());
    fx.update(|l| l.close(&p("editor")).expect("known"));
    assert_eq!(fx.focused(), None);
}

#[test]
fn widgets_dockarea_08_semantics() {
    let mut fx = fixture();
    assert_eq!(fx.nodes(accesskit::Role::TabList).len(), 3);
    let tabs = fx.nodes(accesskit::Role::Tab);
    let find = |name: &str| {
        tabs.iter()
            .find(|(_, n)| n.label() == Some(name))
            .map(|(_, n)| n.clone())
            .unwrap_or_else(|| panic!("no tab {name}"))
    };
    let problems = find("Problems");
    assert_eq!(problems.is_selected(), Some(true));
    assert_eq!(problems.position_in_set(), Some(2));
    assert_eq!(problems.size_of_set(), Some(2));
    assert_eq!(find("Terminal").is_selected(), Some(false));
    // Close buttons for closable panels.
    let mut buttons = fx.names(accesskit::Role::Button);
    buttons.retain(|name| name.starts_with("Close "));
    assert_eq!(
        buttons,
        [
            "Close Editor",
            "Close Problems",
            "Close Search",
            "Close Terminal"
        ]
    );
    assert_eq!(fx.shown(), ["Editor", "Explorer", "Problems"]);
    // Dividers as splitters with the boundary's position.
    let splitters = fx.nodes(accesskit::Role::Splitter);
    let mut names: Vec<_> = splitters
        .iter()
        .map(|(_, n)| n.label().unwrap_or_default().to_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["Resize Editor", "Resize Explorer"]);
    let explorer = splitters
        .iter()
        .find(|(_, n)| n.label() == Some("Resize Explorer"))
        .map(|(_, n)| n.numeric_value())
        .expect("splitter");
    assert_eq!(explorer, Some(25.0));
}

#[test]
fn widgets_dockarea_09_placeholder() {
    let mut fx = fixture_with(DockLayout::empty(), true);
    fx.t.find(&Finder::key("empty"));
    assert_eq!(fx.t.rect(fx.t.find(&Finder::key("empty"))).width(), W);
    fx.update(|l| {
        l.add("search", DockTarget::Edge(DockSide::Left))
            .expect("adds")
    });
    assert!(fx.t.find_all(&Finder::key("empty")).is_empty());
    assert_eq!(fx.shown(), ["Search"]);
    fx.update(|l| l.close(&p("search")).expect("known"));
    fx.t.find(&Finder::key("empty"));
    // Without a placeholder, nothing.
    let mut fx = fixture_with(DockLayout::empty(), false);
    assert!(fx.t.find_all(&Finder::key("empty")).is_empty());
    assert!(fx.nodes(accesskit::Role::TabList).is_empty());
}
