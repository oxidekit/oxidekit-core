//! OxideKit Runtime
//!
//! Window management, event handling, and application lifecycle.

use anyhow::Result;
use oxide_compiler::{compile, ComponentIR, PropertyValue};
use oxide_layout::{AvailableSpace, LayoutTree, NodeId, NodeVisual, Size, StyleBuilder};
use oxide_render::{Color, PrimitiveRenderer, RenderContext};
use oxide_text::{TextRenderer, TextSystem};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wgpu::Surface;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

/// Application manifest loaded from oxide.toml
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub app: AppConfig,
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub dev: DevConfig,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CoreConfig {
    #[serde(default = "default_requires")]
    pub requires: String,
}

fn default_requires() -> String {
    ">=0.1.0".to_string()
}

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub min_width: Option<u32>,
    #[serde(default)]
    pub min_height: Option<u32>,
    #[serde(default = "default_true")]
    pub resizable: bool,
    #[serde(default = "default_true")]
    pub decorations: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: default_title(),
            width: default_width(),
            height: default_height(),
            min_width: None,
            min_height: None,
            resizable: true,
            decorations: true,
        }
    }
}

fn default_title() -> String {
    "OxideKit App".to_string()
}
fn default_width() -> u32 {
    1280
}
fn default_height() -> u32 {
    720
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Default)]
pub struct DevConfig {
    #[serde(default = "default_true")]
    pub hot_reload: bool,
    #[serde(default)]
    pub inspector: bool,
}

/// The main OxideKit application
pub struct Application {
    manifest: Manifest,
    project_path: PathBuf,
}

impl Application {
    /// Load application from an oxide.toml manifest
    pub fn from_manifest(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&content)?;

        // Get the project directory (parent of oxide.toml)
        let project_path = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        tracing::info!(
            "Loaded app: {} v{}",
            manifest.app.name,
            manifest.app.version
        );

        Ok(Self {
            manifest,
            project_path,
        })
    }

    /// Create a default application (for testing)
    pub fn default_app() -> Self {
        Self {
            manifest: Manifest {
                app: AppConfig {
                    id: "dev.oxidekit.default".to_string(),
                    name: "OxideKit App".to_string(),
                    version: "0.1.0".to_string(),
                    description: None,
                },
                core: CoreConfig::default(),
                window: WindowConfig::default(),
                dev: DevConfig::default(),
            },
            project_path: PathBuf::from("."),
        }
    }

    /// Run the application
    pub fn run(self) -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);

        // Try to compile the UI
        let ui_ir = self.compile_ui();

        let mut app_state = AppState::new(self.manifest, ui_ir);

        event_loop.run_app(&mut app_state)?;

        Ok(())
    }

    /// Compile the UI from .oui file
    fn compile_ui(&self) -> Option<ComponentIR> {
        let ui_path = self.project_path.join("ui/app.oui");
        if !ui_path.exists() {
            tracing::warn!("No ui/app.oui file found, using demo UI");
            return None;
        }

        match std::fs::read_to_string(&ui_path) {
            Ok(source) => match compile(&source) {
                Ok(ir) => {
                    tracing::info!("Compiled ui/app.oui successfully");
                    Some(ir)
                }
                Err(e) => {
                    tracing::error!("Failed to compile ui/app.oui: {}", e);
                    None
                }
            },
            Err(e) => {
                tracing::error!("Failed to read ui/app.oui: {}", e);
                None
            }
        }
    }
}

/// Text element to render
struct TextElement {
    node_id: NodeId,
    content: String,
    size: f32,
    color: [f32; 4],
}

/// Internal application state
struct AppState {
    manifest: Manifest,
    ui_ir: Option<ComponentIR>,
    window: Option<Arc<Window>>,
    render_ctx: Option<RenderContext>,
    surface: Option<Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    primitive_renderer: Option<PrimitiveRenderer>,
    text_renderer: Option<TextRenderer>,
    text_system: Option<TextSystem>,
    viewport_size: (u32, u32),
    layout_tree: Option<LayoutTree>,
    root_node: Option<NodeId>,
    text_elements: Vec<TextElement>,
}

impl AppState {
    fn new(manifest: Manifest, ui_ir: Option<ComponentIR>) -> Self {
        Self {
            manifest,
            ui_ir,
            window: None,
            render_ctx: None,
            surface: None,
            surface_config: None,
            primitive_renderer: None,
            text_renderer: None,
            text_system: None,
            viewport_size: (0, 0),
            layout_tree: None,
            root_node: None,
            text_elements: Vec::new(),
        }
    }

    fn build_ui(&mut self) {
        self.text_elements.clear();

        // Build from IR if available
        if let Some(ir) = self.ui_ir.clone() {
            let mut tree = LayoutTree::new();
            let root = build_from_ir(&ir, &mut tree, &mut self.text_elements);
            self.layout_tree = Some(tree);
            self.root_node = Some(root);
            return;
        }

        // Fallback: demo UI
        self.build_demo_ui();
    }

    fn build_demo_ui(&mut self) {
        let mut tree = LayoutTree::new();
        // Root container - full viewport, centered
        let root_style = StyleBuilder::new()
            .flex_column()
            .center()
            .size_full()
            .build();

        // Card container
        let card_style = StyleBuilder::new()
            .flex_column()
            .size(500.0, 300.0)
            .build();
        let card_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(16.0);

        // Title bar
        let title_bar_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(50.0)
            .padding_xy(20.0, 0.0)
            .gap(8.0)
            .build();
        let title_bar_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#111827"))
            .with_radius(16.0);

        // Window control dots
        let dot_style = StyleBuilder::new().size(12.0, 12.0).build();
        let red_dot = tree.new_visual_node(
            dot_style.clone(),
            NodeVisual::default()
                .with_background(hex_to_rgba("#EF4444"))
                .with_radius(6.0),
        );
        let yellow_dot = tree.new_visual_node(
            dot_style.clone(),
            NodeVisual::default()
                .with_background(hex_to_rgba("#F59E0B"))
                .with_radius(6.0),
        );
        let green_dot = tree.new_visual_node(
            dot_style,
            NodeVisual::default()
                .with_background(hex_to_rgba("#22C55E"))
                .with_radius(6.0),
        );

        let title_bar = tree.new_visual_node_with_children(
            title_bar_style,
            title_bar_visual,
            &[red_dot, yellow_dot, green_dot],
        );

        // Content area with colored boxes
        let content_style = StyleBuilder::new()
            .flex_row()
            .center()
            .flex_grow(1.0)
            .width_percent(1.0)
            .gap(20.0)
            .build();

        let box_style = StyleBuilder::new().size(60.0, 60.0).build();
        let colors = ["#EF4444", "#F59E0B", "#22C55E", "#3B82F6"];
        let boxes: Vec<_> = colors
            .iter()
            .map(|c| {
                tree.new_visual_node(
                    box_style.clone(),
                    NodeVisual::default()
                        .with_background(hex_to_rgba(c))
                        .with_radius(8.0),
                )
            })
            .collect();

        let content = tree.new_node_with_children(content_style, &boxes);

        // Button row
        let button_row_style = StyleBuilder::new()
            .flex_row()
            .center()
            .width_percent(1.0)
            .height(70.0)
            .gap(16.0)
            .build();

        let button_style = StyleBuilder::new().size(140.0, 44.0).build();
        let primary_btn = tree.new_visual_node(
            button_style.clone(),
            NodeVisual::default()
                .with_background(hex_to_rgba("#3B82F6"))
                .with_radius(8.0),
        );
        let secondary_btn = tree.new_visual_node(
            button_style,
            NodeVisual::default()
                .with_border(hex_to_rgba("#374151"), 1.0)
                .with_radius(8.0),
        );

        let button_row = tree.new_node_with_children(button_row_style, &[primary_btn, secondary_btn]);

        // Accent bar
        let accent_style = StyleBuilder::new()
            .width_percent(0.9)
            .height(2.0)
            .margin(4.0)
            .build();
        let accent_bar = tree.new_visual_node(
            accent_style,
            NodeVisual::default().with_background(hex_to_rgba("#3B82F6")),
        );

        // Assemble card
        let card = tree.new_visual_node_with_children(
            card_style,
            card_visual,
            &[title_bar, content, button_row, accent_bar],
        );

        // Root
        let root = tree.new_node_with_children(root_style, &[card]);

        self.layout_tree = Some(tree);
        self.root_node = Some(root);
    }

    fn compute_layout(&mut self) {
        if let (Some(tree), Some(root)) = (&mut self.layout_tree, self.root_node) {
            let (w, h) = self.viewport_size;
            tree.compute_layout(
                root,
                Size {
                    width: AvailableSpace::Definite(w as f32),
                    height: AvailableSpace::Definite(h as f32),
                },
            );
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.viewport_size = (width, height);

        if let (Some(ctx), Some(surface), Some(config)) =
            (&self.render_ctx, &self.surface, &mut self.surface_config)
        {
            config.width = width;
            config.height = height;
            surface.configure(&ctx.device, config);

            // Update primitive renderer viewport
            if let Some(renderer) = &self.primitive_renderer {
                renderer.set_viewport(&ctx.queue, width as f32, height as f32);
            }

            // Update text renderer viewport
            if let Some(renderer) = &self.text_renderer {
                renderer.set_viewport(&ctx.queue, width as f32, height as f32);
            }
        }

        // Recompute layout for new viewport size
        self.compute_layout();
    }

    fn render(&mut self) {
        let Some(ctx) = &self.render_ctx else {
            return;
        };
        let Some(surface) = &self.surface else {
            return;
        };

        let output = match surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                if let Some(config) = &self.surface_config {
                    surface.configure(&ctx.device, config);
                }
                return;
            }
            Err(e) => {
                tracing::error!("Surface error: {}", e);
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin rendering primitives
        if let Some(renderer) = &mut self.primitive_renderer {
            renderer.begin();

            // Render UI from layout tree
            if let (Some(tree), Some(root)) = (&self.layout_tree, self.root_node) {
                render_layout_tree(tree, root, renderer);
            }
        }

        // Create command encoder and render
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render primitives with clear
        if let Some(renderer) = &self.primitive_renderer {
            renderer.render(
                &ctx.device,
                &mut encoder,
                &view,
                Some(wgpu::Color {
                    r: 0.043,
                    g: 0.059,
                    b: 0.078,
                    a: 1.0,
                }),
            );
        }

        // Render text elements
        if let (Some(text_renderer), Some(text_system), Some(tree)) =
            (&mut self.text_renderer, &mut self.text_system, &self.layout_tree)
        {
            text_renderer.begin();

            // Get mutable references to satisfy borrow checker
            let (font_system, swash_cache) = text_system.get_render_refs();

            // Render each text element at its layout position
            for text_elem in &self.text_elements {
                // Get the absolute position by traversing to the node
                let rect = tree.get_absolute_rect(text_elem.node_id, (0.0, 0.0));

                // Center the text within the rect (approximately)
                let text_x = rect.x + 10.0; // Small padding
                let text_y = rect.y + rect.height / 2.0 - text_elem.size / 2.0;

                text_renderer.draw_text(
                    &text_elem.content,
                    text_x,
                    text_y,
                    text_elem.size,
                    text_elem.color,
                    font_system,
                    swash_cache,
                );
            }

            text_renderer.render(&ctx.device, &ctx.queue, &mut encoder, &view);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

/// Render the layout tree to primitives
fn render_layout_tree(tree: &LayoutTree, root: NodeId, renderer: &mut PrimitiveRenderer) {
    tree.traverse(root, |_node, rect, visual| {
        if let Some(vis) = visual {
            // Draw background
            if let Some(bg) = vis.background {
                let color = Color::new(bg[0], bg[1], bg[2], bg[3]);
                if vis.corner_radius > 0.0 {
                    renderer.rounded_rect(
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                        vis.corner_radius,
                        color,
                    );
                } else {
                    renderer.rect(rect.x, rect.y, rect.width, rect.height, color);
                }
            }

            // Draw border
            if let Some(border) = vis.border_color {
                if vis.border_width > 0.0 {
                    let color = Color::new(border[0], border[1], border[2], border[3]);
                    renderer.border(
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                        vis.border_width,
                        vis.corner_radius,
                        color,
                    );
                }
            }
        }
    });
}

/// Convert hex color string to RGBA array
fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex)
        .map(|c| c.to_array())
        .unwrap_or([1.0, 1.0, 1.0, 1.0])
}

/// Build layout tree from IR (standalone function to avoid borrow issues)
fn build_from_ir(
    ir: &ComponentIR,
    tree: &mut LayoutTree,
    text_elements: &mut Vec<TextElement>,
) -> NodeId {
    let style = ir_to_style(ir);
    let visual = ir_to_visual(ir);

    // Check if this is a Text component
    if ir.kind == "Text" {
        let node = tree.new_visual_node(style, visual);

        let content = ir
            .props
            .iter()
            .find(|p| p.name == "content")
            .and_then(|p| match &p.value {
                PropertyValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();

        let size = ir
            .props
            .iter()
            .find(|p| p.name == "size")
            .and_then(|p| match &p.value {
                PropertyValue::Number(n) => Some(*n as f32),
                _ => None,
            })
            .unwrap_or(16.0);

        let color = ir
            .props
            .iter()
            .find(|p| p.name == "color")
            .and_then(|p| match &p.value {
                PropertyValue::String(s) => Some(hex_to_rgba(s)),
                _ => None,
            })
            .unwrap_or([0.898, 0.906, 0.922, 1.0]);

        text_elements.push(TextElement {
            node_id: node,
            content,
            size,
            color,
        });

        return node;
    }

    // Build children recursively
    let children: Vec<NodeId> = ir
        .children
        .iter()
        .map(|child| build_from_ir(child, tree, text_elements))
        .collect();

    if children.is_empty() {
        tree.new_visual_node(style, visual)
    } else {
        tree.new_visual_node_with_children(style, visual, &children)
    }
}

/// Convert IR to layout style (standalone function)
fn ir_to_style(ir: &ComponentIR) -> oxide_layout::Style {
    let mut builder = StyleBuilder::new();

    match ir.kind.as_str() {
        "Column" => builder = builder.flex_column(),
        "Row" => builder = builder.flex_row(),
        "Container" => builder = builder.flex_column(),
        _ => {}
    }

    for prop in &ir.props {
        match prop.name.as_str() {
            "align" => {
                if let PropertyValue::String(s) = &prop.value {
                    if s == "center" {
                        builder = builder.align_center();
                    }
                }
            }
            "justify" => {
                if let PropertyValue::String(s) = &prop.value {
                    if s == "center" {
                        builder = builder.justify_center();
                    } else if s == "space-between" {
                        builder = builder.justify_between();
                    }
                }
            }
            "width" => {
                if let PropertyValue::String(s) = &prop.value {
                    if s == "fill" {
                        builder = builder.width_percent(1.0);
                    }
                } else if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.width(*n as f32);
                }
            }
            "height" => {
                if let PropertyValue::String(s) = &prop.value {
                    if s == "fill" {
                        builder = builder.height_percent(1.0);
                    }
                } else if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.height(*n as f32);
                }
            }
            "gap" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.gap(*n as f32);
                }
            }
            "padding" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.padding(*n as f32);
                }
            }
            _ => {}
        }
    }

    for prop in &ir.style {
        if let ("padding", PropertyValue::Number(n)) = (prop.name.as_str(), &prop.value) {
            builder = builder.padding(*n as f32);
        }
    }

    builder.build()
}

/// Convert IR to node visual (standalone function)
fn ir_to_visual(ir: &ComponentIR) -> NodeVisual {
    let mut visual = NodeVisual::default();

    for prop in &ir.style {
        match prop.name.as_str() {
            "background" => {
                if let PropertyValue::String(s) = &prop.value {
                    visual = visual.with_background(hex_to_rgba(s));
                }
            }
            "border" | "border_width" => {
                if let PropertyValue::Number(n) = &prop.value {
                    let border_color = ir
                        .style
                        .iter()
                        .find(|p| p.name == "border_color")
                        .and_then(|p| match &p.value {
                            PropertyValue::String(s) => Some(hex_to_rgba(s)),
                            _ => None,
                        })
                        .unwrap_or([0.216, 0.255, 0.318, 1.0]);
                    visual = visual.with_border(border_color, *n as f32);
                }
            }
            "radius" => {
                if let PropertyValue::Number(n) = &prop.value {
                    visual = visual.with_radius(*n as f32);
                }
            }
            _ => {}
        }
    }

    visual
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // Create window
        let window_config = &self.manifest.window;
        let attrs = WindowAttributes::default()
            .with_title(&window_config.title)
            .with_inner_size(LogicalSize::new(
                window_config.width as f64,
                window_config.height as f64,
            ))
            .with_resizable(window_config.resizable)
            .with_decorations(window_config.decorations);

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );

        tracing::info!(
            "Created window: {}x{}",
            window_config.width,
            window_config.height
        );

        // Initialize rendering
        let render_ctx = pollster::block_on(RenderContext::new());

        // Create surface
        let surface = render_ctx.create_surface(window.clone());
        let surface: Surface<'static> = unsafe { std::mem::transmute(surface) };

        let size = window.inner_size();
        let surface_config =
            render_ctx.configure_surface(&surface, size.width.max(1), size.height.max(1));

        // Create primitive renderer
        let primitive_renderer = render_ctx.create_primitive_renderer(surface_config.format);
        primitive_renderer.set_viewport(&render_ctx.queue, size.width as f32, size.height as f32);

        // Create text renderer and system
        let text_renderer = TextRenderer::new(&render_ctx.device, surface_config.format);
        text_renderer.set_viewport(&render_ctx.queue, size.width as f32, size.height as f32);
        let text_system = TextSystem::new();

        self.viewport_size = (size.width.max(1), size.height.max(1));
        self.window = Some(window);
        self.render_ctx = Some(render_ctx);
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.primitive_renderer = Some(primitive_renderer);
        self.text_renderer = Some(text_renderer);
        self.text_system = Some(text_system);

        // Build and compute initial layout
        self.build_ui();
        self.compute_layout();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                self.render();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing() {
        let toml = r#"
            [app]
            id = "com.example.test"
            name = "Test App"
            version = "0.1.0"

            [window]
            title = "Test"
            width = 800
            height = 600
        "#;

        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.app.name, "Test App");
        assert_eq!(manifest.window.width, 800);
    }
}
