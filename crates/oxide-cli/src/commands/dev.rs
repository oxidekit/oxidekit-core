//! `oxide dev` command - development server with hot reload
//!
//! Starts a development server that watches for file changes and provides
//! hot reload capabilities for .oui files with state preservation.

use anyhow::Result;
use oxide_devtools::hot_reload::{
    HotReloadConfig, HotReloadEvent, HotReloadRuntime,
    WatcherConfig, DevServerConfig, OverlayConfig,
};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// File change event types (for backward compatibility)
#[derive(Debug, Clone)]
pub enum FileChange {
    /// UI file changed (.oui)
    Ui(String),
    /// Source file changed (.rs)
    Source(String),
    /// Config file changed (oxide.toml)
    Config,
}

/// Development server configuration
#[derive(Debug, Clone)]
pub struct DevServerOptions {
    /// Port for the WebSocket server
    pub port: u16,
    /// Whether to open the browser automatically
    pub open: bool,
    /// Whether to enable hot reload
    pub hot_reload: bool,
    /// Whether to show verbose output
    pub verbose: bool,
}

impl Default for DevServerOptions {
    fn default() -> Self {
        Self {
            port: 9876,
            open: false,
            hot_reload: true,
            verbose: false,
        }
    }
}

/// Run the development server with hot reload
pub fn run(port: u16, _open: bool) -> Result<()> {
    let options = DevServerOptions {
        port,
        ..Default::default()
    };
    run_with_options(options)
}

/// Run the development server with full options
pub fn run_with_options(options: DevServerOptions) -> Result<()> {
    let manifest_path = Path::new("oxide.toml");

    if !manifest_path.exists() {
        anyhow::bail!(
            "No oxide.toml found in current directory. Run `oxide new <name>` to create a project."
        );
    }

    // Print startup banner
    print_banner(&options);

    // Create runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    ctrlc::set_handler(move || {
        println!();
        println!("  Shutting down...");
        running_clone.store(false, Ordering::SeqCst);
    })?;

    // Run the dev server
    rt.block_on(async {
        run_async(manifest_path, options, running).await
    })
}

/// Print the startup banner
fn print_banner(options: &DevServerOptions) {
    println!();
    println!("  \x1b[1;36mOxideKit Development Server\x1b[0m");
    println!("  ──────────────────────────────");
    println!();

    if options.hot_reload {
        println!("  \x1b[32m●\x1b[0m Hot reload:  \x1b[1menabled\x1b[0m");
        println!("  \x1b[32m●\x1b[0m WebSocket:   \x1b[1mws://127.0.0.1:{}\x1b[0m", options.port);
    } else {
        println!("  \x1b[33m●\x1b[0m Hot reload:  \x1b[1mdisabled\x1b[0m");
    }

    println!("  \x1b[32m●\x1b[0m Watching:    \x1b[1mui/*.oui, src/*.rs\x1b[0m");
    println!("  \x1b[32m●\x1b[0m Inspector:   \x1b[1mpress F12\x1b[0m");
    println!();
    println!("  Press \x1b[1mCtrl+C\x1b[0m to stop");
    println!();
}

/// Run the development server asynchronously
async fn run_async(
    manifest_path: &Path,
    options: DevServerOptions,
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Get the project directory
    let project_path = manifest_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // Configure hot reload
    let config = HotReloadConfig {
        enabled: options.hot_reload,
        auto_start_server: options.hot_reload,
        watcher: WatcherConfig {
            debounce_ms: 150,
            extensions: vec!["oui".to_string(), "rs".to_string(), "toml".to_string()],
            ignore_dirs: vec![
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                ".oxide".to_string(),
            ],
            recursive: true,
        },
        server: DevServerConfig {
            port: options.port,
            host: "127.0.0.1".to_string(),
            max_clients: 10,
            ping_interval_secs: 30,
            verbose: options.verbose,
        },
        overlay: OverlayConfig::default(),
        ..Default::default()
    };

    // Create and start the hot reload runtime
    let runtime = HotReloadRuntime::new(config);
    let handle = runtime.start(&project_path).await?;

    // Subscribe to events
    let event_rx = handle.subscribe();

    // Event processing loop
    let event_loop = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv() {
            handle_event(&event);
        }
    });

    // Wait for the application to run
    // In production, this would integrate with the oxide-runtime window
    // For now, we run the app in a blocking fashion

    // Check if we have valid IR to run
    if let Some(_ir) = handle.latest_ir() {
        tracing::info!("Running with compiled UI");
    } else {
        tracing::warn!("No valid UI compiled, using demo UI");
    }

    // Run the application
    let app = oxide_runtime::Application::from_manifest(manifest_path)?;

    // Spawn a task to check for shutdown
    let running_check = Arc::clone(&running);
    tokio::spawn(async move {
        while running_check.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        // Request stop
        let _ = handle.request_stop().await;
    });

    // This blocks until the window is closed
    app.run()?;

    // Clean up
    running.store(false, Ordering::SeqCst);
    event_loop.abort();

    println!("  Development server stopped.");
    println!();

    Ok(())
}

/// Handle a hot reload event
fn handle_event(event: &HotReloadEvent) {
    match event {
        HotReloadEvent::FileChanged { path, kind } => {
            let kind_str = match kind {
                oxide_devtools::hot_reload::FileChangeKind::Ui => "UI",
                oxide_devtools::hot_reload::FileChangeKind::Source => "Source",
                oxide_devtools::hot_reload::FileChangeKind::Config => "Config",
                oxide_devtools::hot_reload::FileChangeKind::Asset => "Asset",
                oxide_devtools::hot_reload::FileChangeKind::Unknown => "File",
            };
            println!("  \x1b[33m→\x1b[0m {} changed: {}", kind_str, path.display());
        }

        HotReloadEvent::CompileStarted { files } => {
            let count = files.len();
            if count == 1 {
                println!("  \x1b[36m⟳\x1b[0m Compiling...");
            } else {
                println!("  \x1b[36m⟳\x1b[0m Compiling {} files...", count);
            }
        }

        HotReloadEvent::CompileSuccess { duration_ms, changed_components } => {
            println!(
                "  \x1b[32m✓\x1b[0m Compiled in {}ms ({} component{})",
                duration_ms,
                changed_components.len(),
                if changed_components.len() == 1 { "" } else { "s" }
            );
        }

        HotReloadEvent::CompileError { file, errors } => {
            println!("  \x1b[31m✗\x1b[0m Compilation failed: {}", file.display());
            for error in errors {
                println!(
                    "     \x1b[31mLine {}:{}\x1b[0m: {}",
                    error.line, error.column, error.message
                );
            }
        }

        HotReloadEvent::StateSnapshotted { component_count, size_bytes } => {
            if *size_bytes > 0 {
                tracing::debug!(
                    "State captured: {} components, {} bytes",
                    component_count,
                    size_bytes
                );
            }
        }

        HotReloadEvent::StateRestored { restored_count, skipped_count } => {
            if *restored_count > 0 || *skipped_count > 0 {
                println!(
                    "  \x1b[32m↻\x1b[0m State restored ({} components, {} skipped)",
                    restored_count, skipped_count
                );
            }
        }

        HotReloadEvent::UiUpdated { root_component } => {
            println!("  \x1b[32m●\x1b[0m UI updated: {}", root_component);
        }

        HotReloadEvent::FullReloadRequired { reason } => {
            println!();
            println!("  \x1b[33m⚠\x1b[0m  Full reload required: {}", reason);
            println!("     Restart the dev server to apply changes.");
            println!();
        }

        HotReloadEvent::ServerStarted { port } => {
            tracing::debug!("WebSocket server started on port {}", port);
        }

        HotReloadEvent::ClientConnected { client_id } => {
            println!("  \x1b[32m+\x1b[0m Client connected: {}", &client_id[..8]);
        }

        HotReloadEvent::ClientDisconnected { client_id } => {
            println!("  \x1b[31m-\x1b[0m Client disconnected: {}", &client_id[..8]);
        }

        HotReloadEvent::ErrorOverlayShown { error_count } => {
            tracing::debug!("Error overlay shown with {} errors", error_count);
        }

        HotReloadEvent::ErrorOverlayDismissed => {
            tracing::debug!("Error overlay dismissed");
        }

        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_server_options_default() {
        let options = DevServerOptions::default();
        assert_eq!(options.port, 9876);
        assert!(options.hot_reload);
        assert!(!options.open);
    }
}
