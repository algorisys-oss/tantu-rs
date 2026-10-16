//! Damage from two Scenes, by element id (SCENE-DAMAGE-01..05). Spec:
//! `docs/specs/scene/damage.md`.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash, Hasher};

use tantu_core::{Affine, Rect};

use crate::command::{Clip, Command, Entry};
use crate::{ElementId, Scene};

/// More changed groups than this: full damage without merging.
const MAX_CHANGED: usize = 256;
/// More damage rects than this after merging: full damage (SCENE-DAMAGE-04).
const MAX_RECTS: usize = 16;
/// More of the frame than this damaged: full damage.
const MAX_AREA: f32 = 0.6;

/// A fast non-cryptographic hasher (the Fx hash used by rustc): damage hashing runs over
/// every draw of every frame, and SipHash was most of its cost.
#[derive(Clone, Copy, Debug, Default)]
struct FxHasher(u64);

impl FxHasher {
    const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(Self::SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            let mut word = [0u8; 8];
            word.copy_from_slice(chunk);
            self.add(u64::from_le_bytes(word));
        }
        for &byte in chunks.remainder() {
            self.add(u64::from(byte));
        }
    }

    #[inline]
    fn write_u8(&mut self, n: u8) {
        self.add(u64::from(n));
    }

    #[inline]
    fn write_u32(&mut self, n: u32) {
        self.add(u64::from(n));
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.add(n);
    }

    #[inline]
    fn write_i32(&mut self, n: i32) {
        self.add(n as u32 as u64);
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.add(n as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// A frame's groups, in first-appearance order, with an index by element.
#[derive(Clone, Debug, Default)]
struct Groups {
    index: HashMap<Option<ElementId>, u32, BuildHasherDefault<FxHasher>>,
    list: Vec<(Option<ElementId>, Group)>,
}

impl Groups {
    fn clear(&mut self) {
        self.index.clear();
        self.list.clear();
    }

    fn get(&self, key: &Option<ElementId>) -> Option<(usize, &Group)> {
        let i = *self.index.get(key)? as usize;
        self.list.get(i).map(|(_, g)| (i, g))
    }
}

/// One element's draws in a frame.
#[derive(Clone, Copy, Debug)]
struct Group {
    /// The draws, hashed in order with their context.
    hash: u64,
    /// The union of their window-space bounds, if any is visible.
    bounds: Option<Rect>,
    /// A draw had non-finite bounds.
    non_finite: bool,
}

/// Each unchanged group's predecessor among the unchanged groups (`None`: the first).
type Predecessors =
    HashMap<Option<ElementId>, Option<Option<ElementId>>, BuildHasherDefault<FxHasher>>;

/// Reusable buffers, kept in the Scene so a warm diff doesn't allocate (SCENE-DAMAGE-05).
#[derive(Clone, Debug, Default)]
pub(crate) struct DamageScratch {
    /// This Scene's own groups, valid while `own_generation` is its generation: the next
    /// frame's diff reads them instead of walking this Scene again.
    own: Groups,
    own_generation: Option<u64>,
    /// The previous Scene's groups, when it had none cached.
    old: Groups,
    /// Per old group: matched by an unchanged new group.
    old_unchanged: Vec<bool>,
    /// The unchanged groups in each frame's order (to detect reordering).
    old_sequence: Vec<Option<ElementId>>,
    new_sequence: Vec<Option<ElementId>>,
    old_predecessors: Predecessors,
    new_predecessors: Predecessors,
    stack: Vec<Context>,
    rects: Vec<Rect>,
}

/// What a draw is drawn under.
#[derive(Clone, Copy, Debug)]
struct Context {
    transform: Affine,
    /// The clip's window-space bounding box (`None`: the whole frame).
    clip: Option<Rect>,
    /// The open transforms, clips and layers, hashed as they are pushed.
    hash: u64,
}

impl Scene {
    /// Sets this frame's damage from what changed since `previous` (SCENE-DAMAGE-01..05),
    /// replacing any damage recorded while building. A different size gives full damage.
    pub fn diff_damage(&mut self, previous: &Scene) {
        let mut scratch = self.damage_scratch.take().unwrap_or_default();
        let frame = Rect::from_ltwh(0.0, 0.0, self.size().width, self.size().height);
        let DamageScratch {
            own,
            own_generation,
            old,
            old_unchanged,
            old_sequence,
            new_sequence,
            old_predecessors,
            new_predecessors,
            stack,
            rects,
        } = &mut *scratch;
        // This Scene's groups, kept for the next frame's diff.
        if *own_generation != Some(self.generation) {
            collect(self, frame, own, stack);
            *own_generation = Some(self.generation);
        }
        let full = if self.size() != previous.size() {
            true
        } else {
            // The previous Scene's groups: its own cache when it is current, else a walk.
            let cached = previous
                .damage_scratch
                .as_deref()
                .filter(|s| s.own_generation == Some(previous.generation));
            let old: &Groups = match cached {
                Some(cached) => &cached.own,
                None => {
                    collect(previous, frame, old, stack);
                    old
                }
            };
            let buffers = Buffers {
                old_unchanged,
                old_sequence,
                new_sequence,
                old_predecessors,
                new_predecessors,
            };
            compare(old, own, buffers, rects) || coalesce(rects, frame)
        };
        if full {
            self.set_full_damage();
        } else {
            self.set_damage_rects(rects);
        }
        self.damage_scratch = Some(scratch);
    }
}

/// Groups `scene`'s draws by element, in first-appearance order.
fn collect(scene: &Scene, frame: Rect, groups: &mut Groups, stack: &mut Vec<Context>) {
    groups.clear();
    stack.clear();
    let mut cx = Context {
        transform: Affine::IDENTITY,
        clip: None,
        hash: 0,
    };
    for entry in scene.entries() {
        match &entry.command {
            Command::PushTransform(t) => {
                stack.push(cx);
                cx.transform = cx.transform * *t;
                let mut h = FxHasher(cx.hash);
                1u8.hash(&mut h);
                t.coeffs().map(f32::to_bits).hash(&mut h);
                cx.hash = h.finish();
            }
            Command::PushClip(clip) => {
                stack.push(cx);
                let rect = match clip {
                    Clip::Rect(rect) => *rect,
                    Clip::RoundedRect(shape) => shape.rect,
                };
                let bbox = cx.transform.transform_rect_bbox(rect);
                let outer = cx.clip.unwrap_or(frame);
                cx.clip = Some(outer.intersect(bbox).unwrap_or(Rect::ZERO));
                let mut h = FxHasher(cx.hash);
                2u8.hash(&mut h);
                match clip {
                    Clip::Rect(r) => hash_rect(*r, &mut h),
                    Clip::RoundedRect(s) => hash_shape(s, &mut h),
                }
                cx.hash = h.finish();
            }
            Command::PushLayer(layer) => {
                stack.push(cx);
                let mut h = FxHasher(cx.hash);
                3u8.hash(&mut h);
                layer.opacity.to_bits().hash(&mut h);
                layer
                    .overlay_color
                    .map(|c| [c.r, c.g, c.b, c.a].map(f32::to_bits))
                    .hash(&mut h);
                cx.hash = h.finish();
            }
            Command::PopClip | Command::PopTransform | Command::PopLayer => {
                if let Some(outer) = stack.pop() {
                    cx = outer;
                }
            }
            _ => draw(scene, entry, &cx, frame, groups),
        }
    }
}

fn hash_rect(r: Rect, h: &mut FxHasher) {
    [r.left, r.top, r.right, r.bottom].map(f32::to_bits).hash(h);
}

fn hash_shape(s: &crate::RoundedRect, h: &mut FxHasher) {
    hash_rect(s.rect, h);
    let r = s.radii;
    [r.top_left, r.top_right, r.bottom_right, r.bottom_left]
        .map(f32::to_bits)
        .hash(h);
}

/// Adds one draw to its element's group.
fn draw(scene: &Scene, entry: &Entry, cx: &Context, frame: Rect, groups: &mut Groups) {
    let Groups { index, list } = groups;
    let i = *index.entry(entry.element).or_insert_with(|| {
        list.push((
            entry.element,
            Group {
                hash: 0,
                bounds: None,
                non_finite: false,
            },
        ));
        (list.len() - 1) as u32
    });
    let group = &mut list[i as usize].1;
    let mut h = FxHasher(group.hash);
    cx.hash.hash(&mut h);
    entry.z_index.hash(&mut h);
    hash_draw(scene, entry, &mut h);
    group.hash = h.finish();
    let Some(local) = local_bounds(scene, &entry.command) else {
        return;
    };
    let window = cx.transform.transform_rect_bbox(local);
    if !window.is_finite() {
        group.non_finite = true;
        return;
    }
    let visible = cx
        .clip
        .unwrap_or(frame)
        .intersect(window)
        .and_then(|r| r.intersect(frame))
        .filter(|r| !r.is_empty());
    if let Some(visible) = visible {
        group.bounds = Some(group.bounds.map_or(visible, |b| b.union(visible)));
    }
}

/// Hashes a draw command with its glyph or custom data (SCENE-DAMAGE-02).
fn hash_draw(scene: &Scene, entry: &Entry, h: &mut FxHasher) {
    let color =
        |c: tantu_core::Color, h: &mut FxHasher| [c.r, c.g, c.b, c.a].map(f32::to_bits).hash(h);
    match &entry.command {
        Command::Fill { shape, color: c } => {
            1u8.hash(h);
            hash_shape(shape, h);
            color(*c, h);
        }
        Command::Stroke {
            shape,
            width,
            color: c,
        } => {
            2u8.hash(h);
            hash_shape(shape, h);
            width.to_bits().hash(h);
            color(*c, h);
        }
        Command::BoxShadow(shadow) => {
            3u8.hash(h);
            hash_shape(&shadow.shape, h);
            color(shadow.color, h);
            [
                shadow.offset.x,
                shadow.offset.y,
                shadow.blur_radius,
                shadow.spread_radius,
            ]
            .map(f32::to_bits)
            .hash(h);
        }
        Command::Image(image) => {
            4u8.hash(h);
            image.image.hash(h);
            image
                .src
                .map(|s| [s.left, s.top, s.right, s.bottom].map(f32::to_bits))
                .hash(h);
            hash_rect(image.dest, h);
            (image.sampling as u8).hash(h);
            image.opacity.to_bits().hash(h);
        }
        Command::GlyphRun(run) => {
            5u8.hash(h);
            run.font.hash(h);
            run.font_size.to_bits().hash(h);
            color(run.color, h);
            [run.origin.x, run.origin.y].map(f32::to_bits).hash(h);
            for glyph in scene.glyphs(run) {
                glyph.id.hash(h);
                [glyph.x, glyph.y].map(f32::to_bits).hash(h);
            }
        }
        Command::Custom(custom) => {
            6u8.hash(h);
            custom.kind.hash(h);
            hash_rect(custom.bounds, h);
            scene.custom_data(custom).hash(h);
        }
        Command::PushClip(_)
        | Command::PopClip
        | Command::PushTransform(_)
        | Command::PopTransform
        | Command::PushLayer(_)
        | Command::PopLayer => {}
    }
}

/// Where a draw command can paint, in its local coordinates (SCENE-DAMAGE-03).
fn local_bounds(scene: &Scene, command: &Command) -> Option<Rect> {
    Some(match command {
        Command::Fill { shape, .. } => shape.rect,
        Command::Stroke { shape, width, .. } => shape.rect.inflate(width.abs() / 2.0),
        Command::BoxShadow(shadow) => {
            let sigma = shadow.blur_radius.max(0.0) * 0.577_35 + 0.5;
            shadow
                .shape
                .rect
                .translate(shadow.offset)
                .inflate(shadow.spread_radius + 3.0 * sigma)
        }
        Command::Image(image) => image.dest,
        Command::GlyphRun(run) => {
            let glyphs = scene.glyphs(run);
            let first = glyphs.first()?;
            // By hand: `Rect::union` skips empty rects, and a glyph position is one.
            let (mut left, mut top, mut right, mut bottom) = (first.x, first.y, first.x, first.y);
            for glyph in glyphs {
                left = left.min(glyph.x);
                top = top.min(glyph.y);
                right = right.max(glyph.x);
                bottom = bottom.max(glyph.y);
            }
            Rect::from_ltrb(left, top, right, bottom)
                .translate(tantu_core::Vec2::new(run.origin.x, run.origin.y))
                .inflate(run.font_size.abs())
        }
        Command::Custom(custom) => custom.bounds,
        _ => return None,
    })
}

/// `compare`'s reusable buffers.
struct Buffers<'a> {
    old_unchanged: &'a mut Vec<bool>,
    old_sequence: &'a mut Vec<Option<ElementId>>,
    new_sequence: &'a mut Vec<Option<ElementId>>,
    old_predecessors: &'a mut Predecessors,
    new_predecessors: &'a mut Predecessors,
}

/// Collects the damaged groups' bounds into `rects`; true when the damage must be full.
fn compare(old: &Groups, new: &Groups, buffers: Buffers<'_>, rects: &mut Vec<Rect>) -> bool {
    let Buffers {
        old_unchanged,
        old_sequence,
        new_sequence,
        old_predecessors,
        new_predecessors,
    } = buffers;
    rects.clear();
    old_unchanged.clear();
    old_unchanged.resize(old.list.len(), false);
    new_sequence.clear();
    // One lookup per new group: unchanged, changed or new.
    for (key, n) in &new.list {
        if let Some((i, o)) = old.get(key) {
            if o.hash == n.hash {
                old_unchanged[i] = true;
                new_sequence.push(*key);
            }
        }
    }
    old_sequence.clear();
    old_sequence.extend(
        old.list
            .iter()
            .zip(old_unchanged.iter())
            .filter(|(_, unchanged)| **unchanged)
            .map(|((key, _), _)| *key),
    );
    // The unchanged groups kept their relative order, unless the sequences differ; then
    // each one's predecessor among them decides (SCENE-DAMAGE-02).
    let reordered = old_sequence != new_sequence;
    if reordered {
        for (sequence, predecessors) in [
            (&*old_sequence, &mut *old_predecessors),
            (&*new_sequence, &mut *new_predecessors),
        ] {
            predecessors.clear();
            let mut previous = None;
            for key in sequence {
                predecessors.insert(*key, previous);
                previous = Some(*key);
            }
        }
    }
    // A damaged group adds the union of where it was and where it is.
    let add = |a: Option<&Group>, b: Option<&Group>, rects: &mut Vec<Rect>| -> bool {
        let groups = [a, b];
        if groups.iter().flatten().any(|g| g.non_finite) {
            return true;
        }
        let union = groups
            .iter()
            .flatten()
            .filter_map(|g| g.bounds)
            .reduce(Rect::union);
        if let Some(union) = union {
            rects.push(union);
        }
        false
    };
    let mut changed = 0usize;
    for (key, n) in &new.list {
        let o = old.get(key).map(|(_, o)| o);
        let damaged = match o {
            Some(o) => {
                o.hash != n.hash
                    || (reordered && old_predecessors.get(key) != new_predecessors.get(key))
            }
            None => true,
        };
        if damaged {
            changed += 1;
            if changed > MAX_CHANGED || add(o, Some(n), rects) {
                return true;
            }
        }
    }
    for (key, o) in &old.list {
        if !new.index.contains_key(key) {
            changed += 1;
            if changed > MAX_CHANGED || add(Some(o), None, rects) {
                return true;
            }
        }
    }
    false
}

/// Merges overlapping or touching rects (SCENE-DAMAGE-04); true when the result must be
/// full damage.
fn coalesce(rects: &mut Vec<Rect>, frame: Rect) -> bool {
    let touches = |a: &Rect, b: &Rect| {
        a.left <= b.right && b.left <= a.right && a.top <= b.bottom && b.top <= a.bottom
    };
    let mut merged = true;
    while merged {
        merged = false;
        'outer: for i in 0..rects.len() {
            for j in i + 1..rects.len() {
                if touches(&rects[i], &rects[j]) {
                    rects[i] = rects[i].union(rects[j]);
                    rects.swap_remove(j);
                    merged = true;
                    break 'outer;
                }
            }
        }
    }
    let area: f32 = rects.iter().map(|r| r.width() * r.height()).sum();
    rects.len() > MAX_RECTS || area > MAX_AREA * frame.width() * frame.height()
}
