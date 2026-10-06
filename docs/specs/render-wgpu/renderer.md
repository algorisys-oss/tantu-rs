# GPU renderer (wgpu)

- **Status:** Agreed
- **Crate:** `tantu-render-wgpu`
- **Plan item:** Phase 1, "`tantu-render-wgpu`: rects, rounded rects, borders, shadows, clips,
  images"
- **Related:** [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md),
  [ADR 0006](../../adr/0006-wgpu-as-the-default-gpu-backend.md), [scene](../scene/scene.md)
  (drawing semantics), [renderer and resources](../scene/renderer.md) (contract,
  `RenderReport::for_scene`), [software renderer](../render-soft/renderer.md) (the reference
  this renderer is compared against)

## Purpose

The default renderer: draws a Scene on the GPU with [wgpu](https://wgpu.rs) (Vulkan, Metal,
DX12, GL). It renders either into a window surface (for apps) or into an offscreen texture that
can be read back (for tests and the Phase 1 milestone, which compares it with the software
renderer).

Its behavior rules are the software renderer's rules, restated for this backend, so both
backends are held to the same contract and tested the same way.

## Scope

In scope:

- `WgpuRenderer`: implements `Renderer`. Created offscreen (`new_offscreen`) or for a window
  (`for_window`, any `raw-window-handle` 0.6 window, so it doesn't depend on the platform crates).
- Every Phase 1 command: fills and strokes of (rounded) rects, box shadows, images, clips
  (rect and rounded rect), transforms, layers (opacity, overlay color), custom commands through
  registered handlers.
- `snapshot()`: reads an offscreen target back as `ImageData`.
- Image upload cache by `ImageId`, pruned by `Resources::revision`.

Out of scope (and where it goes):

- Text (as in the software renderer, glyph runs count as `missing_fonts` until Phase 2).
- Sharing one device between several windows, MSAA, partial repaint, layer bounds: later
  (Phase 2 app runner, Phase 5).
- The golden comparison against the software renderer: the milestone item (next spec).

## Dependencies

- `wgpu` 30 (MIT OR Apache-2.0). Allowed only in `tantu-render-*` (AGENTS.md). Its MSRV is
  Rust 1.87, so the workspace MSRV and the pinned toolchain move from 1.85 to 1.87 (decision 12
  in HANDOFF.md anticipated this; `rust-toolchain.toml` and `rust-version` change together).
- `pollster` (MIT OR Apache-2.0) to block on wgpu's async adapter and device requests.
- `raw-window-handle` 0.6 for `for_window` (already in the workspace through `tantu-platform`).
- `tantu-core` directly (as `tantu-render-soft`; see the open question in HANDOFF.md).

## Public API

Crate root `tantu_render_wgpu`.

```rust
use tantu_core::Rect;
use tantu_scene::{CustomKind, ImageData, RenderError, RenderReport, Renderer, Resources, Scene};

/// The wgpu version this crate draws with, for custom handlers.
pub use wgpu;

/// Why a renderer couldn't be created.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateError {
    /// No GPU adapter fits (no driver, or none supports what Tantu needs).
    NoAdapter,
    /// Creating the device or the surface failed.
    Backend(Box<dyn std::error::Error + Send + Sync>),
}
impl std::fmt::Display for CreateError { /* ... */ }
impl std::error::Error for CreateError { /* source() is the Backend error */ }

/// A renderer that draws Scenes with wgpu.
pub struct WgpuRenderer { /* private */ }

impl WgpuRenderer {
    /// An offscreen `width × height` target (RGBA8, premultiplied alpha), scale factor 1,
    /// fully transparent. Blocks while wgpu finds an adapter and creates a device.
    pub fn new_offscreen(width: u32, height: u32) -> Result<WgpuRenderer, CreateError>;
    /// A renderer drawing into `window`'s surface, `width × height` physical pixels.
    pub fn for_window<W>(window: W, width: u32, height: u32) -> Result<WgpuRenderer, CreateError>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync + 'static;
    /// Current target width and height in physical pixels.
    pub fn size(&self) -> (u32, u32);
    /// Current scale factor.
    pub fn scale_factor(&self) -> f32;
    /// Draw custom commands of `kind` with `handler`, replacing any earlier handler for it.
    pub fn register_custom(&mut self, kind: CustomKind, handler: impl CustomHandler + 'static);
    /// An offscreen target's pixels as straight-alpha RGBA8, read back from the GPU. `None` for
    /// a window renderer or a 0-width or 0-height target.
    pub fn snapshot(&self) -> Option<ImageData>;
    /// Information about the adapter in use (name, backend), for logs and test output.
    pub fn adapter_info(&self) -> wgpu::AdapterInfo;
}

impl Renderer for WgpuRenderer { /* resize, render */ }
impl std::fmt::Debug for WgpuRenderer { /* ... */ }

/// Draws one kind of custom command with wgpu.
pub trait CustomHandler {
    /// Record drawing for `canvas.data` into `canvas.target`, inside `canvas.bounds`.
    fn draw(&mut self, canvas: CustomCanvas<'_>);
}

/// What a custom handler draws into.
pub struct CustomCanvas<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    /// Commands recorded here run in paint order with the renderer's own.
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// The texture to draw into (the target, or the open layer's offscreen texture).
    pub target: &'a wgpu::TextureView,
    /// Its format (premultiplied alpha, sRGB-encoded values, no hardware sRGB conversion).
    pub format: wgpu::TextureFormat,
    /// Its size in physical pixels.
    pub target_size: (u32, u32),
    /// Maps the command's local coordinates to target pixels, as `[a, b, c, d, e, f]` (as
    /// `tantu_core::Affine`, times the scale factor).
    pub transform: [f32; 6],
    /// The bounding box of the current clip in target pixels (`[x, y, width, height]`), to use as
    /// a scissor rect; `None` with no open clip. Rounded and rotated clips are approximated by
    /// their bounding box here.
    pub clip: Option<[u32; 4]>,
    /// The command's bounds, in local coordinates.
    pub bounds: Rect,
    /// The command's bytes (`Scene::custom_data`).
    pub data: &'a [u8],
}
```

## Behavior

Tests use offscreen renderers and read pixels with `snapshot()`. As in the software renderer's
spec, "inside" and "outside" a shape mean at least 1 physical pixel away from its edge (edge
anti-aliasing differs between backends and is compared by the milestone, not by these rules), and
colors match within ±1 per channel unless stated.

When no adapter is available, `new_offscreen` returns `Err(CreateError::NoAdapter)` and the
tests report that they were skipped, unless `TANTU_REQUIRE_GPU=1` is set, in which case they
fail. CI sets it on Linux, where Mesa's lavapipe (a software Vulkan driver) is installed.

- **RENDER-WGPU-01:** `new_offscreen(w, h)` (`w`, `h` ≥ 1) has `size() == (w, h)`, scale factor 1,
  and a fully transparent `w × h` snapshot. A 0 width or height is allowed and `snapshot()` is
  then `None`. Creation never panics; with no usable adapter it returns `NoAdapter`.
- **RENDER-WGPU-02:** `resize` sets the size and the scale factor (1 if not finite or not positive)
  and leaves a fully transparent target of the new size.
- **RENDER-WGPU-03:** Every `render` starts from a fully transparent target.
- **RENDER-WGPU-04:** The report is `RenderReport::for_scene(scene, resources, <kind registered>)`
  plus 1 in `missing_fonts` for every glyph run with a present font that isn't hidden or invalid
  (no text yet; glyph runs draw nothing). With a 0 width or height, `render` draws nothing and
  returns the default report.
- **RENDER-WGPU-05:** Scene coordinates are multiplied by the scale factor; content outside the
  target is clipped; uncovered pixels stay transparent.
- **RENDER-WGPU-06:** The same inputs give byte-identical snapshots across two renders of one
  renderer.
- **RENDER-WGPU-07:** Fills: inside a fill the snapshot has the fill's color (opaque exactly;
  translucent within ±1). Blending is source-over on premultiplied sRGB-encoded values: white at
  alpha 0.5 over black is (128, 128, 128, 255) ±1.
- **RENDER-WGPU-08:** Rounded corners and radius clamping as RENDER-SOFT-10 (CSS rule, negative
  radii count as 0).
- **RENDER-WGPU-09:** Strokes lie inside the shape as RENDER-SOFT-11; width ≥ half the shorter side
  fills, width ≤ 0 draws nothing. Empty or reversed rects and transparent colors draw nothing.
- **RENDER-WGPU-10:** Box shadows: blur 0 gives the hard shape inflated by the spread (radii grown,
  not below 0) and moved by the offset; blur > 0 has the properties of RENDER-SOFT-14 (opaque deep
  inside, clear beyond 3 sigma, about half alpha at a straight edge, sigma scaled by the scale
  factor and the transform's scale).
- **RENDER-WGPU-11:** Images: scaled to fill `dest` from `src`; nearest sampling reproduces source
  pixels as uniform blocks; bilinear blends; alpha times `opacity` clamped to 0..=1; missing or
  removed images draw nothing and are counted.
- **RENDER-WGPU-12:** Transforms apply to their scope, compose outer-first, and are restored on
  pop (translate, scale and 90° rotation probes as RENDER-SOFT-18).
- **RENDER-WGPU-13:** Clips (rect and rounded rect, in the coordinates in effect at push) limit
  their scope, nest by intersection and are restored on pop.
- **RENDER-WGPU-14:** Layers are groups composited with opacity clamped to 0..=1 (NaN as 0);
  overlay colors mix source-atop; a non-finite overlay color is ignored.
- **RENDER-WGPU-15:** Custom handlers are called once per entry of their kind, in paint order,
  with the bounds, bytes, transform (times the scale factor) and clip bounding box; a later
  registration replaces the handler; without one nothing is drawn.
- **RENDER-WGPU-16:** Invalid commands and everything inside a non-finite transform or clip draw
  nothing (custom handlers included).
- **RENDER-WGPU-17:** `render` never panics for huge or tiny coordinates, huge blur radii, deep
  nesting and large images (as RENDER-SOFT-24). Images larger than the device's texture size
  limit draw nothing and count as missing.

## Implementation notes

Not rules; they explain how the rules are met.

- **Color.** Targets and offscreen textures use `Rgba8Unorm` (window surfaces a non-sRGB
  format such as `Bgra8Unorm`) so blending happens on sRGB-encoded values, as in the software
  renderer and Flutter. Colors and images are premultiplied before upload.
- **Shapes.** Fills, strokes and shadows are instanced quads whose fragment shader evaluates a
  rounded-rect signed distance in the shape's local space; coverage comes from the distance over
  its screen-space derivative (about one pixel of anti-aliasing). Strokes are the difference of
  the outer and inner shapes. Shadows use the closed-form Gaussian of a blurred rounded rect
  (Evan Wallace's approximation).
- **Clips.** Each open clip has an `R8Unorm` coverage mask the size of the target: the parent mask
  times the clip shape. Shape and image shaders multiply their coverage by the current mask.
  Masks and layer textures are pooled by nesting depth.
- **Layers.** Drawn into an offscreen `Rgba8Unorm` texture, then the overlay color is applied with
  a source-atop blend and the texture is composited with the layer opacity.
- **Huge coordinates.** Device-space positions are clamped to ±2^24, like the software renderer.
- **Batching.** Consecutive shapes are batched into one draw until a scope or pipeline change.

## Performance and allocation

Instance and uniform buffers are reused and grown as needed; textures for masks, layers and
images are pooled or cached. No per-frame allocation is promised yet; Phase 5 profiles it.

## Open questions

Resolved (2026-10-06, decided in autopilot at the user's request; review these):

1. **wgpu 30 and MSRV 1.87.** use the current wgpu and raise the MSRV, rather than stay
   on wgpu 26 (the last release that builds on 1.85).
2. **GPU tests in CI.** install Mesa's lavapipe on the Linux runner and set
   `TANTU_REQUIRE_GPU=1` there; on Windows and macOS runners the tests run if an adapter is found
   and otherwise report a skip.
3. **Custom-handler clip.** a scissor bounding box only, for now. Handlers that need the
   exact rounded clip can get the mask texture later.
4. **One device per renderer.** For Phase 1; the app runner shares one device between
   windows in Phase 2.
