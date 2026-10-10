//! The layout tree's allocation promise (spec `docs/specs/layout/tree.md`, "Performance and
//! allocation"): a pass over a tree whose structure didn't change allocates nothing. A test
//! binary of its own, because it installs a counting global allocator; it holds a single test
//! so no other test allocates concurrently.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use tantu_core::{Size, Vec2};
use tantu_layout::{BoxConstraints, LayoutChildren, LayoutId, LayoutTree, RenderBox};

/// The system allocator, counting allocations.
struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every method forwards to `System` with the caller's arguments unchanged, so the
// `GlobalAlloc` contract is upheld by `System`.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: same layout the caller passed, which satisfies `alloc`'s contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` was allocated by `System` (through `alloc`/`realloc` above) with `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: `ptr` and `layout` come from `System`; the caller upholds `realloc`'s contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Stacks children vertically; tight children when `tight` (so they are boundaries).
struct Stack {
    tight: bool,
}

impl RenderBox for Stack {
    fn perform_layout(&mut self, c: BoxConstraints, children: &mut LayoutChildren<'_>) -> Size {
        let mut y = 0.0f32;
        let mut width = 0.0f32;
        for i in 0..children.len() {
            let child = if self.tight {
                BoxConstraints::tight(Size::new(c.constrain_width(f32::INFINITY), 10.0))
            } else {
                BoxConstraints::new(0.0, c.max_width, 0.0, f32::INFINITY)
            };
            let size = children.layout(i, child);
            children.set_offset(i, Vec2::new(0.0, y));
            y += size.height;
            width = width.max(size.width);
        }
        Size::new(width, y)
    }
}

struct Leaf(Size);

impl RenderBox for Leaf {
    fn perform_layout(&mut self, c: BoxConstraints, _: &mut LayoutChildren<'_>) -> Size {
        c.constrain(self.0)
    }
}

#[test]
fn layout_pass_allocates_nothing_once_warm() {
    let mut tree = LayoutTree::new();
    let root = tree.insert(Stack { tight: false });
    let mut leaves: Vec<LayoutId> = Vec::new();
    let mut groups = Vec::new();
    for g in 0..50 {
        let group = tree.insert(Stack { tight: g % 2 == 0 });
        let kids: Vec<LayoutId> = (0..20)
            .map(|i| tree.insert(Leaf(Size::new(i as f32, 5.0))))
            .collect();
        tree.set_children(group, &kids).expect("valid");
        leaves.extend(&kids);
        groups.push(group);
    }
    tree.set_children(root, &groups).expect("valid");

    let pass = |tree: &mut LayoutTree, width: f32| {
        for (i, leaf) in leaves.iter().enumerate() {
            if i % 7 == 0 {
                tree.mark_needs_layout(*leaf);
            }
        }
        tree.layout(root, BoxConstraints::loose(Size::new(width, 10_000.0)))
    };
    // Warm up: a full pass, marked passes, a constraint change.
    for width in [800.0, 800.0, 600.0, 800.0] {
        pass(&mut tree, width);
    }

    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for width in [800.0, 800.0, 600.0, 800.0, 800.0] {
        pass(&mut tree, width);
    }
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(after - before, 0, "allocations during warm layout passes");
}
