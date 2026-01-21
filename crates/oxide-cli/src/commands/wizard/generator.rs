//! Project generator from wizard configuration
//!
//! Generates OxideKit projects based on the configuration collected by the wizard.

use anyhow::{Context, Result};
use oxide_starters::{StarterRegistry, StarterGenerator as StarterGen};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::config::{ProjectConfig, PluginPreset, ThemeChoice};
use super::prompts;

/// Generate a project from wizard configuration
pub struct ProjectGenerator<'a> {
    config: &'a ProjectConfig,
}

impl<'a> ProjectGenerator<'a> {
    /// Create a new project generator
    pub fn new(config: &'a ProjectConfig) -> Self {
        Self { config }
    }

    /// Generate the project
    pub fn generate(&self) -> Result<GenerationResult> {
        // Determine the actual project directory
        // For starters, StarterGenerator creates at parent_dir/name
        // For blank projects, we use output_dir directly
        let project_dir = self.config.output_dir.clone();
        let mut result = GenerationResult::new(&self.config.name, project_dir.clone());

        // Check if output directory already exists (but don't create it yet for starters)
        if self.config.output_dir.exists() {
            // For init, we allow existing empty directories
            let is_empty = fs::read_dir(&self.config.output_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name() != ".git")
                .count() == 0;

            if !is_empty {
                anyhow::bail!(
                    "Directory '{}' already exists and is not empty",
                    self.config.output_dir.display()
                );
            }
        }

        prompts::info(&format!("Creating project in {}", self.config.output_dir.display()));

        // Generate from starter or blank template
        if let Some(ref starter_id) = self.config.starter {
            // StarterGenerator creates the directory itself, so we don't create it here
            self.generate_from_starter(starter_id, &mut result)?;
        } else {
            // For blank projects, create the directory first
            if !self.config.output_dir.exists() {
                fs::create_dir_all(&self.config.output_dir)
                    .context("Failed to create project directory")?;
            }
            self.generate_blank_project(&mut result)?;
        }

        // Initialize git if requested
        if self.config.init_git {
            self.init_git(&mut result)?;
        }

        result.success = true;
        Ok(result)
    }

    /// Generate project from a starter template
    fn generate_from_starter(&self, starter_id: &str, result: &mut GenerationResult) -> Result<()> {
        let registry = StarterRegistry::with_builtin();

        let starter = registry
            .get(starter_id)
            .ok_or_else(|| anyhow::anyhow!("Starter not found: {}", starter_id))?;

        prompts::step(1, 4, &format!("Applying starter template: {}", starter.name));

        // StarterGenerator.generate() creates project at parent_dir/name
        // We want the project at self.config.output_dir
        // So we use output_dir's parent as parent_dir, and output_dir's filename as the directory name
        let parent_dir = self.config.output_dir.parent()
            .unwrap_or(Path::new("."));
        let dir_name = self.config.output_dir.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.config.name.clone());

        let generator = StarterGen::new(starter);
        let starter_result = generator.generate(&dir_name, parent_dir)?;

        // Update result with the actual project directory from StarterGenerator
        result.project_dir = starter_result.project_dir.clone();
        result.files_created = starter_result.files_created.iter().map(|p| p.display().to_string()).collect();
        result.plugins_to_install = starter_result.plugins_to_install;

        // Now overlay our customizations using the actual project directory
        self.apply_customizations(result)?;

        Ok(())
    }

    /// Generate a blank project from scratch
    fn generate_blank_project(&self, result: &mut GenerationResult) -> Result<()> {
        prompts::step(1, 4, "Creating project structure");

        // Create directory structure
        fs::create_dir_all(self.config.output_dir.join("src"))?;
        fs::create_dir_all(self.config.output_dir.join("ui/components"))?;
        fs::create_dir_all(self.config.output_dir.join("assets/fonts"))?;
        fs::create_dir_all(self.config.output_dir.join("assets/icons"))?;
        fs::create_dir_all(self.config.output_dir.join("assets/images"))?;

        prompts::step(2, 4, "Generating configuration files");

        // Generate oxide.toml
        let manifest = self.generate_manifest();
        fs::write(self.config.output_dir.join("oxide.toml"), manifest)
            .context("Failed to write oxide.toml")?;
        result.files_created.push("oxide.toml".to_string());

        // Generate Cargo.toml
        let cargo_toml = self.generate_cargo_toml();
        fs::write(self.config.output_dir.join("Cargo.toml"), cargo_toml)
            .context("Failed to write Cargo.toml")?;
        result.files_created.push("Cargo.toml".to_string());

        prompts::step(3, 4, "Generating source files");

        // Generate main.rs
        let main_rs = self.generate_main_rs();
        fs::write(self.config.output_dir.join("src/main.rs"), main_rs)
            .context("Failed to write src/main.rs")?;
        result.files_created.push("src/main.rs".to_string());

        // Generate app.oui
        let app_oui = self.generate_app_oui();
        fs::write(self.config.output_dir.join("ui/app.oui"), app_oui)
            .context("Failed to write ui/app.oui")?;
        result.files_created.push("ui/app.oui".to_string());

        // Generate .gitignore
        let gitignore = self.generate_gitignore();
        fs::write(self.config.output_dir.join(".gitignore"), gitignore)
            .context("Failed to write .gitignore")?;
        result.files_created.push(".gitignore".to_string());

        // Generate README.md
        let readme = self.generate_readme();
        fs::write(self.config.output_dir.join("README.md"), readme)
            .context("Failed to write README.md")?;
        result.files_created.push("README.md".to_string());

        // Collect plugins to install
        for preset in &self.config.plugins {
            for plugin_id in preset.plugin_ids() {
                if !result.plugins_to_install.contains(&plugin_id.to_string()) {
                    result.plugins_to_install.push(plugin_id.to_string());
                }
            }
        }

        Ok(())
    }

    /// Apply wizard customizations on top of starter template
    fn apply_customizations(&self, result: &mut GenerationResult) -> Result<()> {
        prompts::step(2, 4, "Applying customizations");

        // Re-generate oxide.toml with our settings in the actual project directory
        let manifest = self.generate_manifest();
        fs::write(result.project_dir.join("oxide.toml"), manifest)
            .context("Failed to write oxide.toml")?;

        // Add additional plugins from presets
        for preset in &self.config.plugins {
            for plugin_id in preset.plugin_ids() {
                if !result.plugins_to_install.contains(&plugin_id.to_string()) {
                    result.plugins_to_install.push(plugin_id.to_string());
                }
            }
        }

        Ok(())
    }

    /// Initialize git repository
    fn init_git(&self, result: &mut GenerationResult) -> Result<()> {
        prompts::step(4, 4, "Initializing git repository");

        let output = Command::new("git")
            .args(["init"])
            .current_dir(&result.project_dir)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                result.git_initialized = true;
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                prompts::warning(&format!("Git init warning: {}", stderr.trim()));
            }
            Err(e) => {
                prompts::warning(&format!("Could not initialize git: {}", e));
            }
        }

        Ok(())
    }

    /// Generate oxide.toml manifest
    fn generate_manifest(&self) -> String {
        let mut manifest = String::new();

        manifest.push_str("[app]\n");
        manifest.push_str(&format!("id = \"{}\"\n", self.config.app_id()));
        manifest.push_str(&format!("name = \"{}\"\n", self.config.name));
        manifest.push_str("version = \"0.1.0\"\n");
        manifest.push_str(&format!("description = \"{}\"\n", self.config.description));

        if let Some(ref author) = self.config.author {
            manifest.push_str(&format!("author = \"{}\"\n", author));
        }

        manifest.push_str("\n[core]\n");
        manifest.push_str("requires = \">=0.1.0\"\n");

        manifest.push_str("\n[window]\n");
        manifest.push_str(&format!("title = \"{}\"\n", self.config.name));
        manifest.push_str("width = 1280\n");
        manifest.push_str("height = 720\n");
        manifest.push_str("resizable = true\n");
        manifest.push_str("decorations = true\n");

        manifest.push_str("\n[theme]\n");
        manifest.push_str(&format!("default = \"{}\"\n", self.config.theme.to_toml_value()));

        // Add extensions/plugins section
        let plugin_ids: Vec<String> = self.config.plugins
            .iter()
            .flat_map(|p| p.plugin_ids())
            .map(|s| format!("\"{}\"", s))
            .collect();

        manifest.push_str("\n[extensions]\n");
        if plugin_ids.is_empty() {
            manifest.push_str("allow = []\n");
        } else {
            manifest.push_str(&format!("allow = [{}]\n", plugin_ids.join(", ")));
        }

        manifest.push_str("\n[dev]\n");
        manifest.push_str("hot_reload = true\n");
        manifest.push_str("inspector = true\n");

        manifest
    }

    /// Generate Cargo.toml
    fn generate_cargo_toml(&self) -> String {
        let package_name = self.config.name.replace('-', "_");

        let mut cargo = String::new();

        cargo.push_str("[package]\n");
        cargo.push_str(&format!("name = \"{}\"\n", package_name));
        cargo.push_str("version = \"0.1.0\"\n");
        cargo.push_str("edition = \"2021\"\n");

        if let Some(ref author) = self.config.author {
            cargo.push_str(&format!("authors = [\"{}\"]\n", author));
        }

        cargo.push_str(&format!("description = \"{}\"\n", self.config.description));

        cargo.push_str("\n[dependencies]\n");
        cargo.push_str("oxide-runtime = { git = \"https://github.com/oxidekit/oxidekit-core\" }\n");
        cargo.push_str("anyhow = \"1.0\"\n");
        cargo.push_str("tracing = \"0.1\"\n");
        cargo.push_str("tracing-subscriber = \"0.3\"\n");

        cargo
    }

    /// Generate main.rs
    fn generate_main_rs(&self) -> String {
        r#"use oxide_runtime::Application;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load and run the application
    let app = Application::from_manifest("oxide.toml")?;
    app.run()
}
"#.to_string()
    }

    /// Generate app.oui
    fn generate_app_oui(&self) -> String {
        let name_pascal = to_pascal_case(&self.config.name);

        let (bg_color, text_color, secondary_color) = match self.config.theme {
            ThemeChoice::Dark | ThemeChoice::System => ("#1F2937", "#E5E7EB", "#9CA3AF"),
            ThemeChoice::Light => ("#FFFFFF", "#1F2937", "#6B7280"),
            ThemeChoice::HighContrast => ("#000000", "#FFFFFF", "#FFFF00"),
            ThemeChoice::Custom => ("#1F2937", "#E5E7EB", "#9CA3AF"),
        };

        format!(
            r#"// {} - Main application UI
// This file defines the root UI component

app {}App {{
    Column {{
        align: center
        justify: center
        width: fill
        height: fill
        gap: 16
        background: "{}"

        Text {{
            content: "Welcome to {}!"
            size: 48
            color: "{}"
        }}

        Text {{
            content: "Built with OxideKit"
            size: 18
            color: "{}"
        }}
    }}
}}
"#,
            self.config.name,
            name_pascal,
            bg_color,
            self.config.name,
            text_color,
            secondary_color
        )
    }

    /// Generate .gitignore
    fn generate_gitignore(&self) -> String {
        r#"/target
/build
*.lock
!extensions.lock
.DS_Store
.env
.env.local
*.log
"#.to_string()
    }

    /// Generate README.md
    fn generate_readme(&self) -> String {
        format!(
            r#"# {}

{}

## Development

```bash
# Run in development mode
oxide dev

# Build for production
oxide build --release

# Run the built application
oxide run
```

## Project Structure

```
{}/
├── oxide.toml          # Project manifest
├── Cargo.toml          # Rust dependencies
├── src/
│   └── main.rs         # Application entry point
├── ui/
│   ├── app.oui         # Root UI component
│   └── components/     # UI components
└── assets/
    ├── fonts/          # Custom fonts
    ├── icons/          # Icons
    └── images/         # Images
```

## License

MIT
"#,
            self.config.name,
            self.config.description,
            self.config.name
        )
    }
}

/// Result of project generation
#[derive(Debug)]
pub struct GenerationResult {
    /// Project name
    pub project_name: String,
    /// Actual project directory path
    pub project_dir: PathBuf,
    /// Whether generation was successful
    pub success: bool,
    /// List of files created
    pub files_created: Vec<String>,
    /// Plugins that should be installed
    pub plugins_to_install: Vec<String>,
    /// Whether git was initialized
    pub git_initialized: bool,
}

impl GenerationResult {
    fn new(name: &str, project_dir: PathBuf) -> Self {
        Self {
            project_name: name.to_string(),
            project_dir,
            success: false,
            files_created: Vec::new(),
            plugins_to_install: Vec::new(),
            git_initialized: false,
        }
    }

    /// Display the generation summary
    pub fn display_summary(&self) {
        println!();
        prompts::success(&format!("Created project: {}", self.project_name));
        println!();

        println!("  {} files created", self.files_created.len());

        if self.git_initialized {
            println!("  Git repository initialized");
        }

        if !self.plugins_to_install.is_empty() {
            println!();
            println!("  Recommended plugins to install:");
            for plugin in &self.plugins_to_install {
                println!("    oxide add {}", plugin);
            }
        }

        println!();
        println!("  Next steps:");
        println!("    cd {}", self.project_name);
        println!("    oxide dev");
        println!();
    }
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my-app"), "MyApp");
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("test"), "Test");
    }

    #[test]
    fn test_generation_result() {
        let result = GenerationResult::new("test-project");
        assert!(!result.success);
        assert_eq!(result.project_name, "test-project");
    }
}
