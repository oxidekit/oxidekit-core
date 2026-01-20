//! Global metrics registry

use crate::types::{Metric, MetricUnit, MetricValue};
use crate::{FrameMetrics, IoMetrics, MemoryMetrics, RenderMetrics};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Global metrics registry singleton
static GLOBAL_REGISTRY: OnceLock<MetricsRegistry> = OnceLock::new();

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Frame metrics window size (number of frames to average)
    #[serde(default = "default_frame_window")]
    pub frame_window_size: usize,

    /// Memory sampling interval in milliseconds
    #[serde(default = "default_memory_interval")]
    pub memory_sample_interval_ms: u64,

    /// Enable detailed IO tracking
    #[serde(default)]
    pub detailed_io: bool,

    /// Enable render pipeline instrumentation
    #[serde(default)]
    pub render_instrumentation: bool,

    /// Metric prefix for exports
    #[serde(default = "default_prefix")]
    pub metric_prefix: String,

    /// Global labels to add to all metrics
    #[serde(default)]
    pub global_labels: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

fn default_frame_window() -> usize {
    120 // 2 seconds at 60fps
}

fn default_memory_interval() -> u64 {
    1000 // 1 second
}

fn default_prefix() -> String {
    "oxide".to_string()
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frame_window_size: default_frame_window(),
            memory_sample_interval_ms: default_memory_interval(),
            detailed_io: false,
            render_instrumentation: false,
            metric_prefix: default_prefix(),
            global_labels: HashMap::new(),
        }
    }
}

/// Central metrics registry
///
/// Thread-safe container for all application metrics.
pub struct MetricsRegistry {
    /// Configuration
    config: RwLock<MetricsConfig>,

    /// Custom metrics
    custom_metrics: RwLock<HashMap<String, Metric>>,

    /// Frame metrics
    frame_metrics: FrameMetrics,

    /// Memory metrics
    memory_metrics: MemoryMetrics,

    /// IO metrics
    io_metrics: IoMetrics,

    /// Render metrics
    render_metrics: RenderMetrics,
}

impl MetricsRegistry {
    /// Create a new metrics registry with default configuration
    pub fn new() -> Self {
        Self::with_config(MetricsConfig::default())
    }

    /// Create a new metrics registry with custom configuration
    pub fn with_config(config: MetricsConfig) -> Self {
        let frame_window_size = config.frame_window_size;
        Self {
            config: RwLock::new(config),
            custom_metrics: RwLock::new(HashMap::new()),
            frame_metrics: FrameMetrics::new(frame_window_size),
            memory_metrics: MemoryMetrics::new(),
            io_metrics: IoMetrics::new(),
            render_metrics: RenderMetrics::new(),
        }
    }

    /// Get or initialize the global metrics registry
    pub fn global() -> &'static MetricsRegistry {
        GLOBAL_REGISTRY.get_or_init(|| MetricsRegistry::new())
    }

    /// Initialize the global registry with custom config
    ///
    /// Must be called before any calls to `global()`.
    /// Returns `Err` if the registry has already been initialized.
    pub fn init_global(config: MetricsConfig) -> Result<(), &'static str> {
        GLOBAL_REGISTRY
            .set(MetricsRegistry::with_config(config))
            .map_err(|_| "Global metrics registry already initialized")
    }

    /// Check if metrics collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.read().enabled
    }

    /// Enable or disable metrics collection
    pub fn set_enabled(&self, enabled: bool) {
        self.config.write().enabled = enabled;
    }

    /// Get the current configuration
    pub fn config(&self) -> MetricsConfig {
        self.config.read().clone()
    }

    /// Update the configuration
    pub fn set_config(&self, config: MetricsConfig) {
        *self.config.write() = config;
    }

    /// Get frame metrics
    pub fn frame_metrics(&self) -> &FrameMetrics {
        &self.frame_metrics
    }

    /// Get memory metrics
    pub fn memory_metrics(&self) -> &MemoryMetrics {
        &self.memory_metrics
    }

    /// Get IO metrics
    pub fn io_metrics(&self) -> &IoMetrics {
        &self.io_metrics
    }

    /// Get render metrics
    pub fn render_metrics(&self) -> &RenderMetrics {
        &self.render_metrics
    }

    /// Register a custom counter metric
    pub fn register_counter(&self, name: &str, description: &str) {
        let metric = Metric::counter(name, description, 0);
        self.custom_metrics
            .write()
            .insert(name.to_string(), metric);
    }

    /// Register a custom gauge metric
    pub fn register_gauge(&self, name: &str, description: &str) {
        let metric = Metric::gauge(name, description, 0.0);
        self.custom_metrics
            .write()
            .insert(name.to_string(), metric);
    }

    /// Register a custom histogram metric
    pub fn register_histogram(&self, name: &str, description: &str) {
        let metric = Metric::histogram(name, description);
        self.custom_metrics
            .write()
            .insert(name.to_string(), metric);
    }

    /// Increment a counter by 1
    pub fn increment_counter(&self, name: &str) {
        self.add_counter(name, 1);
    }

    /// Add to a counter
    pub fn add_counter(&self, name: &str, delta: u64) {
        let mut metrics = self.custom_metrics.write();
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Counter(ref mut v) = metric.value {
                *v += delta;
                metric.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Set a gauge value
    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut metrics = self.custom_metrics.write();
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Gauge(ref mut v) = metric.value {
                *v = value;
                metric.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Observe a histogram value
    pub fn observe_histogram(&self, name: &str, value: f64) {
        let mut metrics = self.custom_metrics.write();
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Histogram(ref mut hist) = metric.value {
                hist.observe(value);
                metric.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Get a custom metric value
    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        self.custom_metrics.read().get(name).cloned()
    }

    /// Get all custom metrics
    pub fn all_custom_metrics(&self) -> HashMap<String, Metric> {
        self.custom_metrics.read().clone()
    }

    /// Get all metrics (built-in + custom) for export
    pub fn all_metrics(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        let config = self.config.read();
        let prefix = &config.metric_prefix;

        // Frame metrics
        metrics.push(
            Metric::gauge(
                &format!("{}_frame_fps", prefix),
                "Current frames per second",
                self.frame_metrics.fps(),
            )
            .with_unit(MetricUnit::Fps),
        );
        metrics.push(
            Metric::gauge(
                &format!("{}_frame_time_ms", prefix),
                "Average frame time in milliseconds",
                self.frame_metrics.avg_frame_time_ms(),
            )
            .with_unit(MetricUnit::Milliseconds),
        );
        metrics.push(
            Metric::counter(
                &format!("{}_frame_count", prefix),
                "Total frames rendered",
                self.frame_metrics.frame_count(),
            )
            .with_unit(MetricUnit::Count),
        );
        metrics.push(
            Metric::counter(
                &format!("{}_jank_count", prefix),
                "Number of janky frames",
                self.frame_metrics.jank_count(),
            )
            .with_unit(MetricUnit::Count),
        );

        // Memory metrics
        let mem_stats = self.memory_metrics.current_stats();
        metrics.push(
            Metric::gauge(
                &format!("{}_memory_heap_bytes", prefix),
                "Current heap memory usage",
                mem_stats.heap_used as f64,
            )
            .with_unit(MetricUnit::Bytes),
        );
        metrics.push(
            Metric::gauge(
                &format!("{}_memory_allocated_bytes", prefix),
                "Total allocated memory",
                mem_stats.allocated as f64,
            )
            .with_unit(MetricUnit::Bytes),
        );
        metrics.push(
            Metric::counter(
                &format!("{}_memory_allocations", prefix),
                "Total memory allocations",
                mem_stats.allocation_count,
            )
            .with_unit(MetricUnit::Count),
        );

        // IO metrics
        let io_stats = self.io_metrics.current_stats();
        metrics.push(
            Metric::counter(
                &format!("{}_io_read_bytes", prefix),
                "Total bytes read",
                io_stats.bytes_read,
            )
            .with_unit(MetricUnit::Bytes),
        );
        metrics.push(
            Metric::counter(
                &format!("{}_io_write_bytes", prefix),
                "Total bytes written",
                io_stats.bytes_written,
            )
            .with_unit(MetricUnit::Bytes),
        );
        metrics.push(
            Metric::counter(
                &format!("{}_io_read_ops", prefix),
                "Total read operations",
                io_stats.read_ops,
            )
            .with_unit(MetricUnit::Count),
        );
        metrics.push(
            Metric::counter(
                &format!("{}_io_write_ops", prefix),
                "Total write operations",
                io_stats.write_ops,
            )
            .with_unit(MetricUnit::Count),
        );

        // Render metrics
        let render_stats = self.render_metrics.current_stats();
        metrics.push(
            Metric::gauge(
                &format!("{}_render_draw_calls", prefix),
                "Draw calls per frame",
                render_stats.draw_calls as f64,
            )
            .with_unit(MetricUnit::Count),
        );
        metrics.push(
            Metric::gauge(
                &format!("{}_render_triangles", prefix),
                "Triangles per frame",
                render_stats.triangles as f64,
            )
            .with_unit(MetricUnit::Count),
        );
        metrics.push(
            Metric::gauge(
                &format!("{}_render_gpu_time_ms", prefix),
                "GPU time in milliseconds",
                render_stats.gpu_time_ms,
            )
            .with_unit(MetricUnit::Milliseconds),
        );

        // Custom metrics
        for (_, metric) in self.custom_metrics.read().iter() {
            metrics.push(metric.clone());
        }

        metrics
    }

    /// Reset all metrics to initial values
    pub fn reset(&self) {
        self.frame_metrics.reset();
        self.memory_metrics.reset();
        self.io_metrics.reset();
        self.render_metrics.reset();
        self.custom_metrics.write().clear();
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safety: MetricsRegistry uses internal locking
unsafe impl Send for MetricsRegistry {}
unsafe impl Sync for MetricsRegistry {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = MetricsRegistry::new();
        assert!(registry.is_enabled());
    }

    #[test]
    fn test_custom_counter() {
        let registry = MetricsRegistry::new();
        registry.register_counter("test_counter", "A test counter");
        registry.increment_counter("test_counter");
        registry.add_counter("test_counter", 5);

        let metric = registry.get_metric("test_counter").unwrap();
        if let MetricValue::Counter(v) = metric.value {
            assert_eq!(v, 6);
        } else {
            panic!("Expected counter");
        }
    }

    #[test]
    fn test_custom_gauge() {
        let registry = MetricsRegistry::new();
        registry.register_gauge("test_gauge", "A test gauge");
        registry.set_gauge("test_gauge", 42.5);

        let metric = registry.get_metric("test_gauge").unwrap();
        if let MetricValue::Gauge(v) = metric.value {
            assert_eq!(v, 42.5);
        } else {
            panic!("Expected gauge");
        }
    }

    #[test]
    fn test_all_metrics() {
        let registry = MetricsRegistry::new();
        let metrics = registry.all_metrics();
        assert!(!metrics.is_empty());
    }
}
