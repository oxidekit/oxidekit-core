//! `oxide run` command - run the application

use anyhow::Result;
use std::path::Path;

pub fn run(release: bool) -> Result<()> {
    let manifest_path = Path::new("oxide.toml");

    if !manifest_path.exists() {
        anyhow::bail!(
            "No oxide.toml found in current directory. Run `oxide new <name>` to create a project."
        );
    }

    if release {
        tracing::info!("Running application (release mode)...");
    } else {
        tracing::info!("Running application...");
    }

    let app = oxide_runtime::Application::from_manifest(manifest_path)?;
    app.run()?;

    Ok(())
}
