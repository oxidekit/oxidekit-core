//! Frame timing and FPS metrics

use crate::types::RollingWindow;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Jank threshold: frames taking longer than 16.67ms * 1.5 = 25ms are considered janky
const JANK_THRESHOLD_MS: f64 = 25.0;

/// Target frame time for 60 FPS
const TARGET_FRAME_TIME_MS: f64 = 16.67;

/// Frame timing metrics
#[derive(Debug)]
pub struct FrameMetrics {
    /// Rolling window of frame times (in ms)
    frame_times: RwLock<RollingWindow>,

    /// Total frame count
    frame_count: AtomicU64,

    /// Number of janky frames
    jank_count: AtomicU64,

    /// Current frame start time
    frame_start: RwLock<Option<Instant>>,

    /// Last frame duration
    last_frame_time: RwLock<Duration>,

    /// Layout time in current frame
    layout_time: RwLock<Duration>,

    /// Render time in current frame
    render_time: RwLock<Duration>,

    /// Event processing time
    event_time: RwLock<Duration>,
}

impl FrameMetrics {
    /// Create new frame metrics with given window size
    pub fn new(window_size: usize) -> Self {
        Self {
            frame_times: RwLock::new(RollingWindow::new(window_size)),
            frame_count: AtomicU64::new(0),
            jank_count: AtomicU64::new(0),
            frame_start: RwLock::new(None),
            last_frame_time: RwLock::new(Duration::ZERO),
            layout_time: RwLock::new(Duration::ZERO),
            render_time: RwLock::new(Duration::ZERO),
            event_time: RwLock::new(Duration::ZERO),
        }
    }

    /// Begin timing a new frame
    pub fn begin_frame(&self) {
        *self.frame_start.write() = Some(Instant::now());
        // Reset per-frame timings
        *self.layout_time.write() = Duration::ZERO;
        *self.render_time.write() = Duration::ZERO;
        *self.event_time.write() = Duration::ZERO;
    }

    /// End the current frame and record timing
    pub fn end_frame(&self) {
        if let Some(start) = self.frame_start.write().take() {
            let duration = start.elapsed();
            let ms = duration.as_secs_f64() * 1000.0;

            self.frame_times.write().push(ms);
            self.frame_count.fetch_add(1, Ordering::Relaxed);
            *self.last_frame_time.write() = duration;

            // Check for jank
            if ms > JANK_THRESHOLD_MS {
                self.jank_count.fetch_add(1, Ordering::Relaxed);
                tracing::debug!(frame_time_ms = ms, "Janky frame detected");
            }
        }
    }

    /// Record frame time directly (in milliseconds)
    pub fn record_frame_time(&self, ms: f64) {
        self.frame_times.write().push(ms);
        self.frame_count.fetch_add(1, Ordering::Relaxed);

        if ms > JANK_THRESHOLD_MS {
            self.jank_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record layout computation time
    pub fn record_layout_time(&self, duration: Duration) {
        *self.layout_time.write() = duration;
    }

    /// Record render time
    pub fn record_render_time(&self, duration: Duration) {
        *self.render_time.write() = duration;
    }

    /// Record event processing time
    pub fn record_event_time(&self, duration: Duration) {
        *self.event_time.write() = duration;
    }

    /// Get current FPS (frames per second)
    pub fn fps(&self) -> f64 {
        let avg_ms = self.frame_times.read().average();
        if avg_ms <= 0.0 {
            0.0
        } else {
            1000.0 / avg_ms
        }
    }

    /// Get average frame time in milliseconds
    pub fn avg_frame_time_ms(&self) -> f64 {
        self.frame_times.read().average()
    }

    /// Get minimum frame time in milliseconds
    pub fn min_frame_time_ms(&self) -> f64 {
        self.frame_times.read().min()
    }

    /// Get maximum frame time in milliseconds
    pub fn max_frame_time_ms(&self) -> f64 {
        self.frame_times.read().max()
    }

    /// Get last frame time
    pub fn last_frame_time(&self) -> Duration {
        *self.last_frame_time.read()
    }

    /// Get layout time for last frame
    pub fn layout_time(&self) -> Duration {
        *self.layout_time.read()
    }

    /// Get render time for last frame
    pub fn render_time(&self) -> Duration {
        *self.render_time.read()
    }

    /// Get event processing time for last frame
    pub fn event_time(&self) -> Duration {
        *self.event_time.read()
    }

    /// Get total frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(Ordering::Relaxed)
    }

    /// Get number of janky frames
    pub fn jank_count(&self) -> u64 {
        self.jank_count.load(Ordering::Relaxed)
    }

    /// Get jank percentage
    pub fn jank_percentage(&self) -> f64 {
        let total = self.frame_count() as f64;
        if total <= 0.0 {
            0.0
        } else {
            (self.jank_count() as f64 / total) * 100.0
        }
    }

    /// Check if app is meeting target frame rate
    pub fn is_smooth(&self) -> bool {
        self.fps() >= 55.0 && self.jank_percentage() < 5.0
    }

    /// Get frame budget utilization (percentage of 16.67ms used)
    pub fn budget_utilization(&self) -> f64 {
        let avg = self.avg_frame_time_ms();
        (avg / TARGET_FRAME_TIME_MS) * 100.0
    }

    /// Get recent frame times for visualization
    pub fn frame_time_history(&self) -> Vec<f64> {
        self.frame_times.read().values()
    }

    /// Get current frame breakdown
    pub fn frame_breakdown(&self) -> FrameBreakdown {
        FrameBreakdown {
            total_ms: self.last_frame_time().as_secs_f64() * 1000.0,
            layout_ms: self.layout_time().as_secs_f64() * 1000.0,
            render_ms: self.render_time().as_secs_f64() * 1000.0,
            event_ms: self.event_time().as_secs_f64() * 1000.0,
        }
    }

    /// Reset all frame metrics
    pub fn reset(&self) {
        *self.frame_times.write() = RollingWindow::new(120);
        self.frame_count.store(0, Ordering::Relaxed);
        self.jank_count.store(0, Ordering::Relaxed);
        *self.frame_start.write() = None;
        *self.last_frame_time.write() = Duration::ZERO;
        *self.layout_time.write() = Duration::ZERO;
        *self.render_time.write() = Duration::ZERO;
        *self.event_time.write() = Duration::ZERO;
    }
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self::new(120)
    }
}

/// Breakdown of time spent in a frame
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FrameBreakdown {
    /// Total frame time in ms
    pub total_ms: f64,
    /// Layout computation time in ms
    pub layout_ms: f64,
    /// Render time in ms
    pub render_ms: f64,
    /// Event processing time in ms
    pub event_ms: f64,
}

impl FrameBreakdown {
    /// Get "other" time (not accounted for by layout, render, event)
    pub fn other_ms(&self) -> f64 {
        (self.total_ms - self.layout_ms - self.render_ms - self.event_ms).max(0.0)
    }

    /// Get layout percentage
    pub fn layout_percent(&self) -> f64 {
        if self.total_ms <= 0.0 {
            0.0
        } else {
            (self.layout_ms / self.total_ms) * 100.0
        }
    }

    /// Get render percentage
    pub fn render_percent(&self) -> f64 {
        if self.total_ms <= 0.0 {
            0.0
        } else {
            (self.render_ms / self.total_ms) * 100.0
        }
    }
}

/// Frame timing guard - automatically records frame time on drop
pub struct FrameGuard<'a> {
    metrics: &'a FrameMetrics,
}

impl<'a> FrameGuard<'a> {
    /// Begin a new frame
    pub fn begin(metrics: &'a FrameMetrics) -> Self {
        metrics.begin_frame();
        Self { metrics }
    }
}

impl<'a> Drop for FrameGuard<'a> {
    fn drop(&mut self) {
        self.metrics.end_frame();
    }
}

/// Layout timing guard
pub struct LayoutGuard<'a> {
    metrics: &'a FrameMetrics,
    start: Instant,
}

impl<'a> LayoutGuard<'a> {
    /// Begin timing layout
    pub fn begin(metrics: &'a FrameMetrics) -> Self {
        Self {
            metrics,
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for LayoutGuard<'a> {
    fn drop(&mut self) {
        self.metrics.record_layout_time(self.start.elapsed());
    }
}

/// Render timing guard
pub struct RenderGuard<'a> {
    metrics: &'a FrameMetrics,
    start: Instant,
}

impl<'a> RenderGuard<'a> {
    /// Begin timing render
    pub fn begin(metrics: &'a FrameMetrics) -> Self {
        Self {
            metrics,
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for RenderGuard<'a> {
    fn drop(&mut self) {
        self.metrics.record_render_time(self.start.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_frame_timing() {
        let metrics = FrameMetrics::new(10);

        metrics.begin_frame();
        sleep(Duration::from_millis(10));
        metrics.end_frame();

        assert_eq!(metrics.frame_count(), 1);
        assert!(metrics.avg_frame_time_ms() >= 10.0);
    }

    #[test]
    fn test_fps_calculation() {
        let metrics = FrameMetrics::new(10);

        // Record 10 frames at 16.67ms each (60fps)
        for _ in 0..10 {
            metrics.record_frame_time(16.67);
        }

        let fps = metrics.fps();
        assert!((fps - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_jank_detection() {
        let metrics = FrameMetrics::new(10);

        // Record some normal frames
        metrics.record_frame_time(16.0);
        metrics.record_frame_time(16.0);

        // Record a janky frame
        metrics.record_frame_time(30.0);

        assert_eq!(metrics.jank_count(), 1);
    }

    #[test]
    fn test_frame_guard() {
        let metrics = FrameMetrics::new(10);

        {
            let _guard = FrameGuard::begin(&metrics);
            sleep(Duration::from_millis(5));
        }

        assert_eq!(metrics.frame_count(), 1);
        assert!(metrics.last_frame_time() >= Duration::from_millis(5));
    }
}
