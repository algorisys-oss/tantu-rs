//! [`SoftRenderer`] and the custom-command handler API.

use std::collections::HashMap;
use std::fmt;

use tantu_core::{Affine, Color, Rect};
use tantu_scene::{
    BoxShadow, Clip, Command, CustomKind, ImageData, ImageDraw, ImageId, ImageSampling, Layer,
    RenderError, RenderReport, Renderer, Resources, RoundedRect, Scene,
};
use tiny_skia::{
    BlendMode, FillRule, FilterQuality, IntSize, Mask, Paint, Path, PathBuilder, PathSegment,
    Pattern, Pixmap, PixmapPaint, PremultipliedColorU8, Shader, SpreadMode, Transform,
};

use crate::blur;

/// Draws one kind of custom command.
pub trait CustomHandler {
    /// Draw `canvas.data` into `canvas.pixmap`, inside `canvas.bounds`.
    fn draw(&mut self, canvas: CustomCanvas<'_>);
}

/// What a custom handler draws into.
pub struct CustomCanvas<'a> {
    /// The target (or the open layer's offscreen target), premultiplied RGBA8.
    pub pixmap: tiny_skia::PixmapMut<'a>,
    /// Maps the command's local coordinates to target pixels (current transform × scale factor).
    pub transform: tiny_skia::Transform,
    /// The current clip as a mask over the pixmap, if any clip is open.
    pub clip: Option<&'a tiny_skia::Mask>,
    /// The command's bounds, in local coordinates.
    pub bounds: Rect,
    /// The command's bytes (`Scene::custom_data`).
    pub data: &'a [u8],
}

/// A renderer that draws Scenes into an in-memory RGBA8 target on the CPU.
pub struct SoftRenderer {
    width: u32,
    height: u32,
    scale_factor: f32,
    /// `None` when the target has no pixels (0 width or height) or couldn't be allocated.
    target: Option<Pixmap>,
    handlers: HashMap<CustomKind, Box<dyn CustomHandler>>,
    /// Uploaded (premultiplied) images. Handles are never reused, so an entry stays valid while
    /// its handle is in the resources.
    images: HashMap<ImageId, Pixmap>,
    /// The `Resources::revision` the image cache was last pruned at.
    images_revision: Option<u64>,
    /// Offscreen targets for open layers, by nesting depth; reused between frames.
    layers: Vec<Pixmap>,
    /// Clip masks, by nesting depth (each already intersected with the outer ones).
    clips: Vec<Mask>,
    /// Scratch space for blurred shadows.
    blur: blur::Scratch,
}

/// Something pushed and not yet popped while drawing a frame.
#[derive(Clone, Copy)]
enum Scope {
    Transform,
    Clip,
    Layer(Layer),
}

impl SoftRenderer {
    /// A `width × height` physical-pixel target, scale factor 1, fully transparent.
    pub fn new(width: u32, height: u32) -> SoftRenderer {
        SoftRenderer {
            width,
            height,
            scale_factor: 1.0,
            target: Pixmap::new(width, height),
            handlers: HashMap::new(),
            images: HashMap::new(),
            images_revision: None,
            layers: Vec::new(),
            clips: Vec::new(),
            blur: blur::Scratch::default(),
        }
    }

    /// Current target width and height in physical pixels.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Current scale factor.
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Draw custom commands of `kind` with `handler`, replacing any earlier handler for it.
    pub fn register_custom(&mut self, kind: CustomKind, handler: impl CustomHandler + 'static) {
        self.handlers.insert(kind, Box::new(handler));
    }

    /// The target's pixels as straight-alpha RGBA8. `None` for a 0-width or 0-height target.
    pub fn snapshot(&self) -> Option<ImageData> {
        let target = self.target.as_ref()?;
        let pixels: Vec<u8> = target
            .pixels()
            .iter()
            .flat_map(|p| {
                let c = p.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect();
        ImageData::rgba8(target.width(), target.height(), pixels).ok()
    }

    /// Drops cached images whose handle is no longer in `resources` (contract item 6).
    fn prune_images(&mut self, resources: &Resources) {
        if self.images_revision != Some(resources.revision()) {
            self.images.retain(|id, _| resources.image(*id).is_some());
            self.images_revision = Some(resources.revision());
        }
    }
}

impl Renderer for SoftRenderer {
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        if (width, height) != (self.width, self.height) || self.target.is_none() {
            self.width = width;
            self.height = height;
            self.target = Pixmap::new(width, height);
        } else if let Some(target) = &mut self.target {
            target.fill(tiny_skia::Color::TRANSPARENT);
        }
    }

    fn render(
        &mut self,
        scene: &Scene,
        resources: &Resources,
    ) -> Result<RenderReport, RenderError> {
        if self.width == 0 || self.height == 0 {
            return Ok(RenderReport::default());
        }
        let handlers = &self.handlers;
        let mut report =
            RenderReport::for_scene(scene, resources, &|kind| handlers.contains_key(&kind));
        self.prune_images(resources);
        let Some(target) = self.target.as_mut() else {
            return Err(RenderError::OutOfMemory);
        };
        target.fill(tiny_skia::Color::TRANSPARENT);

        let mut frame = FrameState {
            target,
            layers: &mut self.layers,
            clips: &mut self.clips,
            images: &mut self.images,
            handlers: &mut self.handlers,
            blur: &mut self.blur,
            scale: self.scale_factor,
            transform: Affine::IDENTITY,
            transforms: Vec::new(),
            scopes: Vec::new(),
            clip_depth: 0,
            layer_depth: 0,
        };
        // Depth inside a scope hidden by a non-finite transform or clip (SCENE-RENDER-04).
        let mut hidden = 0usize;
        for entry in scene.entries() {
            let command = &entry.command;
            if hidden > 0 {
                match command {
                    Command::PushClip(_) | Command::PushTransform(_) | Command::PushLayer(_) => {
                        hidden += 1;
                    }
                    Command::PopClip | Command::PopTransform | Command::PopLayer => hidden -= 1,
                    _ => {}
                }
                continue;
            }
            match command {
                Command::PushTransform(t) if !t.is_finite() => hidden = 1,
                Command::PushClip(clip) if !clip_is_finite(clip) => hidden = 1,
                Command::PushTransform(t) => frame.push_transform(*t),
                Command::PushClip(clip) => frame.push_clip(clip),
                Command::PushLayer(layer) => frame.push_layer(*layer),
                Command::PopClip | Command::PopTransform | Command::PopLayer => frame.pop(),
                draw if !is_drawable(draw, scene) => {}
                Command::Fill { shape, color } => frame.fill(shape, *color),
                Command::Stroke {
                    shape,
                    width,
                    color,
                } => frame.stroke(shape, *width, *color),
                Command::BoxShadow(shadow) => frame.box_shadow(shadow),
                Command::Image(image) => frame.image(image, resources),
                Command::GlyphRun(run) => {
                    // No text yet: a run with a present font is a font this backend couldn't use.
                    if resources.font(run.font).is_some() {
                        report.missing_fonts = report.missing_fonts.saturating_add(1);
                    }
                }
                Command::Custom(custom) => {
                    frame.custom(custom.kind, custom.bounds, scene.custom_data(custom));
                }
            }
        }
        // Close anything left open (a Scene is always balanced, but stay safe).
        while !frame.scopes.is_empty() {
            frame.pop();
        }
        Ok(report)
    }
}

impl fmt::Debug for SoftRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoftRenderer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale_factor", &self.scale_factor)
            .field("custom_kinds", &self.handlers.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

/// Drawing state for one frame, borrowing the renderer's buffers.
struct FrameState<'a> {
    target: &'a mut Pixmap,
    layers: &'a mut Vec<Pixmap>,
    clips: &'a mut Vec<Mask>,
    images: &'a mut HashMap<ImageId, Pixmap>,
    handlers: &'a mut HashMap<CustomKind, Box<dyn CustomHandler>>,
    blur: &'a mut blur::Scratch,
    scale: f32,
    /// Local-to-scene transform of the current scope.
    transform: Affine,
    /// Outer transforms saved by `push_transform`.
    transforms: Vec<Affine>,
    scopes: Vec<Scope>,
    clip_depth: usize,
    layer_depth: usize,
}

impl FrameState<'_> {
    /// Local coordinates to target pixels.
    fn device_transform(&self) -> Transform {
        let [a, b, c, d, e, f] = self.transform.coeffs();
        let s = self.scale;
        Transform::from_row(a * s, b * s, c * s, d * s, e * s, f * s)
    }

    /// The pixmap commands currently draw into and the current clip mask.
    fn surface(&mut self) -> (&mut Pixmap, Option<&Mask>) {
        let clip = self.clip_depth.checked_sub(1).map(|i| &self.clips[i]);
        let target = match self.layer_depth.checked_sub(1) {
            Some(i) => &mut self.layers[i],
            None => &mut *self.target,
        };
        (target, clip)
    }

    fn push_transform(&mut self, transform: Affine) {
        self.transforms.push(self.transform);
        self.transform = self.transform * transform;
        self.scopes.push(Scope::Transform);
    }

    fn push_clip(&mut self, clip: &Clip) {
        let (w, h) = (self.target.width(), self.target.height());
        let depth = self.clip_depth;
        if self.clips.len() <= depth {
            match Mask::new(w, h) {
                Some(mask) => self.clips.push(mask),
                None => return,
            }
        }
        if (self.clips[depth].width(), self.clips[depth].height()) != (w, h) {
            match Mask::new(w, h) {
                Some(mask) => self.clips[depth] = mask,
                None => return,
            }
        }
        let ts = self.device_transform();
        let (outer, rest) = self.clips.split_at_mut(depth);
        let mask = &mut rest[0];
        match outer.last() {
            Some(parent) => mask.data_mut().copy_from_slice(parent.data()),
            None => mask.data_mut().fill(255),
        }
        let shape = match *clip {
            Clip::Rect(rect) => RoundedRect::from_rect(rect),
            Clip::RoundedRect(shape) => shape,
        };
        match rounded_rect_path(&shape).and_then(|path| device_path(&path, ts)) {
            Some(path) => {
                mask.intersect_path(&path, FillRule::Winding, true, Transform::identity())
            }
            None => mask.clear(),
        }
        self.clip_depth += 1;
        self.scopes.push(Scope::Clip);
    }

    fn push_layer(&mut self, layer: Layer) {
        let (w, h) = (self.target.width(), self.target.height());
        let depth = self.layer_depth;
        if self.layers.len() <= depth {
            match Pixmap::new(w, h) {
                Some(pixmap) => self.layers.push(pixmap),
                None => return,
            }
        }
        if (self.layers[depth].width(), self.layers[depth].height()) != (w, h) {
            match Pixmap::new(w, h) {
                Some(pixmap) => self.layers[depth] = pixmap,
                None => return,
            }
        }
        self.layers[depth].fill(tiny_skia::Color::TRANSPARENT);
        self.layer_depth += 1;
        self.scopes.push(Scope::Layer(layer));
    }

    fn pop(&mut self) {
        match self.scopes.pop() {
            Some(Scope::Transform) => {
                self.transform = self.transforms.pop().unwrap_or(Affine::IDENTITY);
            }
            Some(Scope::Clip) => self.clip_depth = self.clip_depth.saturating_sub(1),
            Some(Scope::Layer(layer)) => self.composite_layer(layer),
            None => {}
        }
    }

    /// Applies the overlay color to the top layer and draws it onto the surface below
    /// (RENDER-SOFT-20, -21).
    fn composite_layer(&mut self, layer: Layer) {
        let Some(top) = self.layer_depth.checked_sub(1) else {
            return;
        };
        self.layer_depth = top;
        let (below, rest) = self.layers.split_at_mut(top);
        let pixmap = &mut rest[0];
        if let Some(overlay) = layer.overlay_color.filter(|c| color_is_finite(*c)) {
            if let Some(full) =
                tiny_skia::Rect::from_xywh(0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32)
            {
                let mut paint = solid_paint(overlay);
                paint.blend_mode = BlendMode::SourceAtop;
                paint.anti_alias = false;
                pixmap.fill_rect(full, &paint, Transform::identity(), None);
            }
        }
        // NaN opacity counts as 0 (scene.md, drawing semantics 10).
        let opacity = if layer.opacity.is_nan() {
            0.0
        } else {
            layer.opacity.clamp(0.0, 1.0)
        };
        if opacity <= 0.0 {
            return;
        }
        let paint = PixmapPaint {
            opacity,
            blend_mode: BlendMode::SourceOver,
            quality: FilterQuality::Nearest,
        };
        let dest = match below.last_mut() {
            Some(pixmap) => pixmap,
            None => &mut *self.target,
        };
        dest.draw_pixmap(0, 0, pixmap.as_ref(), &paint, Transform::identity(), None);
    }

    fn fill(&mut self, shape: &RoundedRect, color: Color) {
        let Some(path) = rounded_rect_path(shape) else {
            return;
        };
        if color.clamp().is_transparent() {
            return;
        }
        let Some(path) = device_path(&path, self.device_transform()) else {
            return;
        };
        let paint = solid_paint(color);
        let (target, clip) = self.surface();
        target.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            clip,
        );
    }

    fn stroke(&mut self, shape: &RoundedRect, width: f32, color: Color) {
        if width <= 0.0 || color.clamp().is_transparent() {
            return;
        }
        let rect = shape.rect;
        let shorter = (rect.right - rect.left).min(rect.bottom - rect.top);
        if width >= shorter / 2.0 {
            self.fill(shape, color);
            return;
        }
        let outer = clamp_radii(shape);
        let inner = RoundedRect::new(
            Rect::from_ltrb(
                rect.left + width,
                rect.top + width,
                rect.right - width,
                rect.bottom - width,
            ),
            tantu_scene::BorderRadius {
                top_left: (outer.radii.top_left - width).max(0.0),
                top_right: (outer.radii.top_right - width).max(0.0),
                bottom_right: (outer.radii.bottom_right - width).max(0.0),
                bottom_left: (outer.radii.bottom_left - width).max(0.0),
            },
        );
        let mut pb = PathBuilder::new();
        push_rounded_rect(&mut pb, &outer);
        push_rounded_rect(&mut pb, &inner);
        let Some(path) = pb.finish() else {
            return;
        };
        let Some(path) = device_path(&path, self.device_transform()) else {
            return;
        };
        let paint = solid_paint(color);
        let (target, clip) = self.surface();
        target.fill_path(
            &path,
            &paint,
            FillRule::EvenOdd,
            Transform::identity(),
            clip,
        );
    }

    fn box_shadow(&mut self, shadow: &BoxShadow) {
        if shadow.color.clamp().is_transparent() {
            return;
        }
        let spread = shadow.spread_radius;
        let r = shadow.shape.rect;
        let radii = clamp_radii(&shadow.shape).radii;
        let grow = |radius: f32| {
            if radius > 0.0 {
                (radius + spread).max(0.0)
            } else {
                0.0
            }
        };
        let shape = RoundedRect::new(
            Rect::from_ltrb(
                r.left - spread + shadow.offset.x,
                r.top - spread + shadow.offset.y,
                r.right + spread + shadow.offset.x,
                r.bottom + spread + shadow.offset.y,
            ),
            tantu_scene::BorderRadius {
                top_left: grow(radii.top_left),
                top_right: grow(radii.top_right),
                bottom_right: grow(radii.bottom_right),
                bottom_left: grow(radii.bottom_left),
            },
        );
        let Some(path) = rounded_rect_path(&shape) else {
            return;
        };
        if shadow.blur_radius <= 0.0 {
            self.fill(&shape, shadow.color);
            return;
        }
        // Flutter's conversion, then into device pixels by the transform's average scale.
        let local_sigma = shadow.blur_radius * 0.57735 + 0.5;
        let ts = self.device_transform();
        let sigma = local_sigma * (ts.sx * ts.sy - ts.kx * ts.ky).abs().sqrt();
        let Some(path) = device_path(&path, ts) else {
            return;
        };
        let paint = solid_paint(shadow.color);
        let scratch = &mut *self.blur;
        let clip = self.clip_depth.checked_sub(1).map(|i| &self.clips[i]);
        let target = match self.layer_depth.checked_sub(1) {
            Some(i) => &mut self.layers[i],
            None => &mut *self.target,
        };
        blur::fill_blurred(target, clip, &path, sigma, &paint, scratch);
    }

    fn image(&mut self, draw: &ImageDraw, resources: &Resources) {
        let Some(data) = resources.image(draw.image) else {
            return;
        };
        let opacity = draw.opacity.clamp(0.0, 1.0);
        if opacity <= 0.0 {
            return;
        }
        if let std::collections::hash_map::Entry::Vacant(slot) = self.images.entry(draw.image) {
            match upload(data) {
                Some(pixmap) => {
                    slot.insert(pixmap);
                }
                None => return,
            }
        }
        let src = draw.src.unwrap_or(Rect::from_ltwh(
            0.0,
            0.0,
            data.width() as f32,
            data.height() as f32,
        ));
        let dest = draw.dest;
        let (src_w, src_h) = (src.right - src.left, src.bottom - src.top);
        let (dest_w, dest_h) = (dest.right - dest.left, dest.bottom - dest.top);
        if !(src_w > 0.0 && src_h > 0.0) {
            return;
        }
        let Some(rect) = tiny_skia::Rect::from_ltrb(dest.left, dest.top, dest.right, dest.bottom)
        else {
            return;
        };
        let ts = self.device_transform();
        let Some(path) = device_path(&PathBuilder::from_rect(rect), ts) else {
            return;
        };
        // Image pixels to local coordinates: src's corner onto dest's, scaled to fill it.
        let pattern_ts = Transform::from_translate(dest.left, dest.top)
            .pre_scale(dest_w / src_w, dest_h / src_h)
            .pre_translate(-src.left, -src.top);
        let quality = match draw.sampling {
            ImageSampling::Nearest => FilterQuality::Nearest,
            ImageSampling::Linear => FilterQuality::Bilinear,
        };
        let FrameState {
            images,
            target,
            layers,
            clips,
            clip_depth,
            layer_depth,
            ..
        } = self;
        let Some(pixmap) = images.get(&draw.image) else {
            return;
        };
        let paint = Paint {
            shader: Pattern::new(
                pixmap.as_ref(),
                SpreadMode::Pad,
                quality,
                opacity,
                pattern_ts.post_concat(ts),
            ),
            ..Paint::default()
        };
        let clip = clip_depth.checked_sub(1).map(|i| &clips[i]);
        let target = match layer_depth.checked_sub(1) {
            Some(i) => &mut layers[i],
            None => &mut **target,
        };
        target.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            clip,
        );
    }

    fn custom(&mut self, kind: CustomKind, bounds: Rect, data: &[u8]) {
        let transform = self.device_transform();
        let FrameState {
            handlers,
            target,
            layers,
            clips,
            clip_depth,
            layer_depth,
            ..
        } = self;
        let Some(handler) = handlers.get_mut(&kind) else {
            return;
        };
        let clip = clip_depth.checked_sub(1).map(|i| &clips[i]);
        let pixmap = match layer_depth.checked_sub(1) {
            Some(i) => &mut layers[i],
            None => &mut **target,
        };
        handler.draw(CustomCanvas {
            pixmap: pixmap.as_mut(),
            transform,
            clip,
            bounds,
            data,
        });
    }
}

/// Device coordinates are clamped to ±2^24 pixels before rasterizing: tiny-skia's fixed-point
/// scan converter fails on paths spanning far larger ranges (RENDER-SOFT-24). Everything within
/// the target is unchanged; only geometry far outside it is bent.
const MAX_DEVICE_COORD: f32 = 16_777_216.0;

/// `path` in device pixels: transformed by `ts`, with points clamped to `MAX_DEVICE_COORD`.
/// `None` if the transform fails or nothing is left.
fn device_path(path: &Path, ts: Transform) -> Option<Path> {
    let path = path.clone().transform(ts)?;
    let b = path.bounds();
    let limit = MAX_DEVICE_COORD;
    if b.left() >= -limit && b.top() >= -limit && b.right() <= limit && b.bottom() <= limit {
        return Some(path);
    }
    let c = |p: tiny_skia::Point| (p.x.clamp(-limit, limit), p.y.clamp(-limit, limit));
    let mut pb = PathBuilder::new();
    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(p) => {
                let (x, y) = c(p);
                pb.move_to(x, y);
            }
            PathSegment::LineTo(p) => {
                let (x, y) = c(p);
                pb.line_to(x, y);
            }
            PathSegment::QuadTo(p1, p) => {
                let ((x1, y1), (x, y)) = (c(p1), c(p));
                pb.quad_to(x1, y1, x, y);
            }
            PathSegment::CubicTo(p1, p2, p) => {
                let ((x1, y1), (x2, y2), (x, y)) = (c(p1), c(p2), c(p));
                pb.cubic_to(x1, y1, x2, y2, x, y);
            }
            PathSegment::Close => pb.close(),
        }
    }
    pb.finish()
}

/// Premultiplies straight-alpha RGBA8 into a pixmap.
fn upload(image: &ImageData) -> Option<Pixmap> {
    let size = IntSize::from_wh(image.width(), image.height())?;
    let data: Vec<u8> = image
        .pixels()
        .chunks_exact(4)
        .flat_map(|p| {
            let c: PremultipliedColorU8 =
                tiny_skia::ColorU8::from_rgba(p[0], p[1], p[2], p[3]).premultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    Pixmap::from_vec(data, size)
}

fn solid_paint(color: Color) -> Paint<'static> {
    let c = color.clamp();
    let color = tiny_skia::Color::from_rgba(c.r, c.g, c.b, c.a).unwrap_or(tiny_skia::Color::BLACK);
    Paint {
        shader: Shader::SolidColor(color),
        ..Paint::default()
    }
}

fn color_is_finite(color: Color) -> bool {
    color.r.is_finite() && color.g.is_finite() && color.b.is_finite() && color.a.is_finite()
}

fn clip_is_finite(clip: &Clip) -> bool {
    match clip {
        Clip::Rect(rect) => rect.is_finite(),
        Clip::RoundedRect(shape) => rounded_rect_is_finite(shape),
    }
}

fn rounded_rect_is_finite(shape: &RoundedRect) -> bool {
    let r = shape.radii;
    shape.rect.is_finite()
        && r.top_left.is_finite()
        && r.top_right.is_finite()
        && r.bottom_right.is_finite()
        && r.bottom_left.is_finite()
}

/// The same check as `RenderReport::for_scene` (SCENE-RENDER-05): any non-finite value makes a
/// draw command invalid.
fn is_drawable(command: &Command, scene: &Scene) -> bool {
    match command {
        Command::Fill { shape, color } => rounded_rect_is_finite(shape) && color_is_finite(*color),
        Command::Stroke {
            shape,
            width,
            color,
        } => rounded_rect_is_finite(shape) && width.is_finite() && color_is_finite(*color),
        Command::BoxShadow(s) => {
            rounded_rect_is_finite(&s.shape)
                && color_is_finite(s.color)
                && s.offset.is_finite()
                && s.blur_radius.is_finite()
                && s.spread_radius.is_finite()
        }
        Command::Image(i) => {
            i.dest.is_finite() && i.src.is_none_or(Rect::is_finite) && i.opacity.is_finite()
        }
        Command::GlyphRun(run) => {
            run.font_size.is_finite()
                && color_is_finite(run.color)
                && run.origin.is_finite()
                && scene
                    .glyphs(run)
                    .iter()
                    .all(|g| g.x.is_finite() && g.y.is_finite())
        }
        Command::Custom(c) => c.bounds.is_finite(),
        _ => true,
    }
}

/// Negative radii become 0, then all radii are scaled down by one factor until adjacent radii
/// fit their side (the CSS rule; RENDER-SOFT-10).
fn clamp_radii(shape: &RoundedRect) -> RoundedRect {
    let rect = shape.rect;
    let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
    let r = shape.radii;
    let [tl, tr, br, bl] =
        [r.top_left, r.top_right, r.bottom_right, r.bottom_left].map(|v| v.max(0.0));
    let mut factor = 1.0f32;
    for (side, sum) in [(w, tl + tr), (w, bl + br), (h, tl + bl), (h, tr + br)] {
        if sum > side && sum > 0.0 {
            factor = factor.min(side / sum);
        }
    }
    RoundedRect::new(
        rect,
        tantu_scene::BorderRadius {
            top_left: tl * factor,
            top_right: tr * factor,
            bottom_right: br * factor,
            bottom_left: bl * factor,
        },
    )
}

/// The path of a (rounded) rect, or `None` if it is empty or reversed.
fn rounded_rect_path(shape: &RoundedRect) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, &clamp_radii(shape));
    pb.finish()
}

/// Appends a closed (rounded) rect to `pb`; nothing for an empty or reversed rect. Radii must
/// already be clamped.
fn push_rounded_rect(pb: &mut PathBuilder, shape: &RoundedRect) {
    // Control-point distance for a quarter circle drawn with one cubic.
    const K: f32 = 0.552_284_8;
    let Rect {
        left: l,
        top: t,
        right: r,
        bottom: b,
    } = shape.rect;
    if !(r > l && b > t) {
        return;
    }
    let rad = shape.radii;
    let (tl, tr, br, bl) = (
        rad.top_left,
        rad.top_right,
        rad.bottom_right,
        rad.bottom_left,
    );
    pb.move_to(l + tl, t);
    pb.line_to(r - tr, t);
    if tr > 0.0 {
        pb.cubic_to(r - tr + tr * K, t, r, t + tr - tr * K, r, t + tr);
    }
    pb.line_to(r, b - br);
    if br > 0.0 {
        pb.cubic_to(r, b - br + br * K, r - br + br * K, b, r - br, b);
    }
    pb.line_to(l + bl, b);
    if bl > 0.0 {
        pb.cubic_to(l + bl - bl * K, b, l, b - bl + bl * K, l, b - bl);
    }
    pb.line_to(l, t + tl);
    if tl > 0.0 {
        pb.cubic_to(l, t + tl - tl * K, l + tl - tl * K, t, l + tl, t);
    }
    pb.close();
}
