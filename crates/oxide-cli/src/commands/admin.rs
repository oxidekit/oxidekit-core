//! Admin command - Launch OxideKit Admin Platform

use anyhow::Result;
use tracing::info;

/// Run the admin platform
pub fn run() -> Result<()> {
    info!("Launching OxideKit Admin Platform...");

    // Launch the admin app
    oxide_admin::launch()?;

    Ok(())
}

/// Run admin with a specific config path
pub fn run_with_config(config_path: &str) -> Result<()> {
    info!("Launching OxideKit Admin Platform with config: {}", config_path);

    let path = std::path::Path::new(config_path);
    oxide_admin::launch_with_config(path)?;

    Ok(())
}
