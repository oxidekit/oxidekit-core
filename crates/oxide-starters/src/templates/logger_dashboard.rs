//! Logger Dashboard Starter
//!
//! An internal log/event viewer dashboard.

use crate::{
    StarterSpec, StarterMetadata, StarterCategory, StarterTarget,
    PluginRequirement, PermissionPreset, NetworkPermissions, FilesystemPermissions,
    GeneratedFile, PostInitStep, MessageLevel,
};

/// Create the logger-dashboard starter spec
pub fn create_spec() -> StarterSpec {
    StarterSpec {
        id: "logger-dashboard".to_string(),
        name: "Logger Dashboard".to_string(),
        description: "Internal log viewer with virtualized tables, filters, and diagnostics export".to_string(),
        long_description: Some(
            "A production-ready logger dashboard that includes:\n\
            - Virtualized log table (handles millions of entries)\n\
            - Filters by severity, time range, source\n\
            - Full-text search\n\
            - Log detail panel\n\
            - Export to file\n\
            - Real-time streaming (WebSocket)\n\
            - Diagnostics bundle export\n\n\
            Perfect for internal tools, debugging interfaces, and monitoring dashboards."
                .to_string(),
        ),
        version: "0.1.0".to_string(),
        min_core_version: Some("0.1.0".to_string()),
        metadata: StarterMetadata {
            category: StarterCategory::Monitoring,
            tags: vec![
                "logging".to_string(),
                "monitoring".to_string(),
                "internal".to_string(),
                "debug".to_string(),
                "events".to_string(),
            ],
            author: Some("OxideKit Team".to_string()),
            homepage: Some("https://oxidekit.com/starters/logger-dashboard".to_string()),
            screenshots: vec![
                "https://oxidekit.com/screenshots/logger-main.png".to_string(),
            ],
            official: true,
            featured: false,
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
                id: "native.filesystem".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "native.network".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
            PluginRequirement {
                id: "data.query".to_string(),
                version: Some("^0.1".to_string()),
                optional: false,
            },
        ],
        permissions: PermissionPreset {
            filesystem: FilesystemPermissions {
                read: vec!["logs/*".to_string()],
                write: vec!["exports/*".to_string()],
            },
            network: NetworkPermissions {
                hosts: vec!["localhost".to_string()],
                allow_all: false,
            },
            ..Default::default()
        },
        files: vec![
            GeneratedFile {
                path: "ui/layouts/logger_shell.oui".to_string(),
                template: "content:// Logger shell layout\n\nLoggerShell {\n    TopBar {\n        Row {\n            gap: 16\n            SearchInput { placeholder: \"Search logs...\" }\n            SeverityFilter { }\n            TimeRangeFilter { }\n            Button { text: \"Export\" icon: \"download\" }\n        }\n    }\n    Content {\n        slot: \"main\"\n    }\n    DetailPanel {\n        slot: \"detail\"\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/pages/logs.oui".to_string(),
                template: "content:// Logs viewer\n\nPage {\n    layout: \"logger_shell\"\n\n    VirtualizedTable {\n        columns: [\n            { key: \"timestamp\", label: \"Time\", width: 180 },\n            { key: \"severity\", label: \"Level\", width: 80 },\n            { key: \"source\", label: \"Source\", width: 150 },\n            { key: \"message\", label: \"Message\", flex: 1 }\n        ]\n        rowHeight: 32\n        onRowClick: \"showDetail\"\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/components/log_detail.oui".to_string(),
                template: "content:// Log detail panel\n\nLogDetail {\n    when: \"selectedLog\"\n\n    Column {\n        gap: 16\n        padding: 16\n\n        Row {\n            justify: \"space-between\"\n            Text { content: \"Log Details\" role: \"heading-small\" }\n            Button { icon: \"x\" variant: \"ghost\" onClick: \"closeDetail\" }\n        }\n\n        PropertyList {\n            Property { label: \"Timestamp\" value: \"{{selectedLog.timestamp}}\" }\n            Property { label: \"Severity\" value: \"{{selectedLog.severity}}\" }\n            Property { label: \"Source\" value: \"{{selectedLog.source}}\" }\n        }\n\n        Divider { }\n\n        CodeBlock {\n            content: \"{{selectedLog.message}}\"\n            language: \"plaintext\"\n        }\n\n        when: \"selectedLog.stackTrace\"\n        Card {\n            title: \"Stack Trace\"\n            CodeBlock {\n                content: \"{{selectedLog.stackTrace}}\"\n                language: \"plaintext\"\n            }\n        }\n    }\n}".to_string(),
                condition: None,
            },
            GeneratedFile {
                path: "ui/components/filters.oui".to_string(),
                template: "content:// Filter components\n\nSeverityFilter {\n    Select {\n        options: [\n            { value: \"all\", label: \"All Levels\" },\n            { value: \"error\", label: \"Error\" },\n            { value: \"warn\", label: \"Warning\" },\n            { value: \"info\", label: \"Info\" },\n            { value: \"debug\", label: \"Debug\" }\n        ]\n    }\n}\n\nTimeRangeFilter {\n    Select {\n        options: [\n            { value: \"1h\", label: \"Last Hour\" },\n            { value: \"24h\", label: \"Last 24 Hours\" },\n            { value: \"7d\", label: \"Last 7 Days\" },\n            { value: \"custom\", label: \"Custom Range\" }\n        ]\n    }\n}".to_string(),
                condition: None,
            },
        ],
        post_init: vec![
            PostInitStep::Message {
                text: "Logger dashboard created successfully!".to_string(),
                level: MessageLevel::Success,
            },
            PostInitStep::Command {
                command: "cd {{project_name}} && oxide dev".to_string(),
                description: Some("Start the development server".to_string()),
            },
        ],
        variables: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_dashboard_spec() {
        let spec = create_spec();

        assert_eq!(spec.id, "logger-dashboard");
        assert!(spec.targets.contains(&StarterTarget::Desktop));
    }
}
