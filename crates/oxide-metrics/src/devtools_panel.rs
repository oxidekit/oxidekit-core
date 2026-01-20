//! Devtools metrics panels
//!
//! Feature-gated behind `devtools-panel` feature flag.
//! Provides data structures and utilities for rendering metrics in devtools UI.

use crate::MetricsRegistry;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum data points to retain for visualization
const MAX_DATA_POINTS: usize = 300;

/// Devtools metrics panel manager
pub struct DevtoolsPanel {
    /// Registry reference
    registry: &'static MetricsRegistry,
    /// Frame time history for graph
    frame_time_history: VecDeque<f64>,
    /// FPS history for graph
    fps_history: VecDeque<f64>,
    /// Memory history for graph
    memory_history: VecDeque<u64>,
    /// GPU time history for graph
    gpu_time_history: VecDeque<f64>,
    /// Panel visibility states
    panel_states: PanelStates,
}

/// Panel visibility states
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PanelStates {
    /// Show FPS panel
    pub fps_panel: bool,
    /// Show memory panel
    pub memory_panel: bool,
    /// Show render panel
    pub render_panel: bool,
    /// Show IO panel
    pub io_panel: bool,
    /// Show detailed frame breakdown
    pub frame_breakdown: bool,
    /// Expanded view
    pub expanded: bool,
}

impl DevtoolsPanel {
    /// Create a new devtools panel
    pub fn new() -> Self {
        Self {
            registry: MetricsRegistry::global(),
            frame_time_history: VecDeque::with_capacity(MAX_DATA_POINTS),
            fps_history: VecDeque::with_capacity(MAX_DATA_POINTS),
            memory_history: VecDeque::with_capacity(MAX_DATA_POINTS),
            gpu_time_history: VecDeque::with_capacity(MAX_DATA_POINTS),
            panel_states: PanelStates {
                fps_panel: true,
                memory_panel: true,
                render_panel: false,
                io_panel: false,
                frame_breakdown: false,
                expanded: false,
            },
        }
    }

    /// Update history with current values
    pub fn tick(&mut self) {
        let frame_metrics = self.registry.frame_metrics();
        let memory_metrics = self.registry.memory_metrics();
        let render_metrics = self.registry.render_metrics();

        // Add to histories
        self.push_frame_time(frame_metrics.avg_frame_time_ms());
        self.push_fps(frame_metrics.fps());
        self.push_memory(memory_metrics.heap_used());
        self.push_gpu_time(render_metrics.avg_gpu_time_ms());
    }

    fn push_frame_time(&mut self, value: f64) {
        if self.frame_time_history.len() >= MAX_DATA_POINTS {
            self.frame_time_history.pop_front();
        }
        self.frame_time_history.push_back(value);
    }

    fn push_fps(&mut self, value: f64) {
        if self.fps_history.len() >= MAX_DATA_POINTS {
            self.fps_history.pop_front();
        }
        self.fps_history.push_back(value);
    }

    fn push_memory(&mut self, value: u64) {
        if self.memory_history.len() >= MAX_DATA_POINTS {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(value);
    }

    fn push_gpu_time(&mut self, value: f64) {
        if self.gpu_time_history.len() >= MAX_DATA_POINTS {
            self.gpu_time_history.pop_front();
        }
        self.gpu_time_history.push_back(value);
    }

    /// Get panel states
    pub fn panel_states(&self) -> &PanelStates {
        &self.panel_states
    }

    /// Set panel states
    pub fn set_panel_states(&mut self, states: PanelStates) {
        self.panel_states = states;
    }

    /// Toggle a specific panel
    pub fn toggle_panel(&mut self, panel: PanelType) {
        match panel {
            PanelType::Fps => self.panel_states.fps_panel = !self.panel_states.fps_panel,
            PanelType::Memory => self.panel_states.memory_panel = !self.panel_states.memory_panel,
            PanelType::Render => self.panel_states.render_panel = !self.panel_states.render_panel,
            PanelType::Io => self.panel_states.io_panel = !self.panel_states.io_panel,
            PanelType::FrameBreakdown => self.panel_states.frame_breakdown = !self.panel_states.frame_breakdown,
        }
    }

    /// Get FPS panel data
    pub fn fps_panel_data(&self) -> FpsPanelData {
        let frame_metrics = self.registry.frame_metrics();

        FpsPanelData {
            current_fps: frame_metrics.fps(),
            avg_fps: self.fps_history.iter().sum::<f64>() / self.fps_history.len().max(1) as f64,
            min_fps: self.fps_history.iter().cloned().fold(f64::INFINITY, f64::min),
            max_fps: self.fps_history.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            frame_time_ms: frame_metrics.avg_frame_time_ms(),
            jank_count: frame_metrics.jank_count(),
            jank_percent: frame_metrics.jank_percentage(),
            is_smooth: frame_metrics.is_smooth(),
            history: self.fps_history.iter().cloned().collect(),
            frame_time_history: self.frame_time_history.iter().cloned().collect(),
        }
    }

    /// Get memory panel data
    pub fn memory_panel_data(&self) -> MemoryPanelData {
        let memory_metrics = self.registry.memory_metrics();
        let stats = memory_metrics.current_stats();

        MemoryPanelData {
            heap_bytes: stats.heap_used,
            heap_formatted: stats.format_heap(),
            peak_bytes: stats.peak,
            allocations: stats.allocation_count,
            deallocations: stats.deallocation_count,
            active_allocations: stats.active_allocations(),
            gpu_bytes: stats.gpu_memory,
            gpu_formatted: stats.format_gpu(),
            potential_leak: memory_metrics.potential_leak(),
            trend_percent: memory_metrics.heap_trend(),
            history: self.memory_history.iter().cloned().collect(),
        }
    }

    /// Get render panel data
    pub fn render_panel_data(&self) -> RenderPanelData {
        let render_metrics = self.registry.render_metrics();
        let stats = render_metrics.current_stats();

        RenderPanelData {
            draw_calls: stats.draw_calls,
            triangles: stats.triangles,
            vertices: stats.vertices,
            gpu_time_ms: render_metrics.avg_gpu_time_ms(),
            max_gpu_time_ms: render_metrics.max_gpu_time_ms(),
            state_changes: stats.state_changes,
            texture_binds: stats.texture_binds,
            buffer_updates: stats.buffer_updates,
            shader_compiles: stats.shader_compiles,
            total_frames: stats.total_frames,
            batching_advisory: render_metrics.batching_advisory(),
            gpu_time_history: self.gpu_time_history.iter().cloned().collect(),
        }
    }

    /// Get IO panel data
    pub fn io_panel_data(&self) -> IoPanelData {
        let io_metrics = self.registry.io_metrics();
        let stats = io_metrics.current_stats();

        IoPanelData {
            bytes_read: stats.bytes_read,
            bytes_written: stats.bytes_written,
            read_ops: stats.read_ops,
            write_ops: stats.write_ops,
            avg_read_latency_ms: stats.avg_read_latency_ms,
            avg_write_latency_ms: stats.avg_write_latency_ms,
            network_requests: stats.network_requests,
            network_bytes_in: stats.network_bytes_in,
            network_bytes_out: stats.network_bytes_out,
            failed_ops: stats.failed_ops,
            success_rate: stats.success_rate(),
            slowest_endpoints: io_metrics.slowest_endpoints(5),
        }
    }

    /// Get frame breakdown data
    pub fn frame_breakdown_data(&self) -> FrameBreakdownData {
        let frame_metrics = self.registry.frame_metrics();
        let breakdown = frame_metrics.frame_breakdown();

        FrameBreakdownData {
            total_ms: breakdown.total_ms,
            layout_ms: breakdown.layout_ms,
            layout_percent: breakdown.layout_percent(),
            render_ms: breakdown.render_ms,
            render_percent: breakdown.render_percent(),
            event_ms: breakdown.event_ms,
            other_ms: breakdown.other_ms(),
            budget_utilization: frame_metrics.budget_utilization(),
            over_budget: breakdown.total_ms > 16.67,
        }
    }

    /// Get all panel data as a complete dashboard
    pub fn dashboard_data(&self) -> DashboardData {
        DashboardData {
            fps: self.fps_panel_data(),
            memory: self.memory_panel_data(),
            render: self.render_panel_data(),
            io: self.io_panel_data(),
            frame_breakdown: self.frame_breakdown_data(),
        }
    }

    /// Export current state as JSON (for debugging or external tools)
    pub fn export_json(&self) -> String {
        let data = self.dashboard_data();
        serde_json::to_string_pretty(&data).unwrap_or_default()
    }
}

impl Default for DevtoolsPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Panel types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelType {
    Fps,
    Memory,
    Render,
    Io,
    FrameBreakdown,
}

/// FPS panel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpsPanelData {
    pub current_fps: f64,
    pub avg_fps: f64,
    pub min_fps: f64,
    pub max_fps: f64,
    pub frame_time_ms: f64,
    pub jank_count: u64,
    pub jank_percent: f64,
    pub is_smooth: bool,
    pub history: Vec<f64>,
    pub frame_time_history: Vec<f64>,
}

/// Memory panel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPanelData {
    pub heap_bytes: u64,
    pub heap_formatted: String,
    pub peak_bytes: u64,
    pub allocations: u64,
    pub deallocations: u64,
    pub active_allocations: u64,
    pub gpu_bytes: u64,
    pub gpu_formatted: String,
    pub potential_leak: bool,
    pub trend_percent: f64,
    pub history: Vec<u64>,
}

/// Render panel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderPanelData {
    pub draw_calls: u64,
    pub triangles: u64,
    pub vertices: u64,
    pub gpu_time_ms: f64,
    pub max_gpu_time_ms: f64,
    pub state_changes: u64,
    pub texture_binds: u64,
    pub buffer_updates: u64,
    pub shader_compiles: u64,
    pub total_frames: u64,
    pub batching_advisory: Option<String>,
    pub gpu_time_history: Vec<f64>,
}

/// IO panel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoPanelData {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub read_ops: u64,
    pub write_ops: u64,
    pub avg_read_latency_ms: f64,
    pub avg_write_latency_ms: f64,
    pub network_requests: u64,
    pub network_bytes_in: u64,
    pub network_bytes_out: u64,
    pub failed_ops: u64,
    pub success_rate: f64,
    pub slowest_endpoints: Vec<crate::io::EndpointStats>,
}

/// Frame breakdown data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameBreakdownData {
    pub total_ms: f64,
    pub layout_ms: f64,
    pub layout_percent: f64,
    pub render_ms: f64,
    pub render_percent: f64,
    pub event_ms: f64,
    pub other_ms: f64,
    pub budget_utilization: f64,
    pub over_budget: bool,
}

/// Complete dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub fps: FpsPanelData,
    pub memory: MemoryPanelData,
    pub render: RenderPanelData,
    pub io: IoPanelData,
    pub frame_breakdown: FrameBreakdownData,
}

/// Helper for rendering sparkline graphs
pub struct Sparkline {
    data: Vec<f64>,
    width: usize,
    height: usize,
    min_val: f64,
    max_val: f64,
}

impl Sparkline {
    /// Create a new sparkline from data
    pub fn new(data: Vec<f64>, width: usize, height: usize) -> Self {
        let min_val = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Self {
            data,
            width,
            height,
            min_val,
            max_val,
        }
    }

    /// Get normalized data points (0.0 to 1.0)
    pub fn normalized(&self) -> Vec<f64> {
        let range = self.max_val - self.min_val;
        if range == 0.0 {
            return vec![0.5; self.data.len()];
        }

        self.data
            .iter()
            .map(|&v| (v - self.min_val) / range)
            .collect()
    }

    /// Get points for SVG rendering
    pub fn svg_points(&self) -> Vec<(f64, f64)> {
        let normalized = self.normalized();
        let step = self.width as f64 / normalized.len().max(1) as f64;

        normalized
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = i as f64 * step;
                let y = self.height as f64 * (1.0 - v);
                (x, y)
            })
            .collect()
    }

    /// Render as ASCII art (for terminal display)
    pub fn to_ascii(&self) -> String {
        let chars = ['_', '.', '-', '~', '^'];
        let normalized = self.normalized();

        normalized
            .iter()
            .map(|&v| {
                let idx = (v * (chars.len() - 1) as f64).round() as usize;
                chars[idx.min(chars.len() - 1)]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devtools_panel() {
        let mut panel = DevtoolsPanel::new();

        // Simulate a few ticks
        for _ in 0..10 {
            panel.tick();
        }

        let fps_data = panel.fps_panel_data();
        assert!(fps_data.current_fps >= 0.0);

        let dashboard = panel.dashboard_data();
        assert!(dashboard.fps.current_fps >= 0.0);
    }

    #[test]
    fn test_panel_toggle() {
        let mut panel = DevtoolsPanel::new();

        assert!(panel.panel_states.fps_panel);
        panel.toggle_panel(PanelType::Fps);
        assert!(!panel.panel_states.fps_panel);
    }

    #[test]
    fn test_sparkline() {
        let data = vec![1.0, 2.0, 3.0, 2.0, 1.0];
        let sparkline = Sparkline::new(data, 100, 20);

        let points = sparkline.svg_points();
        assert_eq!(points.len(), 5);

        let ascii = sparkline.to_ascii();
        assert_eq!(ascii.len(), 5);
    }

    #[test]
    fn test_export_json() {
        let panel = DevtoolsPanel::new();
        let json = panel.export_json();
        assert!(json.contains("fps"));
        assert!(json.contains("memory"));
    }
}
