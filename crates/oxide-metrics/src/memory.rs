//! Memory usage metrics

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Memory statistics snapshot
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Heap memory currently in use (bytes)
    pub heap_used: u64,
    /// Total allocated memory (bytes)
    pub allocated: u64,
    /// Peak memory usage (bytes)
    pub peak: u64,
    /// Number of allocations
    pub allocation_count: u64,
    /// Number of deallocations
    pub deallocation_count: u64,
    /// GPU memory used (bytes, if available)
    pub gpu_memory: u64,
    /// Texture memory used (bytes)
    pub texture_memory: u64,
    /// Buffer memory used (bytes)
    pub buffer_memory: u64,
}

impl MemoryStats {
    /// Get active allocations (allocations - deallocations)
    pub fn active_allocations(&self) -> u64 {
        self.allocation_count.saturating_sub(self.deallocation_count)
    }

    /// Get memory utilization as percentage of peak
    pub fn utilization(&self) -> f64 {
        if self.peak == 0 {
            0.0
        } else {
            (self.heap_used as f64 / self.peak as f64) * 100.0
        }
    }

    /// Format as human-readable string
    pub fn format_heap(&self) -> String {
        format_bytes(self.heap_used)
    }

    /// Format GPU memory as human-readable string
    pub fn format_gpu(&self) -> String {
        format_bytes(self.gpu_memory)
    }
}

/// Memory metrics collector
#[derive(Debug)]
pub struct MemoryMetrics {
    /// Current heap usage
    heap_used: AtomicU64,
    /// Total allocated
    allocated: AtomicU64,
    /// Peak usage
    peak: AtomicU64,
    /// Allocation count
    allocation_count: AtomicU64,
    /// Deallocation count
    deallocation_count: AtomicU64,
    /// GPU memory
    gpu_memory: AtomicU64,
    /// Texture memory
    texture_memory: AtomicU64,
    /// Buffer memory
    buffer_memory: AtomicU64,
    /// Historical samples for trending
    history: RwLock<Vec<MemorySample>>,
    /// Maximum history length
    max_history: usize,
}

/// A timestamped memory sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    /// Timestamp of the sample
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Heap usage at this time
    pub heap_used: u64,
    /// GPU memory at this time
    pub gpu_memory: u64,
}

impl MemoryMetrics {
    /// Create new memory metrics
    pub fn new() -> Self {
        Self {
            heap_used: AtomicU64::new(0),
            allocated: AtomicU64::new(0),
            peak: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
            gpu_memory: AtomicU64::new(0),
            texture_memory: AtomicU64::new(0),
            buffer_memory: AtomicU64::new(0),
            history: RwLock::new(Vec::with_capacity(1000)),
            max_history: 1000,
        }
    }

    /// Record current memory state
    pub fn sample(&self) {
        let sample = MemorySample {
            timestamp: chrono::Utc::now(),
            heap_used: self.heap_used.load(Ordering::Relaxed),
            gpu_memory: self.gpu_memory.load(Ordering::Relaxed),
        };

        let mut history = self.history.write();
        if history.len() >= self.max_history {
            history.remove(0);
        }
        history.push(sample);
    }

    /// Update heap memory usage
    pub fn set_heap_used(&self, bytes: u64) {
        self.heap_used.store(bytes, Ordering::Relaxed);

        // Update peak if necessary
        let current_peak = self.peak.load(Ordering::Relaxed);
        if bytes > current_peak {
            self.peak.store(bytes, Ordering::Relaxed);
        }
    }

    /// Record an allocation
    pub fn record_allocation(&self, size: u64) {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        self.allocated.fetch_add(size, Ordering::Relaxed);

        // Update heap estimate
        let current = self.heap_used.fetch_add(size, Ordering::Relaxed) + size;

        // Update peak
        let peak = self.peak.load(Ordering::Relaxed);
        if current > peak {
            self.peak.store(current, Ordering::Relaxed);
        }
    }

    /// Record a deallocation
    pub fn record_deallocation(&self, size: u64) {
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        self.heap_used.fetch_sub(size.min(self.heap_used.load(Ordering::Relaxed)), Ordering::Relaxed);
    }

    /// Update GPU memory usage
    pub fn set_gpu_memory(&self, bytes: u64) {
        self.gpu_memory.store(bytes, Ordering::Relaxed);
    }

    /// Update texture memory usage
    pub fn set_texture_memory(&self, bytes: u64) {
        self.texture_memory.store(bytes, Ordering::Relaxed);
    }

    /// Update buffer memory usage
    pub fn set_buffer_memory(&self, bytes: u64) {
        self.buffer_memory.store(bytes, Ordering::Relaxed);
    }

    /// Get current memory statistics
    pub fn current_stats(&self) -> MemoryStats {
        MemoryStats {
            heap_used: self.heap_used.load(Ordering::Relaxed),
            allocated: self.allocated.load(Ordering::Relaxed),
            peak: self.peak.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
            gpu_memory: self.gpu_memory.load(Ordering::Relaxed),
            texture_memory: self.texture_memory.load(Ordering::Relaxed),
            buffer_memory: self.buffer_memory.load(Ordering::Relaxed),
        }
    }

    /// Get heap usage in bytes
    pub fn heap_used(&self) -> u64 {
        self.heap_used.load(Ordering::Relaxed)
    }

    /// Get GPU memory in bytes
    pub fn gpu_memory(&self) -> u64 {
        self.gpu_memory.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak(&self) -> u64 {
        self.peak.load(Ordering::Relaxed)
    }

    /// Get memory history for visualization
    pub fn history(&self) -> Vec<MemorySample> {
        self.history.read().clone()
    }

    /// Get memory trend (positive = increasing, negative = decreasing)
    pub fn heap_trend(&self) -> f64 {
        let history = self.history.read();
        if history.len() < 10 {
            return 0.0;
        }

        let recent: u64 = history.iter().rev().take(5).map(|s| s.heap_used).sum();
        let older: u64 = history.iter().rev().skip(5).take(5).map(|s| s.heap_used).sum();

        if older == 0 {
            0.0
        } else {
            ((recent as f64 - older as f64) / older as f64) * 100.0
        }
    }

    /// Check for potential memory leak (consistently increasing memory)
    pub fn potential_leak(&self) -> bool {
        self.heap_trend() > 10.0
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.heap_used.store(0, Ordering::Relaxed);
        self.allocated.store(0, Ordering::Relaxed);
        self.peak.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
        self.deallocation_count.store(0, Ordering::Relaxed);
        self.gpu_memory.store(0, Ordering::Relaxed);
        self.texture_memory.store(0, Ordering::Relaxed);
        self.buffer_memory.store(0, Ordering::Relaxed);
        self.history.write().clear();
    }

    /// Sample system memory info
    ///
    /// Note: This is a placeholder. For actual system memory info,
    /// consider using platform-specific APIs or crates like `sysinfo`.
    pub fn sample_system_memory(&self) {
        // Take a snapshot of current metrics
        self.sample();

        tracing::trace!(
            heap_used = self.heap_used(),
            gpu_memory = self.gpu_memory(),
            "Memory sampled"
        );
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Allocation tracking guard - records size on drop
pub struct AllocationGuard<'a> {
    metrics: &'a MemoryMetrics,
    size: u64,
}

impl<'a> AllocationGuard<'a> {
    /// Track an allocation
    pub fn track(metrics: &'a MemoryMetrics, size: u64) -> Self {
        metrics.record_allocation(size);
        Self { metrics, size }
    }
}

impl<'a> Drop for AllocationGuard<'a> {
    fn drop(&mut self) {
        self.metrics.record_deallocation(self.size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats() {
        let metrics = MemoryMetrics::new();

        metrics.record_allocation(1024);
        metrics.record_allocation(2048);

        let stats = metrics.current_stats();
        assert_eq!(stats.allocation_count, 2);
        assert_eq!(stats.heap_used, 3072);
        assert_eq!(stats.peak, 3072);

        metrics.record_deallocation(1024);
        let stats = metrics.current_stats();
        assert_eq!(stats.heap_used, 2048);
        assert_eq!(stats.deallocation_count, 1);
        assert_eq!(stats.peak, 3072); // Peak unchanged
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.46 KB");
        assert_eq!(format_bytes(1500000), "1.43 MB");
        assert_eq!(format_bytes(1500000000), "1.40 GB");
    }

    #[test]
    fn test_allocation_guard() {
        let metrics = MemoryMetrics::new();

        {
            let _guard = AllocationGuard::track(&metrics, 1024);
            assert_eq!(metrics.heap_used(), 1024);
        }

        assert_eq!(metrics.heap_used(), 0);
    }
}
