//! Primitives renderer for rectangles, rounded rectangles, and borders

use crate::Color;
use wgpu::util::DeviceExt;

/// A basic rectangle
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// A rounded rectangle with uniform corner radius
#[derive(Debug, Clone, Copy)]
pub struct RoundedRect {
    pub rect: Rect,
    pub radius: f32,
}

impl RoundedRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            radius,
        }
    }
}

/// A primitive to render
#[derive(Debug, Clone)]
pub enum Primitive {
    /// Solid color rectangle
    Rect {
        rect: Rect,
        color: Color,
    },
    /// Rounded rectangle
    RoundedRect {
        rect: RoundedRect,
        color: Color,
    },
    /// Rectangle with border
    Border {
        rect: Rect,
        color: Color,
        width: f32,
        radius: f32,
    },
}

/// Vertex for primitive rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Per-instance data for batched rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    /// Position and size: [x, y, width, height]
    rect: [f32; 4],
    /// Color: [r, g, b, a]
    color: [f32; 4],
    /// Border radius (all corners)
    radius: f32,
    /// Border width (0 = filled)
    border_width: f32,
    /// Padding for alignment
    _padding: [f32; 2],
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        2 => Float32x4,  // rect
        3 => Float32x4,  // color
        4 => Float32,    // radius
        5 => Float32,    // border_width
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Uniform buffer for viewport info
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    viewport_size: [f32; 2],
    _padding: [f32; 2],
}

/// Renderer for primitive shapes
pub struct PrimitiveRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    instances: Vec<Instance>,
}

impl PrimitiveRenderer {
    /// Create a new primitive renderer
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Primitive Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        // Uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Primitive Uniforms"),
            contents: bytemuck::cast_slice(&[Uniforms {
                viewport_size: [800.0, 600.0],
                _padding: [0.0, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Primitive Bind Group Layout"),
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

        // Bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Primitive Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Primitive Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Primitive Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Instance::desc()],
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
            Vertex { position: [0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { position: [1.0, 0.0], uv: [1.0, 0.0] },
            Vertex { position: [1.0, 1.0], uv: [1.0, 1.0] },
            Vertex { position: [0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { position: [1.0, 1.0], uv: [1.0, 1.0] },
            Vertex { position: [0.0, 1.0], uv: [0.0, 1.0] },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Primitive Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            uniform_bind_group,
            instances: Vec::new(),
        }
    }

    /// Set the viewport size
    pub fn set_viewport(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms {
                viewport_size: [width, height],
                _padding: [0.0, 0.0],
            }]),
        );
    }

    /// Begin a new frame, clearing all primitives
    pub fn begin(&mut self) {
        self.instances.clear();
    }

    /// Add a primitive to render
    pub fn draw(&mut self, primitive: Primitive) {
        let instance = match primitive {
            Primitive::Rect { rect, color } => Instance {
                rect: [rect.x, rect.y, rect.width, rect.height],
                color: color.to_array(),
                radius: 0.0,
                border_width: 0.0,
                _padding: [0.0, 0.0],
            },
            Primitive::RoundedRect { rect, color } => Instance {
                rect: [rect.rect.x, rect.rect.y, rect.rect.width, rect.rect.height],
                color: color.to_array(),
                radius: rect.radius,
                border_width: 0.0,
                _padding: [0.0, 0.0],
            },
            Primitive::Border { rect, color, width, radius } => Instance {
                rect: [rect.x, rect.y, rect.width, rect.height],
                color: color.to_array(),
                radius,
                border_width: width,
                _padding: [0.0, 0.0],
            },
        };
        self.instances.push(instance);
    }

    /// Draw a filled rectangle
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.draw(Primitive::Rect {
            rect: Rect::new(x, y, width, height),
            color,
        });
    }

    /// Draw a rounded rectangle
    pub fn rounded_rect(&mut self, x: f32, y: f32, width: f32, height: f32, radius: f32, color: Color) {
        self.draw(Primitive::RoundedRect {
            rect: RoundedRect::new(x, y, width, height, radius),
            color,
        });
    }

    /// Draw a border
    pub fn border(&mut self, x: f32, y: f32, width: f32, height: f32, border_width: f32, radius: f32, color: Color) {
        self.draw(Primitive::Border {
            rect: Rect::new(x, y, width, height),
            color,
            width: border_width,
            radius,
        });
    }

    /// Render all queued primitives
    pub fn render<'a>(
        &'a self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
    ) {
        if self.instances.is_empty() {
            return;
        }

        // Create instance buffer
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Primitive Instance Buffer"),
            contents: bytemuck::cast_slice(&self.instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let load_op = match clear_color {
            Some(color) => wgpu::LoadOp::Clear(color),
            None => wgpu::LoadOp::Load,
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Primitive Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instances.len() as u32);
    }
}

/// WGSL shader for primitive rendering
const SHADER: &str = r#"
struct Uniforms {
    viewport_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct InstanceInput {
    @location(2) rect: vec4<f32>,      // x, y, width, height
    @location(3) color: vec4<f32>,
    @location(4) radius: f32,
    @location(5) border_width: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) border_width: f32,
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
    out.color = instance.color;
    out.uv = vertex.uv;
    out.size = instance.rect.zw;
    out.radius = instance.radius;
    out.border_width = instance.border_width;

    return out;
}

// Signed distance function for rounded rectangle
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + r;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to pixel coordinates relative to center
    let half_size = in.size * 0.5;
    let p = (in.uv - 0.5) * in.size;

    // Clamp radius to half of the smallest dimension
    let max_radius = min(half_size.x, half_size.y);
    let radius = min(in.radius, max_radius);

    // Calculate signed distance
    let d = sd_rounded_box(p, half_size, radius);

    // Anti-aliased edge
    let aa = 1.0;
    var alpha = 1.0 - smoothstep(-aa, aa, d);

    // Handle border
    if in.border_width > 0.0 {
        let inner_d = sd_rounded_box(p, half_size - in.border_width, max(radius - in.border_width, 0.0));
        let inner_alpha = 1.0 - smoothstep(-aa, aa, inner_d);
        alpha = alpha - inner_alpha;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.width, 100.0);
    }
}
