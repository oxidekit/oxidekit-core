//! Plugin manifest schema and parsing.
//!
//! Every OxideKit plugin must have a `plugin.toml` manifest file that describes
//! the plugin's metadata, capabilities, dependencies, and configuration.

mod category;
mod schema;
mod validation;

pub use category::{PluginCategory, PluginKindConfig};
pub use schema::PluginManifest;
pub use validation::ManifestValidator;

use std::path::Path;
use crate::error::{PluginError, PluginResult};

/// Load a plugin manifest from a file.
pub fn load_manifest<P: AsRef<Path>>(path: P) -> PluginResult<PluginManifest> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(PluginError::ManifestNotFound(path.to_path_buf()));
    }

    let content = std::fs::read_to_string(path)?;
    let manifest: PluginManifest = toml::from_str(&content)?;

    // Validate the manifest
    let validator = ManifestValidator::new();
    validator.validate(&manifest)?;

    Ok(manifest)
}

/// Save a plugin manifest to a file.
pub fn save_manifest<P: AsRef<Path>>(path: P, manifest: &PluginManifest) -> PluginResult<()> {
    let content = toml::to_string_pretty(manifest)?;
    std::fs::write(path, content)?;
    Ok(())
}
