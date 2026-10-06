# Scene and scene builder

- **Status:** Agreed
- **Crate:** `tantu-scene`
- **Plan item:** Phase 1, "`tantu-scene` → `Scene` and its builder"
- **Related:** [ADR 0003](../../adr/0003-scene-as-the-renderer-contract.md) (Scene as the renderer
  contract), [geometry](../core/geometry.md), [color](../core/color.md), [arena](../core/arena.md)
  (`Id::to_bits` for element ids). The `Renderer` trait and the resource registry come in the
  next spec, `docs/specs/scene/renderer.md`.

## Purpose

A `Scene` is one frame of drawing, as data. Paint code (render objects in `tantu-view`, later
devtools) records into it through a `SceneBuilder`. Renderers (`tantu-render-wgpu`, `-soft`,
`-headless`) read it and draw. Nothing in a Scene points back into the UI: it holds plain values,
copies of glyph and custom-command data, and opaque handles to images and fonts.

Users are paint code and renderer authors. App developers only meet the Scene if they write a
custom render object or a custom command.

## Scope

In scope:

- `Scene`: a reusable, flat list of entries, plus side buffers for glyphs and custom-command
  bytes, the scene size and the damage region.
- `SceneBuilder`: records into a `Scene`, tracks the current element id, z-index, transform and
  clip, and finishes the frame (balances scopes, orders by z-index).
- The command set for Phase 1:
  - fills and strokes of rects and rounded rects
  - box shadows
  - images
  - glyph runs
  - custom commands
  - clip, transform and layer scopes (layer = group opacity + overlay color)
- Handle types referenced by commands: `ElementId`, `ImageId`, `FontId`, `CustomKind`.
- A conservative culling query, so paint code can skip off-screen subtrees.
- The drawing semantics every renderer must follow (normative here, tested by the renderer specs).

Out of scope (and where it goes):

- The `Renderer` trait, registering image and font data, custom-command handlers: next spec,
  `scene/renderer.md`.
- Gradients, arbitrary paths, blend modes other than normal, dashed strokes, inner shadows,
  elliptical corner radii, image tiling/9-slice. Later spec items that extend the command set;
  see open question 2.
- Serialization (serde). The types are plain data so it can be added later; see open question 4.
- Diffing two Scenes, layer caching, partial repaint: Phase 5. The Scene only carries damage.
- Hit-testing. It works on the render tree in `tantu-view`, not on the Scene.
- Shaping text. Glyph runs arrive already shaped and positioned (from `tantu-text`).

## Public API

Crate root `tantu_scene`. Geometry and color types come from `tantu_core`.

```rust
use tantu_core::{Affine, Color, Id, Point, Rect, Size, Vec2};

/// The element a command was painted for. Encodes an arena `Id` with `Id::to_bits`.
/// `Option<ElementId>` is 8 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(/* NonZeroU64 */);
impl ElementId {
    /// From a raw value; `None` for 0.
    pub const fn from_raw(raw: u64) -> Option<ElementId>;
    /// The raw value, never 0.
    pub const fn to_raw(self) -> u64;
}
impl From<Id> for ElementId { /* from Id::to_bits */ }

/// Handle to an image registered with the renderer's resources (see `scene/renderer.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageId(/* NonZeroU64 */);
/// Handle to a font face registered with the renderer's resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontId(/* NonZeroU64 */);
// Both: `from_raw(u64) -> Option<Self>` (None for 0) and `to_raw(self) -> u64`, as ElementId.

/// Identifies what a custom command draws, so a renderer can find its handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CustomKind(pub u32);

/// Per-corner circular radii, Flutter's `BorderRadius` (circular corners only).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderRadius { pub top_left: f32, pub top_right: f32, pub bottom_right: f32, pub bottom_left: f32 }
impl BorderRadius {
    /// All four corners zero.
    pub const ZERO: BorderRadius;
    /// The same radius on every corner.
    pub const fn circular(radius: f32) -> BorderRadius;
}

/// A rect with rounded corners. Zero radii make it a plain rect.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RoundedRect { pub rect: Rect, pub radii: BorderRadius }
impl RoundedRect {
    /// A plain rect (zero radii).
    pub const fn from_rect(rect: Rect) -> RoundedRect;
    /// A rect with radii.
    pub const fn new(rect: Rect, radii: BorderRadius) -> RoundedRect;
}

/// A clip shape, in the local coordinates in effect when it is pushed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Clip { Rect(Rect), RoundedRect(RoundedRect) }

/// A group of commands composited together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layer {
    /// Group opacity, nominally 0..=1. Default 1.
    pub opacity: f32,
    /// A color laid over the group's content (Clay's overlay color). Default `None`.
    pub overlay_color: Option<Color>,
}
impl Default for Layer { /* opacity 1.0, no overlay */ }

/// Flutter's `BoxShadow`, drawn outside-in around a rounded rect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShadow {
    /// The shape casting the shadow.
    pub shape: RoundedRect,
    pub color: Color,
    pub offset: Vec2,
    pub blur_radius: f32,
    pub spread_radius: f32,
}

/// How an image is sampled when scaled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageSampling { Nearest, #[default] Linear }

/// An image drawn into a rect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageDraw {
    pub image: ImageId,
    /// Part of the image to draw, in image pixels. `None` draws the whole image.
    pub src: Option<Rect>,
    /// Where to draw it, in local coordinates.
    pub dest: Rect,
    pub sampling: ImageSampling,
    /// Multiplies the image's alpha, nominally 0..=1.
    pub opacity: f32,
}

/// One positioned glyph, relative to its run's origin (baseline).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph { pub id: u32, pub x: f32, pub y: f32 }

/// A run of glyphs in one font, size and color. The glyphs live in the Scene's glyph buffer:
/// read them with `Scene::glyphs`.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphRun {
    pub font: FontId,
    /// Font size in logical pixels (em size).
    pub font_size: f32,
    pub color: Color,
    /// Baseline origin of the run, in local coordinates.
    pub origin: Point,
    /* private: range into the glyph buffer */
}

/// A command a renderer may or may not know how to draw. The bytes live in the Scene's data
/// buffer: read them with `Scene::custom_data`.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomDraw {
    pub kind: CustomKind,
    /// The area it draws into, in local coordinates. Used for culling and damage.
    pub bounds: Rect,
    /* private: range into the data buffer */
}

/// One recorded command.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    PushClip(Clip),
    PopClip,
    PushTransform(Affine),
    PopTransform,
    PushLayer(Layer),
    PopLayer,
    /// Fill a (rounded) rect.
    Fill { shape: RoundedRect, color: Color },
    /// Stroke a (rounded) rect, the stroke lying inside the shape.
    Stroke { shape: RoundedRect, width: f32, color: Color },
    BoxShadow(BoxShadow),
    Image(ImageDraw),
    GlyphRun(GlyphRun),
    Custom(CustomDraw),
}

/// A command plus the element it was painted for and its z-index.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub element: Option<ElementId>,
    pub z_index: i32,
    pub command: Command,
}

/// What changed since the previous frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Damage<'a> {
    /// Repaint everything.
    Full,
    /// Repaint only these rects (scene coordinates).
    Rects(&'a [Rect]),
}

/// Problems found by `SceneBuilder::finish`. The Scene is well-formed even when this is returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneError {
    /// `pop` calls with no open scope (they recorded nothing).
    pub unmatched_pops: u32,
    /// Scopes still open at `finish` (closed automatically).
    pub unclosed_scopes: u32,
}
impl std::fmt::Display for SceneError { /* ... */ }
impl std::error::Error for SceneError {}

/// One frame of drawing, as data. Reuse it across frames: `begin` keeps its buffers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene { /* private */ }

impl Scene {
    /// The version of the command set. Bumped when a command's meaning changes.
    pub const FORMAT_VERSION: u32 = 1;

    /// An empty Scene. Does not allocate.
    pub const fn new() -> Scene;
    /// Clears the Scene, keeping its buffers, and starts recording a frame of `size`
    /// logical pixels.
    pub fn begin(&mut self, size: Size) -> SceneBuilder<'_>;

    /// Size of the frame in logical pixels.
    pub fn size(&self) -> Size;
    /// The recorded entries, in paint order (later entries draw on top).
    pub fn entries(&self) -> &[Entry];
    /// The glyphs of `run`. Empty if `run` didn't come from this Scene's current frame.
    pub fn glyphs(&self, run: &GlyphRun) -> &[Glyph];
    /// The bytes of `custom`. Empty if `custom` didn't come from this Scene's current frame.
    pub fn custom_data(&self, custom: &CustomDraw) -> &[u8];
    /// The damage region.
    pub fn damage(&self) -> Damage<'_>;
}

/// Records one frame into a `Scene`. Dropping it without `finish` leaves the Scene empty.
pub struct SceneBuilder<'a> { /* private */ }

impl SceneBuilder<'_> {
    /// Element id for the following commands. Default `None`.
    pub fn set_element(&mut self, element: Option<ElementId>);
    /// Z-index for the following commands, within the current scope. Default 0.
    pub fn set_z_index(&mut self, z_index: i32);

    /// Clip the following commands to `clip` (intersected with any outer clip).
    pub fn push_clip(&mut self, clip: Clip);
    /// Apply `transform` to the following commands, after the current transform.
    pub fn push_transform(&mut self, transform: Affine);
    /// Composite the following commands as one group.
    pub fn push_layer(&mut self, layer: Layer);
    /// Close the innermost open scope.
    pub fn pop(&mut self);

    /// Fill a rect.
    pub fn fill_rect(&mut self, rect: Rect, color: Color);
    /// Fill a rounded rect.
    pub fn fill(&mut self, shape: RoundedRect, color: Color);
    /// Stroke a rounded rect, inside its edge.
    pub fn stroke(&mut self, shape: RoundedRect, width: f32, color: Color);
    /// Draw a box shadow.
    pub fn box_shadow(&mut self, shadow: BoxShadow);
    /// Draw an image.
    pub fn image(&mut self, image: ImageDraw);
    /// Draw a run of glyphs; `glyphs` is copied into the Scene.
    pub fn glyph_run(&mut self, font: FontId, font_size: f32, color: Color, origin: Point, glyphs: &[Glyph]);
    /// Record a custom command; `data` is copied into the Scene.
    pub fn custom(&mut self, kind: CustomKind, bounds: Rect, data: &[u8]);

    /// True if nothing inside `local_bounds` (current local coordinates) can be visible.
    pub fn is_culled(&self, local_bounds: Rect) -> bool;

    /// Add a damaged rect, in scene coordinates.
    pub fn add_damage(&mut self, rect: Rect);
    /// Mark the whole frame damaged.
    pub fn damage_all(&mut self);

    /// Close open scopes, order entries by z-index, and end the frame.
    pub fn finish(self) -> Result<(), SceneError>;
}
```

## Behavior

Recording

- **SCENE-SCENE-01:** `Scene::new()` and `Scene::default()` are equal: size zero, no entries,
  damage `Full`.
- **SCENE-SCENE-02:** `begin(size)` discards everything from the previous frame (entries, glyphs,
  custom data, damage) and sets the size. Damage starts as `Full`.
- **SCENE-SCENE-03:** If the builder is dropped without `finish`, the Scene has no entries (as
  right after `begin`).
- **SCENE-SCENE-04:** Entries appear in the order they were recorded, unless z-indexes differ
  (SCENE-SCENE-12). Later entries are painted on top of earlier ones.
- **SCENE-SCENE-05:** Each entry carries the element id and z-index in effect when it was recorded.
  Both are sticky: they stay until changed. `begin` resets them to `None` and 0.
- **SCENE-SCENE-06:** `pop` restores the element id and z-index that were in effect when the
  matching scope was pushed, so a child's settings never leak into its parent's later commands.
- **SCENE-SCENE-07:** Every recording method stores its values as given: nothing is clamped,
  normalized or dropped (transparent colors, empty or reversed rects, NaN included). No method
  panics, whatever the input.
- **SCENE-SCENE-08:** `fill_rect(rect, color)` records `Fill` with `RoundedRect::from_rect(rect)`.
  `fill`, `stroke`, `box_shadow` and `image` record the matching command with the given values.

Glyph runs and custom data

- **SCENE-SCENE-09:** `glyph_run` copies `glyphs` into the Scene; `Scene::glyphs(&run)` returns
  them in the same order. An empty slice records a run with no glyphs.
- **SCENE-SCENE-10:** `custom` copies `data` into the Scene; `Scene::custom_data(&c)` returns the
  same bytes.
- **SCENE-SCENE-11:** `glyphs` and `custom_data` return an empty slice, never panic, for a run or
  custom command that doesn't index into this Scene's current buffers (from another Scene, or
  from an earlier frame after the buffers shrank).

Z-index and scopes

- **SCENE-SCENE-12:** `finish` stably sorts the entries of each scope by z-index, ascending: lower
  z is painted first, equal z keeps recording order. The root of the frame is a scope. A nested
  scope (its push entry, its contents and its pop entry) moves as one unit with the z-index of its
  push entry. Entries never move into or out of a scope. Negative z-indexes are allowed.
- **SCENE-SCENE-13:** `push_clip`, `push_transform` and `push_layer` record `PushClip`,
  `PushTransform` and `PushLayer`. `pop` records the matching `PopClip`, `PopTransform` or
  `PopLayer` for the innermost open scope, with the element id and z-index of its push entry.
- **SCENE-SCENE-14:** `pop` with no open scope records nothing and counts as an unmatched pop.
- **SCENE-SCENE-15:** `finish` closes scopes still open, innermost first, as `pop` would.
- **SCENE-SCENE-16:** `finish` returns `Ok(())` if there were no unmatched pops and no unclosed
  scopes, and otherwise `Err(SceneError)` with both counts. Either way the Scene is well-formed:
  every push entry has exactly one matching pop entry of the same kind, properly nested.

Culling

- **SCENE-SCENE-17:** `is_culled(local_bounds)` maps `local_bounds` through the current transform
  (the bounding box of the transformed rect), and returns true if that box does not overlap
  (`Rect::overlaps`) the visible area: the scene rect `(0, 0, size)` intersected with the bounding
  boxes of all open clips, each mapped through the transform in effect when it was pushed.
- **SCENE-SCENE-18:** `is_culled` is conservative: it may return false for content that ends up
  invisible (rotated content, rounded-clip corners, zero opacity), but it never returns true for
  content that could be visible. Empty or non-finite `local_bounds` are culled. A non-finite
  transform or clip makes everything under it culled.
- **SCENE-SCENE-19:** `is_culled` records nothing. Recording methods never cull: paint code decides.

Damage

- **SCENE-SCENE-20:** With no damage calls, `damage()` is `Full`. After `add_damage` calls (and no
  `damage_all`), it is `Rects` with the rects in call order, stored as given. `damage_all` makes it
  `Full`, and later `add_damage` calls in the same frame leave it `Full`.

Handles and values

- **SCENE-SCENE-21:** `ElementId::from(id)` has `to_raw() == id.to_bits()`. `from_raw(0)` is `None`
  and `from_raw(x).unwrap().to_raw() == x` for any other `x`. The same holds for `ImageId` and
  `FontId`. `size_of::<Option<ElementId>>()` is 8.
- **SCENE-SCENE-22:** `Layer::default()` has opacity 1 and no overlay color.
  `BorderRadius::circular(r)` sets all four corners to `r`.
- **SCENE-SCENE-23:** Two Scenes recorded with the same calls compare equal; `Scene`, `Entry` and
  every command type are `Clone + Debug + PartialEq + Send + Sync`, and hold no references, closures
  or `Rc`/`Arc`.

## Drawing semantics (contract for renderers)

These say what a correct renderer draws. They have no rule ids here because they can only be
checked with pixels. `tantu-render-soft` and `tantu-render-wgpu` turn them into numbered rules
and golden tests in their own specs.

1. **Coordinates.** Logical pixels, origin at the top-left of the frame, y down. Renderers
   multiply by the target's scale factor. Commands are in the local coordinates set by the open
   transforms (outermost first), clips in the coordinates in effect when they were pushed.
2. **Order.** Entries are painted in `entries()` order with source-over compositing.
3. **Color math.** Colors are sRGB-encoded with straight alpha; blending happens on premultiplied
   sRGB-encoded values (the way browsers and Flutter blend), not in linear light. Edges are
   anti-aliased.
4. **Rounded rects.** If adjacent radii add up to more than the side between them, all radii are
   scaled down by the same factor until they fit (the CSS rule). Negative radii count as 0.
5. **Stroke.** Lies inside the shape: outer edge on the shape, inner edge `width` in. A width of
   half the shorter side or more fills the shape.
6. **Box shadow.** Flutter's `BoxShadow`: the shape is inflated by `spread_radius`, moved by
   `offset`, and blurred with a Gaussian of sigma `blur_radius · 0.57735 + 0.5` (0 when
   `blur_radius` is 0). The whole shadow is drawn, including the part under the shape; paint
   code draws the shape on top.
7. **Image.** `src` (or the whole image) is scaled to fill `dest`, with no aspect-ratio
   correction. Alpha is multiplied by `opacity` clamped to 0..=1. A handle unknown to the renderer
   draws nothing.
8. **Glyph run.** Each glyph is drawn at `origin + (x, y)`, in `color`, from the font's outlines
   or bitmaps at `font_size`. An unknown font or glyph draws nothing.
9. **Clip.** Content is limited to the intersection of all open clips, anti-aliased.
10. **Layer.** Its content is drawn into an isolated group, then composited with alpha multiplied
    by `opacity` clamped to 0..=1 (NaN counts as 0). If `overlay_color` is set, every pixel of the
    group is mixed toward the overlay's rgb by the overlay's alpha, keeping the pixel's alpha
    (source-atop), before the opacity is applied.
11. **Custom.** A renderer with a handler for the kind lets it draw in `bounds`; without one,
    nothing is drawn (reporting is defined in `scene/renderer.md`).
12. **Invalid values.** A draw command with a non-finite coordinate, size, width or color
    component draws nothing; a non-finite transform or clip hides everything inside its scope.
    Empty or reversed rects draw nothing. None of this is an error.

## Performance and allocation

- **SCENE-SCENE-24:** After a warm-up frame, recording and finishing a frame with no more entries,
  glyphs, custom bytes, damage rects or scope depth than an earlier frame does not allocate.
- `finish` is O(n) when no scope has entries with different z-indexes, O(n log n) otherwise. The
  sort's scratch space is kept in the Scene.
- `size_of::<Entry>()` should stay at or below 64 bytes; checked by a test, but not a promise to
  users.

## Open questions

Resolved (2026-10-06, the proposals were accepted):

1. **Z-index scoping.** Z-index orders siblings within a scope (like CSS stacking contexts), so a
   high z-index can't escape a clip or layer. Popups that must escape go through `Overlay` at the
   root, as in Flutter. One global sort was rejected: it breaks clips and group opacity.
2. **Command set for Phase 1.** Only what the Phase 1 wgpu item needs, plus glyph runs and custom
   commands. Gradients, paths (likely via `kurbo`), blend modes and inner shadows come as later
   additions to this spec, each bumping `FORMAT_VERSION` only if an existing command's meaning
   changes.
3. **One `Fill` for rects and rounded rects.** One command with `RoundedRect`, zero radii meaning a
   plain rect, so backends implement one shape and fast-path zero radii.
4. **Serialization.** No serde now. An optional `serde` feature comes when remote rendering or
   Scene snapshots in tests need it. `FORMAT_VERSION` exists from the start.
5. **Unbalanced scopes.** `finish` auto-closes and returns `Err` (always renderable, bug still
   visible), rather than a `debug_assert!`.

Deferred:

6. **Glyph run shape.** As above (font handle, size, color, origin, glyph id + offset). It may gain
   fields (synthetic bold/italic, variable-font coordinates, hinting) with the `tantu-text` spec
   in Phase 2.
7. **Layer bounds.** A layer has no bounds; renderers can compute them from the layer's commands.
   An optional bounds hint can be added later if profiling shows a need.
