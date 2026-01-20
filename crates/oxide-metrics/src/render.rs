//! Render pipeline metrics

use crate::types::RollingWindow;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Render statistics snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderStats {
    /// Draw calls in current/last frame
    pub draw_calls: u64,
    /// Triangles rendered
    pub triangles: u64,
    /// Vertices processed
    pub vertices: u64,
    /// GPU time in milliseconds
    pub gpu_time_ms: f64,
    /// Pipeline state changes
    pub state_changes: u64,
    /// Texture bindings
    pub texture_binds: u64,
    /// Buffer updates
    pub buffer_updates: u64,
    /// Shader compilations (should be rare after startup)
    pub shader_compiles: u64,
    /// Total frames rendered
    pub total_frames: u64,
}

impl RenderStats {
    /// Vertices per triangle (should be ~3 for indexed geometry)
    pub fn vertices_per_triangle(&self) -> f64 {
        if self.triangles == 0 {
            0.0
        } else {
            self.vertices as f64 / self.triangles as f64
        }
    }

    /// Triangles per draw call
    pub fn triangles_per_draw(&self) -> f64 {
        if self.draw_calls == 0 {
            0.0
        } else {
            self.triangles as f64 / self.draw_calls as f64
        }
    }

    /// Is batching efficient? (more triangles per draw = better)
    pub fn is_well_batched(&self) -> bool {
        self.triangles_per_draw() > 100.0
    }
}

/// Render metrics collector
#[derive(Debug)]
pub struct RenderMetrics {
    /// Draw calls this frame
    draw_calls: AtomicU64,
    /// Triangles this frame
    triangles: AtomicU64,
    /// Vertices this frame
    vertices: AtomicU64,
    /// GPU time history (rolling window, in ms)
    gpu_times: RwLock<RollingWindow>,
    /// Current frame's GPU time
    current_gpu_time: RwLock<Duration>,
    /// State changes this frame
    state_changes: AtomicU64,
    /// Texture binds this frame
    texture_binds: AtomicU64,
    /// Buffer updates this frame
    buffer_updates: AtomicU64,
    /// Shader compilations (cumulative)
    shader_compiles: AtomicU64,
    /// Total frames rendered
    total_frames: AtomicU64,
    /// Per-pipeline statistics
    pipeline_stats: RwLock<HashMap<String, PipelineStats>>,
    /// Frame timing breakdown
    frame_timing: RwLock<FrameTiming>,
}

/// Statistics for a specific render pipeline
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineStats {
    /// Pipeline identifier
    pub name: String,
    /// Number of times used
    pub use_count: u64,
    /// Average GPU time when active (ms)
    pub avg_gpu_time_ms: f64,
    /// Total draw calls through this pipeline
    pub draw_calls: u64,
    /// Total triangles through this pipeline
    pub triangles: u64,
}

/// Detailed frame timing breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameTiming {
    /// Command buffer recording time (ms)
    pub command_recording_ms: f64,
    /// Command buffer submission time (ms)
    pub submit_ms: f64,
    /// Present time (ms)
    pub present_ms: f64,
    /// GPU execution time (ms)
    pub gpu_ms: f64,
    /// CPU-GPU synchronization time (ms)
    pub sync_ms: f64,
}

impl FrameTiming {
    /// Total frame time on CPU side
    pub fn total_cpu_ms(&self) -> f64 {
        self.command_recording_ms + self.submit_ms + self.present_ms + self.sync_ms
    }
}

impl RenderMetrics {
    /// Create new render metrics
    pub fn new() -> Self {
        Self {
            draw_calls: AtomicU64::new(0),
            triangles: AtomicU64::new(0),
            vertices: AtomicU64::new(0),
            gpu_times: RwLock::new(RollingWindow::new(120)),
            current_gpu_time: RwLock::new(Duration::ZERO),
            state_changes: AtomicU64::new(0),
            texture_binds: AtomicU64::new(0),
            buffer_updates: AtomicU64::new(0),
            shader_compiles: AtomicU64::new(0),
            total_frames: AtomicU64::new(0),
            pipeline_stats: RwLock::new(HashMap::new()),
            frame_timing: RwLock::new(FrameTiming::default()),
        }
    }

    /// Begin a new frame (reset per-frame counters)
    pub fn begin_frame(&self) {
        self.draw_calls.store(0, Ordering::Relaxed);
        self.triangles.store(0, Ordering::Relaxed);
        self.vertices.store(0, Ordering::Relaxed);
        self.state_changes.store(0, Ordering::Relaxed);
        self.texture_binds.store(0, Ordering::Relaxed);
        self.buffer_updates.store(0, Ordering::Relaxed);
        *self.current_gpu_time.write() = Duration::ZERO;
        *self.frame_timing.write() = FrameTiming::default();
    }

    /// End the current frame
    pub fn end_frame(&self) {
        self.total_frames.fetch_add(1, Ordering::Relaxed);

        // Record GPU time
        let gpu_time = self.current_gpu_time.read();
        self.gpu_times
            .write()
            .push(gpu_time.as_secs_f64() * 1000.0);
    }

    /// Record a draw call
    pub fn record_draw_call(&self, triangles: u64, vertices: u64) {
        self.draw_calls.fetch_add(1, Ordering::Relaxed);
        self.triangles.fetch_add(triangles, Ordering::Relaxed);
        self.vertices.fetch_add(vertices, Ordering::Relaxed);
    }

    /// Record a draw call through a specific pipeline
    pub fn record_pipeline_draw(&self, pipeline: &str, triangles: u64, vertices: u64, gpu_time: Duration) {
        self.record_draw_call(triangles, vertices);

        let mut stats = self.pipeline_stats.write();
        let pipeline_stat = stats.entry(pipeline.to_string()).or_insert_with(|| PipelineStats {
            name: pipeline.to_string(),
            ..Default::default()
        });

        pipeline_stat.use_count += 1;
        pipeline_stat.draw_calls += 1;
        pipeline_stat.triangles += triangles;

        // Update rolling average GPU time
        let gpu_ms = gpu_time.as_secs_f64() * 1000.0;
        let n = pipeline_stat.use_count as f64;
        pipeline_stat.avg_gpu_time_ms =
            (pipeline_stat.avg_gpu_time_ms * (n - 1.0) + gpu_ms) / n;
    }

    /// Record a state change
    pub fn record_state_change(&self) {
        self.state_changes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a texture bind
    pub fn record_texture_bind(&self) {
        self.texture_binds.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a buffer update
    pub fn record_buffer_update(&self) {
        self.buffer_updates.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a shader compilation
    pub fn record_shader_compile(&self) {
        self.shader_compiles.fetch_add(1, Ordering::Relaxed);
        tracing::debug!("Shader compiled");
    }

    /// Record GPU execution time
    pub fn record_gpu_time(&self, duration: Duration) {
        *self.current_gpu_time.write() = duration;
    }

    /// Record frame timing breakdown
    pub fn record_frame_timing(&self, timing: FrameTiming) {
        *self.frame_timing.write() = timing;
    }

    /// Get current render statistics
    pub fn current_stats(&self) -> RenderStats {
        RenderStats {
            draw_calls: self.draw_calls.load(Ordering::Relaxed),
            triangles: self.triangles.load(Ordering::Relaxed),
            vertices: self.vertices.load(Ordering::Relaxed),
            gpu_time_ms: self.gpu_times.read().average(),
            state_changes: self.state_changes.load(Ordering::Relaxed),
            texture_binds: self.texture_binds.load(Ordering::Relaxed),
            buffer_updates: self.buffer_updates.load(Ordering::Relaxed),
            shader_compiles: self.shader_compiles.load(Ordering::Relaxed),
            total_frames: self.total_frames.load(Ordering::Relaxed),
        }
    }

    /// Get per-pipeline statistics
    pub fn pipeline_stats(&self) -> HashMap<String, PipelineStats> {
        self.pipeline_stats.read().clone()
    }

    /// Get current frame timing
    pub fn frame_timing(&self) -> FrameTiming {
        self.frame_timing.read().clone()
    }

    /// Get GPU time history for visualization
    pub fn gpu_time_history(&self) -> Vec<f64> {
        self.gpu_times.read().values()
    }

    /// Get average GPU time in ms
    pub fn avg_gpu_time_ms(&self) -> f64 {
        self.gpu_times.read().average()
    }

    /// Get max GPU time in ms
    pub fn max_gpu_time_ms(&self) -> f64 {
        self.gpu_times.read().max()
    }

    /// Get hottest pipelines (by GPU time)
    pub fn hottest_pipelines(&self, limit: usize) -> Vec<PipelineStats> {
        let mut pipelines: Vec<_> = self.pipeline_stats.read().values().cloned().collect();
        pipelines.sort_by(|a, b| {
            b.avg_gpu_time_ms
                .partial_cmp(&a.avg_gpu_time_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        pipelines.truncate(limit);
        pipelines
    }

    /// Check if there are too many draw calls
    pub fn too_many_draw_calls(&self) -> bool {
        self.draw_calls.load(Ordering::Relaxed) > 1000
    }

    /// Check if batching could be improved
    pub fn batching_advisory(&self) -> Option<String> {
        let stats = self.current_stats();
        if stats.draw_calls > 100 && stats.triangles_per_draw() < 50.0 {
            Some(format!(
                "Consider batching: {} draw calls with only {:.1} triangles per call",
                stats.draw_calls,
                stats.triangles_per_draw()
            ))
        } else {
            None
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.draw_calls.store(0, Ordering::Relaxed);
        self.triangles.store(0, Ordering::Relaxed);
        self.vertices.store(0, Ordering::Relaxed);
        *self.gpu_times.write() = RollingWindow::new(120);
        *self.current_gpu_time.write() = Duration::ZERO;
        self.state_changes.store(0, Ordering::Relaxed);
        self.texture_binds.store(0, Ordering::Relaxed);
        self.buffer_updates.store(0, Ordering::Relaxed);
        self.shader_compiles.store(0, Ordering::Relaxed);
        self.total_frames.store(0, Ordering::Relaxed);
        self.pipeline_stats.write().clear();
        *self.frame_timing.write() = FrameTiming::default();
    }
}

impl Default for RenderMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Render frame guard - handles begin/end frame automatically
pub struct RenderFrameGuard<'a> {
    metrics: &'a RenderMetrics,
}

impl<'a> RenderFrameGuard<'a> {
    /// Begin a new render frame
    pub fn begin(metrics: &'a RenderMetrics) -> Self {
        metrics.begin_frame();
        Self { metrics }
    }
}

impl<'a> Drop for RenderFrameGuard<'a> {
    fn drop(&mut self) {
        self.metrics.end_frame();
    }
}

/// GPU timing guard
pub struct GpuTimingGuard<'a> {
    metrics: &'a RenderMetrics,
    start: Instant,
}

impl<'a> GpuTimingGuard<'a> {
    /// Start timing GPU work
    pub fn start(metrics: &'a RenderMetrics) -> Self {
        Self {
            metrics,
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for GpuTimingGuard<'a> {
    fn drop(&mut self) {
        self.metrics.record_gpu_time(self.start.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_metrics() {
        let metrics = RenderMetrics::new();

        metrics.begin_frame();
        metrics.record_draw_call(100, 300);
        metrics.record_draw_call(50, 150);
        metrics.record_texture_bind();
        metrics.record_state_change();
        metrics.end_frame();

        let stats = metrics.current_stats();
        assert_eq!(stats.draw_calls, 2);
        assert_eq!(stats.triangles, 150);
        assert_eq!(stats.vertices, 450);
        assert_eq!(stats.texture_binds, 1);
        assert_eq!(stats.state_changes, 1);
        assert_eq!(stats.total_frames, 1);
    }

    #[test]
    fn test_pipeline_stats() {
        let metrics = RenderMetrics::new();

        metrics.record_pipeline_draw("main", 100, 300, Duration::from_micros(500));
        metrics.record_pipeline_draw("main", 200, 600, Duration::from_micros(700));

        let pipelines = metrics.pipeline_stats();
        let main = pipelines.get("main").unwrap();
        assert_eq!(main.use_count, 2);
        assert_eq!(main.triangles, 300);
    }

    #[test]
    fn test_batching_check() {
        let metrics = RenderMetrics::new();

        // Simulate many small draw calls
        for _ in 0..200 {
            metrics.record_draw_call(10, 30);
        }

        assert!(metrics.batching_advisory().is_some());
    }

    #[test]
    fn test_render_frame_guard() {
        let metrics = RenderMetrics::new();

        {
            let _guard = RenderFrameGuard::begin(&metrics);
            metrics.record_draw_call(100, 300);
        }

        let stats = metrics.current_stats();
        assert_eq!(stats.total_frames, 1);
    }
}
