//! Tests for `docs/specs/view/paint.md`, rules VIEW-PAINT-01..06.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use tantu_core::{Affine, Color, Rect, Size, Vec2};
use tantu_layout::{BoxConstraints, NoTextMeasure, RenderConstrainedBox, RenderFlex};
use tantu_scene::{Clip, Command, Scene};
use tantu_view::{AnyView, BuildCx, ElementId, NoPaint, Paint, PaintCx, View, ViewTree};

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

type Log = Rc<RefCell<Vec<String>>>;

/// How a test paint behaves.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// Fill the bounds; children after.
    Fill,
    /// Fill, then push a clip around the children (and report clipping).
    Clip,
    /// Fill and skip the children.
    Skip,
    /// Call `paint_children` twice.
    Twice,
    /// Change the z-index and element id, then fill.
    Meddle,
}

struct TestPaint {
    name: &'static str,
    color: Color,
    mode: Mode,
    overflow: f32,
    log: Log,
}

impl Paint for TestPaint {
    fn paint(&self, cx: &mut PaintCx<'_, '_>) {
        let size = cx.size();
        self.log
            .borrow_mut()
            .push(format!("{} {}x{}", self.name, size.width, size.height));
        let bounds = Rect::from_ltwh(0.0, 0.0, size.width, size.height);
        if self.mode == Mode::Meddle {
            cx.scene().set_z_index(5);
            cx.scene().set_element(None);
        }
        cx.scene().fill_rect(bounds, self.color);
        match self.mode {
            Mode::Clip => {
                cx.scene().push_clip(Clip::Rect(bounds));
                cx.paint_children();
                cx.scene().pop();
            }
            Mode::Skip => cx.skip_children(),
            Mode::Twice => {
                cx.paint_children();
                cx.paint_children();
            }
            Mode::Fill | Mode::Meddle => {}
        }
    }

    fn overflow(&self) -> f32 {
        self.overflow
    }

    fn clips_children(&self) -> bool {
        self.mode == Mode::Clip
    }
}

/// A painted render element: a sized box (leaf) or a column (with children).
struct Painted {
    paint: TestPaint,
    size: Option<Size>,
    children: Vec<AnyView>,
    out: Option<Rc<Cell<Option<ElementId>>>>,
}

impl View for Painted {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = match self.size {
            Some(size) => cx.render(
                RenderConstrainedBox::sized(Some(size.width), Some(size.height)),
                self.children,
            ),
            None => cx.render(RenderFlex::column(), self.children),
        };
        assert!(cx.set_paint(id, self.paint));
        if let Some(out) = self.out {
            out.set(Some(id));
        }
        id
    }
}

fn paint(log: &Log, name: &'static str, mode: Mode) -> TestPaint {
    TestPaint {
        name,
        color: Color::from_rgb8(10, 20, 30),
        mode,
        overflow: 0.0,
        log: log.clone(),
    }
}

fn leaf(p: TestPaint, w: f32, h: f32) -> AnyView {
    AnyView::new(Painted {
        paint: p,
        size: Some(s(w, h)),
        children: vec![],
        out: None,
    })
}

fn column(p: TestPaint, children: Vec<AnyView>) -> AnyView {
    AnyView::new(Painted {
        paint: p,
        size: None,
        children,
        out: None,
    })
}

fn tracked(view: Painted, out: &Rc<Cell<Option<ElementId>>>) -> AnyView {
    AnyView::new(Painted {
        out: Some(out.clone()),
        ..view
    })
}

struct Region(Vec<AnyView>);

impl View for Region {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.region(|cx| {
            for child in self.0 {
                child.build(cx);
            }
        })
    }
}

/// A region that tries to set a paint (and records the answer).
struct RegionSetsPaint(Rc<Cell<Option<bool>>>);

impl View for RegionSetsPaint {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        let id = cx.region(|_| {});
        self.0.set(Some(cx.set_paint(id, NoPaint)));
        id
    }
}

/// Lays out at 800 × 600 and paints into a fresh Scene of that size.
fn render(tree: &mut ViewTree) -> Scene {
    tree.layout(BoxConstraints::tight(s(800.0, 600.0)), &mut NoTextMeasure);
    paint_only(tree)
}

fn paint_only(tree: &ViewTree) -> Scene {
    let mut scene = Scene::new();
    let mut b = scene.begin(s(800.0, 600.0));
    tree.paint(&mut b);
    b.finish().expect("paint leaves scopes balanced");
    scene
}

/// (element, command kind) for every entry.
fn summary(scene: &Scene) -> Vec<(Option<ElementId>, &'static str)> {
    scene
        .entries()
        .iter()
        .map(|e| {
            let kind = match e.command {
                Command::PushTransform(_) => "push_transform",
                Command::PopTransform => "pop_transform",
                Command::PushClip(_) => "push_clip",
                Command::PopClip => "pop_clip",
                Command::Fill { .. } => "fill",
                _ => "other",
            };
            (e.element, kind)
        })
        .collect()
}

#[test]
fn view_paint_01_set_paint() {
    let answer = Rc::new(Cell::new(None));
    let log = Log::default();
    let mut tree = {
        let (answer, log) = (answer.clone(), log.clone());
        ViewTree::new(move || {
            column(
                paint(&log, "col", Mode::Fill),
                vec![
                    AnyView::new(RegionSetsPaint(answer)),
                    // A render element with no paint of its own: its child still paints.
                    AnyView::new(Unpainted(vec![leaf(
                        paint(&log, "leaf", Mode::Fill),
                        4.0,
                        4.0,
                    )])),
                ],
            )
        })
    };
    assert_eq!(answer.get(), Some(false));
    render(&mut tree);
    assert_eq!(*log.borrow(), ["col 800x600", "leaf 4x4"]);
}

/// A column with no paint set.
struct Unpainted(Vec<AnyView>);

impl View for Unpainted {
    fn build(self, cx: &mut BuildCx<'_>) -> ElementId {
        cx.render(RenderFlex::column(), self.0)
    }
}

#[test]
fn view_paint_02_traversal_scopes_and_ids() {
    let log = Log::default();
    let ids: Vec<Rc<Cell<Option<ElementId>>>> = (0..3).map(|_| Rc::default()).collect();
    let mut tree = {
        let (log, ids) = (log.clone(), ids.clone());
        ViewTree::new(move || {
            tracked(
                Painted {
                    paint: paint(&log, "col", Mode::Fill),
                    size: None,
                    children: vec![
                        tracked(
                            Painted {
                                paint: paint(&log, "a", Mode::Fill),
                                size: Some(s(10.0, 5.0)),
                                children: vec![],
                                out: None,
                            },
                            &ids[1],
                        ),
                        AnyView::new(Region(vec![tracked(
                            Painted {
                                paint: paint(&log, "b", Mode::Meddle),
                                size: Some(s(10.0, 5.0)),
                                children: vec![],
                                out: None,
                            },
                            &ids[2],
                        )])),
                    ],
                    out: None,
                },
                &ids[0],
            )
        })
    };
    let scene = render(&mut tree);
    let [col, a, b] = [0, 1, 2].map(|i| ids[i].get().expect("built"));
    let root = tree.root();
    assert_eq!(*log.borrow(), ["col 800x600", "a 10x5", "b 10x5"]);
    // b sits in a region under the column, so its render parent is the column.
    let region = tree.parent(b).expect("b has a parent");
    assert_eq!(tree.parent(region), Some(col));
    assert_eq!(
        summary(&scene),
        [
            (None, "push_transform"),
            (Some(root), "push_transform"),
            (Some(col), "fill"),
            (Some(col), "push_transform"),
            (Some(a), "fill"),
            // A pop records the element of the scope's opener (the parent).
            (Some(col), "pop_transform"),
            (Some(col), "push_transform"),
            // b changed its element id and z-index; the pop restores the parent's.
            (None, "fill"),
            (Some(col), "pop_transform"),
            (Some(root), "pop_transform"),
            (None, "pop_transform"),
        ]
    );
    // Offsets become translations.
    let translations: Vec<Affine> = scene
        .entries()
        .iter()
        .filter_map(|e| match e.command {
            Command::PushTransform(t) => Some(t),
            _ => None,
        })
        .collect();
    assert_eq!(
        translations,
        [
            Affine::translate(Vec2::ZERO),
            Affine::translate(Vec2::ZERO),
            // The column centers its 10-wide children in 800: x = 395.
            Affine::translate(Vec2::new(395.0, 0.0)),
            Affine::translate(Vec2::new(395.0, 5.0)),
        ]
    );
    // b's z-index didn't leak: everything else is at 0.
    for e in scene.entries() {
        if e.element.is_some() {
            assert_eq!(e.z_index, 0);
        }
    }
}

#[test]
fn view_paint_03_children_once() {
    let log = Log::default();
    for (mode, expected) in [
        (Mode::Fill, vec!["parent 800x600", "child 3x3"]),
        (Mode::Twice, vec!["parent 800x600", "child 3x3"]),
        (Mode::Skip, vec!["parent 800x600"]),
        (Mode::Clip, vec!["parent 800x600", "child 3x3"]),
    ] {
        let mut tree = {
            let log = log.clone();
            ViewTree::new(move || {
                column(
                    paint(&log, "parent", mode),
                    vec![leaf(paint(&log, "child", Mode::Fill), 3.0, 3.0)],
                )
            })
        };
        let scene = render(&mut tree);
        assert_eq!(*log.borrow(), expected);
        log.borrow_mut().clear();
        if mode == Mode::Clip {
            // The child's commands are inside the clip.
            let kinds: Vec<&str> = summary(&scene).into_iter().map(|(_, k)| k).collect();
            let clip = kinds.iter().position(|k| *k == "push_clip").expect("clip");
            let pop = kinds.iter().position(|k| *k == "pop_clip").expect("pop");
            let child_fill = kinds.iter().rposition(|k| *k == "fill").expect("fill");
            assert!(clip < child_fill && child_fill < pop, "{kinds:?}");
        }
    }
}

#[test]
fn view_paint_04_size_and_unlaid_elements() {
    let log = Log::default();
    let tree = {
        let log = log.clone();
        ViewTree::new(move || leaf(paint(&log, "leaf", Mode::Fill), 3.0, 3.0))
    };
    // Never laid out: nothing is painted.
    let scene = paint_only(&tree);
    assert!(scene.entries().is_empty());
    assert!(log.borrow().is_empty());
}

#[test]
fn view_paint_05_culling() {
    let log = Log::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || {
            let mut shadowed = paint(&log, "shadowed", Mode::Fill);
            shadowed.overflow = 150.0;
            column(
                paint(&log, "col", Mode::Fill),
                vec![
                    leaf(paint(&log, "tall", Mode::Fill), 10.0, 650.0),
                    // y = 650: below the 600-tall Scene.
                    leaf(paint(&log, "hidden", Mode::Fill), 10.0, 10.0),
                    // y = 660, but its drawing may reach 150 beyond its bounds.
                    leaf(shadowed, 10.0, 10.0),
                    // A non-clipping container off-screen is visited; its leaf is culled.
                    column(
                        paint(&log, "open", Mode::Fill),
                        vec![leaf(paint(&log, "open child", Mode::Fill), 5.0, 5.0)],
                    ),
                    // A clipping container off-screen is skipped with its subtree.
                    column(
                        paint(&log, "clipped", Mode::Clip),
                        vec![leaf(paint(&log, "clipped child", Mode::Fill), 5.0, 5.0)],
                    ),
                ],
            )
        })
    };
    render(&mut tree);
    assert_eq!(
        *log.borrow(),
        ["col 800x600", "tall 10x650", "shadowed 10x10", "open 5x5"]
    );
}

#[test]
fn view_paint_06_deterministic() {
    let log = Log::default();
    let mut tree = {
        let log = log.clone();
        ViewTree::new(move || {
            column(
                paint(&log, "col", Mode::Clip),
                vec![
                    leaf(paint(&log, "a", Mode::Fill), 3.0, 3.0),
                    AnyView::new(Region(vec![leaf(paint(&log, "b", Mode::Meddle), 4.0, 4.0)])),
                ],
            )
        })
    };
    let first = render(&mut tree);
    let len = tree.len();
    let second = paint_only(&tree);
    assert_eq!(first, second);
    assert_eq!(tree.len(), len);
}
