//! Tests for `docs/specs/view/offstage.md`, rules VIEW-OFFSTAGE-01..06.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use tantu_core::{Color, Point, Rect, Size};
use tantu_layout::{
    BoxConstraints, LayoutChildren, NoTextMeasure, RenderBox, RenderConstrainedBox, RenderFlex,
};
use tantu_scene::Scene;
use tantu_view::reactive::{Signal, signal};
use tantu_view::{
    Anchor, AnyView, BuildCx, ElementId, FocusOptions, Handled, IntoProp, OverlayPortal, Paint,
    PaintCx, Phase, PointerButton, PointerEvent, PointerKind, Role, Semantics, SemanticsAction,
    View, ViewTree,
};

type Log = Rc<RefCell<Vec<String>>>;
type Ids = Rc<RefCell<HashMap<&'static str, ElementId>>>;

/// Fills its bounds.
struct Fill;

impl Paint for Fill {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene().fill_rect(
            Rect::from_ltwh(0.0, 0.0, size.width, size.height),
            Color::BLACK,
        );
    }
}

/// A painted, focusable 100 × 20 box with a Button node, logging pointer and focus events.
struct Leaf {
    name: &'static str,
    ids: Ids,
    log: Log,
    builds: Rc<Cell<usize>>,
    label: Option<Signal<String>>,
}

impl View for Leaf {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.builds.set(self.builds.get() + 1);
        let id = cx.render(RenderConstrainedBox::sized(Some(100.0), Some(20.0)), []);
        cx.set_paint(id, Fill);
        cx.focusable(
            id,
            FocusOptions {
                traversable: true,
                focus_on_press: true,
            },
        );
        let name = self.name;
        cx.set_semantics(
            id,
            Semantics {
                role: Role::Button,
                name: Some(name.to_owned()),
                focusable: true,
                actions: tantu_view::SemanticsActions {
                    click: true,
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        if let Some(label) = self.label {
            cx.bind(id, label.into_prop(), |e, text| {
                e.update_semantics(|s| s.name = Some(text));
            });
        }
        let log = self.log.clone();
        cx.on_pointer(id, Phase::Bubble, move |p| {
            let kind = match p.event.kind {
                PointerKind::Down(_) => "down",
                PointerKind::Up(_) => "up",
                PointerKind::Enter => "enter",
                PointerKind::Leave => "leave",
                _ => return Handled::Continue,
            };
            log.borrow_mut().push(format!("{name} {kind}"));
            Handled::Stop
        });
        let log = self.log;
        cx.on_focus_change(id, move |focused| {
            log.borrow_mut().push(format!("{name} focus {focused}"));
        });
        self.ids.borrow_mut().insert(name, id);
        id
    }
}

/// A region hidden while `hidden` is true, around `children`; optionally a focus scope.
struct Hide {
    name: &'static str,
    hidden: Signal<bool>,
    scope: bool,
    ids: Ids,
    children: Vec<AnyView>,
}

impl View for Hide {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let children = self.children;
        let id = cx.region(|cx| {
            for child in children {
                child.build(cx);
            }
        });
        cx.bind(id, self.hidden.into_prop(), |e, hidden| {
            e.set_offstage(hidden)
        });
        if self.scope {
            cx.focus_scope(id);
        }
        self.ids.borrow_mut().insert(self.name, id);
        id
    }
}

/// The test tree, a column:
///
/// ```text
/// col
/// ├─ a                  0..20
/// ├─ H  (hidden)        a region
/// │  ├─ b               20..40   label from `label`
/// │  ├─ S  (scope)      a region, `inner` hides it on its own
/// │  │  └─ s            40..60
/// │  └─ p  (portal)     60..80   overlay `pop` below it
/// └─ c                  80..100
/// ```
struct Fixture {
    tree: ViewTree,
    hidden: Signal<bool>,
    inner: Signal<bool>,
    label: Signal<String>,
    ids: Ids,
    log: Log,
    builds: Rc<Cell<usize>>,
    /// How many times the column was laid out.
    layouts: Rc<Cell<usize>>,
}

impl Fixture {
    fn new() -> Fixture {
        let ids = Ids::default();
        let log = Log::default();
        let builds = Rc::new(Cell::new(0));
        let layouts = Rc::new(Cell::new(0));
        let sigs = Rc::new(Cell::new(None));
        let tree = {
            let (ids, log, builds, sigs) = (ids.clone(), log.clone(), builds.clone(), sigs.clone());
            let layouts = layouts.clone();
            ViewTree::new(move || {
                let (hidden, inner, label) = (signal(false), signal(false), signal("b".to_owned()));
                sigs.set(Some((hidden, inner, label)));
                let leaf = |name, label| Leaf {
                    name,
                    ids: ids.clone(),
                    log: log.clone(),
                    builds: builds.clone(),
                    label,
                };
                let popup = {
                    let (ids, log, builds) = (ids.clone(), log.clone(), builds.clone());
                    OverlayPortal::new(leaf("p", None))
                        .visible(true)
                        .anchor(Anchor::below())
                        .overlay(move || Leaf {
                            name: "pop",
                            ids: ids.clone(),
                            log: log.clone(),
                            builds: builds.clone(),
                            label: None,
                        })
                };
                let scope = Hide {
                    name: "S",
                    hidden: inner,
                    scope: true,
                    ids: ids.clone(),
                    children: vec![AnyView::new(leaf("s", None))],
                };
                let hide = Hide {
                    name: "H",
                    hidden,
                    scope: false,
                    ids: ids.clone(),
                    children: vec![
                        AnyView::new(leaf("b", Some(label))),
                        AnyView::new(scope),
                        AnyView::new(popup),
                    ],
                };
                Column(
                    layouts.clone(),
                    vec![
                        AnyView::new(leaf("a", None)),
                        AnyView::new(hide),
                        AnyView::new(leaf("c", None)),
                    ],
                )
            })
        };
        let (hidden, inner, label) = sigs.get().expect("the app ran");
        let mut f = Fixture {
            tree,
            hidden,
            inner,
            label,
            ids,
            log,
            builds,
            layouts,
        };
        f.frame();
        f
    }

    fn id(&self, name: &str) -> ElementId {
        self.ids.borrow()[name]
    }

    fn frame(&mut self) -> Scene {
        let mut scene = Scene::new();
        self.tree.frame(
            BoxConstraints::loose(Size::new(400.0, 300.0)),
            &mut NoTextMeasure,
            &mut scene,
        );
        scene
    }

    fn hide(&mut self, hidden: bool) -> Scene {
        self.tree.enter(|| self.hidden.set(hidden));
        self.frame()
    }

    fn top(&self, name: &str) -> Option<f32> {
        let layout = self.tree.layout_id(self.id(name))?;
        let tree = self.tree.layout_tree();
        tree.offset(layout).map(|o| o.y)
    }

    fn painted(scene: &Scene, id: ElementId) -> bool {
        scene.entries().iter().any(|e| e.element == Some(id))
    }

    fn pointer(&mut self, kind: PointerKind, x: f32, y: f32) {
        self.tree.dispatch_pointer(PointerEvent {
            modifiers: Default::default(),
            kind,
            position: Point::new(x, y),
        });
    }

    fn take_log(&self) -> Vec<String> {
        std::mem::take(&mut *self.log.borrow_mut())
    }
}

/// A column of children that counts its layouts.
struct Column(Rc<Cell<usize>>, Vec<AnyView>);

struct CountingColumn(Rc<Cell<usize>>, RenderFlex);

impl RenderBox for CountingColumn {
    fn perform_layout(
        &mut self,
        constraints: BoxConstraints,
        children: &mut LayoutChildren<'_>,
    ) -> Size {
        self.0.set(self.0.get() + 1);
        self.1.perform_layout(constraints, children)
    }
}

impl View for Column {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(CountingColumn(self.0, RenderFlex::column()), self.1)
    }
}

#[test]
fn view_offstage_01_hidden_content_is_kept_and_stays_live() {
    let mut f = Fixture::new();
    let (b, s, builds) = (f.id("b"), f.id("s"), f.builds.get());
    f.hide(true);
    assert!(f.tree.contains(b) && f.tree.contains(s), "elements kept");
    f.tree.enter(|| f.label.set("changed".to_owned()));
    f.frame();
    f.hide(false);
    assert!(f.tree.contains(b));
    assert_eq!(f.id("b"), b, "same element");
    assert_eq!(f.builds.get(), builds, "nothing rebuilt");
    assert_eq!(
        f.tree.semantics(b).and_then(|s| s.name.clone()).as_deref(),
        Some("changed"),
        "updates made while hidden were applied"
    );
    // Removing a hidden subtree disposes it like any other.
    f.hide(true);
    let removed = f.tree.remove(f.id("H"));
    assert!(removed >= 4);
    assert!(!f.tree.contains(b));
}

#[test]
fn view_offstage_02_hidden_content_takes_no_layout() {
    let mut f = Fixture::new();
    assert_eq!(f.top("c"), Some(80.0));
    f.hide(true);
    assert_eq!(
        f.top("c"),
        Some(20.0),
        "the column lays out without the region"
    );
    let col = f.tree.parent(f.id("a")).expect("column");
    let layout = f.tree.layout_id(col).expect("render");
    // The column (MainAxisSize::Max) keeps its height; only a and c are its layout children.
    assert_eq!(f.tree.layout_tree().children(layout).len(), 2);
    f.hide(false);
    assert_eq!(f.top("c"), Some(80.0), "re-attached in tree order");
    assert_eq!(f.top("b"), Some(20.0));
}

#[test]
fn view_offstage_03_hidden_content_is_not_painted_or_hit() {
    let mut f = Fixture::new();
    let (b, c) = (f.id("b"), f.id("c"));
    // Hover b, then hide it: it gets Leave.
    f.pointer(PointerKind::Move, 50.0, 30.0);
    assert_eq!(f.take_log(), ["b enter"]);
    let scene = f.hide(true);
    assert!(!Fixture::painted(&scene, b));
    assert!(Fixture::painted(&scene, c));
    assert_eq!(f.take_log(), ["b leave"]);
    let hit = f.tree.hit_test(Point::new(50.0, 30.0));
    assert!(!hit.contains(&b), "{hit:?}");
    assert!(hit.contains(&c), "c moved up into b's place");
    // A capture through a subtree that goes offstage is released without an Up.
    f.hide(false);
    f.take_log();
    f.pointer(PointerKind::Down(PointerButton::Primary), 50.0, 30.0);
    assert!(f.take_log().contains(&"b down".to_owned()));
    f.hide(true);
    f.take_log();
    f.pointer(PointerKind::Up(PointerButton::Primary), 50.0, 30.0);
    let log = f.take_log();
    assert!(!log.contains(&"b up".to_owned()), "{log:?}");
}

#[test]
fn view_offstage_04_hidden_content_takes_no_focus() {
    let mut f = Fixture::new();
    let (a, b, c, s, scope) = (f.id("a"), f.id("b"), f.id("c"), f.id("s"), f.id("S"));
    assert!(f.tree.focus(b));
    f.take_log();
    f.hide(true);
    assert_eq!(f.tree.focused(), None, "focus inside is cleared");
    assert_eq!(f.take_log(), ["b focus false"]);
    assert!(!f.tree.focus(b), "can't focus offstage");
    assert!(f.tree.focus(a));
    assert_eq!(f.tree.focus_next(), Some(c), "traversal skips b, s and p");
    assert_eq!(f.tree.focus_previous(), Some(a));
    // A press on b's old place focuses c, not b.
    f.tree.unfocus();
    f.pointer(PointerKind::Down(PointerButton::Primary), 50.0, 30.0);
    assert_eq!(f.tree.focused(), Some(c));
    // A scope inside keeps its memory for after.
    f.hide(false);
    assert!(f.tree.focus(s));
    f.hide(true);
    assert_eq!(f.tree.focused(), None);
    assert_eq!(
        f.tree.focus_into(scope),
        None,
        "nothing to focus while hidden"
    );
    f.hide(false);
    assert_eq!(f.tree.focus_into(scope), Some(s), "remembered");
}

#[test]
fn view_offstage_05_actions_reveal_and_overlays() {
    let mut f = Fixture::new();
    let (b, pop) = (f.id("b"), f.id("pop"));
    let scene = f.frame();
    assert!(
        Fixture::painted(&scene, pop),
        "the overlay shows while its target does"
    );
    let builds = f.builds.get();
    let scene = f.hide(true);
    assert!(!f.tree.perform_action(b, SemanticsAction::Focus));
    assert!(!f.tree.perform_action(b, SemanticsAction::Click));
    assert!(!f.tree.ensure_visible(b));
    assert_eq!(f.tree.focused(), None);
    assert!(
        !Fixture::painted(&scene, pop),
        "the overlay hides with its target"
    );
    assert!(f.tree.is_offstage(pop));
    let pop_rect = f.tree.bounds(pop).expect("laid out before");
    let hit = f.tree.hit_test(pop_rect.center());
    assert!(!hit.contains(&pop));
    let scene = f.hide(false);
    assert!(Fixture::painted(&scene, pop), "shown again");
    assert_eq!(f.id("pop"), pop, "not rebuilt");
    assert_eq!(f.builds.get(), builds);
}

#[test]
fn view_offstage_06_flags_compose() {
    let mut f = Fixture::new();
    let (a, b, s, scope) = (f.id("a"), f.id("b"), f.id("s"), f.id("S"));
    assert!(!f.tree.is_offstage(a) && !f.tree.is_offstage(b));
    f.tree.enter(|| f.inner.set(true));
    f.frame();
    assert!(f.tree.is_offstage(scope) && f.tree.is_offstage(s));
    assert!(!f.tree.is_offstage(b), "only the inner region");
    assert_eq!(f.top("p"), Some(40.0), "p moved up into s's place");
    f.tree.enter(|| f.inner.set(false));
    f.hide(true);
    assert!(
        f.tree.is_offstage(b) && f.tree.is_offstage(s),
        "inherited from H"
    );
    // Setting the same value again lays nothing out.
    let layouts = f.layouts.get();
    f.hide(true);
    assert_eq!(f.layouts.get(), layouts);
    // A removed id isn't offstage.
    f.tree.remove(f.id("H"));
    assert!(!f.tree.is_offstage(b));
}
