//! Async Validation Support
//!
//! Provides async validators for server-side validation scenarios like
//! checking email availability, validating usernames, etc.

use crate::error::ValidationError;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Type alias for async validation result
pub type AsyncValidationResult = Pin<Box<dyn Future<Output = Option<ValidationError>> + Send>>;

/// Trait for async validators
#[cfg(feature = "async")]
#[async_trait]
pub trait AsyncValidator<T>: Send + Sync
where
    T: Send + Sync,
{
    /// Validate a value asynchronously
    async fn validate_async(&self, value: &T, field_name: &str) -> Option<ValidationError>;

    /// Get a description of this validator
    fn description(&self) -> String;
}

/// Box type for async validators
#[cfg(feature = "async")]
pub type BoxedAsyncValidator<T> = Box<dyn AsyncValidator<T>>;

/// A chain of async validators
#[cfg(feature = "async")]
pub struct AsyncValidatorChain<T>
where
    T: Send + Sync,
{
    validators: Vec<BoxedAsyncValidator<T>>,
}

#[cfg(feature = "async")]
impl<T> AsyncValidatorChain<T>
where
    T: Send + Sync + 'static,
{
    /// Create a new empty async validator chain
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add an async validator to the chain
    pub fn add<V: AsyncValidator<T> + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Box::new(validator));
        self
    }

    /// Validate a value against all async validators in the chain
    pub async fn validate(&self, value: &T, field_name: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for validator in &self.validators {
            if let Some(error) = validator.validate_async(value, field_name).await {
                errors.push(error);
            }
        }
        errors
    }

    /// Validate and return only the first error
    pub async fn validate_first(&self, value: &T, field_name: &str) -> Option<ValidationError> {
        for validator in &self.validators {
            if let Some(error) = validator.validate_async(value, field_name).await {
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

#[cfg(feature = "async")]
impl<T> Default for AsyncValidatorChain<T>
where
    T: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Custom async validator using a closure
#[cfg(feature = "async")]
pub struct AsyncCustom<T, F, Fut>
where
    F: Fn(T, String) -> Fut + Send + Sync,
    Fut: Future<Output = Option<ValidationError>> + Send,
    T: Clone + Send + Sync,
{
    validator_fn: Arc<F>,
    description: String,
    _marker: std::marker::PhantomData<(T, Fut)>,
}

#[cfg(feature = "async")]
impl<T, F, Fut> AsyncCustom<T, F, Fut>
where
    F: Fn(T, String) -> Fut + Send + Sync,
    Fut: Future<Output = Option<ValidationError>> + Send,
    T: Clone + Send + Sync,
{
    /// Create a new custom async validator
    pub fn new(description: impl Into<String>, f: F) -> Self {
        Self {
            validator_fn: Arc::new(f),
            description: description.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "async")]
#[async_trait]
impl<T, F, Fut> AsyncValidator<T> for AsyncCustom<T, F, Fut>
where
    F: Fn(T, String) -> Fut + Send + Sync,
    Fut: Future<Output = Option<ValidationError>> + Send,
    T: Clone + Send + Sync,
{
    async fn validate_async(&self, value: &T, field_name: &str) -> Option<ValidationError> {
        (self.validator_fn)(value.clone(), field_name.to_string()).await
    }

    fn description(&self) -> String {
        self.description.clone()
    }
}

#[cfg(feature = "async")]
impl<T, F, Fut> fmt::Debug for AsyncCustom<T, F, Fut>
where
    F: Fn(T, String) -> Fut + Send + Sync,
    Fut: Future<Output = Option<ValidationError>> + Send,
    T: Clone + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncCustom")
            .field("description", &self.description)
            .finish()
    }
}

/// Debounced async validator wrapper
#[cfg(feature = "async")]
pub struct DebouncedValidator<V> {
    validator: V,
    delay_ms: u64,
}

#[cfg(feature = "async")]
impl<V> DebouncedValidator<V> {
    /// Create a new debounced validator
    pub fn new(validator: V, delay_ms: u64) -> Self {
        Self { validator, delay_ms }
    }

    /// Get the delay in milliseconds
    pub fn delay_ms(&self) -> u64 {
        self.delay_ms
    }
}

#[cfg(feature = "async")]
#[async_trait]
impl<T, V> AsyncValidator<T> for DebouncedValidator<V>
where
    T: Send + Sync,
    V: AsyncValidator<T>,
{
    async fn validate_async(&self, value: &T, field_name: &str) -> Option<ValidationError> {
        tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
        self.validator.validate_async(value, field_name).await
    }

    fn description(&self) -> String {
        format!("Debounced({}ms, {})", self.delay_ms, self.validator.description())
    }
}

/// Validation state for async operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AsyncValidationState {
    /// Not yet validated
    #[default]
    Idle,
    /// Currently validating
    Validating,
    /// Validation completed successfully
    Valid,
    /// Validation completed with errors
    Invalid,
}

impl AsyncValidationState {
    /// Check if currently validating
    pub fn is_validating(&self) -> bool {
        matches!(self, Self::Validating)
    }

    /// Check if validation is complete (either valid or invalid)
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Valid | Self::Invalid)
    }

    /// Check if valid
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Check if invalid
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

/// Helper function to create a custom async validator
#[cfg(feature = "async")]
pub fn async_custom<T, F, Fut>(description: impl Into<String>, f: F) -> AsyncCustom<T, F, Fut>
where
    F: Fn(T, String) -> Fut + Send + Sync,
    Fut: Future<Output = Option<ValidationError>> + Send,
    T: Clone + Send + Sync,
{
    AsyncCustom::new(description, f)
}

/// Helper function to create a debounced validator
#[cfg(feature = "async")]
pub fn debounced<V>(validator: V, delay_ms: u64) -> DebouncedValidator<V> {
    DebouncedValidator::new(validator, delay_ms)
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;

    struct EmailAvailabilityValidator;

    #[async_trait]
    impl AsyncValidator<String> for EmailAvailabilityValidator {
        async fn validate_async(&self, value: &String, field_name: &str) -> Option<ValidationError> {
            // Simulate async check
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;

            if value == "taken@example.com" {
                Some(ValidationError::custom(field_name, "This email is already registered"))
            } else {
                None
            }
        }

        fn description(&self) -> String {
            "EmailAvailability".to_string()
        }
    }

    #[tokio::test]
    async fn test_async_validator() {
        let validator = EmailAvailabilityValidator;

        let result = validator
            .validate_async(&"available@example.com".to_string(), "email")
            .await;
        assert!(result.is_none());

        let result = validator
            .validate_async(&"taken@example.com".to_string(), "email")
            .await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_async_validator_chain() {
        let chain = AsyncValidatorChain::new().add(EmailAvailabilityValidator);

        let errors = chain
            .validate(&"available@example.com".to_string(), "email")
            .await;
        assert!(errors.is_empty());

        let errors = chain
            .validate(&"taken@example.com".to_string(), "email")
            .await;
        assert!(!errors.is_empty());
    }

    #[tokio::test]
    async fn test_async_custom() {
        let validator = async_custom("must_not_be_test", |value: String, field_name: String| async move {
            if value == "test" {
                Some(ValidationError::custom(field_name, "Cannot use 'test'"))
            } else {
                None
            }
        });

        let result = validator.validate_async(&"hello".to_string(), "field").await;
        assert!(result.is_none());

        let result = validator.validate_async(&"test".to_string(), "field").await;
        assert!(result.is_some());
    }

    #[test]
    fn test_async_validation_state() {
        assert!(!AsyncValidationState::Idle.is_validating());
        assert!(AsyncValidationState::Validating.is_validating());
        assert!(AsyncValidationState::Valid.is_complete());
        assert!(AsyncValidationState::Invalid.is_complete());
        assert!(AsyncValidationState::Valid.is_valid());
        assert!(AsyncValidationState::Invalid.is_invalid());
    }
}
