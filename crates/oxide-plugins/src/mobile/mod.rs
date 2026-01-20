//! Mobile plugin framework for OxideKit.
//!
//! This module provides mobile-specific plugin capabilities and backend traits
//! for iOS and Android platforms. Plugins can request mobile capabilities
//! which map to platform-specific permissions.
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_plugins::mobile::{MobileCapability, CameraBackend, PhotoConfig};
//!
//! // Check if a capability requires runtime prompt
//! let cap = MobileCapability::CameraCapture;
//! if cap.requires_runtime_prompt() {
//!     // Request permission from user
//! }
//! ```

pub mod backends;
pub mod capabilities;

// Re-exports
pub use backends::*;
pub use capabilities::*;
