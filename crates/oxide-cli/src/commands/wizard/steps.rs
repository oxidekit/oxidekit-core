//! Wizard step implementations
//!
//! Individual steps for the project creation wizard.

use anyhow::Result;
use oxide_starters::{StarterRegistry, StarterCategory};
use std::path::{Path, PathBuf};

use super::config::{PluginPreset, ThemeChoice};
use super::prompts;

/// Wizard steps controller
pub struct WizardSteps {
    /// Starter registry for template selection
    registry: StarterRegistry,
}

impl WizardSteps {
    /// Create a new wizard steps instance
    pub fn new() -> Self {
        Self {
            registry: StarterRegistry::with_builtin(),
        }
    }

    /// Display the welcome banner for new projects
    pub fn display_welcome_banner(&self) -> Result<()> {
        prompts::display_header("OxideKit Project Wizard");
        prompts::info("Let's create your new OxideKit application!");
        prompts::info("Press Ctrl+C at any time to cancel.");
        println!();
        Ok(())
    }

    /// Display the init banner for existing directories
    pub fn display_init_banner(&self, dir: &Path) -> Result<()> {
        prompts::display_header("OxideKit Project Initialization");
        prompts::info(&format!("Initializing in: {}", dir.display()));
        prompts::info("Press Ctrl+C at any time to cancel.");
        println!();
        Ok(())
    }

    /// Prompt for project name
    pub fn prompt_project_name(&self) -> Result<String> {
        prompts::display_section("Project Name");
        prompts::info("Choose a name for your project (will be used as directory name).");

        prompts::validated_input(
            "Project name",
            Some("my-oxide-app"),
            |input| {
                if input.is_empty() {
                    return Err("Project name cannot be empty".to_string());
                }
                if input.contains(' ') {
                    return Err("Project name cannot contain spaces (use hyphens instead)".to_string());
                }
                if !input.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                    return Err("Project name must start with a letter".to_string());
                }
                if !input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    return Err("Project name can only contain letters, numbers, hyphens, and underscores".to_string());
                }
                Ok(())
            },
        )
    }

    /// Prompt to confirm project name (for init)
    pub fn prompt_confirm_name(&self, current_name: &str) -> Result<String> {
        prompts::display_section("Project Name");
        prompts::info(&format!("Current directory name: {}", current_name));

        prompts::validated_input(
            "Project name",
            Some(current_name),
            |input| {
                if input.is_empty() {
                    return Err("Project name cannot be empty".to_string());
                }
                Ok(())
            },
        )
    }

    /// Prompt for project description
    pub fn prompt_description(&self, _project_name: &str) -> Result<String> {
        prompts::display_section("Description");
        prompts::info("A brief description of your project.");

        prompts::text_input(
            "Description",
            Some("An OxideKit application"),
        )
    }

    /// Prompt for author information
    pub fn prompt_author(&self) -> Result<Option<String>> {
        prompts::display_section("Author");
        prompts::info("Your name or email (optional, press Enter to skip).");

        // Try to get default from git config
        let default_author = get_git_author();

        let author = prompts::text_input(
            "Author",
            default_author.as_deref(),
        )?;

        if author.is_empty() {
            Ok(None)
        } else {
            Ok(Some(author))
        }
    }

    /// Prompt for starter template selection
    pub fn prompt_starter_selection(&self) -> Result<Option<String>> {
        prompts::display_section("Starter Template");
        prompts::info("Choose a starter template to bootstrap your project.");
        prompts::info("Select 'Blank Project' for a minimal setup.");
        println!();

        // Build options list
        let mut options: Vec<StarterOption> = vec![
            StarterOption {
                id: None,
                name: "Blank Project".to_string(),
                description: "Minimal setup - just the essentials".to_string(),
                category: None,
            }
        ];

        // Add starters grouped by category
        for starter in self.registry.list() {
            options.push(StarterOption {
                id: Some(starter.id.clone()),
                name: starter.name.clone(),
                description: starter.description.clone(),
                category: Some(starter.metadata.category),
            });
        }

        // Format options for display
        let display_options: Vec<String> = options
            .iter()
            .map(|opt| {
                let category_tag = opt.category
                    .map(|c| format!("[{}] ", c.as_str()))
                    .unwrap_or_default();
                format!("{}{} - {}", category_tag, opt.name, opt.description)
            })
            .collect();

        let selection = prompts::fuzzy_select("Select a starter", &display_options, 0)?;

        Ok(options[selection].id.clone())
    }

    /// Prompt for plugin preset selection
    pub fn prompt_plugin_presets(&self) -> Result<Vec<PluginPreset>> {
        prompts::display_section("Plugin Capabilities");
        prompts::info("Select the capabilities your app needs.");
        prompts::info("You can add more plugins later with 'oxide add <plugin>'.");
        println!();

        let presets = PluginPreset::all();

        // Format options with descriptions
        let display_options: Vec<String> = presets
            .iter()
            .map(|p| format!("{} - {}", p.name(), p.description()))
            .collect();

        // Default to Core selected
        let defaults: Vec<bool> = presets
            .iter()
            .map(|p| *p == PluginPreset::Core)
            .collect();

        let selections = prompts::multi_select("Select capabilities (Space to toggle)", &display_options, &defaults)?;

        if selections.is_empty() {
            // If nothing selected, use Core as default
            Ok(vec![PluginPreset::Core])
        } else {
            Ok(selections.iter().map(|&i| presets[i]).collect())
        }
    }

    /// Prompt for theme selection
    pub fn prompt_theme_selection(&self) -> Result<ThemeChoice> {
        prompts::display_section("Theme");
        prompts::info("Choose a default theme for your application.");
        println!();

        let themes = ThemeChoice::all();

        let display_options: Vec<String> = themes
            .iter()
            .map(|t| format!("{} - {}", t.name(), t.description()))
            .collect();

        let selection = prompts::select("Select theme", &display_options, 0)?;

        Ok(themes[selection])
    }

    /// Prompt for git initialization
    pub fn prompt_git_init(&self) -> Result<bool> {
        prompts::display_section("Git Repository");

        prompts::confirm("Initialize a git repository?", true)
    }

    /// Prompt for output directory
    pub fn prompt_output_directory(&self, project_name: &str) -> Result<PathBuf> {
        prompts::display_section("Output Directory");

        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let default_path = current_dir.join(project_name);

        prompts::info(&format!("Default: {}", default_path.display()));

        if prompts::confirm("Use this location?", true)? {
            Ok(default_path)
        } else {
            let custom_path = prompts::text_input(
                "Enter custom path",
                Some(&default_path.to_string_lossy()),
            )?;
            Ok(PathBuf::from(custom_path))
        }
    }

    /// Display summary before confirmation
    pub fn display_summary(&self, config: &super::config::ProjectConfig) -> Result<()> {
        prompts::display_section("Project Summary");

        println!("  {} {}", console::style("Name:").bold(), config.name);
        println!("  {} {}", console::style("Description:").bold(), config.description);

        if let Some(ref author) = config.author {
            println!("  {} {}", console::style("Author:").bold(), author);
        }

        let starter_display = config.starter.as_deref().unwrap_or("Blank Project");
        println!("  {} {}", console::style("Template:").bold(), starter_display);

        let plugins_display: Vec<&str> = config.plugins.iter().map(|p| p.name()).collect();
        println!("  {} {}", console::style("Plugins:").bold(), plugins_display.join(", "));

        println!("  {} {}", console::style("Theme:").bold(), config.theme.name());
        println!("  {} {}", console::style("Git:").bold(), if config.init_git { "Yes" } else { "No" });
        println!("  {} {}", console::style("Location:").bold(), config.output_dir.display());

        println!();

        Ok(())
    }

    /// Prompt for final confirmation
    pub fn prompt_confirmation(&self) -> Result<bool> {
        prompts::confirm("Create this project?", true)
    }
}

impl Default for WizardSteps {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct for starter options
struct StarterOption {
    id: Option<String>,
    name: String,
    description: String,
    category: Option<StarterCategory>,
}

/// Try to get author from git config
fn get_git_author() -> Option<String> {
    // Try to read from git config
    let output = std::process::Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            // Also try to get email
            let email_output = std::process::Command::new("git")
                .args(["config", "--get", "user.email"])
                .output()
                .ok();

            if let Some(email_out) = email_output {
                if email_out.status.success() {
                    let email = String::from_utf8_lossy(&email_out.stdout).trim().to_string();
                    if !email.is_empty() {
                        return Some(format!("{} <{}>", name, email));
                    }
                }
            }
            return Some(name);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_steps_creation() {
        let steps = WizardSteps::new();
        assert!(!steps.registry.is_empty());
    }
}
