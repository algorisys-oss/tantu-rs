//! Tests for `docs/specs/widgets/dock-drag.md`, rules WIDGETS-DOCKDRAG-01..07.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::{Point, Rect};
use tantu_view::layout::RenderDock;
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::{
    ElementId, LogicalKey, Modifiers, NamedKey, PointerButton, PointerEvent, PointerKind,
};
use tantu_widgets::{
    DockArea, DockGroupId, DockLayout, DockNode, DockSide, DockTarget, PanelId, Text,
};

const W: f32 = 600.0;
const H: f32 = 400.0;

fn p(id: &str) -> PanelId {
    PanelId::new(id)
}

fn title(id: &PanelId) -> String {
    let mut chars = id.as_str().chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

/// explorer+search | (editor+readme / terminal+problems).
fn ide() -> DockLayout {
    DockLayout::new(DockNode::row([
        (0.25, DockNode::tabs(["explorer", "search"])),
        (
            0.75,
            DockNode::column([
                (0.6, DockNode::tabs(["editor", "readme"])),
                (0.4, DockNode::tabs(["terminal", "problems"])),
            ]),
        ),
    ]))
}

struct Fx {
    t: WidgetTester,
    layout: Signal<DockLayout>,
    active: Signal<Option<DockGroupId>>,
}

fn fixture() -> Fx {
    type Out = Rc<RefCell<Option<(Signal<DockLayout>, Signal<Option<DockGroupId>>)>>>;
    let out: Out = Rc::default();
    let t = {
        let out = out.clone();
        WidgetTester::with_size(W, H, move || {
            let layout = signal(ide());
            let active = signal(None);
            *out.borrow_mut() = Some((layout, active));
            DockArea::new(layout, |id: &PanelId| Text::new(title(id)))
                .title(title)
                .active_group(active)
        })
    };
    let (layout, active) = out.borrow_mut().take().expect("built");
    let mut fx = Fx { t, layout, active };
    fx.pump();
    fx
}

impl Fx {
    fn pump(&mut self) {
        self.t.pump();
        self.t.pump();
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
            .map(|(id, n)| (ElementId::from_raw(id.0).expect("element id"), n))
            .collect()
    }

    fn tab(&mut self, title: &str) -> Rect {
        let id = self
            .nodes(accesskit::Role::Tab)
            .into_iter()
            .find(|(_, n)| n.label() == Some(title))
            .map(|(id, _)| id)
            .unwrap_or_else(|| panic!("no tab {title}"));
        self.t.rect(id)
    }

    /// The strip holding the tab titled `title`.
    fn strip_of(&mut self, title: &str) -> Rect {
        let tab = self.tab(title);
        let strips: Vec<ElementId> = self
            .nodes(accesskit::Role::TabList)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        strips
            .into_iter()
            .map(|id| self.t.rect(id))
            .find(|r| {
                r.top <= tab.center().y
                    && tab.center().y <= r.bottom
                    && r.left <= tab.center().x
                    && tab.center().x <= r.right
            })
            .expect("a strip around the tab")
    }

    fn pointer(&mut self, kind: PointerKind, at: Point) {
        self.t.pointer(PointerEvent {
            kind,
            position: at,
            modifiers: Modifiers::default(),
        });
        self.t.pump();
    }

    /// Presses the tab, moves past the slop, then to `to`; releases there if `release`.
    fn drag(&mut self, title: &str, to: Point, release: bool) {
        let from = self.tab(title).center();
        self.pointer(PointerKind::Down(PointerButton::Primary), from);
        self.pointer(PointerKind::Move, Point::new(from.x + 8.0, from.y));
        self.pointer(PointerKind::Move, to);
        if release {
            self.pointer(PointerKind::Up(PointerButton::Primary), to);
            self.pump();
        }
    }

    fn release(&mut self, at: Point) {
        self.pointer(PointerKind::Up(PointerButton::Primary), at);
        self.pump();
    }

    /// The dock's own fills (the drop indicator), in window coordinates.
    fn indicator(&self) -> Vec<(Rect, tantu_view::core::Color)> {
        let dock = self.t.find(&Finder::render::<RenderDock>());
        let origin = self.t.rect(dock);
        self.t
            .frame()
            .scene
            .entries()
            .iter()
            .filter(|e| e.element == Some(dock))
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

    fn colors(&self) -> tantu_theme::ColorScheme {
        self.t.enter(|| self.t.theme().get().colors)
    }

    fn layer_opacities(&self) -> Vec<f32> {
        self.t
            .frame()
            .scene
            .entries()
            .iter()
            .filter_map(|e| match &e.command {
                Command::PushLayer(layer) => Some(layer.opacity),
                _ => None,
            })
            .collect()
    }

    fn panels(&self, group_of: &str) -> Vec<String> {
        let l = self.get();
        let g = l.group_of(&p(group_of)).expect("in layout");
        l.group(g)
            .expect("group")
            .0
            .iter()
            .map(|p| p.as_str().to_owned())
            .collect()
    }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

#[test]
fn widgets_dockdrag_01_starting_a_drag() {
    let mut fx = fixture();
    let before = fx.get();
    // Within the slop: a click, which activates.
    let search = fx.tab("Search").center();
    fx.pointer(PointerKind::Down(PointerButton::Primary), search);
    fx.pointer(PointerKind::Move, Point::new(search.x + 3.0, search.y));
    fx.release(Point::new(search.x + 3.0, search.y));
    assert_eq!(fx.get().active(fx.group("search")), Some(&p("search")));
    // Past it: a drag; the dragged tab is dimmed and nothing is activated.
    let readme = fx.tab("Readme").center();
    fx.pointer(PointerKind::Down(PointerButton::Primary), readme);
    fx.pointer(
        PointerKind::Move,
        Point::new(readme.x + 10.0, readme.y + 40.0),
    );
    assert!(
        fx.layer_opacities().iter().any(|o| close(*o, 0.38)),
        "{:?}",
        fx.layer_opacities()
    );
    assert_eq!(fx.get().active(fx.group("readme")), Some(&p("editor")));
    fx.pointer(PointerKind::Up(PointerButton::Primary), readme);
    // Secondary and middle buttons never drag.
    let mut fresh = fixture();
    let terminal = fresh.tab("Terminal").center();
    for button in [PointerButton::Secondary, PointerButton::Middle] {
        fresh.pointer(PointerKind::Down(button), terminal);
        fresh.pointer(PointerKind::Move, Point::new(300.0, 100.0));
        assert!(fresh.indicator().is_empty());
        fresh.pointer(PointerKind::Up(button), Point::new(300.0, 100.0));
    }
    let _ = before;
}

#[test]
fn widgets_dockdrag_02_strip_targets() {
    let mut fx = fixture();
    // Over the editor strip, right of its tabs: inserted at the end.
    let strip = fx.strip_of("Editor");
    let readme = fx.tab("Readme");
    let to = Point::new(readme.right + 30.0, strip.center().y);
    fx.drag("Terminal", to, false);
    let primary = fx.colors().primary;
    let line = fx.indicator();
    assert_eq!(line.len(), 1, "{line:?}");
    let (rect, color) = line[0];
    assert_eq!(color, primary);
    assert!(close(rect.width(), 2.0) && close(rect.height(), strip.height()));
    assert!(
        close(rect.center().x, readme.right),
        "{rect:?} vs {readme:?}"
    );
    fx.release(to);
    assert_eq!(fx.panels("editor"), ["editor", "readme", "terminal"]);
    // Reordering within a strip: Editor dropped right of Readme.
    let readme = fx.tab("Readme");
    fx.drag(
        "Editor",
        Point::new(readme.right - 5.0, readme.center().y),
        true,
    );
    assert_eq!(fx.panels("editor"), ["readme", "editor", "terminal"]);
}

#[test]
fn widgets_dockdrag_03_zone_targets() {
    let mut fx = fixture();
    let colors = fx.colors();
    let strip = fx.strip_of("Editor");
    // The editor group's content is below its strip, down to the column's divider.
    let content = Rect::from_ltrb(strip.left, strip.bottom, strip.right, strip.bottom + 120.0);
    // Near the content's right edge (but not the dock's): beside the editor group.
    let to = Point::new(strip.right - 40.0, strip.bottom + 60.0);
    fx.drag("Problems", to, false);
    let zone = fx.indicator();
    assert!(
        zone.iter()
            .any(|(r, c)| *c == colors.primary.with_alpha(0.16)
                && close(r.left, (strip.left + strip.right) / 2.0)
                && close(r.right, strip.right)
                && close(r.top, strip.bottom)),
        "{zone:?}"
    );
    fx.release(to);
    assert_eq!(
        fx.get(),
        DockLayout::new(DockNode::row([
            (0.25, DockNode::tabs(["explorer", "search"])),
            (
                0.75,
                DockNode::column([
                    (
                        0.6,
                        DockNode::row([
                            (0.5, DockNode::tabs(["editor", "readme"])),
                            (0.5, DockNode::tabs(["problems"])),
                        ]),
                    ),
                    (0.4, DockNode::tabs(["terminal"])),
                ]),
            ),
        ]))
    );
    // The centre joins the group, at the end.
    let to = Point::new(strip.left + 100.0, content.center().y);
    fx.drag("Search", to, true);
    assert_eq!(fx.panels("editor"), ["editor", "readme", "search"]);
}

#[test]
fn widgets_dockdrag_04_edge_targets() {
    let mut fx = fixture();
    // Within 16 px of the dock's right edge: a full-height group along it.
    let to = Point::new(W - 6.0, 300.0);
    fx.drag("Problems", to, false);
    let colors = fx.colors();
    let zone = fx.indicator();
    assert!(
        zone.iter()
            .any(|(r, c)| *c == colors.primary.with_alpha(0.16)
                && close(r.left, W * 0.75)
                && close(r.right, W)
                && close(r.top, 0.0)
                && close(r.bottom, H)),
        "{zone:?}"
    );
    fx.release(to);
    assert_eq!(
        fx.get(),
        DockLayout::new(DockNode::row([
            (0.1875, DockNode::tabs(["explorer", "search"])),
            (
                0.5625,
                DockNode::column([
                    (0.6, DockNode::tabs(["editor", "readme"])),
                    (0.4, DockNode::tabs(["terminal"])),
                ]),
            ),
            (0.25, DockNode::tabs(["problems"])),
        ]))
    );
}

#[test]
fn widgets_dockdrag_05_no_target() {
    let mut fx = fixture();
    let before = fx.get();
    // Its own place in its own strip.
    let search = fx.tab("Search");
    fx.drag(
        "Search",
        Point::new(search.center().x + 2.0, search.bottom - 4.0),
        false,
    );
    assert!(fx.indicator().is_empty());
    // Outside the dock, and over a divider (the row's gap is near x 150).
    fx.pointer(PointerKind::Move, Point::new(-20.0, -20.0));
    assert!(fx.indicator().is_empty());
    fx.pointer(PointerKind::Move, Point::new(150.3, 200.0));
    assert!(fx.indicator().is_empty());
    fx.release(Point::new(150.3, 200.0));
    assert_eq!(fx.get(), before);
    // A group's only panel onto its own group's centre.
    fx.drag("Terminal", Point::new(10.0, 10.0), true);
    let tab = fx.tab("Problems").center();
    let content = Point::new(tab.x, tab.y + 80.0);
    fx.drag("Problems", content, false);
    assert!(fx.indicator().is_empty(), "{:?}", fx.indicator());
    fx.release(content);
}

#[test]
fn widgets_dockdrag_06_drop_and_cancel() {
    let mut fx = fixture();
    // A drop activates the panel in its new group, which becomes active and focused.
    let strip = fx.strip_of("Explorer");
    let to = Point::new(strip.right - 5.0, strip.center().y);
    fx.drag("Terminal", to, true);
    let group = fx.group("terminal");
    assert_eq!(fx.get().active(group), Some(&p("terminal")));
    assert_eq!(fx.t.enter(|| fx.active.get_untracked()), Some(group));
    let focus = ElementId::from_raw(fx.t.a11y_tree().focus.0).expect("focused");
    assert_eq!(fx.t.rect(focus), strip);
    // Escape cancels.
    let before = fx.get();
    let strip = fx.strip_of("Editor");
    let to = Point::new(strip.right - 5.0, strip.center().y);
    fx.drag("Problems", to, false);
    assert!(!fx.indicator().is_empty());
    fx.t.press_key(LogicalKey::Named(NamedKey::Escape), Modifiers::default());
    fx.pump();
    assert!(fx.indicator().is_empty());
    fx.release(to);
    assert_eq!(fx.get(), before);
    // The panel leaving the layout cancels too.
    fx.drag("Readme", Point::new(20.0, 200.0), false);
    let layout = fx.layout;
    fx.t.enter(|| layout.update(|l| l.close(&p("readme")).expect("known")));
    fx.pump();
    assert!(fx.indicator().is_empty());
    let after_close = fx.get();
    fx.release(Point::new(20.0, 200.0));
    assert_eq!(fx.get(), after_close);
    // No target: nothing changes.
    let before = fx.get();
    fx.drag("Search", Point::new(-30.0, 10.0), true);
    assert_eq!(fx.get(), before);
}

#[test]
fn widgets_dockdrag_07_keyboard_reorder() {
    let mut fx = fixture();
    let editor = fx.tab("Editor").center();
    fx.pointer(PointerKind::Down(PointerButton::Primary), editor);
    fx.release(editor);
    let chord = Modifiers {
        control: true,
        shift: true,
        ..Default::default()
    };
    fx.t.press_key(LogicalKey::Named(NamedKey::ArrowRight), chord);
    fx.pump();
    assert_eq!(fx.panels("editor"), ["readme", "editor"]);
    assert_eq!(fx.get().active(fx.group("editor")), Some(&p("editor")));
    // At the end: nothing moves.
    fx.t.press_key(LogicalKey::Named(NamedKey::ArrowRight), chord);
    fx.pump();
    assert_eq!(fx.panels("editor"), ["readme", "editor"]);
    fx.t.press_key(LogicalKey::Named(NamedKey::ArrowLeft), chord);
    fx.pump();
    assert_eq!(fx.panels("editor"), ["editor", "readme"]);
    // Still on the strip.
    let focus = ElementId::from_raw(fx.t.a11y_tree().focus.0).expect("focused");
    assert_eq!(fx.t.rect(focus), fx.strip_of("Editor"));
    let _ = (DockSide::Left, DockTarget::Edge(DockSide::Left));
}
