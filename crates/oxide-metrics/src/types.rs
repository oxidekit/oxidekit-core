//! Core metric types and definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MetricValue {
    /// Counter: monotonically increasing value
    Counter(u64),
    /// Gauge: value that can go up or down
    Gauge(f64),
    /// Histogram: distribution of values
    Histogram(HistogramData),
    /// Summary: percentile statistics
    Summary(SummaryData),
}

impl MetricValue {
    /// Get the numeric value (for counters and gauges)
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Counter(v) => Some(*v as f64),
            MetricValue::Gauge(v) => Some(*v),
            _ => None,
        }
    }

    /// Check if this is a counter
    pub fn is_counter(&self) -> bool {
        matches!(self, MetricValue::Counter(_))
    }

    /// Check if this is a gauge
    pub fn is_gauge(&self) -> bool {
        matches!(self, MetricValue::Gauge(_))
    }
}

/// Histogram bucket data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramData {
    /// Bucket boundaries (upper bounds)
    pub buckets: Vec<f64>,
    /// Counts per bucket
    pub counts: Vec<u64>,
    /// Total count of observations
    pub count: u64,
    /// Sum of all observations
    pub sum: f64,
}

impl HistogramData {
    /// Create a new histogram with default buckets
    pub fn new() -> Self {
        Self::with_buckets(vec![
            0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ])
    }

    /// Create a histogram with custom buckets
    pub fn with_buckets(buckets: Vec<f64>) -> Self {
        let counts = vec![0; buckets.len() + 1]; // +1 for +Inf bucket
        Self {
            buckets,
            counts,
            count: 0,
            sum: 0.0,
        }
    }

    /// Record a value in the histogram
    pub fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;

        // Find the appropriate bucket
        for (i, &bound) in self.buckets.iter().enumerate() {
            if value <= bound {
                self.counts[i] += 1;
                return;
            }
        }
        // Value exceeds all bounds, goes in +Inf bucket
        *self.counts.last_mut().unwrap() += 1;
    }

    /// Get the mean value
    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

impl Default for HistogramData {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics (percentiles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryData {
    /// Percentile values (e.g., 0.5, 0.9, 0.99)
    pub quantiles: Vec<f64>,
    /// Values at each quantile
    pub values: Vec<f64>,
    /// Total count
    pub count: u64,
    /// Sum of all values
    pub sum: f64,
}

impl SummaryData {
    /// Create a new summary with default quantiles
    pub fn new() -> Self {
        Self {
            quantiles: vec![0.5, 0.9, 0.95, 0.99],
            values: vec![0.0; 4],
            count: 0,
            sum: 0.0,
        }
    }
}

impl Default for SummaryData {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Metric metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricMetadata {
    /// Metric name (e.g., "oxide_frame_time_seconds")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Unit of measurement
    pub unit: MetricUnit,
    /// Labels/tags
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

impl MetricMetadata {
    /// Create new metric metadata
    pub fn new(name: &str, description: &str, metric_type: MetricType) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            metric_type,
            unit: MetricUnit::None,
            labels: HashMap::new(),
        }
    }

    /// Set the unit
    pub fn with_unit(mut self, unit: MetricUnit) -> Self {
        self.unit = unit;
        self
    }

    /// Add a label
    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_string(), value.to_string());
        self
    }
}

/// Units of measurement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricUnit {
    None,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Bytes,
    Kilobytes,
    Megabytes,
    Count,
    Percent,
    Fps,
}

impl MetricUnit {
    /// Get the Prometheus suffix for this unit
    pub fn prometheus_suffix(&self) -> &'static str {
        match self {
            MetricUnit::None => "",
            MetricUnit::Seconds => "_seconds",
            MetricUnit::Milliseconds => "_milliseconds",
            MetricUnit::Microseconds => "_microseconds",
            MetricUnit::Nanoseconds => "_nanoseconds",
            MetricUnit::Bytes => "_bytes",
            MetricUnit::Kilobytes => "_kilobytes",
            MetricUnit::Megabytes => "_megabytes",
            MetricUnit::Count => "_total",
            MetricUnit::Percent => "_percent",
            MetricUnit::Fps => "_fps",
        }
    }
}

/// A complete metric with metadata and value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metadata
    pub metadata: MetricMetadata,
    /// Current value
    pub value: MetricValue,
    /// Timestamp of last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Metric {
    /// Create a new counter metric
    pub fn counter(name: &str, description: &str, value: u64) -> Self {
        Self {
            metadata: MetricMetadata::new(name, description, MetricType::Counter),
            value: MetricValue::Counter(value),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Create a new gauge metric
    pub fn gauge(name: &str, description: &str, value: f64) -> Self {
        Self {
            metadata: MetricMetadata::new(name, description, MetricType::Gauge),
            value: MetricValue::Gauge(value),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Create a new histogram metric
    pub fn histogram(name: &str, description: &str) -> Self {
        Self {
            metadata: MetricMetadata::new(name, description, MetricType::Histogram),
            value: MetricValue::Histogram(HistogramData::new()),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Set the unit
    pub fn with_unit(mut self, unit: MetricUnit) -> Self {
        self.metadata.unit = unit;
        self
    }

    /// Add a label
    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.metadata.labels.insert(key.to_string(), value.to_string());
        self
    }

    /// Update the value
    pub fn set(&mut self, value: MetricValue) {
        self.value = value;
        self.updated_at = chrono::Utc::now();
    }
}

/// Time span for measuring durations
#[derive(Debug)]
pub struct TimeSpan {
    start: Instant,
    name: String,
}

impl TimeSpan {
    /// Start a new time span
    pub fn start(name: &str) -> Self {
        Self {
            start: Instant::now(),
            name: name.to_string(),
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Get the span name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// End the span and return the duration
    pub fn end(self) -> Duration {
        self.start.elapsed()
    }
}

/// Rolling window for computing averages
#[derive(Debug, Clone)]
pub struct RollingWindow {
    values: Vec<f64>,
    capacity: usize,
    index: usize,
    count: usize,
    sum: f64,
}

impl RollingWindow {
    /// Create a new rolling window with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            values: vec![0.0; capacity],
            capacity,
            index: 0,
            count: 0,
            sum: 0.0,
        }
    }

    /// Add a value to the window
    pub fn push(&mut self, value: f64) {
        // Subtract the value we're replacing from the sum
        if self.count == self.capacity {
            self.sum -= self.values[self.index];
        }

        // Add the new value
        self.values[self.index] = value;
        self.sum += value;

        // Update index and count
        self.index = (self.index + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    /// Get the average of values in the window
    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    /// Get the minimum value in the window
    pub fn min(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.values[..self.count]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min)
        }
    }

    /// Get the maximum value in the window
    pub fn max(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.values[..self.count]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max)
        }
    }

    /// Get the number of values currently in the window
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the window is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get all values in the window (in order from oldest to newest)
    pub fn values(&self) -> Vec<f64> {
        if self.count < self.capacity {
            self.values[..self.count].to_vec()
        } else {
            let mut result = Vec::with_capacity(self.capacity);
            result.extend_from_slice(&self.values[self.index..]);
            result.extend_from_slice(&self.values[..self.index]);
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram() {
        let mut hist = HistogramData::with_buckets(vec![1.0, 5.0, 10.0]);
        hist.observe(0.5);
        hist.observe(3.0);
        hist.observe(7.0);
        hist.observe(15.0);

        assert_eq!(hist.count, 4);
        assert_eq!(hist.counts[0], 1); // <= 1.0
        assert_eq!(hist.counts[1], 1); // <= 5.0
        assert_eq!(hist.counts[2], 1); // <= 10.0
        assert_eq!(hist.counts[3], 1); // +Inf
    }

    #[test]
    fn test_rolling_window() {
        let mut window = RollingWindow::new(3);
        window.push(1.0);
        window.push(2.0);
        window.push(3.0);
        assert_eq!(window.average(), 2.0);

        window.push(4.0);
        assert_eq!(window.average(), 3.0); // (2+3+4)/3

        assert_eq!(window.min(), 2.0);
        assert_eq!(window.max(), 4.0);
    }

    #[test]
    fn test_time_span() {
        let span = TimeSpan::start("test");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let duration = span.end();
        assert!(duration.as_millis() >= 1);
    }
}
