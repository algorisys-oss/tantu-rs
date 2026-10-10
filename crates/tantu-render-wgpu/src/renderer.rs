//! [`WgpuRenderer`] and the custom-command handler API.
//!
//! A frame is rendered in two steps: walking the Scene builds a list of GPU operations and the
//! instance data they draw (`Plan`), then the operations are encoded in order into one command
//! buffer.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use tantu_core::{Affine, Color, Rect};
use tantu_scene::{
    BorderRadius, BoxShadow, Clip, Command, CustomKind, FontData, FontId, Glyph, GlyphRun,
    ImageData, ImageDraw, ImageId, ImageSampling, Layer, RenderError, RenderReport, Renderer,
    Resources, RoundedRect, Scene,
};
use tantu_text::{GlyphMask, GlyphRasterizer};

use crate::gpu::{
    ATLAS_FORMAT, COLOR_FORMAT, Device, FULL_FLOATS, GLYPH_FLOATS, IMAGE_FLOATS, InstanceBuffer,
    MASK_FORMAT, Pipelines, PixelTexture, SHAPE_FLOATS,
};

/// Why a renderer couldn't be created.
#[derive(Debug)]
#[non_exhaustive]
pub enum CreateError {
    /// No GPU adapter fits (no driver, or none supports what Tantu needs).
    NoAdapter,
    /// Creating the device or the surface failed.
    Backend(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for CreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CreateError::NoAdapter => f.write_str("no GPU adapter available"),
            CreateError::Backend(e) => write!(f, "creating the wgpu renderer failed: {e}"),
        }
    }
}

impl std::error::Error for CreateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CreateError::Backend(e) => Some(e.as_ref()),
            CreateError::NoAdapter => None,
        }
    }
}

/// Draws one kind of custom command with wgpu.
pub trait CustomHandler {
    /// Record drawing for `canvas.data` into `canvas.target`, inside `canvas.bounds`.
    fn draw(&mut self, canvas: CustomCanvas<'_>);
}

/// What a custom handler draws into.
pub struct CustomCanvas<'a> {
    /// The renderer's device.
    pub device: &'a wgpu::Device,
    /// The renderer's queue.
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
    /// The bounding box of the current clip in target pixels (`[x, y, width, height]`), for a
    /// scissor rect; `None` with no open clip. Rounded and rotated clips are approximated by
    /// their bounding box here.
    pub clip: Option<[u32; 4]>,
    /// The command's bounds, in local coordinates.
    pub bounds: Rect,
    /// The command's bytes (`Scene::custom_data`).
    pub data: &'a [u8],
}

/// Where frames go.
enum Target {
    /// An offscreen texture; `None` while the size is 0.
    Offscreen(Option<PixelTexture>),
    /// A window surface.
    Surface {
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    },
}

/// A window's display handle, for the wgpu instance (which wants `Debug`).
struct Display<W>(std::sync::Arc<W>);

impl<W> fmt::Debug for Display<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Display")
    }
}

impl<W: raw_window_handle::HasDisplayHandle> raw_window_handle::HasDisplayHandle for Display<W> {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.0.display_handle()
    }
}

/// An uploaded image and a bind group per sampling mode.
struct ImageEntry {
    nearest: wgpu::BindGroup,
    linear: wgpu::BindGroup,
}

/// A renderer that draws Scenes with wgpu.
pub struct WgpuRenderer {
    width: u32,
    height: u32,
    scale_factor: f32,
    gpu: Device,
    pipelines: Pipelines,
    format: wgpu::TextureFormat,
    target: Target,
    globals: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    /// The "no clip" mask: all white, target-sized.
    full_mask: Option<PixelTexture>,
    /// Clip masks by depth (`masks[0]` is depth 1).
    masks: Vec<PixelTexture>,
    /// Layer textures by depth.
    layers: Vec<PixelTexture>,
    images: HashMap<ImageId, ImageEntry>,
    images_revision: Option<u64>,
    handlers: HashMap<CustomKind, Box<dyn CustomHandler>>,
    shape_buffer: InstanceBuffer,
    image_buffer: InstanceBuffer,
    full_buffer: InstanceBuffer,
    glyph_buffer: InstanceBuffer,
    /// Glyph coverage masks (shared code with the software renderer).
    glyphs: GlyphRasterizer,
    /// The glyph atlas, created on first use.
    atlas: Option<Atlas>,
}

/// Starting side of the glyph atlas, in texels.
const ATLAS_START: u32 = 1024;
/// Largest atlas side the renderer grows to (or the device's limit, if lower).
const ATLAS_MAX: u32 = 8192;

/// A shelf-packed `R8Unorm` texture of glyph masks (RENDER-WGPU-18, -19).
struct Atlas {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    side: u32,
    /// Next free position on the current shelf, and the shelf's height so far.
    x: u32,
    y: u32,
    shelf: u32,
    /// Where each mask is, by its `Arc` address; `held` keeps those addresses from being reused.
    slots: HashMap<usize, [u32; 4]>,
    held: Vec<Arc<GlyphMask>>,
}

/// What finding a glyph in the atlas gave.
enum GlyphSlot {
    /// The glyph draws nothing (no outline, unusable size).
    Nothing,
    /// The atlas is full.
    Full,
    /// The mask and its atlas rect `[left, top, right, bottom]`.
    At(Arc<GlyphMask>, [u32; 4]),
}

impl Atlas {
    fn new(device: &wgpu::Device, pipelines: &Pipelines, side: u32) -> Atlas {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tantu glyph atlas"),
            size: wgpu::Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ATLAS_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tantu glyph atlas"),
            layout: &pipelines.image_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&pipelines.nearest),
                },
            ],
        });
        Atlas {
            texture,
            bind_group,
            side,
            x: 0,
            y: 0,
            shelf: 0,
            slots: HashMap::new(),
            held: Vec::new(),
        }
    }

    /// Forgets every mask (the texture's old contents are overwritten as masks come back).
    fn clear(&mut self) {
        (self.x, self.y, self.shelf) = (0, 0, 0);
        self.slots.clear();
        self.held.clear();
    }

    /// The atlas rect of `mask`, uploading it if new; `None` if it doesn't fit.
    fn insert(&mut self, queue: &wgpu::Queue, mask: &Arc<GlyphMask>) -> Option<[u32; 4]> {
        let key = Arc::as_ptr(mask) as usize;
        if let Some(rect) = self.slots.get(&key) {
            return Some(*rect);
        }
        // One texel of padding keeps neighbours apart.
        let (w, h) = (mask.width + 1, mask.height + 1);
        if w > self.side || h > self.side {
            return None;
        }
        if self.x + w > self.side {
            self.y += self.shelf;
            (self.x, self.shelf) = (0, 0);
        }
        if self.y + h > self.side {
            return None;
        }
        let (x, y) = (self.x, self.y);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &mask.coverage,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(mask.width),
                rows_per_image: Some(mask.height),
            },
            wgpu::Extent3d {
                width: mask.width,
                height: mask.height,
                depth_or_array_layers: 1,
            },
        );
        self.x += w;
        self.shelf = self.shelf.max(h);
        let rect = [x, y, x + mask.width, y + mask.height];
        self.slots.insert(key, rect);
        self.held.push(Arc::clone(mask));
        Some(rect)
    }
}

impl WgpuRenderer {
    /// The glyph's mask and atlas rect, creating the atlas on first use.
    fn glyph_slot(
        &mut self,
        font: FontId,
        data: &FontData,
        glyph: u32,
        size: f32,
        x: f32,
    ) -> GlyphSlot {
        let Some(mask) = self.glyphs.mask(font, data, glyph, size, x) else {
            return GlyphSlot::Nothing;
        };
        let atlas = self
            .atlas
            .get_or_insert_with(|| Atlas::new(&self.gpu.device, &self.pipelines, ATLAS_START));
        match atlas.insert(&self.gpu.queue, &mask) {
            Some(rect) => GlyphSlot::At(mask, rect),
            None => GlyphSlot::Full,
        }
    }

    /// Makes room after a frame overflowed the atlas: clear it the first time, then grow it.
    /// Returns false when it can't grow any more.
    fn make_atlas_room(&mut self, first: bool) -> bool {
        let Some(atlas) = self.atlas.as_mut() else {
            return false;
        };
        if first {
            atlas.clear();
            return true;
        }
        let limit = self
            .gpu
            .device
            .limits()
            .max_texture_dimension_2d
            .min(ATLAS_MAX);
        if atlas.side >= limit {
            atlas.clear();
            return false;
        }
        let side = (atlas.side * 2).min(limit);
        self.atlas = Some(Atlas::new(&self.gpu.device, &self.pipelines, side));
        true
    }

    /// An offscreen `width × height` target (RGBA8, premultiplied alpha), scale factor 1, fully
    /// transparent. Blocks while wgpu finds an adapter and creates a device.
    pub fn new_offscreen(width: u32, height: u32) -> Result<WgpuRenderer, CreateError> {
        // `WGPU_BACKEND` and wgpu's other environment variables apply.
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let gpu = Device::new(instance, None)?;
        let mut renderer = WgpuRenderer::with_target(gpu, COLOR_FORMAT, Target::Offscreen(None));
        renderer.resize(width, height, 1.0);
        Ok(renderer)
    }

    /// A renderer drawing into `window`'s surface, `width × height` physical pixels.
    pub fn for_window<W>(window: W, width: u32, height: u32) -> Result<WgpuRenderer, CreateError>
    where
        W: raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle
            + Send
            + Sync
            + 'static,
    {
        // The GL backend needs the display; `WGPU_BACKEND` and wgpu's other variables apply.
        let window = std::sync::Arc::new(window);
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
                Box::new(Display(window.clone())),
            ));
        let surface = instance
            .create_surface(window)
            .map_err(|e| CreateError::Backend(Box::new(e)))?;
        let gpu = Device::new(instance, Some(&surface))?;
        let caps = surface.get_capabilities(&gpu.adapter);
        // Blend on sRGB-encoded values: prefer a format without hardware sRGB conversion.
        let format = [
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
        ]
        .into_iter()
        .find(|f| caps.formats.contains(f))
        .or_else(|| caps.formats.first().copied())
        .ok_or_else(|| CreateError::Backend("the surface supports no formats".into()))?;
        let alpha_mode = [
            wgpu::CompositeAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::Opaque,
        ]
        .into_iter()
        .find(|m| caps.alpha_modes.contains(m))
        .or_else(|| caps.alpha_modes.first().copied())
        .unwrap_or(wgpu::CompositeAlphaMode::Auto);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        let mut renderer =
            WgpuRenderer::with_target(gpu, format, Target::Surface { surface, config });
        renderer.resize(width, height, 1.0);
        Ok(renderer)
    }

    fn with_target(gpu: Device, format: wgpu::TextureFormat, target: Target) -> WgpuRenderer {
        let pipelines = Pipelines::new(&gpu.device, format);
        let globals = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tantu globals"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tantu globals"),
            layout: &pipelines.globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });
        WgpuRenderer {
            width: 0,
            height: 0,
            scale_factor: 1.0,
            gpu,
            pipelines,
            format,
            target,
            globals,
            globals_bind_group,
            full_mask: None,
            masks: Vec::new(),
            layers: Vec::new(),
            images: HashMap::new(),
            images_revision: None,
            handlers: HashMap::new(),
            shape_buffer: InstanceBuffer::new("tantu shapes"),
            image_buffer: InstanceBuffer::new("tantu images"),
            full_buffer: InstanceBuffer::new("tantu full-screen"),
            glyph_buffer: InstanceBuffer::new("tantu glyphs"),
            glyphs: GlyphRasterizer::new(),
            atlas: None,
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

    /// An offscreen target's pixels as straight-alpha RGBA8, read back from the GPU. `None` for
    /// a window renderer or a 0-width or 0-height target.
    pub fn snapshot(&self) -> Option<ImageData> {
        let Target::Offscreen(Some(target)) = &self.target else {
            return None;
        };
        let (w, h) = (self.width, self.height);
        let row = w * 4;
        let padded =
            row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let device = &self.gpu.device;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tantu snapshot"),
            size: padded as u64 * h as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.gpu.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = tx.send(result);
        });
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        rx.recv().ok()?.ok()?;
        let mapped = buffer.get_mapped_range(..).ok()?;
        let mut pixels = Vec::with_capacity((row * h) as usize);
        for y in 0..h as usize {
            let start = y * padded as usize;
            for p in mapped[start..start + row as usize].chunks_exact(4) {
                pixels.extend_from_slice(&demultiply([p[0], p[1], p[2], p[3]]));
            }
        }
        drop(mapped);
        buffer.unmap();
        ImageData::rgba8(w, h, pixels).ok()
    }

    /// Information about the adapter in use (name, backend), for logs and test output.
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.gpu.adapter.get_info()
    }

    /// Drops cached images whose handle is no longer in `resources`.
    fn prune_images(&mut self, resources: &Resources) {
        if self.images_revision != Some(resources.revision()) {
            self.images.retain(|id, _| resources.image(*id).is_some());
            self.images_revision = Some(resources.revision());
        }
    }

    /// Uploads `image` if it isn't cached yet. False if it can't be (larger than the device's
    /// texture limit).
    fn upload_image(&mut self, id: ImageId, image: &ImageData) -> bool {
        if self.images.contains_key(&id) {
            return true;
        }
        let max = self.gpu.device.limits().max_texture_dimension_2d;
        if image.width() > max || image.height() > max {
            return false;
        }
        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        let texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tantu image"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: COLOR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let premultiplied: Vec<u8> = image
            .pixels()
            .chunks_exact(4)
            .flat_map(|p| premultiply([p[0], p[1], p[2], p[3]]))
            .collect();
        self.gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &premultiplied,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width() * 4),
                rows_per_image: Some(image.height()),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = |sampler: &wgpu::Sampler| {
            self.gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("tantu image"),
                    layout: &self.pipelines.image_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                })
        };
        let entry = ImageEntry {
            nearest: bind_group(&self.pipelines.nearest),
            linear: bind_group(&self.pipelines.linear),
        };
        self.images.insert(id, entry);
        true
    }

    /// Makes sure the full mask and enough clip masks and layers exist at the current size.
    fn ensure_textures(&mut self, mask_depth: usize, layer_depth: usize) {
        let size = (self.width, self.height);
        let device = &self.gpu.device;
        let pipelines = &self.pipelines;
        if self.full_mask.is_none() {
            self.full_mask = Some(PixelTexture::new(
                device,
                pipelines,
                MASK_FORMAT,
                size,
                "tantu full mask",
            ));
        }
        while self.masks.len() < mask_depth {
            self.masks.push(PixelTexture::new(
                device,
                pipelines,
                MASK_FORMAT,
                size,
                "tantu clip mask",
            ));
        }
        while self.layers.len() < layer_depth {
            self.layers.push(PixelTexture::new(
                device,
                pipelines,
                COLOR_FORMAT,
                size,
                "tantu layer",
            ));
        }
    }
}

impl Renderer for WgpuRenderer {
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        self.width = width;
        self.height = height;
        self.full_mask = None;
        self.masks.clear();
        self.layers.clear();
        let nonzero = width > 0 && height > 0;
        match &mut self.target {
            Target::Offscreen(texture) => {
                *texture = nonzero.then(|| {
                    PixelTexture::new(
                        &self.gpu.device,
                        &self.pipelines,
                        COLOR_FORMAT,
                        (width, height),
                        "tantu target",
                    )
                });
            }
            Target::Surface { surface, config } => {
                if nonzero {
                    config.width = width;
                    config.height = height;
                    surface.configure(&self.gpu.device, config);
                }
            }
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

        // Step 1: plan. If the glyphs overflow the atlas, make room and plan again
        // (RENDER-WGPU-19): clear it once, then grow it.
        let mut attempt = 0;
        let plan = loop {
            let mut attempt_report = report;
            let mut plan = Plan::new(self.scale_factor, (self.width, self.height));
            plan.walk(scene, resources, self, &mut attempt_report);
            if !plan.atlas_full || attempt >= 4 || !self.make_atlas_room(attempt == 0) {
                if plan.atlas_full {
                    tracing::warn!("glyph atlas full; some glyphs were not drawn");
                }
                report = attempt_report;
                break plan;
            }
            attempt += 1;
        };

        // Step 2: encode.
        self.ensure_textures(plan.max_mask_depth, plan.max_layer_depth);
        let viewport = [self.width as f32, self.height as f32, 0.0, 0.0];
        let globals: Vec<u8> = viewport.iter().flat_map(|f| f.to_le_bytes()).collect();
        let (device, queue) = (&self.gpu.device, &self.gpu.queue);
        queue.write_buffer(&self.globals, 0, &globals);
        self.shape_buffer.upload(device, queue, &plan.shapes);
        self.image_buffer.upload(device, queue, &plan.images);
        self.full_buffer.upload(device, queue, &plan.full);
        self.glyph_buffer.upload(device, queue, &plan.glyphs);

        let frame = match &self.target {
            Target::Offscreen(_) => None,
            Target::Surface { surface, config } => match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame)
                | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Some(frame),
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    surface.configure(device, config);
                    return Err(RenderError::TargetLost);
                }
                wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                    return Ok(report);
                }
                wgpu::CurrentSurfaceTexture::Validation => {
                    return Err(RenderError::Backend(
                        "acquiring the surface texture failed".into(),
                    ));
                }
            },
        };
        let frame_view = frame.as_ref().map(|f| {
            f.texture
                .create_view(&wgpu::TextureViewDescriptor::default())
        });
        let main_view = match (&self.target, &frame_view) {
            (Target::Offscreen(Some(texture)), _) => &texture.view,
            (_, Some(view)) => view,
            _ => return Ok(report),
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("tantu frame"),
        });
        let Some(full_mask) = &self.full_mask else {
            return Err(RenderError::OutOfMemory);
        };
        clear(&mut encoder, main_view, wgpu::Color::TRANSPARENT);
        clear(&mut encoder, &full_mask.view, wgpu::Color::WHITE);

        let encode = Encode {
            pipelines: &self.pipelines,
            globals: &self.globals_bind_group,
            main_view,
            main_format: self.format,
            full_mask,
            masks: &self.masks,
            layers: &self.layers,
            images: &self.images,
            shape_buffer: self.shape_buffer.buffer.as_ref(),
            image_buffer: self.image_buffer.buffer.as_ref(),
            full_buffer: self.full_buffer.buffer.as_ref(),
            glyph_buffer: self.glyph_buffer.buffer.as_ref(),
            atlas: self.atlas.as_ref().map(|a| &a.bind_group),
            size: (self.width, self.height),
        };
        for op in &plan.ops {
            encode.op(&mut encoder, op, device, queue, &mut self.handlers);
        }
        queue.submit([encoder.finish()]);
        if let Some(frame) = frame {
            queue.present(frame);
        }
        let errors = self.gpu.take_errors();
        if !errors.is_empty() {
            return Err(RenderError::Backend(errors.join("; ").into()));
        }
        Ok(report)
    }
}

impl fmt::Debug for WgpuRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WgpuRenderer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale_factor", &self.scale_factor)
            .field("format", &self.format)
            .field("custom_kinds", &self.handlers.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

// ---- Planning ------------------------------------------------------------------------------

/// What draws go into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Surface {
    Main,
    Layer(usize),
}

/// One GPU operation, in paint order.
enum Op<'s> {
    /// Clear a layer texture to transparent.
    ClearLayer(usize),
    /// Draw shape instances `start..end` with mask `mask` (0 = no clip).
    Shapes {
        target: Surface,
        mask: usize,
        start: u32,
        end: u32,
    },
    /// Draw glyph instances `start..end` from the atlas with mask `mask`.
    Glyphs {
        target: Surface,
        mask: usize,
        start: u32,
        end: u32,
    },
    /// Draw one image instance.
    Image {
        target: Surface,
        mask: usize,
        instance: u32,
        image: ImageId,
        linear: bool,
    },
    /// Build clip mask `depth` (≥ 1) from its parent and full-screen instance `instance`.
    Clip { depth: usize, instance: u32 },
    /// Apply an overlay color to a layer.
    Overlay { layer: usize, instance: u32 },
    /// Composite a layer onto the surface below it.
    Composite {
        layer: usize,
        dst: Surface,
        instance: u32,
    },
    /// Call a custom handler.
    Custom {
        kind: CustomKind,
        target: Surface,
        transform: [f32; 6],
        clip: Option<[u32; 4]>,
        bounds: Rect,
        data: &'s [u8],
    },
}

/// Something pushed and not yet popped.
#[derive(Clone, Copy)]
enum Scope {
    Transform,
    Clip,
    Layer(Layer),
}

/// The frame's operations and instance data.
struct Plan<'s> {
    scale: f32,
    size: (u32, u32),
    ops: Vec<Op<'s>>,
    shapes: Vec<f32>,
    images: Vec<f32>,
    full: Vec<f32>,
    glyphs: Vec<f32>,
    /// Some glyph didn't fit in the atlas.
    atlas_full: bool,
    transform: Affine,
    transforms: Vec<Affine>,
    scopes: Vec<Scope>,
    clip_depth: usize,
    layer_depth: usize,
    /// Device-space bounding box of the open clips, per clip depth (index 0: no clip).
    clip_boxes: Vec<Option<Rect>>,
    max_mask_depth: usize,
    max_layer_depth: usize,
}

impl<'s> Plan<'s> {
    fn new(scale: f32, size: (u32, u32)) -> Plan<'s> {
        Plan {
            scale,
            size,
            ops: Vec::new(),
            shapes: Vec::new(),
            images: Vec::new(),
            full: Vec::new(),
            glyphs: Vec::new(),
            atlas_full: false,
            transform: Affine::IDENTITY,
            transforms: Vec::new(),
            scopes: Vec::new(),
            clip_depth: 0,
            layer_depth: 0,
            clip_boxes: vec![None],
            max_mask_depth: 0,
            max_layer_depth: 0,
        }
    }

    fn walk(
        &mut self,
        scene: &'s Scene,
        resources: &Resources,
        renderer: &mut WgpuRenderer,
        report: &mut RenderReport,
    ) {
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
                Command::PushTransform(t) => {
                    self.transforms.push(self.transform);
                    self.transform = self.transform * *t;
                    self.scopes.push(Scope::Transform);
                }
                Command::PushClip(clip) => self.push_clip(clip),
                Command::PushLayer(layer) => {
                    self.ops.push(Op::ClearLayer(self.layer_depth));
                    self.layer_depth += 1;
                    self.max_layer_depth = self.max_layer_depth.max(self.layer_depth);
                    self.scopes.push(Scope::Layer(*layer));
                }
                Command::PopClip | Command::PopTransform | Command::PopLayer => self.pop(),
                draw if !is_drawable(draw, scene) => {}
                Command::Fill { shape, color } => self.fill(shape, *color),
                Command::Stroke {
                    shape,
                    width,
                    color,
                } => self.stroke(shape, *width, *color),
                Command::BoxShadow(shadow) => self.box_shadow(shadow),
                Command::Image(image) => {
                    if let Some(data) = resources.image(image.image) {
                        if renderer.upload_image(image.image, data) {
                            self.image(image, data);
                        } else {
                            report.missing_images = report.missing_images.saturating_add(1);
                        }
                    }
                }
                Command::GlyphRun(run) => {
                    // A missing font was counted by `for_scene`; an unreadable one is ours
                    // (RENDER-WGPU-04).
                    if let Some(font) = resources.font(run.font) {
                        if renderer.glyphs.readable(run.font, font) {
                            self.glyph_run(run, scene.glyphs(run), font, renderer);
                        } else {
                            report.missing_fonts = report.missing_fonts.saturating_add(1);
                        }
                    }
                }
                Command::Custom(custom) => {
                    if renderer.handlers.contains_key(&custom.kind) {
                        self.ops.push(Op::Custom {
                            kind: custom.kind,
                            target: self.surface(),
                            transform: self.device_transform().coeffs(),
                            clip: self.clip_box(),
                            bounds: custom.bounds,
                            data: scene.custom_data(custom),
                        });
                    }
                }
            }
        }
        while !self.scopes.is_empty() {
            self.pop();
        }
    }

    /// Adds a glyph run's glyphs as atlas instances (RENDER-WGPU-18, -19): each mask at device
    /// size, placed at its device position, batched with the previous glyphs when possible.
    fn glyph_run(
        &mut self,
        run: &GlyphRun,
        glyphs: &[Glyph],
        font: &FontData,
        renderer: &mut WgpuRenderer,
    ) {
        let [a, b, c, d, e, f] = self.device_transform().coeffs();
        let size = run.font_size * (a * d - b * c).abs().sqrt();
        if !(size.is_finite() && size > 0.0) {
            return;
        }
        let color = premultiplied(run.color);
        let (width, height) = (self.size.0 as f32, self.size.1 as f32);
        for glyph in glyphs {
            let (lx, ly) = (run.origin.x + glyph.x, run.origin.y + glyph.y);
            let (x, y) = (a * lx + c * ly + e, b * lx + d * ly + f);
            if !(x.abs() < 16_777_216.0 && y.abs() < 16_777_216.0) {
                continue;
            }
            let (mask, rect) = match renderer.glyph_slot(run.font, font, glyph.id, size, x) {
                GlyphSlot::At(mask, rect) => (mask, rect),
                GlyphSlot::Nothing => continue,
                GlyphSlot::Full => {
                    self.atlas_full = true;
                    continue;
                }
            };
            let left = x.floor() + mask.left as f32;
            let top = y.round() + mask.top as f32;
            let (right, bottom) = (left + mask.width as f32, top + mask.height as f32);
            if left >= width || top >= height || right <= 0.0 || bottom <= 0.0 {
                continue;
            }
            let start = (self.glyphs.len() / GLYPH_FLOATS) as u32;
            self.glyphs.extend_from_slice(&[
                left,
                top,
                right,
                bottom,
                rect[0] as f32,
                rect[1] as f32,
                rect[2] as f32,
                rect[3] as f32,
                color[0],
                color[1],
                color[2],
                color[3],
            ]);
            let (target, mask_depth) = (self.surface(), self.clip_depth);
            if let Some(Op::Glyphs {
                target: t,
                mask: m,
                end,
                ..
            }) = self.ops.last_mut()
            {
                if *t == target && *m == mask_depth && *end == start {
                    *end = start + 1;
                    continue;
                }
            }
            self.ops.push(Op::Glyphs {
                target,
                mask: mask_depth,
                start,
                end: start + 1,
            });
        }
    }

    fn surface(&self) -> Surface {
        match self.layer_depth.checked_sub(1) {
            Some(i) => Surface::Layer(i),
            None => Surface::Main,
        }
    }

    fn device_transform(&self) -> Affine {
        Affine::scale(self.scale) * self.transform
    }

    /// The current clip's bounding box as an integer scissor rect.
    fn clip_box(&self) -> Option<[u32; 4]> {
        let b = self.clip_boxes.get(self.clip_depth).copied().flatten()?;
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let l = b.left.clamp(0.0, w).floor();
        let t = b.top.clamp(0.0, h).floor();
        let r = b.right.clamp(0.0, w).ceil();
        let bt = b.bottom.clamp(0.0, h).ceil();
        Some([
            l as u32,
            t as u32,
            (r - l).max(0.0) as u32,
            (bt - t).max(0.0) as u32,
        ])
    }

    fn push_clip(&mut self, clip: &Clip) {
        let shape = clamp_radii(&match *clip {
            Clip::Rect(rect) => RoundedRect::from_rect(rect),
            Clip::RoundedRect(shape) => shape,
        });
        let ts = self.device_transform();
        // Device to local; a degenerate transform clips everything away (an empty rect).
        let (inverse, rect) = match ts.inverse() {
            Some(inv) => (inv, shape.rect),
            None => (Affine::IDENTITY, Rect::ZERO),
        };
        let [a, b, c, d, e, f] = inverse.coeffs();
        let instance = (self.full.len() / FULL_FLOATS) as u32;
        let r = shape.radii;
        self.full.extend_from_slice(&[
            a,
            b,
            c,
            d,
            e,
            f,
            0.0,
            0.0,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            r.top_left,
            r.top_right,
            r.bottom_right,
            r.bottom_left,
        ]);
        let parent = self.clip_boxes.get(self.clip_depth).copied().flatten();
        let own = ts.transform_rect_bbox(rect);
        let bbox = match parent {
            Some(p) => p.intersect(own).unwrap_or(Rect::ZERO),
            None => own,
        };
        self.clip_depth += 1;
        self.max_mask_depth = self.max_mask_depth.max(self.clip_depth);
        self.clip_boxes.truncate(self.clip_depth);
        self.clip_boxes.push(Some(bbox));
        self.ops.push(Op::Clip {
            depth: self.clip_depth,
            instance,
        });
        self.scopes.push(Scope::Clip);
    }

    fn pop(&mut self) {
        match self.scopes.pop() {
            Some(Scope::Transform) => {
                self.transform = self.transforms.pop().unwrap_or(Affine::IDENTITY);
            }
            Some(Scope::Clip) => {
                self.clip_depth = self.clip_depth.saturating_sub(1);
                self.clip_boxes.truncate(self.clip_depth + 1);
                // A later clip at this depth rebuilds the mask; nothing to do on the GPU.
            }
            Some(Scope::Layer(layer)) => self.pop_layer(layer),
            None => {}
        }
    }

    fn pop_layer(&mut self, layer: Layer) {
        let Some(index) = self.layer_depth.checked_sub(1) else {
            return;
        };
        self.layer_depth = index;
        if let Some(overlay) = layer.overlay_color.filter(|c| color_is_finite(*c)) {
            let instance = (self.full.len() / FULL_FLOATS) as u32;
            let mut params = [0.0; FULL_FLOATS];
            params[..4].copy_from_slice(&premultiplied(overlay));
            self.full.extend_from_slice(&params);
            self.ops.push(Op::Overlay {
                layer: index,
                instance,
            });
        }
        let opacity = if layer.opacity.is_nan() {
            0.0
        } else {
            layer.opacity.clamp(0.0, 1.0)
        };
        if opacity <= 0.0 {
            return;
        }
        let instance = (self.full.len() / FULL_FLOATS) as u32;
        let mut params = [0.0; FULL_FLOATS];
        params[0] = opacity;
        self.full.extend_from_slice(&params);
        self.ops.push(Op::Composite {
            layer: index,
            dst: self.surface(),
            instance,
        });
    }

    /// Local units per device pixel, for anti-aliasing margins; `None` if the transform
    /// collapses everything (nothing to draw).
    fn pixel_size(&self) -> Option<f32> {
        let [a, b, c, d, ..] = self.device_transform().coeffs();
        let scale = (a * d - b * c).abs().sqrt();
        let size = 1.0 / scale;
        (scale > 0.0 && size.is_finite()).then_some(size)
    }

    fn push_shape(
        &mut self,
        kind: f32,
        shape: &RoundedRect,
        color: Color,
        param: f32,
        margin: f32,
    ) {
        let [a, b, c, d, e, f] = self.device_transform().coeffs();
        let r = shape.rect;
        let radii = shape.radii;
        let start = (self.shapes.len() / SHAPE_FLOATS) as u32;
        let [cr, cg, cb, ca] = premultiplied(color);
        self.shapes.extend_from_slice(&[
            a,
            b,
            c,
            d,
            e,
            f,
            r.left,
            r.top,
            r.right,
            r.bottom,
            radii.top_left,
            radii.top_right,
            radii.bottom_right,
            radii.bottom_left,
            cr,
            cg,
            cb,
            ca,
            kind,
            param,
            margin,
            0.0,
        ]);
        let (target, mask) = (self.surface(), self.clip_depth);
        if let Some(Op::Shapes {
            target: t,
            mask: m,
            end,
            ..
        }) = self.ops.last_mut()
        {
            if *t == target && *m == mask && *end == start {
                *end = start + 1;
                return;
            }
        }
        self.ops.push(Op::Shapes {
            target,
            mask,
            start,
            end: start + 1,
        });
    }

    fn fill(&mut self, shape: &RoundedRect, color: Color) {
        let r = shape.rect;
        if !(r.right > r.left && r.bottom > r.top) || color.clamp().is_transparent() {
            return;
        }
        let Some(px) = self.pixel_size() else { return };
        self.push_shape(0.0, &clamp_radii(shape), color, 0.0, 2.0 * px);
    }

    fn stroke(&mut self, shape: &RoundedRect, width: f32, color: Color) {
        let r = shape.rect;
        if width <= 0.0 || !(r.right > r.left && r.bottom > r.top) || color.clamp().is_transparent()
        {
            return;
        }
        let shorter = (r.right - r.left).min(r.bottom - r.top);
        if width >= shorter / 2.0 {
            self.fill(shape, color);
            return;
        }
        let Some(px) = self.pixel_size() else { return };
        self.push_shape(1.0, &clamp_radii(shape), color, width, 2.0 * px);
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
        let (dx, dy) = (shadow.offset.x, shadow.offset.y);
        let shape = RoundedRect::new(
            Rect::from_ltrb(
                r.left - spread + dx,
                r.top - spread + dy,
                r.right + spread + dx,
                r.bottom + spread + dy,
            ),
            BorderRadius {
                top_left: grow(radii.top_left),
                top_right: grow(radii.top_right),
                bottom_right: grow(radii.bottom_right),
                bottom_left: grow(radii.bottom_left),
            },
        );
        if shadow.blur_radius <= 0.0 {
            self.fill(&shape, shadow.color);
            return;
        }
        let s = shape.rect;
        if !(s.right > s.left && s.bottom > s.top) {
            return;
        }
        let Some(px) = self.pixel_size() else { return };
        // Flutter's sigma, in local units (the GPU blurs in local space, which matches blurring
        // in device space with sigma scaled by the transform).
        let sigma = shadow.blur_radius * 0.57735 + 0.5;
        if !sigma.is_finite() {
            return;
        }
        let margin = (3.0 * sigma + 2.0 * px).min(1e7);
        self.push_shape(2.0, &clamp_radii(&shape), shadow.color, sigma, margin);
    }

    fn image(&mut self, draw: &ImageDraw, data: &ImageData) {
        let opacity = draw.opacity.clamp(0.0, 1.0);
        if opacity <= 0.0 {
            return;
        }
        let (iw, ih) = (data.width() as f32, data.height() as f32);
        let src = draw.src.unwrap_or(Rect::from_ltwh(0.0, 0.0, iw, ih));
        let dest = draw.dest;
        if src.is_empty() || dest.is_empty() {
            return;
        }
        let Some(px) = self.pixel_size() else { return };
        let [a, b, c, d, e, f] = self.device_transform().coeffs();
        let instance = (self.images.len() / IMAGE_FLOATS) as u32;
        self.images.extend_from_slice(&[
            a,
            b,
            c,
            d,
            e,
            f,
            dest.left,
            dest.top,
            dest.right,
            dest.bottom,
            src.left / iw,
            src.top / ih,
            src.right / iw,
            src.bottom / ih,
            opacity,
            2.0 * px,
        ]);
        self.ops.push(Op::Image {
            target: self.surface(),
            mask: self.clip_depth,
            instance,
            image: draw.image,
            linear: draw.sampling == ImageSampling::Linear,
        });
    }
}

// ---- Encoding ------------------------------------------------------------------------------

/// Everything `Op`s are encoded with.
struct Encode<'a> {
    pipelines: &'a Pipelines,
    globals: &'a wgpu::BindGroup,
    main_view: &'a wgpu::TextureView,
    main_format: wgpu::TextureFormat,
    full_mask: &'a PixelTexture,
    masks: &'a [PixelTexture],
    layers: &'a [PixelTexture],
    images: &'a HashMap<ImageId, ImageEntry>,
    shape_buffer: Option<&'a wgpu::Buffer>,
    image_buffer: Option<&'a wgpu::Buffer>,
    full_buffer: Option<&'a wgpu::Buffer>,
    glyph_buffer: Option<&'a wgpu::Buffer>,
    atlas: Option<&'a wgpu::BindGroup>,
    size: (u32, u32),
}

/// A render pass on `view` that first clears it to `color`.
fn clear(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, color: wgpu::Color) {
    let _pass = begin(encoder, view, wgpu::LoadOp::Clear(color));
}

fn begin<'e>(
    encoder: &'e mut wgpu::CommandEncoder,
    view: &'e wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
) -> wgpu::RenderPass<'e> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("tantu pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

impl Encode<'_> {
    fn view(&self, surface: Surface) -> (&wgpu::TextureView, wgpu::TextureFormat) {
        match surface {
            Surface::Main => (self.main_view, self.main_format),
            Surface::Layer(i) => (&self.layers[i].view, COLOR_FORMAT),
        }
    }

    fn mask(&self, depth: usize) -> &PixelTexture {
        match depth.checked_sub(1) {
            Some(i) => &self.masks[i],
            None => self.full_mask,
        }
    }

    fn op(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        op: &Op<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        handlers: &mut HashMap<CustomKind, Box<dyn CustomHandler>>,
    ) {
        match *op {
            Op::ClearLayer(i) => clear(encoder, &self.layers[i].view, wgpu::Color::TRANSPARENT),
            Op::Shapes {
                target,
                mask,
                start,
                end,
            } => {
                let Some(buffer) = self.shape_buffer else {
                    return;
                };
                let (view, format) = self.view(target);
                let mut pass = begin(encoder, view, wgpu::LoadOp::Load);
                pass.set_pipeline(&self.pipelines.for_format(format).shape);
                pass.set_bind_group(0, self.globals, &[]);
                pass.set_bind_group(1, &self.mask(mask).bind_group, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..6, start..end);
            }
            Op::Glyphs {
                target,
                mask,
                start,
                end,
            } => {
                let (Some(buffer), Some(atlas)) = (self.glyph_buffer, self.atlas) else {
                    return;
                };
                let (view, format) = self.view(target);
                let mut pass = begin(encoder, view, wgpu::LoadOp::Load);
                pass.set_pipeline(&self.pipelines.for_format(format).glyph);
                pass.set_bind_group(0, self.globals, &[]);
                pass.set_bind_group(1, &self.mask(mask).bind_group, &[]);
                pass.set_bind_group(2, atlas, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..6, start..end);
            }
            Op::Image {
                target,
                mask,
                instance,
                image,
                linear,
            } => {
                let (Some(buffer), Some(entry)) = (self.image_buffer, self.images.get(&image))
                else {
                    return;
                };
                let (view, format) = self.view(target);
                let mut pass = begin(encoder, view, wgpu::LoadOp::Load);
                pass.set_pipeline(&self.pipelines.for_format(format).image);
                pass.set_bind_group(0, self.globals, &[]);
                pass.set_bind_group(1, &self.mask(mask).bind_group, &[]);
                pass.set_bind_group(
                    2,
                    if linear {
                        &entry.linear
                    } else {
                        &entry.nearest
                    },
                    &[],
                );
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..6, instance..instance + 1);
            }
            Op::Clip { depth, instance } => {
                let Some(buffer) = self.full_buffer else {
                    return;
                };
                let parent = self.mask(depth - 1);
                let mask = self.mask(depth);
                encoder.copy_texture_to_texture(
                    parent.texture.as_image_copy(),
                    mask.texture.as_image_copy(),
                    wgpu::Extent3d {
                        width: self.size.0,
                        height: self.size.1,
                        depth_or_array_layers: 1,
                    },
                );
                let mut pass = begin(encoder, &mask.view, wgpu::LoadOp::Load);
                pass.set_pipeline(&self.pipelines.clip);
                pass.set_bind_group(0, self.globals, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..3, instance..instance + 1);
            }
            Op::Overlay { layer, instance } => {
                let Some(buffer) = self.full_buffer else {
                    return;
                };
                let mut pass = begin(encoder, &self.layers[layer].view, wgpu::LoadOp::Load);
                pass.set_pipeline(&self.pipelines.for_format(COLOR_FORMAT).overlay);
                pass.set_bind_group(0, self.globals, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..3, instance..instance + 1);
            }
            Op::Composite {
                layer,
                dst,
                instance,
            } => {
                let Some(buffer) = self.full_buffer else {
                    return;
                };
                let (view, format) = self.view(dst);
                let mut pass = begin(encoder, view, wgpu::LoadOp::Load);
                pass.set_pipeline(&self.pipelines.for_format(format).composite);
                pass.set_bind_group(0, self.globals, &[]);
                pass.set_bind_group(1, &self.layers[layer].bind_group, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..3, instance..instance + 1);
            }
            Op::Custom {
                kind,
                target,
                transform,
                clip,
                bounds,
                data,
            } => {
                let Some(handler) = handlers.get_mut(&kind) else {
                    return;
                };
                let (view, format) = self.view(target);
                handler.draw(CustomCanvas {
                    device,
                    queue,
                    encoder,
                    target: view,
                    format,
                    target_size: self.size,
                    transform,
                    clip,
                    bounds,
                    data,
                });
            }
        }
    }
}

// ---- Helpers -------------------------------------------------------------------------------

fn premultiplied(color: Color) -> [f32; 4] {
    let c = color.clamp();
    [c.r * c.a, c.g * c.a, c.b * c.a, c.a]
}

fn premultiply(p: [u8; 4]) -> [u8; 4] {
    let a = p[3] as u32;
    let m = |c: u8| ((c as u32 * a + 127) / 255) as u8;
    [m(p[0]), m(p[1]), m(p[2]), p[3]]
}

fn demultiply(p: [u8; 4]) -> [u8; 4] {
    let a = p[3] as u32;
    if a == 0 {
        return [0, 0, 0, 0];
    }
    let m = |c: u8| ((c as u32 * 255 + a / 2) / a).min(255) as u8;
    [m(p[0]), m(p[1]), m(p[2]), p[3]]
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

/// The same check as `RenderReport::for_scene` (SCENE-RENDER-05).
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

/// Negative radii become 0, then all radii shrink by one factor until adjacent ones fit their
/// side (the CSS rule).
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
        BorderRadius {
            top_left: tl * factor,
            top_right: tr * factor,
            bottom_right: br * factor,
            bottom_left: bl * factor,
        },
    )
}
