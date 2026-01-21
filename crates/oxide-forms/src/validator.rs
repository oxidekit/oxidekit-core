//! Validator Trait and Built-in Validators
//!
//! Provides a trait for creating custom validators and a comprehensive set
//! of built-in validators for common validation scenarios.

use crate::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// Trait for validating form field values
pub trait Validator<T>: Send + Sync {
    /// Validate a value and return an error if invalid
    fn validate(&self, value: &T, field_name: &str) -> Option<ValidationError>;

    /// Get a description of this validator for debugging/display
    fn description(&self) -> String;
}

/// Box type for validators
pub type BoxedValidator<T> = Box<dyn Validator<T>>;

/// A chain of validators to run in sequence
pub struct ValidatorChain<T> {
    validators: Vec<BoxedValidator<T>>,
}

impl<T> ValidatorChain<T> {
    /// Create a new empty validator chain
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a validator to the chain
    pub fn add<V: Validator<T> + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Box::new(validator));
        self
    }

    /// Validate a value against all validators in the chain
    pub fn validate(&self, value: &T, field_name: &str) -> Vec<ValidationError> {
        self.validators
            .iter()
            .filter_map(|v| v.validate(value, field_name))
            .collect()
    }

    /// Validate and return only the first error
    pub fn validate_first(&self, value: &T, field_name: &str) -> Option<ValidationError> {
        for validator in &self.validators {
            if let Some(error) = validator.validate(value, field_name) {
                return Some(error);
            }
        }
        None
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// Get the number of validators
    pub fn len(&self) -> usize {
        self.validators.len()
    }
}

impl<T> Default for ValidatorChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Debug for ValidatorChain<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatorChain")
            .field("count", &self.validators.len())
            .finish()
    }
}

// ============================================================================
// Built-in Validators for String types
// ============================================================================

/// Validator that requires a non-empty value
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Required;

impl Validator<String> for Required {
    fn validate(&self, value: &String, field_name: &str) -> Option<ValidationError> {
        if value.trim().is_empty() {
            Some(ValidationError::required(field_name))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        "Required".to_string()
    }
}

impl Validator<Option<String>> for Required {
    fn validate(&self, value: &Option<String>, field_name: &str) -> Option<ValidationError> {
        match value {
            Some(v) if !v.trim().is_empty() => None,
            _ => Some(ValidationError::required(field_name)),
        }
    }

    fn description(&self) -> String {
        "Required".to_string()
    }
}

/// Validator that allows empty values (skips validation for empty)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Optional;

impl<T> Validator<Option<T>> for Optional {
    fn validate(&self, _value: &Option<T>, _field_name: &str) -> Option<ValidationError> {
        None // Optional never fails
    }

    fn description(&self) -> String {
        "Optional".to_string()
    }
}

/// Minimum length validator
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MinLength(pub usize);

impl Validator<String> for MinLength {
    fn validate(&self, value: &String, field_name: &str) -> Option<ValidationError> {
        let len = value.chars().count();
        if len < self.0 {
            Some(ValidationError::min_length(field_name, self.0, len))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        format!("MinLength({})", self.0)
    }
}

/// Maximum length validator
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MaxLength(pub usize);

impl Validator<String> for MaxLength {
    fn validate(&self, value: &String, field_name: &str) -> Option<ValidationError> {
        let len = value.chars().count();
        if len > self.0 {
            Some(ValidationError::max_length(field_name, self.0, len))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        format!("MaxLength({})", self.0)
    }
}

/// Exact length validator
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExactLength(pub usize);

impl Validator<String> for ExactLength {
    fn validate(&self, value: &String, field_name: &str) -> Option<ValidationError> {
        let len = value.chars().count();
        if len != self.0 {
            Some(ValidationError::exact_length(field_name, self.0, len))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        format!("ExactLength({})", self.0)
    }
}

// ============================================================================
// Built-in Validators for Numeric types
// ============================================================================

/// Minimum value validator
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Min<T>(pub T);

macro_rules! impl_min_validator {
    ($($t:ty),*) => {
        $(
            impl Validator<$t> for Min<$t> {
                fn validate(&self, value: &$t, field_name: &str) -> Option<ValidationError> {
                    if *value < self.0 {
                        Some(ValidationError::min_value(field_name, self.0, *value))
                    } else {
                        None
                    }
                }

                fn description(&self) -> String {
                    format!("Min({})", self.0)
                }
            }
        )*
    };
}

impl_min_validator!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

/// Maximum value validator
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Max<T>(pub T);

macro_rules! impl_max_validator {
    ($($t:ty),*) => {
        $(
            impl Validator<$t> for Max<$t> {
                fn validate(&self, value: &$t, field_name: &str) -> Option<ValidationError> {
                    if *value > self.0 {
                        Some(ValidationError::max_value(field_name, self.0, *value))
                    } else {
                        None
                    }
                }

                fn description(&self) -> String {
                    format!("Max({})", self.0)
                }
            }
        )*
    };
}

impl_max_validator!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

/// Range validator (inclusive)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

impl<T> Range<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

macro_rules! impl_range_validator {
    ($($t:ty),*) => {
        $(
            impl Validator<$t> for Range<$t> {
                fn validate(&self, value: &$t, field_name: &str) -> Option<ValidationError> {
                    if *value < self.min || *value > self.max {
                        Some(ValidationError::out_of_range(field_name, self.min, self.max, *value))
                    } else {
                        None
                    }
                }

                fn description(&self) -> String {
                    format!("Range({}, {})", self.min, self.max)
                }
            }
        )*
    };
}

impl_range_validator!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);

// ============================================================================
// Pattern-based Validators
// ============================================================================

/// Regex pattern validator (requires "regex" feature)
#[cfg(feature = "regex")]
#[derive(Clone)]
pub struct Pattern {
    regex: regex::Regex,
    message: String,
}

#[cfg(feature = "regex")]
impl Pattern {
    /// Create a new pattern validator
    pub fn new(pattern: &str, message: impl Into<String>) -> Result<Self, regex::Error> {
        Ok(Self {
            regex: regex::Regex::new(pattern)?,
            message: message.into(),
        })
    }

    /// Create a pattern validator that panics on invalid regex (for compile-time patterns)
    pub fn new_unchecked(pattern: &str, message: impl Into<String>) -> Self {
        Self {
            regex: regex::Regex::new(pattern).expect("Invalid regex pattern"),
            message: message.into(),
        }
    }
}

#[cfg(feature = "regex")]
impl Validator<String> for Pattern {
    fn validate(&self, value: &String, field_name: &str) -> Option<ValidationError> {
        if !self.regex.is_match(value) {
            Some(ValidationError::pattern(field_name, &self.message))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        format!("Pattern({})", self.message)
    }
}

#[cfg(feature = "regex")]
impl fmt::Debug for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pattern")
            .field("pattern", &self.regex.as_str())
            .field("message", &self.message)
            .finish()
    }
}

// ============================================================================
// Format Validators (Email, URL, Phone)
// ============================================================================

/// Email validator
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Email;

impl Email {
    /// Simple email validation regex pattern
    const EMAIL_PATTERN: &'static str =
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";

    fn is_valid_email(value: &str) -> bool {
        // Basic email validation without regex
        let value = value.trim();
        if value.is_empty() {
            return false;
        }

        // Must contain exactly one @
        let parts: Vec<&str> = value.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        // Local part checks
        if local.is_empty() || local.len() > 64 {
            return false;
        }

        // Domain checks
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }

        // Domain must contain at least one dot
        if !domain.contains('.') {
            return false;
        }

        // Domain parts checks
        for part in domain.split('.') {
            if part.is_empty() || part.len() > 63 {
                return false;
            }
            // Must start and end with alphanumeric
            if !part.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                return false;
            }
            if !part.chars().last().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                return false;
            }
        }

        true
    }
}

impl Validator<String> for Email {
    fn validate(&self, value: &String, field_name: &str) -> Option<ValidationError> {
        if value.is_empty() {
            return None; // Use Required for empty validation
        }
        if !Self::is_valid_email(value) {
            Some(ValidationError::invalid_email(field_name))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        "Email".to_string()
    }
}

/// URL validator
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Url;

impl Url {
    fn is_valid_url(value: &str) -> bool {
        let value = value.trim();
        if value.is_empty() {
            return false;
        }

        // Must start with http:// or https://
        if !value.starts_with("http://") && !value.starts_with("https://") {
            return false;
        }

        // Remove protocol
        let rest = if value.starts_with("https://") {
            &value[8..]
        } else {
            &value[7..]
        };

        // Must have something after protocol
        if rest.is_empty() {
            return false;
        }

        // Get the host part (before any path, query, or fragment)
        let host = rest
            .split('/')
            .next()
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .split('#')
            .next()
            .unwrap_or("");

        // Remove port if present
        let host = host.split(':').next().unwrap_or(host);

        // Host must not be empty
        if host.is_empty() {
            return false;
        }

        // Host must contain at least one character
        !host.is_empty()
    }
}

impl Validator<String> for Url {
    fn validate(&self, value: &String, field_name: &str) -> Option<ValidationError> {
        if value.is_empty() {
            return None; // Use Required for empty validation
        }
        if !Self::is_valid_url(value) {
            Some(ValidationError::invalid_url(field_name))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        "Url".to_string()
    }
}

/// Phone number validator
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Phone;

impl Phone {
    fn is_valid_phone(value: &str) -> bool {
        let value = value.trim();
        if value.is_empty() {
            return false;
        }

        // Remove common formatting characters
        let digits: String = value
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '+')
            .collect();

        // Must have between 7 and 15 digits (E.164 standard)
        let digit_count = digits.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count < 7 || digit_count > 15 {
            return false;
        }

        // If starts with +, it must be followed by digits
        if digits.starts_with('+') && digits.len() < 2 {
            return false;
        }

        true
    }
}

impl Validator<String> for Phone {
    fn validate(&self, value: &String, field_name: &str) -> Option<ValidationError> {
        if value.is_empty() {
            return None; // Use Required for empty validation
        }
        if !Self::is_valid_phone(value) {
            Some(ValidationError::invalid_phone(field_name))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        "Phone".to_string()
    }
}

// ============================================================================
// Cross-field Validators
// ============================================================================

/// Matches another field value (e.g., password confirmation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchesField {
    pub other_field: String,
}

impl MatchesField {
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            other_field: field.into(),
        }
    }
}

/// Context for cross-field validation
pub trait FieldValueProvider {
    fn get_field_value(&self, field_name: &str) -> Option<String>;
}

/// Validator that uses a closure
pub struct Custom<T, F>
where
    F: Fn(&T, &str) -> Option<ValidationError> + Send + Sync,
{
    validator_fn: F,
    description: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T, F> Custom<T, F>
where
    F: Fn(&T, &str) -> Option<ValidationError> + Send + Sync,
{
    /// Create a new custom validator
    pub fn new(description: impl Into<String>, f: F) -> Self {
        Self {
            validator_fn: f,
            description: description.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, F> Validator<T> for Custom<T, F>
where
    T: Send + Sync,
    F: Fn(&T, &str) -> Option<ValidationError> + Send + Sync,
{
    fn validate(&self, value: &T, field_name: &str) -> Option<ValidationError> {
        (self.validator_fn)(value, field_name)
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

impl<T, F> fmt::Debug for Custom<T, F>
where
    F: Fn(&T, &str) -> Option<ValidationError> + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Custom")
            .field("description", &self.description)
            .finish()
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Create a Required validator
pub fn required() -> Required {
    Required
}

/// Create an Optional validator
pub fn optional() -> Optional {
    Optional
}

/// Create a MinLength validator
pub fn min_length(len: usize) -> MinLength {
    MinLength(len)
}

/// Create a MaxLength validator
pub fn max_length(len: usize) -> MaxLength {
    MaxLength(len)
}

/// Create an ExactLength validator
pub fn exact_length(len: usize) -> ExactLength {
    ExactLength(len)
}

/// Create a Min validator
pub fn min<T>(value: T) -> Min<T> {
    Min(value)
}

/// Create a Max validator
pub fn max<T>(value: T) -> Max<T> {
    Max(value)
}

/// Create a Range validator
pub fn range<T>(min: T, max: T) -> Range<T> {
    Range::new(min, max)
}

/// Create an Email validator
pub fn email() -> Email {
    Email
}

/// Create a Url validator
pub fn url() -> Url {
    Url
}

/// Create a Phone validator
pub fn phone() -> Phone {
    Phone
}

/// Create a MatchesField validator
pub fn matches_field(field: impl Into<String>) -> MatchesField {
    MatchesField::new(field)
}

/// Create a custom validator
pub fn custom<T, F>(description: impl Into<String>, f: F) -> Custom<T, F>
where
    F: Fn(&T, &str) -> Option<ValidationError> + Send + Sync,
{
    Custom::new(description, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required() {
        let validator = Required;

        assert!(validator.validate(&String::new(), "field").is_some());
        assert!(validator.validate(&"   ".to_string(), "field").is_some());
        assert!(validator.validate(&"hello".to_string(), "field").is_none());
    }

    #[test]
    fn test_min_length() {
        let validator = MinLength(5);

        assert!(validator.validate(&"hi".to_string(), "field").is_some());
        assert!(validator.validate(&"hello".to_string(), "field").is_none());
        assert!(validator.validate(&"hello world".to_string(), "field").is_none());
    }

    #[test]
    fn test_max_length() {
        let validator = MaxLength(5);

        assert!(validator.validate(&"hi".to_string(), "field").is_none());
        assert!(validator.validate(&"hello".to_string(), "field").is_none());
        assert!(validator.validate(&"hello world".to_string(), "field").is_some());
    }

    #[test]
    fn test_exact_length() {
        let validator = ExactLength(5);

        assert!(validator.validate(&"hi".to_string(), "field").is_some());
        assert!(validator.validate(&"hello".to_string(), "field").is_none());
        assert!(validator.validate(&"hello world".to_string(), "field").is_some());
    }

    #[test]
    fn test_min_numeric() {
        let validator = Min(10i32);

        assert!(validator.validate(&5i32, "field").is_some());
        assert!(validator.validate(&10i32, "field").is_none());
        assert!(validator.validate(&15i32, "field").is_none());
    }

    #[test]
    fn test_max_numeric() {
        let validator = Max(10i32);

        assert!(validator.validate(&5i32, "field").is_none());
        assert!(validator.validate(&10i32, "field").is_none());
        assert!(validator.validate(&15i32, "field").is_some());
    }

    #[test]
    fn test_range_numeric() {
        let validator = Range::new(5i32, 10i32);

        assert!(validator.validate(&3i32, "field").is_some());
        assert!(validator.validate(&5i32, "field").is_none());
        assert!(validator.validate(&7i32, "field").is_none());
        assert!(validator.validate(&10i32, "field").is_none());
        assert!(validator.validate(&12i32, "field").is_some());
    }

    #[test]
    fn test_email() {
        let validator = Email;

        assert!(validator.validate(&"".to_string(), "field").is_none()); // Empty is OK (use Required)
        assert!(validator.validate(&"test@example.com".to_string(), "field").is_none());
        assert!(validator.validate(&"user.name@domain.co.uk".to_string(), "field").is_none());
        assert!(validator.validate(&"invalid".to_string(), "field").is_some());
        assert!(validator.validate(&"@domain.com".to_string(), "field").is_some());
        assert!(validator.validate(&"user@".to_string(), "field").is_some());
    }

    #[test]
    fn test_url() {
        let validator = Url;

        assert!(validator.validate(&"".to_string(), "field").is_none()); // Empty is OK
        assert!(validator.validate(&"https://example.com".to_string(), "field").is_none());
        assert!(validator.validate(&"http://example.com/path".to_string(), "field").is_none());
        assert!(validator.validate(&"https://example.com:8080".to_string(), "field").is_none());
        assert!(validator.validate(&"example.com".to_string(), "field").is_some());
        assert!(validator.validate(&"ftp://example.com".to_string(), "field").is_some());
    }

    #[test]
    fn test_phone() {
        let validator = Phone;

        assert!(validator.validate(&"".to_string(), "field").is_none()); // Empty is OK
        assert!(validator.validate(&"+1234567890".to_string(), "field").is_none());
        assert!(validator.validate(&"123-456-7890".to_string(), "field").is_none());
        assert!(validator.validate(&"(123) 456-7890".to_string(), "field").is_none());
        assert!(validator.validate(&"123".to_string(), "field").is_some()); // Too short
    }

    #[test]
    fn test_custom_validator() {
        let validator = Custom::new("must be even", |value: &i32, field_name: &str| {
            if value % 2 != 0 {
                Some(ValidationError::custom(field_name, "Value must be even"))
            } else {
                None
            }
        });

        assert!(validator.validate(&3, "field").is_some());
        assert!(validator.validate(&4, "field").is_none());
    }

    #[test]
    fn test_validator_chain() {
        let chain = ValidatorChain::new()
            .add(Required)
            .add(MinLength(3))
            .add(MaxLength(10));

        // Empty fails Required
        let errors = chain.validate(&String::new(), "field");
        assert!(!errors.is_empty());

        // Too short fails MinLength
        let errors = chain.validate(&"ab".to_string(), "field");
        assert!(!errors.is_empty());

        // Too long fails MaxLength
        let errors = chain.validate(&"hello world!".to_string(), "field");
        assert!(!errors.is_empty());

        // Valid passes all
        let errors = chain.validate(&"hello".to_string(), "field");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validator_chain_first() {
        let chain = ValidatorChain::new()
            .add(Required)
            .add(MinLength(10));

        // Empty should return Required error first
        let error = chain.validate_first(&String::new(), "field");
        assert!(error.is_some());
        assert_eq!(error.unwrap().code, Some("required".to_string()));
    }
}
