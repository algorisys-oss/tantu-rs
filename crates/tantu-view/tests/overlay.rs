//! Tests for `docs/specs/view/overlay.md`, rules VIEW-OVERLAY-01..07.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_core::{Color, Point, Rect, Size};
use tantu_layout::{
    Alignment, BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderStack, StackParentData,
};
use tantu_reactive::{Signal, on_cleanup, signal};
use tantu_scene::{Clip, Command, Scene};
use tantu_view::{
    Anchor, AnyView, BuildCx, ElementId, FocusOptions, Handled, Key, KeyEvent, Keyed, LogicalKey,
    Modifiers, NamedKey, OverlayPointer, OverlayPortal, Paint, PaintCx, Phase, PointerButton,
    PointerEvent, PointerKind, Prop, Show, View, ViewTree,
};

const WINDOW: Size = Size::new(400.0, 300.0);

/// A box of a fixed size (or sized by a signal) that fills itself with a color and can be
/// focusable.
struct Boxed {
    size: Prop<Size>,
    color: Color,
    focusable: bool,
}

fn boxed(w: f32, h: f32) -> Boxed {
    Boxed {
        size: Prop::Value(Size::new(w, h)),
        color: Color::BLACK,
        focusable: false,
    }
}

struct Fill(Color);

impl Paint for Fill {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene()
            .fill_rect(Rect::from_ltwh(0.0, 0.0, size.width, size.height), self.0);
    }
}

impl View for Boxed {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.render(RenderConstrainedBox::sized(None, None), []);
        cx.bind(id, self.size, |e, size: Size| {
            e.update_render::<RenderConstrainedBox>(|r| {
                r.additional = BoxConstraints::tight(size);
            });
        });
        cx.set_paint(id, Fill(self.color));
        if self.focusable {
            cx.focusable(id, FocusOptions::default());
        }
        id
    }
}

/// `child` positioned at (x, y) in a stack.
struct At<V>(f32, f32, V);

impl<V: View> View for At<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.2.build(cx);
        cx.set_parent_data(
            id,
            StackParentData {
                left: Some(self.0),
                top: Some(self.1),
                ..StackParentData::default()
            },
        );
        id
    }
}

/// A window-filling stack that clips its children to its bounds.
struct ClippedStack(Vec<AnyView>);

struct ClipChildren;

impl Paint for ClipChildren {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        cx.scene().push_clip(Clip::Rect(Rect::from_ltwh(
            0.0,
            0.0,
            size.width,
            size.height,
        )));
        cx.paint_children();
        cx.scene().pop();
    }

    fn clips_children(&self) -> bool {
        true
    }
}

impl View for ClippedStack {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let mut children = vec![AnyView::new(boxed(WINDOW.width, WINDOW.height))];
        children.extend(self.0);
        let id = cx.render(RenderStack::new(), children);
        cx.set_paint(id, ClipChildren);
        id
    }
}

fn frame(tree: &mut ViewTree) -> Scene {
    let mut scene = Scene::new();
    tree.frame(
        BoxConstraints::tight(WINDOW),
        &mut NoTextMeasure,
        &mut scene,
    );
    scene
}

fn find(tree: &ViewTree, key: &str) -> Option<ElementId> {
    tree.find_key(&Key::from(key)).first().copied()
}

/// An element's bounds in window coordinates.
fn rect(tree: &ViewTree, id: ElementId) -> Rect {
    let layout = tree.layout_tree();
    let size = layout
        .size(tree.layout_id(id).expect("render element"))
        .expect("laid out");
    let (mut x, mut y) = (0.0, 0.0);
    let mut next = Some(id);
    while let Some(e) = next {
        if let Some(offset) = tree.layout_id(e).and_then(|l| layout.offset(l)) {
            x += offset.x;
            y += offset.y;
        }
        next = tree.parent(e);
    }
    Rect::from_ltwh(x, y, size.width, size.height)
}

/// A tree with a target box (100 × 40) at (`x`, `y`) and a portal showing an 80 × 30 popup.
struct Fixture {
    tree: ViewTree,
    visible: Signal<bool>,
    disposed: Rc<Cell<u32>>,
}

fn fixture(x: f32, y: f32, anchor: Anchor, popup: Size) -> Fixture {
    let handles: Rc<RefCell<Option<Signal<bool>>>> = Rc::default();
    let disposed = Rc::new(Cell::new(0));
    let (h, d) = (handles.clone(), disposed.clone());
    let tree = ViewTree::new(move || {
        let visible = signal(false);
        *h.borrow_mut() = Some(visible);
        let portal = OverlayPortal::new(Keyed::new("target", boxed(100.0, 40.0)))
            .visible(visible)
            .anchor(anchor)
            .overlay(move || {
                let d = d.clone();
                on_cleanup(move || d.set(d.get() + 1));
                Keyed::new("popup", boxed(popup.width, popup.height))
            });
        ClippedStack(vec![AnyView::new(At(x, y, portal))])
    });
    let visible = handles.borrow_mut().take().expect("built");
    Fixture {
        tree,
        visible,
        disposed,
    }
}

fn shown(x: f32, y: f32, anchor: Anchor, popup: Size) -> Fixture {
    let mut f = fixture(x, y, anchor, popup);
    frame(&mut f.tree);
    f.tree.enter(|| f.visible.set(true));
    frame(&mut f.tree);
    f
}

const POPUP: Size = Size::new(80.0, 30.0);

#[test]
fn view_overlay_01_shown_while_visible_at_the_anchor() {
    let mut f = fixture(0.0, 50.0, Anchor::below(), POPUP);
    frame(&mut f.tree);
    assert_eq!(find(&f.tree, "popup"), None);
    f.tree.enter(|| f.visible.set(true));
    frame(&mut f.tree);
    let popup = find(&f.tree, "popup").expect("shown");
    assert_eq!(rect(&f.tree, popup), Rect::from_ltwh(0.0, 90.0, 80.0, 30.0));
    f.tree.enter(|| f.visible.set(false));
    frame(&mut f.tree);
    assert_eq!(find(&f.tree, "popup"), None);
    assert_eq!(f.disposed.get(), 1);

    // Centered on the target, with an offset.
    let f = shown(
        0.0,
        50.0,
        Anchor::new(Alignment::CENTER, Alignment::CENTER).offset(5.0, 7.0),
        POPUP,
    );
    let popup = find(&f.tree, "popup").expect("shown");
    assert_eq!(
        rect(&f.tree, popup),
        Rect::from_ltwh(15.0, 62.0, 80.0, 30.0)
    );
}

#[test]
fn view_overlay_02_painted_last_outside_clips_by_z_index() {
    let mut f = shown(0.0, 50.0, Anchor::below(), POPUP);
    let scene = frame(&mut f.tree);
    let popup = find(&f.tree, "popup").expect("shown");
    let mut clips = 0;
    let mut popup_at = None;
    for (i, entry) in scene.entries().iter().enumerate() {
        match entry.command {
            Command::PushClip(_) => clips += 1,
            Command::PopClip => clips -= 1,
            Command::Fill { .. } if entry.element == Some(popup) => {
                assert_eq!(clips, 0, "the popup is painted outside the content's clip");
                popup_at = Some(i);
            }
            _ => {}
        }
    }
    let popup_at = popup_at.expect("the popup is painted");
    let last_content_fill = scene
        .entries()
        .iter()
        .rposition(|e| matches!(e.command, Command::Fill { .. }) && e.element != Some(popup))
        .expect("content fills");
    assert!(popup_at > last_content_fill);

    // z-index: the higher one paints later, whatever the order.
    let mut tree = ViewTree::new(|| {
        let portal = |key: &'static str, z: i32| {
            OverlayPortal::new(boxed(10.0, 10.0))
                .visible(true)
                .z_index(z)
                .overlay(move || Keyed::new(key, boxed(5.0, 5.0)))
        };
        ClippedStack(vec![
            AnyView::new(At(0.0, 0.0, portal("high", 1))),
            AnyView::new(At(50.0, 0.0, portal("low", 0))),
        ])
    });
    let scene = frame(&mut tree);
    let position = |key: &str| {
        let id = find(&tree, key).expect("shown");
        scene
            .entries()
            .iter()
            .position(|e| e.element == Some(id))
            .expect("painted")
    };
    assert!(position("high") > position("low"));
}

#[test]
fn view_overlay_03_flip_and_clamp() {
    // Below would cross the bottom edge: flipped above the target.
    let f = shown(0.0, 250.0, Anchor::below(), POPUP);
    let popup = find(&f.tree, "popup").expect("shown");
    assert_eq!(
        rect(&f.tree, popup),
        Rect::from_ltwh(0.0, 220.0, 80.0, 30.0)
    );
    // To the right would cross the right edge: flipped to the left.
    let right = Anchor::new(Alignment::TOP_RIGHT, Alignment::TOP_LEFT);
    let f = shown(300.0, 0.0, right, POPUP);
    let popup = find(&f.tree, "popup").expect("shown");
    assert_eq!(
        rect(&f.tree, popup),
        Rect::from_ltwh(220.0, 0.0, 80.0, 30.0)
    );
    // No flip, clamp: shifted inside the window.
    let f = shown(0.0, 250.0, Anchor::below().flip(false), POPUP);
    let popup = find(&f.tree, "popup").expect("shown");
    assert_eq!(
        rect(&f.tree, popup),
        Rect::from_ltwh(0.0, 270.0, 80.0, 30.0)
    );
    // Neither: left where the anchor puts it.
    let f = shown(0.0, 250.0, Anchor::below().flip(false).clamp(false), POPUP);
    let popup = find(&f.tree, "popup").expect("shown");
    assert_eq!(rect(&f.tree, popup).top, 290.0);
    // Larger than the window: at its top-left.
    let f = shown(0.0, 50.0, Anchor::below(), Size::new(500.0, 400.0));
    let popup = find(&f.tree, "popup").expect("shown");
    assert_eq!(rect(&f.tree, popup).origin(), Point::ZERO);
}

#[test]
fn view_overlay_04_hit_testing() {
    let f = shown(0.0, 50.0, Anchor::below(), POPUP);
    let (popup, target) = (
        find(&f.tree, "popup").expect("shown"),
        find(&f.tree, "target").expect("built"),
    );
    assert_eq!(f.tree.hit_test(Point::new(10.0, 100.0))[0], popup);
    assert_eq!(f.tree.hit_test(Point::new(10.0, 60.0))[0], target);
    // Elsewhere: the content below, and the overlay layer itself isn't hit.
    let below = f.tree.hit_test(Point::new(300.0, 200.0));
    assert!(!below.contains(&popup));
    let root = f.tree.root();
    assert_eq!(below.last(), Some(&root));
    assert_eq!(
        f.tree.children(root).len(),
        2,
        "content and the overlay layer"
    );
    let layer = f.tree.children(root)[1];
    assert!(!below.contains(&layer));

    // Passthrough: the content below gets the point.
    let mut tree = ViewTree::new(|| {
        let portal = OverlayPortal::new(boxed(100.0, 40.0))
            .visible(true)
            .pointer(OverlayPointer::Passthrough)
            .overlay(|| Keyed::new("tip", boxed(80.0, 30.0)));
        ClippedStack(vec![AnyView::new(At(0.0, 50.0, portal))])
    });
    frame(&mut tree);
    let tip = find(&tree, "tip").expect("shown");
    assert!(!tree.hit_test(Point::new(10.0, 100.0)).contains(&tip));
}

fn press(tree: &mut ViewTree, x: f32, y: f32) {
    for kind in [
        PointerKind::Down(PointerButton::Primary),
        PointerKind::Up(PointerButton::Primary),
    ] {
        tree.dispatch_pointer(PointerEvent {
            kind,
            position: Point::new(x, y),
        });
    }
}

#[test]
fn view_overlay_05_outside_press() {
    let log: Rc<RefCell<Vec<&'static str>>> = Rc::default();
    let l = log.clone();
    let mut tree = ViewTree::new(move || {
        let (a, b) = (l.clone(), l.clone());
        let portal = OverlayPortal::new(Keyed::new("target", boxed(100.0, 40.0)))
            .visible(true)
            .on_outside_press(move || a.borrow_mut().push("outside"))
            .overlay(|| Keyed::new("popup", boxed(80.0, 30.0)));
        Logged(ClippedStack(vec![AnyView::new(At(0.0, 50.0, portal))]), b)
    });
    frame(&mut tree);
    press(&mut tree, 10.0, 100.0); // on the popup
    press(&mut tree, 10.0, 60.0); // on the target
    assert_eq!(log.borrow().iter().filter(|e| **e == "outside").count(), 0);
    log.borrow_mut().clear();
    press(&mut tree, 300.0, 200.0);
    assert_eq!(*log.borrow(), ["outside", "down"]);
}

/// Logs primary presses reaching `child`'s element.
struct Logged<V>(V, Rc<RefCell<Vec<&'static str>>>);

impl<V: View> View for Logged<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.0.build(cx);
        let log = self.1;
        cx.on_pointer(id, Phase::Bubble, move |p| {
            if p.event.kind == PointerKind::Down(PointerButton::Primary) {
                log.borrow_mut().push("down");
            }
            Handled::Continue
        });
        id
    }
}

/// The target's size and whether the portal is present.
type SizeAndPresence = (Signal<Size>, Signal<bool>);

#[test]
fn view_overlay_06_follows_the_target_and_goes_with_the_portal() {
    let handles: Rc<RefCell<Option<SizeAndPresence>>> = Rc::default();
    let h = handles.clone();
    let mut tree = ViewTree::new(move || {
        let (size, present) = (signal(Size::new(100.0, 40.0)), signal(true));
        *h.borrow_mut() = Some((size, present));
        let portal = move || {
            OverlayPortal::new(Boxed {
                size: Prop::Dynamic(Box::new(move || size.get())),
                color: Color::BLACK,
                focusable: false,
            })
            .visible(true)
            .overlay(|| Keyed::new("popup", boxed(80.0, 30.0)))
        };
        // `At` inside the `Show`: a region can't hold the stack's parent data.
        ClippedStack(vec![AnyView::new(Show::new(
            move || present.get(),
            move || At(0.0, 50.0, portal()),
        ))])
    });
    let (size, present) = handles.borrow_mut().take().expect("built");
    frame(&mut tree);
    let popup = find(&tree, "popup").expect("shown");
    assert_eq!(rect(&tree, popup).top, 90.0);
    tree.enter(|| size.set(Size::new(100.0, 60.0)));
    frame(&mut tree);
    assert_eq!(rect(&tree, popup).top, 110.0);
    tree.enter(|| present.set(false));
    frame(&mut tree);
    assert_eq!(find(&tree, "popup"), None);
}

#[test]
fn view_overlay_07_focus_in_overlays() {
    let keys: Rc<RefCell<Vec<String>>> = Rc::default();
    let k = keys.clone();
    let mut tree = ViewTree::new(move || {
        let k = k.clone();
        let content = Keyed::new(
            "field",
            Boxed {
                size: Prop::Value(Size::new(50.0, 20.0)),
                color: Color::BLACK,
                focusable: true,
            },
        );
        let portal = OverlayPortal::new(Keyed::new("target", boxed(100.0, 40.0)))
            .visible(true)
            .overlay(move || {
                Keyed::new(
                    "menu",
                    KeyLogged(
                        Boxed {
                            size: Prop::Value(Size::new(80.0, 30.0)),
                            color: Color::BLACK,
                            focusable: true,
                        },
                        k.clone(),
                    ),
                )
            });
        ClippedStack(vec![
            AnyView::new(At(200.0, 0.0, content)),
            AnyView::new(At(0.0, 50.0, portal)),
        ])
    });
    frame(&mut tree);
    let (field, menu) = (
        find(&tree, "field").expect("built"),
        find(&tree, "menu").expect("shown"),
    );
    assert_eq!(tree.focus_next(), Some(field));
    assert_eq!(tree.focus_next(), Some(menu));
    tree.dispatch_key(KeyEvent::press(
        LogicalKey::Named(NamedKey::Enter),
        Modifiers::default(),
    ));
    assert_eq!(*keys.borrow(), ["Enter"]);
}

/// Logs key presses reaching `child`'s element.
struct KeyLogged<V>(V, Rc<RefCell<Vec<String>>>);

impl<V: View> View for KeyLogged<V> {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = self.0.build(cx);
        let log = self.1;
        cx.on_key(id, Phase::Bubble, move |k| {
            if k.event.pressed {
                log.borrow_mut().push(
                    format!("{:?}", k.event.key)
                        .replace("Named(", "")
                        .replace(')', ""),
                );
            }
            Handled::Continue
        });
        id
    }
}
