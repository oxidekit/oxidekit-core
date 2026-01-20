//! `oxide new` command - create a new OxideKit project
//!
//! This command provides an interactive wizard for creating new OxideKit projects
//! with customizable templates, plugins, and themes.

use anyhow::Result;
use std::path::PathBuf;

use super::wizard::{ProjectWizard, ProjectGenerator, WizardConfig, PluginPreset, ThemeChoice};

/// Run the new command with interactive wizard
///
/// # Arguments
///
/// * `name` - Optional project name (will prompt if not provided)
/// * `template` - Legacy template argument (ignored, use starter instead)
/// * `starter` - Starter template ID
/// * `output` - Output directory
/// * `yes` - Skip interactive prompts
/// * `plugins` - Plugin presets to include
/// * `theme` - Theme choice
/// * `git` - Whether to initialize git (None = prompt, Some(true/false) = force)
pub fn run(
    name: Option<&str>,
    _template: &str,
    starter: Option<&str>,
    output: Option<&str>,
    yes: bool,
    plugins: &[String],
    theme: Option<&str>,
    git: Option<bool>,
) -> Result<()> {
    // Build preset config from CLI arguments
    let preset_config = build_preset_config(starter, output, plugins, theme, git);

    // Create wizard (interactive or non-interactive based on --yes flag)
    let wizard = if yes {
        ProjectWizard::non_interactive().with_preset(preset_config)
    } else {
        ProjectWizard::new().with_preset(preset_config)
    };

    // Run the wizard
    let config = wizard.run_new(name)?;

    // Display summary and confirm (unless --yes)
    if !yes {
        let steps = super::wizard::WizardSteps::new();
        steps.display_summary(&config)?;

        if !steps.prompt_confirmation()? {
            println!("Project creation cancelled.");
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

/// Run with legacy simple mode (for backwards compatibility)
pub fn run_simple(name: &str, _template: &str) -> Result<()> {
    run(Some(name), _template, None, None, true, &[], None, Some(false))
}

/// Build preset config from CLI arguments
fn build_preset_config(
    starter: Option<&str>,
    output: Option<&str>,
    plugins: &[String],
    theme: Option<&str>,
    git: Option<bool>,
) -> WizardConfig {
    let mut config = WizardConfig::default();

    // Set starter if provided
    config.starter = starter.map(|s| s.to_string());

    // Set output directory if provided
    config.output_dir = output.map(PathBuf::from);

    // Parse plugin presets
    config.plugins = parse_plugin_presets(plugins);

    // Parse theme
    if let Some(theme_str) = theme {
        config.theme = parse_theme(theme_str);
    }

    // Git initialization
    config.init_git = git.unwrap_or(true);

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
        let plugins = vec!["core".to_string(), "desktop".to_string(), "network".to_string()];
        let presets = parse_plugin_presets(&plugins);
        assert_eq!(presets.len(), 3);
        assert!(presets.contains(&PluginPreset::Core));
        assert!(presets.contains(&PluginPreset::Desktop));
        assert!(presets.contains(&PluginPreset::Network));
    }

    #[test]
    fn test_parse_theme() {
        assert_eq!(parse_theme("dark"), ThemeChoice::Dark);
        assert_eq!(parse_theme("Light"), ThemeChoice::Light);
        assert_eq!(parse_theme("SYSTEM"), ThemeChoice::System);
        assert_eq!(parse_theme("high-contrast"), ThemeChoice::HighContrast);
        assert_eq!(parse_theme("custom"), ThemeChoice::Custom);
    }

    #[test]
    fn test_parse_unknown_theme() {
        // Should default to dark
        assert_eq!(parse_theme("unknown"), ThemeChoice::Dark);
    }

    #[test]
    fn test_build_preset_config() {
        let config = build_preset_config(
            Some("admin-panel"),
            Some("/tmp/test"),
            &["core".to_string(), "network".to_string()],
            Some("light"),
            Some(true),
        );

        assert_eq!(config.starter, Some("admin-panel".to_string()));
        assert_eq!(config.output_dir, Some(PathBuf::from("/tmp/test")));
        assert_eq!(config.plugins.len(), 2);
        assert_eq!(config.theme, ThemeChoice::Light);
        assert!(config.init_git);
    }
}
