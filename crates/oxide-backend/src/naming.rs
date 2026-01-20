//! Naming policy enforcement
//!
//! Enforces consistent naming conventions across API contracts:
//! - camelCase (recommended) for JSON properties
//! - PascalCase for type/schema names
//! - Prevents mixing conventions
//!
//! This eliminates the common failure mode of:
//! > frontend camelCase vs backend snake_case -> weeks of debugging

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Naming conventions supported by OxideKit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NamingConvention {
    /// camelCase - recommended for JSON APIs
    #[default]
    CamelCase,
    /// snake_case - common in Python/Rust backends
    SnakeCase,
    /// PascalCase - for type names
    PascalCase,
    /// kebab-case - for URLs
    KebabCase,
}

impl NamingConvention {
    /// Get the human-readable name of the convention
    pub fn name(&self) -> &'static str {
        match self {
            NamingConvention::CamelCase => "camelCase",
            NamingConvention::SnakeCase => "snake_case",
            NamingConvention::PascalCase => "PascalCase",
            NamingConvention::KebabCase => "kebab-case",
        }
    }

    /// Convert a string to this naming convention
    pub fn convert(&self, input: &str) -> String {
        let words = split_into_words(input);
        match self {
            NamingConvention::CamelCase => to_camel_case(&words),
            NamingConvention::SnakeCase => to_snake_case(&words),
            NamingConvention::PascalCase => to_pascal_case(&words),
            NamingConvention::KebabCase => to_kebab_case(&words),
        }
    }
}

/// Naming policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingPolicy {
    /// Convention for JSON property names
    pub convention: NamingConvention,
    /// Convention for type/schema names (always PascalCase)
    pub type_convention: NamingConvention,
    /// Allow single character names (not recommended)
    pub allow_single_char: bool,
    /// Reserved words that are not allowed
    pub reserved_words: Vec<String>,
    /// Custom allowed names (exceptions to the rule)
    pub allowed_exceptions: Vec<String>,
}

impl Default for NamingPolicy {
    fn default() -> Self {
        Self {
            convention: NamingConvention::CamelCase,
            type_convention: NamingConvention::PascalCase,
            allow_single_char: false,
            reserved_words: vec![
                "class".to_string(),
                "type".to_string(),
                "interface".to_string(),
                "enum".to_string(),
                "default".to_string(),
            ],
            allowed_exceptions: Vec::new(),
        }
    }
}

impl NamingPolicy {
    /// Create a new naming policy with the specified convention
    pub fn new(convention: NamingConvention) -> Self {
        Self {
            convention,
            ..Default::default()
        }
    }

    /// Add an exception to the naming rules
    pub fn with_exception(mut self, name: impl Into<String>) -> Self {
        self.allowed_exceptions.push(name.into());
        self
    }

    /// Allow single character names
    pub fn allow_single_char(mut self, allow: bool) -> Self {
        self.allow_single_char = allow;
        self
    }
}

/// Validates names against naming conventions
#[derive(Debug)]
pub struct NamingValidator {
    convention: NamingConvention,
    policy: NamingPolicy,
}

impl NamingValidator {
    /// Create a new validator with the specified convention
    pub fn new(convention: NamingConvention) -> Self {
        Self {
            convention,
            policy: NamingPolicy::new(convention),
        }
    }

    /// Create a validator with a full policy
    pub fn with_policy(policy: NamingPolicy) -> Self {
        Self {
            convention: policy.convention,
            policy,
        }
    }

    /// Get the convention being used
    pub fn convention(&self) -> NamingConvention {
        self.convention
    }

    /// Check if a name is valid according to the convention
    pub fn is_valid(&self, name: &str) -> bool {
        // Check for exceptions
        if self.policy.allowed_exceptions.contains(&name.to_string()) {
            return true;
        }

        // Check for reserved words
        if self.policy.reserved_words.contains(&name.to_lowercase()) {
            return false;
        }

        // Check length
        if name.is_empty() {
            return false;
        }
        if !self.policy.allow_single_char && name.len() == 1 {
            return false;
        }

        // Check convention
        match self.convention {
            NamingConvention::CamelCase => is_camel_case(name),
            NamingConvention::SnakeCase => is_snake_case(name),
            NamingConvention::PascalCase => is_pascal_case(name),
            NamingConvention::KebabCase => is_kebab_case(name),
        }
    }

    /// Check if a type name is valid (always PascalCase)
    pub fn is_valid_type_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        is_pascal_case(name)
    }

    /// Validate a name and return a detailed result
    pub fn validate(&self, name: &str) -> ValidationResult {
        let mut result = ValidationResult {
            name: name.to_string(),
            valid: true,
            errors: Vec::new(),
            suggestion: None,
        };

        if name.is_empty() {
            result.valid = false;
            result.errors.push("Name cannot be empty".to_string());
            return result;
        }

        if !self.policy.allow_single_char && name.len() == 1 {
            result.valid = false;
            result.errors.push("Single character names are not allowed".to_string());
            return result;
        }

        if self.policy.reserved_words.contains(&name.to_lowercase()) {
            result.valid = false;
            result.errors.push(format!("'{}' is a reserved word", name));
            return result;
        }

        if !self.is_valid(name) {
            result.valid = false;
            result.errors.push(format!(
                "'{}' does not follow {} convention",
                name,
                self.convention.name()
            ));
            result.suggestion = Some(self.convention.convert(name));
        }

        result
    }

    /// Validate multiple names and return all results
    pub fn validate_all(&self, names: &[&str]) -> Vec<ValidationResult> {
        names.iter().map(|n| self.validate(n)).collect()
    }
}

/// Result of a name validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// The name that was validated
    pub name: String,
    /// Whether the name is valid
    pub valid: bool,
    /// List of validation errors
    pub errors: Vec<String>,
    /// Suggested correction if invalid
    pub suggestion: Option<String>,
}

// Regex patterns for naming conventions
static CAMEL_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-zA-Z0-9]*$").unwrap());

static SNAKE_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").unwrap());

static PASCAL_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap());

static KEBAB_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9-]*$").unwrap());

/// Check if a string is camelCase
fn is_camel_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Must start with lowercase
    if !s.chars().next().unwrap().is_ascii_lowercase() {
        return false;
    }
    // Check against regex
    CAMEL_CASE_RE.is_match(s)
}

/// Check if a string is snake_case
fn is_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    SNAKE_CASE_RE.is_match(s)
}

/// Check if a string is PascalCase
fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Must start with uppercase
    if !s.chars().next().unwrap().is_ascii_uppercase() {
        return false;
    }
    PASCAL_CASE_RE.is_match(s)
}

/// Check if a string is kebab-case
fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    KEBAB_CASE_RE.is_match(s)
}

/// Split a string into words, handling different naming conventions
fn split_into_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current_word = String::new();

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            if !current_word.is_empty() {
                words.push(current_word.to_lowercase());
                current_word = String::new();
            }
        } else if c.is_ascii_uppercase() && !current_word.is_empty() {
            words.push(current_word.to_lowercase());
            current_word = c.to_string();
        } else {
            current_word.push(c);
        }
    }

    if !current_word.is_empty() {
        words.push(current_word.to_lowercase());
    }

    words
}

/// Convert words to camelCase
fn to_camel_case(words: &[String]) -> String {
    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].clone();
    for word in &words[1..] {
        if !word.is_empty() {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_ascii_uppercase());
                result.extend(chars);
            }
        }
    }
    result
}

/// Convert words to snake_case
fn to_snake_case(words: &[String]) -> String {
    words.join("_")
}

/// Convert words to PascalCase
fn to_pascal_case(words: &[String]) -> String {
    words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

/// Convert words to kebab-case
fn to_kebab_case(words: &[String]) -> String {
    words.join("-")
}

/// Batch convert names to a convention
pub fn batch_convert(names: &[&str], convention: NamingConvention) -> Vec<String> {
    names.iter().map(|n| convention.convert(n)).collect()
}

/// Check for mixed naming conventions in a set of names
pub fn detect_mixed_conventions(names: &[&str]) -> Option<Vec<NamingConvention>> {
    let mut found = Vec::new();

    for name in names {
        if is_camel_case(name) && !found.contains(&NamingConvention::CamelCase) {
            found.push(NamingConvention::CamelCase);
        }
        if is_snake_case(name) && !found.contains(&NamingConvention::SnakeCase) {
            found.push(NamingConvention::SnakeCase);
        }
        if is_pascal_case(name) && !found.contains(&NamingConvention::PascalCase) {
            found.push(NamingConvention::PascalCase);
        }
        if is_kebab_case(name) && !found.contains(&NamingConvention::KebabCase) {
            found.push(NamingConvention::KebabCase);
        }
    }

    if found.len() > 1 {
        Some(found)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_camel_case() {
        assert!(is_camel_case("userId"));
        assert!(is_camel_case("userName"));
        assert!(is_camel_case("firstName"));
        assert!(is_camel_case("id"));
        assert!(!is_camel_case("user_id"));
        assert!(!is_camel_case("UserId"));
        assert!(!is_camel_case("user-id"));
    }

    #[test]
    fn test_is_snake_case() {
        assert!(is_snake_case("user_id"));
        assert!(is_snake_case("user_name"));
        assert!(is_snake_case("first_name"));
        assert!(!is_snake_case("userId"));
        assert!(!is_snake_case("UserId"));
    }

    #[test]
    fn test_is_pascal_case() {
        assert!(is_pascal_case("User"));
        assert!(is_pascal_case("UserProfile"));
        assert!(is_pascal_case("FirstName"));
        assert!(!is_pascal_case("user"));
        assert!(!is_pascal_case("user_profile"));
    }

    #[test]
    fn test_is_kebab_case() {
        assert!(is_kebab_case("user-id"));
        assert!(is_kebab_case("user-profile"));
        assert!(!is_kebab_case("userId"));
        assert!(!is_kebab_case("user_id"));
    }

    #[test]
    fn test_naming_convention_convert() {
        assert_eq!(NamingConvention::CamelCase.convert("user_id"), "userId");
        assert_eq!(NamingConvention::SnakeCase.convert("userId"), "user_id");
        assert_eq!(NamingConvention::PascalCase.convert("user_id"), "UserId");
        assert_eq!(NamingConvention::KebabCase.convert("userId"), "user-id");
    }

    #[test]
    fn test_naming_validator_camel_case() {
        let validator = NamingValidator::new(NamingConvention::CamelCase);
        assert!(validator.is_valid("userId"));
        assert!(validator.is_valid("firstName"));
        assert!(!validator.is_valid("user_id"));
        assert!(!validator.is_valid("UserId"));
    }

    #[test]
    fn test_naming_validator_snake_case() {
        let validator = NamingValidator::new(NamingConvention::SnakeCase);
        assert!(validator.is_valid("user_id"));
        assert!(validator.is_valid("first_name"));
        assert!(!validator.is_valid("userId"));
        assert!(!validator.is_valid("UserId"));
    }

    #[test]
    fn test_naming_validator_type_names() {
        let validator = NamingValidator::new(NamingConvention::CamelCase);
        assert!(validator.is_valid_type_name("User"));
        assert!(validator.is_valid_type_name("UserProfile"));
        assert!(!validator.is_valid_type_name("user"));
        assert!(!validator.is_valid_type_name("user_profile"));
    }

    #[test]
    fn test_naming_validator_reserved_words() {
        let validator = NamingValidator::new(NamingConvention::CamelCase);
        assert!(!validator.is_valid("class"));
        assert!(!validator.is_valid("type"));
    }

    #[test]
    fn test_naming_validator_exceptions() {
        let policy = NamingPolicy::new(NamingConvention::CamelCase)
            .with_exception("$ref");
        let validator = NamingValidator::with_policy(policy);
        assert!(validator.is_valid("$ref"));
    }

    #[test]
    fn test_naming_validator_single_char() {
        let validator = NamingValidator::new(NamingConvention::CamelCase);
        assert!(!validator.is_valid("x"));

        let policy = NamingPolicy::new(NamingConvention::CamelCase).allow_single_char(true);
        let validator2 = NamingValidator::with_policy(policy);
        assert!(validator2.is_valid("x"));
    }

    #[test]
    fn test_validate_with_suggestion() {
        let validator = NamingValidator::new(NamingConvention::CamelCase);
        let result = validator.validate("user_name");

        assert!(!result.valid);
        assert_eq!(result.suggestion, Some("userName".to_string()));
    }

    #[test]
    fn test_batch_convert() {
        let names = vec!["user_id", "first_name", "last_name"];
        let converted = batch_convert(&names, NamingConvention::CamelCase);

        assert_eq!(converted, vec!["userId", "firstName", "lastName"]);
    }

    #[test]
    fn test_detect_mixed_conventions() {
        let mixed = vec!["userId", "first_name", "LastName"];
        let result = detect_mixed_conventions(&mixed);

        assert!(result.is_some());
        let conventions = result.unwrap();
        assert!(conventions.contains(&NamingConvention::CamelCase));
        assert!(conventions.contains(&NamingConvention::SnakeCase));
    }

    #[test]
    fn test_detect_consistent_conventions() {
        let consistent = vec!["userId", "firstName", "lastName"];
        let result = detect_mixed_conventions(&consistent);

        assert!(result.is_none());
    }

    #[test]
    fn test_split_into_words() {
        assert_eq!(
            split_into_words("userId"),
            vec!["user", "id"]
        );
        assert_eq!(
            split_into_words("user_id"),
            vec!["user", "id"]
        );
        assert_eq!(
            split_into_words("user-id"),
            vec!["user", "id"]
        );
        assert_eq!(
            split_into_words("UserProfile"),
            vec!["user", "profile"]
        );
    }
}
