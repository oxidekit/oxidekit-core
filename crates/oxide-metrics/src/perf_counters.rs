//! Performance counters and detailed instrumentation
//!
//! Feature-gated behind `perf-counters` feature flag.
//! Provides fine-grained performance measurement for optimization work.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Global performance counters
pub struct PerfCounters {
    /// Named counters
    counters: RwLock<HashMap<String, Counter>>,
    /// Named timers
    timers: RwLock<HashMap<String, Timer>>,
    /// Named histograms
    histograms: RwLock<HashMap<String, Histogram>>,
    /// Active timers (for scoped timing)
    active_timers: RwLock<HashMap<u64, ActiveTimer>>,
    /// Next timer ID
    next_timer_id: AtomicU64,
    /// Enabled flag
    enabled: AtomicU64,
}

/// A simple counter
#[derive(Debug)]
struct Counter {
    value: AtomicU64,
    description: String,
}

impl Counter {
    fn new(description: &str) -> Self {
        Self {
            value: AtomicU64::new(0),
            description: description.to_string(),
        }
    }

    fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    fn add(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

/// A timer for measuring durations
#[derive(Debug)]
struct Timer {
    total_ns: AtomicU64,
    count: AtomicU64,
    min_ns: AtomicU64,
    max_ns: AtomicU64,
    description: String,
}

impl Timer {
    fn new(description: &str) -> Self {
        Self {
            total_ns: AtomicU64::new(0),
            count: AtomicU64::new(0),
            min_ns: AtomicU64::new(u64::MAX),
            max_ns: AtomicU64::new(0),
            description: description.to_string(),
        }
    }

    fn record(&self, duration: Duration) {
        let ns = duration.as_nanos() as u64;
        self.total_ns.fetch_add(ns, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);

        // Update min (atomic compare-and-swap loop)
        let mut current_min = self.min_ns.load(Ordering::Relaxed);
        while ns < current_min {
            match self.min_ns.compare_exchange_weak(
                current_min,
                ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }

        // Update max
        let mut current_max = self.max_ns.load(Ordering::Relaxed);
        while ns > current_max {
            match self.max_ns.compare_exchange_weak(
                current_max,
                ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }

    fn get_stats(&self) -> TimerStats {
        let count = self.count.load(Ordering::Relaxed);
        let total = self.total_ns.load(Ordering::Relaxed);
        let min = self.min_ns.load(Ordering::Relaxed);
        let max = self.max_ns.load(Ordering::Relaxed);

        TimerStats {
            count,
            total_ns: total,
            avg_ns: if count > 0 { total / count } else { 0 },
            min_ns: if min == u64::MAX { 0 } else { min },
            max_ns: max,
        }
    }

    fn reset(&self) {
        self.total_ns.store(0, Ordering::Relaxed);
        self.count.store(0, Ordering::Relaxed);
        self.min_ns.store(u64::MAX, Ordering::Relaxed);
        self.max_ns.store(0, Ordering::Relaxed);
    }
}

/// Timer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStats {
    /// Number of samples
    pub count: u64,
    /// Total time in nanoseconds
    pub total_ns: u64,
    /// Average time in nanoseconds
    pub avg_ns: u64,
    /// Minimum time in nanoseconds
    pub min_ns: u64,
    /// Maximum time in nanoseconds
    pub max_ns: u64,
}

impl TimerStats {
    /// Get total time as Duration
    pub fn total(&self) -> Duration {
        Duration::from_nanos(self.total_ns)
    }

    /// Get average time as Duration
    pub fn avg(&self) -> Duration {
        Duration::from_nanos(self.avg_ns)
    }

    /// Get minimum time as Duration
    pub fn min(&self) -> Duration {
        Duration::from_nanos(self.min_ns)
    }

    /// Get maximum time as Duration
    pub fn max(&self) -> Duration {
        Duration::from_nanos(self.max_ns)
    }

    /// Get average in milliseconds
    pub fn avg_ms(&self) -> f64 {
        self.avg_ns as f64 / 1_000_000.0
    }

    /// Get average in microseconds
    pub fn avg_us(&self) -> f64 {
        self.avg_ns as f64 / 1_000.0
    }
}

/// A histogram for tracking value distributions
#[derive(Debug)]
struct Histogram {
    buckets: Vec<AtomicU64>,
    boundaries: Vec<f64>,
    count: AtomicU64,
    sum: AtomicU64, // Stored as fixed-point (multiply by 1000)
    description: String,
}

impl Histogram {
    fn new(description: &str, boundaries: Vec<f64>) -> Self {
        let buckets = (0..=boundaries.len())
            .map(|_| AtomicU64::new(0))
            .collect();
        Self {
            buckets,
            boundaries,
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            description: description.to_string(),
        }
    }

    fn observe(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add((value * 1000.0) as u64, Ordering::Relaxed);

        // Find the bucket
        for (i, &bound) in self.boundaries.iter().enumerate() {
            if value <= bound {
                self.buckets[i].fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        // Overflow bucket
        self.buckets.last().unwrap().fetch_add(1, Ordering::Relaxed);
    }

    fn get_stats(&self) -> HistogramStats {
        let counts: Vec<u64> = self.buckets.iter().map(|b| b.load(Ordering::Relaxed)).collect();
        let count = self.count.load(Ordering::Relaxed);
        let sum = self.sum.load(Ordering::Relaxed) as f64 / 1000.0;

        HistogramStats {
            boundaries: self.boundaries.clone(),
            counts,
            count,
            sum,
            avg: if count > 0 { sum / count as f64 } else { 0.0 },
        }
    }

    fn reset(&self) {
        for bucket in &self.buckets {
            bucket.store(0, Ordering::Relaxed);
        }
        self.count.store(0, Ordering::Relaxed);
        self.sum.store(0, Ordering::Relaxed);
    }
}

/// Histogram statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramStats {
    /// Bucket boundaries
    pub boundaries: Vec<f64>,
    /// Counts per bucket
    pub counts: Vec<u64>,
    /// Total observations
    pub count: u64,
    /// Sum of all values
    pub sum: f64,
    /// Average value
    pub avg: f64,
}

/// An active timer for scoped timing
struct ActiveTimer {
    name: String,
    start: Instant,
}

impl PerfCounters {
    /// Create a new performance counter instance
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            timers: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            active_timers: RwLock::new(HashMap::new()),
            next_timer_id: AtomicU64::new(0),
            enabled: AtomicU64::new(1),
        }
    }

    /// Check if performance counters are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) != 0
    }

    /// Enable or disable performance counters
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(if enabled { 1 } else { 0 }, Ordering::Relaxed);
    }

    // --- Counter operations ---

    /// Create or get a counter
    pub fn counter(&self, name: &str, description: &str) {
        let mut counters = self.counters.write();
        counters.entry(name.to_string())
            .or_insert_with(|| Counter::new(description));
    }

    /// Increment a counter by 1
    pub fn inc(&self, name: &str) {
        if !self.is_enabled() {
            return;
        }
        if let Some(counter) = self.counters.read().get(name) {
            counter.increment();
        }
    }

    /// Add to a counter
    pub fn add(&self, name: &str, n: u64) {
        if !self.is_enabled() {
            return;
        }
        if let Some(counter) = self.counters.read().get(name) {
            counter.add(n);
        }
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.read().get(name).map(|c| c.get())
    }

    // --- Timer operations ---

    /// Create or get a timer
    pub fn timer(&self, name: &str, description: &str) {
        let mut timers = self.timers.write();
        timers.entry(name.to_string())
            .or_insert_with(|| Timer::new(description));
    }

    /// Start timing (returns ID for stopping)
    pub fn start_timer(&self, name: &str) -> u64 {
        if !self.is_enabled() {
            return 0;
        }

        let id = self.next_timer_id.fetch_add(1, Ordering::Relaxed);
        self.active_timers.write().insert(
            id,
            ActiveTimer {
                name: name.to_string(),
                start: Instant::now(),
            },
        );
        id
    }

    /// Stop timing and record duration
    pub fn stop_timer(&self, id: u64) {
        if !self.is_enabled() || id == 0 {
            return;
        }

        if let Some(active) = self.active_timers.write().remove(&id) {
            let duration = active.start.elapsed();
            if let Some(timer) = self.timers.read().get(&active.name) {
                timer.record(duration);
            }
        }
    }

    /// Record a timer duration directly
    pub fn record_timer(&self, name: &str, duration: Duration) {
        if !self.is_enabled() {
            return;
        }
        if let Some(timer) = self.timers.read().get(name) {
            timer.record(duration);
        }
    }

    /// Get timer statistics
    pub fn get_timer(&self, name: &str) -> Option<TimerStats> {
        self.timers.read().get(name).map(|t| t.get_stats())
    }

    // --- Histogram operations ---

    /// Create or get a histogram
    pub fn histogram(&self, name: &str, description: &str, boundaries: Vec<f64>) {
        let mut histograms = self.histograms.write();
        histograms.entry(name.to_string())
            .or_insert_with(|| Histogram::new(description, boundaries));
    }

    /// Observe a histogram value
    pub fn observe(&self, name: &str, value: f64) {
        if !self.is_enabled() {
            return;
        }
        if let Some(hist) = self.histograms.read().get(name) {
            hist.observe(value);
        }
    }

    /// Get histogram statistics
    pub fn get_histogram(&self, name: &str) -> Option<HistogramStats> {
        self.histograms.read().get(name).map(|h| h.get_stats())
    }

    // --- Reporting ---

    /// Get all counter values
    pub fn all_counters(&self) -> HashMap<String, u64> {
        self.counters
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.get()))
            .collect()
    }

    /// Get all timer statistics
    pub fn all_timers(&self) -> HashMap<String, TimerStats> {
        self.timers
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.get_stats()))
            .collect()
    }

    /// Get all histogram statistics
    pub fn all_histograms(&self) -> HashMap<String, HistogramStats> {
        self.histograms
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.get_stats()))
            .collect()
    }

    /// Generate a report
    pub fn report(&self) -> PerfReport {
        PerfReport {
            timestamp: chrono::Utc::now(),
            counters: self.all_counters(),
            timers: self.all_timers(),
            histograms: self.all_histograms(),
        }
    }

    /// Reset all counters, timers, and histograms
    pub fn reset_all(&self) {
        for counter in self.counters.read().values() {
            counter.reset();
        }
        for timer in self.timers.read().values() {
            timer.reset();
        }
        for hist in self.histograms.read().values() {
            hist.reset();
        }
    }
}

impl Default for PerfCounters {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfReport {
    /// Timestamp of the report
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// All counter values
    pub counters: HashMap<String, u64>,
    /// All timer statistics
    pub timers: HashMap<String, TimerStats>,
    /// All histogram statistics
    pub histograms: HashMap<String, HistogramStats>,
}

impl PerfReport {
    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Print a human-readable summary
    pub fn summary(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Performance Report ===\n\n");

        if !self.counters.is_empty() {
            output.push_str("Counters:\n");
            for (name, value) in &self.counters {
                output.push_str(&format!("  {}: {}\n", name, value));
            }
            output.push('\n');
        }

        if !self.timers.is_empty() {
            output.push_str("Timers:\n");
            for (name, stats) in &self.timers {
                output.push_str(&format!(
                    "  {}: count={}, avg={:.3}ms, min={:.3}ms, max={:.3}ms\n",
                    name,
                    stats.count,
                    stats.avg_ms(),
                    stats.min_ns as f64 / 1_000_000.0,
                    stats.max_ns as f64 / 1_000_000.0
                ));
            }
            output.push('\n');
        }

        if !self.histograms.is_empty() {
            output.push_str("Histograms:\n");
            for (name, stats) in &self.histograms {
                output.push_str(&format!(
                    "  {}: count={}, avg={:.3}\n",
                    name, stats.count, stats.avg
                ));
            }
        }

        output
    }
}

/// Timer guard for scoped timing
pub struct TimerGuard<'a> {
    counters: &'a PerfCounters,
    id: u64,
}

impl<'a> TimerGuard<'a> {
    /// Start timing
    pub fn new(counters: &'a PerfCounters, name: &str) -> Self {
        let id = counters.start_timer(name);
        Self { counters, id }
    }
}

impl<'a> Drop for TimerGuard<'a> {
    fn drop(&mut self) {
        self.counters.stop_timer(self.id);
    }
}

/// Macro for easy timer scoping
#[macro_export]
macro_rules! perf_timer {
    ($counters:expr, $name:expr) => {
        let _guard = $crate::perf_counters::TimerGuard::new($counters, $name);
    };
}

/// Pre-defined counters for render pipeline
pub mod render_counters {
    pub const DRAW_CALLS: &str = "render.draw_calls";
    pub const STATE_CHANGES: &str = "render.state_changes";
    pub const BUFFER_UPLOADS: &str = "render.buffer_uploads";
    pub const TEXTURE_BINDS: &str = "render.texture_binds";
    pub const SHADER_SWITCHES: &str = "render.shader_switches";
    pub const PIPELINE_BINDS: &str = "render.pipeline_binds";
}

/// Pre-defined timers for render pipeline
pub mod render_timers {
    pub const FRAME_TOTAL: &str = "render.frame.total";
    pub const FRAME_CPU: &str = "render.frame.cpu";
    pub const FRAME_GPU: &str = "render.frame.gpu";
    pub const LAYOUT: &str = "render.layout";
    pub const TRAVERSE: &str = "render.traverse";
    pub const BATCH: &str = "render.batch";
    pub const SUBMIT: &str = "render.submit";
    pub const PRESENT: &str = "render.present";
}

/// Initialize standard render counters and timers
pub fn init_render_counters(counters: &PerfCounters) {
    // Counters
    counters.counter(render_counters::DRAW_CALLS, "Number of draw calls per frame");
    counters.counter(render_counters::STATE_CHANGES, "Number of render state changes");
    counters.counter(render_counters::BUFFER_UPLOADS, "Number of buffer uploads");
    counters.counter(render_counters::TEXTURE_BINDS, "Number of texture bindings");
    counters.counter(render_counters::SHADER_SWITCHES, "Number of shader switches");
    counters.counter(render_counters::PIPELINE_BINDS, "Number of pipeline bindings");

    // Timers
    counters.timer(render_timers::FRAME_TOTAL, "Total frame time");
    counters.timer(render_timers::FRAME_CPU, "CPU-side frame time");
    counters.timer(render_timers::FRAME_GPU, "GPU-side frame time");
    counters.timer(render_timers::LAYOUT, "Layout computation time");
    counters.timer(render_timers::TRAVERSE, "Tree traversal time");
    counters.timer(render_timers::BATCH, "Batch building time");
    counters.timer(render_timers::SUBMIT, "Command submission time");
    counters.timer(render_timers::PRESENT, "Present/swap time");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_counter() {
        let counters = PerfCounters::new();
        counters.counter("test", "Test counter");

        counters.inc("test");
        counters.inc("test");
        counters.add("test", 5);

        assert_eq!(counters.get_counter("test"), Some(7));
    }

    #[test]
    fn test_timer() {
        let counters = PerfCounters::new();
        counters.timer("test", "Test timer");

        let id = counters.start_timer("test");
        sleep(Duration::from_millis(10));
        counters.stop_timer(id);

        let stats = counters.get_timer("test").unwrap();
        assert_eq!(stats.count, 1);
        assert!(stats.avg_ms() >= 10.0);
    }

    #[test]
    fn test_timer_guard() {
        let counters = PerfCounters::new();
        counters.timer("test", "Test timer");

        {
            let _guard = TimerGuard::new(&counters, "test");
            sleep(Duration::from_millis(5));
        }

        let stats = counters.get_timer("test").unwrap();
        assert_eq!(stats.count, 1);
    }

    #[test]
    fn test_histogram() {
        let counters = PerfCounters::new();
        counters.histogram("test", "Test histogram", vec![1.0, 5.0, 10.0]);

        counters.observe("test", 0.5);
        counters.observe("test", 3.0);
        counters.observe("test", 7.0);
        counters.observe("test", 15.0);

        let stats = counters.get_histogram("test").unwrap();
        assert_eq!(stats.count, 4);
        assert_eq!(stats.counts.len(), 4); // 3 bounds + overflow
    }

    #[test]
    fn test_report() {
        let counters = PerfCounters::new();
        counters.counter("c1", "Counter 1");
        counters.timer("t1", "Timer 1");

        counters.inc("c1");
        counters.record_timer("t1", Duration::from_millis(10));

        let report = counters.report();
        assert!(report.counters.contains_key("c1"));
        assert!(report.timers.contains_key("t1"));

        let summary = report.summary();
        assert!(summary.contains("c1"));
        assert!(summary.contains("t1"));
    }

    #[test]
    fn test_disabled() {
        let counters = PerfCounters::new();
        counters.counter("test", "Test");
        counters.set_enabled(false);

        counters.inc("test");
        assert_eq!(counters.get_counter("test"), Some(0));

        counters.set_enabled(true);
        counters.inc("test");
        assert_eq!(counters.get_counter("test"), Some(1));
    }
}
