# Renderer conformance and the Phase 1 milestone

- **Status:** Implemented
- **Crate:** `tantu-render-conformance` (new); milestone tests in `tantu-render-wgpu` and
  `scene-window`
- **Plan item:** Phase 1, "**Milestone:** a hand-built Scene renders the same in wgpu and
  software (golden diff within the cross-backend tolerance)"
- **Related:** [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md),
  [ADR 0008](../../adr/0008-renderer-conformance-suite.md),
  [software renderer](../render-soft/renderer.md) (produces the goldens),
  [wgpu renderer](../render-wgpu/renderer.md), [scene-window demo](../examples/scene-window.md)

## Purpose

The Scene contract (ADR 0003) promises that one Scene looks the same on every renderer. Each
renderer's probe-based rules check pixels well inside and outside shapes. They deliberately say
nothing about edges, blur falloff or how the commands combine in a full frame. This item closes
that gap, and does it in a way that every backend can reuse, including the ones Tantu wants
later (a browser backend on WebGPU or canvas, a painter for hosting Tantu inside eframe/egui,
remote rendering).

`tantu-render-conformance` is the renderer test kit (ADR 0008). It holds:

- the reference Scenes, one per area of the Scene command set;
- their goldens, rendered by the software renderer and embedded in the crate, so a backend's
  tests need no file access (that matters for a wasm test runner);
- `match_images`, an edge-aware comparison that tolerates anti-aliasing differences but not real
  errors.

A backend proves it conforms with one test: record each reference Scene, render it, and check
the result against its golden. The Phase 1 milestone is that test for wgpu, plus the demo frame
rendered by both backends.

"The same" can't mean byte-equal. The backends anti-alias differently (tiny-skia's coverage
rasterizer vs. a signed-distance shader), blur differently (three box blurs vs. a closed-form
Gaussian) and GPUs round differently. So the comparison has two parts: away from edges the
images must agree closely, and on edges they may differ more, but only in a small fraction of
pixels and never by a lot. A systematic error (a shape in the wrong place, a wrong radius, a
wrong color, a half-pixel shift) fails; anti-aliasing noise doesn't.

## Scope

In scope:

- The `tantu-render-conformance` crate: reference Scenes, embedded goldens, `match_images`.
- Moving the five reference Scenes out of `tantu-render-soft/tests/goldens.rs` and the PNGs out
  of `tantu-render-soft/tests/goldens/` into the new crate. The software renderer's golden
  tests keep their strict check (`diff_images`, tolerance 2) and keep writing the goldens with
  `TANTU_UPDATE_GOLDENS=1`, now into the new crate.
- The milestone tests: wgpu against the goldens, and the `scene-window` demo frame rendered by
  both backends.
- Fixing the wgpu bugs the comparison finds (see "Measurements": one is known).

Out of scope:

- Text (no glyph rasterization in either backend until Phase 2). Reference Scenes for text are
  added with `tantu-text`.
- Custom commands (backend-specific by design).
- Window surfaces: the comparison uses offscreen targets only.
- Widget-level goldens: `tantu-test` (Phase 2) builds on `match_images` for those.
- Other backends: the browser and egui backends are not built here; this crate is what they
  will be tested with.

## Dependencies

`tantu-render-conformance` depends on `tantu-core`, `tantu-scene` and `png` 0.18 (already in the
workspace through `tantu-render-soft`) to decode the embedded goldens. It does **not** depend on
`tantu-render-soft` or any rasterizer, so a backend that uses it doesn't pull in tiny-skia. It
uses no `unsafe` (`#![forbid(unsafe_code)]`).

`tantu-render-soft`, `tantu-render-wgpu` and `scene-window` take it as a dev-dependency.

## Public API

Crate root `tantu_render_conformance`.

```rust
use std::path::PathBuf;
use tantu_scene::{ImageData, Resources, Scene};

/// One reference Scene and its golden image.
#[derive(Debug)]
pub struct ReferenceScene { /* private */ }

impl ReferenceScene {
    /// Its name, which is also the golden's file stem (`shapes_and_strokes`, …).
    pub fn name(&self) -> &'static str;
    /// The target size in physical pixels to render it at.
    pub fn target_size(&self) -> (u32, u32);
    /// The scale factor to render it at.
    pub fn scale_factor(&self) -> f32;
    /// Records the Scene, with the resources it draws (images are added to a new `Resources`).
    pub fn record(&self) -> (Scene, Resources);
    /// The golden PNG bytes, embedded in the crate.
    pub fn golden_png(&self) -> &'static [u8];
    /// The golden decoded as straight-alpha RGBA8.
    pub fn golden(&self) -> Result<ImageData, GoldenError>;
    /// Where the golden lives in the source tree, for writing it with `TANTU_UPDATE_GOLDENS=1`
    /// and for writing failed renderings next to it. Only meaningful in a checkout of the
    /// repository.
    pub fn golden_path(&self) -> PathBuf;
    /// `match_images(golden, actual, tolerance)`.
    pub fn check(&self, actual: &ImageData, tolerance: &MatchTolerance)
        -> Result<ImageMatch, GoldenError>;
}

/// All reference Scenes, in a fixed order.
pub fn reference_scenes() -> &'static [ReferenceScene];

/// The reference Scene called `name`, if there is one.
pub fn reference_scene(name: &str) -> Option<&'static ReferenceScene>;

/// Why a golden couldn't be used.
#[derive(Debug)]
#[non_exhaustive]
pub enum GoldenError {
    /// The embedded PNG doesn't decode (it was replaced by something that isn't a PNG).
    Decode(Box<dyn std::error::Error + Send + Sync>),
    /// The image checked has a different size from the golden.
    SizeMismatch { golden: (u32, u32), actual: (u32, u32) },
}
impl std::fmt::Display for GoldenError { /* ... */ }
impl std::error::Error for GoldenError { /* source() is the Decode error */ }

/// Tolerances for comparing a rendering with a reference produced by another backend.
///
/// Pixel values are compared premultiplied (see [`match_images`]). A pixel is an *edge pixel*
/// when the reference changes by more than `edge_threshold` somewhere in its 3 × 3
/// neighbourhood; every other pixel is an *interior pixel*.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatchTolerance {
    /// How much the reference must change between neighbours to make an edge.
    pub edge_threshold: u8,
    /// Largest difference allowed at an interior pixel.
    pub interior: u8,
    /// Largest difference allowed at an edge pixel.
    pub edge: u8,
    /// An edge pixel differing by more than this counts as a *loose* edge pixel.
    pub loose_edge: u8,
    /// Largest fraction (0..=1) of the edge pixels that may be loose.
    pub loose_edge_fraction: f32,
}

impl MatchTolerance {
    /// The tolerance for comparing a backend with the software goldens, from the measurements
    /// in this spec: edge threshold 8, interior 10, edge 96, loose edge 16, loose edge
    /// fraction 0.05.
    pub const CROSS_BACKEND: MatchTolerance;
}

impl Default for MatchTolerance {
    /// [`MatchTolerance::CROSS_BACKEND`].
    fn default() -> Self;
}

/// The result of [`match_images`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageMatch {
    /// Whether the images match within the tolerance.
    pub passed: bool,
    /// Number of edge pixels (taken from the reference).
    pub edge_pixels: u64,
    /// Largest difference at an interior pixel.
    pub max_interior_delta: u8,
    /// Interior pixels differing by more than `interior`.
    pub interior_failures: u64,
    /// Largest difference at an edge pixel.
    pub max_edge_delta: u8,
    /// Edge pixels differing by more than `edge`.
    pub edge_failures: u64,
    /// Edge pixels differing by more than `loose_edge`.
    pub loose_edge_pixels: u64,
    /// The most loose edge pixels allowed: `floor(loose_edge_fraction × edge_pixels)`.
    pub loose_edge_allowance: u64,
}

/// Compares `actual` with `reference` using `tolerance`. `None` if their sizes differ.
///
/// Edges are found in `reference` only, so the comparison is not symmetric: pass the trusted
/// image (a golden) as `reference`.
pub fn match_images(
    reference: &ImageData,
    actual: &ImageData,
    tolerance: &MatchTolerance,
) -> Option<ImageMatch>;
```

## Behavior

### The comparison

- **RENDER-CONF-01:** `match_images` returns `None` when the two images have different widths or
  heights, and never panics. (`ImageData` is at least 1 × 1; a 1 × 1 image has no neighbours,
  so it has no edge pixels.)
- **RENDER-CONF-02:** Pixels are compared premultiplied: each color channel becomes
  `round(c × a / 255)`. The difference of two pixels is the largest absolute difference over the
  premultiplied red, green and blue and the alpha. So two fully transparent pixels never
  differ, whatever their color channels hold, and an almost transparent pixel can only differ
  by about its alpha.
- **RENDER-CONF-03:** A pixel is an edge pixel when some pixel in its 3 × 3 neighbourhood in the
  reference (cut off at the image border) differs from it, as in RENDER-CONF-02, by more than
  `edge_threshold`. A sharp edge between two flat colors therefore gives a band two pixels wide
  (the last pixel on each side, plus any anti-aliased pixel between). Edges depend only on the
  reference: `edge_pixels` is the same for any `actual`.
- **RENDER-CONF-04:** The images match (`passed`) exactly when there are no interior failures, no
  edge failures, and `loose_edge_pixels ≤ loose_edge_allowance`. The counts and maxima are
  filled in whether it passes or not. A `loose_edge_fraction` that is NaN or negative counts as
  0, one above 1 as 1.
- **RENDER-CONF-05:** An image matches itself under any tolerance, with every count and maximum 0
  except `edge_pixels` and `loose_edge_allowance`.
- **RENDER-CONF-06:** `MatchTolerance::CROSS_BACKEND` and `MatchTolerance::default()` are
  `{ edge_threshold: 8, interior: 10, edge: 96, loose_edge: 16, loose_edge_fraction: 0.05 }`.
- **RENDER-CONF-07:** `CROSS_BACKEND` catches real differences. The `shapes_and_strokes` golden fails
  against a software rendering of that Scene with one shape changed in any of these ways: moved
  by 1 logical pixel, a corner radius changed by 50 %, a fill color changed by 12 levels in one
  channel, or a stroke width changed by 1 logical pixel. (This test lives in
  `tantu-render-soft`, which can render; the conformance crate can't.)

### The reference set

- **RENDER-CONF-08:** `reference_scenes()` holds exactly `shapes_and_strokes`, `shadows`, `images`,
  `clips_and_transforms`, `layers` and `text` (added in Phase 2, RENDER-CONF-14), in that order, with unique names, each at 200 × 200 and
  scale factor 2 (a 100 × 100 logical frame). `reference_scene(name)` finds each of them and
  returns `None` for any other name.
- **RENDER-CONF-09:** Each reference Scene records a frame of its logical size with balanced scopes,
  and `RenderReport::for_scene` on it and its resources is clean. Recording twice gives equal
  Scenes, except that image and font handles differ (each recording adds its images and fonts
  to a new `Resources`, and handles are never reused), with equal data behind them. Each golden decodes to an image of the scene's target size.
- **RENDER-CONF-10:** `check` returns `match_images(golden, actual, tolerance)`, or
  `Err(GoldenError::SizeMismatch)` with both sizes when they differ.
- **RENDER-CONF-11:** The software renderer reproduces every golden within tolerance 2 per channel
  with no differing pixels (`diff_images`), as before the move. With `TANTU_UPDATE_GOLDENS=1`
  it writes them to `golden_path()` instead. (The software renderer's golden tests, now taking
  the Scenes and goldens from this crate.)

- **RENDER-CONF-14:** The `text` reference Scene draws glyph runs in Liberation Sans (embedded
  in the crate with its SIL OFL 1.1 license; `record()` adds it to the returned `Resources`): an
  18 px "Tantu" and a 10 px "Ag 0.5px" with fixed glyph ids and positions (captured once from
  `tantu_text::TextSystem`, so the Scene doesn't depend on shaping), and the 10 px run again
  under a 1.5× scale, on a white background. Its golden is rendered by the software renderer
  like the others.

### The milestone

Like the other wgpu tests, these skip with a message when no GPU adapter is found, unless
`TANTU_REQUIRE_GPU=1` is set (Linux CI, with lavapipe). On a failure the test prints the
`ImageMatch` and the adapter, and writes the rendering next to the golden as
`<name>.wgpu.actual.png` (gitignored).

- **RENDER-CONF-12:** Each reference Scene, rendered by an offscreen `WgpuRenderer` at its target
  size and scale factor, gives a clean report and passes `check` with
  `MatchTolerance::CROSS_BACKEND`.
- **RENDER-CONF-13:** The `scene-window` demo frame at time 0.5, for a 900 × 600 target at scale 1
  and at scale 2, rendered by an offscreen `WgpuRenderer`, matches the software renderer's
  rendering of the same frame (the reference) with `MatchTolerance::CROSS_BACKEND`. There is no
  committed golden for it: both sides are rendered in the test, so the demo can change freely.

## Measurements

Taken on 2026-10-10 with a throwaway test (not committed), using the current renderers.
Differences are premultiplied as in RENDER-CONF-02, edges as in RENDER-CONF-03 with threshold 8.
Adapters: Intel Iris Xe on Vulkan, the same on GL, and llvmpipe (lavapipe, as in Linux CI). The
three agreed to within a few pixels; the worst of them is shown.

| Frame | Edge pixels | Max interior | Max edge | Edge pixels > 16 |
|---|---|---|---|---|
| shapes_and_strokes | 4 272 | **215** (bug, below) | 215 | 3.2 % |
| shadows | 2 725 | 4 | 15 | 0 % |
| images | 4 083 | 1 | 22 | 0.2 % |
| clips_and_transforms | 5 023 | 0 | 58 | 1.8 % |
| layers | 2 509 | 1 | 16 | 0 % |
| demo frame (scale 1 and 2) | 65 289 | 8 (llvmpipe; 3 on Intel) | 28 | 0.1 % |

The RENDER-CONF-06 values leave headroom over these (interior 10 over 8, edge 96 over 58, 5 %
over 1.8 %) for other GPUs, while staying far below what a real error produces (a 1-pixel shift
or a wrong radius makes most edge pixels along it differ by 100 or more).

Comparing straight-alpha values was tried first and rejected: near-transparent anti-aliased
pixels (alpha 6 vs. 0) then differ by 255 in their color channels.

**Bug found.** In `shapes_and_strokes`, the blue rect with radii `top_left 0, top_right 15,
bottom_right 4, bottom_left 30` on a 38 × 30 rect differs in 49 interior pixels. The software
renderer is right: by the CSS rule (RENDER-SOFT-10, restated by RENDER-WGPU-08) no scaling is
needed, since each side's two radii fit, so the 30-pixel bottom-left corner curves along the
whole left side. The wgpu shader picks a corner's radius by quadrant (which half of the rect the
point is in), so a radius larger than half a side is cut off at the middle and the upper half of
the left side comes out square. It is fixed through the bug-fix loop: a `test:` commit with a
failing RENDER-WGPU-08 probe for a radius over half a side, then a `fix:` commit.

## Performance and allocation

None beyond the general rules in AGENTS.md. `match_images` is test tooling: O(pixels), with one
allocation for the premultiplied reference so each pixel's neighbourhood isn't converted nine
times. Goldens are decoded on each `golden()` call (a few milliseconds for 200 × 200).

## Open questions

Resolved (2026-10-10; the user said "your pick", with the goal of an extensible framework that
will later render in a browser and inside eframe):

1. **Where the reference Scenes live.** In a new crate, `tantu-render-conformance`, rather than a
   test file shared by path between two crates or a public module of `tantu-render-soft`. Any
   backend, ours or a third party's, gets the whole kit from one dev-dependency without pulling
   in tiny-skia (ADR 0008).
2. **Goldens embedded** with `include_bytes!`, so the check works without the source tree (a
   published crate, a wasm test runner). Writing them still goes to the source tree.
3. **Tolerance values.** The RENDER-CONF-06 values, measured above. The Windows and macOS CI
   runners use adapters not measured here; if they fail by a small margin, the change goes
   through a `spec:` commit with the new measurement, not a silent edit.
4. **Demo frame.** Compared live (both backends in the test), no committed golden.
5. **Plan wording.** The milestone now reads "renders the same … within the cross-backend
   tolerance", because "identically" promises something no two rasterizers deliver.
