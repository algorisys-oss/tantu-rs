//! SCENE-SCENE-24: no allocation once warm. A test binary of its own, because it installs a
//! counting global allocator; it holds a single test so no other test allocates concurrently.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use tantu_core::{Affine, Color, Point, Rect, Size, Vec2};
use tantu_scene::{Clip, CustomKind, ElementId, FontId, Glyph, Layer, Scene};

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

/// A frame with `n` elements, each in nested scopes, with glyphs, custom data, damage and
/// varied z-indexes (so `finish` has to sort).
fn record(scene: &mut Scene, n: u64, glyphs: &[Glyph], data: &[u8]) {
    let font = FontId::from_raw(1).expect("1 is not 0");
    let mut b = scene.begin(Size::new(800.0, 600.0));
    for i in 0..n {
        b.set_element(ElementId::from_raw(i + 1));
        b.set_z_index((i % 3) as i32 - 1);
        b.push_transform(Affine::translate(Vec2::new(i as f32, 0.0)));
        b.push_layer(Layer::default());
        b.push_clip(Clip::Rect(Rect::from_ltwh(0.0, 0.0, 50.0, 50.0)));
        b.set_z_index((i % 2) as i32);
        if !b.is_culled(Rect::from_ltwh(0.0, 0.0, 10.0, 10.0)) {
            b.fill_rect(Rect::from_ltwh(0.0, 0.0, 10.0, 10.0), Color::BLACK);
        }
        b.glyph_run(font, 12.0, Color::BLACK, Point::ZERO, glyphs);
        b.custom(CustomKind(1), Rect::from_ltwh(0.0, 0.0, 5.0, 5.0), data);
        b.add_damage(Rect::from_ltwh(i as f32, 0.0, 10.0, 10.0));
        b.pop();
        b.pop();
        b.pop();
    }
    b.finish().expect("scopes are balanced");
}

#[test]
fn scene_scene_24_no_allocation_once_warm() {
    let glyphs = vec![
        Glyph {
            id: 1,
            x: 0.0,
            y: 0.0
        };
        16
    ];
    let data = vec![7u8; 32];
    let mut scene = Scene::new();
    record(&mut scene, 200, &glyphs, &data); // warm-up

    let before = ALLOCATIONS.load(Ordering::Relaxed);
    record(&mut scene, 200, &glyphs, &data);
    record(&mut scene, 50, &glyphs[..4], &data[..8]);
    record(&mut scene, 200, &glyphs, &data);
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(after - before, 0, "allocations after warm-up");
    assert!(!scene.entries().is_empty());
}
