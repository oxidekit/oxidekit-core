//! Recipes - Task-Oriented Building Blocks
//!
//! Recipes are structured "how to build X" instructions with explicit inputs/outputs.
//! They enable AI assistants to generate correct, version-pinned code changes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A recipe for building a specific UI pattern or feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique recipe ID (e.g., "admin.login_form")
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Category
    pub category: RecipeCategory,

    /// Tags for discovery
    #[serde(default)]
    pub tags: Vec<String>,

    /// Required extensions
    #[serde(default)]
    pub required_extensions: Vec<String>,

    /// Required permissions
    #[serde(default)]
    pub required_permissions: Vec<String>,

    /// Input parameters the recipe accepts
    #[serde(default)]
    pub inputs: Vec<RecipeInput>,

    /// Files this recipe creates or modifies
    pub outputs: Vec<RecipeOutput>,

    /// Components used by this recipe
    #[serde(default)]
    pub components_used: Vec<String>,

    /// Steps to apply the recipe
    pub steps: Vec<RecipeStep>,

    /// Wiring for actions/events
    #[serde(default)]
    pub event_wiring: Vec<EventWiring>,

    /// Optional variations (e.g., dark theme, compact layout)
    #[serde(default)]
    pub variations: Vec<RecipeVariation>,

    /// Example result preview
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<RecipePreview>,
}

/// Recipe categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipeCategory {
    /// Admin panel patterns
    Admin,
    /// Authentication patterns
    Auth,
    /// Dashboard patterns
    Dashboard,
    /// Data display patterns
    Data,
    /// Form patterns
    Forms,
    /// Navigation patterns
    Navigation,
    /// Layout patterns
    Layout,
    /// Settings patterns
    Settings,
    /// General/other
    General,
}

impl std::fmt::Display for RecipeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Auth => write!(f, "auth"),
            Self::Dashboard => write!(f, "dashboard"),
            Self::Data => write!(f, "data"),
            Self::Forms => write!(f, "forms"),
            Self::Navigation => write!(f, "navigation"),
            Self::Layout => write!(f, "layout"),
            Self::Settings => write!(f, "settings"),
            Self::General => write!(f, "general"),
        }
    }
}

/// Input parameter for a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInput {
    /// Parameter name
    pub name: String,

    /// Description
    pub description: String,

    /// Parameter type
    pub input_type: InputType,

    /// Whether this is required
    #[serde(default)]
    pub required: bool,

    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Example value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}

/// Input types for recipe parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    String,
    Bool,
    Number,
    Path,
    ComponentRef,
    Enum { values: Vec<String> },
    Array { item_type: Box<InputType> },
}

/// Output file from a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeOutput {
    /// Output path (may include template variables like {name})
    pub path: String,

    /// Type of output
    pub output_type: OutputType,

    /// Description
    pub description: String,
}

/// Type of output file
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    /// OxideKit UI file
    UiFile,
    /// Rust source file
    RustFile,
    /// Configuration file (toml, json)
    Config,
    /// Asset file
    Asset,
}

/// A step in applying a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    /// Step number (1-indexed)
    pub step: u32,

    /// Step title
    pub title: String,

    /// Step description
    pub description: String,

    /// Action to take
    pub action: StepAction,

    /// Condition for this step (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

/// Action in a recipe step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepAction {
    /// Create a new file
    CreateFile {
        path: String,
        template: String,
    },
    /// Modify an existing file
    ModifyFile {
        path: String,
        modifications: Vec<FileModification>,
    },
    /// Run a command
    RunCommand {
        command: String,
    },
    /// Add a dependency
    AddDependency {
        package: String,
        version: String,
    },
    /// Configure something
    Configure {
        file: String,
        key: String,
        value: serde_json::Value,
    },
}

/// File modification specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileModification {
    /// Insert content at a location
    Insert {
        after: Option<String>,
        before: Option<String>,
        content: String,
    },
    /// Replace content
    Replace {
        find: String,
        replace: String,
    },
    /// Append to file
    Append {
        content: String,
    },
}

/// Event wiring instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWiring {
    /// Source component
    pub source: String,

    /// Event name
    pub event: String,

    /// Handler function
    pub handler: String,

    /// Description of what this wiring does
    pub description: String,

    /// Handler code template
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handler_template: Option<String>,
}

/// Recipe variation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeVariation {
    /// Variation ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Modifications from base recipe
    pub modifications: Vec<VariationModification>,
}

/// Modification for a variation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariationModification {
    /// Which step to modify
    pub step: u32,

    /// What to change
    pub change: String,
}

/// Recipe preview information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipePreview {
    /// Screenshot URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_url: Option<String>,

    /// ASCII art preview
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ascii_preview: Option<String>,

    /// Generated code preview
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_preview: Option<String>,
}

/// A patch plan generated by applying a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchPlan {
    /// Recipe that generated this plan
    pub recipe_id: String,

    /// Input values used
    pub inputs: HashMap<String, serde_json::Value>,

    /// Files to create
    pub files_to_create: Vec<FileCreate>,

    /// Files to modify
    pub files_to_modify: Vec<FileModify>,

    /// Commands to run
    pub commands: Vec<String>,

    /// Dependencies to add
    pub dependencies: Vec<DependencyAdd>,

    /// Warnings/notes
    #[serde(default)]
    pub warnings: Vec<String>,

    /// Whether this is a dry run
    #[serde(default)]
    pub dry_run: bool,
}

/// File to create in a patch plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreate {
    pub path: String,
    pub content: String,
    pub description: String,
}

/// File to modify in a patch plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModify {
    pub path: String,
    pub diff: String,
    pub description: String,
}

/// Dependency to add
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAdd {
    pub package: String,
    pub version: String,
    pub dev_only: bool,
}

/// Recipe builder for fluent API
pub struct RecipeBuilder {
    recipe: Recipe,
}

impl RecipeBuilder {
    pub fn new(id: &str, category: RecipeCategory) -> Self {
        Self {
            recipe: Recipe {
                id: id.to_string(),
                name: id.to_string(),
                description: String::new(),
                category,
                tags: Vec::new(),
                required_extensions: Vec::new(),
                required_permissions: Vec::new(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                components_used: Vec::new(),
                steps: Vec::new(),
                event_wiring: Vec::new(),
                variations: Vec::new(),
                preview: None,
            },
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.recipe.name = name.to_string();
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.recipe.description = desc.to_string();
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.recipe.tags.push(tag.to_string());
        self
    }

    pub fn requires_extension(mut self, ext: &str) -> Self {
        self.recipe.required_extensions.push(ext.to_string());
        self
    }

    pub fn requires_permission(mut self, perm: &str) -> Self {
        self.recipe.required_permissions.push(perm.to_string());
        self
    }

    pub fn input(mut self, input: RecipeInput) -> Self {
        self.recipe.inputs.push(input);
        self
    }

    pub fn output(mut self, path: &str, output_type: OutputType, desc: &str) -> Self {
        self.recipe.outputs.push(RecipeOutput {
            path: path.to_string(),
            output_type,
            description: desc.to_string(),
        });
        self
    }

    pub fn uses_component(mut self, component: &str) -> Self {
        self.recipe.components_used.push(component.to_string());
        self
    }

    pub fn step(mut self, step: RecipeStep) -> Self {
        self.recipe.steps.push(step);
        self
    }

    pub fn event_wire(mut self, wiring: EventWiring) -> Self {
        self.recipe.event_wiring.push(wiring);
        self
    }

    pub fn variation(mut self, variation: RecipeVariation) -> Self {
        self.recipe.variations.push(variation);
        self
    }

    pub fn build(self) -> Recipe {
        self.recipe
    }
}

/// Get built-in admin recipes
pub fn admin_recipes() -> Vec<Recipe> {
    vec![
        build_login_form_recipe(),
        build_sidebar_shell_recipe(),
        build_datatable_page_recipe(),
        build_settings_page_recipe(),
        build_dashboard_recipe(),
    ]
}

fn build_login_form_recipe() -> Recipe {
    RecipeBuilder::new("admin.login_form", RecipeCategory::Auth)
        .name("Login Form")
        .description("A complete login form with email/password fields, validation, and submit handling")
        .tag("auth")
        .tag("form")
        .tag("admin")
        .requires_extension("oxide.forms")
        .input(RecipeInput {
            name: "page_path".to_string(),
            description: "Path for the login page UI file".to_string(),
            input_type: InputType::Path,
            required: true,
            default: Some(serde_json::json!("ui/pages/login.oui")),
            example: None,
        })
        .input(RecipeInput {
            name: "logo_url".to_string(),
            description: "Optional logo image URL".to_string(),
            input_type: InputType::String,
            required: false,
            default: None,
            example: Some(serde_json::json!("/assets/logo.svg")),
        })
        .input(RecipeInput {
            name: "include_remember_me".to_string(),
            description: "Include 'Remember me' checkbox".to_string(),
            input_type: InputType::Bool,
            required: false,
            default: Some(serde_json::json!(true)),
            example: None,
        })
        .output("ui/pages/login.oui", OutputType::UiFile, "Login page UI file")
        .uses_component("ui.Card")
        .uses_component("ui.Text")
        .uses_component("ui.Button")
        .uses_component("ui.TextField")
        .uses_component("ui.Checkbox")
        .step(RecipeStep {
            step: 1,
            title: "Create login page".to_string(),
            description: "Create the login page UI file with form components".to_string(),
            action: StepAction::CreateFile {
                path: "{page_path}".to_string(),
                template: r#"// Login Page
// Generated by OxideKit AI Connector

Page {
    layout: "center"
    padding: spacing.xl

    Card {
        variant: "elevated"
        max_width: 400

        // Header
        slot: "header" {
            VStack {
                spacing: spacing.md
                align: "center"

                {{#if logo_url}}
                Image {
                    src: "{{logo_url}}"
                    width: 64
                    height: 64
                }
                {{/if}}

                Text {
                    role: "heading"
                    content: "Sign In"
                }
            }
        }

        // Form
        VStack {
            spacing: spacing.lg

            TextField {
                label: "Email"
                type: "email"
                placeholder: "you@example.com"
                required: true
                bind: email
            }

            TextField {
                label: "Password"
                type: "password"
                placeholder: "Enter your password"
                required: true
                bind: password
            }

            {{#if include_remember_me}}
            Checkbox {
                label: "Remember me"
                bind: remember_me
            }
            {{/if}}

            Button {
                variant: "primary"
                label: "Sign In"
                full_width: true
                loading: is_loading
                on_click: handle_login
            }
        }

        // Footer
        slot: "footer" {
            HStack {
                justify: "center"
                spacing: spacing.sm

                Text {
                    role: "caption"
                    content: "Don't have an account?"
                }

                Button {
                    variant: "ghost"
                    label: "Sign up"
                    on_click: navigate_signup
                }
            }
        }
    }
}
"#.to_string(),
            },
            condition: None,
        })
        .event_wire(EventWiring {
            source: "Button".to_string(),
            event: "on_click".to_string(),
            handler: "handle_login".to_string(),
            description: "Submits the login form".to_string(),
            handler_template: Some(r#"
fn handle_login() {
    set_is_loading(true);

    match auth::login(&email, &password) {
        Ok(session) => {
            if remember_me {
                auth::save_session(&session);
            }
            navigate("/dashboard");
        }
        Err(e) => {
            show_error(&e.message);
        }
    }

    set_is_loading(false);
}
"#.to_string()),
        })
        .variation(RecipeVariation {
            id: "compact".to_string(),
            name: "Compact Layout".to_string(),
            description: "Smaller spacing for compact layouts".to_string(),
            modifications: vec![
                VariationModification {
                    step: 1,
                    change: "Replace spacing.lg with spacing.md, spacing.xl with spacing.lg".to_string(),
                },
            ],
        })
        .build()
}

fn build_sidebar_shell_recipe() -> Recipe {
    RecipeBuilder::new("admin.sidebar_shell", RecipeCategory::Layout)
        .name("Sidebar Shell")
        .description("Admin layout with collapsible sidebar, header, and main content area")
        .tag("layout")
        .tag("admin")
        .tag("sidebar")
        .input(RecipeInput {
            name: "layout_path".to_string(),
            description: "Path for the layout UI file".to_string(),
            input_type: InputType::Path,
            required: true,
            default: Some(serde_json::json!("ui/layouts/admin.oui")),
            example: None,
        })
        .input(RecipeInput {
            name: "app_name".to_string(),
            description: "Application name for the header".to_string(),
            input_type: InputType::String,
            required: true,
            default: None,
            example: Some(serde_json::json!("My Admin")),
        })
        .input(RecipeInput {
            name: "menu_items".to_string(),
            description: "Menu items for the sidebar".to_string(),
            input_type: InputType::Array {
                item_type: Box::new(InputType::String)
            },
            required: false,
            default: Some(serde_json::json!(["Dashboard", "Users", "Settings"])),
            example: None,
        })
        .output("ui/layouts/admin.oui", OutputType::UiFile, "Admin layout file")
        .uses_component("ui.Card")
        .uses_component("ui.Text")
        .uses_component("ui.Button")
        .uses_component("ui.IconButton")
        .uses_component("ui.Divider")
        .step(RecipeStep {
            step: 1,
            title: "Create admin layout".to_string(),
            description: "Create the sidebar shell layout".to_string(),
            action: StepAction::CreateFile {
                path: "{layout_path}".to_string(),
                template: r#"// Admin Sidebar Shell Layout
// Generated by OxideKit AI Connector

Layout {
    HStack {
        spacing: 0

        // Sidebar
        VStack {
            width: sidebar_collapsed ? 64 : 240
            background: token.color.surface
            border_right: token.color.border
            transition: "width 200ms"

            // Logo/Brand
            HStack {
                padding: spacing.md
                justify: sidebar_collapsed ? "center" : "start"

                Image {
                    src: "/assets/logo.svg"
                    width: 32
                    height: 32
                }

                if !sidebar_collapsed {
                    Text {
                        role: "title"
                        content: "{{app_name}}"
                        margin_left: spacing.sm
                    }
                }
            }

            Divider {}

            // Navigation
            VStack {
                padding: spacing.sm
                spacing: spacing.xs
                flex: 1

                {{#each menu_items}}
                NavItem {
                    label: "{{this}}"
                    icon: "{{icon_for this}}"
                    collapsed: sidebar_collapsed
                    active: current_route == "{{route_for this}}"
                    on_click: || navigate("{{route_for this}}")
                }
                {{/each}}
            }

            Divider {}

            // Collapse toggle
            HStack {
                padding: spacing.sm
                justify: "center"

                IconButton {
                    icon: sidebar_collapsed ? "chevron-right" : "chevron-left"
                    aria_label: sidebar_collapsed ? "Expand sidebar" : "Collapse sidebar"
                    on_click: toggle_sidebar
                }
            }
        }

        // Main content area
        VStack {
            flex: 1

            // Header
            HStack {
                height: 64
                padding_x: spacing.lg
                background: token.color.surface
                border_bottom: token.color.border
                align: "center"
                justify: "space-between"

                // Breadcrumb / Title
                Text {
                    role: "heading"
                    content: page_title
                }

                // User menu
                HStack {
                    spacing: spacing.sm

                    IconButton {
                        icon: "bell"
                        aria_label: "Notifications"
                        on_click: open_notifications
                    }

                    Avatar {
                        src: user.avatar_url
                        fallback: user.initials
                        size: "sm"
                    }
                }
            }

            // Content
            VStack {
                flex: 1
                padding: spacing.lg
                overflow: "auto"

                slot: "content"
            }
        }
    }
}
"#.to_string(),
            },
            condition: None,
        })
        .event_wire(EventWiring {
            source: "IconButton".to_string(),
            event: "on_click".to_string(),
            handler: "toggle_sidebar".to_string(),
            description: "Toggles sidebar collapsed state".to_string(),
            handler_template: Some(r#"
fn toggle_sidebar() {
    set_sidebar_collapsed(!sidebar_collapsed);
}
"#.to_string()),
        })
        .build()
}

fn build_datatable_page_recipe() -> Recipe {
    RecipeBuilder::new("admin.datatable_page", RecipeCategory::Data)
        .name("Data Table Page")
        .description("Page with a data table including search, filters, sorting, and pagination")
        .tag("data")
        .tag("table")
        .tag("admin")
        .requires_extension("oxide.tables")
        .input(RecipeInput {
            name: "page_path".to_string(),
            description: "Path for the page UI file".to_string(),
            input_type: InputType::Path,
            required: true,
            default: Some(serde_json::json!("ui/pages/users.oui")),
            example: None,
        })
        .input(RecipeInput {
            name: "entity_name".to_string(),
            description: "Name of the entity (e.g., 'User', 'Product')".to_string(),
            input_type: InputType::String,
            required: true,
            default: None,
            example: Some(serde_json::json!("User")),
        })
        .input(RecipeInput {
            name: "columns".to_string(),
            description: "Table columns".to_string(),
            input_type: InputType::Array {
                item_type: Box::new(InputType::String)
            },
            required: true,
            default: None,
            example: Some(serde_json::json!(["Name", "Email", "Role", "Status"])),
        })
        .output("ui/pages/{entity_name}.oui", OutputType::UiFile, "Data table page")
        .uses_component("ui.Text")
        .uses_component("ui.Button")
        .uses_component("ui.TextField")
        .uses_component("ui.DataTable")
        .uses_component("ui.TableColumn")
        .uses_component("ui.Badge")
        .step(RecipeStep {
            step: 1,
            title: "Create data table page".to_string(),
            description: "Create the page with data table and controls".to_string(),
            action: StepAction::CreateFile {
                path: "{page_path}".to_string(),
                template: r#"// {{entity_name}} List Page
// Generated by OxideKit AI Connector

Page {
    layout: "admin"

    VStack {
        spacing: spacing.lg

        // Header
        HStack {
            justify: "space-between"
            align: "center"

            Text {
                role: "heading"
                content: "{{entity_name}}s"
            }

            Button {
                variant: "primary"
                label: "Add {{entity_name}}"
                icon: "plus"
                on_click: open_create_modal
            }
        }

        // Filters bar
        Card {
            variant: "outlined"
            padding: spacing.md

            HStack {
                spacing: spacing.md
                align: "center"

                TextField {
                    placeholder: "Search..."
                    icon: "search"
                    bind: search_query
                    on_change: debounce(filter_data, 300)
                }

                Select {
                    placeholder: "Status"
                    options: status_options
                    bind: status_filter
                    on_change: filter_data
                }

                Button {
                    variant: "ghost"
                    label: "Clear filters"
                    disabled: !has_filters
                    on_click: clear_filters
                }
            }
        }

        // Data table
        DataTable {
            data: filtered_data
            loading: is_loading
            sortable: true
            selectable: true
            on_sort: handle_sort
            on_select: handle_selection

            {{#each columns}}
            TableColumn {
                key: "{{snake_case this}}"
                header: "{{this}}"
                sortable: true
            }
            {{/each}}

            TableColumn {
                key: "actions"
                header: ""
                width: 100

                slot: "cell" {
                    HStack {
                        spacing: spacing.xs

                        IconButton {
                            icon: "edit"
                            aria_label: "Edit"
                            size: "sm"
                            on_click: || open_edit_modal(row)
                        }

                        IconButton {
                            icon: "trash"
                            aria_label: "Delete"
                            size: "sm"
                            variant: "ghost"
                            on_click: || confirm_delete(row)
                        }
                    }
                }
            }
        }

        // Pagination
        HStack {
            justify: "space-between"
            align: "center"

            Text {
                role: "caption"
                content: pagination_info
            }

            Pagination {
                current: current_page
                total: total_pages
                on_change: handle_page_change
            }
        }
    }
}
"#.to_string(),
            },
            condition: None,
        })
        .event_wire(EventWiring {
            source: "DataTable".to_string(),
            event: "on_sort".to_string(),
            handler: "handle_sort".to_string(),
            description: "Handles column sorting".to_string(),
            handler_template: Some(r#"
fn handle_sort(column: &str, direction: SortDirection) {
    set_sort_column(column);
    set_sort_direction(direction);
    fetch_data();
}
"#.to_string()),
        })
        .build()
}

fn build_settings_page_recipe() -> Recipe {
    RecipeBuilder::new("admin.settings_page", RecipeCategory::Settings)
        .name("Settings Page")
        .description("Settings page with grouped options and save functionality")
        .tag("settings")
        .tag("admin")
        .tag("form")
        .requires_extension("oxide.forms")
        .input(RecipeInput {
            name: "page_path".to_string(),
            description: "Path for the settings page".to_string(),
            input_type: InputType::Path,
            required: true,
            default: Some(serde_json::json!("ui/pages/settings.oui")),
            example: None,
        })
        .input(RecipeInput {
            name: "sections".to_string(),
            description: "Settings sections".to_string(),
            input_type: InputType::Array {
                item_type: Box::new(InputType::String)
            },
            required: false,
            default: Some(serde_json::json!(["Profile", "Notifications", "Security"])),
            example: None,
        })
        .output("ui/pages/settings.oui", OutputType::UiFile, "Settings page")
        .uses_component("ui.Card")
        .uses_component("ui.Text")
        .uses_component("ui.Button")
        .uses_component("ui.TextField")
        .uses_component("ui.Switch")
        .uses_component("ui.Divider")
        .step(RecipeStep {
            step: 1,
            title: "Create settings page".to_string(),
            description: "Create the settings page with sections".to_string(),
            action: StepAction::CreateFile {
                path: "{page_path}".to_string(),
                template: r#"// Settings Page
// Generated by OxideKit AI Connector

Page {
    layout: "admin"
    max_width: 800

    VStack {
        spacing: spacing.xl

        // Header
        HStack {
            justify: "space-between"
            align: "center"

            Text {
                role: "heading"
                content: "Settings"
            }

            Button {
                variant: "primary"
                label: "Save Changes"
                loading: is_saving
                disabled: !has_changes
                on_click: save_settings
            }
        }

        // Settings sections
        {{#each sections}}
        Card {
            variant: "outlined"

            slot: "header" {
                Text {
                    role: "title"
                    content: "{{this}}"
                }
            }

            VStack {
                spacing: spacing.lg

                // Section-specific content would go here
                // This is a template placeholder

                SettingsRow {
                    label: "Example Setting"
                    description: "Description of what this setting does"

                    Switch {
                        bind: settings.example_{{snake_case this}}
                    }
                }
            }
        }
        {{/each}}

        // Danger zone
        Card {
            variant: "outlined"
            border_color: token.color.danger

            slot: "header" {
                Text {
                    role: "title"
                    color: token.color.danger
                    content: "Danger Zone"
                }
            }

            VStack {
                spacing: spacing.md

                HStack {
                    justify: "space-between"
                    align: "center"

                    VStack {
                        spacing: spacing.xs

                        Text {
                            content: "Delete Account"
                            weight: "semibold"
                        }

                        Text {
                            role: "caption"
                            content: "Permanently delete your account and all data"
                        }
                    }

                    Button {
                        variant: "danger"
                        label: "Delete Account"
                        on_click: confirm_delete_account
                    }
                }
            }
        }
    }
}
"#.to_string(),
            },
            condition: None,
        })
        .event_wire(EventWiring {
            source: "Button".to_string(),
            event: "on_click".to_string(),
            handler: "save_settings".to_string(),
            description: "Saves all settings changes".to_string(),
            handler_template: Some(r#"
async fn save_settings() {
    set_is_saving(true);

    match api::update_settings(&settings).await {
        Ok(_) => {
            show_success("Settings saved successfully");
            set_has_changes(false);
        }
        Err(e) => {
            show_error(&e.message);
        }
    }

    set_is_saving(false);
}
"#.to_string()),
        })
        .build()
}

fn build_dashboard_recipe() -> Recipe {
    RecipeBuilder::new("admin.dashboard", RecipeCategory::Dashboard)
        .name("Dashboard")
        .description("Admin dashboard with stats cards, charts, and activity feed")
        .tag("dashboard")
        .tag("admin")
        .tag("charts")
        .requires_extension("oxide.charts")
        .input(RecipeInput {
            name: "page_path".to_string(),
            description: "Path for the dashboard page".to_string(),
            input_type: InputType::Path,
            required: true,
            default: Some(serde_json::json!("ui/pages/dashboard.oui")),
            example: None,
        })
        .input(RecipeInput {
            name: "stat_cards".to_string(),
            description: "Stat cards to display".to_string(),
            input_type: InputType::Array {
                item_type: Box::new(InputType::String)
            },
            required: false,
            default: Some(serde_json::json!(["Total Users", "Revenue", "Active Sessions", "Conversion Rate"])),
            example: None,
        })
        .output("ui/pages/dashboard.oui", OutputType::UiFile, "Dashboard page")
        .uses_component("ui.Card")
        .uses_component("ui.Text")
        .uses_component("ui.Badge")
        .uses_component("ui.LineChart")
        .uses_component("ui.BarChart")
        .step(RecipeStep {
            step: 1,
            title: "Create dashboard page".to_string(),
            description: "Create the dashboard with stats and charts".to_string(),
            action: StepAction::CreateFile {
                path: "{page_path}".to_string(),
                template: r#"// Dashboard
// Generated by OxideKit AI Connector

Page {
    layout: "admin"

    VStack {
        spacing: spacing.lg

        // Header
        HStack {
            justify: "space-between"
            align: "center"

            Text {
                role: "heading"
                content: "Dashboard"
            }

            Select {
                value: time_range
                options: ["Last 7 days", "Last 30 days", "Last 90 days"]
                on_change: set_time_range
            }
        }

        // Stats cards
        Grid {
            columns: 4
            gap: spacing.md

            {{#each stat_cards}}
            StatCard {
                title: "{{this}}"
                value: stats.{{snake_case this}}
                change: stats.{{snake_case this}}_change
                trend: stats.{{snake_case this}}_trend
            }
            {{/each}}
        }

        // Charts row
        Grid {
            columns: 2
            gap: spacing.lg

            Card {
                slot: "header" {
                    Text {
                        role: "title"
                        content: "Revenue Over Time"
                    }
                }

                LineChart {
                    data: revenue_data
                    x_key: "date"
                    y_key: "amount"
                    color: token.color.primary
                    height: 300
                }
            }

            Card {
                slot: "header" {
                    Text {
                        role: "title"
                        content: "Users by Source"
                    }
                }

                BarChart {
                    data: users_by_source
                    x_key: "source"
                    y_key: "count"
                    height: 300
                }
            }
        }

        // Activity feed
        Card {
            slot: "header" {
                HStack {
                    justify: "space-between"
                    align: "center"

                    Text {
                        role: "title"
                        content: "Recent Activity"
                    }

                    Button {
                        variant: "ghost"
                        label: "View all"
                        on_click: navigate_activity
                    }
                }
            }

            VStack {
                spacing: 0

                for activity in recent_activities {
                    ActivityItem {
                        icon: activity.icon
                        title: activity.title
                        description: activity.description
                        timestamp: activity.timestamp
                    }

                    if !last {
                        Divider {}
                    }
                }
            }
        }
    }
}
"#.to_string(),
            },
            condition: None,
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_builder() {
        let recipe = RecipeBuilder::new("test.recipe", RecipeCategory::General)
            .name("Test Recipe")
            .description("A test recipe")
            .tag("test")
            .uses_component("ui.Button")
            .build();

        assert_eq!(recipe.id, "test.recipe");
        assert_eq!(recipe.category, RecipeCategory::General);
        assert!(recipe.components_used.contains(&"ui.Button".to_string()));
    }

    #[test]
    fn test_admin_recipes() {
        let recipes = admin_recipes();
        assert!(!recipes.is_empty());
        assert!(recipes.iter().any(|r| r.id == "admin.login_form"));
        assert!(recipes.iter().any(|r| r.id == "admin.sidebar_shell"));
    }

    #[test]
    fn test_recipe_category_display() {
        assert_eq!(RecipeCategory::Admin.to_string(), "admin");
        assert_eq!(RecipeCategory::Auth.to_string(), "auth");
    }
}
