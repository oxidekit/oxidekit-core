//! OxideKit Metrics and Observability System
//!
//! Enterprise-grade observability for OxideKit applications.
//!
//! # Design Principles
//!
//! - **Low overhead**: Metrics collection has minimal impact on performance
//! - **Thread-safe**: All operations are safe for concurrent access
//! - **Optional exporters**: Prometheus and structured logs are feature-gated
//! - **Production-ready**: Designed for production monitoring
//!
//! # Features
//!
//! - `full` - Enable all observability features
//! - `prometheus` - Enable Prometheus metrics exporter
//! - `structured-logs` - Enable structured JSON log exporter
//! - `devtools-panel` - Enable devtools metrics panels
//! - `perf-counters` - Enable detailed performance counters
//!
//! # Core Metrics
//!
//! - **Frame metrics**: FPS, frame time, jank detection
//! - **Memory metrics**: Heap usage, allocations, GC pressure
//! - **IO metrics**: Read/write ops, network latency
//! - **Render metrics**: Draw calls, GPU time, pipeline stats
//!
//! # Example
//!
//! ```rust,no_run
//! use oxide_metrics::{MetricsRegistry, FrameMetrics};
//!
//! let registry = MetricsRegistry::global();
//!
//! // Record frame timing
//! registry.frame_metrics().record_frame_time(16.67);
//!
//! // Get current FPS
//! let fps = registry.frame_metrics().fps();
//! ```

mod types;
mod registry;
mod collector;
mod frame;
mod memory;
mod io;
mod render;
mod tracing_hooks;

#[cfg(feature = "prometheus")]
mod prometheus_exporter;

#[cfg(feature = "structured-logs")]
mod log_exporter;

#[cfg(feature = "devtools-panel")]
mod devtools_panel;

#[cfg(feature = "perf-counters")]
mod perf_counters;

pub use types::*;
pub use registry::*;
pub use collector::*;
pub use frame::*;
pub use memory::*;
pub use io::*;
pub use render::*;
pub use tracing_hooks::*;

#[cfg(feature = "prometheus")]
pub use prometheus_exporter::*;

#[cfg(feature = "structured-logs")]
pub use log_exporter::*;

#[cfg(feature = "devtools-panel")]
pub use devtools_panel::*;

#[cfg(feature = "perf-counters")]
pub use perf_counters::*;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        MetricsRegistry, MetricsConfig, MetricValue, MetricType,
        FrameMetrics, MemoryMetrics, IoMetrics, RenderMetrics,
        MetricsCollector, MetricsSnapshot,
        TracingHooks, SpanGuard,
    };

    #[cfg(feature = "prometheus")]
    pub use crate::PrometheusExporter;

    #[cfg(feature = "structured-logs")]
    pub use crate::LogExporter;

    #[cfg(feature = "devtools-panel")]
    pub use crate::DevtoolsPanel;

    #[cfg(feature = "perf-counters")]
    pub use crate::PerfCounters;
}

/// Version of the metrics system
pub const METRICS_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!METRICS_VERSION.is_empty());
    }

    #[test]
    fn test_global_registry() {
        let registry = MetricsRegistry::global();
        assert!(registry.is_enabled());
    }
}
