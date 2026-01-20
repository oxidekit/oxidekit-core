//! Hello OxideKit - A minimal demonstration application
//!
//! This example shows the basic structure of an OxideKit application.

use oxide_runtime::Application;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load the application from the manifest
    let app = Application::from_manifest("examples/hello-oxidekit/oxide.toml")?;

    // Run the application
    app.run()
}
