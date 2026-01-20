//! Crash Handling
//!
//! Catches panics and writes crash reports to local files.

use crate::{AppInfo, RedactionRules, redact_string};
use serde::{Deserialize, Serialize};
use std::fs;
use std::panic::{self, PanicHookInfo};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Global crash handler configuration
static CRASH_CONFIG: OnceLock<CrashConfig> = OnceLock::new();

/// Crash handler configuration
#[derive(Debug, Clone)]
pub struct CrashConfig {
    /// Directory to store crash reports
    pub crash_dir: PathBuf,

    /// Application info
    pub app_info: AppInfo,

    /// Redaction rules for crash reports
    pub redaction_rules: RedactionRules,

    /// Maximum number of crash files to keep
    pub max_crash_files: usize,

    /// Whether to include stack traces
    pub include_stack_trace: bool,
}

impl Default for CrashConfig {
    fn default() -> Self {
        Self {
            crash_dir: PathBuf::from(".oxidekit/crashes"),
            app_info: AppInfo::from_env("Unknown", "0.0.0", "unknown"),
            redaction_rules: RedactionRules::default(),
            max_crash_files: 10,
            include_stack_trace: true,
        }
    }
}

/// A crash report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    /// Unique crash ID
    pub id: uuid::Uuid,

    /// Timestamp of the crash
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Application info at time of crash
    pub app_info: AppInfo,

    /// Panic message
    pub message: String,

    /// Location of the panic (file:line)
    pub location: Option<String>,

    /// Stack trace (if available)
    pub stack_trace: Option<Vec<StackFrame>>,

    /// System info snapshot
    pub system_info: SystemSnapshot,
}

/// A frame in the stack trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Frame index
    pub index: usize,

    /// Symbol name (demangled if possible)
    pub symbol: Option<String>,

    /// File path (redacted)
    pub file: Option<String>,

    /// Line number
    pub line: Option<u32>,

    /// Column number
    pub column: Option<u32>,
}

/// System snapshot at time of crash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Operating system
    pub os: String,

    /// Architecture
    pub arch: String,

    /// Number of CPUs
    pub num_cpus: usize,

    /// Current thread name
    pub thread_name: Option<String>,

    /// Thread ID
    pub thread_id: String,
}

impl CrashReport {
    /// Create a new crash report from panic info
    pub fn from_panic(info: &PanicHookInfo<'_>, config: &CrashConfig) -> Self {
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        // Redact the message
        let message = redact_string(&message, &config.redaction_rules);

        // Get location
        let location = info.location().map(|loc| {
            let loc_str = format!("{}:{}:{}", loc.file(), loc.line(), loc.column());
            redact_string(&loc_str, &config.redaction_rules)
        });

        // Capture stack trace
        let stack_trace = if config.include_stack_trace {
            Some(capture_stack_trace(&config.redaction_rules))
        } else {
            None
        };

        // Get current thread info
        let thread = std::thread::current();
        let thread_name = thread.name().map(|s| s.to_string());

        Self {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            app_info: config.app_info.clone(),
            message,
            location,
            stack_trace,
            system_info: SystemSnapshot {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                num_cpus: std::thread::available_parallelism()
                    .map(|p| p.get())
                    .unwrap_or(1),
                thread_name,
                thread_id: format!("{:?}", thread.id()),
            },
        }
    }

    /// Save the crash report to a file
    pub fn save_to_file(&self, crash_dir: &Path) -> std::io::Result<PathBuf> {
        // Ensure crash directory exists
        fs::create_dir_all(crash_dir)?;

        // Generate filename: crash_<timestamp>_<short_id>.json
        let timestamp = self.timestamp.format("%Y%m%d_%H%M%S");
        let short_id = &self.id.to_string()[..8];
        let filename = format!("crash_{}_{}.json", timestamp, short_id);
        let path = crash_dir.join(filename);

        // Write crash report
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;

        Ok(path)
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Capture the current stack trace
fn capture_stack_trace(rules: &RedactionRules) -> Vec<StackFrame> {
    let mut frames = Vec::new();

    // Use backtrace crate if available, otherwise return empty
    // For now, we'll capture a simple representation
    let bt = std::backtrace::Backtrace::capture();
    let bt_str = format!("{}", bt);

    // Parse the backtrace string into frames
    for (index, line) in bt_str.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        // Redact the line
        let redacted_line = redact_string(line, rules);

        frames.push(StackFrame {
            index,
            symbol: Some(redacted_line),
            file: None,
            line: None,
            column: None,
        });

        // Limit to 50 frames
        if frames.len() >= 50 {
            break;
        }
    }

    frames
}

/// Install the crash handler
///
/// This sets up a panic hook that captures crash reports and writes them to files.
/// Call this early in your application's startup.
pub fn install_crash_handler(config: CrashConfig) {
    // Store config for later use
    let _ = CRASH_CONFIG.set(config.clone());

    // Get the previous panic hook (for chaining)
    let prev_hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        // Get config (should always succeed since we set it above)
        if let Some(config) = CRASH_CONFIG.get() {
            // Create crash report
            let report = CrashReport::from_panic(info, config);

            // Try to save it
            match report.save_to_file(&config.crash_dir) {
                Ok(path) => {
                    eprintln!("Crash report saved to: {}", path.display());
                }
                Err(e) => {
                    eprintln!("Failed to save crash report: {}", e);
                }
            }

            // Clean up old crash files
            if let Err(e) = cleanup_old_crashes(&config.crash_dir, config.max_crash_files) {
                eprintln!("Failed to clean up old crash files: {}", e);
            }
        }

        // Call the previous panic hook
        prev_hook(info);
    }));
}

/// Clean up old crash files, keeping only the most recent ones
fn cleanup_old_crashes(crash_dir: &Path, max_files: usize) -> std::io::Result<()> {
    if !crash_dir.exists() {
        return Ok(());
    }

    let mut crash_files: Vec<_> = fs::read_dir(crash_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("crash_")
                && entry.file_name().to_string_lossy().ends_with(".json")
        })
        .collect();

    // Sort by modification time (newest first)
    crash_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    // Remove old files
    for file in crash_files.into_iter().skip(max_files) {
        let _ = fs::remove_file(file.path());
    }

    Ok(())
}

/// Get the crash handler configuration (if installed)
pub fn get_crash_config() -> Option<&'static CrashConfig> {
    CRASH_CONFIG.get()
}

/// List all crash reports in the crash directory
pub fn list_crash_reports(crash_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    if !crash_dir.exists() {
        return Ok(Vec::new());
    }

    let mut reports: Vec<_> = fs::read_dir(crash_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("crash_")
                && entry.file_name().to_string_lossy().ends_with(".json")
        })
        .map(|entry| entry.path())
        .collect();

    // Sort by filename (which includes timestamp)
    reports.sort();
    reports.reverse(); // Newest first

    Ok(reports)
}

/// Load a crash report from file
pub fn load_crash_report(path: &Path) -> std::io::Result<CrashReport> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_report_creation() {
        let config = CrashConfig::default();
        let report = CrashReport {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            app_info: config.app_info.clone(),
            message: "Test panic".to_string(),
            location: Some("test.rs:42:5".to_string()),
            stack_trace: None,
            system_info: SystemSnapshot {
                os: "test".to_string(),
                arch: "x86_64".to_string(),
                num_cpus: 4,
                thread_name: Some("main".to_string()),
                thread_id: "ThreadId(1)".to_string(),
            },
        };

        let json = report.to_json().unwrap();
        assert!(json.contains("Test panic"));
    }

    #[test]
    fn test_crash_report_serialization() {
        let report = CrashReport {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            app_info: AppInfo::from_env("TestApp", "1.0.0", "test"),
            message: "Test crash".to_string(),
            location: None,
            stack_trace: Some(vec![StackFrame {
                index: 0,
                symbol: Some("test_function".to_string()),
                file: Some("test.rs".to_string()),
                line: Some(10),
                column: Some(1),
            }]),
            system_info: SystemSnapshot {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                num_cpus: 8,
                thread_name: Some("worker".to_string()),
                thread_id: "ThreadId(2)".to_string(),
            },
        };

        // Serialize
        let json = serde_json::to_string(&report).unwrap();

        // Deserialize
        let parsed: CrashReport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.message, "Test crash");
        assert_eq!(parsed.app_info.name, "TestApp");
    }

    #[test]
    fn test_system_snapshot() {
        let snapshot = SystemSnapshot {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            num_cpus: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1),
            thread_name: std::thread::current().name().map(|s| s.to_string()),
            thread_id: format!("{:?}", std::thread::current().id()),
        };

        assert!(!snapshot.os.is_empty());
        assert!(!snapshot.arch.is_empty());
        assert!(snapshot.num_cpus > 0);
    }
}
