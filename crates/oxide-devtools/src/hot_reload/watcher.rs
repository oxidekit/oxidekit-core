//! File watcher with debouncing for hot reload
//!
//! Monitors project directories for file changes and emits debounced events.

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use crossbeam_channel::{bounded, Receiver, Sender};
use thiserror::Error;

use super::events::{FileChangeKind, HotReloadEvent, EventSender};
use super::DEFAULT_DEBOUNCE_MS;

/// Errors that can occur during file watching
#[derive(Debug, Error)]
pub enum WatchError {
    #[error("Failed to create watcher: {0}")]
    CreateWatcher(#[from] notify::Error),

    #[error("Failed to watch path {path}: {source}")]
    WatchPath {
        path: PathBuf,
        #[source]
        source: notify::Error,
    },

    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),
}

/// A file watch event
#[derive(Debug, Clone)]
pub struct WatchEvent {
    /// The path that changed
    pub path: PathBuf,
    /// The kind of change
    pub kind: WatchEventKind,
    /// The file type classification
    pub file_kind: FileChangeKind,
    /// When the change was detected
    pub timestamp: Instant,
}

/// The kind of file watch event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchEventKind {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
}

impl WatchEventKind {
    fn from_notify_kind(kind: &EventKind) -> Option<Self> {
        match kind {
            EventKind::Create(_) => Some(Self::Created),
            EventKind::Modify(_) => Some(Self::Modified),
            EventKind::Remove(_) => Some(Self::Deleted),
            _ => None,
        }
    }
}

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Debounce duration for coalescing rapid changes
    pub debounce_ms: u64,
    /// File extensions to watch (empty = all files)
    pub extensions: Vec<String>,
    /// Directories to ignore
    pub ignore_dirs: Vec<String>,
    /// Whether to watch recursively
    pub recursive: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            extensions: vec![
                "oui".to_string(),
                "rs".to_string(),
                "toml".to_string(),
            ],
            ignore_dirs: vec![
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                ".oxide".to_string(),
            ],
            recursive: true,
        }
    }
}

/// File watcher with debouncing support
pub struct FileWatcher {
    config: WatcherConfig,
    watcher: RecommendedWatcher,
    pending_events: Arc<Mutex<HashMap<PathBuf, (WatchEvent, Instant)>>>,
    event_sender: Sender<WatchEvent>,
    event_receiver: Receiver<WatchEvent>,
    hot_reload_sender: Option<EventSender>,
}

impl FileWatcher {
    /// Create a new file watcher with the given configuration
    pub fn new(config: WatcherConfig) -> Result<Self, WatchError> {
        let (event_sender, event_receiver) = bounded(100);
        let pending_events = Arc::new(Mutex::new(HashMap::new()));

        let pending_clone = Arc::clone(&pending_events);
        let sender_clone = event_sender.clone();
        let debounce_duration = Duration::from_millis(config.debounce_ms);
        let config_clone = config.clone();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    Self::handle_notify_event(
                        event,
                        &pending_clone,
                        &sender_clone,
                        debounce_duration,
                        &config_clone,
                    );
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(100)),
        )?;

        Ok(Self {
            config,
            watcher,
            pending_events,
            event_sender,
            event_receiver,
            hot_reload_sender: None,
        })
    }

    /// Create a new file watcher with default configuration
    pub fn with_defaults() -> Result<Self, WatchError> {
        Self::new(WatcherConfig::default())
    }

    /// Set the event sender for hot reload events
    pub fn set_event_sender(&mut self, sender: EventSender) {
        self.hot_reload_sender = Some(sender);
    }

    /// Watch a directory for changes
    pub fn watch(&mut self, path: impl AsRef<Path>) -> Result<(), WatchError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(WatchError::PathNotFound(path.to_path_buf()));
        }

        let mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        self.watcher.watch(path, mode).map_err(|e| WatchError::WatchPath {
            path: path.to_path_buf(),
            source: e,
        })?;

        tracing::debug!("Watching path: {}", path.display());
        Ok(())
    }

    /// Stop watching a directory
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<(), WatchError> {
        let path = path.as_ref();
        self.watcher.unwatch(path).map_err(|e| WatchError::WatchPath {
            path: path.to_path_buf(),
            source: e,
        })?;
        tracing::debug!("Stopped watching path: {}", path.display());
        Ok(())
    }

    /// Get the event receiver for consuming watch events
    pub fn receiver(&self) -> &Receiver<WatchEvent> {
        &self.event_receiver
    }

    /// Try to receive a watch event without blocking
    pub fn try_recv(&self) -> Option<WatchEvent> {
        self.event_receiver.try_recv().ok()
    }

    /// Flush any pending debounced events
    pub fn flush_pending(&self) {
        let mut pending = self.pending_events.lock();
        let now = Instant::now();
        let debounce_duration = Duration::from_millis(self.config.debounce_ms);

        let ready: Vec<_> = pending
            .iter()
            .filter(|(_, (_, time))| now.duration_since(*time) >= debounce_duration)
            .map(|(path, (event, _))| (path.clone(), event.clone()))
            .collect();

        for (path, event) in ready {
            pending.remove(&path);
            let _ = self.event_sender.try_send(event.clone());

            // Also publish to hot reload event bus if configured
            if let Some(sender) = &self.hot_reload_sender {
                sender.publish(HotReloadEvent::FileChanged {
                    path: event.path,
                    kind: event.file_kind,
                });
            }
        }
    }

    /// Handle a notify event
    fn handle_notify_event(
        event: Event,
        pending: &Arc<Mutex<HashMap<PathBuf, (WatchEvent, Instant)>>>,
        sender: &Sender<WatchEvent>,
        debounce_duration: Duration,
        config: &WatcherConfig,
    ) {
        let Some(event_kind) = WatchEventKind::from_notify_kind(&event.kind) else {
            return;
        };

        for path in event.paths {
            // Check if this path should be ignored
            if Self::should_ignore(&path, config) {
                continue;
            }

            // Check extension filter
            if !config.extensions.is_empty() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !config.extensions.iter().any(|e| e == ext) {
                    continue;
                }
            }

            let file_kind = FileChangeKind::from_path(&path);
            let now = Instant::now();

            let watch_event = WatchEvent {
                path: path.clone(),
                kind: event_kind,
                file_kind,
                timestamp: now,
            };

            // Check if we already have a pending event for this path
            let mut pending_guard = pending.lock();
            if let Some((existing, last_time)) = pending_guard.get(&path) {
                // If enough time has passed, send the existing event
                if now.duration_since(*last_time) >= debounce_duration {
                    let _ = sender.try_send(existing.clone());
                    pending_guard.remove(&path);
                }
            }

            // Update or insert the pending event
            pending_guard.insert(path, (watch_event, now));
        }

        // Check for events that are ready to be sent
        let now = Instant::now();
        let mut pending_guard = pending.lock();
        let ready: Vec<PathBuf> = pending_guard
            .iter()
            .filter(|(_, (_, time))| now.duration_since(*time) >= debounce_duration)
            .map(|(path, _)| path.clone())
            .collect();

        for path in ready {
            if let Some((event, _)) = pending_guard.remove(&path) {
                let _ = sender.try_send(event);
            }
        }
    }

    /// Check if a path should be ignored
    fn should_ignore(path: &Path, config: &WatcherConfig) -> bool {
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    if config.ignore_dirs.iter().any(|d| d == name_str) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Background task for processing debounced events
pub async fn debounce_task(
    pending: Arc<Mutex<HashMap<PathBuf, (WatchEvent, Instant)>>>,
    sender: Sender<WatchEvent>,
    debounce_duration: Duration,
    hot_reload_sender: Option<EventSender>,
) {
    let check_interval = Duration::from_millis(50);

    loop {
        tokio::time::sleep(check_interval).await;

        let now = Instant::now();
        let mut pending_guard = pending.lock();

        let ready: Vec<(PathBuf, WatchEvent)> = pending_guard
            .iter()
            .filter(|(_, (_, time))| now.duration_since(*time) >= debounce_duration)
            .map(|(path, (event, _))| (path.clone(), event.clone()))
            .collect();

        for (path, event) in ready {
            pending_guard.remove(&path);
            let _ = sender.try_send(event.clone());

            // Publish to hot reload event bus
            if let Some(hr_sender) = &hot_reload_sender {
                hr_sender.publish(HotReloadEvent::FileChanged {
                    path: event.path,
                    kind: event.file_kind,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert_eq!(config.debounce_ms, DEFAULT_DEBOUNCE_MS);
        assert!(config.extensions.contains(&"oui".to_string()));
        assert!(config.extensions.contains(&"rs".to_string()));
        assert!(config.ignore_dirs.contains(&"target".to_string()));
    }

    #[test]
    fn test_should_ignore() {
        let config = WatcherConfig::default();

        assert!(FileWatcher::should_ignore(
            Path::new("project/target/debug/main"),
            &config
        ));
        assert!(FileWatcher::should_ignore(
            Path::new("project/.git/HEAD"),
            &config
        ));
        assert!(!FileWatcher::should_ignore(
            Path::new("project/src/main.rs"),
            &config
        ));
    }

    #[test]
    fn test_watch_event_kind() {
        assert_eq!(
            WatchEventKind::from_notify_kind(&EventKind::Create(notify::event::CreateKind::File)),
            Some(WatchEventKind::Created)
        );
        assert_eq!(
            WatchEventKind::from_notify_kind(&EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content
            ))),
            Some(WatchEventKind::Modified)
        );
    }
}
