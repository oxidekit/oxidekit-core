//! OxideKit GPU Renderer
//!
//! GPU-accelerated rendering using wgpu.

pub use wgpu;

mod primitives;
mod color;

pub use primitives::{Primitive, PrimitiveRenderer, Rect, RoundedRect};
pub use color::Color;

use wgpu::{
    Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration, SurfaceTexture, TextureView,
};

/// GPU rendering context
pub struct RenderContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl RenderContext {
    /// Create a new render context
    pub async fn new() -> Self {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("Failed to create device");

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    /// Create a surface for a window
    pub fn create_surface<'w>(
        &self,
        target: impl Into<wgpu::SurfaceTarget<'w>>,
    ) -> Surface<'w> {
        self.instance
            .create_surface(target)
            .expect("Failed to create surface")
    }

    /// Configure a surface for rendering
    pub fn configure_surface(&self, surface: &Surface, width: u32, height: u32) -> SurfaceConfiguration {
        let caps = surface.get_capabilities(&self.adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&self.device, &config);
        config
    }

    /// Create a primitive renderer for this context
    pub fn create_primitive_renderer(&self, format: wgpu::TextureFormat) -> PrimitiveRenderer {
        PrimitiveRenderer::new(&self.device, format)
    }
}

/// A frame ready for rendering
pub struct Frame<'a> {
    pub texture: SurfaceTexture,
    pub view: TextureView,
    pub device: &'a Device,
    pub queue: &'a Queue,
}

impl<'a> Frame<'a> {
    /// Create a new frame from a surface texture
    pub fn new(texture: SurfaceTexture, device: &'a Device, queue: &'a Queue) -> Self {
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            device,
            queue,
        }
    }

    /// Clear the frame with a color
    pub fn clear(&self, r: f64, g: f64, b: f64, a: f64) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Clear Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Present the frame
    pub fn present(self) {
        self.texture.present();
    }
}
