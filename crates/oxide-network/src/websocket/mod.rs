//! WebSocket support for OxideKit.
//!
//! Provides a unified WebSocket client with:
//! - Automatic reconnection
//! - Message queuing during disconnection
//! - Heartbeat/ping-pong support
//! - Auth integration

mod client;
mod message;

pub use client::*;
pub use message::*;
