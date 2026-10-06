# Phase 1 demo: a Scene in a window

- **Status:** Implemented
- **Crate:** `scene-window` (`examples/scene-window`)
- **Plan item:** Phase 1, "`examples/scene-window`: Phase 1 demo"
- **Related:** [scene](../scene/scene.md), [wgpu renderer](../render-wgpu/renderer.md),
  [platform](../platform/platform.md), [winit shell](../platform-winit/shell.md)

## Purpose

The first thing a person can look at: a window showing a hand-built Scene drawn by the GPU
renderer, with every Phase 1 command in it (rounded rects, strokes, shadows, an image, clips,
transforms, layers with opacity and an overlay color) and a little animation, redrawn on resize
and scale-factor changes. It shows the Phase 1 pieces working together before the `App` runner
and widgets exist.

It is a temporary exception to "examples depend only on the `tantu` facade" (AGENTS.md): it uses
`tantu-platform-winit`, `tantu-render-wgpu`, `tantu-scene` and `tantu-core` directly, because
the facade arrives in Phase 2. It is rewritten on the facade then.

## Scope

In scope:

- `scene_window::demo_scene(builder_target, size, time, image)`: records the demo frame for a
  window of `size` logical pixels at animation time `time` seconds.
- `scene_window::demo_image()`: the procedurally generated image the demo draws.
- `main`: opens a window with `WinitPlatform`, renders `demo_scene` with `WgpuRenderer` every
  frame (continuous redraw for the animation), follows resizes and scale-factor changes, closes
  on request.

Out of scope: text (Phase 2), input handling beyond closing the window.

## Public API

```rust
use tantu_core::Size;
use tantu_scene::{ImageData, ImageId, Scene};

/// The 64 × 64 RGBA8 image the demo draws (a hue gradient with a checker pattern).
pub fn demo_image() -> ImageData;

/// Records the demo frame into `scene` for a window of `size` logical pixels at animation time
/// `time` (seconds), drawing `image` (registered from `demo_image`).
pub fn demo_scene(scene: &mut Scene, size: Size, time: f32, image: ImageId);
```

## Behavior

- **SCENE-WINDOW-01:** `demo_image()` is a valid 64 × 64 image with every pixel opaque.
- **SCENE-WINDOW-02:** For any window size from 1 × 1 to 4000 × 3000 and any finite time,
  `demo_scene` records a balanced frame of that size (its `finish` returns `Ok`), and with the
  image registered, `RenderReport::for_scene` is clean: no invalid commands, nothing missing.
- **SCENE-WINDOW-03:** Rendered by the software renderer at 900 × 600 (scale 1) and 450 × 300
  logical pixels at scale 2, the background fills the whole window (opaque corner pixels) and the
  frame is not a single color.

Window behavior (`main`) is checked by running `cargo run -p scene-window`: a window opens,
animates, follows resizing and closes with the window's close button. That is a manual check
(the window tests need a display, see the winit shell spec).

## Performance and allocation

The `Scene` is reused across frames (`Scene::begin`), so recording allocates nothing once warm.

## Open questions

Resolved (2026-10-06, decided in autopilot): depending on the crates directly until the facade
exists, rather than adding a minimal facade early (the user agreed to proceed on this basis).
