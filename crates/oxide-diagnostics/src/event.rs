//! Diagnostic Events
//!
//! Structured events that capture diagnostic information.

use crate::error::{ErrorCode, ErrorInfo, Severity, SourceLocation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A diagnostic event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    /// Unique event ID
    pub id: Uuid,

    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,

    /// Error code
    pub error_code: ErrorCode,

    /// Severity level
    pub severity: Severity,

    /// Human-readable message
    pub message: String,

    /// Category for grouping
    pub category: EventCategory,

    /// Component ID (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,

    /// Source location (if available)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,

    /// Extension ID (if involved)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_id: Option<String>,

    /// Additional context data
    #[serde(default)]
    pub context: std::collections::HashMap<String, serde_json::Value>,

    /// Reproduction hints
    #[serde(default)]
    pub hints: Vec<String>,

    /// Whether user has been notified
    #[serde(default)]
    pub user_notified: bool,
}

/// Event categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EventCategory {
    /// UI component related
    Ui,
    /// Layout engine related
    Layout,
    /// Rendering related
    Render,
    /// Extension related
    Extension,
    /// Network related
    Network,
    /// File system related
    FileSystem,
    /// Configuration related
    Config,
    /// Compiler related
    Compiler,
    /// Runtime related
    Runtime,
    /// System related
    System,
    /// Performance related
    Performance,
    /// Security related
    Security,
}

impl DiagnosticEvent {
    /// Create a new diagnostic event
    pub fn new(error_code: ErrorCode, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            error_code,
            severity,
            message: message.into(),
            category: Self::category_from_code(&error_code),
            component_id: None,
            source_location: None,
            extension_id: None,
            context: std::collections::HashMap::new(),
            hints: Vec::new(),
            user_notified: false,
        }
    }

    /// Create from ErrorInfo
    pub fn from_error_info(info: ErrorInfo) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            error_code: info.code,
            severity: info.severity,
            message: info.message,
            category: Self::category_from_code(&info.code),
            component_id: info.component_id,
            source_location: info.source_location,
            extension_id: info.extension_id,
            context: std::collections::HashMap::new(),
            hints: info.hints,
            user_notified: false,
        }
    }

    /// Derive category from error code domain
    fn category_from_code(code: &ErrorCode) -> EventCategory {
        use crate::error::ErrorDomain;
        match code.domain {
            ErrorDomain::Ui => EventCategory::Ui,
            ErrorDomain::Layout => EventCategory::Layout,
            ErrorDomain::Render => EventCategory::Render,
            ErrorDomain::Ext => EventCategory::Extension,
            ErrorDomain::Net => EventCategory::Network,
            ErrorDomain::Fs => EventCategory::FileSystem,
            ErrorDomain::Config => EventCategory::Config,
            ErrorDomain::Compiler => EventCategory::Compiler,
            ErrorDomain::Runtime => EventCategory::Runtime,
            ErrorDomain::System => EventCategory::System,
        }
    }

    /// Add context data
    pub fn with_context(mut self, key: &str, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.context.insert(key.to_string(), v);
        }
        self
    }

    /// Add component ID
    pub fn with_component(mut self, component_id: impl Into<String>) -> Self {
        self.component_id = Some(component_id.into());
        self
    }

    /// Add source location
    pub fn with_location(mut self, file: impl Into<String>, line: usize, column: usize) -> Self {
        self.source_location = Some(SourceLocation {
            file: file.into(),
            line,
            column,
        });
        self
    }

    /// Add extension ID
    pub fn with_extension(mut self, extension_id: impl Into<String>) -> Self {
        self.extension_id = Some(extension_id.into());
        self
    }

    /// Add hint
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    /// Mark as user notified
    pub fn mark_notified(mut self) -> Self {
        self.user_notified = true;
        self
    }

    /// Check if event should be included in diagnostics bundle
    pub fn include_in_bundle(&self) -> bool {
        self.severity.include_in_bundle()
    }

    /// Check if event is recent (within the last N hours)
    pub fn is_recent(&self, hours: i64) -> bool {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        self.timestamp > cutoff
    }
}

/// Event filter for querying events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by minimum severity
    pub min_severity: Option<Severity>,

    /// Filter by category
    pub category: Option<EventCategory>,

    /// Filter by component ID
    pub component_id: Option<String>,

    /// Filter by extension ID
    pub extension_id: Option<String>,

    /// Filter by time range (start)
    pub from: Option<DateTime<Utc>>,

    /// Filter by time range (end)
    pub to: Option<DateTime<Utc>>,

    /// Maximum number of results
    pub limit: Option<usize>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_severity(mut self, severity: Severity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    pub fn category(mut self, category: EventCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn component(mut self, component_id: impl Into<String>) -> Self {
        self.component_id = Some(component_id.into());
        self
    }

    pub fn extension(mut self, extension_id: impl Into<String>) -> Self {
        self.extension_id = Some(extension_id.into());
        self
    }

    pub fn from(mut self, from: DateTime<Utc>) -> Self {
        self.from = Some(from);
        self
    }

    pub fn to(mut self, to: DateTime<Utc>) -> Self {
        self.to = Some(to);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &DiagnosticEvent) -> bool {
        if let Some(min_sev) = self.min_severity {
            if event.severity < min_sev {
                return false;
            }
        }

        if let Some(cat) = self.category {
            if event.category != cat {
                return false;
            }
        }

        if let Some(ref comp) = self.component_id {
            if event.component_id.as_ref() != Some(comp) {
                return false;
            }
        }

        if let Some(ref ext) = self.extension_id {
            if event.extension_id.as_ref() != Some(ext) {
                return false;
            }
        }

        if let Some(from) = self.from {
            if event.timestamp < from {
                return false;
            }
        }

        if let Some(to) = self.to {
            if event.timestamp > to {
                return false;
            }
        }

        true
    }
}

/// Event statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventStats {
    pub total: usize,
    pub by_severity: std::collections::HashMap<String, usize>,
    pub by_category: std::collections::HashMap<String, usize>,
    pub by_error_code: std::collections::HashMap<String, usize>,
}

impl EventStats {
    /// Compute statistics from events
    pub fn from_events(events: &[DiagnosticEvent]) -> Self {
        let mut stats = Self {
            total: events.len(),
            ..Default::default()
        };

        for event in events {
            *stats.by_severity
                .entry(format!("{:?}", event.severity))
                .or_insert(0) += 1;

            *stats.by_category
                .entry(format!("{:?}", event.category))
                .or_insert(0) += 1;

            *stats.by_error_code
                .entry(event.error_code.to_string())
                .or_insert(0) += 1;
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = DiagnosticEvent::new(
            ErrorCode::UI_INVALID_PROP,
            Severity::Error,
            "Invalid property 'foo' on Button",
        )
        .with_component("ui.Button")
        .with_context("property", "foo");

        assert_eq!(event.category, EventCategory::Ui);
        assert!(event.component_id.is_some());
        assert!(event.context.contains_key("property"));
    }

    #[test]
    fn test_event_filter() {
        let event = DiagnosticEvent::new(
            ErrorCode::UI_INVALID_PROP,
            Severity::Warning,
            "Test warning",
        );

        let filter = EventFilter::new()
            .min_severity(Severity::Warning)
            .category(EventCategory::Ui);

        assert!(filter.matches(&event));

        let strict_filter = EventFilter::new()
            .min_severity(Severity::Error);

        assert!(!strict_filter.matches(&event));
    }

    #[test]
    fn test_event_stats() {
        let events = vec![
            DiagnosticEvent::new(ErrorCode::UI_INVALID_PROP, Severity::Warning, "a"),
            DiagnosticEvent::new(ErrorCode::UI_INVALID_PROP, Severity::Error, "b"),
            DiagnosticEvent::new(ErrorCode::LAYOUT_OVERFLOW, Severity::Warning, "c"),
        ];

        let stats = EventStats::from_events(&events);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.by_error_code.get("OXD-UI-0102"), Some(&2));
    }
}
