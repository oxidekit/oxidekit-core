//! Diagnostics Bundle Export
//!
//! Creates structured bundles of diagnostic data for export and sharing.

use crate::{AppInfo, DiagnosticEvent, LogEntry};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A diagnostics bundle for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsBundle {
    /// Bundle ID
    pub id: Uuid,

    /// Bundle format version
    pub version: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Application information
    pub app_info: AppInfo,

    /// System information
    pub system_info: SystemInfo,

    /// Diagnostic events
    pub events: Vec<DiagnosticEvent>,

    /// Recent logs (redacted)
    pub logs: Vec<LogEntry>,

    /// Installed extensions
    #[serde(default)]
    pub extensions: Vec<ExtensionInfo>,

    /// Permissions configuration summary
    #[serde(default)]
    pub permissions: PermissionsSummary,

    /// Crash report (if present)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crash_report: Option<BundleCrashReport>,

    /// Bundle metadata
    #[serde(default)]
    pub metadata: BundleMetadata,
}

impl DiagnosticsBundle {
    /// Create a new diagnostics bundle
    pub fn new(
        app_info: AppInfo,
        events: Vec<DiagnosticEvent>,
        logs: Vec<LogEntry>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            version: "1.0".to_string(),
            created_at: Utc::now(),
            app_info,
            system_info: SystemInfo::current(),
            events,
            logs,
            extensions: Vec::new(),
            permissions: PermissionsSummary::default(),
            crash_report: None,
            metadata: BundleMetadata::default(),
        }
    }

    /// Add extension information
    pub fn with_extensions(mut self, extensions: Vec<ExtensionInfo>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Add permissions summary
    pub fn with_permissions(mut self, permissions: PermissionsSummary) -> Self {
        self.permissions = permissions;
        self
    }

    /// Add crash report
    pub fn with_crash_report(mut self, report: BundleCrashReport) -> Self {
        self.crash_report = Some(report);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.metadata.custom.insert(key.to_string(), v);
        }
        self
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export to minified JSON
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Save to file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let json = self.to_json().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?;
        std::fs::write(path, json)
    }

    /// Load from file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })
    }

    /// Get event statistics
    pub fn event_stats(&self) -> crate::event::EventStats {
        crate::event::EventStats::from_events(&self.events)
    }

    /// Filter events by severity
    pub fn filter_events_by_severity(&self, min_severity: crate::Severity) -> Vec<&DiagnosticEvent> {
        self.events
            .iter()
            .filter(|e| e.severity >= min_severity)
            .collect()
    }
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,

    /// OS version
    pub os_version: String,

    /// Architecture
    pub arch: String,

    /// CPU info
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,

    /// Memory info (total, in MB)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_mb: Option<u64>,

    /// GPU info
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu: Option<String>,

    /// Display info
    #[serde(default)]
    pub displays: Vec<DisplayInfo>,
}

impl SystemInfo {
    /// Get current system information
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            os_version: Self::get_os_version(),
            arch: std::env::consts::ARCH.to_string(),
            cpu: None, // Would require sys-info crate
            memory_mb: None,
            gpu: None,
            displays: Vec::new(),
        }
    }

    fn get_os_version() -> String {
        // Basic version detection
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        }

        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/etc/os-release")
                .ok()
                .and_then(|content| {
                    content
                        .lines()
                        .find(|l| l.starts_with("VERSION_ID="))
                        .map(|l| l.trim_start_matches("VERSION_ID=").trim_matches('"').to_string())
                })
                .unwrap_or_else(|| "unknown".to_string())
        }

        #[cfg(target_os = "windows")]
        {
            "unknown".to_string()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            "unknown".to_string()
        }
    }
}

/// Display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    /// Display name/identifier
    pub name: String,

    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,

    /// Scale factor
    pub scale: f32,
}

/// Extension information for bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    /// Extension ID
    pub id: String,

    /// Extension version
    pub version: String,

    /// Extension source (marketplace, local, etc.)
    pub source: String,

    /// Whether extension is enabled
    pub enabled: bool,
}

/// Permissions summary (no detailed info for privacy)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionsSummary {
    /// Number of filesystem permissions
    pub filesystem_count: usize,

    /// Number of network permissions
    pub network_count: usize,

    /// Number of system permissions
    pub system_count: usize,

    /// Whether any dangerous permissions are enabled
    pub has_dangerous: bool,
}

/// Crash report data (lightweight version for bundle inclusion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleCrashReport {
    /// Crash ID
    pub id: Uuid,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Crash reason (high-level)
    pub reason: String,

    /// Error code (if mapped)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<crate::ErrorCode>,

    /// Thread ID where crash occurred
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    /// Signal (on Unix)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<i32>,

    /// Stack trace (if available and requested)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<Vec<BundleStackFrame>>,
}

impl BundleCrashReport {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            reason: reason.into(),
            error_code: None,
            thread_id: None,
            signal: None,
            stack_trace: None,
        }
    }
}

/// Stack frame (minimal, for bundle crash reports)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleStackFrame {
    /// Instruction pointer (hex)
    pub ip: String,

    /// Symbol name (if available)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    /// Module name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,

    /// Offset within module
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
}

/// Bundle metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleMetadata {
    /// User-provided description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// User consent for auto-report
    #[serde(default)]
    pub auto_report_consent: bool,

    /// Custom fields
    #[serde(default, flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ErrorCode, Severity};

    #[test]
    fn test_bundle_creation() {
        let app_info = AppInfo::from_env("TestApp", "1.0.0", "abc123");
        let events = vec![
            DiagnosticEvent::new(ErrorCode::UI_INVALID_PROP, Severity::Warning, "test"),
        ];

        let bundle = DiagnosticsBundle::new(app_info, events, vec![]);

        assert_eq!(bundle.version, "1.0");
        assert_eq!(bundle.events.len(), 1);
    }

    #[test]
    fn test_bundle_serialization() {
        let app_info = AppInfo::from_env("TestApp", "1.0.0", "abc123");
        let bundle = DiagnosticsBundle::new(app_info, vec![], vec![]);

        let json = bundle.to_json().unwrap();
        assert!(json.contains("TestApp"));
        assert!(json.contains("version"));
    }

    #[test]
    fn test_system_info() {
        let info = SystemInfo::current();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
    }
}
