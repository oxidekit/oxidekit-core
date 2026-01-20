//! Interactive project wizard for OxideKit
//!
//! This module provides an interactive CLI wizard for creating and initializing
//! OxideKit projects. It guides users through template selection, plugin configuration,
//! theme choices, and project settings.

mod prompts;
mod config;
mod steps;
mod generator;

pub use config::{WizardConfig, ProjectConfig, PluginPreset, ThemeChoice};
pub use steps::WizardSteps;
pub use generator::{ProjectGenerator, GenerationResult};

use anyhow::Result;
use std::path::Path;

/// Interactive project creation wizard
pub struct ProjectWizard {
    /// Skip interactive prompts (use defaults)
    skip_prompts: bool,
    /// Pre-configured options
    preset_config: Option<WizardConfig>,
}

impl ProjectWizard {
    /// Create a new wizard instance
    pub fn new() -> Self {
        Self {
            skip_prompts: false,
            preset_config: None,
        }
    }

    /// Create a wizard that skips interactive prompts (--yes flag)
    pub fn non_interactive() -> Self {
        Self {
            skip_prompts: true,
            preset_config: None,
        }
    }

    /// Set preset configuration (for CLI flags)
    pub fn with_preset(mut self, config: WizardConfig) -> Self {
        self.preset_config = Some(config);
        self
    }

    /// Run the wizard for creating a new project
    pub fn run_new(&self, name: Option<&str>) -> Result<ProjectConfig> {
        if self.skip_prompts {
            return self.run_non_interactive(name);
        }

        let steps = WizardSteps::new();

        // Step 1: Project name (if not provided)
        let project_name = match name {
            Some(n) => n.to_string(),
            None => steps.prompt_project_name()?,
        };

        // Step 2: Project description
        let description = steps.prompt_description(&project_name)?;

        // Step 3: Author
        let author = steps.prompt_author()?;

        // Step 4: Starter template selection
        let starter = steps.prompt_starter_selection()?;

        // Step 5: Plugin presets
        let plugins = steps.prompt_plugin_presets()?;

        // Step 6: Theme selection
        let theme = steps.prompt_theme_selection()?;

        // Step 7: Git initialization
        let init_git = steps.prompt_git_init()?;

        // Step 8: Output directory
        let output_dir = steps.prompt_output_directory(&project_name)?;

        Ok(ProjectConfig {
            name: project_name,
            description,
            author,
            starter,
            plugins,
            theme,
            init_git,
            output_dir,
        })
    }

    /// Run the wizard for initializing an existing directory
    pub fn run_init(&self, dir: &Path) -> Result<ProjectConfig> {
        let project_name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-app")
            .to_string();

        if self.skip_prompts {
            return Ok(ProjectConfig::default_for_name(&project_name));
        }

        let steps = WizardSteps::new();

        // Display initialization banner
        steps.display_init_banner(dir)?;

        // Step 1: Confirm project name
        let project_name = steps.prompt_confirm_name(&project_name)?;

        // Step 2: Project description
        let description = steps.prompt_description(&project_name)?;

        // Step 3: Author
        let author = steps.prompt_author()?;

        // Step 4: Starter template selection
        let starter = steps.prompt_starter_selection()?;

        // Step 5: Plugin presets
        let plugins = steps.prompt_plugin_presets()?;

        // Step 6: Theme selection
        let theme = steps.prompt_theme_selection()?;

        // Step 7: Git initialization (check if already a git repo)
        let init_git = if dir.join(".git").exists() {
            false
        } else {
            steps.prompt_git_init()?
        };

        Ok(ProjectConfig {
            name: project_name,
            description,
            author,
            starter,
            plugins,
            theme,
            init_git,
            output_dir: dir.to_path_buf(),
        })
    }

    /// Run non-interactive mode with defaults
    fn run_non_interactive(&self, name: Option<&str>) -> Result<ProjectConfig> {
        let project_name = name.unwrap_or("my-oxide-app").to_string();

        if let Some(preset) = &self.preset_config {
            Ok(ProjectConfig {
                name: project_name.clone(),
                description: preset.description.clone().unwrap_or_else(|| format!("An OxideKit application")),
                author: preset.author.clone(),
                starter: preset.starter.clone(),
                plugins: preset.plugins.clone(),
                theme: preset.theme.clone(),
                init_git: preset.init_git,
                output_dir: preset.output_dir.clone().unwrap_or_else(|| std::env::current_dir().unwrap().join(&project_name)),
            })
        } else {
            Ok(ProjectConfig::default_for_name(&project_name))
        }
    }
}

impl Default for ProjectWizard {
    fn default() -> Self {
        Self::new()
    }
}
