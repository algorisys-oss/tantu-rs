//! Tests for `docs/specs/layout/tree.md`, rules LAYOUT-TREE-01..18. The allocation promise is
//! checked in `tree_alloc.rs`.

mod common;

use common::*;
use tantu_core::{Size, Vec2};
use tantu_layout::{BoxConstraints, LayoutChildren, LayoutId, LayoutTree, RenderBox, TreeError};

const INF: f32 = f32::INFINITY;

/// The tree used by the relayout tests:
///
/// ```text
/// root  Column(Loose)            boundary (root of the pass)
/// ├─ a  Column(Loose)
/// │  ├─ a1 Leaf 10×5
/// │  └─ a2 Leaf 20×5
/// └─ b  Column(Tight)
///    └─ b1 Column(Loose)         boundary (tight constraints)
///       └─ b11 Leaf 7×3
/// ```
struct Fixture {
    tree: LayoutTree,
    log: Log,
    root: LayoutId,
    a: LayoutId,
    a1: LayoutId,
    a2: LayoutId,
    b: LayoutId,
    b1: LayoutId,
    b11: LayoutId,
}

fn fixture() -> Fixture {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(column(&log, "root", ChildMode::Loose));
    let a = tree.insert(column(&log, "a", ChildMode::Loose));
    let a1 = tree.insert(leaf(&log, "a1", 10.0, 5.0));
    let a2 = tree.insert(leaf(&log, "a2", 20.0, 5.0));
    let b = tree.insert(column(&log, "b", ChildMode::Tight));
    let b1 = tree.insert(column(&log, "b1", ChildMode::Loose));
    let b11 = tree.insert(leaf(&log, "b11", 7.0, 3.0));
    tree.set_children(root, &[a, b]).expect("valid");
    tree.set_children(a, &[a1, a2]).expect("valid");
    tree.set_children(b, &[b1]).expect("valid");
    tree.set_children(b1, &[b11]).expect("valid");
    Fixture {
        tree,
        log,
        root,
        a,
        a1,
        a2,
        b,
        b1,
        b11,
    }
}

/// A fixture after one full pass, with the log cleared.
fn laid_out() -> Fixture {
    let mut f = fixture();
    f.tree.layout(f.root, loose(100.0, 100.0));
    f.log.take();
    f
}

fn names(v: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = v.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

#[test]
fn layout_tree_01_new_nodes() {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    assert!(tree.is_empty());
    let a = tree.insert(leaf(&log, "a", 1.0, 1.0));
    let b = tree.insert(leaf(&log, "b", 1.0, 1.0));
    assert_ne!(a, b);
    assert_ne!(a.to_bits(), 0);
    assert_eq!(a.to_bits(), a.id().to_bits());
    assert_eq!(tree.len(), 2);
    assert!(tree.contains(a));
    assert_eq!(tree.parent(a), None);
    assert!(tree.children(a).is_empty());
    assert_eq!(tree.parent_data::<u32>(a), None);
    assert_eq!(tree.size(a), None);
    assert_eq!(tree.constraints(a), None);
    assert_eq!(tree.offset(a), Some(Vec2::ZERO));
    assert!(tree.needs_layout(a));
    assert_eq!(std::mem::size_of::<LayoutId>(), 8);
}

#[test]
fn layout_tree_02_remove_takes_the_subtree() {
    let mut f = laid_out();
    assert_eq!(f.tree.len(), 7);
    assert_eq!(f.tree.remove(f.b), 3);
    assert_eq!(f.tree.len(), 4);
    for gone in [f.b, f.b1, f.b11] {
        assert!(!f.tree.contains(gone));
        assert_eq!(f.tree.size(gone), None);
        assert_eq!(f.tree.offset(gone), None);
        assert_eq!(f.tree.constraints(gone), None);
        assert_eq!(f.tree.parent(gone), None);
        assert!(f.tree.children(gone).is_empty());
        assert!(!f.tree.needs_layout(gone));
        assert!(!f.tree.is_relayout_boundary(gone));
        assert!(f.tree.get::<Column>(gone).is_none());
        assert!(f.tree.get_mut::<Column>(gone).is_none());
        assert!(!f.tree.set(gone, Fixed(Size::ZERO)));
        assert!(!f.tree.replace(gone, Fixed(Size::ZERO)));
        assert!(!f.tree.set_parent_data(gone, Some(1u32)));
        assert_eq!(f.tree.min_intrinsic_width(gone, INF), 0.0);
        assert_eq!(f.tree.layout(gone, loose(10.0, 10.0)), Size::ZERO);
        f.tree.mark_needs_layout(gone);
        assert_eq!(f.tree.remove(gone), 0);
    }
    assert_eq!(f.tree.children(f.root), &[f.a]);
    assert!(f.tree.needs_layout(f.root));
    // Ids are never reused.
    let log = Log::default();
    for _ in 0..10 {
        let id = f.tree.insert(leaf(&log, "new", 1.0, 1.0));
        assert!(![f.b, f.b1, f.b11].contains(&id));
    }
    assert!(!f.tree.contains(f.b));
    // A foreign id never panics.
    let mut other = LayoutTree::new();
    let _ = other.size(f.root);
    other.mark_needs_layout(f.a1);
    let _ = other.layout(f.root, loose(1.0, 1.0));
}

#[test]
fn layout_tree_03_set_children() {
    let mut f = laid_out();
    assert_eq!(f.tree.children(f.a), &[f.a1, f.a2]);
    assert_eq!(f.tree.parent(f.a1), Some(f.a));
    // The same list again: nothing marked.
    f.tree.set_children(f.a, &[f.a1, f.a2]).expect("valid");
    assert!(!f.tree.needs_layout(f.a));
    // Reordering marks.
    f.tree.set_children(f.a, &[f.a2, f.a1]).expect("valid");
    assert_eq!(f.tree.children(f.a), &[f.a2, f.a1]);
    assert!(f.tree.needs_layout(f.a));
    // Dropping a child detaches it with its subtree.
    f.tree.set_children(f.root, &[f.a]).expect("valid");
    assert_eq!(f.tree.parent(f.b), None);
    assert!(f.tree.contains(f.b) && f.tree.contains(f.b11));
    assert_eq!(f.tree.children(f.b), &[f.b1]);
    // A detached node can be attached elsewhere.
    f.tree.set_children(f.a1, &[f.b]).expect("valid");
    assert_eq!(f.tree.parent(f.b), Some(f.a1));
    // An empty list detaches everything.
    f.tree.set_children(f.a, &[]).expect("valid");
    assert!(f.tree.children(f.a).is_empty());
    assert_eq!(f.tree.parent(f.a1), None);
}

#[test]
fn layout_tree_04_set_children_errors() {
    let mut f = laid_out();
    let log = Log::default();
    let stale = f.tree.insert(leaf(&log, "x", 1.0, 1.0));
    f.tree.remove(stale);
    let before = |t: &LayoutTree, f: &Fixture| {
        (
            t.children(f.root).to_vec(),
            t.children(f.a).to_vec(),
            t.children(f.b).to_vec(),
        )
    };
    let snapshot = before(&f.tree, &f);
    assert_eq!(
        f.tree.set_children(stale, &[]),
        Err(TreeError::UnknownNode(stale))
    );
    assert_eq!(
        f.tree.set_children(f.a, &[f.a1, stale]),
        Err(TreeError::UnknownNode(stale))
    );
    assert_eq!(
        f.tree.set_children(f.a, &[f.b1]),
        Err(TreeError::HasParent {
            child: f.b1,
            parent: f.b
        })
    );
    assert_eq!(
        f.tree.set_children(f.b11, &[f.root]),
        Err(TreeError::Cycle(f.root))
    );
    assert_eq!(
        f.tree.set_children(f.b11, &[f.b11]),
        Err(TreeError::Cycle(f.b11))
    );
    assert_eq!(
        f.tree.set_children(f.a, &[f.a1, f.a2, f.a1]),
        Err(TreeError::Duplicate(f.a1))
    );
    assert_eq!(before(&f.tree, &f), snapshot);
    assert!(!f.tree.needs_layout(f.a));
    assert!(!f.tree.needs_layout(f.root));
    assert!(TreeError::Cycle(f.root).to_string().contains("ancestor"));
}

#[test]
fn layout_tree_05_get_get_mut_replace() {
    let mut f = laid_out();
    assert_eq!(f.tree.get::<Leaf>(f.a1).map(|l| l.name), Some("a1"));
    assert!(f.tree.get::<Column>(f.a1).is_none());
    assert!(!f.tree.needs_layout(f.a1));
    // get_mut marks even without a change.
    assert!(f.tree.get_mut::<Column>(f.a1).is_none());
    assert!(!f.tree.needs_layout(f.a1));
    assert!(f.tree.get_mut::<Leaf>(f.a1).is_some());
    assert!(f.tree.needs_layout(f.a1) && f.tree.needs_layout(f.a));
    // A change through get_mut shows in the next pass.
    f.tree.layout(f.root, loose(100.0, 100.0));
    f.tree.get_mut::<Leaf>(f.a1).expect("leaf").size = Size::new(10.0, 50.0);
    f.tree.layout(f.root, loose(100.0, 100.0));
    assert_eq!(f.tree.size(f.a1), Some(Size::new(10.0, 50.0)));
    // replace keeps children and parent data and marks.
    f.tree.set_parent_data(f.a, Some(7u32));
    f.tree.layout(f.root, loose(100.0, 100.0));
    assert!(f.tree.replace(f.a, column(&f.log, "a'", ChildMode::Loose)));
    assert_eq!(f.tree.get::<Column>(f.a).map(|c| c.name), Some("a'"));
    assert_eq!(f.tree.children(f.a), &[f.a1, f.a2]);
    assert_eq!(f.tree.parent_data::<u32>(f.a), Some(&7));
    assert!(f.tree.needs_layout(f.a));
}

/// Reads its first child's `u32` parent data during layout into the log.
struct ParentDataReader(Log);

impl RenderBox for ParentDataReader {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        self.0.push(format!("{:?}", children.parent_data::<u32>(0)));
        self.0.push(format!("{:?}", children.parent_data::<f64>(0)));
        children.layout(0, c);
        c.smallest()
    }
}

#[test]
fn layout_tree_06_parent_data() {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let parent = tree.insert(ParentDataReader(log.clone()));
    let child = tree.insert(leaf(&log, "child", 1.0, 1.0));
    tree.set_children(parent, &[child]).expect("valid");
    // A root's parent data: stored, nothing to mark.
    assert!(tree.set_parent_data(parent, Some("root data")));
    assert_eq!(tree.parent_data::<&str>(parent), Some(&"root data"));

    assert!(tree.set_parent_data(child, Some(3u32)));
    assert_eq!(tree.parent_data::<u32>(child), Some(&3));
    assert_eq!(tree.parent_data::<i32>(child), None);
    tree.layout(parent, loose(10.0, 10.0));
    assert_eq!(log.take(), ["Some(3)", "None", "child"]);
    assert!(!tree.needs_layout(parent));

    assert!(tree.set_parent_data(child, Some(4u32)));
    assert!(tree.needs_layout(parent));
    tree.layout(parent, loose(10.0, 10.0));
    assert_eq!(log.take(), ["Some(4)", "None"]);

    assert!(tree.set_parent_data(child, None::<u32>));
    assert_eq!(tree.parent_data::<u32>(child), None);
    assert!(tree.needs_layout(parent));
}

#[test]
fn layout_tree_07_layout_runs_the_root() {
    let mut f = fixture();
    let log2 = f.log.clone();
    let other_root = f.tree.insert(leaf(&log2, "other", 1.0, 1.0));
    f.tree.set_parent_data(f.root, Some(1u32));
    let size = f.tree.layout(f.root, loose(100.0, 100.0));
    // root: a (20 wide, 10 tall) above b (tight at 100 × 10 → 100 wide).
    assert_eq!(size, Size::new(100.0, 20.0));
    assert_eq!(f.tree.size(f.root), Some(size));
    assert_eq!(f.tree.offset(f.root), Some(Vec2::ZERO));
    assert_eq!(
        f.log.take_sorted(),
        names(&["root", "a", "a1", "a2", "b", "b1", "b11"])
    );
    // Not in the subtree: untouched.
    assert!(f.tree.needs_layout(other_root));
    assert_eq!(f.tree.size(other_root), None);
    // Same constraints, nothing marked: nothing runs.
    f.tree.layout(f.root, loose(100.0, 100.0));
    assert!(f.log.take().is_empty());
    // New constraints: the root runs.
    let size = f.tree.layout(f.root, loose(50.0, 100.0));
    assert_eq!(size, Size::new(50.0, 20.0));
    assert!(f.log.take().contains(&"root".to_string()));
    // Unknown root.
    f.tree.remove(other_root);
    assert_eq!(f.tree.layout(other_root, loose(1.0, 1.0)), Size::ZERO);
}

/// Probes out-of-range indices and records the answers in the log.
struct Prober(Log);

impl RenderBox for Prober {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let n = children.len();
        self.0
            .push(format!("len {n} empty {}", children.is_empty()));
        self.0.push(format!("id {:?}", children.id(n).is_some()));
        self.0.push(format!("layout {:?}", children.layout(n, c)));
        children.layout_ignoring_size(n + 5, c);
        children.set_offset(n, Vec2::new(5.0, 5.0));
        self.0
            .push(format!("data {:?}", children.parent_data::<u32>(n)));
        self.0.push(format!(
            "intrinsics {} {} {} {}",
            children.min_intrinsic_width(n, INF),
            children.max_intrinsic_width(n, INF),
            children.min_intrinsic_height(n, INF),
            children.max_intrinsic_height(n, INF)
        ));
        for i in 0..n {
            assert!(children.id(i).is_some());
            children.layout(i, c);
            children.set_offset(i, Vec2::new(i as f32, 2.0 * i as f32));
        }
        c.smallest()
    }
}

#[test]
fn layout_tree_08_children_context() {
    let f = laid_out();
    // Offsets set by the column: a at the top, b below a.
    assert_eq!(f.tree.offset(f.a), Some(Vec2::new(0.0, 0.0)));
    assert_eq!(f.tree.offset(f.b), Some(Vec2::new(0.0, 10.0)));
    assert_eq!(f.tree.offset(f.a2), Some(Vec2::new(0.0, 5.0)));
    assert_eq!(f.tree.size(f.a2), Some(Size::new(20.0, 5.0)));

    let log = Log::default();
    let mut tree = LayoutTree::new();
    let p = tree.insert(Prober(log.clone()));
    let kids: Vec<LayoutId> = (0..3)
        .map(|_| tree.insert(Fixed(Size::new(1.0, 1.0))))
        .collect();
    tree.set_children(p, &kids).expect("valid");
    tree.layout(p, loose(10.0, 10.0));
    assert_eq!(
        log.take(),
        [
            "len 3 empty false",
            "id false",
            "layout Size { width: 0.0, height: 0.0 }",
            "data None",
            "intrinsics 0 0 0 0",
        ]
    );
    for (i, kid) in kids.iter().enumerate() {
        assert_eq!(tree.offset(*kid), Some(Vec2::new(i as f32, 2.0 * i as f32)));
    }
    // Offsets persist until set again.
    tree.mark_needs_layout(p);
    let _ = tree.layout(p, loose(10.0, 10.0));
    assert_eq!(tree.offset(kids[2]), Some(Vec2::new(2.0, 4.0)));
}

#[test]
fn layout_tree_09_sizes_are_constrained() {
    let mut tree = LayoutTree::new();
    let liar = tree.insert(Liar(Size::new(1000.0, f32::NAN)));
    let c = BoxConstraints::new(10.0, 100.0, 5.0, 50.0);
    assert_eq!(tree.layout(liar, c), Size::new(100.0, 5.0));
    assert_eq!(tree.size(liar), Some(Size::new(100.0, 5.0)));
    assert_eq!(tree.constraints(liar), Some(c));
    let small = tree.insert(Liar(Size::new(-5.0, 1.0)));
    assert_eq!(tree.layout(small, c), Size::new(10.0, 5.0));
}

#[test]
fn layout_tree_10_caching() {
    let mut f = laid_out();
    for id in [f.root, f.a, f.a1, f.a2, f.b, f.b1, f.b11] {
        assert!(!f.tree.needs_layout(id), "{id:?}");
    }
    assert_eq!(
        f.tree.layout(f.root, loose(100.0, 100.0)),
        Size::new(100.0, 20.0)
    );
    assert!(f.log.take().is_empty());
    // A taller root re-lays out the root only: its children get the same constraints as
    // before (they depend on the root's max width) and aren't dirty.
    f.tree
        .layout(f.root, BoxConstraints::new(0.0, 100.0, 0.0, 200.0));
    assert_eq!(f.log.take(), ["root"]);
}

#[test]
fn layout_tree_11_relayout_boundaries() {
    let f = laid_out();
    assert!(f.tree.is_relayout_boundary(f.root));
    assert!(!f.tree.is_relayout_boundary(f.a));
    assert!(!f.tree.is_relayout_boundary(f.a1));
    assert!(!f.tree.is_relayout_boundary(f.b));
    assert!(f.tree.is_relayout_boundary(f.b1));
    assert!(!f.tree.is_relayout_boundary(f.b11));

    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(column(&log, "root", ChildMode::IgnoringSize));
    let ignored = tree.insert(leaf(&log, "ignored", 1.0, 1.0));
    let fill = tree.insert(Fill {
        name: "fill",
        log: log.clone(),
    });
    let inner = tree.insert(column(&log, "inner", ChildMode::Loose));
    let filled = tree.insert(Fill {
        name: "filled",
        log: log.clone(),
    });
    tree.set_children(root, &[ignored, inner]).expect("valid");
    tree.set_children(inner, &[fill]).expect("valid");
    tree.set_children(fill, &[filled]).expect("valid");
    tree.layout(root, loose(100.0, 100.0));
    assert!(tree.is_relayout_boundary(ignored)); // laid out ignoring its size
    assert!(tree.is_relayout_boundary(fill)); // sized_by_parent
    assert!(tree.is_relayout_boundary(filled)); // sized_by_parent
}

#[test]
fn layout_tree_12_marking_stops_at_boundaries() {
    let mut f = laid_out();
    f.tree.mark_needs_layout(f.a1);
    for (id, expected) in [
        (f.a1, true),
        (f.a, true),
        (f.root, true),
        (f.a2, false),
        (f.b, false),
        (f.b1, false),
    ] {
        assert_eq!(f.tree.needs_layout(id), expected, "{id:?}");
    }
    let mut f = laid_out();
    f.tree.mark_needs_layout(f.b11);
    for (id, expected) in [
        (f.b11, true),
        (f.b1, true),
        (f.b, false),
        (f.root, false),
        (f.a, false),
    ] {
        assert_eq!(f.tree.needs_layout(id), expected, "{id:?}");
    }
    // A node never laid out counts as not a boundary: `fresh` hangs under the leaf a1, which
    // doesn't lay out children, so it is never laid out and marking it reaches the root.
    let mut f = laid_out();
    let fresh = f.tree.insert(leaf(&f.log, "fresh", 1.0, 1.0));
    f.tree.set_children(f.a1, &[fresh]).expect("valid");
    f.tree.layout(f.root, loose(100.0, 100.0));
    assert_eq!(f.tree.size(fresh), None);
    assert!(!f.tree.needs_layout(f.root));
    f.tree.mark_needs_layout(fresh);
    assert!(f.tree.needs_layout(f.a1) && f.tree.needs_layout(f.a) && f.tree.needs_layout(f.root));
}

/// Picks a node of the fixture.
type Pick = fn(&Fixture) -> LayoutId;

#[test]
fn layout_tree_13_minimal_relayout() {
    let cases: [(&[Pick], &[&str]); 5] = [
        (&[|f| f.a1], &["root", "a", "a1"]),
        (&[|f| f.a2], &["root", "a", "a2"]),
        (&[|f| f.b11], &["b1", "b11"]),
        (&[|f| f.a1, |f| f.b11], &["root", "a", "a1", "b1", "b11"]),
        // b is dirty and so is b1 (a boundary inside b's relayout): b1 runs once.
        (&[|f| f.b, |f| f.b11], &["root", "b", "b1", "b11"]),
    ];
    for (marks, expected) in cases {
        let mut f = laid_out();
        for mark in marks {
            let id = mark(&f);
            f.tree.mark_needs_layout(id);
        }
        f.tree.layout(f.root, loose(100.0, 100.0));
        assert_eq!(f.log.take_sorted(), names(expected), "{expected:?}");
        for id in [f.root, f.a, f.a1, f.a2, f.b, f.b1, f.b11] {
            assert!(!f.tree.needs_layout(id), "{expected:?}: {id:?}");
        }
    }
    // A child that changes size makes its non-boundary parent re-lay out its siblings' offsets.
    let mut f = laid_out();
    f.tree.get_mut::<Leaf>(f.a1).expect("leaf").size = Size::new(10.0, 15.0);
    f.tree.layout(f.root, loose(100.0, 100.0));
    assert_eq!(f.tree.offset(f.a2), Some(Vec2::new(0.0, 15.0)));
    assert_eq!(f.tree.offset(f.b), Some(Vec2::new(0.0, 20.0)));
    assert_eq!(f.log.take_sorted(), names(&["root", "a", "a1"]));
}

#[test]
fn layout_tree_14_structural_changes() {
    // A new child list.
    let mut f = laid_out();
    let extra = f.tree.insert(leaf(&f.log, "extra", 5.0, 5.0));
    f.tree
        .set_children(f.a, &[f.a1, f.a2, extra])
        .expect("valid");
    f.tree.layout(f.root, loose(100.0, 100.0));
    assert_eq!(f.log.take_sorted(), names(&["root", "a", "extra"]));
    assert_eq!(f.tree.offset(extra), Some(Vec2::new(0.0, 10.0)));
    // A removed child.
    f.tree.remove(f.a1);
    f.tree.layout(f.root, loose(100.0, 100.0));
    assert_eq!(f.log.take_sorted(), names(&["root", "a"]));
    assert_eq!(f.tree.offset(f.a2), Some(Vec2::ZERO));
    // A detached node keeps its geometry and isn't laid out by its old tree.
    let size = f.tree.size(f.b1);
    let offset = f.tree.offset(f.b1);
    f.tree.set_children(f.b, &[]).expect("valid");
    f.tree.mark_needs_layout(f.b11);
    f.tree.layout(f.root, loose(100.0, 100.0));
    assert_eq!(f.log.take_sorted(), names(&["root", "b"]));
    assert_eq!(f.tree.size(f.b1), size);
    assert_eq!(f.tree.offset(f.b1), offset);
    assert!(f.tree.needs_layout(f.b11));
}

#[test]
fn layout_tree_15_intrinsics_never_lay_out() {
    let mut f = laid_out();
    assert_eq!(f.tree.min_intrinsic_width(f.a, INF), 20.0);
    assert_eq!(f.tree.max_intrinsic_height(f.a, 100.0), 10.0);
    assert_eq!(f.tree.min_intrinsic_height(f.a1, 3.0), 5.0);
    assert_eq!(f.tree.max_intrinsic_width(f.a1, 3.0), 10.0);
    let log = f.log.take();
    assert!(log.iter().all(|e| e.contains(':')), "{log:?}");
    assert!(log.contains(&"a2:min_w".to_string()));
    // Defaults are 0; NaN and negative are reported as 0.
    let mut tree = LayoutTree::new();
    let plain = tree.insert(Plain);
    assert_eq!(tree.min_intrinsic_width(plain, INF), 0.0);
    assert_eq!(tree.max_intrinsic_width(plain, INF), 0.0);
    assert_eq!(tree.min_intrinsic_height(plain, INF), 0.0);
    assert_eq!(tree.max_intrinsic_height(plain, INF), 0.0);
    let bad = tree.insert(Liar(Size::new(f32::NAN, -4.0)));
    assert_eq!(tree.min_intrinsic_width(bad, INF), 0.0);
    assert_eq!(tree.max_intrinsic_height(bad, INF), 0.0);
    let big = tree.insert(Liar(Size::new(INF, 3.0)));
    assert_eq!(tree.min_intrinsic_width(big, 1.0), INF);
    assert_eq!(tree.size(big), None);
}

#[test]
fn layout_tree_16_intrinsic_cache() {
    let mut f = laid_out();
    assert_eq!(f.tree.min_intrinsic_width(f.a, INF), 20.0);
    f.log.take();
    // Cached per node, method and argument.
    assert_eq!(f.tree.min_intrinsic_width(f.a, INF), 20.0);
    assert!(f.log.take().is_empty());
    assert_eq!(f.tree.min_intrinsic_width(f.a, 50.0), 20.0);
    assert_eq!(
        f.log.take_sorted(),
        names(&["a1:min_w", "a2:min_w", "a:min_w"])
    );
    assert_eq!(f.tree.max_intrinsic_height(f.a, 50.0), 10.0);
    assert_eq!(
        f.log.take_sorted(),
        names(&["a1:max_h", "a2:max_h", "a:max_h"])
    );
    // Marking a descendant clears the caches on its path.
    f.tree.get_mut::<Leaf>(f.a2).expect("leaf").size = Size::new(30.0, 5.0);
    assert_eq!(f.tree.min_intrinsic_width(f.a, INF), 30.0);
    assert_eq!(f.log.take_sorted(), names(&["a2:min_w", "a:min_w"]));

    // Without cached intrinsics, marking stops at the boundary b1.
    let mut f = laid_out();
    f.tree.mark_needs_layout(f.b11);
    assert!(!f.tree.needs_layout(f.b) && !f.tree.needs_layout(f.root));
    // With cached intrinsics on b11 and b1, marking continues past b1.
    let mut f = laid_out();
    f.tree.min_intrinsic_width(f.b1, INF);
    f.tree.mark_needs_layout(f.b11);
    assert!(f.tree.needs_layout(f.b1));
    assert!(f.tree.needs_layout(f.b) && f.tree.needs_layout(f.root));
}

/// Nests `depth` loose columns and returns (tree, root, deepest).
fn deep(depth: usize) -> (LayoutTree, LayoutId, LayoutId) {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let root = tree.insert(column(&log, "c", ChildMode::Loose));
    let mut parent = root;
    for _ in 0..depth {
        let child = tree.insert(column(&log, "c", ChildMode::Loose));
        tree.set_children(parent, &[child]).expect("valid");
        parent = child;
    }
    let leaf_id = tree.insert(leaf(&log, "leaf", 3.0, 4.0));
    tree.set_children(parent, &[leaf_id]).expect("valid");
    (tree, root, leaf_id)
}

#[test]
fn layout_tree_17_robustness() {
    let (mut tree, root, deepest) = deep(1000);
    assert_eq!(tree.layout(root, loose(100.0, 100.0)), Size::new(3.0, 4.0));
    tree.mark_needs_layout(deepest);
    assert_eq!(tree.layout(root, loose(100.0, 100.0)), Size::new(3.0, 4.0));
    assert_eq!(tree.min_intrinsic_width(root, INF), 3.0);
    // Any constraints.
    let mut f = laid_out();
    for c in [
        BoxConstraints::new(f32::NAN, f32::NAN, f32::NAN, f32::NAN),
        BoxConstraints::new(INF, INF, INF, INF),
        BoxConstraints::new(50.0, 10.0, -5.0, -INF),
        BoxConstraints::UNCONSTRAINED,
    ] {
        let _ = f.tree.layout(f.root, c);
        let _ = f.tree.min_intrinsic_width(f.root, f32::NAN);
        let _ = f.tree.max_intrinsic_height(f.root, -INF);
    }
}

#[test]
fn layout_tree_18_set_marks_only_on_change() {
    let mut tree = LayoutTree::new();
    let parent = tree.insert(Fixed(Size::new(50.0, 50.0)));
    let id = tree.insert(Fixed(Size::new(10.0, 10.0)));
    tree.set_children(parent, &[id]).expect("valid");
    tree.set_parent_data(id, Some(9u8));
    tree.layout(id, loose(100.0, 100.0));
    assert!(!tree.needs_layout(id));
    // Equal: nothing changes.
    assert!(!tree.set(id, Fixed(Size::new(10.0, 10.0))));
    assert!(!tree.needs_layout(id));
    // Different value: replaced and marked.
    assert!(tree.set(id, Fixed(Size::new(20.0, 10.0))));
    assert!(tree.needs_layout(id));
    assert_eq!(tree.get::<Fixed>(id), Some(&Fixed(Size::new(20.0, 10.0))));
    assert_eq!(tree.parent_data::<u8>(id), Some(&9));
    tree.layout(id, loose(100.0, 100.0));
    assert_eq!(tree.size(id), Some(Size::new(20.0, 10.0)));
    // Different type: replaced and marked, children kept.
    let child = tree.insert(Fixed(Size::ZERO));
    tree.set_children(id, &[child]).expect("valid");
    tree.layout(id, loose(100.0, 100.0));
    assert!(tree.set(id, Fill2(1)));
    assert!(tree.needs_layout(id));
    assert_eq!(tree.children(id), &[child]);
    assert!(tree.get::<Fixed>(id).is_none());
    // Unknown id.
    tree.remove(child);
    assert!(!tree.set(child, Fixed(Size::ZERO)));
}

/// Another comparable layout type.
#[derive(PartialEq)]
struct Fill2(u8);

impl RenderBox for Fill2 {
    fn perform_layout(&mut self, c: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        c.biggest()
    }
}

/// Records `children.size(i)` before and after laying out its children.
struct SizeReader(Log);

impl RenderBox for SizeReader {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let n = children.len();
        self.0.push(format!("before {:?}", children.size(0)));
        if n > 0 {
            let got = children.layout(0, c);
            self.0.push(format!("same {}", got == children.size(0)));
        }
        self.0.push(format!("after {:?}", children.size(0)));
        self.0.push(format!("never {:?}", children.size(1)));
        self.0
            .push(format!("out of range {:?}", children.size(n + 3)));
        c.smallest()
    }
}

#[test]
fn layout_tree_19_child_sizes() {
    let log = Log::default();
    let mut tree = LayoutTree::new();
    let p = tree.insert(SizeReader(log.clone()));
    let first = tree.insert(Fixed(Size::new(7.0, 3.0)));
    let second = tree.insert(Fixed(Size::new(1.0, 1.0)));
    tree.set_children(p, &[first, second]).expect("valid");
    tree.layout(p, loose(10.0, 10.0));
    let zero = format!("{:?}", Size::ZERO);
    let seven = format!("{:?}", Size::new(7.0, 3.0));
    assert_eq!(
        log.take(),
        [
            format!("before {zero}"),
            "same true".to_string(),
            format!("after {seven}"),
            format!("never {zero}"),
            format!("out of range {zero}"),
        ]
    );
    // The last layout's size is kept for the next pass.
    tree.mark_needs_layout(p);
    tree.layout(p, loose(10.0, 10.0));
    assert_eq!(log.take()[0], format!("before {seven}"));
}
