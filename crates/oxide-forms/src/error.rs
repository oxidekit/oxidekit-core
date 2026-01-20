//! Error Types for Form Validation
//!
//! Provides structured error types for form field validation, including
//! support for multiple errors per field and form-level error aggregation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A validation error for a specific field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationError {
    /// The field path (e.g., "email", "address.city")
    pub field: String,

    /// Human-readable error message
    pub message: String,

    /// Error code for programmatic handling
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Additional context for the error
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub context: HashMap<String, String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: None,
            context: HashMap::new(),
        }
    }

    /// Create a validation error with an error code
    pub fn with_code(
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: Some(code.into()),
            context: HashMap::new(),
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Create a "required" error
    pub fn required(field: impl Into<String>) -> Self {
        Self::with_code(field, "This field is required", "required")
    }

    /// Create a "min_length" error
    pub fn min_length(field: impl Into<String>, min: usize, actual: usize) -> Self {
        Self::with_code(
            field,
            format!("Must be at least {} characters (currently {})", min, actual),
            "min_length",
        )
        .with_context("min", min.to_string())
        .with_context("actual", actual.to_string())
    }

    /// Create a "max_length" error
    pub fn max_length(field: impl Into<String>, max: usize, actual: usize) -> Self {
        Self::with_code(
            field,
            format!("Must be at most {} characters (currently {})", max, actual),
            "max_length",
        )
        .with_context("max", max.to_string())
        .with_context("actual", actual.to_string())
    }

    /// Create an "exact_length" error
    pub fn exact_length(field: impl Into<String>, expected: usize, actual: usize) -> Self {
        Self::with_code(
            field,
            format!(
                "Must be exactly {} characters (currently {})",
                expected, actual
            ),
            "exact_length",
        )
        .with_context("expected", expected.to_string())
        .with_context("actual", actual.to_string())
    }

    /// Create a "min" error for numeric values
    pub fn min_value<T: fmt::Display>(field: impl Into<String>, min: T, actual: T) -> Self {
        Self::with_code(
            field,
            format!("Must be at least {} (currently {})", min, actual),
            "min_value",
        )
        .with_context("min", min.to_string())
        .with_context("actual", actual.to_string())
    }

    /// Create a "max" error for numeric values
    pub fn max_value<T: fmt::Display>(field: impl Into<String>, max: T, actual: T) -> Self {
        Self::with_code(
            field,
            format!("Must be at most {} (currently {})", max, actual),
            "max_value",
        )
        .with_context("max", max.to_string())
        .with_context("actual", actual.to_string())
    }

    /// Create a "range" error for numeric values
    pub fn out_of_range<T: fmt::Display>(
        field: impl Into<String>,
        min: T,
        max: T,
        actual: T,
    ) -> Self {
        Self::with_code(
            field,
            format!("Must be between {} and {} (currently {})", min, max, actual),
            "out_of_range",
        )
        .with_context("min", min.to_string())
        .with_context("max", max.to_string())
        .with_context("actual", actual.to_string())
    }

    /// Create a "pattern" error
    pub fn pattern(field: impl Into<String>, pattern_description: impl Into<String>) -> Self {
        Self::with_code(field, pattern_description, "pattern")
    }

    /// Create an "invalid_email" error
    pub fn invalid_email(field: impl Into<String>) -> Self {
        Self::with_code(field, "Invalid email address", "invalid_email")
    }

    /// Create an "invalid_url" error
    pub fn invalid_url(field: impl Into<String>) -> Self {
        Self::with_code(field, "Invalid URL", "invalid_url")
    }

    /// Create an "invalid_phone" error
    pub fn invalid_phone(field: impl Into<String>) -> Self {
        Self::with_code(field, "Invalid phone number", "invalid_phone")
    }

    /// Create a "mismatch" error (e.g., for password confirmation)
    pub fn mismatch(field: impl Into<String>, other_field: impl Into<String>) -> Self {
        let other = other_field.into();
        Self::with_code(
            field,
            format!("Does not match {}", other),
            "mismatch",
        )
        .with_context("other_field", other)
    }

    /// Create a "custom" error
    pub fn custom(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::with_code(field, message, "custom")
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// A collection of validation errors for a form
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationErrors {
    /// Errors grouped by field name
    errors: HashMap<String, Vec<ValidationError>>,
}

impl ValidationErrors {
    /// Create an empty error collection
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    /// Add an error to the collection
    pub fn add(&mut self, error: ValidationError) {
        self.errors
            .entry(error.field.clone())
            .or_default()
            .push(error);
    }

    /// Add multiple errors
    pub fn add_all(&mut self, errors: impl IntoIterator<Item = ValidationError>) {
        for error in errors {
            self.add(error);
        }
    }

    /// Check if there are any errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the total number of errors
    pub fn len(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }

    /// Check if a specific field has errors
    pub fn has_errors(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }

    /// Get errors for a specific field
    pub fn get_errors(&self, field: &str) -> Option<&Vec<ValidationError>> {
        self.errors.get(field)
    }

    /// Get the first error for a field
    pub fn get_first_error(&self, field: &str) -> Option<&ValidationError> {
        self.errors.get(field).and_then(|v| v.first())
    }

    /// Get all errors as a flat list
    pub fn all_errors(&self) -> Vec<&ValidationError> {
        self.errors.values().flatten().collect()
    }

    /// Get all field names that have errors
    pub fn field_names(&self) -> Vec<&str> {
        self.errors.keys().map(|s| s.as_str()).collect()
    }

    /// Clear all errors
    pub fn clear(&mut self) {
        self.errors.clear();
    }

    /// Clear errors for a specific field
    pub fn clear_field(&mut self, field: &str) {
        self.errors.remove(field);
    }

    /// Merge another set of errors into this one
    pub fn merge(&mut self, other: ValidationErrors) {
        for (field, errors) in other.errors {
            self.errors.entry(field).or_default().extend(errors);
        }
    }

    /// Convert to a map of field -> first error message
    pub fn to_message_map(&self) -> HashMap<String, String> {
        self.errors
            .iter()
            .filter_map(|(field, errors)| {
                errors.first().map(|e| (field.clone(), e.message.clone()))
            })
            .collect()
    }

    /// Iterate over all errors
    pub fn iter(&self) -> impl Iterator<Item = &ValidationError> {
        self.errors.values().flatten()
    }
}

impl FromIterator<ValidationError> for ValidationErrors {
    fn from_iter<T: IntoIterator<Item = ValidationError>>(iter: T) -> Self {
        let mut errors = ValidationErrors::new();
        for error in iter {
            errors.add(error);
        }
        errors
    }
}

impl IntoIterator for ValidationErrors {
    type Item = ValidationError;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors
            .into_values()
            .flatten()
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let messages: Vec<String> = self.iter().map(|e| e.to_string()).collect();
        write!(f, "{}", messages.join("; "))
    }
}

/// Result type for validation operations
pub type ValidationResult = Result<(), ValidationErrors>;

/// Helper trait for converting validation results
pub trait IntoValidationResult {
    fn into_validation_result(self) -> ValidationResult;
}

impl IntoValidationResult for Option<ValidationError> {
    fn into_validation_result(self) -> ValidationResult {
        match self {
            Some(error) => {
                let mut errors = ValidationErrors::new();
                errors.add(error);
                Err(errors)
            }
            None => Ok(()),
        }
    }
}

impl IntoValidationResult for Vec<ValidationError> {
    fn into_validation_result(self) -> ValidationResult {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.into_iter().collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_new() {
        let error = ValidationError::new("email", "Invalid email");
        assert_eq!(error.field, "email");
        assert_eq!(error.message, "Invalid email");
        assert!(error.code.is_none());
    }

    #[test]
    fn test_validation_error_with_code() {
        let error = ValidationError::with_code("email", "Invalid email", "invalid_email");
        assert_eq!(error.code, Some("invalid_email".to_string()));
    }

    #[test]
    fn test_validation_error_with_context() {
        let error = ValidationError::min_length("name", 5, 3);
        assert_eq!(error.context.get("min"), Some(&"5".to_string()));
        assert_eq!(error.context.get("actual"), Some(&"3".to_string()));
    }

    #[test]
    fn test_validation_errors_collection() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::required("email"));
        errors.add(ValidationError::min_length("password", 8, 5));

        assert!(!errors.is_empty());
        assert_eq!(errors.len(), 2);
        assert!(errors.has_errors("email"));
        assert!(errors.has_errors("password"));
        assert!(!errors.has_errors("name"));
    }

    #[test]
    fn test_validation_errors_multiple_per_field() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::required("password"));
        errors.add(ValidationError::min_length("password", 8, 0));

        assert_eq!(errors.len(), 2);
        assert_eq!(errors.get_errors("password").map(|v| v.len()), Some(2));
    }

    #[test]
    fn test_validation_errors_merge() {
        let mut errors1 = ValidationErrors::new();
        errors1.add(ValidationError::required("email"));

        let mut errors2 = ValidationErrors::new();
        errors2.add(ValidationError::required("password"));

        errors1.merge(errors2);
        assert_eq!(errors1.len(), 2);
    }

    #[test]
    fn test_validation_errors_to_message_map() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::required("email"));
        errors.add(ValidationError::min_length("password", 8, 5));

        let map = errors.to_message_map();
        assert!(map.contains_key("email"));
        assert!(map.contains_key("password"));
    }
}
