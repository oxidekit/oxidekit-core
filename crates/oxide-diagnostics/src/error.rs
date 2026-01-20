//! Error Model and Error Codes
//!
//! Structured error codes following the format: OXD-<DOMAIN>-<NUMBER>

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error domain categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum ErrorDomain {
    /// UI component errors (01xx)
    Ui,
    /// Layout engine errors (02xx)
    Layout,
    /// Rendering errors (03xx)
    Render,
    /// Extension errors (04xx)
    Ext,
    /// Network errors (05xx)
    Net,
    /// File system errors (06xx)
    Fs,
    /// Configuration errors (07xx)
    Config,
    /// Compiler errors (08xx)
    Compiler,
    /// Runtime errors (09xx)
    Runtime,
    /// System errors (10xx)
    System,
}

impl ErrorDomain {
    /// Get the domain prefix for error codes
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Ui => "UI",
            Self::Layout => "LAYOUT",
            Self::Render => "RENDER",
            Self::Ext => "EXT",
            Self::Net => "NET",
            Self::Fs => "FS",
            Self::Config => "CONFIG",
            Self::Compiler => "COMPILER",
            Self::Runtime => "RUNTIME",
            Self::System => "SYSTEM",
        }
    }

    /// Get the numeric range start for this domain
    pub fn range_start(&self) -> u16 {
        match self {
            Self::Ui => 100,
            Self::Layout => 200,
            Self::Render => 300,
            Self::Ext => 400,
            Self::Net => 500,
            Self::Fs => 600,
            Self::Config => 700,
            Self::Compiler => 800,
            Self::Runtime => 900,
            Self::System => 1000,
        }
    }
}

/// Structured error code
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ErrorCode {
    /// Error domain
    pub domain: ErrorDomain,
    /// Error number within domain (0-99)
    pub number: u16,
}

impl ErrorCode {
    /// Create a new error code
    pub fn new(domain: ErrorDomain, number: u16) -> Self {
        Self {
            domain,
            number: number % 100, // Ensure within range
        }
    }

    /// Get the full numeric code
    pub fn code(&self) -> u16 {
        self.domain.range_start() + self.number
    }

    /// Format as standard string (OXD-DOMAIN-NNNN)
    pub fn to_string_code(&self) -> String {
        format!("OXD-{}-{:04}", self.domain.prefix(), self.code())
    }

    // Common UI errors
    pub const UI_UNKNOWN_COMPONENT: Self = Self { domain: ErrorDomain::Ui, number: 1 };
    pub const UI_INVALID_PROP: Self = Self { domain: ErrorDomain::Ui, number: 2 };
    pub const UI_MISSING_REQUIRED_PROP: Self = Self { domain: ErrorDomain::Ui, number: 3 };
    pub const UI_INVALID_CHILD: Self = Self { domain: ErrorDomain::Ui, number: 4 };
    pub const UI_DEPRECATED_COMPONENT: Self = Self { domain: ErrorDomain::Ui, number: 5 };
    pub const UI_ACCESSIBILITY_VIOLATION: Self = Self { domain: ErrorDomain::Ui, number: 6 };

    // Common Layout errors
    pub const LAYOUT_OVERFLOW: Self = Self { domain: ErrorDomain::Layout, number: 1 };
    pub const LAYOUT_INVALID_SIZE: Self = Self { domain: ErrorDomain::Layout, number: 2 };
    pub const LAYOUT_CYCLE_DETECTED: Self = Self { domain: ErrorDomain::Layout, number: 3 };

    // Common Render errors
    pub const RENDER_GPU_ERROR: Self = Self { domain: ErrorDomain::Render, number: 1 };
    pub const RENDER_SHADER_COMPILE: Self = Self { domain: ErrorDomain::Render, number: 2 };
    pub const RENDER_OUT_OF_MEMORY: Self = Self { domain: ErrorDomain::Render, number: 3 };
    pub const RENDER_TEXTURE_ERROR: Self = Self { domain: ErrorDomain::Render, number: 4 };

    // Common Extension errors
    pub const EXT_NOT_FOUND: Self = Self { domain: ErrorDomain::Ext, number: 1 };
    pub const EXT_VERSION_MISMATCH: Self = Self { domain: ErrorDomain::Ext, number: 2 };
    pub const EXT_PERMISSION_DENIED: Self = Self { domain: ErrorDomain::Ext, number: 3 };
    pub const EXT_LOAD_FAILED: Self = Self { domain: ErrorDomain::Ext, number: 4 };

    // Common Network errors
    pub const NET_CONNECTION_FAILED: Self = Self { domain: ErrorDomain::Net, number: 1 };
    pub const NET_TIMEOUT: Self = Self { domain: ErrorDomain::Net, number: 2 };
    pub const NET_TLS_ERROR: Self = Self { domain: ErrorDomain::Net, number: 3 };
    pub const NET_INVALID_RESPONSE: Self = Self { domain: ErrorDomain::Net, number: 4 };

    // Common File system errors
    pub const FS_NOT_FOUND: Self = Self { domain: ErrorDomain::Fs, number: 1 };
    pub const FS_PERMISSION_DENIED: Self = Self { domain: ErrorDomain::Fs, number: 2 };
    pub const FS_READ_ERROR: Self = Self { domain: ErrorDomain::Fs, number: 3 };
    pub const FS_WRITE_ERROR: Self = Self { domain: ErrorDomain::Fs, number: 4 };

    // Common Config errors
    pub const CONFIG_INVALID_FORMAT: Self = Self { domain: ErrorDomain::Config, number: 1 };
    pub const CONFIG_MISSING_FIELD: Self = Self { domain: ErrorDomain::Config, number: 2 };
    pub const CONFIG_INVALID_VALUE: Self = Self { domain: ErrorDomain::Config, number: 3 };

    // Common Compiler errors
    pub const COMPILER_SYNTAX_ERROR: Self = Self { domain: ErrorDomain::Compiler, number: 1 };
    pub const COMPILER_PARSE_ERROR: Self = Self { domain: ErrorDomain::Compiler, number: 2 };
    pub const COMPILER_VALIDATION_ERROR: Self = Self { domain: ErrorDomain::Compiler, number: 3 };

    // Common Runtime errors
    pub const RUNTIME_PANIC: Self = Self { domain: ErrorDomain::Runtime, number: 1 };
    pub const RUNTIME_INITIALIZATION: Self = Self { domain: ErrorDomain::Runtime, number: 2 };
    pub const RUNTIME_STATE_ERROR: Self = Self { domain: ErrorDomain::Runtime, number: 3 };

    // Common System errors
    pub const SYSTEM_OUT_OF_MEMORY: Self = Self { domain: ErrorDomain::System, number: 1 };
    pub const SYSTEM_THREAD_PANIC: Self = Self { domain: ErrorDomain::System, number: 2 };
    pub const SYSTEM_RESOURCE_EXHAUSTED: Self = Self { domain: ErrorDomain::System, number: 3 };
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_code())
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational, not an error
    Info,
    /// Warning, operation completed but with issues
    Warning,
    /// Error, operation failed but app can continue
    Error,
    /// Fatal, app must terminate
    Fatal,
}

impl Severity {
    /// Check if this severity should trigger an alert
    pub fn is_alertable(&self) -> bool {
        matches!(self, Severity::Error | Severity::Fatal)
    }

    /// Check if this severity should be included in diagnostics bundle
    pub fn include_in_bundle(&self) -> bool {
        // Include warnings and above
        *self >= Severity::Warning
    }
}

/// Error metadata for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: ErrorCode,

    /// Severity
    pub severity: Severity,

    /// Human-readable message
    pub message: String,

    /// Component ID (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,

    /// Source location
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,

    /// Extension ID (if involved)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_id: Option<String>,

    /// Reproduction hints
    #[serde(default)]
    pub hints: Vec<String>,

    /// Related error codes
    #[serde(default)]
    pub related_codes: Vec<ErrorCode>,
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path (may be redacted)
    pub file: String,

    /// Line number
    pub line: usize,

    /// Column number
    pub column: usize,
}

impl ErrorInfo {
    /// Create a new error info
    pub fn new(code: ErrorCode, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            component_id: None,
            source_location: None,
            extension_id: None,
            hints: Vec::new(),
            related_codes: Vec::new(),
        }
    }

    pub fn with_component(mut self, component_id: impl Into<String>) -> Self {
        self.component_id = Some(component_id.into());
        self
    }

    pub fn with_location(mut self, file: impl Into<String>, line: usize, column: usize) -> Self {
        self.source_location = Some(SourceLocation {
            file: file.into(),
            line,
            column,
        });
        self
    }

    pub fn with_extension(mut self, extension_id: impl Into<String>) -> Self {
        self.extension_id = Some(extension_id.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    pub fn with_related(mut self, code: ErrorCode) -> Self {
        self.related_codes.push(code);
        self
    }
}

/// Error registry for looking up error information
pub struct ErrorRegistry {
    entries: std::collections::HashMap<ErrorCode, ErrorDescription>,
}

/// Description of an error code
#[derive(Debug, Clone)]
pub struct ErrorDescription {
    pub code: ErrorCode,
    pub title: String,
    pub description: String,
    pub documentation_url: Option<String>,
    pub fixes: Vec<String>,
}

impl ErrorRegistry {
    /// Create a new error registry with built-in errors
    pub fn new() -> Self {
        let mut entries = std::collections::HashMap::new();

        // Register UI errors
        entries.insert(ErrorCode::UI_UNKNOWN_COMPONENT, ErrorDescription {
            code: ErrorCode::UI_UNKNOWN_COMPONENT,
            title: "Unknown Component".into(),
            description: "The specified component was not found in the registry".into(),
            documentation_url: Some("https://oxidekit.com/errors/ui-0101".into()),
            fixes: vec![
                "Check for typos in the component name".into(),
                "Ensure the component pack is installed".into(),
            ],
        });

        entries.insert(ErrorCode::UI_INVALID_PROP, ErrorDescription {
            code: ErrorCode::UI_INVALID_PROP,
            title: "Invalid Property".into(),
            description: "The property is not valid for this component".into(),
            documentation_url: Some("https://oxidekit.com/errors/ui-0102".into()),
            fixes: vec![
                "Check the component documentation for valid properties".into(),
                "Use 'oxide export ai-schema' to see all component specs".into(),
            ],
        });

        entries.insert(ErrorCode::UI_MISSING_REQUIRED_PROP, ErrorDescription {
            code: ErrorCode::UI_MISSING_REQUIRED_PROP,
            title: "Missing Required Property".into(),
            description: "A required property was not provided".into(),
            documentation_url: Some("https://oxidekit.com/errors/ui-0103".into()),
            fixes: vec![
                "Add the missing required property".into(),
            ],
        });

        // Add more error descriptions...

        Self { entries }
    }

    /// Look up error description
    pub fn get(&self, code: ErrorCode) -> Option<&ErrorDescription> {
        self.entries.get(&code)
    }
}

impl Default for ErrorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_format() {
        let code = ErrorCode::UI_INVALID_PROP;
        assert_eq!(code.to_string_code(), "OXD-UI-0102");
    }

    #[test]
    fn test_error_code_display() {
        let code = ErrorCode::new(ErrorDomain::Layout, 1);
        assert_eq!(format!("{}", code), "OXD-LAYOUT-0201");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
    }

    #[test]
    fn test_error_info_builder() {
        let info = ErrorInfo::new(
            ErrorCode::UI_UNKNOWN_COMPONENT,
            Severity::Error,
            "Component 'Foo' not found",
        )
        .with_component("ui.Foo")
        .with_hint("Did you mean 'ui.Form'?");

        assert_eq!(info.code, ErrorCode::UI_UNKNOWN_COMPONENT);
        assert_eq!(info.component_id, Some("ui.Foo".into()));
        assert_eq!(info.hints.len(), 1);
    }

    #[test]
    fn test_error_registry() {
        let registry = ErrorRegistry::new();
        let desc = registry.get(ErrorCode::UI_UNKNOWN_COMPONENT);

        assert!(desc.is_some());
        assert_eq!(desc.unwrap().title, "Unknown Component");
    }
}
