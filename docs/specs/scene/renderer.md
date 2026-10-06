# Renderer trait and resources

- **Status:** Implemented
- **Crate:** `tantu-scene`
- **Plan item:** Phase 1, "`tantu-scene` → `Renderer` trait, image/font resource registry,
  custom-command handlers"
- **Related:** [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md) (Scene as the renderer
  contract), [scene](scene.md) (the `Scene`, handle types and drawing semantics)

## Purpose

A `Scene` refers to images and fonts by handle (`ImageId`, `FontId`) and to user-drawn content by
`CustomKind`. This spec defines where the data behind those handles lives (`Resources`), and the
`Renderer` trait every backend (`tantu-render-wgpu`, `-soft`, `-headless`) implements to draw a
Scene with those resources into its target.

Users: the app runner and `tantu-text` register images and fonts; renderer authors implement
`Renderer`; the app runner calls it once per frame per window.

## Scope

In scope:

- `ImageData` (RGBA8 pixels) and `FontData` (font file bytes + face index), validated on
  construction, cheap to clone (shared buffers).
- `Resources`: registers image and font data, issues handles, looks them up, removes them, and
  exposes a revision counter so renderers can keep their caches (GPU textures, glyph atlases) in
  sync.
- The `Renderer` trait: resize the target, render a Scene, report what couldn't be drawn.
- `RenderReport` and `RenderError`.
- The contract every renderer follows (normative here, tested by each renderer's spec).

Out of scope (and where it goes):

- Registering custom-command handlers. A handler draws with the backend's own API (a wgpu
  render pass, a tiny-skia pixmap), so each backend crate defines its own handler trait and
  registration method. This spec only fixes what happens when there is no handler.
- Parsing fonts, shaping, glyph rasterization and glyph caches: `tantu-text` and the renderers.
- Decoding image files (PNG, JPEG): a later loader (Phase 3, `Image` widget), which produces
  `ImageData`.
- Other pixel formats (premultiplied, 16-bit, compressed, GPU-resident textures), updating an
  image's pixels in place, mipmaps. See open question 3.
- Creating windows and surfaces: `tantu-platform-*`. A renderer is created by its backend crate,
  already bound to its target.
- Partial repaint using damage: Phase 5.

## Public API

Crate root `tantu_scene`, next to the Scene types.

```rust
use std::sync::Arc;

/// RGBA8 pixels, sRGB-encoded, straight (not premultiplied) alpha, rows top to bottom, no
/// padding. Cloning shares the pixel buffer.
#[derive(Clone, PartialEq, Eq)]
pub struct ImageData { /* width, height, pixels: Arc<[u8]> */ }

impl ImageData {
    /// Checks the size and the buffer length (`width · height · 4`).
    pub fn rgba8(width: u32, height: u32, pixels: impl Into<Arc<[u8]>>) -> Result<ImageData, ResourceError>;
    /// Width in pixels, ≥ 1.
    pub fn width(&self) -> u32;
    /// Height in pixels, ≥ 1.
    pub fn height(&self) -> u32;
    /// The pixels, `width · height · 4` bytes.
    pub fn pixels(&self) -> &[u8];
}
impl std::fmt::Debug for ImageData { /* size only, not the pixels */ }

/// A font file (TTF/OTF, or a collection) and the index of the face to use. Cloning shares the
/// bytes. The bytes are not parsed here.
#[derive(Clone, PartialEq, Eq)]
pub struct FontData { /* bytes: Arc<[u8]>, index: u32 */ }

impl FontData {
    /// Fails only for empty `bytes`.
    pub fn new(bytes: impl Into<Arc<[u8]>>, index: u32) -> Result<FontData, ResourceError>;
    /// The font file.
    pub fn bytes(&self) -> &[u8];
    /// Face index within a collection; 0 for a single-face file.
    pub fn index(&self) -> u32;
}
impl std::fmt::Debug for FontData { /* length and index, not the bytes */ }

/// Why image or font data was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceError {
    /// Width or height is 0, or `width · height · 4` overflows `usize`.
    InvalidImageSize { width: u32, height: u32 },
    /// The pixel buffer is not `width · height · 4` bytes.
    PixelDataLength { expected: usize, actual: usize },
    /// The font file is empty.
    EmptyFontData,
}
impl std::fmt::Display for ResourceError { /* ... */ }
impl std::error::Error for ResourceError {}

/// The images and fonts a Scene's handles refer to. One per app, shared by its windows'
/// renderers (wrap it in `Arc`/`RwLock` as the app runner needs).
#[derive(Clone, Debug, Default)]
pub struct Resources { /* private */ }

impl Resources {
    /// No resources, revision 0.
    pub fn new() -> Resources;

    /// Stores an image and returns its handle.
    pub fn add_image(&mut self, image: ImageData) -> ImageId;
    /// The image for `id`, or `None` if it was removed or never added here.
    pub fn image(&self, id: ImageId) -> Option<&ImageData>;
    /// Removes and returns the image for `id`.
    pub fn remove_image(&mut self, id: ImageId) -> Option<ImageData>;

    /// Stores a font face and returns its handle.
    pub fn add_font(&mut self, font: FontData) -> FontId;
    /// The font for `id`, or `None` if it was removed or never added here.
    pub fn font(&self, id: FontId) -> Option<&FontData>;
    /// Removes and returns the font for `id`.
    pub fn remove_font(&mut self, id: FontId) -> Option<FontData>;

    /// Increases whenever an image or font is added or removed. Renderers compare it with the
    /// value they last saw to know when to re-check their caches.
    pub fn revision(&self) -> u64;
}

/// What a renderer couldn't draw in one frame. All zero means everything was drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderReport {
    /// `Image` commands whose handle isn't in the resources (or that the backend couldn't upload).
    pub missing_images: u32,
    /// `GlyphRun` commands whose font isn't in the resources (or that the backend couldn't load).
    pub missing_fonts: u32,
    /// `Custom` commands with no handler for their kind.
    pub unhandled_custom: u32,
    /// Draw commands skipped because of non-finite values (scene.md, drawing semantics 12).
    pub invalid_commands: u32,
}

impl RenderReport {
    /// True if every count is 0.
    pub fn is_clean(&self) -> bool;
    /// The report every renderer gives for `scene` with `resources`, before any backend-specific
    /// failures (an image that couldn't be uploaded, a font that couldn't be parsed), which the
    /// backend adds on top. `handles_custom` says whether the renderer has a handler for a kind.
    pub fn for_scene(scene: &Scene, resources: &Resources, handles_custom: &dyn Fn(CustomKind) -> bool) -> RenderReport;
}

/// Why a frame couldn't be rendered at all.
#[derive(Debug)]
#[non_exhaustive]
pub enum RenderError {
    /// The target (e.g. a window surface) was lost or is outdated. Call `resize` and try again.
    TargetLost,
    /// The backend ran out of memory.
    OutOfMemory,
    /// Any other backend failure.
    Backend(Box<dyn std::error::Error + Send + Sync>),
}
impl std::fmt::Display for RenderError { /* ... */ }
impl std::error::Error for RenderError { /* source() is the Backend error */ }

/// Draws Scenes into a target. Implemented by each backend; object-safe.
pub trait Renderer {
    /// Sets the target size in physical pixels and the scale from logical to physical pixels.
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32);
    /// Draws `scene`, looking up its handles in `resources`.
    fn render(&mut self, scene: &Scene, resources: &Resources) -> Result<RenderReport, RenderError>;
}
```

## Behavior

Image and font data

- **SCENE-RES-01:** `ImageData::rgba8(w, h, pixels)` with `w ≥ 1`, `h ≥ 1` and exactly `w · h · 4`
  bytes succeeds; `width`, `height` and `pixels` return the values given.
- **SCENE-RES-02:** A width or height of 0, or a `w · h · 4` that overflows `usize`, gives
  `Err(InvalidImageSize { width, height })`. This is checked before the buffer length.
- **SCENE-RES-03:** A buffer of any other length gives `Err(PixelDataLength { expected, actual })`.
- **SCENE-RES-04:** `FontData::new(bytes, index)` fails with `EmptyFontData` for empty bytes and
  otherwise succeeds, returning the bytes and index as given. The bytes are not parsed and the
  index is not checked.
- **SCENE-RES-05:** Cloning `ImageData` or `FontData` shares the buffer instead of copying it, and
  their `Debug` output does not contain the pixels or font bytes.

Resources

- **SCENE-RES-06:** `add_image` and `add_font` return a handle that differs from every other
  handle issued in the process, by any `Resources`, including handles already removed. Handles
  are never reused.
- **SCENE-RES-07:** `image(id)` and `font(id)` return the data added with that handle until it is
  removed, then `None`. A handle issued by another `Resources` returns `None`.
- **SCENE-RES-08:** `remove_image` and `remove_font` return the removed data, or `None` (changing
  nothing) for a handle that is not present.
- **SCENE-RES-09:** `revision()` is 0 for a new `Resources`, increases by exactly 1 for each add
  and each remove that removed something, and is unchanged by lookups and by removes that
  return `None`.
- **SCENE-RES-10:** A clone of `Resources` has the same contents and revision, and changes to one
  don't affect the other. `Resources`, `ImageData` and `FontData` are `Send + Sync`.

Renderer types

- **SCENE-RENDER-01:** `RenderReport::default()` has every count 0 and `is_clean()` true;
  `is_clean()` is false if any count is non-zero.
- **SCENE-RENDER-02:** `Renderer` is object-safe: a `Box<dyn Renderer>` can be resized and render
  a Scene.
- **SCENE-RENDER-03:** `RenderError` and `ResourceError` have non-empty `Display` messages;
  `RenderError::Backend(e)` returns `e` from `source()`.

Shared report counting (`RenderReport::for_scene`)

- **SCENE-RENDER-04:** Entries are examined in order. A `PushTransform` with a non-finite
  coefficient, or a `PushClip` with a non-finite edge or radius, adds 1 to `invalid_commands`, and
  every entry inside that scope (up to its matching pop) is skipped without being examined.
  `PushLayer` and pop entries are never counted.
- **SCENE-RENDER-05:** A draw command (`Fill`, `Stroke`, `BoxShadow`, `Image`, `GlyphRun`,
  `Custom`) with any non-finite `f32` field adds 1 to `invalid_commands` and is not examined
  further. The fields are: every rect edge and radius; colors; stroke width; shadow offset, blur
  and spread; image `src`, `dest` and `opacity`; glyph run `font_size` and `origin`, and the `x`
  and `y` of each of its glyphs (`Scene::glyphs`); custom `bounds`.
- **SCENE-RENDER-06:** Otherwise an `Image` whose handle `resources.image` doesn't find adds 1 to
  `missing_images`, a `GlyphRun` whose font `resources.font` doesn't find adds 1 to
  `missing_fonts`, and a `Custom` command for which `handles_custom(kind)` is false adds 1 to
  `unhandled_custom`.
- **SCENE-RENDER-07:** Nothing else is counted: empty or reversed rects, transparent colors,
  negative widths or radii, zero or NaN layer opacity, and a glyph run with no glyphs are all
  valid (they draw nothing or are clamped, per the drawing semantics).
- **SCENE-RENDER-08:** Counts saturate at `u32::MAX`. `for_scene` never panics and does not
  allocate.

## Renderer contract

What every `Renderer` implementation must do. No rule ids here: each renderer's spec turns these
into numbered rules and tests (headless and soft in Phase 1, wgpu with the golden milestone).

1. **Target and scale.** `resize` sets the target to `width × height` physical pixels. Scene
   coordinates are multiplied by `scale_factor`; a non-finite or non-positive scale factor is
   treated as 1. Before the first `resize` the target size is whatever the backend crate's
   constructor set.
2. **Each frame starts clear.** `render` clears the whole target to transparent, then draws the
   Scene's entries in order following the drawing semantics in [scene.md](scene.md). Scene
   content outside the target is clipped; target area outside the Scene stays transparent.
3. **Zero-size targets.** With a 0 width or height, `render` draws nothing and returns
   `Ok(RenderReport::default())`.
4. **Missing resources are not errors.** An `Image` or `GlyphRun` whose handle isn't in
   `resources`, a `Custom` command with no handler, and a draw command with non-finite values
   each draw nothing and are counted in the `RenderReport`. The frame continues. Only failures of
   the target or the backend itself return `Err`. The report is
   `RenderReport::for_scene(scene, resources, handles_custom)` (SCENE-RENDER-04..08), plus 1 for
   each image or font the backend itself couldn't use, so every backend reports the same counts
   for the same input.
5. **No panics.** `render` never panics, for any Scene and any `Resources`.
6. **Caches follow the revision.** A renderer may cache uploaded images and loaded fonts by
   handle. When `resources.revision()` differs from the last one it saw, it drops cache entries
   whose handle is no longer present. Since handles are never reused, a cached entry for a
   present handle is always current.
7. **Pure function of its inputs.** Rendering never calls back into the UI. The same Scene,
   resources, target size, scale factor and custom handlers give the same pixels on the same
   backend.
8. **Damage.** Phase 1 renderers may ignore `Scene::damage` and repaint everything (rule 2).
   Using it is a later, opt-in optimization for targets that keep the previous frame's pixels.
9. **Custom handlers.** Each backend crate defines how handlers are registered, keyed by
   `CustomKind`. A handler gets the command's bounds, its bytes (`Scene::custom_data`) and the
   current transform and clip, and draws only inside the bounds.

## Performance and allocation

- `ImageData` and `FontData` clones are reference-count increments, not copies (SCENE-RES-05).
- `Resources` lookups are O(1) on average. `revision()` is O(1).
- `render` must not allocate per frame once warm, beyond what the backend API itself does
  (each renderer's spec states its own budget).

## Open questions

Resolved (2026-10-06, the proposals were accepted):

1. **Process-wide handle uniqueness (SCENE-RES-06).** Handles come from one atomic counter in the
   process, so a handle from one `Resources` can never alias data in another. It is the only
   process-wide state in the crate and holds no data. A counter per `Resources` was rejected: a
   stray handle would silently draw the wrong image.
2. **Where handlers are registered (contract item 9).** Per backend, because a handler needs the
   backend's drawing API. A backend-neutral fallback may be added later.
3. **Pixel formats and updates.** Straight-alpha RGBA8 only, and no in-place update: replace an
   image by adding a new one and removing the old. Revisit for video, canvases and large
   thumbnails (an `update_image` with a per-image version).
4. **Synchronization.** `Resources` uses `&mut self` for changes; the app runner decides how to
   share it (`Arc<RwLock<Resources>>` across windows). No internal locking.
5. **Report detail.** Counts only, so `render` doesn't allocate. Backends log details with
   `tracing`, once per handle or kind, not every frame.

Resolved with the shared-counting amendment (2026-10-06, the proposal was accepted):

6. **Shared counting.** `RenderReport::for_scene` in `tantu-scene`, used by every renderer, so headless, soft and wgpu report the same counts for the same Scene (and headless
   test results predict real backends). A non-finite layer overlay color is not counted (the
   overlay is ignored); the soft renderer spec will say so in its drawing rules.
