//! Key naming conventions for teams
//!
//! Enforces consistent translation key naming across the team.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A convention violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConventionViolation {
    /// The offending key
    pub key: String,
    /// The rule that was violated
    pub rule: String,
    /// Suggestion for fixing
    pub suggestion: Option<String>,
    /// Severity level
    pub severity: ViolationSeverity,
}

/// Severity of a convention violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    /// Warning only
    Warning,
    /// Error - should be fixed
    Error,
}

/// Key naming convention rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyNamingConvention {
    /// Naming style for keys
    #[serde(default)]
    pub style: KeyStyle,
    /// Minimum depth (number of segments)
    #[serde(default)]
    pub min_depth: usize,
    /// Maximum depth
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Required prefixes (namespaces)
    #[serde(default)]
    pub required_prefixes: HashSet<String>,
    /// Forbidden prefixes
    #[serde(default)]
    pub forbidden_prefixes: HashSet<String>,
    /// Reserved words that can't be used
    #[serde(default)]
    pub reserved_words: HashSet<String>,
    /// Maximum segment length
    #[serde(default)]
    pub max_segment_length: Option<usize>,
    /// Allowed characters pattern
    #[serde(default)]
    pub allowed_chars: Option<String>,
    /// Separator character
    #[serde(default = "default_separator")]
    pub separator: char,
}

fn default_separator() -> char {
    '.'
}

/// Style for key naming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyStyle {
    /// snake_case
    #[default]
    SnakeCase,
    /// camelCase
    CamelCase,
    /// kebab-case
    KebabCase,
    /// Any style allowed
    Any,
}

impl KeyStyle {
    /// Check if a segment matches this style
    pub fn matches(&self, segment: &str) -> bool {
        match self {
            KeyStyle::SnakeCase => is_snake_case(segment),
            KeyStyle::CamelCase => is_camel_case(segment),
            KeyStyle::KebabCase => is_kebab_case(segment),
            KeyStyle::Any => true,
        }
    }

    /// Get the style name
    pub fn name(&self) -> &'static str {
        match self {
            KeyStyle::SnakeCase => "snake_case",
            KeyStyle::CamelCase => "camelCase",
            KeyStyle::KebabCase => "kebab-case",
            KeyStyle::Any => "any",
        }
    }

    /// Convert a segment to this style
    pub fn convert(&self, segment: &str) -> String {
        match self {
            KeyStyle::SnakeCase => to_snake_case(segment),
            KeyStyle::CamelCase => to_camel_case(segment),
            KeyStyle::KebabCase => to_kebab_case(segment),
            KeyStyle::Any => segment.to_string(),
        }
    }
}

impl Default for KeyNamingConvention {
    fn default() -> Self {
        Self {
            style: KeyStyle::SnakeCase,
            min_depth: 2,
            max_depth: Some(5),
            required_prefixes: HashSet::new(),
            forbidden_prefixes: HashSet::new(),
            reserved_words: HashSet::new(),
            max_segment_length: Some(30),
            allowed_chars: Some(r"^[a-z][a-z0-9_]*$".to_string()),
            separator: '.',
        }
    }
}

impl KeyNamingConvention {
    /// Create a new convention with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the key style
    pub fn with_style(mut self, style: KeyStyle) -> Self {
        self.style = style;
        self
    }

    /// Set minimum depth
    pub fn with_min_depth(mut self, depth: usize) -> Self {
        self.min_depth = depth;
        self
    }

    /// Set maximum depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Add required prefix
    pub fn with_required_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.required_prefixes.insert(prefix.into());
        self
    }

    /// Add forbidden prefix
    pub fn with_forbidden_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.forbidden_prefixes.insert(prefix.into());
        self
    }

    /// Add reserved word
    pub fn with_reserved_word(mut self, word: impl Into<String>) -> Self {
        self.reserved_words.insert(word.into());
        self
    }

    /// Preset: Standard namespace convention
    pub fn standard() -> Self {
        Self::default()
            .with_required_prefix("common")
            .with_required_prefix("auth")
            .with_required_prefix("errors")
            .with_required_prefix("validation")
            .with_required_prefix("navigation")
            .with_required_prefix("actions")
            .with_reserved_word("test")
            .with_reserved_word("debug")
            .with_reserved_word("tmp")
    }
}

/// Checker for key naming conventions
pub struct ConventionChecker {
    convention: KeyNamingConvention,
    regex_pattern: Option<regex::Regex>,
}

impl ConventionChecker {
    /// Create a new checker
    pub fn new(convention: KeyNamingConvention) -> Self {
        let regex_pattern = convention
            .allowed_chars
            .as_ref()
            .and_then(|p| regex::Regex::new(p).ok());

        Self {
            convention,
            regex_pattern,
        }
    }

    /// Check a single key
    pub fn check_key(&self, key: &str) -> Vec<ConventionViolation> {
        let mut violations = Vec::new();
        let segments: Vec<&str> = key.split(self.convention.separator).collect();

        // Check depth
        if segments.len() < self.convention.min_depth {
            violations.push(ConventionViolation {
                key: key.to_string(),
                rule: format!(
                    "Key must have at least {} segments (has {})",
                    self.convention.min_depth,
                    segments.len()
                ),
                suggestion: Some(format!(
                    "Add more namespace segments, e.g., 'namespace.{}'",
                    key
                )),
                severity: ViolationSeverity::Error,
            });
        }

        if let Some(max_depth) = self.convention.max_depth {
            if segments.len() > max_depth {
                violations.push(ConventionViolation {
                    key: key.to_string(),
                    rule: format!(
                        "Key exceeds maximum depth of {} (has {})",
                        max_depth,
                        segments.len()
                    ),
                    suggestion: Some("Consider consolidating namespace segments".to_string()),
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        // Check required prefixes
        if !self.convention.required_prefixes.is_empty() {
            let first_segment = segments.first().copied().unwrap_or("");
            if !self.convention.required_prefixes.contains(first_segment) {
                violations.push(ConventionViolation {
                    key: key.to_string(),
                    rule: format!(
                        "Key must start with one of: {:?}",
                        self.convention.required_prefixes
                    ),
                    suggestion: None,
                    severity: ViolationSeverity::Error,
                });
            }
        }

        // Check forbidden prefixes
        for prefix in &self.convention.forbidden_prefixes {
            if key.starts_with(prefix) {
                violations.push(ConventionViolation {
                    key: key.to_string(),
                    rule: format!("Prefix '{}' is forbidden", prefix),
                    suggestion: None,
                    severity: ViolationSeverity::Error,
                });
            }
        }

        // Check reserved words
        for segment in &segments {
            if self.convention.reserved_words.contains(*segment) {
                violations.push(ConventionViolation {
                    key: key.to_string(),
                    rule: format!("'{}' is a reserved word", segment),
                    suggestion: None,
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        // Check segment style
        for segment in &segments {
            if !self.convention.style.matches(segment) {
                let suggested = self.convention.style.convert(segment);
                violations.push(ConventionViolation {
                    key: key.to_string(),
                    rule: format!(
                        "Segment '{}' doesn't match {} style",
                        segment,
                        self.convention.style.name()
                    ),
                    suggestion: Some(format!("Use '{}' instead", suggested)),
                    severity: ViolationSeverity::Warning,
                });
            }

            // Check segment length
            if let Some(max_len) = self.convention.max_segment_length {
                if segment.len() > max_len {
                    violations.push(ConventionViolation {
                        key: key.to_string(),
                        rule: format!(
                            "Segment '{}' exceeds maximum length of {}",
                            segment, max_len
                        ),
                        suggestion: Some("Use a shorter, more concise name".to_string()),
                        severity: ViolationSeverity::Warning,
                    });
                }
            }

            // Check allowed characters
            if let Some(ref regex) = self.regex_pattern {
                if !regex.is_match(segment) {
                    violations.push(ConventionViolation {
                        key: key.to_string(),
                        rule: format!("Segment '{}' contains invalid characters", segment),
                        suggestion: Some("Use only lowercase letters, numbers, and underscores".to_string()),
                        severity: ViolationSeverity::Error,
                    });
                }
            }
        }

        violations
    }

    /// Check multiple keys
    pub fn check_keys(&self, keys: &[&str]) -> Vec<ConventionViolation> {
        keys.iter().flat_map(|k| self.check_key(k)).collect()
    }

    /// Check if a key is valid (has no errors)
    pub fn is_valid(&self, key: &str) -> bool {
        self.check_key(key)
            .iter()
            .all(|v| v.severity != ViolationSeverity::Error)
    }

    /// Suggest a corrected key
    pub fn suggest_correction(&self, key: &str) -> String {
        let segments: Vec<&str> = key.split(self.convention.separator).collect();
        let corrected: Vec<String> = segments
            .iter()
            .map(|s| self.convention.style.convert(s))
            .collect();
        corrected.join(&self.convention.separator.to_string())
    }
}

// Helper functions for style detection and conversion

fn is_snake_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().map_or(false, |c| c.is_lowercase())
        && s.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
        && !s.contains("__")
}

fn is_camel_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().map_or(false, |c| c.is_lowercase())
        && s.chars().all(|c| c.is_alphanumeric())
}

fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().map_or(false, |c| c.is_lowercase())
        && s.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '-')
        && !s.contains("--")
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result.replace('-', "_").replace("__", "_")
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
    }
    result
}

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result.replace('_', "-").replace("--", "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_detection() {
        assert!(is_snake_case("hello_world"));
        assert!(is_snake_case("hello"));
        assert!(!is_snake_case("HelloWorld"));
        assert!(!is_snake_case("hello-world"));

        assert!(is_camel_case("helloWorld"));
        assert!(is_camel_case("hello"));
        assert!(!is_camel_case("hello_world"));

        assert!(is_kebab_case("hello-world"));
        assert!(is_kebab_case("hello"));
        assert!(!is_kebab_case("hello_world"));
    }

    #[test]
    fn test_style_conversion() {
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
    }

    #[test]
    fn test_convention_checker() {
        let convention = KeyNamingConvention::default()
            .with_min_depth(2)
            .with_max_depth(4)
            .with_style(KeyStyle::SnakeCase);

        let checker = ConventionChecker::new(convention);

        // Valid key
        assert!(checker.is_valid("common.greeting"));

        // Too shallow
        let violations = checker.check_key("greeting");
        assert!(!violations.is_empty());

        // Invalid style
        let violations = checker.check_key("common.helloWorld");
        assert!(violations.iter().any(|v| v.rule.contains("style")));
    }

    #[test]
    fn test_required_prefixes() {
        let convention = KeyNamingConvention::default()
            .with_required_prefix("common")
            .with_required_prefix("auth");

        let checker = ConventionChecker::new(convention);

        assert!(checker.is_valid("common.button"));
        assert!(checker.is_valid("auth.login"));

        let violations = checker.check_key("unknown.key");
        assert!(violations.iter().any(|v| v.rule.contains("must start with")));
    }
}
