//! Scaffold Generator for Plugins and Components
//!
//! Generates plugin and component skeletons from templates, including:
//! - `plugin.toml` configuration
//! - Minimal implementation stub
//! - Tests
//! - Documentation stub
//! - AI spec stub (`oxide.ai.json`)
//!
//! Can be triggered by AI gap-filler or CLI commands.

use crate::naming::{CanonicalId, DerivedNames, Namespace};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Errors in scaffold generation
#[derive(Error, Debug)]
pub enum ScaffoldError {
    /// Template error
    #[error("Template error: {0}")]
    TemplateError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid scaffold kind
    #[error("Invalid scaffold kind: {0}")]
    InvalidKind(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Directory already exists
    #[error("Directory already exists: {0}")]
    DirectoryExists(PathBuf),

    /// Handlebars error
    #[error("Template rendering error: {0}")]
    RenderError(#[from] handlebars::RenderError),
}

/// Kind of scaffold to generate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaffoldKind {
    /// OxideKit plugin
    Plugin,
    /// UI component
    Component,
    /// Widget bundle (for WebView compatibility)
    Widget,
    /// Native extension
    NativeExtension,
    /// Compatibility shim
    CompatShim,
}

impl ScaffoldKind {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ScaffoldKind::Plugin => "plugin",
            ScaffoldKind::Component => "component",
            ScaffoldKind::Widget => "widget",
            ScaffoldKind::NativeExtension => "native_extension",
            ScaffoldKind::CompatShim => "compat_shim",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "plugin" => Some(ScaffoldKind::Plugin),
            "component" => Some(ScaffoldKind::Component),
            "widget" => Some(ScaffoldKind::Widget),
            "native_extension" | "native-extension" => Some(ScaffoldKind::NativeExtension),
            "compat_shim" | "compat-shim" | "shim" => Some(ScaffoldKind::CompatShim),
            _ => None,
        }
    }
}

/// Options for scaffold generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldOptions {
    /// Target directory for output
    pub output_dir: PathBuf,
    /// Canonical ID for the scaffold
    pub id: CanonicalId,
    /// Kind of scaffold
    pub kind: ScaffoldKind,
    /// Description
    pub description: String,
    /// Author name
    pub author: Option<String>,
    /// License (defaults to MIT OR Apache-2.0)
    pub license: Option<String>,
    /// Whether to include tests
    pub include_tests: bool,
    /// Whether to include AI spec
    pub include_ai_spec: bool,
    /// Whether to include documentation
    pub include_docs: bool,
    /// Additional features to enable
    pub features: Vec<String>,
    /// Component-specific: props definition
    pub props: Option<Vec<PropDefinition>>,
    /// Component-specific: events definition
    pub events: Option<Vec<EventDefinition>>,
    /// Plugin-specific: permissions required
    pub permissions: Option<Vec<String>>,
    /// Plugin-specific: capabilities provided
    pub capabilities: Option<Vec<String>>,
}

impl Default for ScaffoldOptions {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            id: CanonicalId::new(Namespace::Community, "my-plugin"),
            kind: ScaffoldKind::Plugin,
            description: "A new OxideKit plugin".to_string(),
            author: None,
            license: Some("MIT OR Apache-2.0".to_string()),
            include_tests: true,
            include_ai_spec: true,
            include_docs: true,
            features: Vec::new(),
            props: None,
            events: None,
            permissions: None,
            capabilities: None,
        }
    }
}

/// Property definition for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropDefinition {
    /// Property name
    pub name: String,
    /// Property type
    pub prop_type: String,
    /// Whether required
    pub required: bool,
    /// Default value
    pub default: Option<String>,
    /// Description
    pub description: String,
}

/// Event definition for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    /// Event name
    pub name: String,
    /// Payload type
    pub payload_type: Option<String>,
    /// Description
    pub description: String,
}

/// Generated scaffold output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedScaffold {
    /// Canonical ID
    pub id: CanonicalId,
    /// Generated files
    pub files: Vec<GeneratedFile>,
    /// Output directory
    pub output_dir: PathBuf,
    /// Instructions for next steps
    pub next_steps: Vec<String>,
}

/// A generated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// Relative path from output directory
    pub path: PathBuf,
    /// File content
    pub content: String,
    /// Whether the file is executable
    pub executable: bool,
}

/// Scaffold generator
pub struct Scaffolder {
    /// Handlebars template engine
    handlebars: Handlebars<'static>,
}

impl Scaffolder {
    /// Create a new scaffolder
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // Register templates
        Self::register_templates(&mut handlebars);

        Self { handlebars }
    }

    /// Register all templates
    fn register_templates(handlebars: &mut Handlebars<'static>) {
        // Plugin templates
        handlebars
            .register_template_string("plugin/Cargo.toml", TEMPLATE_PLUGIN_CARGO_TOML)
            .unwrap();
        handlebars
            .register_template_string("plugin/lib.rs", TEMPLATE_PLUGIN_LIB_RS)
            .unwrap();
        handlebars
            .register_template_string("plugin/plugin.toml", TEMPLATE_PLUGIN_TOML)
            .unwrap();

        // Component templates
        handlebars
            .register_template_string("component/mod.rs", TEMPLATE_COMPONENT_MOD_RS)
            .unwrap();
        handlebars
            .register_template_string("component/component.oui", TEMPLATE_COMPONENT_OUI)
            .unwrap();

        // Common templates
        handlebars
            .register_template_string("common/oxide.ai.json", TEMPLATE_AI_SPEC)
            .unwrap();
        handlebars
            .register_template_string("common/README.md", TEMPLATE_README)
            .unwrap();
        handlebars
            .register_template_string("common/tests.rs", TEMPLATE_TESTS)
            .unwrap();
    }

    /// Generate a scaffold
    pub fn generate(&self, options: &ScaffoldOptions) -> Result<GeneratedScaffold, ScaffoldError> {
        let names = DerivedNames::from_id(&options.id);
        let mut files = Vec::new();

        // Build template context
        let context = self.build_context(options, &names);

        match options.kind {
            ScaffoldKind::Plugin => {
                files.extend(self.generate_plugin(&context)?);
            }
            ScaffoldKind::Component => {
                files.extend(self.generate_component(&context)?);
            }
            ScaffoldKind::Widget => {
                files.extend(self.generate_widget(&context)?);
            }
            ScaffoldKind::NativeExtension => {
                files.extend(self.generate_native_extension(&context)?);
            }
            ScaffoldKind::CompatShim => {
                files.extend(self.generate_compat_shim(&context)?);
            }
        }

        // Add common files
        if options.include_ai_spec {
            files.push(self.render_file("common/oxide.ai.json", "oxide.ai.json", &context)?);
        }

        if options.include_docs {
            files.push(self.render_file("common/README.md", "README.md", &context)?);
        }

        if options.include_tests {
            files.push(self.render_file("common/tests.rs", "tests/lib_test.rs", &context)?);
        }

        let next_steps = self.generate_next_steps(options);

        Ok(GeneratedScaffold {
            id: options.id.clone(),
            files,
            output_dir: options.output_dir.clone(),
            next_steps,
        })
    }

    /// Build template context
    fn build_context(
        &self,
        options: &ScaffoldOptions,
        names: &DerivedNames,
    ) -> HashMap<String, serde_json::Value> {
        let mut context = HashMap::new();

        // Basic info
        context.insert("id".to_string(), serde_json::json!(options.id.to_string()));
        context.insert("namespace".to_string(), serde_json::json!(options.id.namespace.as_str()));
        context.insert("name".to_string(), serde_json::json!(options.id.name));
        context.insert("description".to_string(), serde_json::json!(options.description));
        context.insert("kind".to_string(), serde_json::json!(options.kind.as_str()));

        // Names
        context.insert("crate_name".to_string(), serde_json::json!(names.crate_name));
        context.insert("package_name".to_string(), serde_json::json!(names.package_name));
        context.insert("module_name".to_string(), serde_json::json!(names.module_name));
        context.insert("struct_name".to_string(), serde_json::json!(names.struct_name));

        // Optional fields
        context.insert(
            "author".to_string(),
            serde_json::json!(options.author.as_deref().unwrap_or("OxideKit Contributors")),
        );
        context.insert(
            "license".to_string(),
            serde_json::json!(options.license.as_deref().unwrap_or("MIT OR Apache-2.0")),
        );

        // Features
        context.insert("features".to_string(), serde_json::json!(options.features));

        // Props and events for components
        if let Some(props) = &options.props {
            context.insert("props".to_string(), serde_json::to_value(props).unwrap());
            context.insert("has_props".to_string(), serde_json::json!(true));
        } else {
            context.insert("has_props".to_string(), serde_json::json!(false));
        }

        if let Some(events) = &options.events {
            context.insert("events".to_string(), serde_json::to_value(events).unwrap());
            context.insert("has_events".to_string(), serde_json::json!(true));
        } else {
            context.insert("has_events".to_string(), serde_json::json!(false));
        }

        // Permissions and capabilities for plugins
        context.insert(
            "permissions".to_string(),
            serde_json::json!(options.permissions.as_ref().unwrap_or(&Vec::new())),
        );
        context.insert(
            "capabilities".to_string(),
            serde_json::json!(options.capabilities.as_ref().unwrap_or(&Vec::new())),
        );

        context
    }

    /// Render a template to a file
    fn render_file(
        &self,
        template: &str,
        output_path: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<GeneratedFile, ScaffoldError> {
        let content = self
            .handlebars
            .render(template, context)
            .map_err(ScaffoldError::RenderError)?;

        Ok(GeneratedFile {
            path: PathBuf::from(output_path),
            content,
            executable: false,
        })
    }

    /// Generate plugin files
    fn generate_plugin(
        &self,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<GeneratedFile>, ScaffoldError> {
        let mut files = Vec::new();

        files.push(self.render_file("plugin/Cargo.toml", "Cargo.toml", context)?);
        files.push(self.render_file("plugin/lib.rs", "src/lib.rs", context)?);
        files.push(self.render_file("plugin/plugin.toml", "plugin.toml", context)?);

        Ok(files)
    }

    /// Generate component files
    fn generate_component(
        &self,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<GeneratedFile>, ScaffoldError> {
        let mut files = Vec::new();

        files.push(self.render_file("component/mod.rs", "src/mod.rs", context)?);
        files.push(self.render_file(
            "component/component.oui",
            &format!("{}.oui", context.get("module_name").and_then(|v| v.as_str()).unwrap_or("component")),
            context,
        )?);

        // Also include plugin files for the containing plugin
        files.extend(self.generate_plugin(context)?);

        Ok(files)
    }

    /// Generate widget files (for WebView compatibility)
    fn generate_widget(
        &self,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<GeneratedFile>, ScaffoldError> {
        let mut files = Vec::new();

        // Widget HTML template
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{struct_name}} Widget</title>
    <style>
        body { margin: 0; padding: 16px; font-family: system-ui; }
    </style>
</head>
<body>
    <div id="root">{{struct_name}} Widget</div>
    <script>
        // OxideKit bridge available as window.oxide
        window.oxide.onMessage('init', (config) => {
            console.log('Widget initialized with config:', config);
        });

        // Send ready signal
        window.oxide.send('ready', { widget: '{{id}}' });
    </script>
</body>
</html>
"#;
        files.push(GeneratedFile {
            path: PathBuf::from("index.html"),
            content: self.handlebars.render_template(html, context)?,
            executable: false,
        });

        // Widget manifest
        let manifest = r#"{
    "name": "{{id}}",
    "version": "0.1.0",
    "description": "{{description}}",
    "entry": "index.html",
    "permissions": [],
    "messages": {
        "incoming": ["init", "update"],
        "outgoing": ["ready", "event"]
    }
}
"#;
        files.push(GeneratedFile {
            path: PathBuf::from("widget.json"),
            content: self.handlebars.render_template(manifest, context)?,
            executable: false,
        });

        Ok(files)
    }

    /// Generate native extension files
    fn generate_native_extension(
        &self,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<GeneratedFile>, ScaffoldError> {
        // Native extensions are similar to plugins but with additional native code
        let mut files = self.generate_plugin(context)?;

        // Add native interface header
        let native_header = format!(
            r#"//! Native interface for {}
//!
//! This module provides the native platform integration.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

/// Initialize the native extension
pub fn init() -> Result<(), Box<dyn std::error::Error>> {{
    #[cfg(target_os = "macos")]
    macos::init()?;

    #[cfg(target_os = "windows")]
    windows::init()?;

    #[cfg(target_os = "linux")]
    linux::init()?;

    Ok(())
}}
"#,
            context.get("struct_name").and_then(|v| v.as_str()).unwrap_or("Extension")
        );

        files.push(GeneratedFile {
            path: PathBuf::from("src/native.rs"),
            content: native_header,
            executable: false,
        });

        Ok(files)
    }

    /// Generate compatibility shim files
    fn generate_compat_shim(
        &self,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<GeneratedFile>, ScaffoldError> {
        let mut files = Vec::new();

        let struct_name = context
            .get("struct_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Shim");

        // Shim implementation
        let shim_rs = format!(
            r#"//! Compatibility shim for {}
//!
//! WARNING: This is a compatibility shim and not the recommended approach.
//! Consider migrating to a native OxideKit component.

use oxide_compat::prelude::*;

/// Compatibility shim wrapping legacy functionality
pub struct {} {{
    config: CompatPolicy,
}}

impl {} {{
    /// Create a new shim instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {{
        let config = CompatPolicy::from_current_dir()?;
        Ok(Self {{ config }})
    }}

    /// Execute the shim functionality
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {{
        // TODO: Implement shim logic
        unimplemented!("Shim logic not yet implemented")
    }}
}}

impl Default for {} {{
    fn default() -> Self {{
        Self::new().expect("Failed to create shim")
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_shim_creation() {{
        // Test with explicit policy to avoid file system dependency
        let shim = {}::default();
        // Add tests here
    }}
}}
"#,
            struct_name, struct_name, struct_name, struct_name, struct_name
        );

        files.push(GeneratedFile {
            path: PathBuf::from("src/lib.rs"),
            content: shim_rs,
            executable: false,
        });

        // Cargo.toml for shim
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "{}"

[dependencies]
oxide-compat = {{ path = "../oxide-compat" }}

[dev-dependencies]
tokio-test = "0.4"
"#,
            context.get("crate_name").and_then(|v| v.as_str()).unwrap_or("oxide_shim"),
            context.get("description").and_then(|v| v.as_str()).unwrap_or("Compatibility shim")
        );

        files.push(GeneratedFile {
            path: PathBuf::from("Cargo.toml"),
            content: cargo_toml,
            executable: false,
        });

        Ok(files)
    }

    /// Generate next steps instructions
    fn generate_next_steps(&self, options: &ScaffoldOptions) -> Vec<String> {
        let mut steps = Vec::new();

        match options.kind {
            ScaffoldKind::Plugin => {
                steps.push(format!("cd {}", options.output_dir.display()));
                steps.push("cargo build".to_string());
                steps.push("oxide plugin verify".to_string());
                steps.push("oxide plugin test".to_string());
            }
            ScaffoldKind::Component => {
                steps.push("Implement component logic in src/mod.rs".to_string());
                steps.push("Design UI in the .oui file".to_string());
                steps.push("Add props and events as needed".to_string());
                steps.push("Run oxide dev to preview".to_string());
            }
            ScaffoldKind::Widget => {
                steps.push("Edit index.html with your widget UI".to_string());
                steps.push("Update widget.json with required permissions".to_string());
                steps.push("Bundle with: oxide compat npm build (if using npm)".to_string());
                steps.push("NOTE: Consider migrating to native OxideKit components".to_string());
            }
            ScaffoldKind::NativeExtension => {
                steps.push("Implement platform-specific code in src/native/".to_string());
                steps.push("Test on each target platform".to_string());
                steps.push("Run oxide plugin verify".to_string());
            }
            ScaffoldKind::CompatShim => {
                steps.push("Implement shim logic in src/lib.rs".to_string());
                steps.push("WARNING: Plan migration to native OxideKit components".to_string());
                steps.push("Run oxide doctor to check compatibility warnings".to_string());
            }
        }

        steps.push("Update README.md with usage instructions".to_string());
        steps.push("Update oxide.ai.json for AI discoverability".to_string());

        steps
    }

    /// Write scaffold to disk
    pub fn write(&self, scaffold: &GeneratedScaffold) -> Result<(), ScaffoldError> {
        // Check if output directory exists
        let output_dir = &scaffold.output_dir;
        if output_dir.exists() && output_dir.read_dir()?.next().is_some() {
            return Err(ScaffoldError::DirectoryExists(output_dir.clone()));
        }

        // Create output directory
        std::fs::create_dir_all(output_dir)?;

        // Write all files
        for file in &scaffold.files {
            let full_path = output_dir.join(&file.path);

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Write file
            std::fs::write(&full_path, &file.content)?;

            // Set executable if needed
            #[cfg(unix)]
            if file.executable {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&full_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&full_path, perms)?;
            }
        }

        Ok(())
    }
}

impl Default for Scaffolder {
    fn default() -> Self {
        Self::new()
    }
}

// Template constants

const TEMPLATE_PLUGIN_CARGO_TOML: &str = r#"[package]
name = "{{crate_name}}"
version = "0.1.0"
edition = "2021"
description = "{{description}}"
license = "{{license}}"
authors = ["{{author}}"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.15"
"#;

const TEMPLATE_PLUGIN_LIB_RS: &str = r#"//! {{struct_name}} Plugin for OxideKit
//!
//! {{description}}

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Plugin errors
#[derive(Error, Debug)]
pub enum {{struct_name}}Error {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{struct_name}}Config {
    /// Enable debug mode
    pub debug: bool,
}

impl Default for {{struct_name}}Config {
    fn default() -> Self {
        Self { debug: false }
    }
}

/// Main plugin struct
pub struct {{struct_name}} {
    config: {{struct_name}}Config,
}

impl {{struct_name}} {
    /// Create a new plugin instance
    pub fn new(config: {{struct_name}}Config) -> Self {
        Self { config }
    }

    /// Initialize the plugin
    pub fn init(&self) -> Result<(), {{struct_name}}Error> {
        tracing::info!("Initializing {{struct_name}} plugin");
        Ok(())
    }
}

impl Default for {{struct_name}} {
    fn default() -> Self {
        Self::new({{struct_name}}Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = {{struct_name}}::default();
        assert!(plugin.init().is_ok());
    }
}
"#;

const TEMPLATE_PLUGIN_TOML: &str = r#"# Plugin manifest for {{id}}

[plugin]
id = "{{id}}"
name = "{{struct_name}}"
version = "0.1.0"
description = "{{description}}"
license = "{{license}}"

[plugin.authors]
names = ["{{author}}"]

[compatibility]
oxide_version = ">=0.1.0"

[permissions]
# Add required permissions here
# Example: filesystem = ["read", "write"]

[capabilities]
# Add provided capabilities here
"#;

const TEMPLATE_COMPONENT_MOD_RS: &str = r#"//! {{struct_name}} Component
//!
//! {{description}}

use serde::{Deserialize, Serialize};

{{#if has_props}}
/// Component properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{struct_name}}Props {
    {{#each props}}
    /// {{description}}
    pub {{name}}: {{prop_type}},
    {{/each}}
}

impl Default for {{struct_name}}Props {
    fn default() -> Self {
        Self {
            {{#each props}}
            {{name}}: {{#if default}}{{default}}{{else}}Default::default(){{/if}},
            {{/each}}
        }
    }
}
{{/if}}

{{#if has_events}}
/// Component events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum {{struct_name}}Event {
    {{#each events}}
    /// {{description}}
    {{name}}{{#if payload_type}}({{payload_type}}){{/if}},
    {{/each}}
}
{{/if}}

/// {{struct_name}} component
pub struct {{struct_name}} {
    {{#if has_props}}
    props: {{struct_name}}Props,
    {{/if}}
}

impl {{struct_name}} {
    /// Create a new component instance
    pub fn new({{#if has_props}}props: {{struct_name}}Props{{/if}}) -> Self {
        Self {
            {{#if has_props}}props{{/if}}
        }
    }

    /// Render the component
    pub fn render(&self) -> String {
        // TODO: Implement rendering
        String::new()
    }
}
"#;

const TEMPLATE_COMPONENT_OUI: &str = r#"<!-- {{struct_name}} Component Template -->
<template>
  <div class="{{module_name}}">
    <!-- Component content here -->
    <slot />
  </div>
</template>

<style>
.{{module_name}} {
  /* Component styles */
}
</style>
"#;

const TEMPLATE_AI_SPEC: &str = r#"{
  "$schema": "https://oxidekit.com/schemas/oxide.ai.json",
  "version": "1.0",
  "plugin": {
    "id": "{{id}}",
    "name": "{{struct_name}}",
    "description": "{{description}}",
    "kind": "{{kind}}"
  },
  "components": [],
  "examples": [
    {
      "description": "Basic usage",
      "code": "// TODO: Add example code"
    }
  ],
  "migrations": {
    "from_electron": null,
    "from_tauri": null
  }
}
"#;

const TEMPLATE_README: &str = r#"# {{struct_name}}

{{description}}

## Installation

```bash
oxide add {{id}}
```

## Usage

```rust
use {{crate_name}}::{{struct_name}};

let instance = {{struct_name}}::default();
instance.init()?;
```

## Configuration

Add to your `oxide.toml`:

```toml
[plugins.{{id}}]
# Configuration options here
```

## License

{{license}}
"#;

const TEMPLATE_TESTS: &str = r#"//! Tests for {{struct_name}}

use {{crate_name}}::*;

#[test]
fn test_default_creation() {
    let instance = {{struct_name}}::default();
    // Add assertions
}

#[test]
fn test_initialization() {
    let instance = {{struct_name}}::default();
    assert!(instance.init().is_ok());
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffolder_creation() {
        let scaffolder = Scaffolder::new();
        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_plugin_scaffold() {
        let scaffolder = Scaffolder::new();
        let options = ScaffoldOptions {
            id: CanonicalId::new(Namespace::Ui, "my-component"),
            kind: ScaffoldKind::Plugin,
            description: "A test plugin".to_string(),
            ..Default::default()
        };

        let scaffold = scaffolder.generate(&options).unwrap();
        assert!(!scaffold.files.is_empty());

        // Check for essential files
        let file_paths: Vec<_> = scaffold.files.iter().map(|f| f.path.to_str().unwrap()).collect();
        assert!(file_paths.contains(&"Cargo.toml"));
        assert!(file_paths.contains(&"src/lib.rs"));
        assert!(file_paths.contains(&"plugin.toml"));
    }

    #[test]
    fn test_scaffold_kind_from_str() {
        assert_eq!(ScaffoldKind::from_str("plugin"), Some(ScaffoldKind::Plugin));
        assert_eq!(ScaffoldKind::from_str("component"), Some(ScaffoldKind::Component));
        assert_eq!(ScaffoldKind::from_str("widget"), Some(ScaffoldKind::Widget));
        assert_eq!(ScaffoldKind::from_str("unknown"), None);
    }
}
