//! Tests for `docs/specs/widgets/split-view.md`, rules WIDGETS-SPLIT-01..08 and
//! WIDGETS-GRID-29.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_test::{Finder, WidgetTester, accesskit};
use tantu_view::core::{Point, Rect, Vec2};
use tantu_view::layout::RenderConstrainedBox;
use tantu_view::reactive::{Signal, signal};
use tantu_view::scene::Command;
use tantu_view::{
    BuildCx, CursorIcon, ElementId, Keyed, LogicalKey, Modifiers, NamedKey, PointerButton,
    PointerEvent, PointerKind, SemanticsAction, View,
};
use tantu_widgets::{
    CellValue, Column, DataGrid, Expanded, FnSource, GridColumn, SizedBox, SplitPane, SplitView,
    TextEditingController, TextField,
};

/// 402 px wide: two 1 px gaps leave 400 px for the panes.
const W: f32 = 402.0;
const H: f32 = 200.0;

/// A box filling its pane that counts its builds.
struct Counted(Rc<Cell<usize>>);

impl View for Counted {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        self.0.set(self.0.get() + 1);
        cx.render(RenderConstrainedBox::expand(), [])
    }
}

/// Folders (min 50, collapsible, labelled), b (min 50) and c, at 25 %, 25 % and 50 %.
struct Fx {
    t: WidgetTester,
    fractions: Signal<Vec<f32>>,
    builds: Rc<Cell<usize>>,
}

fn fixture() -> Fx {
    let out: Rc<Cell<Option<Signal<Vec<f32>>>>> = Rc::default();
    let builds: Rc<Cell<usize>> = Rc::default();
    let t = {
        let (out, builds) = (out.clone(), builds.clone());
        WidgetTester::with_size(W, H, move || {
            let fractions = signal(vec![0.25, 0.25, 0.5]);
            out.set(Some(fractions));
            SplitView::horizontal()
                .fractions(fractions)
                .pane(
                    SplitPane::new(Keyed::new("a", Counted(builds)))
                        .min_size(50.0)
                        .collapsible(true)
                        .label("Folders"),
                )
                .pane(SplitPane::new(Keyed::new("b", SizedBox::expand())).min_size(50.0))
                .pane(SplitPane::new(Keyed::new("c", SizedBox::expand())))
        })
    };
    let mut fx = Fx {
        t,
        fractions: out.get().expect("built"),
        builds,
    };
    fx.t.pump();
    fx
}

impl Fx {
    fn widths(&self) -> Vec<f32> {
        ["a", "b", "c"]
            .iter()
            .map(|k| self.t.rect(self.t.find(&Finder::key(*k))).width())
            .collect()
    }

    fn fractions(&self) -> Vec<f32> {
        self.t.enter(|| self.fractions.get_untracked())
    }

    fn set_fractions(&mut self, v: Vec<f32>) {
        let fractions = self.fractions;
        self.t.enter(|| fractions.set(v));
        self.t.pump();
        self.t.pump();
    }

    fn divider(&self, label: &str) -> ElementId {
        self.t.find(&Finder::semantics_label(label))
    }

    fn center(&self, label: &str) -> Point {
        self.t.rect(self.divider(label)).center()
    }

    fn pointer(&mut self, kind: PointerKind, at: Point) {
        self.t.pointer(PointerEvent {
            kind,
            position: at,
            modifiers: Modifiers::default(),
        });
        self.t.pump();
    }

    /// A press on the divider, a move by `delta` and a release, with `button`.
    fn drag(&mut self, label: &str, delta: Vec2, button: PointerButton) {
        let start = self.center(label);
        self.pointer(PointerKind::Down(button), start);
        self.pointer(PointerKind::Move, start + delta);
        self.pointer(PointerKind::Up(button), start + delta);
        self.t.pump();
    }

    fn key(&mut self, key: NamedKey) {
        self.key_with(key, Modifiers::default());
    }

    fn key_with(&mut self, key: NamedKey, modifiers: Modifiers) {
        self.t.press_key(LogicalKey::Named(key), modifiers);
        self.t.pump();
        self.t.pump();
    }

    fn focused(&mut self) -> Option<ElementId> {
        ElementId::from_raw(self.t.a11y_tree().focus.0)
    }
}

fn assert_close(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len(), "{actual:?} vs {expected:?}");
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() < 0.01, "{actual:?} vs {expected:?}");
    }
}

#[test]
fn widgets_split_01_fractions_size_the_panes() {
    let mut fx = fixture();
    assert_close(&fx.widths(), &[100.0, 100.0, 200.0]);
    assert_eq!(fx.builds.get(), 1);
    // Setting the signal relayouts without rebuilding a pane.
    fx.set_fractions(vec![0.5, 0.25, 0.25]);
    assert_close(&fx.widths(), &[200.0, 100.0, 100.0]);
    assert_eq!(fx.builds.get(), 1);
    // A short vector is cleaned (missing entries are 1/3) and not rewritten.
    fx.set_fractions(vec![1.0]);
    assert_close(&fx.widths(), &[240.0, 80.0, 80.0]);
    assert_eq!(fx.fractions(), [1.0]);

    // Without a shared signal the split starts at equal shares.
    let mut t = WidgetTester::with_size(W, H, || {
        SplitView::vertical()
            .pane(SplitPane::new(Keyed::new("top", SizedBox::expand())))
            .pane(SplitPane::new(Keyed::new("bottom", SizedBox::expand())))
    });
    t.pump();
    let top = t.rect(t.find(&Finder::key("top")));
    assert_eq!(top.height(), (H - 1.0) / 2.0);
    assert_eq!(top.width(), W);
}

/// The fills of `element`'s own paint, in window coordinates, with their colors.
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

fn strokes(t: &WidgetTester, element: ElementId) -> Vec<tantu_view::core::Color> {
    t.frame()
        .scene
        .entries()
        .iter()
        .filter(|e| e.element == Some(element))
        .filter_map(|e| match &e.command {
            Command::Stroke { color, .. } => Some(*color),
            _ => None,
        })
        .collect()
}

#[test]
fn widgets_split_02_look() {
    let mut fx = fixture();
    let colors = fx.t.enter(|| fx.t.theme().get().colors);
    let first = fx.divider("Resize Folders");
    // A 1 px outline_variant line filling the first gap (x 100..101), full height.
    let line = fills(&fx.t, first);
    assert_eq!(
        line,
        [(
            Rect::from_ltrb(100.0, 0.0, 101.0, H),
            colors.outline_variant
        )]
    );
    // Hovered: a 3 px primary line centred on the gap.
    let center = fx.center("Resize Folders");
    fx.pointer(PointerKind::Move, center);
    assert_eq!(
        fills(&fx.t, first),
        [(Rect::from_ltrb(99.0, 0.0, 102.0, H), colors.primary)]
    );
    assert!(strokes(&fx.t, first).is_empty());
    // Focused: a primary outline around the grab area.
    fx.key(NamedKey::Tab);
    assert_eq!(fx.focused(), Some(first));
    assert_eq!(strokes(&fx.t, first), [colors.primary]);
    // A theme change repaints with the new colors, keeping the sizes.
    let theme = fx.t.theme();
    fx.t.enter(|| theme.set(tantu_theme::Theme::dark()));
    fx.t.pump();
    let dark = tantu_theme::Theme::dark().colors;
    assert!(fills(&fx.t, first).iter().all(|(_, c)| *c == dark.primary));
    assert_close(&fx.widths(), &[100.0, 100.0, 200.0]);
}

#[test]
fn widgets_split_03_dragging_moves_one_boundary() {
    let mut fx = fixture();
    fx.drag(
        "Resize Folders",
        Vec2::new(40.0, 25.0),
        PointerButton::Primary,
    );
    // Only the two neighbours change; the cross-axis movement is ignored.
    assert_close(&fx.widths(), &[140.0, 60.0, 200.0]);
    assert_close(&fx.fractions(), &[0.35, 0.15, 0.5]);
    // Clamped at the neighbour's minimum (b stops at 50).
    fx.drag(
        "Resize Folders",
        Vec2::new(100.0, 0.0),
        PointerButton::Primary,
    );
    assert_close(&fx.widths(), &[150.0, 50.0, 200.0]);
    // The second divider: c has no minimum and isn't collapsible, so it stops at 0.
    fx.drag(
        "Resize pane 2",
        Vec2::new(300.0, 0.0),
        PointerButton::Primary,
    );
    assert_close(&fx.widths(), &[150.0, 250.0, 0.0]);
    assert_close(&fx.fractions(), &[0.375, 0.625, 0.0]);
    // A secondary button does nothing.
    let before = fx.fractions();
    fx.drag(
        "Resize Folders",
        Vec2::new(-60.0, 0.0),
        PointerButton::Secondary,
    );
    assert_eq!(fx.fractions(), before);
}

#[test]
fn widgets_split_03_each_move_writes_the_whole_vector() {
    let mut fx = fixture();
    let start = fx.center("Resize Folders");
    fx.pointer(PointerKind::Down(PointerButton::Primary), start);
    fx.pointer(PointerKind::Move, start + Vec2::new(10.0, 0.0));
    assert_close(&fx.fractions(), &[0.275, 0.225, 0.5]);
    // The pointer outrunning the handle keeps moving the boundary.
    fx.pointer(PointerKind::Move, start + Vec2::new(-30.0, 90.0));
    assert_close(&fx.fractions(), &[0.175, 0.325, 0.5]);
    fx.pointer(PointerKind::Up(PointerButton::Primary), start);
}

#[test]
fn widgets_split_04_collapsing_by_dragging() {
    let mut fx = fixture();
    // Below half of Folders' 50 px minimum: it collapses, its space going to b.
    fx.drag(
        "Resize Folders",
        Vec2::new(-80.0, 0.0),
        PointerButton::Primary,
    );
    assert_close(&fx.widths(), &[0.0, 200.0, 200.0]);
    assert_close(&fx.fractions(), &[0.0, 0.5, 0.5]);
    // Dragged back past half its minimum: restored at the minimum.
    fx.drag(
        "Resize Folders",
        Vec2::new(30.0, 0.0),
        PointerButton::Primary,
    );
    assert_close(&fx.widths(), &[50.0, 150.0, 200.0]);
    // Between half the minimum and the minimum, a pane stops at its minimum.
    fx.drag(
        "Resize Folders",
        Vec2::new(-20.0, 0.0),
        PointerButton::Primary,
    );
    assert_close(&fx.widths(), &[50.0, 150.0, 200.0]);
    // A non-collapsible pane never collapses (b stops at 50).
    fx.drag(
        "Resize Folders",
        Vec2::new(200.0, 0.0),
        PointerButton::Primary,
    );
    assert_close(&fx.widths(), &[150.0, 50.0, 200.0]);
}

#[test]
fn widgets_split_05_resize_cursors() {
    let mut fx = fixture();
    let center = fx.center("Resize Folders");
    fx.pointer(PointerKind::Move, center);
    assert_eq!(fx.t.cursor(), CursorIcon::ResizeHorizontal);
    // A drag keeps the cursor when the pointer outruns the handle.
    fx.pointer(PointerKind::Down(PointerButton::Primary), center);
    fx.pointer(
        PointerKind::Move,
        center + Vec2::new(0.0, 0.0) + Vec2::new(150.0, 50.0),
    );
    assert_eq!(fx.t.cursor(), CursorIcon::ResizeHorizontal);
    fx.pointer(
        PointerKind::Up(PointerButton::Primary),
        center + Vec2::new(150.0, 50.0),
    );
    // Over a pane: the default arrow.
    let a = fx.t.rect(fx.t.find(&Finder::key("c"))).center();
    fx.pointer(PointerKind::Move, a);
    assert_eq!(fx.t.cursor(), CursorIcon::Default);

    // Stacked panes resize vertically.
    let mut t = WidgetTester::with_size(W, H, || {
        SplitView::vertical()
            .pane(SplitPane::new(SizedBox::expand()))
            .pane(SplitPane::new(SizedBox::expand()))
    });
    t.pump();
    t.hover(&Finder::semantics_label("Resize pane 1"));
    assert_eq!(t.cursor(), CursorIcon::ResizeVertical);
}

#[test]
fn widgets_split_06_keyboard() {
    let mut fx = fixture();
    // Dividers come after the panes' contents in traversal order, in order.
    fx.key(NamedKey::Tab);
    let first = fx.divider("Resize Folders");
    assert_eq!(fx.focused(), Some(first));
    fx.key(NamedKey::ArrowRight);
    assert_close(&fx.widths(), &[110.0, 90.0, 200.0]);
    // Both axes' arrows work; Up and Left move towards the start.
    fx.key(NamedKey::ArrowUp);
    fx.key(NamedKey::ArrowLeft);
    assert_close(&fx.widths(), &[90.0, 110.0, 200.0]);
    fx.key(NamedKey::ArrowDown);
    assert_close(&fx.widths(), &[100.0, 100.0, 200.0]);
    // Repeats move again; modified keys pass through.
    fx.t.press_key(
        LogicalKey::Named(NamedKey::ArrowRight),
        Modifiers::default(),
    );
    fx.t.pump();
    assert_close(&fx.widths(), &[110.0, 90.0, 200.0]);
    fx.key_with(
        NamedKey::ArrowRight,
        Modifiers {
            control: true,
            ..Default::default()
        },
    );
    assert_close(&fx.widths(), &[110.0, 90.0, 200.0]);
    // Home and End go as far as the minimums allow (not collapsing).
    fx.key(NamedKey::Home);
    assert_close(&fx.widths(), &[50.0, 150.0, 200.0]);
    fx.key(NamedKey::End);
    assert_close(&fx.widths(), &[150.0, 50.0, 200.0]);
    // Enter collapses the preceding collapsible pane and restores it.
    fx.key(NamedKey::Enter);
    assert_close(&fx.widths(), &[0.0, 200.0, 200.0]);
    fx.key(NamedKey::Enter);
    assert_close(&fx.widths(), &[150.0, 50.0, 200.0]);
    // The second divider is the next stop; with no collapsible neighbour Enter does nothing.
    fx.key(NamedKey::Tab);
    assert_eq!(fx.focused(), Some(fx.divider("Resize pane 2")));
    fx.key(NamedKey::Enter);
    assert_close(&fx.widths(), &[150.0, 50.0, 200.0]);
}

#[test]
fn widgets_split_07_pressing_a_divider_keeps_the_focus() {
    let out: Rc<RefCell<Option<TextEditingController>>> = Rc::default();
    let mut t = {
        let out = out.clone();
        WidgetTester::with_size(W, H, move || {
            let controller = TextEditingController::new("");
            *out.borrow_mut() = Some(controller);
            SplitView::horizontal()
                .pane(SplitPane::new(SizedBox::expand()))
                .pane(SplitPane::new(Keyed::new(
                    "field",
                    TextField::new(controller),
                )))
        })
    };
    t.pump();
    t.focus(&Finder::key("field"));
    let field = ElementId::from_raw(t.a11y_tree().focus.0);
    t.drag(
        &Finder::semantics_label("Resize pane 1"),
        Vec2::new(-30.0, 0.0),
    );
    assert_eq!(ElementId::from_raw(t.a11y_tree().focus.0), field);
    assert!(field.is_some());
}

#[test]
fn widgets_split_08_semantics() {
    let mut fx = fixture();
    let tree = fx.t.a11y_tree();
    let splitters: Vec<&accesskit::Node> = tree
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == accesskit::Role::Splitter)
        .map(|(_, n)| n)
        .collect();
    assert_eq!(splitters.len(), 2);
    let labels: Vec<_> = splitters.iter().map(|n| n.label()).collect();
    assert_eq!(labels, [Some("Resize Folders"), Some("Resize pane 2")]);
    // The boundary's position as a percentage, and the 10 px step as one.
    assert_eq!(splitters[0].numeric_value(), Some(25.0));
    assert_eq!(splitters[1].numeric_value(), Some(50.0));
    assert_eq!(splitters[0].min_numeric_value(), Some(0.0));
    assert_eq!(splitters[0].max_numeric_value(), Some(100.0));
    assert_eq!(splitters[0].numeric_value_step(), Some(2.5));
    // A labelled pane is a group with its label; unlabelled panes add nothing.
    assert!(
        tree.nodes
            .iter()
            .any(|(_, n)| n.role() == accesskit::Role::Group && n.label() == Some("Folders"))
    );
    // Increment moves the boundary 10 px towards the end.
    assert!(fx.t.perform_action(
        &Finder::semantics_label("Resize Folders"),
        SemanticsAction::Increment
    ));
    fx.t.pump();
    assert_close(&fx.widths(), &[110.0, 90.0, 200.0]);
    let tree = fx.t.a11y_tree();
    let value = tree
        .nodes
        .iter()
        .find(|(_, n)| n.label() == Some("Resize Folders"))
        .and_then(|(_, n)| n.numeric_value());
    assert_eq!(value, Some(27.5));

    // One pane: no divider.
    let mut t = WidgetTester::with_size(W, H, || {
        SplitView::horizontal().pane(SplitPane::new(SizedBox::expand()))
    });
    assert!(
        !t.a11y_tree()
            .nodes
            .iter()
            .any(|(_, n)| n.role() == accesskit::Role::Splitter)
    );
}

#[test]
fn widgets_grid_29_resize_handle_cursor() {
    let mut t = WidgetTester::with_size(400.0, 300.0, || {
        Column::new().child(Expanded::new(DataGrid::new(
            [GridColumn::new("Name"), GridColumn::new("City")],
            Rc::new(FnSource::new(10, |r, c| {
                CellValue::Text(format!("r{r}c{c}"))
            })),
        )))
    });
    t.pump();
    t.pump();
    let header = t.rect(t.find(&Finder::text("Name")));
    // The handle lies inside the column's right edge (120 px wide, 6 px handle).
    let on_handle = Point::new(117.0, header.center().y);
    t.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: on_handle,
        modifiers: Modifiers::default(),
    });
    t.pump();
    assert_eq!(t.cursor(), CursorIcon::ResizeHorizontal);
    t.pointer(PointerEvent {
        kind: PointerKind::Move,
        position: Point::new(60.0, header.center().y),
        modifiers: Modifiers::default(),
    });
    t.pump();
    assert_eq!(t.cursor(), CursorIcon::Default);
}
