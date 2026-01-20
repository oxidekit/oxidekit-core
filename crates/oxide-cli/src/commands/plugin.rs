//! Plugin management commands.
//!
//! Provides CLI commands for managing OxideKit plugins:
//!
//! - `oxide plugin new` - Create a new plugin
//! - `oxide plugin add` - Install a plugin
//! - `oxide plugin remove` - Uninstall a plugin
//! - `oxide plugin list` - List installed plugins
//! - `oxide plugin verify` - Verify a plugin's security
//! - `oxide plugin info` - Show plugin information

use anyhow::{Context, Result};
use oxide_plugins::{
    PluginManager, PluginId, InstallSource, PluginScaffold, ScaffoldOptions,
    VerificationStatus, installation::parse_install_specifier,
};
use std::path::PathBuf;

/// Create a new plugin project.
pub fn run_new(
    plugin_id: &str,
    output_dir: Option<&str>,
    publisher: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    let id = PluginId::parse(plugin_id)
        .context("Invalid plugin ID format")?;

    let output = output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let options = ScaffoldOptions {
        publisher: publisher.unwrap_or("your-name").to_string(),
        description: description.map(String::from),
        license: "MIT".to_string(),
        init_git: true,
    };

    let scaffold = PluginScaffold::new(&output);
    let plugin_dir = scaffold.generate(&id, &options)
        .context("Failed to generate plugin")?;

    println!("Created plugin {} at {}", id, plugin_dir.display());
    println!();
    println!("Next steps:");
    println!("  cd {}", plugin_dir.display());
    println!("  # Edit plugin.toml and src/lib.rs");
    println!("  cargo build");
    println!();

    Ok(())
}

/// Install a plugin.
pub fn run_add(specifier: &str) -> Result<()> {
    let project_root = find_project_root()?;
    let mut manager = PluginManager::new(&project_root)
        .context("Failed to initialize plugin manager")?;

    // Parse the specifier
    let (plugin_id, source) = parse_install_specifier(specifier)
        .context("Invalid install specifier")?;

    println!("Installing {}...", plugin_id);

    let manifest = manager.install(plugin_id.full_name(), source)
        .context("Failed to install plugin")?;

    println!("Installed {} v{}", manifest.plugin.id, manifest.plugin.version);
    println!("  Publisher: {}", manifest.plugin.publisher);
    println!("  Category: {}", manifest.plugin.kind);

    if !manifest.required_capabilities().is_empty() {
        println!("  Capabilities required:");
        for cap in manifest.required_capabilities() {
            println!("    - {}", cap);
        }
        println!();
        println!("Note: You may need to add capabilities to your oxide.toml");
    }

    Ok(())
}

/// Uninstall a plugin.
pub fn run_remove(plugin_id: &str) -> Result<()> {
    let project_root = find_project_root()?;
    let mut manager = PluginManager::new(&project_root)
        .context("Failed to initialize plugin manager")?;

    manager.uninstall(plugin_id)
        .context("Failed to uninstall plugin")?;

    println!("Uninstalled {}", plugin_id);

    Ok(())
}

/// List installed plugins.
pub fn run_list(category: Option<&str>, namespace: Option<&str>) -> Result<()> {
    let project_root = find_project_root()?;
    let mut manager = PluginManager::new(&project_root)
        .context("Failed to initialize plugin manager")?;

    let plugins = manager.discover_plugins()
        .context("Failed to discover plugins")?;

    if plugins.is_empty() {
        println!("No plugins installed.");
        println!();
        println!("Install a plugin with: oxide plugin add <plugin-id>");
        return Ok(());
    }

    // Filter by category and namespace
    let filtered: Vec<_> = plugins.iter()
        .filter(|p| {
            let matches_category = category
                .map(|c| p.manifest.plugin.kind.to_string() == c)
                .unwrap_or(true);
            let matches_namespace = namespace
                .map(|n| p.manifest.plugin.id.namespace().as_str() == n)
                .unwrap_or(true);
            matches_category && matches_namespace
        })
        .collect();

    println!("Installed plugins ({}):", filtered.len());
    println!();

    for plugin in filtered {
        let trust_badge = match plugin.trust_level {
            oxide_plugins::TrustLevel::Official => "[Official]",
            oxide_plugins::TrustLevel::Verified => "[Verified]",
            oxide_plugins::TrustLevel::Community => "[Community]",
        };

        println!(
            "  {} v{} {}",
            plugin.id,
            plugin.manifest.plugin.version,
            trust_badge
        );
        println!(
            "    {} - {}",
            plugin.manifest.plugin.kind,
            plugin.manifest.plugin.description
        );
    }

    Ok(())
}

/// Verify a plugin's security.
pub fn run_verify(plugin_id: &str) -> Result<()> {
    let project_root = find_project_root()?;
    let mut manager = PluginManager::new(&project_root)
        .context("Failed to initialize plugin manager")?;

    // Discover plugins first
    manager.discover_plugins()
        .context("Failed to discover plugins")?;

    let report = manager.verify_plugin(plugin_id)
        .context("Failed to verify plugin")?;

    println!("Verification report for {}", plugin_id);
    println!();

    // Overall status
    let status_str = match report.status {
        VerificationStatus::Passed => "PASSED",
        VerificationStatus::PassedWithWarnings => "PASSED WITH WARNINGS",
        VerificationStatus::Failed => "FAILED",
    };
    println!("Status: {}", status_str);
    println!();

    // Checks
    println!("Checks:");
    for check in &report.checks {
        let icon = if check.passed { "[OK]" } else { "[FAIL]" };
        print!("  {} {}", icon, check.name);
        if let Some(msg) = &check.message {
            print!(" - {}", msg);
        }
        println!();
    }

    // Warnings
    if !report.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  [WARN] {}", warning);
        }
    }

    // Errors
    if !report.errors.is_empty() {
        println!();
        println!("Errors:");
        for error in &report.errors {
            println!("  [ERROR] {}", error);
        }
    }

    // Exit with error code if verification failed
    if report.status == VerificationStatus::Failed {
        std::process::exit(1);
    }

    Ok(())
}

/// Show plugin information.
pub fn run_info(plugin_id: &str) -> Result<()> {
    let project_root = find_project_root()?;
    let mut manager = PluginManager::new(&project_root)
        .context("Failed to initialize plugin manager")?;

    // Discover plugins first
    manager.discover_plugins()
        .context("Failed to discover plugins")?;

    let plugin = manager.get_plugin(plugin_id)
        .context("Plugin not found")?;

    println!("Plugin: {}", plugin.id);
    println!();
    println!("  Version:     {}", plugin.manifest.plugin.version);
    println!("  Category:    {}", plugin.manifest.plugin.kind);
    println!("  Publisher:   {}", plugin.manifest.plugin.publisher);
    println!("  License:     {}", plugin.manifest.plugin.license);
    println!("  Trust Level: {:?}", plugin.trust_level);
    println!();
    println!("  Description: {}", plugin.manifest.plugin.description);
    println!();
    println!("  Location:    {}", plugin.install_path.display());

    if let Some(repo) = &plugin.manifest.plugin.repository {
        println!("  Repository:  {}", repo);
    }

    // Capabilities
    let caps = plugin.manifest.required_capabilities();
    if !caps.is_empty() {
        println!();
        println!("  Capabilities:");
        for cap in caps {
            println!("    - {}", cap);
        }
    }

    // Dependencies
    if !plugin.manifest.dependencies.plugins.is_empty() {
        println!();
        println!("  Dependencies:");
        for dep in &plugin.manifest.dependencies.plugins {
            println!("    - {} ({})", dep.id, dep.version);
        }
    }

    Ok(())
}

/// Search the registry for plugins.
pub fn run_search(query: &str, _category: Option<&str>) -> Result<()> {
    println!("Searching for '{}'...", query);
    println!();

    // In a real implementation, this would query the registry
    // For now, show a placeholder message
    println!("Registry search not yet implemented.");
    println!();
    println!("To install a plugin locally, use:");
    println!("  oxide plugin add path ../my-plugin");

    Ok(())
}

/// Find the project root by looking for oxide.toml or Cargo.toml.
fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        if current.join("oxide.toml").exists() || current.join("Cargo.toml").exists() {
            return Ok(current);
        }

        if !current.pop() {
            // Reached filesystem root
            return Ok(std::env::current_dir()?);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_project_root() {
        // This will find the oxide-cli crate root
        let result = find_project_root();
        assert!(result.is_ok());
    }
}
