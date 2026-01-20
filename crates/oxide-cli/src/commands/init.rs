//! `oxide init` command - initialize an existing directory as OxideKit project
//!
//! This command provides an interactive wizard for initializing the current
//! directory (or a specified directory) as an OxideKit project.

use anyhow::{Context, Result};
use std::path::Path;

use super::wizard::{ProjectWizard, ProjectGenerator, WizardConfig, PluginPreset, ThemeChoice};

/// Run the init command with interactive wizard
pub fn run(
    starter: Option<&str>,
    yes: bool,
    plugins: &[String],
    theme: Option<&str>,
) -> Result<()> {
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;

    run_in_directory(&current_dir, starter, yes, plugins, theme)
}

/// Run init in a specific directory
pub fn run_in_directory(
    dir: &Path,
    starter: Option<&str>,
    yes: bool,
    plugins: &[String],
    theme: Option<&str>,
) -> Result<()> {
    // Check if directory is suitable for initialization
    validate_directory(dir)?;

    // Build preset config from CLI arguments
    let preset_config = build_preset_config(starter, plugins, theme);

    // Create and run the wizard
    let wizard = if yes {
        ProjectWizard::non_interactive().with_preset(preset_config)
    } else {
        ProjectWizard::new().with_preset(preset_config)
    };

    let config = wizard.run_init(dir)?;

    // Display summary and confirm
    if !yes {
        let steps = super::wizard::WizardSteps::new();
        steps.display_summary(&config)?;

        if !steps.prompt_confirmation()? {
            println!("Initialization cancelled.");
            return Ok(());
        }
    }

    // Generate the project
    let generator = ProjectGenerator::new(&config);
    let result = generator.generate()?;

    // Display results
    result.display_summary();

    Ok(())
}

/// Validate that the directory is suitable for initialization
fn validate_directory(dir: &Path) -> Result<()> {
    if !dir.exists() {
        anyhow::bail!("Directory does not exist: {}", dir.display());
    }

    if !dir.is_dir() {
        anyhow::bail!("Path is not a directory: {}", dir.display());
    }

    // Check if already an OxideKit project
    if dir.join("oxide.toml").exists() {
        anyhow::bail!(
            "Directory already contains an OxideKit project (oxide.toml exists).\n\
             Use 'oxide new <name>' to create a new project in a different directory."
        );
    }

    // Check if directory is empty (excluding .git)
    let non_git_entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() != ".git" && e.file_name() != ".gitignore")
        .collect();

    if !non_git_entries.is_empty() {
        // Show what files exist
        let file_names: Vec<String> = non_git_entries
            .iter()
            .take(5)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        let more = if non_git_entries.len() > 5 {
            format!(" (and {} more)", non_git_entries.len() - 5)
        } else {
            String::new()
        };

        anyhow::bail!(
            "Directory is not empty. Found: {}{}\n\n\
             To initialize in a non-empty directory, please ensure it only contains:\n\
             - .git directory (for existing repositories)\n\
             - .gitignore file\n\n\
             Or use 'oxide new <name>' to create a new project in a fresh directory.",
            file_names.join(", "),
            more
        );
    }

    Ok(())
}

/// Build preset config from CLI arguments
fn build_preset_config(
    starter: Option<&str>,
    plugins: &[String],
    theme: Option<&str>,
) -> WizardConfig {
    let mut config = WizardConfig::default();

    // Set starter if provided
    config.starter = starter.map(|s| s.to_string());

    // Parse plugin presets
    config.plugins = parse_plugin_presets(plugins);

    // Parse theme
    if let Some(theme_str) = theme {
        config.theme = parse_theme(theme_str);
    }

    // Default to init git (usually already a git repo for init)
    config.init_git = false;

    config
}

/// Parse plugin preset strings into PluginPreset enum
fn parse_plugin_presets(plugins: &[String]) -> Vec<PluginPreset> {
    plugins
        .iter()
        .filter_map(|p| match p.to_lowercase().as_str() {
            "core" => Some(PluginPreset::Core),
            "desktop" => Some(PluginPreset::Desktop),
            "web" => Some(PluginPreset::Web),
            "native" => Some(PluginPreset::Native),
            "network" => Some(PluginPreset::Network),
            "storage" => Some(PluginPreset::Storage),
            "crypto" => Some(PluginPreset::Crypto),
            "full" | "all" => Some(PluginPreset::Full),
            _ => {
                tracing::warn!("Unknown plugin preset: {}", p);
                None
            }
        })
        .collect()
}

/// Parse theme string into ThemeChoice enum
fn parse_theme(theme: &str) -> ThemeChoice {
    match theme.to_lowercase().as_str() {
        "dark" => ThemeChoice::Dark,
        "light" => ThemeChoice::Light,
        "system" | "auto" => ThemeChoice::System,
        "high-contrast" | "highcontrast" | "contrast" => ThemeChoice::HighContrast,
        "custom" => ThemeChoice::Custom,
        _ => {
            tracing::warn!("Unknown theme '{}', using dark", theme);
            ThemeChoice::Dark
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugin_presets() {
        let plugins = vec!["core".to_string(), "desktop".to_string()];
        let presets = parse_plugin_presets(&plugins);
        assert_eq!(presets.len(), 2);
        assert!(presets.contains(&PluginPreset::Core));
        assert!(presets.contains(&PluginPreset::Desktop));
    }

    #[test]
    fn test_parse_theme() {
        assert_eq!(parse_theme("dark"), ThemeChoice::Dark);
        assert_eq!(parse_theme("Light"), ThemeChoice::Light);
        assert_eq!(parse_theme("system"), ThemeChoice::System);
        assert_eq!(parse_theme("high-contrast"), ThemeChoice::HighContrast);
    }

    #[test]
    fn test_parse_unknown_preset() {
        let plugins = vec!["unknown".to_string()];
        let presets = parse_plugin_presets(&plugins);
        assert!(presets.is_empty());
    }
}
