//! Privacy-Safe Redaction
//!
//! Rules and utilities for redacting sensitive data from diagnostics.

use crate::LogEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Redaction rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRules {
    /// Patterns to redact (regex)
    #[serde(default)]
    pub patterns: Vec<RedactionPattern>,

    /// Field names to always redact
    #[serde(default)]
    pub redact_fields: HashSet<String>,

    /// Whether to redact file paths
    #[serde(default = "default_true")]
    pub redact_paths: bool,

    /// Whether to redact IP addresses
    #[serde(default = "default_true")]
    pub redact_ips: bool,

    /// Whether to redact email addresses
    #[serde(default = "default_true")]
    pub redact_emails: bool,

    /// Custom redaction placeholder
    #[serde(default = "default_placeholder")]
    pub placeholder: String,
}

fn default_true() -> bool {
    true
}

fn default_placeholder() -> String {
    "[REDACTED]".to_string()
}

impl Default for RedactionRules {
    fn default() -> Self {
        Self {
            patterns: vec![
                // API keys and tokens
                RedactionPattern::new("api[_-]?key", "[API_KEY]"),
                RedactionPattern::new("token", "[TOKEN]"),
                RedactionPattern::new("secret", "[SECRET]"),
                RedactionPattern::new("password", "[PASSWORD]"),
                RedactionPattern::new("auth", "[AUTH]"),
                RedactionPattern::new("bearer\\s+[a-zA-Z0-9._-]+", "[BEARER_TOKEN]"),

                // Credit card patterns (simplified)
                RedactionPattern::new(r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b", "[CARD_NUMBER]"),

                // SSN patterns (US)
                RedactionPattern::new(r"\b\d{3}[- ]?\d{2}[- ]?\d{4}\b", "[SSN]"),
            ],
            redact_fields: [
                "password",
                "secret",
                "token",
                "api_key",
                "apiKey",
                "auth",
                "authorization",
                "cookie",
                "session",
                "credential",
                "private_key",
                "privateKey",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            redact_paths: true,
            redact_ips: true,
            redact_emails: true,
            placeholder: default_placeholder(),
        }
    }
}

/// A redaction pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionPattern {
    /// Pattern name/description
    pub name: String,

    /// Regex pattern (case-insensitive)
    pub pattern: String,

    /// Replacement text
    pub replacement: String,
}

impl RedactionPattern {
    pub fn new(pattern: &str, replacement: &str) -> Self {
        Self {
            name: pattern.to_string(),
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
        }
    }
}

/// Redact a string using the rules
pub fn redact_string(input: &str, rules: &RedactionRules) -> String {
    let mut result = input.to_string();

    // Apply pattern-based redactions
    for pattern in &rules.patterns {
        if let Ok(re) = regex_lite::Regex::new(&format!("(?i){}", pattern.pattern)) {
            result = re.replace_all(&result, pattern.replacement.as_str()).to_string();
        }
    }

    // Redact IP addresses
    if rules.redact_ips {
        // IPv4
        if let Ok(re) = regex_lite::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b") {
            result = re.replace_all(&result, "[IP_ADDRESS]").to_string();
        }
        // IPv6 (simplified)
        if let Ok(re) = regex_lite::Regex::new(r"\b[0-9a-fA-F:]{7,39}\b") {
            result = re.replace_all(&result, "[IP_ADDRESS]").to_string();
        }
    }

    // Redact email addresses
    if rules.redact_emails {
        if let Ok(re) = regex_lite::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b") {
            result = re.replace_all(&result, "[EMAIL]").to_string();
        }
    }

    // Redact file paths
    if rules.redact_paths {
        // Unix paths with username
        if let Ok(re) = regex_lite::Regex::new(r"/Users/[^/\s]+") {
            result = re.replace_all(&result, "/Users/[USER]").to_string();
        }
        if let Ok(re) = regex_lite::Regex::new(r"/home/[^/\s]+") {
            result = re.replace_all(&result, "/home/[USER]").to_string();
        }
        // Windows paths with username
        if let Ok(re) = regex_lite::Regex::new(r"C:\\Users\\[^\\]+") {
            result = re.replace_all(&result, r"C:\Users\[USER]").to_string();
        }
    }

    result
}

/// Redact a log entry
pub fn redact_log_entry(mut entry: LogEntry, rules: &RedactionRules) -> LogEntry {
    // Redact message
    entry.message = redact_string(&entry.message, rules);

    // Redact fields
    let mut redacted_fields = std::collections::HashMap::new();
    for (key, value) in entry.fields {
        if rules.redact_fields.contains(&key.to_lowercase()) {
            redacted_fields.insert(key, serde_json::json!(rules.placeholder));
        } else if let serde_json::Value::String(s) = &value {
            redacted_fields.insert(key, serde_json::json!(redact_string(s, rules)));
        } else {
            redacted_fields.insert(key, value);
        }
    }
    entry.fields = redacted_fields;

    entry
}

/// Redact a JSON value recursively
pub fn redact_json(value: serde_json::Value, rules: &RedactionRules) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            serde_json::Value::String(redact_string(&s, rules))
        }
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                if rules.redact_fields.contains(&k.to_lowercase()) {
                    new_map.insert(k, serde_json::json!(rules.placeholder));
                } else {
                    new_map.insert(k, redact_json(v, rules));
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(
                arr.into_iter()
                    .map(|v| redact_json(v, rules))
                    .collect()
            )
        }
        other => other,
    }
}

/// Sensitive data detector (for warnings)
pub struct SensitiveDataDetector {
    rules: RedactionRules,
}

impl SensitiveDataDetector {
    pub fn new(rules: RedactionRules) -> Self {
        Self { rules }
    }

    /// Check if a string contains potentially sensitive data
    pub fn contains_sensitive(&self, input: &str) -> bool {
        // Check patterns
        for pattern in &self.rules.patterns {
            if let Ok(re) = regex_lite::Regex::new(&format!("(?i){}", pattern.pattern)) {
                if re.is_match(input) {
                    return true;
                }
            }
        }

        // Check for emails
        if self.rules.redact_emails {
            if let Ok(re) = regex_lite::Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}") {
                if re.is_match(input) {
                    return true;
                }
            }
        }

        false
    }

    /// Get list of potentially sensitive fields in a map
    pub fn sensitive_fields(&self, fields: &std::collections::HashMap<String, serde_json::Value>) -> Vec<String> {
        let mut sensitive = Vec::new();
        for key in fields.keys() {
            if self.rules.redact_fields.contains(&key.to_lowercase()) {
                sensitive.push(key.clone());
            }
        }
        sensitive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_api_key() {
        let rules = RedactionRules::default();
        // Test the pattern matches api_key
        let input = "Using api_key value";
        let result = redact_string(input, &rules);
        assert!(result.contains("[API_KEY]"));

        // Test bearer token redaction
        let input2 = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let result2 = redact_string(input2, &rules);
        assert!(result2.contains("[BEARER_TOKEN]"));
    }

    #[test]
    fn test_redact_email() {
        let rules = RedactionRules::default();
        let input = "Contact: user@example.com";
        let result = redact_string(input, &rules);
        assert!(!result.contains("user@example.com"));
        assert!(result.contains("[EMAIL]"));
    }

    #[test]
    fn test_redact_ip() {
        let rules = RedactionRules::default();
        let input = "Connection from 192.168.1.100";
        let result = redact_string(input, &rules);
        assert!(!result.contains("192.168.1.100"));
        assert!(result.contains("[IP_ADDRESS]"));
    }

    #[test]
    fn test_redact_path() {
        let rules = RedactionRules::default();
        let input = "File at /Users/johndoe/secret.txt";
        let result = redact_string(input, &rules);
        assert!(!result.contains("johndoe"));
        assert!(result.contains("[USER]"));
    }

    #[test]
    fn test_redact_log_entry() {
        let rules = RedactionRules::default();
        let entry = LogEntry::new(crate::LogLevel::Info, "test", "Password: secret123")
            .with_field("api_key", "sk_test_12345");

        let redacted = redact_log_entry(entry, &rules);
        assert!(!redacted.message.contains("secret123"));
        assert_eq!(
            redacted.fields.get("api_key"),
            Some(&serde_json::json!("[REDACTED]"))
        );
    }

    #[test]
    fn test_sensitive_detector() {
        let detector = SensitiveDataDetector::new(RedactionRules::default());

        assert!(detector.contains_sensitive("api_key: test"));
        assert!(detector.contains_sensitive("user@example.com"));
        assert!(!detector.contains_sensitive("hello world"));
    }
}
