//! Hot Reload Runtime
//!
//! Coordinates all hot reload components into a unified runtime.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use thiserror::Error;

use super::compiler::{IncrementalCompiler, CompileError, CompilerConfig};
use super::events::{EventBus, EventSender, FileChangeKind, HotReloadEvent, CompileErrorInfo, ErrorSeverity};
use super::overlay::{ErrorOverlay, OverlayConfig, DiagnosticDisplay};
use super::server::{DevServer, DevServerConfig, DevServerEventHandler};
use super::state::{StateManager, StateSnapshot};
use super::watcher::{FileWatcher, WatcherConfig, WatchError};
use oxide_compiler::ComponentIR;

/// Errors that can occur in the hot reload runtime
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Failed to start file watcher: {0}")]
    WatcherError(#[from] WatchError),

    #[error("Compilation error: {0}")]
    CompileError(#[from] CompileError),

    #[error("Server error: {0}")]
    ServerError(#[from] super::server::ServerError),

    #[error("Project not found: {0}")]
    ProjectNotFound(PathBuf),

    #[error("Runtime already started")]
    AlreadyStarted,

    #[error("Runtime not started")]
    NotStarted,
}

/// Configuration for the hot reload runtime
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// File watcher configuration
    pub watcher: WatcherConfig,
    /// Dev server configuration
    pub server: DevServerConfig,
    /// Compiler configuration
    pub compiler: CompilerConfig,
    /// Error overlay configuration
    pub overlay: OverlayConfig,
    /// Whether hot reload is enabled
    pub enabled: bool,
    /// Whether to automatically start the dev server
    pub auto_start_server: bool,
    /// Paths to watch (relative to project root)
    pub watch_paths: Vec<String>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            watcher: WatcherConfig::default(),
            server: DevServerConfig::default(),
            compiler: CompilerConfig::default(),
            overlay: OverlayConfig::default(),
            enabled: true,
            auto_start_server: true,
            watch_paths: vec!["ui".to_string(), "src".to_string()],
        }
    }
}

/// Current state of the hot reload runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Runtime is stopped
    Stopped,
    /// Runtime is starting
    Starting,
    /// Runtime is running
    Running,
    /// Runtime is compiling
    Compiling,
    /// Runtime has errors
    Error,
    /// Runtime is stopping
    Stopping,
}

/// Handle for controlling the hot reload runtime
pub struct HotReloadHandle {
    /// Channel for sending commands to the runtime
    command_tx: mpsc::Sender<RuntimeCommand>,
    /// Current runtime state
    state: Arc<RwLock<RuntimeState>>,
    /// Latest compiled IR
    latest_ir: Arc<RwLock<Option<ComponentIR>>>,
    /// State manager for accessing state
    state_manager: Arc<StateManager>,
    /// Error overlay for accessing errors
    error_overlay: Arc<ErrorOverlay>,
    /// Event bus for subscribing to events
    event_bus: Arc<EventBus>,
}

impl HotReloadHandle {
    /// Get the current runtime state
    pub fn state(&self) -> RuntimeState {
        *self.state.read()
    }

    /// Get the latest compiled IR
    pub fn latest_ir(&self) -> Option<ComponentIR> {
        self.latest_ir.read().clone()
    }

    /// Request a manual reload
    pub async fn request_reload(&self) -> Result<(), RuntimeError> {
        self.command_tx
            .send(RuntimeCommand::Reload)
            .await
            .map_err(|_| RuntimeError::NotStarted)
    }

    /// Request the runtime to stop
    pub async fn request_stop(&self) -> Result<(), RuntimeError> {
        self.command_tx
            .send(RuntimeCommand::Stop)
            .await
            .map_err(|_| RuntimeError::NotStarted)
    }

    /// Capture current state
    pub fn capture_state(&self) -> StateSnapshot {
        self.state_manager.capture()
    }

    /// Get the error overlay
    pub fn error_overlay(&self) -> &Arc<ErrorOverlay> {
        &self.error_overlay
    }

    /// Subscribe to hot reload events
    pub fn subscribe(&self) -> crossbeam_channel::Receiver<HotReloadEvent> {
        self.event_bus.subscribe()
    }

    /// Check if there are compilation errors
    pub fn has_errors(&self) -> bool {
        self.error_overlay.error_count() > 0
    }
}

/// Commands that can be sent to the runtime
#[derive(Debug)]
enum RuntimeCommand {
    /// Request a reload
    Reload,
    /// Stop the runtime
    Stop,
    /// Compile a specific file
    CompileFile(PathBuf),
}

/// The hot reload runtime
pub struct HotReloadRuntime {
    config: HotReloadConfig,
    project_path: PathBuf,
    state: Arc<RwLock<RuntimeState>>,
    compiler: Arc<IncrementalCompiler>,
    state_manager: Arc<StateManager>,
    error_overlay: Arc<ErrorOverlay>,
    event_bus: Arc<EventBus>,
    latest_ir: Arc<RwLock<Option<ComponentIR>>>,
}

impl HotReloadRuntime {
    /// Create a new hot reload runtime
    pub fn new(config: HotReloadConfig) -> Self {
        let compiler_config = CompilerConfig {
            base_dir: PathBuf::from("."),
            ..config.compiler.clone()
        };

        Self {
            config,
            project_path: PathBuf::new(),
            state: Arc::new(RwLock::new(RuntimeState::Stopped)),
            compiler: Arc::new(IncrementalCompiler::new(compiler_config)),
            state_manager: Arc::new(StateManager::new()),
            error_overlay: Arc::new(ErrorOverlay::with_defaults()),
            event_bus: Arc::new(EventBus::with_default_buffer()),
            latest_ir: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new hot reload runtime with default configuration
    pub fn with_defaults() -> Self {
        Self::new(HotReloadConfig::default())
    }

    /// Start the hot reload runtime
    pub async fn start(mut self, project_path: impl AsRef<Path>) -> Result<HotReloadHandle, RuntimeError> {
        let project_path = project_path.as_ref().to_path_buf();

        if !project_path.exists() {
            return Err(RuntimeError::ProjectNotFound(project_path));
        }

        if *self.state.read() != RuntimeState::Stopped {
            return Err(RuntimeError::AlreadyStarted);
        }

        *self.state.write() = RuntimeState::Starting;
        self.project_path = project_path.clone();

        // Update compiler base dir
        self.compiler = Arc::new(IncrementalCompiler::new(CompilerConfig {
            base_dir: project_path.clone(),
            ..self.config.compiler.clone()
        }));

        // Create command channel
        let (command_tx, mut command_rx) = mpsc::channel::<RuntimeCommand>(32);

        // Create the handle
        let handle = HotReloadHandle {
            command_tx,
            state: Arc::clone(&self.state),
            latest_ir: Arc::clone(&self.latest_ir),
            state_manager: Arc::clone(&self.state_manager),
            error_overlay: Arc::clone(&self.error_overlay),
            event_bus: Arc::clone(&self.event_bus),
        };

        // Create file watcher
        let mut watcher = FileWatcher::new(self.config.watcher.clone())?;
        watcher.set_event_sender(self.event_bus.sender());

        // Watch configured paths
        for watch_path in &self.config.watch_paths {
            let full_path = project_path.join(watch_path);
            if full_path.exists() {
                watcher.watch(&full_path)?;
            }
        }

        // Watch oxide.toml
        let manifest_path = project_path.join("oxide.toml");
        if manifest_path.exists() {
            watcher.watch(&manifest_path)?;
        }

        // Start dev server if configured
        let server = if self.config.auto_start_server {
            let mut server = DevServer::new(self.config.server.clone());
            server.start().await?;
            self.event_bus.publish(HotReloadEvent::ServerStarted {
                port: self.config.server.port,
            });
            Some(Arc::new(server))
        } else {
            None
        };

        // Do initial compilation
        let initial_ir = self.initial_compile(&project_path);
        if let Some(ir) = initial_ir {
            *self.latest_ir.write() = Some(ir);
        }

        *self.state.write() = RuntimeState::Running;

        // Clone what we need for the background task
        let state = Arc::clone(&self.state);
        let compiler = Arc::clone(&self.compiler);
        let state_manager = Arc::clone(&self.state_manager);
        let error_overlay = Arc::clone(&self.error_overlay);
        let event_bus = Arc::clone(&self.event_bus);
        let latest_ir = Arc::clone(&self.latest_ir);
        let project_path_clone = project_path.clone();

        // Spawn the main runtime loop
        tokio::spawn(async move {
            let event_rx = event_bus.subscribe();

            loop {
                // Check for commands (async)
                tokio::select! {
                    biased;

                    Some(cmd) = command_rx.recv() => {
                        match cmd {
                            RuntimeCommand::Stop => {
                                *state.write() = RuntimeState::Stopping;
                                break;
                            }
                            RuntimeCommand::Reload => {
                                Self::handle_reload(
                                    &project_path_clone,
                                    &compiler,
                                    &state_manager,
                                    &error_overlay,
                                    &event_bus,
                                    &latest_ir,
                                    &state,
                                ).await;
                            }
                            RuntimeCommand::CompileFile(path) => {
                                Self::handle_file_compile(
                                    &path,
                                    &compiler,
                                    &error_overlay,
                                    &event_bus,
                                    &latest_ir,
                                    &state,
                                ).await;
                            }
                        }
                    }

                    // Poll for events at regular intervals
                    _ = tokio::time::sleep(Duration::from_millis(50)) => {
                        // Check for file change events (non-blocking)
                        while let Ok(event) = event_rx.try_recv() {
                            if let HotReloadEvent::FileChanged { path, kind } = event {
                                match kind {
                                    FileChangeKind::Ui => {
                                        Self::handle_ui_change(
                                            &path,
                                            &compiler,
                                            &state_manager,
                                            &error_overlay,
                                            &event_bus,
                                            &latest_ir,
                                            &state,
                                            server.as_ref(),
                                        ).await;
                                    }
                                    FileChangeKind::Source | FileChangeKind::Config => {
                                        event_bus.publish(HotReloadEvent::FullReloadRequired {
                                            reason: format!(
                                                "{} changed, restart required",
                                                if kind == FileChangeKind::Source { "Rust source" } else { "Config" }
                                            ),
                                        });
                                        if let Some(server) = &server {
                                            let _ = server.notify_full_reload(
                                                "Source code or config changed".to_string()
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }

            *state.write() = RuntimeState::Stopped;
            tracing::info!("Hot reload runtime stopped");
        });

        tracing::info!("Hot reload runtime started for {}", project_path.display());

        Ok(handle)
    }

    /// Perform initial compilation of the project
    fn initial_compile(&self, project_path: &Path) -> Option<ComponentIR> {
        let ui_path = project_path.join("ui/app.oui");

        if !ui_path.exists() {
            tracing::warn!("No ui/app.oui found, skipping initial compile");
            return None;
        }

        let start = Instant::now();

        match self.compiler.compile_file(&ui_path) {
            Ok(result) => {
                let duration = start.elapsed();
                tracing::info!(
                    "Initial compilation completed in {:?} ({} components)",
                    duration,
                    result.components.len()
                );

                self.event_bus.publish(HotReloadEvent::CompileSuccess {
                    duration_ms: duration.as_millis() as u64,
                    changed_components: result.components,
                });

                Some(result.ir)
            }
            Err(e) => {
                tracing::error!("Initial compilation failed: {}", e);

                let errors = vec![CompileErrorInfo {
                    line: 0,
                    column: 0,
                    message: e.to_string(),
                    severity: ErrorSeverity::Error,
                    code: None,
                }];

                self.event_bus.publish(HotReloadEvent::CompileError {
                    file: ui_path.clone(),
                    errors: errors.clone(),
                });

                let diagnostics = errors
                    .into_iter()
                    .map(|e| DiagnosticDisplay::from_compile_error(ui_path.clone(), e))
                    .collect();

                self.error_overlay.show(diagnostics);

                None
            }
        }
    }

    /// Handle a UI file change
    async fn handle_ui_change(
        path: &Path,
        compiler: &Arc<IncrementalCompiler>,
        state_manager: &Arc<StateManager>,
        error_overlay: &Arc<ErrorOverlay>,
        event_bus: &Arc<EventBus>,
        latest_ir: &Arc<RwLock<Option<ComponentIR>>>,
        state: &Arc<RwLock<RuntimeState>>,
        server: Option<&Arc<DevServer>>,
    ) {
        tracing::debug!("UI file changed: {}", path.display());

        // Set compiling state
        *state.write() = RuntimeState::Compiling;

        // Capture state before reload
        let snapshot = state_manager.capture();
        event_bus.publish(HotReloadEvent::StateSnapshotted {
            component_count: snapshot.component_count(),
            size_bytes: snapshot.size_bytes(),
        });

        event_bus.publish(HotReloadEvent::CompileStarted {
            files: vec![path.to_path_buf()],
        });

        let start = Instant::now();

        // Invalidate cache for this file
        compiler.invalidate(path);

        // Recompile
        match compiler.compile_file(path) {
            Ok(result) => {
                let duration = start.elapsed();

                // Update latest IR
                *latest_ir.write() = Some(result.ir.clone());

                // Clear error overlay
                error_overlay.dismiss();

                // Publish success event
                event_bus.publish(HotReloadEvent::CompileSuccess {
                    duration_ms: duration.as_millis() as u64,
                    changed_components: result.components.clone(),
                });

                // Notify server
                if let Some(server) = server {
                    let _ = server.notify_hot_reload(
                        vec![path.to_string_lossy().to_string()],
                        result.components,
                        duration.as_millis() as u64,
                    );
                }

                // Restore state
                event_bus.publish(HotReloadEvent::StateRestoring);
                if let Ok(diff) = state_manager.restore(snapshot) {
                    event_bus.publish(HotReloadEvent::StateRestored {
                        restored_count: diff.modified.len(),
                        skipped_count: diff.removed.len(),
                    });
                }

                event_bus.publish(HotReloadEvent::UiUpdated {
                    root_component: result.ir.kind,
                });

                *state.write() = RuntimeState::Running;

                tracing::info!(
                    "Hot reload completed in {:?}",
                    duration
                );
            }
            Err(e) => {
                let errors = vec![CompileErrorInfo {
                    line: match &e {
                        CompileError::CompilationFailed { line, .. } => *line,
                        _ => 0,
                    },
                    column: match &e {
                        CompileError::CompilationFailed { column, .. } => *column,
                        _ => 0,
                    },
                    message: e.to_string(),
                    severity: ErrorSeverity::Error,
                    code: None,
                }];

                event_bus.publish(HotReloadEvent::CompileError {
                    file: path.to_path_buf(),
                    errors: errors.clone(),
                });

                // Show error overlay
                let diagnostics = errors
                    .into_iter()
                    .map(|e| DiagnosticDisplay::from_compile_error(path.to_path_buf(), e))
                    .collect();
                error_overlay.show(diagnostics);

                event_bus.publish(HotReloadEvent::ErrorOverlayShown {
                    error_count: error_overlay.error_count(),
                });

                // Notify server
                if let Some(server) = server {
                    let _ = server.notify_compile_error(
                        path.to_string_lossy().to_string(),
                        vec![CompileErrorInfo {
                            line: 0,
                            column: 0,
                            message: e.to_string(),
                            severity: ErrorSeverity::Error,
                            code: None,
                        }],
                    );
                }

                *state.write() = RuntimeState::Error;

                tracing::error!("Hot reload failed: {}", e);
            }
        }
    }

    /// Handle a manual reload request
    async fn handle_reload(
        project_path: &Path,
        compiler: &Arc<IncrementalCompiler>,
        state_manager: &Arc<StateManager>,
        error_overlay: &Arc<ErrorOverlay>,
        event_bus: &Arc<EventBus>,
        latest_ir: &Arc<RwLock<Option<ComponentIR>>>,
        state: &Arc<RwLock<RuntimeState>>,
    ) {
        let ui_path = project_path.join("ui/app.oui");
        if ui_path.exists() {
            Self::handle_ui_change(
                &ui_path,
                compiler,
                state_manager,
                error_overlay,
                event_bus,
                latest_ir,
                state,
                None,
            )
            .await;
        }
    }

    /// Handle compiling a specific file
    async fn handle_file_compile(
        path: &Path,
        compiler: &Arc<IncrementalCompiler>,
        error_overlay: &Arc<ErrorOverlay>,
        event_bus: &Arc<EventBus>,
        latest_ir: &Arc<RwLock<Option<ComponentIR>>>,
        state: &Arc<RwLock<RuntimeState>>,
    ) {
        *state.write() = RuntimeState::Compiling;

        event_bus.publish(HotReloadEvent::CompileStarted {
            files: vec![path.to_path_buf()],
        });

        let start = Instant::now();

        match compiler.compile_file(path) {
            Ok(result) => {
                let duration = start.elapsed();
                *latest_ir.write() = Some(result.ir);

                error_overlay.dismiss();

                event_bus.publish(HotReloadEvent::CompileSuccess {
                    duration_ms: duration.as_millis() as u64,
                    changed_components: result.components,
                });

                *state.write() = RuntimeState::Running;
            }
            Err(e) => {
                let errors = vec![CompileErrorInfo {
                    line: 0,
                    column: 0,
                    message: e.to_string(),
                    severity: ErrorSeverity::Error,
                    code: None,
                }];

                event_bus.publish(HotReloadEvent::CompileError {
                    file: path.to_path_buf(),
                    errors: errors.clone(),
                });

                let diagnostics = errors
                    .into_iter()
                    .map(|e| DiagnosticDisplay::from_compile_error(path.to_path_buf(), e))
                    .collect();
                error_overlay.show(diagnostics);

                *state.write() = RuntimeState::Error;
            }
        }
    }
}

impl Default for HotReloadRuntime {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_config_default() {
        let config = HotReloadConfig::default();
        assert!(config.enabled);
        assert!(config.auto_start_server);
        assert!(config.watch_paths.contains(&"ui".to_string()));
    }

    #[test]
    fn test_runtime_state() {
        assert_eq!(RuntimeState::Stopped, RuntimeState::Stopped);
        assert_ne!(RuntimeState::Running, RuntimeState::Stopped);
    }
}
