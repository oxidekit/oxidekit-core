//! Full Migration Converter
//!
//! Generates complete OxideKit project output from analysis, tokens, and mappings.
//! Creates starter-compatible projects with extracted themes and placeholder pages.

use crate::analyzer::AnalysisResult;
use crate::error::{IssueCategory, MigrateError, MigrateResult, MigrationIssue};
use crate::mapper::{MappingResult, LayoutPattern};
use crate::tokens::ExtractedTokens;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Migration output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Starter template to use
    pub starter: StarterTemplate,
    /// Output directory path
    pub output_dir: PathBuf,
    /// Project name
    pub project_name: String,
    /// Apply extracted theme
    pub apply_theme: bool,
    /// Generate placeholder pages
    pub generate_placeholders: bool,
    /// Include migration TODO list
    pub include_todos: bool,
    /// Overwrite existing files
    pub overwrite: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            starter: StarterTemplate::AdminPanel,
            output_dir: PathBuf::from("./migrated"),
            project_name: "migrated-app".into(),
            apply_theme: true,
            generate_placeholders: true,
            include_todos: true,
            overwrite: false,
        }
    }
}

/// Available starter templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StarterTemplate {
    /// Admin panel/dashboard template
    AdminPanel,
    /// Landing page template
    LandingPage,
    /// E-commerce storefront
    Ecommerce,
    /// Blog/content site
    Blog,
    /// Minimal/empty template
    Minimal,
}

impl Default for StarterTemplate {
    fn default() -> Self {
        StarterTemplate::AdminPanel
    }
}

impl std::fmt::Display for StarterTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StarterTemplate::AdminPanel => write!(f, "admin-panel"),
            StarterTemplate::LandingPage => write!(f, "landing-page"),
            StarterTemplate::Ecommerce => write!(f, "ecommerce"),
            StarterTemplate::Blog => write!(f, "blog"),
            StarterTemplate::Minimal => write!(f, "minimal"),
        }
    }
}

impl StarterTemplate {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin-panel" | "admin" | "dashboard" => Some(StarterTemplate::AdminPanel),
            "landing-page" | "landing" => Some(StarterTemplate::LandingPage),
            "ecommerce" | "store" | "shop" => Some(StarterTemplate::Ecommerce),
            "blog" | "content" => Some(StarterTemplate::Blog),
            "minimal" | "empty" | "blank" => Some(StarterTemplate::Minimal),
            _ => None,
        }
    }
}

/// Migration output result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationOutput {
    /// Files generated
    pub files: Vec<GeneratedFile>,
    /// Directories created
    pub directories: Vec<PathBuf>,
    /// TODO items for manual completion
    pub todos: Vec<TodoItem>,
    /// Migration issues/warnings
    pub issues: Vec<MigrationIssue>,
    /// Summary statistics
    pub summary: MigrationSummary,
}

/// A generated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// Relative path from output directory
    pub path: PathBuf,
    /// File contents
    pub content: String,
    /// File type/category
    pub file_type: FileType,
    /// Generation confidence
    pub confidence: f32,
    /// Needs manual review
    pub needs_review: bool,
}

/// File type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    /// Theme configuration
    Theme,
    /// Typography configuration
    Typography,
    /// Font configuration
    Fonts,
    /// Design pack part
    DesignPart,
    /// Component definition
    Component,
    /// Page/view
    Page,
    /// Layout definition
    Layout,
    /// Configuration file
    Config,
    /// Documentation
    Documentation,
}

/// A TODO item for manual completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// TODO description
    pub description: String,
    /// Priority (1 = highest)
    pub priority: u8,
    /// Category
    pub category: TodoCategory,
    /// Related file (if any)
    pub file: Option<PathBuf>,
    /// Estimated effort
    pub effort: Effort,
}

/// TODO categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoCategory {
    /// Design/styling
    Design,
    /// Component implementation
    Component,
    /// Data/state management
    Data,
    /// Integration
    Integration,
    /// Testing
    Testing,
    /// Documentation
    Documentation,
}

impl std::fmt::Display for TodoCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoCategory::Design => write!(f, "Design"),
            TodoCategory::Component => write!(f, "Component"),
            TodoCategory::Data => write!(f, "Data"),
            TodoCategory::Integration => write!(f, "Integration"),
            TodoCategory::Testing => write!(f, "Testing"),
            TodoCategory::Documentation => write!(f, "Documentation"),
        }
    }
}

/// Effort estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effort {
    /// < 30 minutes
    Low,
    /// 30 min - 2 hours
    Medium,
    /// > 2 hours
    High,
}

impl std::fmt::Display for Effort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effort::Low => write!(f, "Low (< 30 min)"),
            Effort::Medium => write!(f, "Medium (30 min - 2 hrs)"),
            Effort::High => write!(f, "High (> 2 hrs)"),
        }
    }
}

/// Migration summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationSummary {
    /// Total files generated
    pub files_generated: usize,
    /// Files needing review
    pub files_needing_review: usize,
    /// Components mapped automatically
    pub components_auto_mapped: usize,
    /// Components needing manual work
    pub components_manual: usize,
    /// Total TODO items
    pub todo_count: usize,
    /// Estimated total effort
    pub estimated_effort: String,
    /// Overall migration completeness (0-100%)
    pub completeness_percent: u8,
}

/// Migration converter
pub struct Converter {
    /// Configuration
    config: MigrationConfig,
}

impl Converter {
    /// Create a new converter with configuration
    pub fn new(config: MigrationConfig) -> Self {
        Self { config }
    }

    /// Create converter with default configuration
    pub fn with_defaults() -> Self {
        Self::new(MigrationConfig::default())
    }

    /// Set the output directory
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.output_dir = path.into();
        self
    }

    /// Set the project name
    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.config.project_name = name.into();
        self
    }

    /// Set the starter template
    pub fn starter(mut self, starter: StarterTemplate) -> Self {
        self.config.starter = starter;
        self
    }

    /// Convert analysis, tokens, and mappings to OxideKit project
    pub fn convert(
        &self,
        analysis: &AnalysisResult,
        tokens: &ExtractedTokens,
        mappings: &MappingResult,
    ) -> MigrateResult<MigrationOutput> {
        let mut output = MigrationOutput {
            files: Vec::new(),
            directories: Vec::new(),
            todos: Vec::new(),
            issues: Vec::new(),
            summary: MigrationSummary::default(),
        };

        // Create directory structure
        self.create_directories(&mut output)?;

        // Generate project manifest
        self.generate_manifest(analysis, &mut output)?;

        // Generate theme files
        self.generate_theme_files(tokens, &mut output)?;

        // Generate design pack parts
        self.generate_design_parts(mappings, &mut output)?;

        // Generate layout files
        self.generate_layouts(mappings, &mut output)?;

        // Generate placeholder pages
        if self.config.generate_placeholders {
            self.generate_placeholder_pages(analysis, mappings, &mut output)?;
        }

        // Generate TODO list
        if self.config.include_todos {
            self.generate_todos(analysis, tokens, mappings, &mut output)?;
        }

        // Generate migration documentation
        self.generate_documentation(analysis, mappings, &mut output)?;

        // Calculate summary
        self.calculate_summary(analysis, mappings, &mut output);

        Ok(output)
    }

    /// Write migration output to filesystem
    pub fn write_output(&self, output: &MigrationOutput) -> MigrateResult<()> {
        let base_path = &self.config.output_dir;

        // Create directories
        for dir in &output.directories {
            let full_path = base_path.join(dir);
            if !full_path.exists() {
                fs::create_dir_all(&full_path)?;
            }
        }

        // Write files
        for file in &output.files {
            let full_path = base_path.join(&file.path);

            // Check if file exists and overwrite is disabled
            if full_path.exists() && !self.config.overwrite {
                return Err(MigrateError::OutputGeneration(format!(
                    "File already exists and overwrite is disabled: {}",
                    full_path.display()
                )));
            }

            // Ensure parent directory exists
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&full_path, &file.content)?;
        }

        Ok(())
    }

    /// Create directory structure
    fn create_directories(&self, output: &mut MigrationOutput) -> MigrateResult<()> {
        let dirs = vec![
            PathBuf::from("src"),
            PathBuf::from("src/pages"),
            PathBuf::from("src/components"),
            PathBuf::from("src/layouts"),
            PathBuf::from("design"),
            PathBuf::from("design/parts"),
            PathBuf::from("theme"),
            PathBuf::from("assets"),
            PathBuf::from("assets/fonts"),
        ];

        output.directories = dirs;
        Ok(())
    }

    /// Generate project manifest
    fn generate_manifest(
        &self,
        analysis: &AnalysisResult,
        output: &mut MigrationOutput,
    ) -> MigrateResult<()> {
        let manifest = format!(
            r#"# OxideKit Application Manifest
# Generated by oxide-migrate from {} template

[app]
name = "{}"
version = "0.1.0"
description = "Migrated from {}"

[build]
starter = "{}"
theme = "theme/theme.generated.toml"

[features]
# Enable features based on detected components
sidebar = {}
navbar = {}
data_tables = {}
forms = {}

[migration]
source_framework = "{}"
migration_confidence = {:.2}
generated_at = "{}"
"#,
            analysis.framework,
            self.config.project_name,
            analysis.framework,
            self.config.starter,
            analysis
                .components
                .iter()
                .any(|c| c.component_type == crate::analyzer::ComponentType::Sidebar),
            analysis
                .components
                .iter()
                .any(|c| c.component_type == crate::analyzer::ComponentType::Navbar),
            analysis
                .components
                .iter()
                .any(|c| c.component_type == crate::analyzer::ComponentType::Table
                    || c.component_type == crate::analyzer::ComponentType::DataTable),
            analysis
                .components
                .iter()
                .any(|c| c.component_type == crate::analyzer::ComponentType::Form),
            analysis.framework,
            analysis.migration_confidence,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        );

        output.files.push(GeneratedFile {
            path: PathBuf::from("oxide.toml"),
            content: manifest,
            file_type: FileType::Config,
            confidence: 1.0,
            needs_review: false,
        });

        Ok(())
    }

    /// Generate theme files
    fn generate_theme_files(
        &self,
        tokens: &ExtractedTokens,
        output: &mut MigrationOutput,
    ) -> MigrateResult<()> {
        // Main theme file
        let theme_toml = toml::to_string_pretty(&tokens.theme)?;
        output.files.push(GeneratedFile {
            path: PathBuf::from("theme/theme.generated.toml"),
            content: theme_toml,
            file_type: FileType::Theme,
            confidence: tokens.confidence.overall,
            needs_review: tokens.confidence.overall < 0.7,
        });

        // Typography file
        let typography_toml = toml::to_string_pretty(&tokens.typography)?;
        output.files.push(GeneratedFile {
            path: PathBuf::from("theme/typography.generated.toml"),
            content: typography_toml,
            file_type: FileType::Typography,
            confidence: tokens.confidence.typography,
            needs_review: tokens.confidence.typography < 0.7,
        });

        // Fonts file
        let fonts_toml = toml::to_string_pretty(&tokens.fonts)?;
        output.files.push(GeneratedFile {
            path: PathBuf::from("theme/fonts.generated.toml"),
            content: fonts_toml,
            file_type: FileType::Fonts,
            confidence: tokens.confidence.typography,
            needs_review: tokens.fonts.all_fonts.is_empty(),
        });

        Ok(())
    }

    /// Generate design pack parts
    fn generate_design_parts(
        &self,
        mappings: &MappingResult,
        output: &mut MigrationOutput,
    ) -> MigrateResult<()> {
        for part in &mappings.design_parts {
            let file_name = part.tag.replace("part:", "") + ".toml";
            output.files.push(GeneratedFile {
                path: PathBuf::from(format!("design/parts/{}", file_name)),
                content: part.config_toml.clone(),
                file_type: FileType::DesignPart,
                confidence: part.confidence,
                needs_review: part.confidence < 0.7,
            });
        }

        // Generate main design pack index
        let parts_list: Vec<String> = mappings
            .design_parts
            .iter()
            .map(|p| format!("\"parts/{}\"", p.tag.replace("part:", "") + ".toml"))
            .collect();

        let design_index = format!(
            r#"# OxideKit Design Pack
# Generated by oxide-migrate

[design_pack]
name = "{}-design"
version = "1.0.0"
description = "Migrated design pack"

[parts]
includes = [
    {}
]

[layout]
pattern = "{:?}"
"#,
            self.config.project_name,
            parts_list.join(",\n    "),
            mappings.layout.pattern,
        );

        output.files.push(GeneratedFile {
            path: PathBuf::from("design/design-pack.toml"),
            content: design_index,
            file_type: FileType::Config,
            confidence: mappings.confidence,
            needs_review: false,
        });

        Ok(())
    }

    /// Generate layout files
    fn generate_layouts(
        &self,
        mappings: &MappingResult,
        output: &mut MigrationOutput,
    ) -> MigrateResult<()> {
        let layout = &mappings.layout;

        // Generate main layout based on pattern
        let main_layout = match layout.pattern {
            LayoutPattern::FullDashboard => self.generate_dashboard_layout(layout),
            LayoutPattern::SidebarLayout => self.generate_sidebar_layout(layout),
            LayoutPattern::NavbarLayout => self.generate_navbar_layout(layout),
            LayoutPattern::GridLayout => self.generate_grid_layout(layout),
            LayoutPattern::LandingPage => self.generate_landing_layout(layout),
            LayoutPattern::SingleColumn | LayoutPattern::Unknown => {
                self.generate_simple_layout(layout)
            }
        };

        output.files.push(GeneratedFile {
            path: PathBuf::from("src/layouts/main.oxide"),
            content: main_layout,
            file_type: FileType::Layout,
            confidence: 0.85,
            needs_review: true,
        });

        Ok(())
    }

    fn generate_dashboard_layout(&self, layout: &crate::mapper::LayoutMapping) -> String {
        let sidebar_width = layout
            .sidebar
            .as_ref()
            .and_then(|s| s.width)
            .unwrap_or(256.0);

        format!(
            r#"// Main Dashboard Layout
// Generated by oxide-migrate

<layout name="dashboard">
  <ui.Flex direction="row" height="100vh">
    // Sidebar
    <ui.Sidenav width="{}" collapsible>
      <slot name="sidebar" />
    </ui.Sidenav>

    <ui.Flex direction="column" flex="1">
      // Top navbar
      <ui.Toolbar fixed>
        <slot name="navbar" />
      </ui.Toolbar>

      // Main content area
      <ui.Box flex="1" overflow="auto" padding="md">
        <ui.Container maxWidth="1280">
          <slot name="breadcrumbs" />
          <slot name="content" />
        </ui.Container>
      </ui.Box>
    </ui.Flex>
  </ui.Flex>
</layout>
"#,
            sidebar_width
        )
    }

    fn generate_sidebar_layout(&self, layout: &crate::mapper::LayoutMapping) -> String {
        let sidebar_width = layout
            .sidebar
            .as_ref()
            .and_then(|s| s.width)
            .unwrap_or(256.0);

        format!(
            r#"// Sidebar Layout
// Generated by oxide-migrate

<layout name="sidebar">
  <ui.Flex direction="row" height="100vh">
    // Sidebar
    <ui.Sidenav width="{}" collapsible>
      <slot name="sidebar" />
    </ui.Sidenav>

    // Main content area
    <ui.Box flex="1" overflow="auto" padding="md">
      <ui.Container>
        <slot name="content" />
      </ui.Container>
    </ui.Box>
  </ui.Flex>
</layout>
"#,
            sidebar_width
        )
    }

    fn generate_navbar_layout(&self, _layout: &crate::mapper::LayoutMapping) -> String {
        r#"// Navbar Layout
// Generated by oxide-migrate

<layout name="navbar">
  <ui.Flex direction="column" height="100vh">
    // Top navbar
    <ui.Toolbar fixed>
      <slot name="navbar" />
    </ui.Toolbar>

    // Main content area
    <ui.Box flex="1" overflow="auto" padding="md">
      <ui.Container>
        <slot name="content" />
      </ui.Container>
    </ui.Box>

    // Optional footer
    <ui.Box as="footer" padding="md">
      <slot name="footer" />
    </ui.Box>
  </ui.Flex>
</layout>
"#
        .into()
    }

    fn generate_grid_layout(&self, layout: &crate::mapper::LayoutMapping) -> String {
        let columns = layout.grid_system.columns;
        let gutter = layout.grid_system.gutter.unwrap_or(16.0);

        format!(
            r#"// Grid Layout
// Generated by oxide-migrate

<layout name="grid">
  <ui.Container>
    <ui.Grid columns="{}" gap="{}">
      <slot name="content" />
    </ui.Grid>
  </ui.Container>
</layout>
"#,
            columns, gutter
        )
    }

    fn generate_landing_layout(&self, _layout: &crate::mapper::LayoutMapping) -> String {
        r#"// Landing Page Layout
// Generated by oxide-migrate

<layout name="landing">
  <ui.Flex direction="column" minHeight="100vh">
    // Header/navbar
    <ui.Toolbar transparent>
      <slot name="navbar" />
    </ui.Toolbar>

    // Hero section
    <ui.Box as="section">
      <slot name="hero" />
    </ui.Box>

    // Main content sections
    <ui.Box flex="1">
      <slot name="content" />
    </ui.Box>

    // Footer
    <ui.Box as="footer" padding="lg">
      <slot name="footer" />
    </ui.Box>
  </ui.Flex>
</layout>
"#
        .into()
    }

    fn generate_simple_layout(&self, _layout: &crate::mapper::LayoutMapping) -> String {
        r#"// Simple Layout
// Generated by oxide-migrate

<layout name="simple">
  <ui.Container maxWidth="960" padding="md">
    <slot name="content" />
  </ui.Container>
</layout>
"#
        .into()
    }

    /// Generate placeholder pages
    fn generate_placeholder_pages(
        &self,
        analysis: &AnalysisResult,
        mappings: &MappingResult,
        output: &mut MigrationOutput,
    ) -> MigrateResult<()> {
        // Dashboard/home page
        let dashboard_page = self.generate_dashboard_page(analysis, mappings);
        output.files.push(GeneratedFile {
            path: PathBuf::from("src/pages/dashboard.oxide"),
            content: dashboard_page,
            file_type: FileType::Page,
            confidence: 0.7,
            needs_review: true,
        });

        // If data tables detected, generate a table page
        let has_tables = analysis
            .components
            .iter()
            .any(|c| c.component_type == crate::analyzer::ComponentType::Table);
        if has_tables {
            let table_page = self.generate_table_page();
            output.files.push(GeneratedFile {
                path: PathBuf::from("src/pages/data-list.oxide"),
                content: table_page,
                file_type: FileType::Page,
                confidence: 0.65,
                needs_review: true,
            });
        }

        // If forms detected, generate a form page
        let has_forms = analysis
            .components
            .iter()
            .any(|c| c.component_type == crate::analyzer::ComponentType::Form);
        if has_forms {
            let form_page = self.generate_form_page();
            output.files.push(GeneratedFile {
                path: PathBuf::from("src/pages/form-example.oxide"),
                content: form_page,
                file_type: FileType::Page,
                confidence: 0.6,
                needs_review: true,
            });
        }

        Ok(())
    }

    fn generate_dashboard_page(
        &self,
        analysis: &AnalysisResult,
        _mappings: &MappingResult,
    ) -> String {
        let stats_section = if analysis
            .components
            .iter()
            .any(|c| c.component_type == crate::analyzer::ComponentType::Statistics)
        {
            r#"
    // Statistics cards
    <ui.Grid columns="4" gap="md">
      <ui.Card>
        <ui.Card.Body>
          <ui.Text variant="caption">Total Users</ui.Text>
          <ui.Text variant="title">1,234</ui.Text>
        </ui.Card.Body>
      </ui.Card>
      <ui.Card>
        <ui.Card.Body>
          <ui.Text variant="caption">Revenue</ui.Text>
          <ui.Text variant="title">$56,789</ui.Text>
        </ui.Card.Body>
      </ui.Card>
      <ui.Card>
        <ui.Card.Body>
          <ui.Text variant="caption">Orders</ui.Text>
          <ui.Text variant="title">892</ui.Text>
        </ui.Card.Body>
      </ui.Card>
      <ui.Card>
        <ui.Card.Body>
          <ui.Text variant="caption">Conversion</ui.Text>
          <ui.Text variant="title">3.24%</ui.Text>
        </ui.Card.Body>
      </ui.Card>
    </ui.Grid>
"#
        } else {
            ""
        };

        format!(
            r#"// Dashboard Page
// Generated by oxide-migrate
// TODO: Replace placeholder content with actual data

<page layout="dashboard" title="Dashboard">
  <slot name="content">
    <ui.Box marginBottom="lg">
      <ui.Text variant="heading">Dashboard</ui.Text>
      <ui.Text variant="body" color="text.secondary">
        Welcome to your migrated dashboard
      </ui.Text>
    </ui.Box>
{}
    // Main content area
    <ui.Grid columns="2" gap="md" marginTop="lg">
      <ui.Card>
        <ui.Card.Header>
          <ui.Text variant="subtitle">Recent Activity</ui.Text>
        </ui.Card.Header>
        <ui.Card.Body>
          <!-- TODO: Add activity list -->
          <ui.Text color="text.secondary">No recent activity</ui.Text>
        </ui.Card.Body>
      </ui.Card>

      <ui.Card>
        <ui.Card.Header>
          <ui.Text variant="subtitle">Quick Actions</ui.Text>
        </ui.Card.Header>
        <ui.Card.Body>
          <ui.Stack direction="column" gap="sm">
            <ui.Button variant="primary">Create New</ui.Button>
            <ui.Button variant="secondary">View Reports</ui.Button>
          </ui.Stack>
        </ui.Card.Body>
      </ui.Card>
    </ui.Grid>
  </slot>
</page>
"#,
            stats_section
        )
    }

    fn generate_table_page(&self) -> String {
        r#"// Data List Page
// Generated by oxide-migrate
// TODO: Replace with actual data source

<page layout="dashboard" title="Data List">
  <slot name="content">
    <ui.Box marginBottom="md">
      <ui.Flex justify="space-between" align="center">
        <ui.Text variant="heading">Data List</ui.Text>
        <ui.Button variant="primary">Add New</ui.Button>
      </ui.Flex>
    </ui.Box>

    <ui.Card>
      <ui.DataTable
        columns={[
          { key: "id", label: "ID", sortable: true },
          { key: "name", label: "Name", sortable: true },
          { key: "status", label: "Status" },
          { key: "date", label: "Date", sortable: true },
          { key: "actions", label: "Actions" }
        ]}
        data={[]}
        pagination
        searchable
      >
        <!-- TODO: Add row template -->
      </ui.DataTable>
    </ui.Card>
  </slot>
</page>
"#
        .into()
    }

    fn generate_form_page(&self) -> String {
        r#"// Form Example Page
// Generated by oxide-migrate
// TODO: Add form validation and submission logic

<page layout="dashboard" title="Form Example">
  <slot name="content">
    <ui.Box maxWidth="600">
      <ui.Text variant="heading" marginBottom="md">Form Example</ui.Text>

      <ui.Card>
        <ui.Card.Body>
          <ui.Form onSubmit={handleSubmit}>
            <ui.FormField label="Name" required>
              <ui.Input placeholder="Enter your name" />
            </ui.FormField>

            <ui.FormField label="Email" required>
              <ui.Input type="email" placeholder="Enter your email" />
            </ui.FormField>

            <ui.FormField label="Category">
              <ui.Select placeholder="Select a category">
                <ui.Select.Option value="1">Option 1</ui.Select.Option>
                <ui.Select.Option value="2">Option 2</ui.Select.Option>
                <ui.Select.Option value="3">Option 3</ui.Select.Option>
              </ui.Select>
            </ui.FormField>

            <ui.FormField label="Description">
              <ui.Textarea rows="4" placeholder="Enter description" />
            </ui.FormField>

            <ui.FormField>
              <ui.Checkbox label="I agree to the terms and conditions" />
            </ui.FormField>

            <ui.Flex gap="sm" marginTop="md">
              <ui.Button type="submit" variant="primary">Submit</ui.Button>
              <ui.Button type="reset" variant="secondary">Reset</ui.Button>
            </ui.Flex>
          </ui.Form>
        </ui.Card.Body>
      </ui.Card>
    </ui.Box>
  </slot>
</page>
"#
        .into()
    }

    /// Generate TODO list
    fn generate_todos(
        &self,
        analysis: &AnalysisResult,
        tokens: &ExtractedTokens,
        mappings: &MappingResult,
        output: &mut MigrationOutput,
    ) -> MigrateResult<()> {
        // Add TODO items based on migration issues
        for issue in &tokens.issues {
            if issue.severity >= crate::error::Severity::Warning {
                output.todos.push(TodoItem {
                    description: issue.message.clone(),
                    priority: match issue.severity {
                        crate::error::Severity::Error => 1,
                        crate::error::Severity::Warning => 2,
                        crate::error::Severity::Info => 3,
                    },
                    category: match issue.category {
                        IssueCategory::ColorToken | IssueCategory::Typography => TodoCategory::Design,
                        IssueCategory::ComponentMapping => TodoCategory::Component,
                        _ => TodoCategory::Design,
                    },
                    file: issue.source_file.as_ref().map(PathBuf::from),
                    effort: Effort::Medium,
                });
            }
        }

        // Add TODOs for components needing manual review
        for mapping in &mappings.components {
            if mapping.needs_review {
                output.todos.push(TodoItem {
                    description: format!(
                        "Review mapping for {:?} -> {}",
                        mapping.source_type, mapping.target_component
                    ),
                    priority: 2,
                    category: TodoCategory::Component,
                    file: None,
                    effort: Effort::Low,
                });
            }
        }

        // Add general TODOs
        output.todos.push(TodoItem {
            description: "Review and customize theme colors".into(),
            priority: 1,
            category: TodoCategory::Design,
            file: Some(PathBuf::from("theme/theme.generated.toml")),
            effort: Effort::Medium,
        });

        output.todos.push(TodoItem {
            description: "Add application data sources and state management".into(),
            priority: 1,
            category: TodoCategory::Data,
            file: None,
            effort: Effort::High,
        });

        output.todos.push(TodoItem {
            description: "Implement navigation and routing".into(),
            priority: 1,
            category: TodoCategory::Integration,
            file: None,
            effort: Effort::Medium,
        });

        output.todos.push(TodoItem {
            description: "Test all migrated components".into(),
            priority: 2,
            category: TodoCategory::Testing,
            file: None,
            effort: Effort::High,
        });

        // Check for unmapped components
        let unmapped_count = analysis.inventory.unmappable_count;
        if unmapped_count > 0 {
            output.todos.push(TodoItem {
                description: format!(
                    "Implement {} components that have no OxideKit equivalent",
                    unmapped_count
                ),
                priority: 1,
                category: TodoCategory::Component,
                file: None,
                effort: Effort::High,
            });
        }

        Ok(())
    }

    /// Generate migration documentation
    fn generate_documentation(
        &self,
        analysis: &AnalysisResult,
        mappings: &MappingResult,
        output: &mut MigrationOutput,
    ) -> MigrateResult<()> {
        let doc = format!(
            r#"# Migration Documentation

Generated by oxide-migrate on {}

## Source Analysis

- **Framework**: {}
- **Framework Version**: {}
- **Migration Confidence**: {:.0}%

## Files Analyzed

- HTML files: {}
- CSS files: {}
- JS/TS files: {}

## Components Detected

Total component types: {}
Total instances: {}

### Component Mapping Summary

| Component Type | Count | OxideKit Equivalent | Confidence |
|---------------|-------|---------------------|------------|
{}

## Layout Detection

- **Pattern**: {}
- **Has Sidebar**: {}
- **Has Navbar**: {}
- **Grid System**: {} columns

## Design Tokens Extracted

See `theme/theme.generated.toml` for the extracted design tokens.

### Colors

The following semantic colors were extracted:
- Primary, Secondary, Success, Warning, Danger, Info
- Background, Surface, Text, Border

### Typography

Font families, sizes, and weights were extracted to `theme/typography.generated.toml`.

## Next Steps

1. Review the generated theme files and customize as needed
2. Replace placeholder content with actual data
3. Implement navigation and routing
4. Test all migrated components
5. See `TODOS.md` for the complete migration checklist

## Known Limitations

- Some custom CSS may not be fully converted
- JavaScript functionality needs manual implementation
- Third-party component libraries may need alternatives

---

Generated by OxideKit Migrate Tool
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            analysis.framework,
            analysis.version,
            analysis.migration_confidence * 100.0,
            analysis.files_analyzed.html_files,
            analysis.files_analyzed.css_files,
            analysis.files_analyzed.js_files,
            analysis.inventory.total_types,
            analysis.inventory.total_instances,
            mappings
                .components
                .iter()
                .map(|m| format!(
                    "| {:?} | {} | {} | {:.0}% |",
                    m.source_type,
                    analysis
                        .components
                        .iter()
                        .find(|c| c.component_type == m.source_type)
                        .map(|c| c.occurrences)
                        .unwrap_or(0),
                    m.target_component,
                    m.confidence * 100.0
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            mappings.layout.pattern,
            mappings.layout.sidebar.is_some(),
            mappings.layout.navbar.is_some(),
            mappings.layout.grid_system.columns,
        );

        output.files.push(GeneratedFile {
            path: PathBuf::from("MIGRATION.md"),
            content: doc,
            file_type: FileType::Documentation,
            confidence: 1.0,
            needs_review: false,
        });

        // Generate TODOs file
        if !output.todos.is_empty() {
            let todos_md = self.generate_todos_markdown(&output.todos);
            output.files.push(GeneratedFile {
                path: PathBuf::from("TODOS.md"),
                content: todos_md,
                file_type: FileType::Documentation,
                confidence: 1.0,
                needs_review: false,
            });
        }

        Ok(())
    }

    fn generate_todos_markdown(&self, todos: &[TodoItem]) -> String {
        let mut md = String::from("# Migration TODO List\n\n");
        md.push_str("## Priority 1 (Critical)\n\n");

        for todo in todos.iter().filter(|t| t.priority == 1) {
            md.push_str(&format!(
                "- [ ] **[{}]** {} (Effort: {})\n",
                todo.category, todo.description, todo.effort
            ));
            if let Some(ref file) = todo.file {
                md.push_str(&format!("  - File: `{}`\n", file.display()));
            }
        }

        md.push_str("\n## Priority 2 (Important)\n\n");
        for todo in todos.iter().filter(|t| t.priority == 2) {
            md.push_str(&format!(
                "- [ ] **[{}]** {} (Effort: {})\n",
                todo.category, todo.description, todo.effort
            ));
        }

        md.push_str("\n## Priority 3 (Nice to have)\n\n");
        for todo in todos.iter().filter(|t| t.priority >= 3) {
            md.push_str(&format!(
                "- [ ] **[{}]** {} (Effort: {})\n",
                todo.category, todo.description, todo.effort
            ));
        }

        md
    }

    /// Calculate summary statistics
    fn calculate_summary(
        &self,
        analysis: &AnalysisResult,
        mappings: &MappingResult,
        output: &mut MigrationOutput,
    ) {
        output.summary.files_generated = output.files.len();
        output.summary.files_needing_review = output.files.iter().filter(|f| f.needs_review).count();
        output.summary.components_auto_mapped = mappings
            .components
            .iter()
            .filter(|c| !c.needs_review)
            .count();
        output.summary.components_manual = mappings.components.iter().filter(|c| c.needs_review).count();
        output.summary.todo_count = output.todos.len();

        // Estimate effort
        let high_count = output.todos.iter().filter(|t| t.effort == Effort::High).count();
        let medium_count = output.todos.iter().filter(|t| t.effort == Effort::Medium).count();
        let total_hours = high_count * 4 + medium_count * 1;
        output.summary.estimated_effort = if total_hours == 0 {
            "< 1 hour".into()
        } else if total_hours < 8 {
            format!("{}-{} hours", total_hours, total_hours + 2)
        } else {
            format!("{}-{} days", total_hours / 8, (total_hours / 8) + 1)
        };

        // Calculate completeness
        let auto_mapped = output.summary.components_auto_mapped as f32;
        let total = (auto_mapped + output.summary.components_manual as f32).max(1.0);
        output.summary.completeness_percent =
            ((auto_mapped / total) * analysis.migration_confidence * 100.0) as u8;
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::ComponentType;

    #[test]
    fn test_starter_template_from_str() {
        assert_eq!(
            StarterTemplate::from_str("admin-panel"),
            Some(StarterTemplate::AdminPanel)
        );
        assert_eq!(
            StarterTemplate::from_str("dashboard"),
            Some(StarterTemplate::AdminPanel)
        );
        assert_eq!(
            StarterTemplate::from_str("minimal"),
            Some(StarterTemplate::Minimal)
        );
        assert_eq!(StarterTemplate::from_str("unknown"), None);
    }

    #[test]
    fn test_converter_creation() {
        let converter = Converter::with_defaults()
            .output_dir("/tmp/test")
            .project_name("test-project")
            .starter(StarterTemplate::AdminPanel);

        assert_eq!(converter.config.output_dir, PathBuf::from("/tmp/test"));
        assert_eq!(converter.config.project_name, "test-project");
        assert_eq!(converter.config.starter, StarterTemplate::AdminPanel);
    }

    #[test]
    fn test_directory_creation() {
        let converter = Converter::with_defaults();
        let mut output = MigrationOutput {
            files: Vec::new(),
            directories: Vec::new(),
            todos: Vec::new(),
            issues: Vec::new(),
            summary: MigrationSummary::default(),
        };

        converter.create_directories(&mut output).unwrap();

        assert!(output.directories.contains(&PathBuf::from("src")));
        assert!(output.directories.contains(&PathBuf::from("theme")));
        assert!(output.directories.contains(&PathBuf::from("design")));
    }

    #[test]
    fn test_todo_generation() {
        let todos = vec![
            TodoItem {
                description: "High priority task".into(),
                priority: 1,
                category: TodoCategory::Design,
                file: None,
                effort: Effort::High,
            },
            TodoItem {
                description: "Medium priority task".into(),
                priority: 2,
                category: TodoCategory::Component,
                file: Some(PathBuf::from("test.toml")),
                effort: Effort::Medium,
            },
        ];

        let converter = Converter::with_defaults();
        let md = converter.generate_todos_markdown(&todos);

        assert!(md.contains("High priority task"));
        assert!(md.contains("Medium priority task"));
        assert!(md.contains("Priority 1"));
        assert!(md.contains("Priority 2"));
    }
}
