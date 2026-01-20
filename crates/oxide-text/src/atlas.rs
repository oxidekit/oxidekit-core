//! Glyph atlas for efficient text rendering
//!
//! Stores rasterized glyphs in a GPU texture for batch rendering.

use std::collections::HashMap;

/// Unique identifier for a glyph in the atlas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: cosmic_text::fontdb::ID,
    /// Glyph ID within the font
    pub glyph_id: u16,
    /// Font size in 1/64 points (for sub-pixel precision)
    pub font_size_64: u32,
}

/// Information about a glyph in the atlas
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// UV coordinates in the atlas (normalized 0-1)
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    /// Size of the glyph in pixels
    pub width: u32,
    pub height: u32,
    /// Offset from baseline
    pub offset_x: i32,
    pub offset_y: i32,
}

/// Simple rectangle packer using shelf algorithm
struct ShelfPacker {
    width: u32,
    height: u32,
    current_x: u32,
    current_y: u32,
    shelf_height: u32,
}

impl ShelfPacker {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            current_x: 0,
            current_y: 0,
            shelf_height: 0,
        }
    }

    fn pack(&mut self, glyph_width: u32, glyph_height: u32) -> Option<(u32, u32)> {
        // Add 1px padding
        let w = glyph_width + 1;
        let h = glyph_height + 1;

        // Check if we need a new shelf
        if self.current_x + w > self.width {
            self.current_x = 0;
            self.current_y += self.shelf_height;
            self.shelf_height = 0;
        }

        // Check if we have room
        if self.current_y + h > self.height {
            return None;
        }

        let pos = (self.current_x, self.current_y);
        self.current_x += w;
        self.shelf_height = self.shelf_height.max(h);

        Some(pos)
    }
}

/// Glyph atlas storing rasterized glyphs in a GPU texture
pub struct GlyphAtlas {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub sampler: wgpu::Sampler,
    glyphs: HashMap<GlyphKey, GlyphInfo>,
    packer: ShelfPacker,
    data: Vec<u8>,
    width: u32,
    height: u32,
    dirty: bool,
}

impl GlyphAtlas {
    /// Create a new glyph atlas
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glyph Atlas Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph Atlas Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            texture_view,
            bind_group_layout,
            bind_group,
            sampler,
            glyphs: HashMap::new(),
            packer: ShelfPacker::new(width, height),
            data: vec![0u8; (width * height) as usize],
            width,
            height,
            dirty: false,
        }
    }

    /// Get glyph info if already in atlas
    pub fn get(&self, key: &GlyphKey) -> Option<&GlyphInfo> {
        self.glyphs.get(key)
    }

    /// Insert a glyph into the atlas
    /// Returns the glyph info if successful, None if atlas is full
    pub fn insert(
        &mut self,
        key: GlyphKey,
        glyph_data: &[u8],
        glyph_width: u32,
        glyph_height: u32,
        offset_x: i32,
        offset_y: i32,
    ) -> Option<GlyphInfo> {
        if glyph_width == 0 || glyph_height == 0 {
            // Empty glyph (e.g., space)
            let info = GlyphInfo {
                uv_min: [0.0, 0.0],
                uv_max: [0.0, 0.0],
                width: 0,
                height: 0,
                offset_x,
                offset_y,
            };
            self.glyphs.insert(key, info);
            return Some(info);
        }

        let (x, y) = self.packer.pack(glyph_width, glyph_height)?;

        // Copy glyph data into atlas
        for row in 0..glyph_height {
            let src_start = (row * glyph_width) as usize;
            let src_end = src_start + glyph_width as usize;
            let dst_start = ((y + row) * self.width + x) as usize;

            if src_end <= glyph_data.len() && dst_start + glyph_width as usize <= self.data.len() {
                self.data[dst_start..dst_start + glyph_width as usize]
                    .copy_from_slice(&glyph_data[src_start..src_end]);
            }
        }

        self.dirty = true;

        let info = GlyphInfo {
            uv_min: [x as f32 / self.width as f32, y as f32 / self.height as f32],
            uv_max: [
                (x + glyph_width) as f32 / self.width as f32,
                (y + glyph_height) as f32 / self.height as f32,
            ],
            width: glyph_width,
            height: glyph_height,
            offset_x,
            offset_y,
        };

        self.glyphs.insert(key, info);
        Some(info)
    }

    /// Upload atlas data to GPU if dirty
    pub fn upload(&mut self, queue: &wgpu::Queue) {
        if !self.dirty {
            return;
        }

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.width),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.dirty = false;
    }
}
