//! OxideKit Diagnostics
//!
//! Production-safe diagnostics and error reporting system.
//!
//! # Design Principles
//!
//! - **Devtools are for developers. Diagnostics are for production.**
//! - No devtools code ships in release builds
//! - No data exfiltration without explicit consent
//! - Structured, machine-readable error codes
//! - Privacy-safe by default
//!
//! # Features
//!
//! - `full` - Enable all diagnostics features
//! - `bundle-export` - Enable diagnostics bundle export
//! - `auto-report` - Enable optional auto-reporting (requires opt-in)
//! - `crash-handler` - Enable crash handling
//! - `devtools` - Development-only features (never ship in release)

mod error;
mod event;
mod bundle;
mod redact;

#[cfg(feature = "crash-handler")]
mod crash;

#[cfg(feature = "auto-report")]
mod report;

pub use error::*;
pub use event::*;
pub use bundle::*;
pub use redact::*;

#[cfg(feature = "crash-handler")]
pub use crash::*;

#[cfg(feature = "auto-report")]
pub use report::*;

use std::sync::RwLock;
use std::collections::VecDeque;

/// Maximum number of events to retain in memory
const MAX_EVENTS: usize = 1000;

/// Maximum number of log entries to retain
const MAX_LOGS: usize = 500;

/// Global diagnostics collector
pub struct DiagnosticsCollector {
    /// Application info
    app_info: AppInfo,

    /// Recent diagnostic events
    events: RwLock<VecDeque<DiagnosticEvent>>,

    /// Recent log entries
    logs: RwLock<VecDeque<LogEntry>>,

    /// Configuration
    config: DiagnosticsConfig,
}

impl DiagnosticsCollector {
    /// Create a new diagnostics collector
    pub fn new(app_info: AppInfo, config: DiagnosticsConfig) -> Self {
        Self {
            app_info,
            events: RwLock::new(VecDeque::with_capacity(MAX_EVENTS)),
            logs: RwLock::new(VecDeque::with_capacity(MAX_LOGS)),
            config,
        }
    }

    /// Record a diagnostic event
    pub fn record_event(&self, event: DiagnosticEvent) {
        if let Ok(mut events) = self.events.write() {
            if events.len() >= MAX_EVENTS {
                events.pop_front();
            }

            tracing::debug!(
                code = %event.error_code,
                severity = ?event.severity,
                "Diagnostic event recorded"
            );

            events.push_back(event);
        }
    }

    /// Record a log entry
    pub fn record_log(&self, entry: LogEntry) {
        if let Ok(mut logs) = self.logs.write() {
            if logs.len() >= MAX_LOGS {
                logs.pop_front();
            }
            logs.push_back(entry);
        }
    }

    /// Get recent events
    pub fn get_events(&self) -> Vec<DiagnosticEvent> {
        self.events
            .read()
            .map(|e| e.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get recent logs
    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.logs
            .read()
            .map(|l| l.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Clear all events
    pub fn clear_events(&self) {
        if let Ok(mut events) = self.events.write() {
            events.clear();
        }
    }

    /// Clear all logs
    pub fn clear_logs(&self) {
        if let Ok(mut logs) = self.logs.write() {
            logs.clear();
        }
    }

    /// Get app info
    pub fn app_info(&self) -> &AppInfo {
        &self.app_info
    }

    /// Get config
    pub fn config(&self) -> &DiagnosticsConfig {
        &self.config
    }

    /// Export a diagnostics bundle
    #[cfg(feature = "bundle-export")]
    pub fn export_bundle(&self) -> DiagnosticsBundle {
        let events = self.get_events();
        let logs = self.get_logs();

        // Redact sensitive data
        let redacted_logs: Vec<LogEntry> = logs
            .into_iter()
            .map(|l| redact_log_entry(l, &self.config.redaction_rules))
            .collect();

        DiagnosticsBundle::new(
            self.app_info.clone(),
            events,
            redacted_logs,
        )
    }
}

/// Application information for diagnostics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppInfo {
    /// Application name
    pub name: String,

    /// Application version
    pub version: String,

    /// Build ID (git hash or build number)
    pub build_id: String,

    /// OxideKit core version
    pub oxidekit_version: String,

    /// Operating system
    pub os: String,

    /// Architecture
    pub arch: String,

    /// Build profile (dev, release, release+diagnostics)
    pub build_profile: BuildProfile,
}

impl AppInfo {
    /// Create AppInfo with current environment
    pub fn from_env(name: &str, version: &str, build_id: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            build_id: build_id.to_string(),
            oxidekit_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            build_profile: BuildProfile::current(),
        }
    }
}

/// Build profile
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BuildProfile {
    Dev,
    Release,
    ReleaseDiagnostics,
}

impl BuildProfile {
    /// Detect current build profile
    pub fn current() -> Self {
        #[cfg(debug_assertions)]
        {
            BuildProfile::Dev
        }
        #[cfg(not(debug_assertions))]
        {
            #[cfg(feature = "full")]
            {
                BuildProfile::ReleaseDiagnostics
            }
            #[cfg(not(feature = "full"))]
            {
                BuildProfile::Release
            }
        }
    }

    /// Check if devtools should be available
    pub fn devtools_available(&self) -> bool {
        matches!(self, BuildProfile::Dev)
    }

    /// Check if diagnostics should be available
    pub fn diagnostics_available(&self) -> bool {
        !matches!(self, BuildProfile::Release)
    }
}

/// Diagnostics configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticsConfig {
    /// Enable auto-reporting (requires opt-in)
    #[serde(default)]
    pub auto_report: bool,

    /// Auto-report endpoint URL
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Redaction rules
    #[serde(default)]
    pub redaction_rules: RedactionRules,

    /// Include stack traces in error reports
    #[serde(default)]
    pub include_stack_traces: bool,

    /// Include full file paths (privacy concern)
    #[serde(default)]
    pub include_full_paths: bool,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            auto_report: false,
            endpoint: None,
            redaction_rules: RedactionRules::default(),
            include_stack_traces: false,
            include_full_paths: false,
        }
    }
}

/// Log entry for diagnostics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Log level
    pub level: LogLevel,

    /// Category (ui, layout, render, extension, network)
    pub category: String,

    /// Message
    pub message: String,

    /// Additional fields
    #[serde(default)]
    pub fields: std::collections::HashMap<String, serde_json::Value>,
}

/// Log levels
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogEntry {
    pub fn new(level: LogLevel, category: &str, message: &str) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level,
            category: category.to_string(),
            message: message.to_string(),
            fields: std::collections::HashMap::new(),
        }
    }

    pub fn with_field(mut self, key: &str, value: impl serde::Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.fields.insert(key.to_string(), v);
        }
        self
    }
}

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        DiagnosticsCollector, DiagnosticsConfig, AppInfo, BuildProfile,
        DiagnosticEvent, ErrorCode, ErrorDomain, Severity,
        LogEntry, LogLevel,
    };

    #[cfg(feature = "bundle-export")]
    pub use crate::DiagnosticsBundle;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_info() {
        let info = AppInfo::from_env("TestApp", "1.0.0", "abc123");
        assert_eq!(info.name, "TestApp");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_build_profile() {
        let profile = BuildProfile::current();
        // In test mode, should be Dev
        assert!(profile.devtools_available());
    }

    #[test]
    fn test_log_entry() {
        let entry = LogEntry::new(LogLevel::Info, "test", "Hello")
            .with_field("count", 42);

        assert_eq!(entry.category, "test");
        assert!(entry.fields.contains_key("count"));
    }

    #[test]
    fn test_collector() {
        let app_info = AppInfo::from_env("Test", "1.0.0", "test");
        let collector = DiagnosticsCollector::new(app_info, DiagnosticsConfig::default());

        let event = DiagnosticEvent::new(
            ErrorCode::new(ErrorDomain::Ui, 100),
            Severity::Warning,
            "Test warning",
        );

        collector.record_event(event);
        assert_eq!(collector.get_events().len(), 1);
    }
}
