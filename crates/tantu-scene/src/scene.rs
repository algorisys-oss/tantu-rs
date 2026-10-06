//! [`Scene`], the per-frame display list, and [`SceneBuilder`], which records into it.

use std::fmt;

use tantu_core::{Affine, Color, Point, Rect, Size};

use crate::command::CustomDraw;
use crate::command::{
    BoxShadow, Clip, Command, Entry, Glyph, GlyphRun, ImageDraw, Layer, RoundedRect,
};
use crate::handles::{CustomKind, ElementId, FontId};

/// What changed since the previous frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Damage<'a> {
    /// Repaint everything.
    Full,
    /// Repaint only these rects (scene coordinates).
    Rects(&'a [Rect]),
}

/// Problems found by [`SceneBuilder::finish`]. The Scene is well-formed even when this is
/// returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneError {
    /// `pop` calls with no open scope (they recorded nothing).
    pub unmatched_pops: u32,
    /// Scopes still open at `finish` (closed automatically).
    pub unclosed_scopes: u32,
}

impl fmt::Display for SceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for SceneError {}

/// How the frame's damage was described.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DamageState {
    /// No damage calls: the whole frame.
    Unset,
    /// `add_damage` calls: the rects in `Scene::damage_rects`.
    Rects,
    /// `damage_all`: the whole frame, whatever comes later.
    All,
}

/// The kind of an open scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScopeKind {
    Clip,
    Transform,
    Layer,
}

/// Builder state saved when a scope is pushed and restored when it is popped.
#[derive(Clone, Copy, Debug)]
struct Frame {
    kind: ScopeKind,
    element: Option<ElementId>,
    z_index: i32,
    transform: Affine,
    visible: Option<Rect>,
}

/// One unit to order within a scope: a single entry, or a whole nested scope.
#[derive(Clone, Copy, Debug)]
struct Unit {
    z_index: i32,
    start: u32,
    end: u32,
}

/// Buffers kept between frames so recording doesn't allocate once warm.
#[derive(Clone, Debug, Default)]
struct Scratch {
    stack: Vec<Frame>,
    units: Vec<Unit>,
    sorted: Vec<Entry>,
}

/// One frame of drawing, as data. Reuse it across frames: [`Scene::begin`] keeps its buffers.
#[derive(Clone)]
pub struct Scene {
    size: Size,
    entries: Vec<Entry>,
    glyphs: Vec<Glyph>,
    data: Vec<u8>,
    damage: DamageState,
    damage_rects: Vec<Rect>,
    scratch: Scratch,
}

impl Scene {
    /// The version of the command set. Bumped when a command's meaning changes.
    pub const FORMAT_VERSION: u32 = 1;

    /// An empty Scene. Does not allocate.
    pub const fn new() -> Scene {
        todo!()
    }

    /// Clears the Scene, keeping its buffers, and starts recording a frame of `size` logical
    /// pixels.
    pub fn begin(&mut self, size: Size) -> SceneBuilder<'_> {
        todo!()
    }

    /// Size of the frame in logical pixels.
    pub fn size(&self) -> Size {
        todo!()
    }

    /// The recorded entries, in paint order (later entries draw on top).
    pub fn entries(&self) -> &[Entry] {
        todo!()
    }

    /// The glyphs of `run`. Empty if `run` didn't come from this Scene's current frame.
    pub fn glyphs(&self, run: &GlyphRun) -> &[Glyph] {
        todo!()
    }

    /// The bytes of `custom`. Empty if `custom` didn't come from this Scene's current frame.
    pub fn custom_data(&self, custom: &CustomDraw) -> &[u8] {
        todo!()
    }

    /// The damage region.
    pub fn damage(&self) -> Damage<'_> {
        todo!()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Scene::new()
    }
}

impl PartialEq for Scene {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl fmt::Debug for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scene")
            .field("size", &self.size)
            .field("entries", &self.entries)
            .field("glyphs", &self.glyphs)
            .field("data", &self.data)
            .field("damage", &self.damage())
            .finish()
    }
}

/// Records one frame into a [`Scene`]. Dropping it without [`finish`](SceneBuilder::finish)
/// leaves the Scene empty.
pub struct SceneBuilder<'a> {
    scene: &'a mut Scene,
    element: Option<ElementId>,
    z_index: i32,
    transform: Affine,
    visible: Option<Rect>,
    unmatched_pops: u32,
    finished: bool,
}

impl SceneBuilder<'_> {
    /// Element id for the following commands. Default `None`.
    pub fn set_element(&mut self, element: Option<ElementId>) {
        todo!()
    }

    /// Z-index for the following commands, within the current scope. Default 0.
    pub fn set_z_index(&mut self, z_index: i32) {
        todo!()
    }

    /// Clip the following commands to `clip` (intersected with any outer clip).
    pub fn push_clip(&mut self, clip: Clip) {
        todo!()
    }

    /// Apply `transform` to the following commands, after the current transform.
    pub fn push_transform(&mut self, transform: Affine) {
        todo!()
    }

    /// Composite the following commands as one group.
    pub fn push_layer(&mut self, layer: Layer) {
        todo!()
    }

    /// Close the innermost open scope. With no open scope, records nothing and is reported by
    /// [`finish`](SceneBuilder::finish).
    pub fn pop(&mut self) {
        todo!()
    }

    /// Fill a rect.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        todo!()
    }

    /// Fill a rounded rect.
    pub fn fill(&mut self, shape: RoundedRect, color: Color) {
        todo!()
    }

    /// Stroke a rounded rect, inside its edge.
    pub fn stroke(&mut self, shape: RoundedRect, width: f32, color: Color) {
        todo!()
    }

    /// Draw a box shadow.
    pub fn box_shadow(&mut self, shadow: BoxShadow) {
        todo!()
    }

    /// Draw an image.
    pub fn image(&mut self, image: ImageDraw) {
        todo!()
    }

    /// Draw a run of glyphs; `glyphs` is copied into the Scene.
    pub fn glyph_run(
        &mut self,
        font: FontId,
        font_size: f32,
        color: Color,
        origin: Point,
        glyphs: &[Glyph],
    ) {
        todo!()
    }

    /// Record a custom command; `data` is copied into the Scene.
    pub fn custom(&mut self, kind: CustomKind, bounds: Rect, data: &[u8]) {
        todo!()
    }

    /// True if nothing inside `local_bounds` (current local coordinates) can be visible.
    ///
    /// Conservative: it may return false for content that ends up invisible, never true for
    /// content that could be visible.
    pub fn is_culled(&self, local_bounds: Rect) -> bool {
        todo!()
    }

    /// Add a damaged rect, in scene coordinates.
    pub fn add_damage(&mut self, rect: Rect) {
        todo!()
    }

    /// Mark the whole frame damaged.
    pub fn damage_all(&mut self) {
        todo!()
    }

    /// Close open scopes, order entries by z-index within each scope, and end the frame.
    ///
    /// Returns an error if `pop` was called with no open scope or scopes were left open. The
    /// Scene is well-formed either way.
    pub fn finish(self) -> Result<(), SceneError> {
        todo!()
    }
}

impl Drop for SceneBuilder<'_> {
    fn drop(&mut self) {
        if !self.finished {
            todo!()
        }
    }
}
