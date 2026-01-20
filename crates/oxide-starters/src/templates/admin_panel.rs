//! Admin Panel Starter
//!
//! A full-featured admin panel starter that replaces Bootstrap/Tailwind admin templates.

use crate::{
    StarterSpec, StarterMetadata, StarterCategory, StarterTarget,
    PluginRequirement, PermissionPreset, NetworkPermissions,
    GeneratedFile, PostInitStep, MessageLevel,
};

/// Create the admin-panel starter spec
pub fn create_spec() -> StarterSpec {
    StarterSpec {
        id: "admin-panel".to_string(),
        name: "Admin Panel".to_string(),
        description: "Full-featured admin panel with sidebar, tables, forms, and charts".to_string(),
        long_description: Some(
            "A production-ready admin panel starter that includes:\n\
            - Sidebar + topbar navigation shell\n\
            - Dashboard with stats and charts\n\
            - Data tables with sorting, filtering, pagination\n\
            - CRUD forms with validation\n\
            - User management views\n\
            - Settings pages\n\
            - Dark/light theme support\n\n\
            Perfect for building internal tools, admin dashboards, and back-office applications."
                .to_string(),
        ),
        version: "0.1.0".to_string(),
        min_core_version: Some("0.1.0".to_string()),
        metadata: StarterMetadata {
            category: StarterCategory::Admin,
            tags: vec![
                "admin".to_string(),
                "dashboard".to_string(),
                "tables".to_string(),
                "forms".to_string(),
                "crud".to_string(),
            ],
            author: Some("OxideKit Team".to_string()),
            homepage: Some("https://oxidekit.com/starters/admin-panel".to_string()),
            screenshots: vec![
                "https://oxidekit.com/screenshots/admin-dashboard.png".to_string(),
                "https://oxidekit.com/screenshots/admin-tables.png".to_string(),
            ],
            official: true,
            featured: true,
        },
        targets: vec![StarterTarget::Desktop, StarterTarget::Web],
        plugins: vec![
            PluginRequirement {
                id: "ui.core".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.data".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.forms".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.navigation".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.charts".to_string(),
                version: Some("^0.1".to_string()),
                optional: true,
            },
            PluginRequirement {
                id: "design.admin.modern".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "data.query".to_string(),
                version: Some("^0.1".to_string()),
                optional: true,
            },
        ],
        permissions: PermissionPreset {
            network: NetworkPermissions {
                hosts: vec!["*".to_string()],
                allow_all: true,
            },
            ..Default::default()
        },
        files: vec![
            GeneratedFile {
                path: "ui/layouts/admin_shell.oui".to_string(),
                template: "content:// Admin shell layout\n\nAdminShell {\n    Sidebar {\n        // Navigation items\n    }\n    TopBar {\n        // Search, user menu\n    }\n    Content {\n        slot: \"main\"\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/dashboard.oui".to_string(),
                template: "content:// Dashboard page\n\nPage {\n    layout: \"admin_shell\"\n\n    Column {\n        gap: 24\n\n        Text { content: \"Dashboard\" role: \"heading\" }\n\n        Row {\n            gap: 16\n            StatCard { title: \"Users\" value: \"1,234\" }\n            StatCard { title: \"Revenue\" value: \"$12.4k\" }\n            StatCard { title: \"Orders\" value: \"567\" }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/users.oui".to_string(),
                template: "content:// Users page\n\nPage {\n    layout: \"admin_shell\"\n\n    Column {\n        gap: 16\n\n        Row {\n            justify: \"space-between\"\n            Text { content: \"Users\" role: \"heading\" }\n            Button { text: \"Add User\" variant: \"primary\" }\n        }\n\n        DataTable {\n            columns: [\"Name\", \"Email\", \"Role\", \"Status\"]\n            // Data binding here\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/settings.oui".to_string(),
                template: "content:// Settings page\n\nPage {\n    layout: \"admin_shell\"\n\n    Column {\n        gap: 24\n\n        Text { content: \"Settings\" role: \"heading\" }\n\n        Card {\n            Form {\n                TextField { label: \"App Name\" name: \"app_name\" }\n                Toggle { label: \"Dark Mode\" name: \"dark_mode\" }\n                Button { text: \"Save\" variant: \"primary\" type: \"submit\" }\n            }\n        }\n    }\n}".to_string(),
                condition: None,
            },
        ],
        post_init: vec![
            PostInitStep::Message {
                text: "Admin panel created successfully!".to_string(),
                level: MessageLevel::Success,
            },
            PostInitStep::Command {
                command: "cd {{project_name}} && oxide dev".to_string(),
                description: Some("Start the development server".to_string()),
            },
            PostInitStep::Message {
                text: "Open http://localhost:3000 to view your admin panel".to_string(),
                level: MessageLevel::Info,
            },
        ],
        variables: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_panel_spec() {
        let spec = create_spec();

        assert_eq!(spec.id, "admin-panel");
        assert!(spec.metadata.official);
        assert!(spec.targets.contains(&StarterTarget::Desktop));
        assert!(!spec.plugins.is_empty());
    }
}
