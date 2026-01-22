//! OxideKit Runtime
//!
//! Window management, event handling, application lifecycle, and animation support.

pub mod animation;
pub mod events;
pub mod reactive;
pub mod text_input;

pub use animation::{AnimationRuntime, Animatable, properties as anim_properties};
pub use events::{EventManager, UiEvent, MouseButton, Modifiers, EventHandler, EventType, HandlerAction};
pub use reactive::{ReactiveState, StateValue, StateBinding};
pub use text_input::{TextInputManager, TextInputState};

// Re-export file picker for easy access
pub use oxide_file_picker::{OpenDialog, SaveDialog, DirectoryDialog, FileFilter};

use anyhow::Result;
use oxide_compiler::{compile, ComponentIR, PropertyValue};
use oxide_layout::{AvailableSpace, LayoutTree, NodeId, NodeVisual, Size, StyleBuilder};
use oxide_render::{Color, PrimitiveRenderer, RenderContext};
use oxide_text::{TextRenderer, TextSystem};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use wgpu::Surface;

/// Command from UI to backend
#[derive(Debug, Clone)]
pub enum AppCommand {
    /// Call a named function with arguments
    CallFunction { name: String, args: Vec<events::ActionValue> },
    /// Navigate to a route
    Navigate { path: String },
    /// Custom command
    Custom(String),
}

/// State update from backend to UI
#[derive(Debug, Clone)]
pub struct StateUpdate {
    pub key: String,
    pub value: StateValue,
}

/// Shared application context for communication between UI and backend
pub struct AppContext {
    /// State updates to be applied to UI (backend writes, UI reads)
    pub state_updates: Arc<Mutex<Vec<StateUpdate>>>,
    /// Commands from UI (UI writes, backend reads)
    pub commands: Arc<Mutex<Vec<AppCommand>>>,
    /// Shared state that can be read/written by both
    pub shared_state: Arc<Mutex<HashMap<String, String>>>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            state_updates: Arc::new(Mutex::new(Vec::new())),
            commands: Arc::new(Mutex::new(Vec::new())),
            shared_state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Push a state update (called from backend)
    pub fn push_state_update(&self, key: impl Into<String>, value: StateValue) {
        if let Ok(mut updates) = self.state_updates.lock() {
            updates.push(StateUpdate {
                key: key.into(),
                value,
            });
        }
    }

    /// Push a command (called from UI)
    pub fn push_command(&self, cmd: AppCommand) {
        if let Ok(mut commands) = self.commands.lock() {
            commands.push(cmd);
        }
    }

    /// Take all pending commands (called from backend)
    pub fn take_commands(&self) -> Vec<AppCommand> {
        if let Ok(mut commands) = self.commands.lock() {
            std::mem::take(&mut *commands)
        } else {
            Vec::new()
        }
    }

    /// Take all pending state updates (called from UI)
    pub fn take_state_updates(&self) -> Vec<StateUpdate> {
        if let Ok(mut updates) = self.state_updates.lock() {
            std::mem::take(&mut *updates)
        } else {
            Vec::new()
        }
    }

    /// Set shared state value
    pub fn set_shared(&self, key: impl Into<String>, value: impl Into<String>) {
        if let Ok(mut state) = self.shared_state.lock() {
            state.insert(key.into(), value.into());
        }
    }

    /// Get shared state value
    pub fn get_shared(&self, key: &str) -> Option<String> {
        self.shared_state.lock().ok()?.get(key).cloned()
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AppContext {
    fn clone(&self) -> Self {
        Self {
            state_updates: Arc::clone(&self.state_updates),
            commands: Arc::clone(&self.commands),
            shared_state: Arc::clone(&self.shared_state),
        }
    }
}
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton as WinitMouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
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
    /// Enable layout debug overlay (shows bounding boxes)
    #[serde(default)]
    pub debug_layout: bool,
}

/// The main OxideKit application
pub struct Application {
    manifest: Manifest,
    project_path: PathBuf,
    context: Option<AppContext>,
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
            context: None,
        })
    }

    /// Set the application context for UI-backend communication
    pub fn with_context(mut self, context: AppContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Get a clone of the context (for use in backend threads)
    pub fn context(&self) -> Option<AppContext> {
        self.context.clone()
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
            context: None,
        }
    }

    /// Run the application
    pub fn run(self) -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);

        // Try to compile the UI
        let ui_ir = self.compile_ui();

        let mut app_state = AppState::new(self.manifest, ui_ir, self.context);

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
    /// Computed absolute position (updated after layout)
    computed_x: f32,
    computed_y: f32,
    computed_height: f32,
    /// State binding variable name (if content is bound to state)
    binding: Option<String>,
}

/// Dev overlay log entry
#[derive(Debug, Clone)]
struct DevLogEntry {
    timestamp: std::time::Instant,
    category: &'static str,
    message: String,
}

/// Dev overlay state
struct DevOverlay {
    visible: bool,
    logs: Vec<DevLogEntry>,
    max_logs: usize,
}

/// Render data for dev overlay (pre-computed to avoid borrow issues)
struct DevOverlayRenderData {
    panel_x: f32,
    panel_y: f32,
    panel_width: f32,
    panel_height: f32,
    scale: f32,
    log_entries: Vec<(f32, f32, &'static str)>, // (y, x, category)
    texts: Vec<(f32, f32, String, f32, [f32; 4])>, // (x, y, content, size, color)
}

impl DevOverlay {
    fn new() -> Self {
        Self {
            visible: false,
            logs: Vec::new(),
            max_logs: 20,
        }
    }

    fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.log("DEV", "Dev overlay enabled (F12 to hide)");
        }
    }

    fn log(&mut self, category: &'static str, message: impl Into<String>) {
        self.logs.push(DevLogEntry {
            timestamp: std::time::Instant::now(),
            category,
            message: message.into(),
        });
        // Keep only recent logs
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
    }
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
    /// Physical viewport size (in device pixels)
    viewport_size: (u32, u32),
    /// Logical viewport size (in logical pixels, for layout)
    logical_size: (f32, f32),
    /// Device scale factor (physical / logical)
    scale_factor: f64,
    layout_tree: Option<LayoutTree>,
    root_node: Option<NodeId>,
    text_elements: Vec<TextElement>,
    /// Number of nodes in layout tree (for debug assertions)
    node_count: usize,
    /// Previous node count (for detecting duplicates)
    prev_node_count: usize,
    /// Event manager for handling user input
    event_manager: EventManager,
    /// Reactive state for UI data binding
    reactive_state: ReactiveState,
    /// Last state version (for change detection)
    last_state_version: u64,
    /// Text input manager for handling text fields
    text_input_manager: TextInputManager,
    /// Current keyboard modifiers state
    keyboard_modifiers: KeyboardModifiers,
    /// Application context for UI-backend communication
    app_context: Option<AppContext>,
    /// Dev overlay for debugging
    dev_overlay: DevOverlay,
}

/// Keyboard modifier state
#[derive(Debug, Default, Clone, Copy)]
struct KeyboardModifiers {
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
}

impl AppState {
    fn new(manifest: Manifest, ui_ir: Option<ComponentIR>, app_context: Option<AppContext>) -> Self {
        let mut dev_overlay = DevOverlay::new();
        // Auto-enable dev overlay in dev mode
        if manifest.dev.inspector {
            dev_overlay.visible = true;
            dev_overlay.log("DEV", "Dev overlay auto-enabled (inspector: true)");
        }

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
            logical_size: (0.0, 0.0),
            scale_factor: 1.0,
            layout_tree: None,
            root_node: None,
            text_elements: Vec::new(),
            node_count: 0,
            prev_node_count: 0,
            event_manager: EventManager::new(),
            reactive_state: ReactiveState::new(),
            last_state_version: 0,
            text_input_manager: TextInputManager::new(),
            keyboard_modifiers: KeyboardModifiers::default(),
            app_context,
            dev_overlay,
        }
    }

    fn build_ui(&mut self) {
        // Store previous count for duplicate detection
        self.prev_node_count = self.node_count;
        self.text_elements.clear();
        self.event_manager.clear_handlers();
        self.dev_overlay.log("DEV", "Rebuilding UI...");

        // Build from IR if available
        if let Some(ir) = self.ui_ir.clone() {
            let mut tree = LayoutTree::new();
            let mut text_elements = Vec::new();
            let mut event_manager = EventManager::new();

            // Use text_system for measurement if available
            let root = if let Some(text_system) = &mut self.text_system {
                build_from_ir_with_measurement(&ir, &mut tree, &mut text_elements, text_system, &mut event_manager)
            } else {
                // Fallback without measurement (will use estimates)
                build_from_ir(&ir, &mut tree, &mut text_elements, &mut event_manager)
            };

            // Count nodes
            let mut count = 0;
            tree.traverse(root, |_, _, _| count += 1);
            self.node_count = count;

            // Debug: check for unexpected node count changes
            if self.prev_node_count > 0 && self.node_count != self.prev_node_count {
                tracing::warn!(
                    "Node count changed: {} -> {} (possible duplication issue)",
                    self.prev_node_count,
                    self.node_count
                );
            }

            if self.manifest.dev.debug_layout {
                tracing::info!(
                    "Layout tree: {} nodes, {} text elements, {} handlers",
                    self.node_count,
                    text_elements.len(),
                    event_manager.handlers.len()
                );
            }

            self.text_elements = text_elements;
            self.event_manager = event_manager;
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
        let root = match self.root_node {
            Some(r) => r,
            None => return,
        };

        let tree = match &mut self.layout_tree {
            Some(t) => t,
            None => return,
        };

        // Use logical size for layout computation
        let (w, h) = self.logical_size;
        tree.compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(w),
                height: AvailableSpace::Definite(h),
            },
        );

        // Build a map of node_id -> absolute rect by traversing the tree
        use std::collections::HashMap;
        let mut node_rects: HashMap<NodeId, oxide_layout::ComputedRect> = HashMap::new();

        tree.traverse(root, |node, rect, _visual| {
            node_rects.insert(node, rect);
        });

        // Update each text element's computed position
        for text_elem in &mut self.text_elements {
            if let Some(rect) = node_rects.get(&text_elem.node_id) {
                text_elem.computed_x = rect.x;
                text_elem.computed_y = rect.y;
                text_elem.computed_height = rect.height;
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        // Store physical size
        self.viewport_size = (width, height);

        // Compute logical size from physical size using scale factor
        self.logical_size = (
            width as f32 / self.scale_factor as f32,
            height as f32 / self.scale_factor as f32,
        );

        if let (Some(ctx), Some(surface), Some(config)) =
            (&self.render_ctx, &self.surface, &mut self.surface_config)
        {
            // Surface uses physical pixels
            config.width = width;
            config.height = height;
            surface.configure(&ctx.device, config);

            // Renderers use physical pixels for GPU viewport
            if let Some(renderer) = &self.primitive_renderer {
                renderer.set_viewport(&ctx.queue, width as f32, height as f32);
            }

            if let Some(renderer) = &self.text_renderer {
                renderer.set_viewport(&ctx.queue, width as f32, height as f32);
            }
        }

        // Recompute layout for new viewport size (uses logical_size)
        self.compute_layout();
    }

    /// Apply pending state updates from the application context
    fn apply_state_updates(&mut self) {
        let updates = if let Some(ctx) = &self.app_context {
            ctx.take_state_updates()
        } else {
            return;
        };

        let mut needs_rebuild = false;
        for update in updates {
            tracing::debug!("Applying state update: {} = {:?}", update.key, update.value);
            self.dev_overlay.log("UPDATE", format!("{} = {}", update.key, update.value.to_string_value()));
            self.reactive_state.set(&update.key, update.value);
            needs_rebuild = true;
        }

        // If state changed, rebuild UI to reflect text content changes
        if needs_rebuild {
            self.build_ui();
            self.compute_layout();
        }
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

        // Scale factor for converting logical to physical coordinates
        let scale = self.scale_factor as f32;

        // Collect dev overlay data before rendering (to avoid borrow issues)
        let dev_overlay_visible = self.dev_overlay.visible;
        let dev_overlay_data = if dev_overlay_visible {
            Some(self.collect_dev_overlay_render_data())
        } else {
            None
        };

        // Begin rendering primitives
        if let Some(renderer) = &mut self.primitive_renderer {
            renderer.begin();

            // Render UI from layout tree
            if let (Some(tree), Some(root)) = (&self.layout_tree, self.root_node) {
                render_layout_tree(tree, root, renderer, scale, &self.event_manager);

                // Debug overlay: render bounding boxes
                if self.manifest.dev.debug_layout {
                    render_debug_overlay(tree, root, renderer, scale);
                }
            }

            // Render dev overlay background if visible
            if let Some(ref data) = dev_overlay_data {
                render_dev_overlay_background(renderer, data);
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
        if let (Some(text_renderer), Some(text_system)) =
            (&mut self.text_renderer, &mut self.text_system)
        {
            text_renderer.begin();

            // Get mutable references to satisfy borrow checker
            let (font_system, swash_cache) = text_system.get_render_refs();

            // Render each text element at its computed layout position
            let scale = self.scale_factor as f32;
            for text_elem in &self.text_elements {
                // Resolve content - check if bound to state
                let content = if let Some(var) = &text_elem.binding {
                    // Get value from reactive state
                    self.reactive_state
                        .get(var)
                        .and_then(|v| v.as_string())
                        .unwrap_or(&text_elem.content)
                        .to_string()
                } else {
                    text_elem.content.clone()
                };

                // Use pre-computed absolute positions (logical coords)
                // Center text vertically within its container
                let text_x = (text_elem.computed_x + 4.0) * scale; // Small padding
                let text_y = (text_elem.computed_y + text_elem.computed_height / 2.0 - text_elem.size / 2.0) * scale;
                let scaled_font_size = text_elem.size * scale;

                text_renderer.draw_text(
                    &content,
                    text_x,
                    text_y,
                    scaled_font_size,
                    text_elem.color,
                    font_system,
                    swash_cache,
                );
            }

            // Render dev overlay text if visible
            if let Some(ref data) = dev_overlay_data {
                for (x, y, content, size, color) in &data.texts {
                    text_renderer.draw_text(
                        content,
                        *x,
                        *y,
                        *size,
                        *color,
                        font_system,
                        swash_cache,
                    );
                }
            }

            text_renderer.render(&ctx.device, &ctx.queue, &mut encoder, &view);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    /// Collect dev overlay render data (pre-computed to avoid borrow issues)
    fn collect_dev_overlay_render_data(&self) -> DevOverlayRenderData {
        let scale = self.scale_factor as f32;
        let (vw, vh) = self.viewport_size;

        let panel_width = 300.0 * scale;
        let panel_x = vw as f32 - panel_width - 10.0 * scale;
        let panel_y = 10.0 * scale;
        let panel_height = (vh as f32 - 20.0 * scale).min(400.0 * scale);

        // Collect log entry categories for color dots
        let log_y = panel_y + 120.0 * scale;
        let log_start_y = log_y + 25.0 * scale;
        let line_height = 18.0 * scale;
        let mut log_entries = Vec::new();
        for (i, entry) in self.dev_overlay.logs.iter().rev().take(12).enumerate() {
            let y = log_start_y + i as f32 * line_height;
            if y > panel_y + panel_height - 10.0 * scale {
                break;
            }
            log_entries.push((y, panel_x + 10.0 * scale, entry.category));
        }

        // Collect texts
        let mut texts = Vec::new();

        // Header text
        texts.push((
            panel_x + 10.0 * scale,
            panel_y + 7.0 * scale,
            "DEV OVERLAY (F12)".to_string(),
            11.0 * scale,
            [0.9, 0.95, 1.0, 1.0],
        ));

        // State section
        let state_y = panel_y + 40.0 * scale;
        texts.push((
            panel_x + 10.0 * scale,
            state_y,
            "STATE".to_string(),
            10.0 * scale,
            [0.6, 0.7, 0.8, 1.0],
        ));

        // Show reactive state values
        let mut state_line = 0;
        for (key, value) in self.reactive_state.iter() {
            let val_str = value.to_string_value();
            let display_str = if val_str.len() > 25 {
                format!("{}...", &val_str[..22])
            } else {
                val_str
            };
            texts.push((
                panel_x + 10.0 * scale,
                state_y + 15.0 * scale + state_line as f32 * 14.0 * scale,
                format!("{}: {}", key, display_str),
                9.0 * scale,
                [0.7, 0.8, 0.9, 1.0],
            ));
            state_line += 1;
            if state_line >= 4 {
                break;
            }
        }

        // Event log section
        texts.push((
            panel_x + 10.0 * scale,
            log_y + 5.0 * scale,
            "EVENT LOG".to_string(),
            10.0 * scale,
            [0.6, 0.7, 0.8, 1.0],
        ));

        // Recent log entries
        for (i, entry) in self.dev_overlay.logs.iter().rev().take(12).enumerate() {
            let y = log_start_y + i as f32 * line_height;
            if y > panel_y + panel_height - 10.0 * scale {
                break;
            }
            let msg = if entry.message.len() > 30 {
                format!("{}...", &entry.message[..27])
            } else {
                entry.message.clone()
            };
            texts.push((
                panel_x + 22.0 * scale, // After the color dot
                y,
                format!("[{}] {}", entry.category, msg),
                8.0 * scale,
                [0.75, 0.8, 0.85, 1.0],
            ));
        }

        DevOverlayRenderData {
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            scale,
            log_entries,
            texts,
        }
    }
}

/// Render the dev overlay background (free function to avoid borrow issues)
fn render_dev_overlay_background(renderer: &mut PrimitiveRenderer, data: &DevOverlayRenderData) {
    let scale = data.scale;

    // Background panel with transparency
    renderer.rounded_rect(
        data.panel_x,
        data.panel_y,
        data.panel_width,
        data.panel_height,
        8.0 * scale,
        Color::new(0.05, 0.07, 0.1, 0.92),
    );

    // Header bar
    renderer.rounded_rect(
        data.panel_x,
        data.panel_y,
        data.panel_width,
        28.0 * scale,
        8.0 * scale,
        Color::new(0.15, 0.2, 0.3, 1.0),
    );

    // State section header line
    let state_y = data.panel_y + 35.0 * scale;
    renderer.rect(
        data.panel_x + 8.0 * scale,
        state_y,
        data.panel_width - 16.0 * scale,
        1.0,
        Color::new(0.3, 0.4, 0.5, 0.5),
    );

    // Event log section separator
    let log_y = data.panel_y + 120.0 * scale;
    renderer.rect(
        data.panel_x + 8.0 * scale,
        log_y,
        data.panel_width - 16.0 * scale,
        1.0,
        Color::new(0.3, 0.4, 0.5, 0.5),
    );

    // Category color indicators for recent logs
    for (y, x, category) in &data.log_entries {
        let dot_color = match *category {
            "EVENT" => Color::new(0.4, 0.8, 1.0, 1.0),   // Cyan for events
            "NAV" => Color::new(0.4, 1.0, 0.6, 1.0),     // Green for navigation
            "STATE" => Color::new(1.0, 0.8, 0.4, 1.0),   // Yellow for state
            "CALL" => Color::new(0.8, 0.6, 1.0, 1.0),    // Purple for calls
            "HANDLER" => Color::new(1.0, 0.6, 0.4, 1.0), // Orange for handlers
            "UPDATE" => Color::new(0.6, 1.0, 0.8, 1.0),  // Light green for updates
            "WARN" => Color::new(1.0, 0.4, 0.4, 1.0),    // Red for warnings
            _ => Color::new(0.6, 0.6, 0.6, 1.0),         // Gray for others
        };
        renderer.rounded_rect(
            *x,
            *y + 3.0 * scale,
            6.0 * scale,
            6.0 * scale,
            3.0 * scale,
            dot_color,
        );
    }
}

/// Render the layout tree to primitives
/// Coordinates are scaled by scale_factor to convert from logical to physical pixels
fn render_layout_tree(
    tree: &LayoutTree,
    root: NodeId,
    renderer: &mut PrimitiveRenderer,
    scale: f32,
    event_manager: &EventManager,
) {
    tree.traverse(root, |node, rect, visual| {
        if let Some(vis) = visual {
            // Scale coordinates from logical to physical pixels
            let x = rect.x * scale;
            let y = rect.y * scale;
            let w = rect.width * scale;
            let h = rect.height * scale;
            let radius = vis.corner_radius * scale;
            let border_w = vis.border_width * scale;

            // Check interactive state for this node
            let interactive = event_manager.get_state(node);
            let has_handlers = event_manager.handlers.contains_key(&node);

            // Draw background with hover/press effects
            if let Some(bg) = vis.background {
                let mut color = Color::new(bg[0], bg[1], bg[2], bg[3]);

                // Apply interactive state visual feedback
                if has_handlers {
                    if interactive.pressed {
                        // Darken when pressed
                        color = Color::new(
                            (bg[0] * 0.7).min(1.0),
                            (bg[1] * 0.7).min(1.0),
                            (bg[2] * 0.7).min(1.0),
                            bg[3],
                        );
                    } else if interactive.hovered {
                        // Lighten when hovered
                        color = Color::new(
                            (bg[0] * 1.15).min(1.0),
                            (bg[1] * 1.15).min(1.0),
                            (bg[2] * 1.15).min(1.0),
                            bg[3],
                        );
                    }
                }

                if radius > 0.0 {
                    renderer.rounded_rect(x, y, w, h, radius, color);
                } else {
                    renderer.rect(x, y, w, h, color);
                }
            }

            // Draw border with optional hover highlight
            if let Some(border) = vis.border_color {
                if border_w > 0.0 {
                    let mut color = Color::new(border[0], border[1], border[2], border[3]);

                    // Brighten border on hover for interactive elements
                    if has_handlers && interactive.hovered {
                        color = Color::new(
                            (border[0] * 1.3).min(1.0),
                            (border[1] * 1.3).min(1.0),
                            (border[2] * 1.3).min(1.0),
                            border[3],
                        );
                    }

                    renderer.border(x, y, w, h, border_w, radius, color);
                }
            }
        }
    });
}

/// Render debug overlay showing bounding boxes for all nodes
fn render_debug_overlay(tree: &LayoutTree, root: NodeId, renderer: &mut PrimitiveRenderer, scale: f32) {
    // Use different colors to distinguish nodes
    let border_colors = [
        Color::new(1.0, 0.0, 0.0, 0.7),   // Red - content box
        Color::new(0.0, 1.0, 0.0, 0.7),   // Green
        Color::new(0.0, 0.0, 1.0, 0.7),   // Blue
        Color::new(1.0, 1.0, 0.0, 0.7),   // Yellow
        Color::new(1.0, 0.0, 1.0, 0.7),   // Magenta
        Color::new(0.0, 1.0, 1.0, 0.7),   // Cyan
    ];
    let padding_color = Color::new(0.0, 0.5, 0.0, 0.2);  // Green tint for padding
    let margin_color = Color::new(1.0, 0.5, 0.0, 0.15);  // Orange tint for margin
    let scroll_indicator = Color::new(0.0, 0.5, 1.0, 0.5); // Blue for scroll regions

    let mut color_idx = 0;

    tree.traverse(root, |node, rect, visual| {
        let border_color = border_colors[color_idx % border_colors.len()];
        color_idx += 1;

        // Scale coordinates
        let x = rect.x * scale;
        let y = rect.y * scale;
        let w = rect.width * scale;
        let h = rect.height * scale;

        // Get style for this node to check padding/margin
        if let Some(style) = tree.get_style(node) {
            // Draw margin area (outer)
            let margin_top = resolve_length(&style.margin.top) * scale;
            let margin_right = resolve_length(&style.margin.right) * scale;
            let margin_bottom = resolve_length(&style.margin.bottom) * scale;
            let margin_left = resolve_length(&style.margin.left) * scale;

            if margin_top > 0.0 || margin_right > 0.0 || margin_bottom > 0.0 || margin_left > 0.0 {
                // Draw margin as outer highlight
                let outer_x = x - margin_left;
                let outer_y = y - margin_top;
                let outer_w = w + margin_left + margin_right;
                let _outer_h = h + margin_top + margin_bottom;
                renderer.rect(outer_x, outer_y, outer_w, margin_top, margin_color); // Top
                renderer.rect(outer_x, y + h, outer_w, margin_bottom, margin_color); // Bottom
                renderer.rect(outer_x, y, margin_left, h, margin_color); // Left
                renderer.rect(x + w, y, margin_right, h, margin_color); // Right
            }

            // Draw padding area (inner)
            let pad_top = resolve_length_percent(&style.padding.top) * scale;
            let pad_right = resolve_length_percent(&style.padding.right) * scale;
            let pad_bottom = resolve_length_percent(&style.padding.bottom) * scale;
            let pad_left = resolve_length_percent(&style.padding.left) * scale;

            if pad_top > 0.0 || pad_right > 0.0 || pad_bottom > 0.0 || pad_left > 0.0 {
                renderer.rect(x, y, w, pad_top, padding_color); // Top
                renderer.rect(x, y + h - pad_bottom, w, pad_bottom, padding_color); // Bottom
                renderer.rect(x, y + pad_top, pad_left, h - pad_top - pad_bottom, padding_color); // Left
                renderer.rect(x + w - pad_right, y + pad_top, pad_right, h - pad_top - pad_bottom, padding_color); // Right
            }

            // Mark scroll containers with special indicator
            if tree.clips_content(node) {
                // Draw scroll indicator in corner
                let indicator_size = 8.0 * scale;
                renderer.rect(x + w - indicator_size - 2.0, y + 2.0, indicator_size, indicator_size, scroll_indicator);
            }
        }

        // Draw border around the content box
        renderer.border(x, y, w, h, 1.0, 0.0, border_color);

        // Show clipping indicator if this node clips children
        if let Some(vis) = visual {
            if vis.clips_children {
                // Dashed corner indicator for clip regions
                let dash = 4.0 * scale;
                renderer.rect(x, y, dash, 2.0, scroll_indicator);
                renderer.rect(x, y, 2.0, dash, scroll_indicator);
                renderer.rect(x + w - dash, y, dash, 2.0, scroll_indicator);
                renderer.rect(x + w - 2.0, y, 2.0, dash, scroll_indicator);
            }
        }
    });
}

/// Helper to resolve LengthPercentageAuto to f32
fn resolve_length(value: &oxide_layout::LengthPercentageAuto) -> f32 {
    match value {
        oxide_layout::LengthPercentageAuto::Length(l) => *l,
        _ => 0.0,
    }
}

/// Helper to resolve LengthPercentage to f32
fn resolve_length_percent(value: &oxide_layout::LengthPercentage) -> f32 {
    match value {
        oxide_layout::LengthPercentage::Length(l) => *l,
        _ => 0.0,
    }
}

/// Convert hex color string to RGBA array
fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex)
        .map(|c| c.to_array())
        .unwrap_or([1.0, 1.0, 1.0, 1.0])
}

/// Build layout tree from IR with proper text measurement
fn build_from_ir_with_measurement(
    ir: &ComponentIR,
    tree: &mut LayoutTree,
    text_elements: &mut Vec<TextElement>,
    text_system: &mut TextSystem,
    event_manager: &mut EventManager,
) -> NodeId {
    let visual = ir_to_visual(ir);

    // Check if this is a Text component
    if ir.kind == "Text" {
        // Extract content - could be a string or a binding
        let (content, binding) = ir
            .props
            .iter()
            .find(|p| p.name == "content")
            .map(|p| match &p.value {
                PropertyValue::String(s) => (s.clone(), None),
                PropertyValue::Binding { var } => (format!("{{{}}}", var), Some(var.clone())),
                _ => (String::new(), None),
            })
            .unwrap_or((String::new(), None));

        let font_size = ir
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

        // Measure text dimensions (use placeholder for bindings)
        let measure_text = if binding.is_some() { "0.00".to_string() } else { content.clone() };
        let (text_width, text_height) = text_system.measure_text(&measure_text, font_size);

        // Create style with measured dimensions
        let style = StyleBuilder::new()
            .size(text_width, text_height)
            .build();

        let node = tree.new_visual_node(style, visual);

        text_elements.push(TextElement {
            node_id: node,
            content,
            size: font_size,
            color,
            computed_x: 0.0,
            computed_y: 0.0,
            computed_height: 0.0,
            binding,
        });

        // Register handlers for this node
        register_handlers(node, ir, event_manager);

        return node;
    }

    // For non-Text nodes, use normal style conversion
    let style = ir_to_style(ir);

    // Build children recursively
    let children: Vec<NodeId> = ir
        .children
        .iter()
        .map(|child| build_from_ir_with_measurement(child, tree, text_elements, text_system, event_manager))
        .collect();

    let node = if children.is_empty() {
        tree.new_visual_node(style, visual)
    } else {
        tree.new_visual_node_with_children(style, visual, &children)
    };

    // Register handlers for this node
    register_handlers(node, ir, event_manager);

    node
}

/// Register event handlers for a node from IR
fn register_handlers(node: NodeId, ir: &ComponentIR, event_manager: &mut EventManager) {
    for handler_ir in &ir.handlers {
        // Convert event string to EventType
        let event_type = match handler_ir.event.to_lowercase().as_str() {
            "click" => events::EventType::Click,
            "doubleclick" | "dblclick" => events::EventType::DoubleClick,
            "mousedown" => events::EventType::MouseDown,
            "mouseup" => events::EventType::MouseUp,
            "mouseenter" | "hover" => events::EventType::MouseEnter,
            "mouseleave" => events::EventType::MouseLeave,
            "mousemove" => events::EventType::MouseMove,
            "focus" => events::EventType::Focus,
            "blur" => events::EventType::Blur,
            "keydown" => events::EventType::KeyDown,
            "keyup" => events::EventType::KeyUp,
            "input" => events::EventType::TextInput,
            _ => {
                tracing::warn!("Unknown event type: {}", handler_ir.event);
                continue;
            }
        };

        // Parse the handler expression
        let action = parse_handler_action(&handler_ir.handler);

        event_manager.register_handler(
            node,
            events::EventHandler {
                event_type,
                action,
            },
        );

        tracing::info!(
            "Registered {} handler for node {:?}: {}",
            handler_ir.event,
            node,
            handler_ir.handler
        );
    }
}

/// Parse a handler expression string into a HandlerAction
fn parse_handler_action(expr: &str) -> events::HandlerAction {
    let expr = expr.trim();

    // Check for state mutation patterns: state.field += value
    if expr.starts_with("state.") {
        if let Some(action) = parse_state_mutation(expr) {
            return action;
        }
    }

    // Check for navigate pattern: navigate('/path')
    if expr.starts_with("navigate(") && expr.ends_with(')') {
        let inner = &expr[9..expr.len() - 1].trim();
        let path = inner.trim_matches(|c| c == '"' || c == '\'');
        return events::HandlerAction::Navigate {
            path: path.to_string(),
        };
    }

    // Check for function call pattern: funcName(args)
    if let Some(paren_pos) = expr.find('(') {
        if expr.ends_with(')') {
            let name = expr[..paren_pos].trim().to_string();
            return events::HandlerAction::FunctionCall {
                name,
                args: vec![],
            };
        }
    }

    // Fallback to raw expression
    events::HandlerAction::Raw(expr.to_string())
}

/// Parse a state mutation expression like "state.count += 1"
fn parse_state_mutation(expr: &str) -> Option<events::HandlerAction> {
    let rest = expr.strip_prefix("state.")?;

    let (field, op, value_str) = if let Some(pos) = rest.find("+=") {
        (&rest[..pos], events::MutationOp::Add, rest[pos + 2..].trim())
    } else if let Some(pos) = rest.find("-=") {
        (
            &rest[..pos],
            events::MutationOp::Subtract,
            rest[pos + 2..].trim(),
        )
    } else if let Some(pos) = rest.find("*=") {
        (
            &rest[..pos],
            events::MutationOp::Multiply,
            rest[pos + 2..].trim(),
        )
    } else if let Some(pos) = rest.find("/=") {
        (
            &rest[..pos],
            events::MutationOp::Divide,
            rest[pos + 2..].trim(),
        )
    } else if let Some(pos) = rest.find('=') {
        if pos > 0 && !rest[..pos].ends_with(['!', '<', '>', '=']) {
            (&rest[..pos], events::MutationOp::Set, rest[pos + 1..].trim())
        } else {
            return None;
        }
    } else {
        return None;
    };

    let field = field.trim().to_string();

    let value = if let Ok(n) = value_str.parse::<f64>() {
        events::ActionValue::Number(n)
    } else if value_str == "true" {
        events::ActionValue::Bool(true)
    } else if value_str == "false" {
        events::ActionValue::Bool(false)
    } else {
        events::ActionValue::String(value_str.trim_matches('"').to_string())
    };

    Some(events::HandlerAction::StateMutation { field, op, value })
}

/// Build layout tree from IR (fallback without measurement)
fn build_from_ir(
    ir: &ComponentIR,
    tree: &mut LayoutTree,
    text_elements: &mut Vec<TextElement>,
    event_manager: &mut EventManager,
) -> NodeId {
    let visual = ir_to_visual(ir);

    // Check if this is a Text component
    if ir.kind == "Text" {
        // Extract content - could be a string or a binding
        let (content, binding) = ir
            .props
            .iter()
            .find(|p| p.name == "content")
            .map(|p| match &p.value {
                PropertyValue::String(s) => (s.clone(), None),
                PropertyValue::Binding { var } => (format!("{{{}}}", var), Some(var.clone())),
                _ => (String::new(), None),
            })
            .unwrap_or((String::new(), None));

        let font_size = ir
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

        // Estimate text dimensions (fallback)
        let measure_text = if binding.is_some() { "0.00".to_string() } else { content.clone() };
        let estimated_width = measure_text.len() as f32 * font_size * 0.6;
        let estimated_height = font_size * 1.2;

        let style = StyleBuilder::new()
            .size(estimated_width, estimated_height)
            .build();

        let node = tree.new_visual_node(style, visual);

        text_elements.push(TextElement {
            node_id: node,
            content,
            size: font_size,
            color,
            computed_x: 0.0,
            computed_y: 0.0,
            computed_height: 0.0,
            binding,
        });

        // Register handlers
        register_handlers(node, ir, event_manager);

        return node;
    }

    // For non-Text nodes, use normal style conversion
    let style = ir_to_style(ir);

    // Build children recursively
    let children: Vec<NodeId> = ir
        .children
        .iter()
        .map(|child| build_from_ir(child, tree, text_elements, event_manager))
        .collect();

    let node = if children.is_empty() {
        tree.new_visual_node(style, visual)
    } else {
        tree.new_visual_node_with_children(style, visual, &children)
    };

    // Register handlers
    register_handlers(node, ir, event_manager);

    node
}

/// Convert IR to layout style (standalone function)
fn ir_to_style(ir: &ComponentIR) -> oxide_layout::Style {
    let mut builder = StyleBuilder::new();

    match ir.kind.as_str() {
        "Column" => builder = builder.flex_column(),
        "Row" => builder = builder.flex_row(),
        "Container" => builder = builder.flex_column(),
        "Scroll" | "ScrollView" => {
            builder = builder.flex_column().overflow_scroll();
        }
        "ScrollX" => {
            builder = builder.flex_row().overflow_x_scroll().overflow_y_hidden();
        }
        "ScrollY" => {
            builder = builder.flex_column().overflow_y_scroll().overflow_x_hidden();
        }
        _ => {}
    }

    // Process properties from ir.props (this is where .oui properties go)
    for prop in &ir.props {
        match prop.name.as_str() {
            "align" => {
                if let PropertyValue::String(s) = &prop.value {
                    match s.as_str() {
                        "center" => builder = builder.align_center(),
                        "start" => builder = builder.align_start(),
                        "end" => builder = builder.align_end(),
                        _ => {}
                    }
                }
            }
            "justify" => {
                if let PropertyValue::String(s) = &prop.value {
                    match s.as_str() {
                        "center" => builder = builder.justify_center(),
                        "space_between" | "space-between" => builder = builder.justify_between(),
                        "start" => builder = builder.justify_start(),
                        "end" => builder = builder.justify_end(),
                        _ => {}
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
            "flex" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.flex_grow(*n as f32);
                }
            }
            "overflow" => {
                if let PropertyValue::String(s) = &prop.value {
                    match s.as_str() {
                        "hidden" => builder = builder.overflow_hidden(),
                        "scroll" => builder = builder.overflow_scroll(),
                        "visible" => builder = builder.overflow_visible(),
                        _ => {}
                    }
                }
            }
            "min_width" | "minWidth" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.min_width(*n as f32);
                }
            }
            "min_height" | "minHeight" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.min_height(*n as f32);
                }
            }
            "max_width" | "maxWidth" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.max_width(*n as f32);
                }
            }
            "max_height" | "maxHeight" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.max_height(*n as f32);
                }
            }
            "flex_wrap" | "wrap" => {
                if let PropertyValue::Bool(b) = &prop.value {
                    if *b {
                        builder = builder.flex_wrap();
                    }
                } else if let PropertyValue::String(s) = &prop.value {
                    match s.as_str() {
                        "wrap" => builder = builder.flex_wrap(),
                        "nowrap" => builder = builder.flex_nowrap(),
                        _ => {}
                    }
                }
            }
            "position" => {
                if let PropertyValue::String(s) = &prop.value {
                    match s.as_str() {
                        "absolute" => builder = builder.position_absolute(),
                        "relative" => builder = builder.position_relative(),
                        _ => {}
                    }
                }
            }
            "aspect_ratio" | "aspectRatio" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.aspect_ratio(*n as f32);
                }
            }
            _ => {}
        }
    }

    // Also check ir.style for any style-specific properties
    for prop in &ir.style {
        match prop.name.as_str() {
            "padding" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.padding(*n as f32);
                }
            }
            "gap" => {
                if let PropertyValue::Number(n) = &prop.value {
                    builder = builder.gap(*n as f32);
                }
            }
            _ => {}
        }
    }

    builder.build()
}

/// Convert IR to node visual (standalone function)
fn ir_to_visual(ir: &ComponentIR) -> NodeVisual {
    let mut visual = NodeVisual::default();

    // Scroll containers should clip their children
    if matches!(ir.kind.as_str(), "Scroll" | "ScrollView" | "ScrollX" | "ScrollY") {
        visual = visual.with_clips_children(true);
    }

    // Process props (where .oui properties go) and style together
    let all_props: Vec<_> = ir.props.iter().chain(ir.style.iter()).collect();

    for prop in &all_props {
        match prop.name.as_str() {
            "background" => {
                if let PropertyValue::String(s) = &prop.value {
                    visual = visual.with_background(hex_to_rgba(s));
                }
            }
            "border" | "border_width" => {
                if let PropertyValue::Number(n) = &prop.value {
                    let border_color = all_props
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
            "clip" | "clips" => {
                if let PropertyValue::Bool(b) = &prop.value {
                    visual = visual.with_clips_children(*b);
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

        // Get physical size and scale factor
        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        // Compute logical size
        let logical_width = size.width as f32 / scale_factor as f32;
        let logical_height = size.height as f32 / scale_factor as f32;

        tracing::info!(
            "DPI: scale_factor={:.2}, physical={}x{}, logical={:.0}x{:.0}",
            scale_factor,
            size.width,
            size.height,
            logical_width,
            logical_height
        );

        let surface_config =
            render_ctx.configure_surface(&surface, size.width.max(1), size.height.max(1));

        // Create primitive renderer (uses physical pixels)
        let primitive_renderer = render_ctx.create_primitive_renderer(surface_config.format);
        primitive_renderer.set_viewport(&render_ctx.queue, size.width as f32, size.height as f32);

        // Create text renderer and system
        let text_renderer = TextRenderer::new(&render_ctx.device, surface_config.format);
        text_renderer.set_viewport(&render_ctx.queue, size.width as f32, size.height as f32);
        let text_system = TextSystem::new();

        // Store sizes and scale factor
        self.viewport_size = (size.width.max(1), size.height.max(1));
        self.logical_size = (logical_width, logical_height);
        self.scale_factor = scale_factor;
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
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                tracing::info!("Scale factor changed: {:.2}", scale_factor);
                self.scale_factor = scale_factor;
                // Recompute logical size
                let (pw, ph) = self.viewport_size;
                self.logical_size = (
                    pw as f32 / scale_factor as f32,
                    ph as f32 / scale_factor as f32,
                );
                // Recompute layout with new logical size
                self.compute_layout();
            }
            WindowEvent::RedrawRequested => {
                // Apply any pending state updates from backend
                self.apply_state_updates();
                self.render();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                // Convert to logical pixels
                let x = position.x as f32 / self.scale_factor as f32;
                let y = position.y as f32 / self.scale_factor as f32;

                if let (Some(tree), Some(root)) = (&self.layout_tree, self.root_node) {
                    let events = self.event_manager.on_mouse_move(x, y, tree, root);
                    self.process_ui_events(&events);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let (x, y) = self.event_manager.mouse_position;
                let btn = match button {
                    WinitMouseButton::Left => MouseButton::Left,
                    WinitMouseButton::Right => MouseButton::Right,
                    WinitMouseButton::Middle => MouseButton::Middle,
                    _ => MouseButton::Left, // Default to left for other buttons
                };

                if let (Some(tree), Some(root)) = (&self.layout_tree, self.root_node) {
                    let events = match state {
                        ElementState::Pressed => self.event_manager.on_mouse_down(x, y, btn, tree, root),
                        ElementState::Released => self.event_manager.on_mouse_up(x, y, btn, tree, root),
                    };
                    self.process_ui_events(&events);
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                self.keyboard_modifiers = KeyboardModifiers {
                    shift: state.shift_key(),
                    ctrl: state.control_key(),
                    alt: state.alt_key(),
                    meta: state.super_key(),
                };
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let modifiers = Modifiers {
                    shift: self.keyboard_modifiers.shift,
                    ctrl: self.keyboard_modifiers.ctrl,
                    alt: self.keyboard_modifiers.alt,
                    meta: self.keyboard_modifiers.meta,
                };

                // Get key string
                let key_str = match &event.logical_key {
                    Key::Named(named) => format!("{:?}", named),
                    Key::Character(c) => c.to_string(),
                    _ => "Unknown".to_string(),
                };

                // Check for F12 to toggle dev overlay
                if event.state == ElementState::Pressed {
                    if let Key::Named(NamedKey::F12) = &event.logical_key {
                        self.dev_overlay.toggle();
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                }

                // First, let text input manager handle it
                if event.state == ElementState::Pressed {
                    let handled = self.text_input_manager.on_key_down(
                        &key_str,
                        modifiers.shift,
                        modifiers.ctrl,
                        modifiers.meta,
                    );
                    if handled {
                        // Text input handled the key, request redraw
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                }

                // If not handled by text input, dispatch to event system
                let events = match event.state {
                    ElementState::Pressed => self.event_manager.on_key_down(key_str, modifiers),
                    ElementState::Released => self.event_manager.on_key_up(key_str, modifiers),
                };
                self.process_ui_events(&events);
            }
            WindowEvent::Ime(ime) => {
                // Handle IME input for international text
                match ime {
                    winit::event::Ime::Commit(text) => {
                        self.text_input_manager.on_text_input(&text);
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl AppState {
    /// Process UI events and execute handlers
    fn process_ui_events(&mut self, events: &[(NodeId, UiEvent)]) {
        // Log events to dev overlay
        for (node, event) in events {
            let event_name = match event {
                UiEvent::Click { x, y, .. } => {
                    // Check if this node has handlers
                    let has_handler = self.event_manager.handlers.contains_key(node);
                    if has_handler {
                        format!("Click({:.0},{:.0})", x, y)
                    } else {
                        format!("Click({:.0},{:.0}) [no handler]", x, y)
                    }
                }
                UiEvent::DoubleClick { .. } => "DoubleClick".to_string(),
                UiEvent::MouseDown { .. } => "MouseDown".to_string(),
                UiEvent::MouseUp { .. } => "MouseUp".to_string(),
                UiEvent::MouseEnter { .. } => "MouseEnter".to_string(),
                UiEvent::MouseLeave => "MouseLeave".to_string(),
                UiEvent::MouseMove { .. } => continue, // Skip move events (too noisy)
                UiEvent::KeyDown { key, .. } => format!("KeyDown({})", key),
                UiEvent::KeyUp { .. } => "KeyUp".to_string(),
                UiEvent::TextInput { text } => format!("TextInput({})", text),
                UiEvent::Focus => "Focus".to_string(),
                UiEvent::Blur => "Blur".to_string(),
            };
            self.dev_overlay.log("EVENT", format!("{} {:?}", event_name, node));
        }

        // Get handlers to execute
        let actions = self.event_manager.dispatch_events(events);
        let mut state_changed = false;

        // Log handler count or lack thereof for click events
        if !actions.is_empty() {
            self.dev_overlay.log("HANDLER", format!("{} handler(s) found", actions.len()));
        } else {
            // Check if any click events had no handlers
            for (node, event) in events {
                if matches!(event, UiEvent::Click { .. }) {
                    if !self.event_manager.handlers.contains_key(node) {
                        self.dev_overlay.log("WARN", format!("No click handler on {:?}", node));
                    }
                }
            }
        }

        // Execute each action
        for (_node, handler) in actions {
            match &handler.action {
                HandlerAction::StateMutation { field, op, value } => {
                    tracing::debug!("State mutation: {} {:?} {:?}", field, op, value);
                    self.dev_overlay.log("STATE", format!("{} {:?} {:?}", field, op, value));
                    // Execute the mutation on reactive state
                    if self.reactive_state.mutate(field, *op, value) {
                        state_changed = true;
                        tracing::info!(
                            "State '{}' updated to: {:?}",
                            field,
                            self.reactive_state.get(field)
                        );
                    } else {
                        tracing::warn!("Failed to mutate state '{}' with {:?}", field, op);
                    }
                }
                HandlerAction::FunctionCall { name, args } => {
                    tracing::debug!("Function call: {}({:?})", name, args);
                    self.dev_overlay.log("CALL", format!("{}({:?})", name, args));
                    // Push command to context for backend to handle
                    if let Some(ctx) = &self.app_context {
                        ctx.push_command(AppCommand::CallFunction {
                            name: name.clone(),
                            args: args.clone(),
                        });
                    }
                }
                HandlerAction::Navigate { path } => {
                    tracing::info!("Navigate to: {}", path);
                    self.dev_overlay.log("NAV", format!("-> {}", path));
                    // Push navigation command to context
                    if let Some(ctx) = &self.app_context {
                        ctx.push_command(AppCommand::Navigate { path: path.clone() });
                    }
                    // Also update reactive state for view switching
                    self.reactive_state.set("view", StateValue::String(path.clone()));
                    state_changed = true;
                }
                HandlerAction::Raw(expr) => {
                    tracing::debug!("Raw handler: {}", expr);
                    self.dev_overlay.log("RAW", expr.clone());
                }
            }
        }

        // Check if state changed and UI needs rebuild
        if state_changed && self.reactive_state.has_changed_since(self.last_state_version) {
            self.last_state_version = self.reactive_state.version();
            // Rebuild UI to reflect state changes
            // For now, we just request a redraw - in the future, we'd do partial updates
            tracing::debug!("State changed, requesting redraw (version {})", self.last_state_version);
        }

        // Request redraw if any events occurred
        if !events.is_empty() || state_changed {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    /// Initialize state from a JSON schema
    ///
    /// Call this to set up initial state values for the application.
    pub fn init_state(&mut self, json: &str) -> Result<(), serde_json::Error> {
        self.reactive_state.init_from_json(json)
    }

    /// Get a reference to the reactive state
    pub fn state(&self) -> &ReactiveState {
        &self.reactive_state
    }

    /// Get a mutable reference to the reactive state
    pub fn state_mut(&mut self) -> &mut ReactiveState {
        &mut self.reactive_state
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
