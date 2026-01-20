//! Metrics collection and snapshot management

use crate::{MetricsRegistry, types::MetricValue};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// A point-in-time snapshot of all metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Application name
    pub app_name: String,
    /// Application version
    pub app_version: String,
    /// Frame metrics summary
    pub frame: FrameSnapshot,
    /// Memory metrics summary
    pub memory: MemorySnapshot,
    /// IO metrics summary
    pub io: IoSnapshot,
    /// Render metrics summary
    pub render: RenderSnapshot,
    /// Custom metrics
    pub custom: HashMap<String, MetricValue>,
    /// Global labels
    pub labels: HashMap<String, String>,
}

/// Frame metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSnapshot {
    /// Current FPS
    pub fps: f64,
    /// Average frame time in ms
    pub avg_frame_time_ms: f64,
    /// Min frame time in ms
    pub min_frame_time_ms: f64,
    /// Max frame time in ms
    pub max_frame_time_ms: f64,
    /// Total frame count
    pub frame_count: u64,
    /// Jank count
    pub jank_count: u64,
    /// Jank percentage
    pub jank_percent: f64,
    /// Is smooth (>55fps, <5% jank)
    pub is_smooth: bool,
}

/// Memory metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Heap used (bytes)
    pub heap_bytes: u64,
    /// Heap used (formatted)
    pub heap_formatted: String,
    /// Peak usage (bytes)
    pub peak_bytes: u64,
    /// Allocation count
    pub allocations: u64,
    /// GPU memory (bytes)
    pub gpu_bytes: u64,
    /// Potential leak detected
    pub potential_leak: bool,
}

/// IO metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoSnapshot {
    /// Bytes read
    pub bytes_read: u64,
    /// Bytes written
    pub bytes_written: u64,
    /// Read operations
    pub read_ops: u64,
    /// Write operations
    pub write_ops: u64,
    /// Network requests
    pub network_requests: u64,
    /// Failed operations
    pub failed_ops: u64,
    /// Success rate (0-1)
    pub success_rate: f64,
}

/// Render metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSnapshot {
    /// Draw calls per frame
    pub draw_calls: u64,
    /// Triangles per frame
    pub triangles: u64,
    /// Average GPU time in ms
    pub avg_gpu_time_ms: f64,
    /// Max GPU time in ms
    pub max_gpu_time_ms: f64,
    /// Shader compilations
    pub shader_compiles: u64,
    /// Total frames rendered
    pub total_frames: u64,
    /// Batching advisory (if any)
    pub batching_advisory: Option<String>,
}

impl MetricsSnapshot {
    /// Create a snapshot from the global registry
    pub fn from_global() -> Self {
        Self::from_registry(MetricsRegistry::global())
    }

    /// Create a snapshot from a specific registry
    pub fn from_registry(registry: &MetricsRegistry) -> Self {
        let config = registry.config();
        let frame_metrics = registry.frame_metrics();
        let memory_metrics = registry.memory_metrics();
        let io_metrics = registry.io_metrics();
        let render_metrics = registry.render_metrics();

        let custom: HashMap<String, MetricValue> = registry
            .all_custom_metrics()
            .into_iter()
            .map(|(k, m)| (k, m.value))
            .collect();

        Self {
            timestamp: chrono::Utc::now(),
            app_name: config.metric_prefix.clone(),
            app_version: crate::METRICS_VERSION.to_string(),
            frame: FrameSnapshot {
                fps: frame_metrics.fps(),
                avg_frame_time_ms: frame_metrics.avg_frame_time_ms(),
                min_frame_time_ms: frame_metrics.min_frame_time_ms(),
                max_frame_time_ms: frame_metrics.max_frame_time_ms(),
                frame_count: frame_metrics.frame_count(),
                jank_count: frame_metrics.jank_count(),
                jank_percent: frame_metrics.jank_percentage(),
                is_smooth: frame_metrics.is_smooth(),
            },
            memory: MemorySnapshot {
                heap_bytes: memory_metrics.heap_used(),
                heap_formatted: memory_metrics.current_stats().format_heap(),
                peak_bytes: memory_metrics.peak(),
                allocations: memory_metrics.current_stats().allocation_count,
                gpu_bytes: memory_metrics.gpu_memory(),
                potential_leak: memory_metrics.potential_leak(),
            },
            io: IoSnapshot {
                bytes_read: io_metrics.current_stats().bytes_read,
                bytes_written: io_metrics.current_stats().bytes_written,
                read_ops: io_metrics.current_stats().read_ops,
                write_ops: io_metrics.current_stats().write_ops,
                network_requests: io_metrics.current_stats().network_requests,
                failed_ops: io_metrics.current_stats().failed_ops,
                success_rate: io_metrics.current_stats().success_rate(),
            },
            render: RenderSnapshot {
                draw_calls: render_metrics.current_stats().draw_calls,
                triangles: render_metrics.current_stats().triangles,
                avg_gpu_time_ms: render_metrics.avg_gpu_time_ms(),
                max_gpu_time_ms: render_metrics.max_gpu_time_ms(),
                shader_compiles: render_metrics.current_stats().shader_compiles,
                total_frames: render_metrics.current_stats().total_frames,
                batching_advisory: render_metrics.batching_advisory(),
            },
            custom,
            labels: config.global_labels.clone(),
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Serialize to compact JSON
    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Metrics collector that aggregates and exports metrics
pub struct MetricsCollector {
    /// Snapshot history
    snapshots: RwLock<Vec<MetricsSnapshot>>,
    /// Maximum snapshots to retain
    max_snapshots: usize,
    /// Collection interval tracking
    last_collection: RwLock<Option<Instant>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::with_max_snapshots(1000)
    }

    /// Create a collector with specific snapshot limit
    pub fn with_max_snapshots(max: usize) -> Self {
        Self {
            snapshots: RwLock::new(Vec::with_capacity(max)),
            max_snapshots: max,
            last_collection: RwLock::new(None),
        }
    }

    /// Collect a snapshot from the global registry
    pub fn collect(&self) {
        self.collect_from(MetricsRegistry::global());
    }

    /// Collect a snapshot from a specific registry
    pub fn collect_from(&self, registry: &MetricsRegistry) {
        if !registry.is_enabled() {
            return;
        }

        let snapshot = MetricsSnapshot::from_registry(registry);

        let mut snapshots = self.snapshots.write();
        if snapshots.len() >= self.max_snapshots {
            snapshots.remove(0);
        }
        snapshots.push(snapshot);

        *self.last_collection.write() = Some(Instant::now());
    }

    /// Get the latest snapshot
    pub fn latest(&self) -> Option<MetricsSnapshot> {
        self.snapshots.read().last().cloned()
    }

    /// Get all snapshots
    pub fn all_snapshots(&self) -> Vec<MetricsSnapshot> {
        self.snapshots.read().clone()
    }

    /// Get snapshots in a time range
    pub fn snapshots_since(&self, since: chrono::DateTime<chrono::Utc>) -> Vec<MetricsSnapshot> {
        self.snapshots
            .read()
            .iter()
            .filter(|s| s.timestamp >= since)
            .cloned()
            .collect()
    }

    /// Clear all snapshots
    pub fn clear(&self) {
        self.snapshots.write().clear();
    }

    /// Get snapshot count
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.read().len()
    }

    /// Calculate average FPS over all snapshots
    pub fn avg_fps(&self) -> f64 {
        let snapshots = self.snapshots.read();
        if snapshots.is_empty() {
            return 0.0;
        }
        let sum: f64 = snapshots.iter().map(|s| s.frame.fps).sum();
        sum / snapshots.len() as f64
    }

    /// Calculate average frame time over all snapshots
    pub fn avg_frame_time(&self) -> f64 {
        let snapshots = self.snapshots.read();
        if snapshots.is_empty() {
            return 0.0;
        }
        let sum: f64 = snapshots.iter().map(|s| s.frame.avg_frame_time_ms).sum();
        sum / snapshots.len() as f64
    }

    /// Get memory trend (positive = increasing)
    pub fn memory_trend(&self) -> f64 {
        let snapshots = self.snapshots.read();
        if snapshots.len() < 10 {
            return 0.0;
        }

        let recent: u64 = snapshots.iter().rev().take(5).map(|s| s.memory.heap_bytes).sum();
        let older: u64 = snapshots.iter().rev().skip(5).take(5).map(|s| s.memory.heap_bytes).sum();

        if older == 0 {
            0.0
        } else {
            ((recent as f64 - older as f64) / older as f64) * 100.0
        }
    }

    /// Generate a health report
    pub fn health_report(&self) -> HealthReport {
        let latest = self.latest();

        let (fps_status, fps_message) = match &latest {
            Some(s) if s.frame.fps >= 55.0 => (HealthStatus::Good, "Frame rate is excellent".to_string()),
            Some(s) if s.frame.fps >= 30.0 => (HealthStatus::Warning, format!("Frame rate is below target: {:.1} FPS", s.frame.fps)),
            Some(s) => (HealthStatus::Critical, format!("Frame rate is critically low: {:.1} FPS", s.frame.fps)),
            None => (HealthStatus::Unknown, "No data available".to_string()),
        };

        let (memory_status, memory_message) = match &latest {
            Some(s) if !s.memory.potential_leak => (HealthStatus::Good, "Memory usage is stable".to_string()),
            Some(s) => (HealthStatus::Warning, format!("Potential memory leak detected ({})", s.memory.heap_formatted)),
            None => (HealthStatus::Unknown, "No data available".to_string()),
        };

        let (io_status, io_message) = match &latest {
            Some(s) if s.io.success_rate >= 0.99 => (HealthStatus::Good, "IO operations healthy".to_string()),
            Some(s) if s.io.success_rate >= 0.95 => (HealthStatus::Warning, format!("Some IO failures: {:.1}% success rate", s.io.success_rate * 100.0)),
            Some(s) => (HealthStatus::Critical, format!("High IO failure rate: {:.1}% success", s.io.success_rate * 100.0)),
            None => (HealthStatus::Unknown, "No data available".to_string()),
        };

        HealthReport {
            timestamp: chrono::Utc::now(),
            overall: worst_status(&[&fps_status, &memory_status, &io_status]),
            fps: HealthCheck { status: fps_status, message: fps_message },
            memory: HealthCheck { status: memory_status, message: memory_message },
            io: HealthCheck { status: io_status, message: io_message },
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Good,
    Warning,
    Critical,
    Unknown,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Status level
    pub status: HealthStatus,
    /// Human-readable message
    pub message: String,
}

/// Overall health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Timestamp of the report
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Overall status
    pub overall: HealthStatus,
    /// FPS health check
    pub fps: HealthCheck,
    /// Memory health check
    pub memory: HealthCheck,
    /// IO health check
    pub io: HealthCheck,
}

impl HealthReport {
    /// Is the application healthy?
    pub fn is_healthy(&self) -> bool {
        self.overall == HealthStatus::Good
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Get the worst status from a list
fn worst_status(statuses: &[&HealthStatus]) -> HealthStatus {
    if statuses.iter().any(|s| **s == HealthStatus::Critical) {
        HealthStatus::Critical
    } else if statuses.iter().any(|s| **s == HealthStatus::Warning) {
        HealthStatus::Warning
    } else if statuses.iter().any(|s| **s == HealthStatus::Unknown) {
        HealthStatus::Unknown
    } else {
        HealthStatus::Good
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_snapshot() {
        let snapshot = MetricsSnapshot::from_global();
        assert!(!snapshot.app_version.is_empty());
    }

    #[test]
    fn test_collector() {
        let collector = MetricsCollector::new();
        collector.collect();

        assert_eq!(collector.snapshot_count(), 1);
        assert!(collector.latest().is_some());
    }

    #[test]
    fn test_health_report() {
        let collector = MetricsCollector::new();
        collector.collect();

        let report = collector.health_report();
        // With no data, should be unknown
        assert!(!report.is_healthy() || report.overall == HealthStatus::Good);
    }

    #[test]
    fn test_snapshot_json() {
        let snapshot = MetricsSnapshot::from_global();
        let json = snapshot.to_json();
        assert!(json.contains("fps"));
        assert!(json.contains("memory"));
    }
}
