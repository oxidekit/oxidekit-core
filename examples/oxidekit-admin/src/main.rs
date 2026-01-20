//! OxideKit Admin - Golden Reference App
//!
//! This application validates the OxideKit platform by exercising:
//! - UI rendering (sidebar, tables, forms, charts)
//! - Backend integration (contract-first API client)
//! - Plugin system (ui.data, ui.forms, native.fs)
//! - Permissions (network allowlist, filesystem access)
//! - Diagnostics (crash reporting, performance metrics)
//! - Theming (light/dark mode toggle)
//! - Hot reload (state preservation across changes)

use oxide_runtime::Application;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("OxideKit Admin - Golden Reference App");
    println!("=====================================");
    println!("Starting windowed application...");
    println!();

    // Load the application from the manifest
    let app = Application::from_manifest("oxide.toml")?;

    // Run the application (opens window)
    app.run()
}
