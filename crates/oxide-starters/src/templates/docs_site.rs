//! Docs Site Starter
//!
//! A documentation site starter that replaces Zola/Docusaurus setups.

use crate::{
    StarterSpec, StarterMetadata, StarterCategory, StarterTarget,
    PluginRequirement, PermissionPreset,
    GeneratedFile, PostInitStep, MessageLevel,
};

/// Create the docs-site starter spec
pub fn create_spec() -> StarterSpec {
    StarterSpec {
        id: "docs-site".to_string(),
        name: "Documentation Site".to_string(),
        description: "Static documentation site with sidebar navigation, search, and versioning".to_string(),
        long_description: Some(
            "A production-ready documentation site starter that includes:\n\
            - Sidebar navigation with sections\n\
            - Markdown/MDX content support\n\
            - Code block syntax highlighting\n\
            - Static search index\n\
            - Version selector\n\
            - Mobile-responsive layout\n\
            - Light/dark theme\n\n\
            Perfect for project documentation, API docs, and knowledge bases."
                .to_string(),
        ),
        version: "0.1.0".to_string(),
        min_core_version: Some("0.1.0".to_string()),
        metadata: StarterMetadata {
            category: StarterCategory::Docs,
            tags: vec![
                "docs".to_string(),
                "documentation".to_string(),
                "static".to_string(),
                "markdown".to_string(),
            ],
            author: Some("OxideKit Team".to_string()),
            homepage: Some("https://oxidekit.com/starters/docs-site".to_string()),
            screenshots: vec![
                "https://oxidekit.com/screenshots/docs-home.png".to_string(),
            ],
            official: true,
            featured: true,
        },
        targets: vec![StarterTarget::Static, StarterTarget::Web],
        plugins: vec![
            PluginRequirement {
                id: "ui.core".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "ui.navigation".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "design.docs.modern".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
        ],
        permissions: PermissionPreset::default(),
        files: vec![
            GeneratedFile {
                path: "ui/layouts/docs_shell.oui".to_string(),
                template: "content:// Docs shell layout\n\nDocsShell {\n    Header {\n        Logo { }\n        NavLinks {\n            Link { href: \"/docs\" text: \"Docs\" }\n            Link { href: \"/api\" text: \"API\" }\n            Link { href: \"/blog\" text: \"Blog\" }\n        }\n        SearchButton { }\n        ThemeToggle { }\n    }\n    Sidebar {\n        slot: \"nav\"\n    }\n    Content {\n        slot: \"main\"\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/index.oui".to_string(),
                template: "content:// Docs homepage\n\nPage {\n    Column {\n        gap: 32\n        align: \"center\"\n        padding: 48\n\n        Text { content: \"{{project_name}}\" role: \"display\" }\n        Text { content: \"Documentation\" role: \"body-large\" }\n\n        Row {\n            gap: 16\n            Button { text: \"Get Started\" href: \"/docs/getting-started\" variant: \"primary\" }\n            Button { text: \"View on GitHub\" href: \"#\" variant: \"secondary\" }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/docs/getting-started.oui".to_string(),
                template: "content:// Getting Started\n\nPage {\n    layout: \"docs_shell\"\n\n    Markdown {\n        source: \"content/docs/getting-started.md\"\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "content/docs/getting-started.md".to_string(),
                template: "content:# Getting Started\n\nWelcome to {{project_name}}!\n\n## Installation\n\n```bash\noxide add {{project_name}}\n```\n\n## Quick Start\n\n1. Create a new project\n2. Configure your settings\n3. Start building!\n\n## Next Steps\n\n- [Configuration](/docs/configuration)\n- [API Reference](/api)\n".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "content/docs/configuration.md".to_string(),
                template: "content:# Configuration\n\nLearn how to configure {{project_name}}.\n\n## Basic Configuration\n\nEdit your `oxide.toml` file:\n\n```toml\n[app]\nname = \"my-app\"\nversion = \"1.0.0\"\n```\n\n## Advanced Options\n\nSee the [API Reference](/api) for all available options.\n".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/components/sidebar_nav.oui".to_string(),
                template: "content:// Sidebar navigation\n\nSidebarNav {\n    Section {\n        title: \"Getting Started\"\n        NavItem { href: \"/docs/getting-started\" text: \"Introduction\" }\n        NavItem { href: \"/docs/installation\" text: \"Installation\" }\n    }\n    Section {\n        title: \"Guides\"\n        NavItem { href: \"/docs/configuration\" text: \"Configuration\" }\n        NavItem { href: \"/docs/deployment\" text: \"Deployment\" }\n    }\n}".to_string(),
                condition: None,
            },
        ],
        post_init: vec![
            PostInitStep::Message {
                text: "Documentation site created successfully!".to_string(),
                level: MessageLevel::Success,
            },
            PostInitStep::Command {
                command: "cd {{project_name}} && oxide dev".to_string(),
                description: Some("Start the development server".to_string()),
            },
            PostInitStep::Command {
                command: "oxide build --target static".to_string(),
                description: Some("Build for production (static output)".to_string()),
            },
        ],
        variables: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs_site_spec() {
        let spec = create_spec();

        assert_eq!(spec.id, "docs-site");
        assert!(spec.metadata.official);
        assert!(spec.targets.contains(&StarterTarget::Static));
    }
}
