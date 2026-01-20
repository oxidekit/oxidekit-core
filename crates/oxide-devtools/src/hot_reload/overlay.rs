//! Error overlay for displaying compilation errors during development
//!
//! Renders a visual overlay showing compilation errors, warnings, and diagnostics.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

use super::events::{CompileErrorInfo, ErrorSeverity};

/// Configuration for the error overlay
#[derive(Debug, Clone)]
pub struct OverlayConfig {
    /// Whether the overlay is enabled
    pub enabled: bool,
    /// Maximum number of errors to display
    pub max_errors: usize,
    /// Whether to show warnings
    pub show_warnings: bool,
    /// Auto-dismiss after successful compilation (in seconds, 0 = never)
    pub auto_dismiss_seconds: u64,
    /// Overlay position
    pub position: OverlayPosition,
    /// Overlay theme
    pub theme: OverlayTheme,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_errors: 10,
            show_warnings: true,
            auto_dismiss_seconds: 3,
            position: OverlayPosition::BottomRight,
            theme: OverlayTheme::default(),
        }
    }
}

/// Position of the error overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    FullScreen,
}

/// Theme colors for the error overlay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayTheme {
    /// Background color (RGBA hex)
    pub background: String,
    /// Text color
    pub text: String,
    /// Error color
    pub error: String,
    /// Warning color
    pub warning: String,
    /// Info color
    pub info: String,
    /// Border color
    pub border: String,
    /// Header background
    pub header_background: String,
    /// Code background
    pub code_background: String,
}

impl Default for OverlayTheme {
    fn default() -> Self {
        Self {
            background: "#1F1F1FE6".to_string(),      // Semi-transparent dark gray
            text: "#E5E7EB".to_string(),               // Light gray
            error: "#EF4444".to_string(),              // Red
            warning: "#F59E0B".to_string(),            // Amber
            info: "#3B82F6".to_string(),               // Blue
            border: "#374151".to_string(),             // Dark gray
            header_background: "#111827".to_string(),  // Darker gray
            code_background: "#0D1117".to_string(),    // Near black
        }
    }
}

/// A diagnostic to display in the overlay
#[derive(Debug, Clone)]
pub struct DiagnosticDisplay {
    /// File path where the error occurred
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Error message
    pub message: String,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Error code (if available)
    pub code: Option<String>,
    /// Source code snippet
    pub source_snippet: Option<SourceSnippet>,
    /// When this diagnostic was created
    pub timestamp: Instant,
}

/// Serializable version of DiagnosticDisplay (without timestamp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticDisplayData {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: ErrorSeverity,
    pub code: Option<String>,
    pub source_snippet: Option<SourceSnippet>,
}

impl From<&DiagnosticDisplay> for DiagnosticDisplayData {
    fn from(d: &DiagnosticDisplay) -> Self {
        Self {
            file: d.file.clone(),
            line: d.line,
            column: d.column,
            message: d.message.clone(),
            severity: d.severity,
            code: d.code.clone(),
            source_snippet: d.source_snippet.clone(),
        }
    }
}

impl DiagnosticDisplay {
    /// Create a new diagnostic from compilation error info
    pub fn from_compile_error(file: PathBuf, error: CompileErrorInfo) -> Self {
        Self {
            file,
            line: error.line,
            column: error.column,
            message: error.message,
            severity: error.severity,
            code: error.code,
            source_snippet: None,
            timestamp: Instant::now(),
        }
    }

    /// Add a source snippet to this diagnostic
    pub fn with_snippet(mut self, snippet: SourceSnippet) -> Self {
        self.source_snippet = Some(snippet);
        self
    }

    /// Get the severity color from the theme
    pub fn severity_color<'a>(&self, theme: &'a OverlayTheme) -> &'a str {
        match self.severity {
            ErrorSeverity::Error => &theme.error,
            ErrorSeverity::Warning => &theme.warning,
            ErrorSeverity::Info | ErrorSeverity::Hint => &theme.info,
        }
    }
}

/// Source code snippet for error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSnippet {
    /// Lines of source code
    pub lines: Vec<SourceLine>,
    /// The highlighted line (index into lines array)
    pub highlight_index: usize,
    /// Column range to highlight
    pub highlight_columns: Option<(usize, usize)>,
}

/// A line of source code in a snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLine {
    /// Line number (1-indexed)
    pub number: usize,
    /// Line content
    pub content: String,
    /// Whether this is the error line
    pub is_error_line: bool,
}

impl SourceSnippet {
    /// Create a source snippet from file content
    pub fn from_source(
        source: &str,
        error_line: usize,
        error_column: usize,
        context_lines: usize,
    ) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let error_index = error_line.saturating_sub(1);

        let start = error_index.saturating_sub(context_lines);
        let end = (error_index + context_lines + 1).min(lines.len());

        let snippet_lines: Vec<SourceLine> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, content)| {
                let line_number = start + i + 1;
                SourceLine {
                    number: line_number,
                    content: content.to_string(),
                    is_error_line: line_number == error_line,
                }
            })
            .collect();

        let highlight_index = error_index.saturating_sub(start);

        Self {
            lines: snippet_lines,
            highlight_index,
            highlight_columns: Some((error_column, error_column + 1)),
        }
    }
}

/// State of the error overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayState {
    /// Overlay is hidden
    Hidden,
    /// Overlay is visible with errors
    Visible,
    /// Overlay is fading out after errors were fixed
    Dismissing,
}

/// Error overlay manager
pub struct ErrorOverlay {
    config: OverlayConfig,
    /// Current diagnostics to display
    diagnostics: Arc<RwLock<VecDeque<DiagnosticDisplay>>>,
    /// Current overlay state
    state: Arc<RwLock<OverlayState>>,
    /// When the overlay should auto-dismiss
    dismiss_at: Arc<RwLock<Option<Instant>>>,
    /// Currently selected diagnostic index
    selected_index: Arc<RwLock<usize>>,
}

impl ErrorOverlay {
    /// Create a new error overlay with the given configuration
    pub fn new(config: OverlayConfig) -> Self {
        Self {
            config,
            diagnostics: Arc::new(RwLock::new(VecDeque::new())),
            state: Arc::new(RwLock::new(OverlayState::Hidden)),
            dismiss_at: Arc::new(RwLock::new(None)),
            selected_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new error overlay with default configuration
    pub fn with_defaults() -> Self {
        Self::new(OverlayConfig::default())
    }

    /// Show the overlay with the given diagnostics
    pub fn show(&self, diagnostics: Vec<DiagnosticDisplay>) {
        if !self.config.enabled || diagnostics.is_empty() {
            return;
        }

        let mut diag_guard = self.diagnostics.write();
        diag_guard.clear();

        // Filter by severity and limit count
        let filtered: Vec<_> = diagnostics
            .into_iter()
            .filter(|d| {
                self.config.show_warnings || matches!(d.severity, ErrorSeverity::Error)
            })
            .take(self.config.max_errors)
            .collect();

        for d in filtered {
            diag_guard.push_back(d);
        }

        if !diag_guard.is_empty() {
            *self.state.write() = OverlayState::Visible;
            *self.dismiss_at.write() = None;
            *self.selected_index.write() = 0;

            tracing::debug!("Showing error overlay with {} diagnostics", diag_guard.len());
        }
    }

    /// Hide the overlay immediately
    pub fn hide(&self) {
        *self.state.write() = OverlayState::Hidden;
        *self.dismiss_at.write() = None;
        self.diagnostics.write().clear();
        tracing::debug!("Hiding error overlay");
    }

    /// Start the dismiss animation
    pub fn dismiss(&self) {
        if *self.state.read() == OverlayState::Visible {
            if self.config.auto_dismiss_seconds > 0 {
                *self.state.write() = OverlayState::Dismissing;
                *self.dismiss_at.write() = Some(
                    Instant::now() + Duration::from_secs(self.config.auto_dismiss_seconds),
                );
            } else {
                self.hide();
            }
        }
    }

    /// Update the overlay state (call periodically)
    pub fn update(&self) {
        let dismiss_at = *self.dismiss_at.read();
        if let Some(dismiss_time) = dismiss_at {
            if Instant::now() >= dismiss_time {
                self.hide();
            }
        }
    }

    /// Get the current overlay state
    pub fn state(&self) -> OverlayState {
        *self.state.read()
    }

    /// Get the current diagnostics
    pub fn diagnostics(&self) -> Vec<DiagnosticDisplay> {
        self.diagnostics.read().iter().cloned().collect()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .read()
            .iter()
            .filter(|d| matches!(d.severity, ErrorSeverity::Error))
            .count()
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .read()
            .iter()
            .filter(|d| matches!(d.severity, ErrorSeverity::Warning))
            .count()
    }

    /// Navigate to the next diagnostic
    pub fn next(&self) {
        let mut index = self.selected_index.write();
        let count = self.diagnostics.read().len();
        if count > 0 {
            *index = (*index + 1) % count;
        }
    }

    /// Navigate to the previous diagnostic
    pub fn previous(&self) {
        let mut index = self.selected_index.write();
        let count = self.diagnostics.read().len();
        if count > 0 {
            *index = if *index == 0 { count - 1 } else { *index - 1 };
        }
    }

    /// Get the currently selected diagnostic
    pub fn selected(&self) -> Option<DiagnosticDisplay> {
        let index = *self.selected_index.read();
        self.diagnostics.read().get(index).cloned()
    }

    /// Get the selected index
    pub fn selected_index(&self) -> usize {
        *self.selected_index.read()
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        matches!(*self.state.read(), OverlayState::Visible | OverlayState::Dismissing)
    }

    /// Generate render data for the overlay
    pub fn render_data(&self) -> Option<OverlayRenderData> {
        if !self.is_visible() {
            return None;
        }

        let diagnostics = self.diagnostics();
        if diagnostics.is_empty() {
            return None;
        }

        let selected = *self.selected_index.read();
        let state = *self.state.read();

        // Calculate opacity for dismiss animation
        let opacity = if state == OverlayState::Dismissing {
            if let Some(dismiss_at) = *self.dismiss_at.read() {
                let total = Duration::from_secs(self.config.auto_dismiss_seconds);
                let remaining = dismiss_at.saturating_duration_since(Instant::now());
                (remaining.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0)
            } else {
                1.0
            }
        } else {
            1.0
        };

        Some(OverlayRenderData {
            diagnostics,
            selected_index: selected,
            error_count: self.error_count(),
            warning_count: self.warning_count(),
            position: self.config.position,
            theme: self.config.theme.clone(),
            opacity,
        })
    }
}

impl Default for ErrorOverlay {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Data needed to render the error overlay
#[derive(Debug, Clone)]
pub struct OverlayRenderData {
    /// Diagnostics to display
    pub diagnostics: Vec<DiagnosticDisplay>,
    /// Currently selected diagnostic index
    pub selected_index: usize,
    /// Number of errors
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
    /// Overlay position
    pub position: OverlayPosition,
    /// Theme colors
    pub theme: OverlayTheme,
    /// Opacity (for dismiss animation)
    pub opacity: f32,
}

/// Serializable version of OverlayRenderData
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayRenderDataSerializable {
    pub diagnostics: Vec<DiagnosticDisplayData>,
    pub selected_index: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub position: OverlayPosition,
    pub theme: OverlayTheme,
    pub opacity: f32,
}

impl From<&OverlayRenderData> for OverlayRenderDataSerializable {
    fn from(data: &OverlayRenderData) -> Self {
        Self {
            diagnostics: data.diagnostics.iter().map(|d| d.into()).collect(),
            selected_index: data.selected_index,
            error_count: data.error_count,
            warning_count: data.warning_count,
            position: data.position,
            theme: data.theme.clone(),
            opacity: data.opacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_overlay_show_hide() {
        let overlay = ErrorOverlay::with_defaults();

        assert!(!overlay.is_visible());

        let diagnostic = DiagnosticDisplay {
            file: PathBuf::from("test.oui"),
            line: 1,
            column: 1,
            message: "Test error".to_string(),
            severity: ErrorSeverity::Error,
            code: None,
            source_snippet: None,
            timestamp: Instant::now(),
        };

        overlay.show(vec![diagnostic]);
        assert!(overlay.is_visible());
        assert_eq!(overlay.error_count(), 1);

        overlay.hide();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_source_snippet() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5";
        let snippet = SourceSnippet::from_source(source, 3, 5, 1);

        assert_eq!(snippet.lines.len(), 3);
        assert_eq!(snippet.lines[1].number, 3);
        assert!(snippet.lines[1].is_error_line);
    }

    #[test]
    fn test_diagnostic_navigation() {
        let overlay = ErrorOverlay::with_defaults();

        let diagnostics = vec![
            DiagnosticDisplay {
                file: PathBuf::from("a.oui"),
                line: 1,
                column: 1,
                message: "Error A".to_string(),
                severity: ErrorSeverity::Error,
                code: None,
                source_snippet: None,
                timestamp: Instant::now(),
            },
            DiagnosticDisplay {
                file: PathBuf::from("b.oui"),
                line: 2,
                column: 1,
                message: "Error B".to_string(),
                severity: ErrorSeverity::Error,
                code: None,
                source_snippet: None,
                timestamp: Instant::now(),
            },
        ];

        overlay.show(diagnostics);

        assert_eq!(overlay.selected_index(), 0);

        overlay.next();
        assert_eq!(overlay.selected_index(), 1);

        overlay.next();
        assert_eq!(overlay.selected_index(), 0); // Wraps around

        overlay.previous();
        assert_eq!(overlay.selected_index(), 1);
    }
}
