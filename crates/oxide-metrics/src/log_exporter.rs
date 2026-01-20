//! Structured log exporter
//!
//! Feature-gated behind `structured-logs` feature flag.
//! Exports metrics in structured JSON format compatible with:
//! - OpenTelemetry logging
//! - ELK Stack (Elasticsearch, Logstash, Kibana)
//! - Splunk
//! - CloudWatch Logs
//! - DataDog

use crate::{MetricsRegistry, MetricsSnapshot};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

/// Log export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Standard JSON lines (one JSON object per line)
    JsonLines,
    /// OpenTelemetry-compatible format
    OpenTelemetry,
    /// Elasticsearch-compatible format (with @timestamp)
    Elasticsearch,
    /// CloudWatch-compatible format
    CloudWatch,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::JsonLines
    }
}

/// Log exporter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogExporterConfig {
    /// Output format
    #[serde(default)]
    pub format: LogFormat,
    /// Output destination
    #[serde(default)]
    pub output: LogOutput,
    /// Include full snapshot data
    #[serde(default = "default_true")]
    pub include_full_snapshot: bool,
    /// Include labels in each record
    #[serde(default = "default_true")]
    pub include_labels: bool,
    /// Application name for logs
    #[serde(default = "default_app_name")]
    pub app_name: String,
    /// Environment name
    #[serde(default = "default_env")]
    pub environment: String,
    /// Additional fields to include
    #[serde(default)]
    pub extra_fields: HashMap<String, String>,
    /// Maximum records to buffer before flush
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
}

fn default_true() -> bool {
    true
}

fn default_app_name() -> String {
    "oxidekit".to_string()
}

fn default_env() -> String {
    "development".to_string()
}

fn default_buffer_size() -> usize {
    100
}

impl Default for LogExporterConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::JsonLines,
            output: LogOutput::Stdout,
            include_full_snapshot: true,
            include_labels: true,
            app_name: default_app_name(),
            environment: default_env(),
            extra_fields: HashMap::new(),
            buffer_size: default_buffer_size(),
        }
    }
}

/// Log output destination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "path")]
pub enum LogOutput {
    /// Write to stdout
    Stdout,
    /// Write to stderr
    Stderr,
    /// Write to a file
    File(PathBuf),
    /// Write to a rotating file (base path, max size in bytes)
    RotatingFile { path: PathBuf, max_size: u64 },
}

impl Default for LogOutput {
    fn default() -> Self {
        Self::Stdout
    }
}

/// Structured log exporter
pub struct LogExporter {
    config: LogExporterConfig,
    registry: &'static MetricsRegistry,
    buffer: RwLock<Vec<LogRecord>>,
}

/// A structured log record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    /// Timestamp in ISO 8601 format
    pub timestamp: String,
    /// Log level
    pub level: String,
    /// Logger/source name
    pub logger: String,
    /// Message
    pub message: String,
    /// Structured fields
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

impl LogRecord {
    /// Create a new log record for metrics
    pub fn metrics(snapshot: &MetricsSnapshot, config: &LogExporterConfig) -> Self {
        let mut fields = HashMap::new();

        // Add core metrics
        fields.insert("fps".to_string(), serde_json::json!(snapshot.frame.fps));
        fields.insert(
            "frame_time_ms".to_string(),
            serde_json::json!(snapshot.frame.avg_frame_time_ms),
        );
        fields.insert(
            "jank_percent".to_string(),
            serde_json::json!(snapshot.frame.jank_percent),
        );
        fields.insert(
            "memory_bytes".to_string(),
            serde_json::json!(snapshot.memory.heap_bytes),
        );
        fields.insert(
            "gpu_time_ms".to_string(),
            serde_json::json!(snapshot.render.avg_gpu_time_ms),
        );
        fields.insert(
            "draw_calls".to_string(),
            serde_json::json!(snapshot.render.draw_calls),
        );

        // Add app info
        fields.insert("app".to_string(), serde_json::json!(config.app_name));
        fields.insert("env".to_string(), serde_json::json!(config.environment));

        // Add extra fields
        for (k, v) in &config.extra_fields {
            fields.insert(k.clone(), serde_json::json!(v));
        }

        // Add labels if configured
        if config.include_labels {
            for (k, v) in &snapshot.labels {
                fields.insert(format!("label_{}", k), serde_json::json!(v));
            }
        }

        // Add full snapshot if configured
        if config.include_full_snapshot {
            fields.insert("snapshot".to_string(), serde_json::to_value(snapshot).unwrap_or_default());
        }

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: "INFO".to_string(),
            logger: "oxide.metrics".to_string(),
            message: format!(
                "metrics: fps={:.1}, frame_time={:.2}ms, memory={}",
                snapshot.frame.fps,
                snapshot.frame.avg_frame_time_ms,
                snapshot.memory.heap_formatted
            ),
            fields,
        }
    }

    /// Format as JSON lines
    pub fn to_json_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Format for OpenTelemetry
    pub fn to_otel(&self) -> String {
        let otel = serde_json::json!({
            "timestamp": self.timestamp,
            "severityText": self.level,
            "body": self.message,
            "attributes": self.fields,
            "resource": {
                "service.name": self.logger,
            }
        });
        serde_json::to_string(&otel).unwrap_or_default()
    }

    /// Format for Elasticsearch
    pub fn to_elasticsearch(&self) -> String {
        let mut es = self.fields.clone();
        es.insert("@timestamp".to_string(), serde_json::json!(self.timestamp));
        es.insert("level".to_string(), serde_json::json!(self.level));
        es.insert("logger".to_string(), serde_json::json!(self.logger));
        es.insert("message".to_string(), serde_json::json!(self.message));
        serde_json::to_string(&es).unwrap_or_default()
    }

    /// Format for CloudWatch
    pub fn to_cloudwatch(&self) -> String {
        let cw = serde_json::json!({
            "timestamp": chrono::DateTime::parse_from_rfc3339(&self.timestamp)
                .map(|t| t.timestamp_millis())
                .unwrap_or(0),
            "message": self.message,
            "data": self.fields,
        });
        serde_json::to_string(&cw).unwrap_or_default()
    }

    /// Format according to the specified format
    pub fn format(&self, format: LogFormat) -> String {
        match format {
            LogFormat::JsonLines => self.to_json_line(),
            LogFormat::OpenTelemetry => self.to_otel(),
            LogFormat::Elasticsearch => self.to_elasticsearch(),
            LogFormat::CloudWatch => self.to_cloudwatch(),
        }
    }
}

impl LogExporter {
    /// Create a new log exporter with default config
    pub fn new() -> Self {
        Self::with_config(LogExporterConfig::default())
    }

    /// Create a log exporter with custom config
    pub fn with_config(config: LogExporterConfig) -> Self {
        Self {
            config,
            registry: MetricsRegistry::global(),
            buffer: RwLock::new(Vec::new()),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &LogExporterConfig {
        &self.config
    }

    /// Export current metrics
    pub fn export(&self) -> Result<(), std::io::Error> {
        let snapshot = MetricsSnapshot::from_registry(self.registry);
        let record = LogRecord::metrics(&snapshot, &self.config);
        self.write_record(&record)
    }

    /// Export and add to buffer
    pub fn export_buffered(&self) -> Result<(), std::io::Error> {
        let snapshot = MetricsSnapshot::from_registry(self.registry);
        let record = LogRecord::metrics(&snapshot, &self.config);

        let mut buffer = self.buffer.write();
        buffer.push(record);

        // Auto-flush if buffer is full
        if buffer.len() >= self.config.buffer_size {
            drop(buffer);
            self.flush()?;
        }

        Ok(())
    }

    /// Flush the buffer
    pub fn flush(&self) -> Result<(), std::io::Error> {
        let records: Vec<LogRecord> = {
            let mut buffer = self.buffer.write();
            std::mem::take(&mut *buffer)
        };

        for record in records {
            self.write_record(&record)?;
        }

        Ok(())
    }

    /// Write a single record
    fn write_record(&self, record: &LogRecord) -> Result<(), std::io::Error> {
        let line = record.format(self.config.format);

        match &self.config.output {
            LogOutput::Stdout => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                writeln!(handle, "{}", line)?;
            }
            LogOutput::Stderr => {
                let stderr = std::io::stderr();
                let mut handle = stderr.lock();
                writeln!(handle, "{}", line)?;
            }
            LogOutput::File(path) => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                writeln!(file, "{}", line)?;
            }
            LogOutput::RotatingFile { path, max_size } => {
                // Simple rotation: check size and rename if needed
                if path.exists() {
                    let metadata = std::fs::metadata(path)?;
                    if metadata.len() >= *max_size {
                        let rotated = path.with_extension(format!(
                            "{}.log",
                            chrono::Utc::now().format("%Y%m%d_%H%M%S")
                        ));
                        std::fs::rename(path, rotated)?;
                    }
                }

                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                writeln!(file, "{}", line)?;
            }
        }

        Ok(())
    }

    /// Create a log record for a custom event
    pub fn log_event(&self, level: &str, message: &str, fields: HashMap<String, serde_json::Value>) -> Result<(), std::io::Error> {
        let mut all_fields = fields;
        all_fields.insert("app".to_string(), serde_json::json!(self.config.app_name));
        all_fields.insert("env".to_string(), serde_json::json!(self.config.environment));

        let record = LogRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: level.to_string(),
            logger: "oxide.app".to_string(),
            message: message.to_string(),
            fields: all_fields,
        };

        self.write_record(&record)
    }

    /// Get buffer size
    pub fn buffer_len(&self) -> usize {
        self.buffer.read().len()
    }

    /// Clear the buffer without writing
    pub fn clear_buffer(&self) {
        self.buffer.write().clear();
    }
}

impl Default for LogExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LogExporter {
    fn drop(&mut self) {
        // Attempt to flush remaining records
        let _ = self.flush();
    }
}

/// Builder for creating log exporters
pub struct LogExporterBuilder {
    config: LogExporterConfig,
}

impl LogExporterBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: LogExporterConfig::default(),
        }
    }

    /// Set the output format
    pub fn format(mut self, format: LogFormat) -> Self {
        self.config.format = format;
        self
    }

    /// Set output to stdout
    pub fn stdout(mut self) -> Self {
        self.config.output = LogOutput::Stdout;
        self
    }

    /// Set output to stderr
    pub fn stderr(mut self) -> Self {
        self.config.output = LogOutput::Stderr;
        self
    }

    /// Set output to a file
    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.output = LogOutput::File(path.into());
        self
    }

    /// Set output to a rotating file
    pub fn rotating_file(mut self, path: impl Into<PathBuf>, max_size: u64) -> Self {
        self.config.output = LogOutput::RotatingFile {
            path: path.into(),
            max_size,
        };
        self
    }

    /// Set the application name
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.config.app_name = name.into();
        self
    }

    /// Set the environment
    pub fn environment(mut self, env: impl Into<String>) -> Self {
        self.config.environment = env.into();
        self
    }

    /// Add an extra field
    pub fn extra_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.extra_fields.insert(key.into(), value.into());
        self
    }

    /// Set whether to include full snapshots
    pub fn include_full_snapshot(mut self, include: bool) -> Self {
        self.config.include_full_snapshot = include;
        self
    }

    /// Set the buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Build the exporter
    pub fn build(self) -> LogExporter {
        LogExporter::with_config(self.config)
    }
}

impl Default for LogExporterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_record_json_lines() {
        let snapshot = MetricsSnapshot::from_global();
        let config = LogExporterConfig::default();
        let record = LogRecord::metrics(&snapshot, &config);

        let json = record.to_json_line();
        assert!(json.contains("fps"));
        assert!(json.contains("oxide.metrics"));
    }

    #[test]
    fn test_log_record_otel() {
        let snapshot = MetricsSnapshot::from_global();
        let config = LogExporterConfig::default();
        let record = LogRecord::metrics(&snapshot, &config);

        let otel = record.to_otel();
        assert!(otel.contains("severityText"));
        assert!(otel.contains("attributes"));
    }

    #[test]
    fn test_log_record_elasticsearch() {
        let snapshot = MetricsSnapshot::from_global();
        let config = LogExporterConfig::default();
        let record = LogRecord::metrics(&snapshot, &config);

        let es = record.to_elasticsearch();
        assert!(es.contains("@timestamp"));
    }

    #[test]
    fn test_exporter_builder() {
        let exporter = LogExporterBuilder::new()
            .format(LogFormat::OpenTelemetry)
            .app_name("test-app")
            .environment("test")
            .extra_field("version", "1.0.0")
            .buffer_size(50)
            .build();

        assert_eq!(exporter.config.format, LogFormat::OpenTelemetry);
        assert_eq!(exporter.config.app_name, "test-app");
        assert_eq!(exporter.config.buffer_size, 50);
    }

    #[test]
    fn test_buffering() {
        let exporter = LogExporterBuilder::new()
            .stdout()
            .buffer_size(10)
            .build();

        // Add some records to the buffer
        for _ in 0..5 {
            let _ = exporter.export_buffered();
        }

        assert_eq!(exporter.buffer_len(), 5);

        exporter.clear_buffer();
        assert_eq!(exporter.buffer_len(), 0);
    }
}
