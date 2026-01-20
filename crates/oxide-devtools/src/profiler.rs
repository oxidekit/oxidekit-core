//! Performance Profiler
//!
//! Provides detailed performance metrics for frame timing, layout, rendering,
//! and custom spans for profiling specific code sections.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance profiler for tracking frame times and custom spans
#[derive(Debug)]
pub struct Profiler {
    /// Historical frame times
    frame_times: Vec<FrameRecord>,
    /// Current frame being recorded
    current_frame: Option<FrameRecorder>,
    /// Named timing spans
    spans: HashMap<String, SpanRecorder>,
    /// Historical span data
    span_history: HashMap<String, Vec<Duration>>,
    /// Maximum history size
    max_history: usize,
    /// Profiler enabled state
    enabled: bool,
}

/// Record of a completed frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameRecord {
    /// Total frame time
    pub total: Duration,
    /// Layout computation time
    pub layout: Duration,
    /// Style resolution time
    pub style: Duration,
    /// Render time
    pub render: Duration,
    /// Event processing time
    pub events: Duration,
    /// Number of components
    pub component_count: usize,
    /// Number of layout nodes
    pub layout_node_count: usize,
    /// Frame timestamp
    pub timestamp: std::time::SystemTime,
}

impl Default for FrameRecord {
    fn default() -> Self {
        Self {
            total: Duration::ZERO,
            layout: Duration::ZERO,
            style: Duration::ZERO,
            render: Duration::ZERO,
            events: Duration::ZERO,
            component_count: 0,
            layout_node_count: 0,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// Active frame being recorded
#[derive(Debug)]
struct FrameRecorder {
    start: Instant,
    layout_start: Option<Instant>,
    layout_duration: Duration,
    style_start: Option<Instant>,
    style_duration: Duration,
    render_start: Option<Instant>,
    render_duration: Duration,
    events_start: Option<Instant>,
    events_duration: Duration,
    component_count: usize,
    layout_node_count: usize,
}

impl FrameRecorder {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            layout_start: None,
            layout_duration: Duration::ZERO,
            style_start: None,
            style_duration: Duration::ZERO,
            render_start: None,
            render_duration: Duration::ZERO,
            events_start: None,
            events_duration: Duration::ZERO,
            component_count: 0,
            layout_node_count: 0,
        }
    }

    fn finish(self) -> FrameRecord {
        FrameRecord {
            total: self.start.elapsed(),
            layout: self.layout_duration,
            style: self.style_duration,
            render: self.render_duration,
            events: self.events_duration,
            component_count: self.component_count,
            layout_node_count: self.layout_node_count,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// Active span being recorded
#[derive(Debug)]
struct SpanRecorder {
    start: Instant,
    parent: Option<String>,
}

impl Profiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(120),
            current_frame: None,
            spans: HashMap::new(),
            span_history: HashMap::new(),
            max_history: 120, // 2 seconds at 60fps
            enabled: true,
        }
    }

    /// Create a profiler with custom history size
    pub fn with_history_size(max_history: usize) -> Self {
        Self {
            frame_times: Vec::with_capacity(max_history),
            current_frame: None,
            spans: HashMap::new(),
            span_history: HashMap::new(),
            max_history,
            enabled: true,
        }
    }

    /// Enable or disable the profiler
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if profiler is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start timing a new frame
    pub fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }
        self.current_frame = Some(FrameRecorder::new());
    }

    /// End the current frame
    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }
        if let Some(frame) = self.current_frame.take() {
            let record = frame.finish();
            self.frame_times.push(record);

            // Keep within max history
            while self.frame_times.len() > self.max_history {
                self.frame_times.remove(0);
            }
        }
    }

    /// Begin layout timing within current frame
    pub fn begin_layout(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            frame.layout_start = Some(Instant::now());
        }
    }

    /// End layout timing
    pub fn end_layout(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            if let Some(start) = frame.layout_start.take() {
                frame.layout_duration += start.elapsed();
            }
        }
    }

    /// Begin style resolution timing
    pub fn begin_style(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            frame.style_start = Some(Instant::now());
        }
    }

    /// End style resolution timing
    pub fn end_style(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            if let Some(start) = frame.style_start.take() {
                frame.style_duration += start.elapsed();
            }
        }
    }

    /// Begin render timing
    pub fn begin_render(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            frame.render_start = Some(Instant::now());
        }
    }

    /// End render timing
    pub fn end_render(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            if let Some(start) = frame.render_start.take() {
                frame.render_duration += start.elapsed();
            }
        }
    }

    /// Begin event processing timing
    pub fn begin_events(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            frame.events_start = Some(Instant::now());
        }
    }

    /// End event processing timing
    pub fn end_events(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            if let Some(start) = frame.events_start.take() {
                frame.events_duration += start.elapsed();
            }
        }
    }

    /// Set component count for current frame
    pub fn set_component_count(&mut self, count: usize) {
        if let Some(ref mut frame) = self.current_frame {
            frame.component_count = count;
        }
    }

    /// Set layout node count for current frame
    pub fn set_layout_node_count(&mut self, count: usize) {
        if let Some(ref mut frame) = self.current_frame {
            frame.layout_node_count = count;
        }
    }

    /// Start a named timing span
    pub fn begin_span(&mut self, name: &str) {
        if !self.enabled {
            return;
        }
        self.spans.insert(
            name.to_string(),
            SpanRecorder {
                start: Instant::now(),
                parent: None,
            },
        );
    }

    /// End a named timing span and return duration
    pub fn end_span(&mut self, name: &str) -> Option<Duration> {
        if !self.enabled {
            return None;
        }
        let duration = self.spans.remove(name).map(|s| s.start.elapsed())?;

        // Record to history
        self.span_history
            .entry(name.to_string())
            .or_insert_with(|| Vec::with_capacity(self.max_history))
            .push(duration);

        // Trim history
        if let Some(history) = self.span_history.get_mut(name) {
            while history.len() > self.max_history {
                history.remove(0);
            }
        }

        Some(duration)
    }

    /// Get average frame time
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.frame_times.iter().map(|r| r.total).sum();
        total / self.frame_times.len() as u32
    }

    /// Get estimated FPS
    pub fn fps(&self) -> f64 {
        let avg = self.average_frame_time();
        if avg.is_zero() {
            return 0.0;
        }
        1.0 / avg.as_secs_f64()
    }

    /// Get minimum frame time
    pub fn min_frame_time(&self) -> Duration {
        self.frame_times
            .iter()
            .map(|r| r.total)
            .min()
            .unwrap_or(Duration::ZERO)
    }

    /// Get maximum frame time
    pub fn max_frame_time(&self) -> Duration {
        self.frame_times
            .iter()
            .map(|r| r.total)
            .max()
            .unwrap_or(Duration::ZERO)
    }

    /// Get average layout time
    pub fn average_layout_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.frame_times.iter().map(|r| r.layout).sum();
        total / self.frame_times.len() as u32
    }

    /// Get average render time
    pub fn average_render_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.frame_times.iter().map(|r| r.render).sum();
        total / self.frame_times.len() as u32
    }

    /// Get frame history
    pub fn frame_history(&self) -> &[FrameRecord] {
        &self.frame_times
    }

    /// Get the last frame record
    pub fn last_frame(&self) -> Option<&FrameRecord> {
        self.frame_times.last()
    }

    /// Get span average duration
    pub fn span_average(&self, name: &str) -> Option<Duration> {
        let history = self.span_history.get(name)?;
        if history.is_empty() {
            return None;
        }
        let total: Duration = history.iter().sum();
        Some(total / history.len() as u32)
    }

    /// Get profiler summary as serializable data
    pub fn summary(&self) -> ProfilerSummary {
        ProfilerSummary {
            fps: self.fps(),
            avg_frame_time_ms: self.average_frame_time().as_secs_f64() * 1000.0,
            min_frame_time_ms: self.min_frame_time().as_secs_f64() * 1000.0,
            max_frame_time_ms: self.max_frame_time().as_secs_f64() * 1000.0,
            avg_layout_time_ms: self.average_layout_time().as_secs_f64() * 1000.0,
            avg_render_time_ms: self.average_render_time().as_secs_f64() * 1000.0,
            frame_count: self.frame_times.len(),
            component_count: self.frame_times.last().map(|f| f.component_count).unwrap_or(0),
        }
    }

    /// Clear all recorded data
    pub fn clear(&mut self) {
        self.frame_times.clear();
        self.spans.clear();
        self.span_history.clear();
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable profiler summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerSummary {
    /// Frames per second
    pub fps: f64,
    /// Average frame time in milliseconds
    pub avg_frame_time_ms: f64,
    /// Minimum frame time in milliseconds
    pub min_frame_time_ms: f64,
    /// Maximum frame time in milliseconds
    pub max_frame_time_ms: f64,
    /// Average layout time in milliseconds
    pub avg_layout_time_ms: f64,
    /// Average render time in milliseconds
    pub avg_render_time_ms: f64,
    /// Number of frames recorded
    pub frame_count: usize,
    /// Last known component count
    pub component_count: usize,
}

/// RAII guard for timing a span
pub struct SpanGuard<'a> {
    profiler: &'a mut Profiler,
    name: String,
}

impl<'a> SpanGuard<'a> {
    /// Create a new span guard
    pub fn new(profiler: &'a mut Profiler, name: impl Into<String>) -> Self {
        let name = name.into();
        profiler.begin_span(&name);
        Self { profiler, name }
    }
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        self.profiler.end_span(&self.name);
    }
}

/// Macro for timing a code block
#[macro_export]
macro_rules! profile_span {
    ($profiler:expr, $name:expr, $code:block) => {{
        $profiler.begin_span($name);
        let result = $code;
        $profiler.end_span($name);
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_profiler_frame() {
        let mut profiler = Profiler::new();

        profiler.begin_frame();
        sleep(Duration::from_millis(1));
        profiler.end_frame();

        assert!(profiler.average_frame_time() >= Duration::from_millis(1));
        assert_eq!(profiler.frame_history().len(), 1);
    }

    #[test]
    fn test_profiler_disabled() {
        let mut profiler = Profiler::new();
        profiler.set_enabled(false);

        profiler.begin_frame();
        profiler.end_frame();

        assert_eq!(profiler.frame_history().len(), 0);
    }

    #[test]
    fn test_profiler_spans() {
        let mut profiler = Profiler::new();

        profiler.begin_span("test");
        sleep(Duration::from_millis(1));
        let duration = profiler.end_span("test");

        assert!(duration.is_some());
        assert!(duration.unwrap() >= Duration::from_millis(1));
    }

    #[test]
    fn test_profiler_summary() {
        let mut profiler = Profiler::new();

        for _ in 0..5 {
            profiler.begin_frame();
            profiler.end_frame();
        }

        let summary = profiler.summary();
        assert_eq!(summary.frame_count, 5);
    }

    #[test]
    fn test_frame_phases() {
        let mut profiler = Profiler::new();

        profiler.begin_frame();

        profiler.begin_layout();
        sleep(Duration::from_micros(100));
        profiler.end_layout();

        profiler.begin_render();
        sleep(Duration::from_micros(100));
        profiler.end_render();

        profiler.set_component_count(42);
        profiler.end_frame();

        let last = profiler.last_frame().unwrap();
        assert!(last.layout > Duration::ZERO);
        assert!(last.render > Duration::ZERO);
        assert_eq!(last.component_count, 42);
    }
}
