//! Plugin scaffolding and boilerplate generation.
//!
//! Generates new plugin projects with proper structure, manifests, and
//! starter code.
//!
//! # Usage
//!
//! ```bash
//! oxide plugin new ui.tables
//! oxide plugin new native.keychain
//! oxide plugin new theme.admin.modern
//! ```

use std::path::{Path, PathBuf};
use std::fs;
use tracing::info;

use crate::error::{PluginError, PluginResult};
use crate::PluginCategory;
use crate::namespace::{Namespace, PluginId};

/// Plugin scaffolding generator.
pub struct PluginScaffold {
    /// Output directory for the new plugin.
    output_dir: PathBuf,
}

impl PluginScaffold {
    /// Create a new scaffold generator.
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Generate a new plugin.
    pub fn generate(&self, plugin_id: &PluginId, options: &ScaffoldOptions) -> PluginResult<PathBuf> {
        let category = self.category_from_namespace(plugin_id.namespace());
        let plugin_dir = self.output_dir.join(plugin_id.name());

        // Check if directory already exists
        if plugin_dir.exists() {
            return Err(PluginError::InstallationFailed(format!(
                "Directory already exists: {}",
                plugin_dir.display()
            )));
        }

        // Create directory structure
        fs::create_dir_all(&plugin_dir)?;
        fs::create_dir_all(plugin_dir.join("src"))?;
        fs::create_dir_all(plugin_dir.join("tests"))?;

        // Generate files based on category
        match category {
            PluginCategory::Ui => self.generate_ui_plugin(plugin_id, &plugin_dir, options)?,
            PluginCategory::Native => self.generate_native_plugin(plugin_id, &plugin_dir, options)?,
            PluginCategory::Service => self.generate_service_plugin(plugin_id, &plugin_dir, options)?,
            PluginCategory::Tooling => self.generate_tooling_plugin(plugin_id, &plugin_dir, options)?,
            PluginCategory::Theme => self.generate_theme_plugin(plugin_id, &plugin_dir, options)?,
            PluginCategory::Design => self.generate_design_plugin(plugin_id, &plugin_dir, options)?,
        }

        // Generate common files
        self.generate_readme(plugin_id, &plugin_dir, &category)?;
        self.generate_license(&plugin_dir, &options.license)?;
        self.generate_gitignore(&plugin_dir)?;
        self.generate_ci_config(&plugin_dir)?;

        info!("Created plugin {} at {:?}", plugin_id, plugin_dir);

        Ok(plugin_dir)
    }

    /// Determine the plugin category from namespace.
    fn category_from_namespace(&self, namespace: Namespace) -> PluginCategory {
        match namespace {
            Namespace::Ui => PluginCategory::Ui,
            Namespace::Native => PluginCategory::Native,
            Namespace::Auth | Namespace::Db | Namespace::Data => PluginCategory::Service,
            Namespace::Tool => PluginCategory::Tooling,
            Namespace::Theme | Namespace::Icons | Namespace::Fonts => PluginCategory::Theme,
            Namespace::Design => PluginCategory::Design,
        }
    }

    /// Generate a UI plugin.
    fn generate_ui_plugin(
        &self,
        plugin_id: &PluginId,
        plugin_dir: &Path,
        options: &ScaffoldOptions,
    ) -> PluginResult<()> {
        // Generate plugin.toml
        let manifest = format!(
            r#"[plugin]
id = "{id}"
kind = "ui"
version = "0.1.0"
publisher = "{publisher}"
description = "{description}"
license = "{license}"
keywords = ["ui", "component"]

[plugin.requires]
core = ">=0.1.0"

[ui]
is_pack = false

[[ui.components]]
name = "{component_name}"
description = "{description}"

[[ui.components.props]]
name = "children"
type = "Children"
required = false
description = "Child elements"
"#,
            id = plugin_id.full_name(),
            publisher = options.publisher,
            description = options.description.as_deref().unwrap_or("A UI component"),
            license = options.license,
            component_name = to_pascal_case(plugin_id.name()),
        );

        fs::write(plugin_dir.join("plugin.toml"), manifest)?;

        // Generate src/lib.rs
        let lib_rs = format!(
            r#"//! {description}
//!
//! This is a UI component plugin for OxideKit.

/// The main component.
pub struct {component_name} {{
    // Component state
}}

impl {component_name} {{
    /// Create a new instance.
    pub fn new() -> Self {{
        Self {{}}
    }}

    /// Render the component.
    pub fn render(&self) {{
        // TODO: Implement rendering
    }}
}}

impl Default for {component_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_component_creation() {{
        let component = {component_name}::new();
        // Add tests
    }}
}}
"#,
            description = options.description.as_deref().unwrap_or("A UI component"),
            component_name = to_pascal_case(plugin_id.name()),
        );

        fs::write(plugin_dir.join("src/lib.rs"), lib_rs)?;

        // Generate Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "oxide-plugin-{name}"
version = "0.1.0"
edition = "2021"
license = "{license}"
description = "{description}"

[dependencies]
# Add your dependencies here

[dev-dependencies]
"#,
            name = plugin_id.name().replace('.', "-"),
            license = options.license,
            description = options.description.as_deref().unwrap_or("A UI component"),
        );

        fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)?;

        Ok(())
    }

    /// Generate a native plugin.
    fn generate_native_plugin(
        &self,
        plugin_id: &PluginId,
        plugin_dir: &Path,
        options: &ScaffoldOptions,
    ) -> PluginResult<()> {
        // Generate plugin.toml
        let manifest = format!(
            r#"[plugin]
id = "{id}"
kind = "native"
version = "0.1.0"
publisher = "{publisher}"
description = "{description}"
license = "{license}"
keywords = ["native", "os"]

[plugin.requires]
core = ">=0.1.0"

[native]
capabilities = ["filesystem.read"]  # Declare required capabilities

[native.platforms.macos]
min_os_version = "10.15"

[native.platforms.windows]
min_os_version = "10"

[native.platforms.linux]
"#,
            id = plugin_id.full_name(),
            publisher = options.publisher,
            description = options.description.as_deref().unwrap_or("A native capability plugin"),
            license = options.license,
        );

        fs::write(plugin_dir.join("plugin.toml"), manifest)?;

        // Generate src/lib.rs
        let lib_rs = format!(
            r#"//! {description}
//!
//! This is a native capability plugin for OxideKit.
//!
//! # Capabilities
//!
//! This plugin requires the following capabilities:
//! - `filesystem.read` - Read files from the filesystem

use std::path::Path;

/// The main service provided by this plugin.
pub struct {service_name} {{
    // Service state
}}

impl {service_name} {{
    /// Create a new instance.
    pub fn new() -> Self {{
        Self {{}}
    }}

    /// Example method using filesystem capability.
    pub fn read_file(&self, path: &Path) -> std::io::Result<String> {{
        std::fs::read_to_string(path)
    }}
}}

impl Default for {service_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_service_creation() {{
        let service = {service_name}::new();
        // Add tests
    }}
}}
"#,
            description = options.description.as_deref().unwrap_or("A native capability plugin"),
            service_name = to_pascal_case(plugin_id.name()),
        );

        fs::write(plugin_dir.join("src/lib.rs"), lib_rs)?;

        // Generate Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "oxide-plugin-{name}"
version = "0.1.0"
edition = "2021"
license = "{license}"
description = "{description}"

[dependencies]

[dev-dependencies]
"#,
            name = plugin_id.name().replace('.', "-"),
            license = options.license,
            description = options.description.as_deref().unwrap_or("A native capability plugin"),
        );

        fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)?;

        Ok(())
    }

    /// Generate a service plugin.
    fn generate_service_plugin(
        &self,
        plugin_id: &PluginId,
        plugin_dir: &Path,
        options: &ScaffoldOptions,
    ) -> PluginResult<()> {
        // Generate plugin.toml
        let manifest = format!(
            r#"[plugin]
id = "{id}"
kind = "service"
version = "0.1.0"
publisher = "{publisher}"
description = "{description}"
license = "{license}"
keywords = ["service"]

[plugin.requires]
core = ">=0.1.0"

[service]

[[service.entrypoints]]
name = "init"
type = "init"
description = "Initialize the service"
"#,
            id = plugin_id.full_name(),
            publisher = options.publisher,
            description = options.description.as_deref().unwrap_or("A service plugin"),
            license = options.license,
        );

        fs::write(plugin_dir.join("plugin.toml"), manifest)?;

        // Generate src/lib.rs
        let lib_rs = format!(
            r#"//! {description}
//!
//! This is a service plugin for OxideKit.

/// The main service.
pub struct {service_name} {{
    initialized: bool,
}}

impl {service_name} {{
    /// Create a new instance.
    pub fn new() -> Self {{
        Self {{
            initialized: false,
        }}
    }}

    /// Initialize the service.
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {{
        self.initialized = true;
        Ok(())
    }}

    /// Check if the service is initialized.
    pub fn is_initialized(&self) -> bool {{
        self.initialized
    }}
}}

impl Default for {service_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_service_init() {{
        let mut service = {service_name}::new();
        assert!(!service.is_initialized());
        service.init().unwrap();
        assert!(service.is_initialized());
    }}
}}
"#,
            description = options.description.as_deref().unwrap_or("A service plugin"),
            service_name = to_pascal_case(plugin_id.name()),
        );

        fs::write(plugin_dir.join("src/lib.rs"), lib_rs)?;

        // Generate Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "oxide-plugin-{name}"
version = "0.1.0"
edition = "2021"
license = "{license}"
description = "{description}"

[dependencies]

[dev-dependencies]
"#,
            name = plugin_id.name().replace('.', "-"),
            license = options.license,
            description = options.description.as_deref().unwrap_or("A service plugin"),
        );

        fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)?;

        Ok(())
    }

    /// Generate a tooling plugin.
    fn generate_tooling_plugin(
        &self,
        plugin_id: &PluginId,
        plugin_dir: &Path,
        options: &ScaffoldOptions,
    ) -> PluginResult<()> {
        // Generate plugin.toml
        let manifest = format!(
            r#"[plugin]
id = "{id}"
kind = "tooling"
version = "0.1.0"
publisher = "{publisher}"
description = "{description}"
license = "{license}"
keywords = ["tool", "dev"]

[plugin.requires]
core = ">=0.1.0"

[tooling]

[[tooling.commands]]
name = "{command_name}"
description = "Run the tool"

[[tooling.commands.args]]
name = "input"
type = "string"
required = true
description = "Input file or directory"

[[tooling.hooks]]
event = "pre-build"
priority = 0
description = "Run before build"
"#,
            id = plugin_id.full_name(),
            publisher = options.publisher,
            description = options.description.as_deref().unwrap_or("A tooling plugin"),
            license = options.license,
            command_name = plugin_id.name().replace('.', "-"),
        );

        fs::write(plugin_dir.join("plugin.toml"), manifest)?;

        // Generate src/lib.rs
        let lib_rs = format!(
            r#"//! {description}
//!
//! This is a tooling plugin for OxideKit.

use std::path::Path;

/// The main tool implementation.
pub struct {tool_name} {{
    // Tool state
}}

impl {tool_name} {{
    /// Create a new instance.
    pub fn new() -> Self {{
        Self {{}}
    }}

    /// Run the tool on the given input.
    pub fn run(&self, input: &Path) -> Result<(), Box<dyn std::error::Error>> {{
        println!("Processing: {{}}", input.display());
        // TODO: Implement tool logic
        Ok(())
    }}

    /// Hook called before build.
    pub fn pre_build(&self) -> Result<(), Box<dyn std::error::Error>> {{
        // TODO: Implement pre-build hook
        Ok(())
    }}
}}

impl Default for {tool_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_tool_creation() {{
        let tool = {tool_name}::new();
        // Add tests
    }}
}}
"#,
            description = options.description.as_deref().unwrap_or("A tooling plugin"),
            tool_name = to_pascal_case(plugin_id.name()),
        );

        fs::write(plugin_dir.join("src/lib.rs"), lib_rs)?;

        // Generate Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "oxide-plugin-{name}"
version = "0.1.0"
edition = "2021"
license = "{license}"
description = "{description}"

[dependencies]

[dev-dependencies]
"#,
            name = plugin_id.name().replace('.', "-"),
            license = options.license,
            description = options.description.as_deref().unwrap_or("A tooling plugin"),
        );

        fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)?;

        Ok(())
    }

    /// Generate a theme plugin.
    fn generate_theme_plugin(
        &self,
        plugin_id: &PluginId,
        plugin_dir: &Path,
        options: &ScaffoldOptions,
    ) -> PluginResult<()> {
        // Create additional directories
        fs::create_dir_all(plugin_dir.join("tokens"))?;
        fs::create_dir_all(plugin_dir.join("typography"))?;
        fs::create_dir_all(plugin_dir.join("previews"))?;

        // Generate plugin.toml
        let manifest = format!(
            r#"[plugin]
id = "{id}"
kind = "theme"
version = "0.1.0"
publisher = "{publisher}"
description = "{description}"
license = "{license}"
keywords = ["theme", "design-tokens"]

[plugin.requires]
core = ">=0.1.0"

[theme]
tokens = ["tokens/colors.toml", "tokens/spacing.toml"]
typography = ["typography/roles.toml"]
color_schemes = ["light", "dark"]
previews = ["previews/preview.png"]
"#,
            id = plugin_id.full_name(),
            publisher = options.publisher,
            description = options.description.as_deref().unwrap_or("A theme pack"),
            license = options.license,
        );

        fs::write(plugin_dir.join("plugin.toml"), manifest)?;

        // Generate tokens/colors.toml
        let colors = r##"# Color tokens

[colors.primary]
50 = "#e3f2fd"
100 = "#bbdefb"
200 = "#90caf9"
300 = "#64b5f6"
400 = "#42a5f5"
500 = "#2196f3"
600 = "#1e88e5"
700 = "#1976d2"
800 = "#1565c0"
900 = "#0d47a1"

[colors.neutral]
50 = "#fafafa"
100 = "#f5f5f5"
200 = "#eeeeee"
300 = "#e0e0e0"
400 = "#bdbdbd"
500 = "#9e9e9e"
600 = "#757575"
700 = "#616161"
800 = "#424242"
900 = "#212121"

[colors.semantic]
success = "#4caf50"
warning = "#ff9800"
error = "#f44336"
info = "#2196f3"
"##;

        fs::write(plugin_dir.join("tokens/colors.toml"), colors)?;

        // Generate tokens/spacing.toml
        let spacing = r#"# Spacing tokens

[spacing]
xs = "4px"
sm = "8px"
md = "16px"
lg = "24px"
xl = "32px"
"2xl" = "48px"
"3xl" = "64px"

[radius]
none = "0"
sm = "4px"
md = "8px"
lg = "16px"
full = "9999px"
"#;

        fs::write(plugin_dir.join("tokens/spacing.toml"), spacing)?;

        // Generate typography/roles.toml
        let typography = r#"# Typography roles

[roles.display]
font_family = "system-ui"
font_weight = "700"
font_size = "48px"
line_height = "1.2"
letter_spacing = "-0.02em"

[roles.heading]
font_family = "system-ui"
font_weight = "600"
font_size = "24px"
line_height = "1.3"

[roles.body]
font_family = "system-ui"
font_weight = "400"
font_size = "16px"
line_height = "1.5"

[roles.caption]
font_family = "system-ui"
font_weight = "400"
font_size = "12px"
line_height = "1.4"
"#;

        fs::write(plugin_dir.join("typography/roles.toml"), typography)?;

        Ok(())
    }

    /// Generate a design plugin.
    fn generate_design_plugin(
        &self,
        plugin_id: &PluginId,
        plugin_dir: &Path,
        options: &ScaffoldOptions,
    ) -> PluginResult<()> {
        // Create additional directories
        fs::create_dir_all(plugin_dir.join("parts/sidebar"))?;
        fs::create_dir_all(plugin_dir.join("parts/header"))?;
        fs::create_dir_all(plugin_dir.join("parts/dashboard"))?;
        fs::create_dir_all(plugin_dir.join("previews"))?;

        // Generate plugin.toml
        let manifest = format!(
            r#"[plugin]
id = "{id}"
kind = "design"
version = "0.1.0"
publisher = "{publisher}"
description = "{description}"
license = "{license}"
keywords = ["design", "template", "admin"]

[plugin.requires]
core = ">=0.1.0"

[design]
template_type = "admin"
previews = ["previews/full.png", "previews/mobile.png"]

[[design.parts]]
name = "sidebar"
tags = ["navigation", "layout"]
description = "Collapsible sidebar navigation"
path = "parts/sidebar"

[[design.parts]]
name = "header"
tags = ["navigation", "layout"]
description = "Top header with user menu"
path = "parts/header"

[[design.parts]]
name = "dashboard"
tags = ["page", "charts"]
description = "Dashboard layout with widgets"
path = "parts/dashboard"
"#,
            id = plugin_id.full_name(),
            publisher = options.publisher,
            description = options.description.as_deref().unwrap_or("A design template"),
            license = options.license,
        );

        fs::write(plugin_dir.join("plugin.toml"), manifest)?;

        // Generate placeholder files for parts
        let sidebar_placeholder = r#"<!-- Sidebar component -->
<nav class="sidebar">
  <div class="sidebar-header">
    <h1>Logo</h1>
  </div>
  <ul class="sidebar-menu">
    <li><a href="/">Dashboard</a></li>
    <li><a href="/users">Users</a></li>
    <li><a href="/settings">Settings</a></li>
  </ul>
</nav>
"#;

        fs::write(plugin_dir.join("parts/sidebar/sidebar.html"), sidebar_placeholder)?;

        let header_placeholder = r#"<!-- Header component -->
<header class="header">
  <div class="header-left">
    <button class="menu-toggle">Menu</button>
  </div>
  <div class="header-right">
    <span class="user-name">User</span>
  </div>
</header>
"#;

        fs::write(plugin_dir.join("parts/header/header.html"), header_placeholder)?;

        Ok(())
    }

    /// Generate README.md.
    fn generate_readme(
        &self,
        plugin_id: &PluginId,
        plugin_dir: &Path,
        category: &PluginCategory,
    ) -> PluginResult<()> {
        let readme = format!(
            r#"# {title}

{description}

## Installation

```bash
oxide add {id}
```

## Usage

```rust
// TODO: Add usage examples
```

## Category

This is a **{category}** plugin.

## License

{license}
"#,
            title = to_title_case(plugin_id.name()),
            description = format!("A {} plugin for OxideKit.", category.description().to_lowercase()),
            id = plugin_id.full_name(),
            category = category,
            license = "MIT OR Apache-2.0",
        );

        fs::write(plugin_dir.join("README.md"), readme)?;
        Ok(())
    }

    /// Generate license file.
    fn generate_license(&self, plugin_dir: &Path, license: &str) -> PluginResult<()> {
        let license_text = match license {
            "MIT" => include_str!("templates/LICENSE-MIT"),
            "Apache-2.0" => include_str!("templates/LICENSE-APACHE"),
            _ => "See LICENSE file for license information.",
        };

        fs::write(plugin_dir.join("LICENSE"), license_text)?;
        Ok(())
    }

    /// Generate .gitignore.
    fn generate_gitignore(&self, plugin_dir: &Path) -> PluginResult<()> {
        let gitignore = r#"/target
Cargo.lock
*.swp
*.swo
.DS_Store
.idea/
.vscode/
"#;

        fs::write(plugin_dir.join(".gitignore"), gitignore)?;
        Ok(())
    }

    /// Generate CI configuration.
    fn generate_ci_config(&self, plugin_dir: &Path) -> PluginResult<()> {
        fs::create_dir_all(plugin_dir.join(".github/workflows"))?;

        let ci_yml = r#"name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check

  publish:
    needs: test
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: oxide publish
        env:
          OXIDE_TOKEN: ${{ secrets.OXIDE_TOKEN }}
"#;

        fs::write(plugin_dir.join(".github/workflows/ci.yml"), ci_yml)?;
        Ok(())
    }
}

/// Options for plugin scaffolding.
#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    /// Publisher name.
    pub publisher: String,
    /// Optional description.
    pub description: Option<String>,
    /// License (default: MIT).
    pub license: String,
    /// Whether to initialize a git repository.
    pub init_git: bool,
}

impl Default for ScaffoldOptions {
    fn default() -> Self {
        Self {
            publisher: "your-name".to_string(),
            description: None,
            license: "MIT".to_string(),
            init_git: true,
        }
    }
}

impl ScaffoldOptions {
    /// Create options with a publisher name.
    pub fn with_publisher(publisher: &str) -> Self {
        Self {
            publisher: publisher.to_string(),
            ..Default::default()
        }
    }
}

/// Convert a name to PascalCase.
fn to_pascal_case(s: &str) -> String {
    s.split(['.', '-', '_'])
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Convert a name to Title Case.
fn to_title_case(s: &str) -> String {
    s.split(['.', '-', '_'])
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("data.tables"), "DataTables");
        assert_eq!(to_pascal_case("my-component"), "MyComponent");
        assert_eq!(to_pascal_case("simple"), "Simple");
    }

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("data.tables"), "Data Tables");
        assert_eq!(to_title_case("my-component"), "My Component");
    }

    #[test]
    fn test_scaffold_ui_plugin() {
        let dir = tempdir().unwrap();
        let scaffold = PluginScaffold::new(dir.path());

        let plugin_id = PluginId::parse("ui.test").unwrap();
        let options = ScaffoldOptions::with_publisher("test-publisher");

        let result = scaffold.generate(&plugin_id, &options);
        assert!(result.is_ok());

        let plugin_dir = result.unwrap();
        assert!(plugin_dir.join("plugin.toml").exists());
        assert!(plugin_dir.join("src/lib.rs").exists());
        assert!(plugin_dir.join("Cargo.toml").exists());
        assert!(plugin_dir.join("README.md").exists());
    }

    #[test]
    fn test_scaffold_theme_plugin() {
        let dir = tempdir().unwrap();
        let scaffold = PluginScaffold::new(dir.path());

        let plugin_id = PluginId::parse("theme.modern").unwrap();
        let options = ScaffoldOptions::with_publisher("test-publisher");

        let result = scaffold.generate(&plugin_id, &options);
        assert!(result.is_ok());

        let plugin_dir = result.unwrap();
        assert!(plugin_dir.join("plugin.toml").exists());
        assert!(plugin_dir.join("tokens/colors.toml").exists());
        assert!(plugin_dir.join("typography/roles.toml").exists());
    }
}
