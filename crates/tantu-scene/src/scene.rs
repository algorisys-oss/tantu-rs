//! [`Scene`], the per-frame display list, and [`SceneBuilder`], which records into it.

use std::fmt;

use tantu_core::{Affine, Color, Point, Rect, Size};

use crate::command::{
    BoxShadow, Clip, Command, CustomDraw, Entry, Glyph, GlyphRun, ImageDraw, Layer, RoundedRect,
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
        write!(
            f,
            "unbalanced scene scopes: {} pop(s) with no open scope, {} scope(s) left open",
            self.unmatched_pops, self.unclosed_scopes
        )
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

/// One unit to order within a scope: a single entry, or a whole nested scope
/// (`start..end` in the unsorted entries).
#[derive(Clone, Copy, Debug)]
struct Unit {
    z_index: i32,
    start: usize,
    end: usize,
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
        Scene {
            size: Size::ZERO,
            entries: Vec::new(),
            glyphs: Vec::new(),
            data: Vec::new(),
            damage: DamageState::Unset,
            damage_rects: Vec::new(),
            scratch: Scratch {
                stack: Vec::new(),
                units: Vec::new(),
                sorted: Vec::new(),
            },
        }
    }

    /// Clears the Scene, keeping its buffers, and starts recording a frame of `size` logical
    /// pixels.
    pub fn begin(&mut self, size: Size) -> SceneBuilder<'_> {
        self.clear();
        self.size = size;
        SceneBuilder {
            scene: self,
            element: None,
            z_index: 0,
            transform: Affine::IDENTITY,
            visible: Some(Rect::from_origin_size(Point::ZERO, size)),
            unmatched_pops: 0,
            finished: false,
        }
    }

    /// Size of the frame in logical pixels.
    pub fn size(&self) -> Size {
        self.size
    }

    /// The recorded entries, in paint order (later entries draw on top).
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The glyphs of `run`, a run from this Scene's current frame.
    ///
    /// For any other run the result is unspecified, but never a panic: empty if it indexes past
    /// the glyph buffer.
    pub fn glyphs(&self, run: &GlyphRun) -> &[Glyph] {
        sub_slice(&self.glyphs, run.start, run.len)
    }

    /// The bytes of `custom`, a custom command from this Scene's current frame.
    ///
    /// For any other one the result is unspecified, but never a panic: empty if it indexes past
    /// the data buffer.
    pub fn custom_data(&self, custom: &CustomDraw) -> &[u8] {
        sub_slice(&self.data, custom.start, custom.len)
    }

    /// The damage region.
    pub fn damage(&self) -> Damage<'_> {
        match self.damage {
            DamageState::Rects => Damage::Rects(&self.damage_rects),
            DamageState::Unset | DamageState::All => Damage::Full,
        }
    }

    /// Empties every buffer, keeping its capacity.
    fn clear(&mut self) {
        self.entries.clear();
        self.glyphs.clear();
        self.data.clear();
        self.damage = DamageState::Unset;
        self.damage_rects.clear();
        self.scratch.stack.clear();
        self.scratch.units.clear();
        self.scratch.sorted.clear();
    }
}

/// `len` items of `buf` from `start`, or empty if that range is out of bounds.
fn sub_slice<T>(buf: &[T], start: u32, len: u32) -> &[T] {
    let start = start as usize;
    start
        .checked_add(len as usize)
        .and_then(|end| buf.get(start..end))
        .unwrap_or(&[])
}

/// `(start, len)` of `extra` items appended to a buffer holding `current`, or `None` if either
/// doesn't fit in `u32`.
fn span(current: usize, extra: usize) -> Option<(u32, u32)> {
    Some((u32::try_from(current).ok()?, u32::try_from(extra).ok()?))
}

impl Default for Scene {
    fn default() -> Self {
        Scene::new()
    }
}

impl PartialEq for Scene {
    /// Compares what renderers see: size, entries, glyph and custom data, damage. Internal
    /// scratch buffers are ignored.
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
            && self.entries == other.entries
            && self.glyphs == other.glyphs
            && self.data == other.data
            && self.damage() == other.damage()
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
    /// Local-to-scene transform of the following commands.
    transform: Affine,
    /// Bounding box, in scene coordinates, of what can still be visible; `None` if nothing can.
    visible: Option<Rect>,
    unmatched_pops: u32,
    finished: bool,
}

impl SceneBuilder<'_> {
    /// Element id for the following commands. Default `None`.
    pub fn set_element(&mut self, element: Option<ElementId>) {
        self.element = element;
    }

    /// Z-index for the following commands, within the current scope. Default 0.
    pub fn set_z_index(&mut self, z_index: i32) {
        self.z_index = z_index;
    }

    /// Clip the following commands to `clip` (intersected with any outer clip).
    pub fn push_clip(&mut self, clip: Clip) {
        self.open_scope(ScopeKind::Clip, Command::PushClip(clip));
        let bounds = match clip {
            Clip::Rect(rect) => rect,
            Clip::RoundedRect(shape) => shape.rect,
        };
        self.visible = if self.transform.is_finite() && bounds.is_finite() {
            let bounds = self.transform.transform_rect_bbox(bounds);
            self.visible.and_then(|visible| visible.intersect(bounds))
        } else {
            None
        };
    }

    /// Apply `transform` to the following commands, after the current transform.
    pub fn push_transform(&mut self, transform: Affine) {
        self.open_scope(ScopeKind::Transform, Command::PushTransform(transform));
        self.transform = self.transform * transform;
    }

    /// Composite the following commands as one group.
    pub fn push_layer(&mut self, layer: Layer) {
        self.open_scope(ScopeKind::Layer, Command::PushLayer(layer));
    }

    /// Close the innermost open scope. With no open scope, records nothing and is reported by
    /// [`finish`](SceneBuilder::finish).
    pub fn pop(&mut self) {
        let Some(frame) = self.scene.scratch.stack.pop() else {
            self.unmatched_pops = self.unmatched_pops.saturating_add(1);
            return;
        };
        let command = match frame.kind {
            ScopeKind::Clip => Command::PopClip,
            ScopeKind::Transform => Command::PopTransform,
            ScopeKind::Layer => Command::PopLayer,
        };
        self.scene.entries.push(Entry {
            element: frame.element,
            z_index: frame.z_index,
            command,
        });
        self.element = frame.element;
        self.z_index = frame.z_index;
        self.transform = frame.transform;
        self.visible = frame.visible;
    }

    /// Fill a rect.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.fill(RoundedRect::from_rect(rect), color);
    }

    /// Fill a rounded rect.
    pub fn fill(&mut self, shape: RoundedRect, color: Color) {
        self.record(Command::Fill { shape, color });
    }

    /// Stroke a rounded rect, inside its edge.
    pub fn stroke(&mut self, shape: RoundedRect, width: f32, color: Color) {
        self.record(Command::Stroke {
            shape,
            width,
            color,
        });
    }

    /// Draw a box shadow.
    pub fn box_shadow(&mut self, shadow: BoxShadow) {
        self.record(Command::BoxShadow(shadow));
    }

    /// Draw an image.
    pub fn image(&mut self, image: ImageDraw) {
        self.record(Command::Image(image));
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
        // A run that can't be indexed with u32 records no glyphs rather than failing.
        let (start, len) = match span(self.scene.glyphs.len(), glyphs.len()) {
            Some(span) => {
                self.scene.glyphs.extend_from_slice(glyphs);
                span
            }
            None => (u32::MAX, 0),
        };
        self.record(Command::GlyphRun(GlyphRun {
            font,
            font_size,
            color,
            origin,
            start,
            len,
        }));
    }

    /// Record a custom command; `data` is copied into the Scene.
    pub fn custom(&mut self, kind: CustomKind, bounds: Rect, data: &[u8]) {
        let (start, len) = match span(self.scene.data.len(), data.len()) {
            Some(span) => {
                self.scene.data.extend_from_slice(data);
                span
            }
            None => (u32::MAX, 0),
        };
        self.record(Command::Custom(CustomDraw {
            kind,
            bounds,
            start,
            len,
        }));
    }

    /// True if nothing inside `local_bounds` (current local coordinates) can be visible.
    ///
    /// Conservative: it may return false for content that ends up invisible, never true for
    /// content that could be visible.
    pub fn is_culled(&self, local_bounds: Rect) -> bool {
        if !local_bounds.is_finite() || local_bounds.is_empty() || !self.transform.is_finite() {
            return true;
        }
        match self.visible {
            Some(visible) => !self
                .transform
                .transform_rect_bbox(local_bounds)
                .overlaps(visible),
            None => true,
        }
    }

    /// Add a damaged rect, in scene coordinates.
    pub fn add_damage(&mut self, rect: Rect) {
        if self.scene.damage != DamageState::All {
            self.scene.damage = DamageState::Rects;
            self.scene.damage_rects.push(rect);
        }
    }

    /// Mark the whole frame damaged.
    pub fn damage_all(&mut self) {
        self.scene.damage = DamageState::All;
        self.scene.damage_rects.clear();
    }

    /// Close open scopes, order entries by z-index within each scope, and end the frame.
    ///
    /// Returns an error if `pop` was called with no open scope or scopes were left open. The
    /// Scene is well-formed either way.
    pub fn finish(mut self) -> Result<(), SceneError> {
        let unclosed = self.scene.scratch.stack.len();
        for _ in 0..unclosed {
            self.pop();
        }
        self.finished = true;
        sort_by_z(self.scene);

        let error = SceneError {
            unmatched_pops: self.unmatched_pops,
            unclosed_scopes: u32::try_from(unclosed).unwrap_or(u32::MAX),
        };
        if error.unmatched_pops == 0 && error.unclosed_scopes == 0 {
            Ok(())
        } else {
            Err(error)
        }
    }

    /// Records `command` with the current element id and z-index.
    fn record(&mut self, command: Command) {
        self.scene.entries.push(Entry {
            element: self.element,
            z_index: self.z_index,
            command,
        });
    }

    /// Records a push entry and saves the state its pop restores.
    fn open_scope(&mut self, kind: ScopeKind, command: Command) {
        self.record(command);
        self.scene.scratch.stack.push(Frame {
            kind,
            element: self.element,
            z_index: self.z_index,
            transform: self.transform,
            visible: self.visible,
        });
    }
}

impl Drop for SceneBuilder<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.scene.clear();
        }
    }
}

/// Stably orders the entries of every scope by z-index (SCENE-SCENE-12). The entries must be
/// well-formed (balanced, properly nested scopes).
fn sort_by_z(scene: &mut Scene) {
    let Some(first) = scene.entries.first() else {
        return;
    };
    let first = first.z_index;
    if scene.entries.iter().all(|e| e.z_index == first) {
        return;
    }
    let Scratch { units, sorted, .. } = &mut scene.scratch;
    sorted.clear();
    emit_scope(&scene.entries, 0, scene.entries.len(), units, sorted);
    std::mem::swap(&mut scene.entries, sorted);
    sorted.clear();
}

/// Appends the entries `src[start..end]`, one scope's contents, to `out` in z order, recursing
/// into nested scopes. `units` is a shared stack of scratch space.
fn emit_scope(
    src: &[Entry],
    start: usize,
    end: usize,
    units: &mut Vec<Unit>,
    out: &mut Vec<Entry>,
) {
    let base = units.len();
    let mut i = start;
    while i < end {
        let next = unit_end(src, i);
        units.push(Unit {
            z_index: src[i].z_index,
            start: i,
            end: next,
        });
        i = next;
    }
    // Keys are unique (z, position), so an unstable sort is stable here and doesn't allocate.
    units[base..].sort_unstable_by_key(|u| (u.z_index, u.start));
    for k in base..units.len() {
        let unit = units[k];
        out.push(src[unit.start].clone());
        if unit.end - unit.start > 1 {
            // A scope: its contents, then its pop entry.
            emit_scope(src, unit.start + 1, unit.end - 1, units, out);
            out.push(src[unit.end - 1].clone());
        }
    }
    units.truncate(base);
}

/// The index just past the unit starting at `i`: `i + 1` for a single entry, or just past the
/// matching pop for a push entry.
fn unit_end(src: &[Entry], i: usize) -> usize {
    if !is_push(&src[i].command) {
        return i + 1;
    }
    let mut depth = 0usize;
    for (j, entry) in src.iter().enumerate().skip(i) {
        if is_push(&entry.command) {
            depth += 1;
        } else if is_pop(&entry.command) {
            depth -= 1;
            if depth == 0 {
                return j + 1;
            }
        }
    }
    // Unreachable for well-formed entries; treat an unmatched push as a single entry.
    i + 1
}

fn is_push(command: &Command) -> bool {
    matches!(
        command,
        Command::PushClip(_) | Command::PushTransform(_) | Command::PushLayer(_)
    )
}

fn is_pop(command: &Command) -> bool {
    matches!(
        command,
        Command::PopClip | Command::PopTransform | Command::PopLayer
    )
}
