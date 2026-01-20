//! HTTP client abstraction for OxideKit.
//!
//! Provides a unified HTTP client with:
//! - Request/response interceptors
//! - Automatic retry logic
//! - Auth integration
//! - Network allowlist enforcement
//! - Offline detection

mod client;
mod request;
mod response;

pub use client::*;
pub use request::*;
pub use response::*;
