//! GPU text renderer
//!
//! Renders text using a glyph atlas and instanced quads.

use crate::atlas::{GlyphAtlas, GlyphKey};
use cosmic_text::{Buffer, SwashCache, SwashContent};
use wgpu::util::DeviceExt;

/// Vertex for text rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl TextVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Per-glyph instance data
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphInstance {
    /// Position and size: [x, y, width, height]
    rect: [f32; 4],
    /// UV coordinates: [u_min, v_min, u_max, v_max]
    uv: [f32; 4],
    /// Color: [r, g, b, a]
    color: [f32; 4],
}

impl GlyphInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        2 => Float32x4,  // rect
        3 => Float32x4,  // uv
        4 => Float32x4,  // color
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GlyphInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Uniform buffer for text rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TextUniforms {
    viewport_size: [f32; 2],
    _padding: [f32; 2],
}

/// GPU text renderer
pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    atlas: GlyphAtlas,
    instances: Vec<GlyphInstance>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // Create glyph atlas (1024x1024 should be plenty for most use cases)
        let atlas = GlyphAtlas::new(device, 1024, 1024);

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(TEXT_SHADER.into()),
        });

        // Uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Uniforms"),
            contents: bytemuck::cast_slice(&[TextUniforms {
                viewport_size: [800.0, 600.0],
                _padding: [0.0, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Uniform bind group layout
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Uniform bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &atlas.bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[TextVertex::desc(), GlyphInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Vertex buffer (unit quad)
        let vertices = [
            TextVertex { position: [0.0, 0.0], uv: [0.0, 0.0] },
            TextVertex { position: [1.0, 0.0], uv: [1.0, 0.0] },
            TextVertex { position: [1.0, 1.0], uv: [1.0, 1.0] },
            TextVertex { position: [0.0, 0.0], uv: [0.0, 0.0] },
            TextVertex { position: [1.0, 1.0], uv: [1.0, 1.0] },
            TextVertex { position: [0.0, 1.0], uv: [0.0, 1.0] },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            uniform_bind_group,
            atlas,
            instances: Vec::new(),
        }
    }

    /// Set the viewport size
    pub fn set_viewport(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[TextUniforms {
                viewport_size: [width, height],
                _padding: [0.0, 0.0],
            }]),
        );
    }

    /// Begin a new frame
    pub fn begin(&mut self) {
        self.instances.clear();
    }

    /// Draw text from a cosmic-text buffer
    pub fn draw_buffer(
        &mut self,
        buffer: &Buffer,
        x: f32,
        y: f32,
        color: [f32; 4],
        font_system: &mut cosmic_text::FontSystem,
        swash_cache: &mut SwashCache,
    ) {
        let font_size_64 = (buffer.metrics().font_size * 64.0) as u32;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((x, y), 1.0);

                let glyph_key = GlyphKey {
                    font_id: physical_glyph.cache_key.font_id,
                    glyph_id: physical_glyph.cache_key.glyph_id,
                    font_size_64,
                };

                // Check if glyph is in atlas
                let glyph_info = if let Some(info) = self.atlas.get(&glyph_key) {
                    *info
                } else {
                    // Rasterize glyph using swash
                    let image = swash_cache.get_image_uncached(font_system, physical_glyph.cache_key);

                    if let Some(image) = image {
                        if image.placement.width > 0 && image.placement.height > 0 {
                            // Convert image data based on content type
                            let alpha_data: Vec<u8> = match image.content {
                                SwashContent::Mask => image.data.clone(),
                                SwashContent::SubpixelMask | SwashContent::Color => {
                                    // For color/subpixel, extract alpha or convert to grayscale
                                    image.data.chunks(4).map(|pixel| pixel.get(3).copied().unwrap_or(255)).collect()
                                }
                            };

                            if let Some(info) = self.atlas.insert(
                                glyph_key,
                                &alpha_data,
                                image.placement.width,
                                image.placement.height,
                                image.placement.left,
                                image.placement.top,
                            ) {
                                info
                            } else {
                                // Atlas full, skip glyph
                                continue;
                            }
                        } else {
                            // Empty glyph
                            if let Some(info) = self.atlas.insert(glyph_key, &[], 0, 0, 0, 0) {
                                info
                            } else {
                                continue;
                            }
                        }
                    } else {
                        // No image, skip
                        continue;
                    }
                };

                // Skip empty glyphs (like spaces)
                if glyph_info.width == 0 || glyph_info.height == 0 {
                    continue;
                }

                // Calculate glyph position
                let glyph_x = physical_glyph.x as f32 + glyph_info.offset_x as f32;
                let glyph_y = physical_glyph.y as f32 - glyph_info.offset_y as f32;

                self.instances.push(GlyphInstance {
                    rect: [
                        glyph_x,
                        glyph_y,
                        glyph_info.width as f32,
                        glyph_info.height as f32,
                    ],
                    uv: [
                        glyph_info.uv_min[0],
                        glyph_info.uv_min[1],
                        glyph_info.uv_max[0],
                        glyph_info.uv_max[1],
                    ],
                    color,
                });
            }
        }
    }

    /// Draw text at a position with default styling
    pub fn draw_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        color: [f32; 4],
        font_system: &mut cosmic_text::FontSystem,
        swash_cache: &mut SwashCache,
    ) {
        let metrics = cosmic_text::Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_text(
            font_system,
            text,
            cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Advanced,
        );
        buffer.shape_until_scroll(font_system, false);

        self.draw_buffer(&buffer, x, y, color, font_system, swash_cache);
    }

    /// Render all queued text
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        if self.instances.is_empty() {
            return;
        }

        // Upload atlas if needed
        self.atlas.upload(queue);

        // Create instance buffer
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Text Instance Buffer"),
            contents: bytemuck::cast_slice(&self.instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Text Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.atlas.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instances.len() as u32);
    }
}

/// WGSL shader for text rendering
const TEXT_SHADER: &str = r#"
struct Uniforms {
    viewport_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var glyph_texture: texture_2d<f32>;
@group(1) @binding(1)
var glyph_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct InstanceInput {
    @location(2) rect: vec4<f32>,      // x, y, width, height
    @location(3) uv_rect: vec4<f32>,   // u_min, v_min, u_max, v_max
    @location(4) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform vertex position to pixel coordinates
    let pixel_pos = instance.rect.xy + vertex.position * instance.rect.zw;

    // Convert to NDC (-1 to 1)
    let ndc = (pixel_pos / uniforms.viewport_size) * 2.0 - 1.0;

    // Flip Y axis (screen coordinates have Y going down)
    out.clip_position = vec4<f32>(ndc.x, -ndc.y, 0.0, 1.0);

    // Interpolate UV coordinates
    out.uv = mix(instance.uv_rect.xy, instance.uv_rect.zw, vertex.uv);
    out.color = instance.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(glyph_texture, glyph_sampler, in.uv).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
"#;
