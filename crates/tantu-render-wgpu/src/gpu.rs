//! Device setup, pipelines and the GPU-side resources the renderer reuses between frames.

use std::sync::{Arc, Mutex};

use crate::renderer::CreateError;

/// Format of offscreen targets, layer textures and image textures (premultiplied values).
pub(crate) const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Format of clip masks.
pub(crate) const MASK_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

/// Floats per shape instance: transform (6), rect (4), radii (4), color (4), params (4).
pub(crate) const SHAPE_FLOATS: usize = 22;
/// Floats per image instance: transform (6), dest (4), uv (4), opacity and margin (2).
pub(crate) const IMAGE_FLOATS: usize = 16;
/// Floats per full-screen instance: four vec4 parameters.
pub(crate) const FULL_FLOATS: usize = 16;
/// Floats per glyph instance: device rect (4), atlas rect (4), premultiplied color (4).
pub(crate) const GLYPH_FLOATS: usize = 12;
/// Format of the glyph atlas (coverage).
pub(crate) const ATLAS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

/// The adapter, device and queue, and errors wgpu reported instead of panicking.
pub(crate) struct Device {
    /// Kept alive for as long as the surfaces and adapter created from it.
    _instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) errors: Arc<Mutex<Vec<String>>>,
}

impl Device {
    /// Finds an adapter (a real one, else a software fallback) and creates a device.
    pub(crate) fn new(
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Device, CreateError> {
        let request = |force_fallback_adapter| {
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter,
                compatible_surface: surface,
                apply_limit_buckets: false,
            }))
        };
        let adapter = request(false)
            .or_else(|_| request(true))
            .map_err(|_| CreateError::NoAdapter)?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("tantu"),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .map_err(|e| CreateError::Backend(Box::new(e)))?;
        // Validation errors are reported by `render` instead of panicking (RENDER-WGPU-17).
        let errors = Arc::new(Mutex::new(Vec::new()));
        let sink = errors.clone();
        device.on_uncaptured_error(Arc::new(move |e: wgpu::Error| {
            if let Ok(mut errors) = sink.lock() {
                errors.push(e.to_string());
            }
        }));
        Ok(Device {
            _instance: instance,
            adapter,
            device,
            queue,
            errors,
        })
    }

    /// Errors reported since the last call.
    pub(crate) fn take_errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .map(|mut e| std::mem::take(&mut *e))
            .unwrap_or_default()
    }
}

/// Render pipelines for one color format.
pub(crate) struct ColorPipelines {
    pub(crate) shape: wgpu::RenderPipeline,
    pub(crate) image: wgpu::RenderPipeline,
    pub(crate) glyph: wgpu::RenderPipeline,
    pub(crate) overlay: wgpu::RenderPipeline,
    pub(crate) composite: wgpu::RenderPipeline,
}

/// Pipelines, layouts and samplers, created once per renderer.
pub(crate) struct Pipelines {
    pub(crate) globals_layout: wgpu::BindGroupLayout,
    pub(crate) pixel_layout: wgpu::BindGroupLayout,
    pub(crate) image_layout: wgpu::BindGroupLayout,
    pub(crate) nearest: wgpu::Sampler,
    pub(crate) linear: wgpu::Sampler,
    /// For the target format, then (if different) for `COLOR_FORMAT` (layers).
    pub(crate) color: Vec<(wgpu::TextureFormat, ColorPipelines)>,
    pub(crate) clip: wgpu::RenderPipeline,
}

const PREMULTIPLIED_OVER: wgpu::BlendState = wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING;

/// dst.rgb = src.rgb · dst.a + dst.rgb · (1 − src.a); dst.a unchanged.
const SOURCE_ATOP: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::DstAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    },
};

/// dst = dst · src (the clip mask times a new clip's coverage).
const MULTIPLY: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::Src,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::SrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
};

fn instance_layout(
    attributes: &[wgpu::VertexAttribute],
    floats: usize,
) -> wgpu::VertexBufferLayout<'_> {
    wgpu::VertexBufferLayout {
        array_stride: (floats * 4) as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes,
    }
}

const SHAPE_ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
    0 => Float32x4, 1 => Float32x2, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4
];
const IMAGE_ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
    0 => Float32x4, 1 => Float32x2, 2 => Float32x4, 3 => Float32x4, 4 => Float32x2
];
const GLYPH_ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4];
const FULL_ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
    0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4
];

impl Pipelines {
    pub(crate) fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Pipelines {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tantu shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });
        let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tantu globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let pixel_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tantu pixel texture"),
            entries: &[texture_entry(0)],
        });
        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tantu image"),
            entries: &[
                texture_entry(0),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = |filter| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("tantu sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                mag_filter: filter,
                min_filter: filter,
                ..Default::default()
            })
        };
        let nearest = sampler(wgpu::FilterMode::Nearest);
        let linear = sampler(wgpu::FilterMode::Linear);

        let layout = |groups: &[Option<&wgpu::BindGroupLayout>]| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("tantu layout"),
                bind_group_layouts: groups,
                immediate_size: 0,
            })
        };
        let shape_layout = layout(&[Some(&globals_layout), Some(&pixel_layout)]);
        let image_pipeline_layout = layout(&[
            Some(&globals_layout),
            Some(&pixel_layout),
            Some(&image_layout),
        ]);
        let full_layout = layout(&[Some(&globals_layout)]);
        let composite_layout = layout(&[Some(&globals_layout), Some(&pixel_layout)]);

        let pipeline = |layout: &wgpu::PipelineLayout,
                        vs: &str,
                        fs: &str,
                        buffer: wgpu::VertexBufferLayout<'_>,
                        format: wgpu::TextureFormat,
                        blend: wgpu::BlendState| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(fs),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers: &[Some(buffer)],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let color_pipelines = |format| ColorPipelines {
            shape: pipeline(
                &shape_layout,
                "vs_shape",
                "fs_shape",
                instance_layout(&SHAPE_ATTRIBUTES, SHAPE_FLOATS),
                format,
                PREMULTIPLIED_OVER,
            ),
            image: pipeline(
                &image_pipeline_layout,
                "vs_image",
                "fs_image",
                instance_layout(&IMAGE_ATTRIBUTES, IMAGE_FLOATS),
                format,
                PREMULTIPLIED_OVER,
            ),
            // The atlas goes in the image slot (group 2), read with textureLoad.
            glyph: pipeline(
                &image_pipeline_layout,
                "vs_glyph",
                "fs_glyph",
                instance_layout(&GLYPH_ATTRIBUTES, GLYPH_FLOATS),
                format,
                PREMULTIPLIED_OVER,
            ),
            overlay: pipeline(
                &full_layout,
                "vs_full",
                "fs_overlay",
                instance_layout(&FULL_ATTRIBUTES, FULL_FLOATS),
                format,
                SOURCE_ATOP,
            ),
            composite: pipeline(
                &composite_layout,
                "vs_full",
                "fs_composite",
                instance_layout(&FULL_ATTRIBUTES, FULL_FLOATS),
                format,
                PREMULTIPLIED_OVER,
            ),
        };
        let mut color = vec![(target_format, color_pipelines(target_format))];
        if target_format != COLOR_FORMAT {
            color.push((COLOR_FORMAT, color_pipelines(COLOR_FORMAT)));
        }
        let clip = pipeline(
            &full_layout,
            "vs_full",
            "fs_clip",
            instance_layout(&FULL_ATTRIBUTES, FULL_FLOATS),
            MASK_FORMAT,
            MULTIPLY,
        );
        Pipelines {
            globals_layout,
            pixel_layout,
            image_layout,
            nearest,
            linear,
            color,
            clip,
        }
    }

    /// The pipelines for `format`.
    pub(crate) fn for_format(&self, format: wgpu::TextureFormat) -> &ColorPipelines {
        self.color
            .iter()
            .find(|(f, _)| *f == format)
            .map(|(_, p)| p)
            .unwrap_or(&self.color[0].1)
    }
}

/// A target-sized texture with its view and a bind group reading it by pixel position.
pub(crate) struct PixelTexture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl PixelTexture {
    pub(crate) fn new(
        device: &wgpu::Device,
        pipelines: &Pipelines,
        format: wgpu::TextureFormat,
        (width, height): (u32, u32),
        label: &str,
    ) -> PixelTexture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &pipelines.pixel_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });
        PixelTexture {
            texture,
            view,
            bind_group,
        }
    }
}

/// A vertex buffer that grows to fit the frame's instances.
pub(crate) struct InstanceBuffer {
    pub(crate) buffer: Option<wgpu::Buffer>,
    label: &'static str,
}

impl InstanceBuffer {
    pub(crate) const fn new(label: &'static str) -> InstanceBuffer {
        InstanceBuffer {
            buffer: None,
            label,
        }
    }

    /// Uploads `data`, growing the buffer if needed.
    pub(crate) fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[f32]) {
        if data.is_empty() {
            return;
        }
        let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_le_bytes()).collect();
        let size = bytes.len() as u64;
        if self.buffer.as_ref().is_none_or(|b| b.size() < size) {
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: size.next_power_of_two(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, &bytes);
        }
    }
}
