//! Tests for `docs/specs/layout/stack.md`, rules LAYOUT-STACK-01..08.

mod common;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{
    Alignment, BoxConstraints, LayoutId, LayoutTree, RenderStack, StackFit, StackParentData,
};

const INF: f32 = f32::INFINITY;
const NAN: f32 = f32::NAN;

fn s(w: f32, h: f32) -> Size {
    Size::new(w, h)
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// How a test child is set up.
enum Data {
    None,
    Positioned(StackParentData),
    Other,
}

struct Stack {
    tree: LayoutTree,
    root: LayoutId,
    kids: Vec<LayoutId>,
}

fn stack(render: RenderStack, kids: Vec<(f32, f32, Data)>) -> Stack {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(render);
    let ids: Vec<LayoutId> = kids
        .into_iter()
        .map(|(w, h, data)| {
            let id = tree.insert(leaf(&log, "kid", w, h));
            match data {
                Data::None => {}
                Data::Positioned(p) => {
                    tree.set_parent_data(id, Some(p));
                }
                Data::Other => {
                    tree.set_parent_data(id, Some("not stack data"));
                }
            }
            id
        })
        .collect();
    tree.set_children(root, &ids).expect("valid");
    Stack {
        tree,
        root,
        kids: ids,
    }
}

impl Stack {
    fn layout(&mut self, c: BoxConstraints) -> Size {
        self.tree.layout(self.root, c)
    }

    fn constraints(&self, i: usize) -> BoxConstraints {
        self.tree.constraints(self.kids[i]).expect("laid out")
    }

    fn size(&self, i: usize) -> Size {
        self.tree.size(self.kids[i]).expect("laid out")
    }

    fn offset(&self, i: usize) -> Vec2 {
        self.tree.offset(self.kids[i]).expect("exists")
    }
}

fn pos(f: impl FnOnce(&mut StackParentData)) -> Data {
    let mut p = StackParentData::default();
    f(&mut p);
    Data::Positioned(p)
}

fn with(alignment: Alignment, fit: StackFit) -> RenderStack {
    RenderStack { alignment, fit }
}

#[test]
fn layout_stack_01_types() {
    let expected = RenderStack {
        alignment: Alignment::TOP_LEFT,
        fit: StackFit::Loose,
    };
    assert_eq!(RenderStack::new(), expected);
    assert_eq!(RenderStack::default(), expected);
    let none = StackParentData::default();
    assert_eq!(
        (
            none.left,
            none.top,
            none.right,
            none.bottom,
            none.width,
            none.height
        ),
        (None, None, None, None, None, None)
    );
    assert!(!none.is_positioned());
    for set in [
        StackParentData {
            left: Some(0.0),
            ..none
        },
        StackParentData {
            top: Some(1.0),
            ..none
        },
        StackParentData {
            right: Some(-1.0),
            ..none
        },
        StackParentData {
            bottom: Some(2.0),
            ..none
        },
        StackParentData {
            width: Some(3.0),
            ..none
        },
        StackParentData {
            height: Some(4.0),
            ..none
        },
    ] {
        assert!(set.is_positioned(), "{set:?}");
    }
    let fill = StackParentData::fill();
    assert_eq!(
        (
            fill.left,
            fill.top,
            fill.right,
            fill.bottom,
            fill.width,
            fill.height
        ),
        (Some(0.0), Some(0.0), Some(0.0), Some(0.0), None, None)
    );
}

#[test]
fn layout_stack_02_non_positioned_constraints() {
    let kids = || {
        vec![
            (30.0, 20.0, Data::None),
            (50.0, 60.0, Data::Positioned(StackParentData::default())),
            (10.0, 10.0, Data::Other),
        ]
    };
    let incoming = BoxConstraints::new(10.0, 200.0, 10.0, 100.0);
    let cases = [
        (StackFit::Loose, BoxConstraints::new(0.0, 200.0, 0.0, 100.0)),
        (StackFit::Expand, BoxConstraints::tight(s(200.0, 100.0))),
        (StackFit::Passthrough, incoming),
    ];
    for (fit, expected) in cases {
        let mut st = stack(with(Alignment::TOP_LEFT, fit), kids());
        st.layout(incoming);
        for i in 0..3 {
            assert_eq!(st.constraints(i), expected, "{fit:?} child {i}");
        }
    }
}

#[test]
fn layout_stack_03_stack_size() {
    let mut st = stack(
        RenderStack::new(),
        vec![(30.0, 20.0, Data::None), (50.0, 60.0, Data::None)],
    );
    assert_eq!(st.layout(loose(200.0, 100.0)), s(50.0, 60.0));
    assert_eq!(
        st.layout(BoxConstraints::new(100.0, 200.0, 0.0, 100.0)),
        s(100.0, 60.0)
    );
    // Only positioned children: as large as allowed, or the minimum when unbounded.
    let only_positioned = || vec![(30.0, 20.0, pos(|p| p.left = Some(5.0)))];
    let mut st = stack(RenderStack::new(), only_positioned());
    assert_eq!(st.layout(loose(200.0, 100.0)), s(200.0, 100.0));
    let mut st = stack(RenderStack::new(), only_positioned());
    assert_eq!(
        st.layout(BoxConstraints::new(30.0, INF, 0.0, 100.0)),
        s(30.0, 100.0)
    );
    let mut st = stack(RenderStack::new(), only_positioned());
    assert_eq!(st.layout(BoxConstraints::UNCONSTRAINED), s(0.0, 0.0));
}

#[test]
fn layout_stack_04_non_positioned_alignment() {
    let kids = || vec![(30.0, 20.0, Data::None), (50.0, 60.0, Data::None)];
    let mut st = stack(with(Alignment::CENTER, StackFit::Loose), kids());
    st.layout(loose(200.0, 100.0));
    assert_eq!(st.offset(0), v(10.0, 20.0));
    assert_eq!(st.offset(1), v(0.0, 0.0));
    let mut st = stack(RenderStack::new(), kids());
    st.layout(loose(200.0, 100.0));
    assert_eq!(st.offset(0), v(0.0, 0.0));
    // Against a larger stack (tight constraints).
    let mut st = stack(with(Alignment::BOTTOM_RIGHT, StackFit::Loose), kids());
    st.layout(BoxConstraints::tight(s(200.0, 100.0)));
    assert_eq!(st.offset(0), v(170.0, 80.0));
}

/// A stack sized 200 × 100 by its first (non-positioned) child, plus positioned ones.
fn positioned(alignment: Alignment, kids: Vec<(f32, f32, Data)>) -> Stack {
    let mut all = vec![(200.0, 100.0, Data::None)];
    all.extend(kids);
    let mut st = stack(with(alignment, StackFit::Loose), all);
    assert_eq!(st.layout(loose(400.0, 400.0)), s(200.0, 100.0));
    st
}

#[test]
fn layout_stack_05_positioned_constraints() {
    let st = positioned(
        Alignment::TOP_LEFT,
        vec![
            (
                30.0,
                20.0,
                pos(|p| (p.left, p.top) = (Some(10.0), Some(5.0))),
            ),
            (
                30.0,
                20.0,
                pos(|p| (p.left, p.right) = (Some(10.0), Some(20.0))),
            ),
            (
                30.0,
                20.0,
                pos(|p| {
                    (p.left, p.right) = (Some(0.0), Some(0.0));
                    (p.width, p.height) = (Some(50.0), Some(40.0));
                }),
            ),
            (30.0, 20.0, Data::Positioned(StackParentData::fill())),
            (
                30.0,
                20.0,
                pos(|p| (p.left, p.right) = (Some(10.0), Some(300.0))),
            ),
            (
                30.0,
                20.0,
                pos(|p| (p.top, p.bottom) = (Some(10.0), Some(30.0))),
            ),
        ],
    );
    assert_eq!(st.constraints(1), BoxConstraints::UNCONSTRAINED);
    assert_eq!(
        st.constraints(2),
        BoxConstraints::new(170.0, 170.0, 0.0, INF)
    );
    assert_eq!(st.size(2), s(170.0, 20.0));
    assert_eq!(st.constraints(3), BoxConstraints::tight(s(50.0, 40.0)));
    assert_eq!(st.constraints(4), BoxConstraints::tight(s(200.0, 100.0)));
    assert_eq!(st.constraints(5), BoxConstraints::new(0.0, 0.0, 0.0, INF));
    assert_eq!(st.constraints(6), BoxConstraints::new(0.0, INF, 60.0, 60.0));
}

#[test]
fn layout_stack_06_positioned_offsets() {
    let st = positioned(
        Alignment::TOP_LEFT,
        vec![
            (
                30.0,
                20.0,
                pos(|p| (p.left, p.top) = (Some(10.0), Some(5.0))),
            ),
            (
                30.0,
                20.0,
                pos(|p| (p.right, p.bottom) = (Some(20.0), Some(10.0))),
            ),
            (
                30.0,
                20.0,
                pos(|p| (p.left, p.right) = (Some(10.0), Some(20.0))),
            ),
            (100.0, 20.0, pos(|p| p.left = Some(150.0))),
            (
                30.0,
                20.0,
                pos(|p| (p.left, p.top) = (Some(-10.0), Some(-5.0))),
            ),
        ],
    );
    assert_eq!(st.offset(1), v(10.0, 5.0));
    assert_eq!(st.offset(2), v(150.0, 70.0));
    assert_eq!(st.offset(3), v(10.0, 0.0));
    // Extending outside the stack is kept.
    assert_eq!(st.offset(4), v(150.0, 0.0));
    assert_eq!(st.offset(5), v(-10.0, -5.0));
    // On an axis with no edge set, the stack's alignment places the child.
    let st = positioned(
        Alignment::CENTER,
        vec![(30.0, 20.0, pos(|p| p.top = Some(5.0)))],
    );
    assert_eq!(st.offset(1), v(85.0, 5.0));
    let st = positioned(
        Alignment::BOTTOM_RIGHT,
        vec![(30.0, 20.0, pos(|p| p.width = Some(40.0)))],
    );
    assert_eq!(st.offset(1), v(160.0, 80.0));
}

#[test]
fn layout_stack_07_intrinsics() {
    let mut st = stack(
        RenderStack::new(),
        vec![
            (30.0, 20.0, Data::None),
            (50.0, 60.0, Data::None),
            (500.0, 500.0, pos(|p| p.left = Some(0.0))),
        ],
    );
    assert_eq!(st.tree.min_intrinsic_width(st.root, INF), 50.0);
    assert_eq!(st.tree.max_intrinsic_width(st.root, INF), 50.0);
    assert_eq!(st.tree.min_intrinsic_height(st.root, INF), 60.0);
    assert_eq!(st.tree.max_intrinsic_height(st.root, INF), 60.0);
    let mut st = stack(
        RenderStack::new(),
        vec![(500.0, 500.0, pos(|p| p.left = Some(0.0)))],
    );
    assert_eq!(st.tree.min_intrinsic_width(st.root, INF), 0.0);
}

#[test]
fn layout_stack_08_robustness() {
    let mut st = stack(RenderStack::new(), vec![]);
    assert_eq!(st.layout(loose(200.0, 100.0)), s(200.0, 100.0));
    // A NaN field counts as not set.
    assert!(
        !StackParentData {
            left: Some(NAN),
            ..Default::default()
        }
        .is_positioned()
    );
    let mut st = stack(
        RenderStack::new(),
        vec![
            (30.0, 20.0, pos(|p| p.left = Some(NAN))),
            (
                30.0,
                20.0,
                pos(|p| (p.left, p.top) = (Some(NAN), Some(5.0))),
            ),
        ],
    );
    assert_eq!(st.layout(loose(200.0, 100.0)), s(30.0, 20.0));
    assert_eq!(st.constraints(0), loose(200.0, 100.0)); // non-positioned
    assert_eq!(st.offset(1), v(0.0, 5.0)); // x from alignment
    // Nothing panics.
    for fit in [StackFit::Loose, StackFit::Expand, StackFit::Passthrough] {
        let mut st = stack(
            with(Alignment::new(NAN, INF), fit),
            vec![
                (NAN, INF, Data::None),
                (
                    10.0,
                    10.0,
                    pos(|p| {
                        (p.left, p.right) = (Some(INF), Some(-INF));
                        (p.width, p.height) = (Some(-5.0), Some(NAN));
                    }),
                ),
            ],
        );
        for c in [
            BoxConstraints::UNCONSTRAINED,
            BoxConstraints::new(NAN, NAN, NAN, NAN),
            BoxConstraints::new(80.0, 20.0, INF, INF),
        ] {
            st.tree.mark_needs_layout(st.root);
            let _ = st.layout(c);
            let _ = st.tree.min_intrinsic_width(st.root, NAN);
        }
    }
}
