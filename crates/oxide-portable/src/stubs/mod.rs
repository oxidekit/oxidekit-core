//! Target stubs for cross-platform compilation.
//!
//! This module provides stub implementations for platform-specific APIs,
//! allowing code to compile for targets even when the actual implementation
//! isn't available.

pub mod web;
pub mod ios;
pub mod android;

use crate::target::{Platform, Target};

/// Check if stubs should be used for the current target.
pub fn should_use_stubs() -> bool {
    let target = Target::current();
    !matches!(
        target.platform(),
        Platform::MacOS | Platform::Windows | Platform::Linux
    )
}

/// Trait for stub implementations.
pub trait Stub {
    /// The name of the feature this stub provides.
    const FEATURE_NAME: &'static str;

    /// Check if the real implementation is available.
    fn is_available() -> bool;

    /// Get a stub-specific error message.
    fn not_available_message() -> String {
        format!(
            "Feature '{}' is not available on this platform. Using stub implementation.",
            Self::FEATURE_NAME
        )
    }
}

/// Result type for stubbed operations.
pub type StubResult<T> = Result<T, StubError>;

/// Error for stub operations.
#[derive(Debug, Clone)]
pub struct StubError {
    /// Feature that is stubbed
    pub feature: String,
    /// Message describing the limitation
    pub message: String,
}

impl std::fmt::Display for StubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.feature, self.message)
    }
}

impl std::error::Error for StubError {}

impl StubError {
    /// Create a new stub error.
    pub fn new(feature: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            feature: feature.into(),
            message: message.into(),
        }
    }

    /// Create an "unavailable" error.
    pub fn unavailable(feature: impl Into<String>) -> Self {
        let feature = feature.into();
        Self {
            message: format!("{} is not available on this platform", feature),
            feature,
        }
    }
}

/// Macro to create a stubbed function that returns an error on unsupported platforms.
#[macro_export]
macro_rules! stub_fn {
    ($feature:expr, $fn_name:ident($($arg:ident : $arg_ty:ty),*) -> $ret:ty) => {
        pub fn $fn_name($($arg: $arg_ty),*) -> $crate::stubs::StubResult<$ret> {
            Err($crate::stubs::StubError::unavailable($feature))
        }
    };
}

/// Macro to create a stubbed function that returns a default value on unsupported platforms.
#[macro_export]
macro_rules! stub_fn_default {
    ($fn_name:ident($($arg:ident : $arg_ty:ty),*) -> $ret:ty, $default:expr) => {
        pub fn $fn_name($($arg: $arg_ty),*) -> $ret {
            $default
        }
    };
}
