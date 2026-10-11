//! Tests for `docs/specs/a11y/semantics.md`, rules VIEW-SEM-01..03.

use std::cell::RefCell;
use std::rc::Rc;

use tantu_core::EdgeInsets;
use tantu_core::{Point, Rect, Size, Vec2};
use tantu_layout::{
    BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex, RenderPadding,
};
use tantu_reactive::{Signal, signal};
use tantu_scene::Scene;
use tantu_view::{
    ActionCx, AnyView, BuildCx, Dyn, ElementId, FocusOptions, Handled, LogicalKey, Modifiers,
    NamedKey, Phase, PointerKind, Prop, Role, Semantics, SemanticsAction, SemanticsActions, View,
    ViewTree,
};

type Log = Rc<RefCell<Vec<String>>>;
/// An action handler a test node installs.
type ActionHandler = Box<dyn Fn(&ActionCx<'_>) -> bool>;

/// A box with semantics, optional focus, and handlers logging what reaches it.
struct Node {
    name: &'static str,
    size: Size,
    semantics: Option<Semantics>,
    focus: Option<FocusOptions>,
    children: Vec<AnyView>,
    padding: f32,
    log: Log,
    /// A reactive name, when given.
    reactive_name: Option<Signal<String>>,
    /// Whether the node handles actions, and which.
    handles: Option<ActionHandler>,
}

fn node(name: &'static str, log: &Log) -> Node {
    Node {
        name,
        size: Size::new(100.0, 20.0),
        semantics: None,
        focus: None,
        children: Vec::new(),
        padding: 0.0,
        log: log.clone(),
        reactive_name: None,
        handles: None,
    }
}

fn button_semantics() -> Semantics {
    Semantics {
        role: Role::Button,
        name: Some("Go".into()),
        focusable: true,
        actions: SemanticsActions {
            click: true,
            focus: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

impl View for Node {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = if self.padding > 0.0 {
            cx.render(
                RenderPadding::new(EdgeInsets::all(self.padding)),
                self.children,
            )
        } else if self.children.is_empty() {
            cx.render(
                RenderConstrainedBox::sized(Some(self.size.width), Some(self.size.height)),
                [],
            )
        } else {
            cx.render(RenderFlex::column(), self.children)
        };
        if let Some(semantics) = self.semantics {
            cx.set_semantics(id, semantics);
        }
        if let Some(name) = self.reactive_name {
            cx.bind(
                id,
                Prop::Dynamic(Box::new(move || name.get())),
                |e, name: String| {
                    e.update_semantics(|s| s.name = Some(name));
                },
            );
        }
        if let Some(options) = self.focus {
            cx.focusable(id, options);
        }
        let (name, log) = (self.name, self.log.clone());
        cx.on_focus_change(id, move |on| {
            log.borrow_mut().push(format!("{name} focus {on}"))
        });
        let (name, log) = (self.name, self.log.clone());
        cx.on_pointer(id, Phase::Bubble, move |p| {
            let kind = match p.event.kind {
                PointerKind::Down(_) => "down",
                PointerKind::Up(_) => "up",
                _ => return Handled::Continue,
            };
            log.borrow_mut()
                .push(format!("{name} {kind} {},{}", p.local.x, p.local.y));
            Handled::Stop
        });
        let (name, log) = (self.name, self.log.clone());
        cx.on_key(id, Phase::Bubble, move |k| {
            if !k.event.pressed {
                return Handled::Continue;
            }
            let key = match &k.event.key {
                LogicalKey::Named(named) => format!("{named:?}"),
                LogicalKey::Character(c) => c.to_string(),
            };
            log.borrow_mut().push(format!("{name} key {key}"));
            Handled::Stop
        });
        if let Some(handler) = self.handles {
            let (name, log) = (self.name, self.log.clone());
            cx.on_action(id, move |cx| {
                let handled = handler(cx);
                log.borrow_mut()
                    .push(format!("{name} action {:?} -> {handled}", cx.action));
                handled
            });
        }
        id
    }
}

fn frame(tree: &mut ViewTree) {
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::tight(Size::new(300.0, 200.0)),
        &mut NoTextMeasure,
        &mut scene,
    );
}

/// The ids of the nodes named in `names`, by their build order (a `Node` records nothing,
/// so tests find them through the tree's children).
fn child(tree: &ViewTree, parent: ElementId, index: usize) -> ElementId {
    tree.children(parent)[index]
}

#[test]
fn view_sem_01_set_update_query_remove() {
    let log: Log = Rc::default();
    let name_out: Rc<RefCell<Option<Signal<String>>>> = Rc::default();
    let show_out: Rc<RefCell<Option<Signal<bool>>>> = Rc::default();
    let mut tree = {
        let (log, name_out, show_out) = (log.clone(), name_out.clone(), show_out.clone());
        ViewTree::new(move || {
            let name = signal("Go".to_owned());
            *name_out.borrow_mut() = Some(name);
            let show = signal(true);
            *show_out.borrow_mut() = Some(show);
            let mut button = node("b", &log);
            button.semantics = Some(button_semantics());
            button.reactive_name = Some(name);
            let plain = node("p", &log);
            let log2 = log.clone();
            let mut col = node("col", &log);
            col.children = vec![
                AnyView::new(button),
                AnyView::new(plain),
                AnyView::new(Dyn::new(move || {
                    if show.get() {
                        let mut b = node("d", &log2);
                        b.semantics = Some(button_semantics());
                        AnyView::new(b)
                    } else {
                        AnyView::new(node("e", &log2))
                    }
                })),
            ];
            col
        })
    };
    let root = tree.root();
    let col = child(&tree, root, 0);
    let (button_id, plain_id, region) = (
        child(&tree, col, 0),
        child(&tree, col, 1),
        child(&tree, col, 2),
    );
    assert_eq!(
        tree.semantics(button_id).map(|s| s.role),
        Some(Role::Button)
    );
    assert_eq!(
        tree.semantics(button_id)
            .and_then(|s| s.name.clone())
            .as_deref(),
        Some("Go")
    );
    assert!(tree.semantics(plain_id).is_none());
    assert!(tree.semantics(root).is_none());
    // A region can't carry semantics.
    let dyn_child = child(&tree, region, 0);
    assert!(tree.semantics(region).is_none());
    assert!(tree.semantics(dyn_child).is_some());
    // Reactive update at the next frame.
    let name = name_out.borrow().expect("built");
    tree.enter(|| name.set("Stop".to_owned()));
    frame(&mut tree);
    assert_eq!(
        tree.semantics(button_id)
            .and_then(|s| s.name.clone())
            .as_deref(),
        Some("Stop")
    );
    // Removal drops it.
    let show = show_out.borrow().expect("built");
    tree.enter(|| show.set(false));
    frame(&mut tree);
    assert!(tree.semantics(dyn_child).is_none());
    assert!(!tree.contains(dyn_child));
}

#[test]
fn view_sem_02_bounds() {
    let log: Log = Rc::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || {
            let mut inner = node("inner", &log);
            inner.size = Size::new(50.0, 10.0);
            let mut padded = node("padded", &log);
            padded.padding = 10.0;
            padded.children = vec![AnyView::new(inner)];
            let mut col = node("col", &log);
            col.children = vec![
                AnyView::new(node("first", &log)),
                AnyView::new(padded),
                AnyView::new(Dyn::new({
                    let log = log.clone();
                    move || {
                        let mut a = node("a", &log);
                        a.size = Size::new(30.0, 5.0);
                        let mut b = node("b", &log);
                        b.size = Size::new(60.0, 5.0);
                        let mut col = node("dyncol", &log);
                        col.children = vec![AnyView::new(a), AnyView::new(b)];
                        col
                    }
                })),
            ];
            col
        })
    };
    frame(&mut tree);
    let root = tree.root();
    let col = child(&tree, root, 0);
    let (first, padded, region) = (
        child(&tree, col, 0),
        child(&tree, col, 1),
        child(&tree, col, 2),
    );
    // The column centers its children (cross axis) in a 300 px wide root.
    assert_eq!(
        tree.bounds(first),
        Some(Rect::from_ltwh(100.0, 0.0, 100.0, 20.0))
    );
    let inner = child(&tree, padded, 0);
    assert_eq!(
        tree.bounds(padded),
        Some(Rect::from_ltwh(115.0, 20.0, 70.0, 30.0))
    );
    assert_eq!(
        tree.bounds(inner),
        Some(Rect::from_ltwh(125.0, 30.0, 50.0, 10.0))
    );
    // A region: the union of its children.
    let dyncol = child(&tree, region, 0);
    assert_eq!(tree.bounds(region), tree.bounds(dyncol));
    assert_eq!(
        tree.bounds(dyncol),
        Some(Rect::from_ltwh(120.0, 50.0, 60.0, 10.0))
    );
    assert_eq!(
        tree.bounds(ElementId::from_raw(u64::MAX).expect("non-zero")),
        None
    );
    let _ = (Point::ZERO, Vec2::ZERO);
}

#[test]
fn view_sem_03_perform_action() {
    let log: Log = Rc::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || {
            let mut button = node("b", &log);
            button.semantics = Some(button_semantics());
            button.focus = Some(FocusOptions::default());
            let mut slider = node("s", &log);
            slider.semantics = Some(Semantics {
                role: Role::Slider,
                focusable: true,
                actions: SemanticsActions {
                    increment: true,
                    decrement: true,
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            slider.focus = Some(FocusOptions::default());
            let mut field = node("f", &log);
            field.semantics = Some(Semantics {
                role: Role::TextInput,
                focusable: true,
                actions: SemanticsActions {
                    set_value: true,
                    focus: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            field.focus = Some(FocusOptions::default());
            field.handles = Some(Box::new(|cx| {
                matches!(cx.action, SemanticsAction::SetValue(_))
            }));
            // A scroll-view-like ancestor handling ScrollIntoView for descendants.
            let mut item = node("item", &log);
            item.semantics = Some(Semantics {
                role: Role::ListItem,
                actions: SemanticsActions {
                    scroll_into_view: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            let mut scroll = node("scroll", &log);
            scroll.semantics = Some(Semantics {
                role: Role::ScrollView,
                ..Default::default()
            });
            scroll.handles = Some(Box::new(|cx| {
                // The target is the item, the handler's element the scroll view.
                matches!(cx.action, SemanticsAction::ScrollIntoView)
                    && cx.tree.parent(cx.target) == Some(cx.element)
            }));
            scroll.children = vec![AnyView::new(item)];
            let mut plain = node("plain", &log);
            plain.semantics = Some(Semantics {
                role: Role::Label,
                ..Default::default()
            });
            let mut col = node("col", &log);
            col.children = vec![
                AnyView::new(button),
                AnyView::new(slider),
                AnyView::new(field),
                AnyView::new(scroll),
                AnyView::new(plain),
            ];
            col
        })
    };
    frame(&mut tree);
    let col = child(&tree, tree.root(), 0);
    let (button, slider, field, scroll, plain) = (
        child(&tree, col, 0),
        child(&tree, col, 1),
        child(&tree, col, 2),
        child(&tree, col, 3),
        child(&tree, col, 4),
    );
    let item = child(&tree, scroll, 0);
    let take = |log: &Log| std::mem::take(&mut *log.borrow_mut());
    // Focus.
    assert!(tree.perform_action(button, SemanticsAction::Focus));
    assert_eq!(tree.focused(), Some(button));
    assert_eq!(take(&log), ["b focus true"]);
    // Click: a press and release at the center (50, 10 in the element).
    assert!(tree.perform_action(button, SemanticsAction::Click));
    assert_eq!(take(&log), ["b down 50,10", "b up 50,10"]);
    // Increment/decrement: focus, then arrow keys.
    assert!(tree.perform_action(slider, SemanticsAction::Increment));
    assert_eq!(
        take(&log),
        ["b focus false", "s focus true", "s key ArrowRight"]
    );
    assert!(tree.perform_action(slider, SemanticsAction::Decrement));
    assert_eq!(take(&log), ["s key ArrowLeft"]);
    // SetValue goes to the element's action handler.
    assert!(tree.perform_action(field, SemanticsAction::SetValue("hi".into())));
    let logged = take(&log);
    assert!(
        logged.contains(&"f action SetValue(\"hi\") -> true".to_owned()),
        "{logged:?}"
    );
    // ScrollIntoView bubbles to the scroll view's handler with the target.
    assert!(tree.perform_action(item, SemanticsAction::ScrollIntoView));
    assert_eq!(take(&log), ["scroll action ScrollIntoView -> true"]);
    // Unsupported, missing or removed: false, no panic.
    assert!(!tree.perform_action(plain, SemanticsAction::Click));
    assert!(!tree.perform_action(button, SemanticsAction::Increment));
    // ScrollIntoView on the scroll view itself is `ensure_visible` (VIEW-REVEAL-03): it asks
    // the view's ancestors, none of which handle it, never the view's own handler.
    assert!(!tree.perform_action(scroll, SemanticsAction::ScrollIntoView));
    assert!(take(&log).is_empty());
    assert!(!tree.perform_action(plain, SemanticsAction::ScrollIntoView));
    assert!(!tree.perform_action(
        ElementId::from_raw(u64::MAX).expect("non-zero"),
        SemanticsAction::Click
    ));
    assert!(take(&log).is_empty());
    let _ = (Modifiers::default(), NamedKey::Enter);
}
