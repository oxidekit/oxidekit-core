//! Hot Reload System for OxideKit
//!
//! Provides live reloading of .oui files and state preservation during development.
//!
//! # Architecture
//!
//! The hot reload system consists of several components:
//!
//! - **FileWatcher**: Monitors file changes with debouncing
//! - **StateManager**: Captures and restores application state
//! - **IncrementalCompiler**: Recompiles only changed .oui files
//! - **DevServer**: WebSocket server for browser/runtime communication
//! - **ErrorOverlay**: Displays compilation errors in the UI
//!
//! # Usage
//!
//! ```rust,ignore
//! use oxide_devtools::hot_reload::{HotReloadRuntime, HotReloadConfig};
//!
//! let config = HotReloadConfig::default();
//! let runtime = HotReloadRuntime::new(config)?;
//! runtime.start(project_path)?;
//! ```

mod watcher;
mod state;
mod compiler;
mod server;
mod overlay;
mod events;
mod runtime;

#[cfg(test)]
mod tests;

pub use watcher::{FileWatcher, WatchEvent, WatchEventKind, WatcherConfig};
pub use state::{StateManager, StateSnapshot, StateDiff};
pub use compiler::{IncrementalCompiler, CompileResult, CompileError};
pub use server::{DevServer, DevServerConfig, ClientMessage, ServerMessage};
pub use overlay::{ErrorOverlay, OverlayConfig, DiagnosticDisplay};
pub use events::{HotReloadEvent, EventBus, FileChangeKind};
pub use runtime::{HotReloadRuntime, HotReloadConfig, HotReloadHandle};

/// Version of the hot reload protocol
pub const PROTOCOL_VERSION: u32 = 1;

/// Default WebSocket port for the dev server
pub const DEFAULT_WS_PORT: u16 = 9876;

/// Default debounce duration in milliseconds
pub const DEFAULT_DEBOUNCE_MS: u64 = 150;
