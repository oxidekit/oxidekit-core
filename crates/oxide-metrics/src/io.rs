//! IO and network metrics

use crate::types::RollingWindow;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// IO statistics snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IoStats {
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Number of read operations
    pub read_ops: u64,
    /// Number of write operations
    pub write_ops: u64,
    /// Average read latency in milliseconds
    pub avg_read_latency_ms: f64,
    /// Average write latency in milliseconds
    pub avg_write_latency_ms: f64,
    /// Network requests made
    pub network_requests: u64,
    /// Network bytes received
    pub network_bytes_in: u64,
    /// Network bytes sent
    pub network_bytes_out: u64,
    /// Failed operations
    pub failed_ops: u64,
}

impl IoStats {
    /// Total IO operations
    pub fn total_ops(&self) -> u64 {
        self.read_ops + self.write_ops
    }

    /// Total bytes transferred
    pub fn total_bytes(&self) -> u64 {
        self.bytes_read + self.bytes_written
    }

    /// Total network bytes
    pub fn total_network_bytes(&self) -> u64 {
        self.network_bytes_in + self.network_bytes_out
    }

    /// Success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        let total = self.total_ops() + self.network_requests;
        if total == 0 {
            1.0
        } else {
            1.0 - (self.failed_ops as f64 / total as f64)
        }
    }
}

/// IO metrics collector
#[derive(Debug)]
pub struct IoMetrics {
    /// Total bytes read
    bytes_read: AtomicU64,
    /// Total bytes written
    bytes_written: AtomicU64,
    /// Read operations count
    read_ops: AtomicU64,
    /// Write operations count
    write_ops: AtomicU64,
    /// Read latencies (rolling window, in ms)
    read_latencies: RwLock<RollingWindow>,
    /// Write latencies (rolling window, in ms)
    write_latencies: RwLock<RollingWindow>,
    /// Network request count
    network_requests: AtomicU64,
    /// Network bytes received
    network_bytes_in: AtomicU64,
    /// Network bytes sent
    network_bytes_out: AtomicU64,
    /// Failed operations count
    failed_ops: AtomicU64,
    /// Per-endpoint statistics
    endpoint_stats: RwLock<HashMap<String, EndpointStats>>,
    /// Per-file statistics (when detailed tracking enabled)
    file_stats: RwLock<HashMap<String, FileStats>>,
}

/// Statistics for a network endpoint
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EndpointStats {
    /// Endpoint URL/identifier
    pub endpoint: String,
    /// Number of requests
    pub request_count: u64,
    /// Average response time in ms
    pub avg_response_time_ms: f64,
    /// Number of failures
    pub failures: u64,
    /// Total bytes transferred
    pub total_bytes: u64,
}

/// Statistics for a file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileStats {
    /// File path
    pub path: String,
    /// Read count
    pub reads: u64,
    /// Write count
    pub writes: u64,
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
}

impl IoMetrics {
    /// Create new IO metrics
    pub fn new() -> Self {
        Self {
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
            read_ops: AtomicU64::new(0),
            write_ops: AtomicU64::new(0),
            read_latencies: RwLock::new(RollingWindow::new(100)),
            write_latencies: RwLock::new(RollingWindow::new(100)),
            network_requests: AtomicU64::new(0),
            network_bytes_in: AtomicU64::new(0),
            network_bytes_out: AtomicU64::new(0),
            failed_ops: AtomicU64::new(0),
            endpoint_stats: RwLock::new(HashMap::new()),
            file_stats: RwLock::new(HashMap::new()),
        }
    }

    /// Record a read operation
    pub fn record_read(&self, bytes: u64, latency: Duration) {
        self.bytes_read.fetch_add(bytes, Ordering::Relaxed);
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        self.read_latencies
            .write()
            .push(latency.as_secs_f64() * 1000.0);
    }

    /// Record a write operation
    pub fn record_write(&self, bytes: u64, latency: Duration) {
        self.bytes_written.fetch_add(bytes, Ordering::Relaxed);
        self.write_ops.fetch_add(1, Ordering::Relaxed);
        self.write_latencies
            .write()
            .push(latency.as_secs_f64() * 1000.0);
    }

    /// Record a read operation for a specific file
    pub fn record_file_read(&self, path: &str, bytes: u64, latency: Duration) {
        self.record_read(bytes, latency);

        let mut stats = self.file_stats.write();
        let file_stat = stats.entry(path.to_string()).or_insert_with(|| FileStats {
            path: path.to_string(),
            ..Default::default()
        });
        file_stat.reads += 1;
        file_stat.bytes_read += bytes;
    }

    /// Record a write operation for a specific file
    pub fn record_file_write(&self, path: &str, bytes: u64, latency: Duration) {
        self.record_write(bytes, latency);

        let mut stats = self.file_stats.write();
        let file_stat = stats.entry(path.to_string()).or_insert_with(|| FileStats {
            path: path.to_string(),
            ..Default::default()
        });
        file_stat.writes += 1;
        file_stat.bytes_written += bytes;
    }

    /// Record a network request
    pub fn record_network_request(
        &self,
        endpoint: &str,
        bytes_in: u64,
        bytes_out: u64,
        latency: Duration,
        success: bool,
    ) {
        self.network_requests.fetch_add(1, Ordering::Relaxed);
        self.network_bytes_in.fetch_add(bytes_in, Ordering::Relaxed);
        self.network_bytes_out
            .fetch_add(bytes_out, Ordering::Relaxed);

        if !success {
            self.failed_ops.fetch_add(1, Ordering::Relaxed);
        }

        // Update endpoint stats
        let mut stats = self.endpoint_stats.write();
        let endpoint_stat = stats.entry(endpoint.to_string()).or_insert_with(|| EndpointStats {
            endpoint: endpoint.to_string(),
            ..Default::default()
        });

        endpoint_stat.request_count += 1;
        endpoint_stat.total_bytes += bytes_in + bytes_out;
        if !success {
            endpoint_stat.failures += 1;
        }

        // Update rolling average
        let latency_ms = latency.as_secs_f64() * 1000.0;
        let n = endpoint_stat.request_count as f64;
        endpoint_stat.avg_response_time_ms =
            (endpoint_stat.avg_response_time_ms * (n - 1.0) + latency_ms) / n;
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        self.failed_ops.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current IO statistics
    pub fn current_stats(&self) -> IoStats {
        IoStats {
            bytes_read: self.bytes_read.load(Ordering::Relaxed),
            bytes_written: self.bytes_written.load(Ordering::Relaxed),
            read_ops: self.read_ops.load(Ordering::Relaxed),
            write_ops: self.write_ops.load(Ordering::Relaxed),
            avg_read_latency_ms: self.read_latencies.read().average(),
            avg_write_latency_ms: self.write_latencies.read().average(),
            network_requests: self.network_requests.load(Ordering::Relaxed),
            network_bytes_in: self.network_bytes_in.load(Ordering::Relaxed),
            network_bytes_out: self.network_bytes_out.load(Ordering::Relaxed),
            failed_ops: self.failed_ops.load(Ordering::Relaxed),
        }
    }

    /// Get endpoint statistics
    pub fn endpoint_stats(&self) -> HashMap<String, EndpointStats> {
        self.endpoint_stats.read().clone()
    }

    /// Get file statistics
    pub fn file_stats(&self) -> HashMap<String, FileStats> {
        self.file_stats.read().clone()
    }

    /// Get slowest endpoints
    pub fn slowest_endpoints(&self, limit: usize) -> Vec<EndpointStats> {
        let mut endpoints: Vec<_> = self.endpoint_stats.read().values().cloned().collect();
        endpoints.sort_by(|a, b| {
            b.avg_response_time_ms
                .partial_cmp(&a.avg_response_time_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        endpoints.truncate(limit);
        endpoints
    }

    /// Get most active files
    pub fn most_active_files(&self, limit: usize) -> Vec<FileStats> {
        let mut files: Vec<_> = self.file_stats.read().values().cloned().collect();
        files.sort_by(|a, b| {
            (b.reads + b.writes).cmp(&(a.reads + a.writes))
        });
        files.truncate(limit);
        files
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.bytes_read.store(0, Ordering::Relaxed);
        self.bytes_written.store(0, Ordering::Relaxed);
        self.read_ops.store(0, Ordering::Relaxed);
        self.write_ops.store(0, Ordering::Relaxed);
        *self.read_latencies.write() = RollingWindow::new(100);
        *self.write_latencies.write() = RollingWindow::new(100);
        self.network_requests.store(0, Ordering::Relaxed);
        self.network_bytes_in.store(0, Ordering::Relaxed);
        self.network_bytes_out.store(0, Ordering::Relaxed);
        self.failed_ops.store(0, Ordering::Relaxed);
        self.endpoint_stats.write().clear();
        self.file_stats.write().clear();
    }
}

impl Default for IoMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// IO operation timing guard
pub struct IoGuard<'a> {
    metrics: &'a IoMetrics,
    start: Instant,
    bytes: u64,
    is_read: bool,
}

impl<'a> IoGuard<'a> {
    /// Start timing a read operation
    pub fn read(metrics: &'a IoMetrics, bytes: u64) -> Self {
        Self {
            metrics,
            start: Instant::now(),
            bytes,
            is_read: true,
        }
    }

    /// Start timing a write operation
    pub fn write(metrics: &'a IoMetrics, bytes: u64) -> Self {
        Self {
            metrics,
            start: Instant::now(),
            bytes,
            is_read: false,
        }
    }
}

impl<'a> Drop for IoGuard<'a> {
    fn drop(&mut self) {
        let latency = self.start.elapsed();
        if self.is_read {
            self.metrics.record_read(self.bytes, latency);
        } else {
            self.metrics.record_write(self.bytes, latency);
        }
    }
}

/// Network request timing guard
pub struct NetworkGuard<'a> {
    metrics: &'a IoMetrics,
    endpoint: String,
    start: Instant,
    bytes_out: u64,
    bytes_in: u64,
    success: bool,
}

impl<'a> NetworkGuard<'a> {
    /// Start timing a network request
    pub fn start(metrics: &'a IoMetrics, endpoint: &str, bytes_out: u64) -> Self {
        Self {
            metrics,
            endpoint: endpoint.to_string(),
            start: Instant::now(),
            bytes_out,
            bytes_in: 0,
            success: true,
        }
    }

    /// Set response bytes
    pub fn set_response_bytes(&mut self, bytes: u64) {
        self.bytes_in = bytes;
    }

    /// Mark as failed
    pub fn mark_failed(&mut self) {
        self.success = false;
    }
}

impl<'a> Drop for NetworkGuard<'a> {
    fn drop(&mut self) {
        self.metrics.record_network_request(
            &self.endpoint,
            self.bytes_in,
            self.bytes_out,
            self.start.elapsed(),
            self.success,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_metrics() {
        let metrics = IoMetrics::new();

        metrics.record_read(1024, Duration::from_millis(10));
        metrics.record_write(512, Duration::from_millis(5));

        let stats = metrics.current_stats();
        assert_eq!(stats.bytes_read, 1024);
        assert_eq!(stats.bytes_written, 512);
        assert_eq!(stats.read_ops, 1);
        assert_eq!(stats.write_ops, 1);
    }

    #[test]
    fn test_network_metrics() {
        let metrics = IoMetrics::new();

        metrics.record_network_request("https://api.example.com", 100, 50, Duration::from_millis(50), true);
        metrics.record_network_request("https://api.example.com", 200, 100, Duration::from_millis(100), false);

        let stats = metrics.current_stats();
        assert_eq!(stats.network_requests, 2);
        assert_eq!(stats.network_bytes_in, 300);
        assert_eq!(stats.network_bytes_out, 150);
        assert_eq!(stats.failed_ops, 1);

        let endpoint_stats = metrics.endpoint_stats();
        let api_stats = endpoint_stats.get("https://api.example.com").unwrap();
        assert_eq!(api_stats.request_count, 2);
        assert_eq!(api_stats.failures, 1);
    }

    #[test]
    fn test_io_guard() {
        let metrics = IoMetrics::new();

        {
            let _guard = IoGuard::read(&metrics, 1024);
            std::thread::sleep(Duration::from_millis(1));
        }

        let stats = metrics.current_stats();
        assert_eq!(stats.read_ops, 1);
        assert_eq!(stats.bytes_read, 1024);
        assert!(stats.avg_read_latency_ms >= 1.0);
    }
}
