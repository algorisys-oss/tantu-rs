//! SCENE-DAMAGE-05: `diff_damage` allocates nothing once warm. A test binary of its own,
//! because it installs a counting global allocator; it holds a single test.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use tantu_core::{Affine, Color, Rect, Size, Vec2};
use tantu_scene::{ElementId, Scene};

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

/// `n` elements, each a box in a transform, element `changed` in another color.
fn record(scene: &mut Scene, n: u64, changed: u64) {
    let mut b = scene.begin(Size::new(800.0, 600.0));
    for i in 0..n {
        b.set_element(ElementId::from_raw(i + 1));
        b.push_transform(Affine::translate(Vec2::new(
            (i % 40) as f32 * 20.0,
            (i / 40) as f32 * 20.0,
        )));
        let color = if i == changed {
            Color::WHITE
        } else {
            Color::BLACK
        };
        b.fill_rect(Rect::from_ltwh(0.0, 0.0, 10.0, 10.0), color);
        b.pop();
    }
    b.finish().expect("balanced");
}

#[test]
fn scene_damage_05_no_allocation_once_warm() {
    let (mut previous, mut next) = (Scene::new(), Scene::new());
    // Warm up: record and diff both ways a few times.
    for round in 0..4 {
        record(&mut previous, 600, round);
        record(&mut next, 600, round + 1);
        next.diff_damage(&previous);
        previous.diff_damage(&next);
    }
    record(&mut previous, 600, 7);
    record(&mut next, 600, 8);
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    next.diff_damage(&previous);
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(after - before, 0, "diff_damage allocated once warm");
}
