//! Starter Generator
//!
//! Generates projects from starter templates.

use crate::{StarterSpec, GeneratedFile, PostInitStep};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Project generator from starter templates
pub struct StarterGenerator<'a> {
    /// The starter spec to generate from
    spec: &'a StarterSpec,

    /// Template variables
    variables: HashMap<String, String>,
}

impl<'a> StarterGenerator<'a> {
    /// Create a new generator for a starter
    pub fn new(spec: &'a StarterSpec) -> Self {
        Self {
            spec,
            variables: HashMap::new(),
        }
    }

    /// Set a template variable
    pub fn with_variable(mut self, name: &str, value: &str) -> Self {
        self.variables.insert(name.to_string(), value.to_string());
        self
    }

    /// Set multiple template variables
    pub fn with_variables(mut self, vars: HashMap<String, String>) -> Self {
        self.variables.extend(vars);
        self
    }

    /// Generate a project
    pub fn generate(&self, project_name: &str, output_dir: &Path) -> anyhow::Result<GenerationResult> {
        // Set up default variables
        let mut vars = self.variables.clone();
        vars.insert("project_name".to_string(), project_name.to_string());
        vars.insert("starter_id".to_string(), self.spec.id.clone());
        vars.insert("starter_version".to_string(), self.spec.version.clone());

        // Create output directory
        let project_dir = output_dir.join(project_name);
        if project_dir.exists() {
            anyhow::bail!("Directory already exists: {}", project_dir.display());
        }
        fs::create_dir_all(&project_dir)?;

        let mut result = GenerationResult {
            project_dir: project_dir.clone(),
            files_created: Vec::new(),
            plugins_to_install: Vec::new(),
            post_init_steps: Vec::new(),
        };

        // Generate files
        for file_spec in &self.spec.files {
            if self.should_generate_file(file_spec, &vars) {
                let file_path = project_dir.join(&file_spec.path);
                self.generate_file(&file_path, &file_spec.template, &vars)?;
                result.files_created.push(file_path);
            }
        }

        // Generate core project files
        self.generate_oxide_toml(&project_dir, project_name, &vars)?;
        result.files_created.push(project_dir.join("oxide.toml"));

        self.generate_gitignore(&project_dir)?;
        result.files_created.push(project_dir.join(".gitignore"));

        // Create src directory
        fs::create_dir_all(project_dir.join("src"))?;
        self.generate_main_rs(&project_dir, project_name)?;
        result.files_created.push(project_dir.join("src/main.rs"));

        // Create ui directory
        fs::create_dir_all(project_dir.join("ui"))?;
        self.generate_app_oui(&project_dir, project_name)?;
        result.files_created.push(project_dir.join("ui/app.oui"));

        // Create assets directories
        fs::create_dir_all(project_dir.join("assets/fonts"))?;
        fs::create_dir_all(project_dir.join("assets/icons"))?;
        fs::create_dir_all(project_dir.join("assets/images"))?;

        // Record plugins to install
        for plugin in &self.spec.plugins {
            if !plugin.optional {
                result.plugins_to_install.push(plugin.id.clone());
            }
        }

        // Record post-init steps
        result.post_init_steps = self.spec.post_init.clone();

        Ok(result)
    }

    /// Check if a file should be generated based on conditions
    fn should_generate_file(&self, file_spec: &GeneratedFile, vars: &HashMap<String, String>) -> bool {
        if let Some(ref condition) = file_spec.condition {
            // Simple condition evaluation (expand as needed)
            // Format: "target == 'desktop'" or "variable == 'value'"
            if let Some((key, value)) = condition.split_once("==") {
                let key = key.trim().trim_matches('"').trim_matches('\'');
                let value = value.trim().trim_matches('"').trim_matches('\'');

                if let Some(actual) = vars.get(key) {
                    return actual == value;
                }
            }
            return false;
        }
        true
    }

    /// Generate a single file from template
    fn generate_file(&self, path: &Path, template_name: &str, vars: &HashMap<String, String>) -> anyhow::Result<()> {
        // Get template content
        let content = self.get_template_content(template_name)?;

        // Apply variable substitution
        let content = self.substitute_variables(&content, vars);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write file
        fs::write(path, content)?;

        Ok(())
    }

    /// Get template content by name
    fn get_template_content(&self, template_name: &str) -> anyhow::Result<String> {
        // Built-in templates
        let content = match template_name {
            "oxide_toml" => include_str!("../templates/shared/oxide.toml.template"),
            "main_rs" => include_str!("../templates/shared/main.rs.template"),
            "app_oui" => include_str!("../templates/shared/app.oui.template"),
            "gitignore" => include_str!("../templates/shared/gitignore.template"),
            _ => {
                // Check if it's inline content (starts with content:)
                if template_name.starts_with("content:") {
                    return Ok(template_name.strip_prefix("content:").unwrap().to_string());
                }
                anyhow::bail!("Unknown template: {}", template_name);
            }
        };

        Ok(content.to_string())
    }

    /// Substitute variables in content
    fn substitute_variables(&self, content: &str, vars: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }

    /// Generate oxide.toml
    fn generate_oxide_toml(&self, project_dir: &Path, project_name: &str, _vars: &HashMap<String, String>) -> anyhow::Result<()> {
        let content = format!(
            r#"[app]
name = "{name}"
version = "0.1.0"
description = "A {starter} app built with OxideKit"

[build]
# Build configuration
entry = "ui/app.oui"
assets = "assets"

[dev]
# Development configuration
hot_reload = true
port = 3000
"#,
            name = project_name,
            starter = self.spec.name,
        );

        fs::write(project_dir.join("oxide.toml"), content)?;
        Ok(())
    }

    /// Generate .gitignore
    fn generate_gitignore(&self, project_dir: &Path) -> anyhow::Result<()> {
        let content = r#"# Build output
/target/
/build/
/dist/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# OxideKit
/.oxidekit/
/diagnostics.json
"#;

        fs::write(project_dir.join(".gitignore"), content)?;
        Ok(())
    }

    /// Generate src/main.rs
    fn generate_main_rs(&self, project_dir: &Path, project_name: &str) -> anyhow::Result<()> {
        let content = format!(
            r#"//! {} - Built with OxideKit
//!
//! Generated from the {} starter template.

use oxide_runtime::prelude::*;

fn main() -> anyhow::Result<()> {{
    // Initialize the OxideKit runtime
    let app = OxideApp::new("{}")?;

    // Run the application
    app.run()
}}
"#,
            project_name,
            self.spec.name,
            project_name,
        );

        fs::write(project_dir.join("src/main.rs"), content)?;
        Ok(())
    }

    /// Generate ui/app.oui
    fn generate_app_oui(&self, project_dir: &Path, project_name: &str) -> anyhow::Result<()> {
        let content = format!(
            r#"// {} - Main UI file
// Generated from the {} starter

App {{
    Window {{
        title: "{}"
        width: 1200
        height: 800

        Column {{
            padding: 24
            gap: 16

            Text {{
                content: "Welcome to {}"
                role: "heading"
            }}

            Text {{
                content: "Built with OxideKit"
                role: "body"
            }}
        }}
    }}
}}
"#,
            project_name,
            self.spec.name,
            project_name,
            project_name,
        );

        fs::write(project_dir.join("ui/app.oui"), content)?;
        Ok(())
    }
}

/// Result of project generation
#[derive(Debug)]
pub struct GenerationResult {
    /// Path to the generated project
    pub project_dir: PathBuf,

    /// Files that were created
    pub files_created: Vec<PathBuf>,

    /// Plugins that should be installed
    pub plugins_to_install: Vec<String>,

    /// Post-initialization steps
    pub post_init_steps: Vec<PostInitStep>,
}

impl GenerationResult {
    /// Get summary message for the user
    pub fn summary(&self) -> String {
        let mut msg = format!(
            "Created project at: {}\n\n",
            self.project_dir.display()
        );

        msg.push_str(&format!("Files created: {}\n", self.files_created.len()));

        if !self.plugins_to_install.is_empty() {
            msg.push_str("\nPlugins to install:\n");
            for plugin in &self.plugins_to_install {
                msg.push_str(&format!("  - {}\n", plugin));
            }
        }

        if !self.post_init_steps.is_empty() {
            msg.push_str("\nNext steps:\n");
            for (i, step) in self.post_init_steps.iter().enumerate() {
                match step {
                    PostInitStep::Command { command, description } => {
                        if let Some(desc) = description {
                            msg.push_str(&format!("  {}. {} ({})\n", i + 1, desc, command));
                        } else {
                            msg.push_str(&format!("  {}. Run: {}\n", i + 1, command));
                        }
                    }
                    PostInitStep::Message { text, .. } => {
                        msg.push_str(&format!("  {}. {}\n", i + 1, text));
                    }
                    PostInitStep::OpenUrl { url } => {
                        msg.push_str(&format!("  {}. Open: {}\n", i + 1, url));
                    }
                }
            }
        }

        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StarterRegistry;
    use tempfile::tempdir;

    #[test]
    fn test_generate_project() {
        let registry = StarterRegistry::with_builtin();
        let spec = registry.get("admin-panel").unwrap();

        let generator = StarterGenerator::new(spec);

        let temp_dir = tempdir().unwrap();
        let result = generator.generate("test-app", temp_dir.path()).unwrap();

        assert!(result.project_dir.exists());
        assert!(result.project_dir.join("oxide.toml").exists());
        assert!(result.project_dir.join("src/main.rs").exists());
        assert!(result.project_dir.join("ui/app.oui").exists());
    }

    #[test]
    fn test_variable_substitution() {
        let registry = StarterRegistry::with_builtin();
        let spec = registry.get("admin-panel").unwrap();

        let generator = StarterGenerator::new(spec)
            .with_variable("custom_var", "custom_value");

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "TestProject".to_string());

        let content = "Hello {{name}}!";
        let result = generator.substitute_variables(content, &vars);

        assert_eq!(result, "Hello TestProject!");
    }
}
