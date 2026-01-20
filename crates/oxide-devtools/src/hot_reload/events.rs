//! Event system for hot reload communication
//!
//! Provides an event bus for coordinating hot reload activities between components.

use crossbeam_channel::{bounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

/// Events that can occur during hot reload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HotReloadEvent {
    /// A file has changed
    FileChanged {
        path: PathBuf,
        kind: FileChangeKind,
    },

    /// Compilation started
    CompileStarted {
        files: Vec<PathBuf>,
    },

    /// Compilation completed successfully
    CompileSuccess {
        duration_ms: u64,
        changed_components: Vec<String>,
    },

    /// Compilation failed
    CompileError {
        file: PathBuf,
        errors: Vec<CompileErrorInfo>,
    },

    /// State snapshot captured
    StateSnapshotted {
        component_count: usize,
        size_bytes: usize,
    },

    /// State restoration started
    StateRestoring,

    /// State restoration completed
    StateRestored {
        restored_count: usize,
        skipped_count: usize,
    },

    /// UI tree updated
    UiUpdated {
        root_component: String,
    },

    /// Full reload required (e.g., Rust code changed)
    FullReloadRequired {
        reason: String,
    },

    /// Dev server started
    ServerStarted {
        port: u16,
    },

    /// Client connected to dev server
    ClientConnected {
        client_id: String,
    },

    /// Client disconnected from dev server
    ClientDisconnected {
        client_id: String,
    },

    /// Error overlay shown
    ErrorOverlayShown {
        error_count: usize,
    },

    /// Error overlay dismissed
    ErrorOverlayDismissed,
}

/// Types of file changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeKind {
    /// .oui file changed (hot reloadable)
    Ui,
    /// Rust source changed (requires restart)
    Source,
    /// oxide.toml changed (requires restart)
    Config,
    /// Asset file changed (may be hot reloadable)
    Asset,
    /// Unknown file type
    Unknown,
}

impl FileChangeKind {
    /// Determine the file change kind from a path
    pub fn from_path(path: &std::path::Path) -> Self {
        let extension = path.extension().and_then(|e| e.to_str());
        let filename = path.file_name().and_then(|n| n.to_str());

        match extension {
            Some("oui") => Self::Ui,
            Some("rs") => Self::Source,
            Some("png" | "jpg" | "jpeg" | "svg" | "gif" | "webp") => Self::Asset,
            Some("ttf" | "otf" | "woff" | "woff2") => Self::Asset,
            _ => {
                if filename == Some("oxide.toml") {
                    Self::Config
                } else {
                    Self::Unknown
                }
            }
        }
    }

    /// Whether this change kind supports hot reload
    pub fn is_hot_reloadable(&self) -> bool {
        matches!(self, Self::Ui | Self::Asset)
    }
}

/// Information about a compilation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileErrorInfo {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: ErrorSeverity,
    pub code: Option<String>,
}

/// Severity level of an error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Event bus for publishing and subscribing to hot reload events
pub struct EventBus {
    sender: Sender<HotReloadEvent>,
    receiver: Receiver<HotReloadEvent>,
    subscribers: Arc<RwLock<Vec<Sender<HotReloadEvent>>>>,
}

impl EventBus {
    /// Create a new event bus with the specified buffer size
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = bounded(buffer_size);
        Self {
            sender,
            receiver,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new event bus with default buffer size (100)
    pub fn with_default_buffer() -> Self {
        Self::new(100)
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: HotReloadEvent) {
        // Send to main receiver
        let _ = self.sender.try_send(event.clone());

        // Send to all subscribers
        let subscribers = self.subscribers.read();
        for subscriber in subscribers.iter() {
            let _ = subscriber.try_send(event.clone());
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> Receiver<HotReloadEvent> {
        let (tx, rx) = bounded(100);
        self.subscribers.write().push(tx);
        rx
    }

    /// Get the main event receiver
    pub fn receiver(&self) -> &Receiver<HotReloadEvent> {
        &self.receiver
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&self) -> Option<HotReloadEvent> {
        self.receiver.try_recv().ok()
    }

    /// Receive an event, blocking until one is available
    pub fn recv(&self) -> Result<HotReloadEvent, crossbeam_channel::RecvError> {
        self.receiver.recv()
    }

    /// Create a sender handle for publishing events
    pub fn sender(&self) -> EventSender {
        EventSender {
            sender: self.sender.clone(),
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::with_default_buffer()
    }
}

/// A cloneable sender handle for the event bus
#[derive(Clone)]
pub struct EventSender {
    sender: Sender<HotReloadEvent>,
    subscribers: Arc<RwLock<Vec<Sender<HotReloadEvent>>>>,
}

impl EventSender {
    /// Publish an event
    pub fn publish(&self, event: HotReloadEvent) {
        let _ = self.sender.try_send(event.clone());

        let subscribers = self.subscribers.read();
        for subscriber in subscribers.iter() {
            let _ = subscriber.try_send(event.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_file_change_kind_from_path() {
        assert_eq!(
            FileChangeKind::from_path(Path::new("ui/app.oui")),
            FileChangeKind::Ui
        );
        assert_eq!(
            FileChangeKind::from_path(Path::new("src/main.rs")),
            FileChangeKind::Source
        );
        assert_eq!(
            FileChangeKind::from_path(Path::new("oxide.toml")),
            FileChangeKind::Config
        );
        assert_eq!(
            FileChangeKind::from_path(Path::new("assets/logo.png")),
            FileChangeKind::Asset
        );
    }

    #[test]
    fn test_event_bus_publish_subscribe() {
        let bus = EventBus::with_default_buffer();
        let subscriber = bus.subscribe();

        bus.publish(HotReloadEvent::ServerStarted { port: 9876 });

        let event = subscriber.try_recv().unwrap();
        if let HotReloadEvent::ServerStarted { port } = event {
            assert_eq!(port, 9876);
        } else {
            panic!("Unexpected event type");
        }
    }
}
