//! Protocol Extensions and Code Actions
//!
//! Implements code actions (quick fixes) for:
//! - Adding missing props with defaults
//! - Replacing invalid enum values
//! - Creating missing i18n key stubs
//! - Installing missing plugins
//! - Migrating deprecated APIs

use crate::document::DocumentStore;
use crate::project::ProjectContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

/// Code action engine
pub struct CodeActionEngine {
    documents: Arc<RwLock<DocumentStore>>,
    project: Arc<RwLock<Option<ProjectContext>>>,
}

impl CodeActionEngine {
    /// Create a new code action engine
    pub fn new(
        documents: Arc<RwLock<DocumentStore>>,
        project: Arc<RwLock<Option<ProjectContext>>>,
    ) -> Self {
        Self { documents, project }
    }

    /// Get code actions for a range
    pub async fn actions(
        &self,
        uri: &Url,
        range: Range,
        diagnostics: &[Diagnostic],
    ) -> CodeActionResponse {
        let mut actions = vec![];

        let docs = self.documents.read().await;
        let doc = match docs.get(uri) {
            Some(d) => d,
            None => return actions,
        };

        let project = self.project.read().await;

        // Process each diagnostic
        for diagnostic in diagnostics {
            if let Some(code) = &diagnostic.code {
                let code_str = match code {
                    NumberOrString::String(s) => s.as_str(),
                    NumberOrString::Number(n) => {
                        // Convert number to string for matching
                        match n {
                            _ => continue,
                        }
                    }
                };

                match code_str {
                    "OUI001" => {
                        // Unknown component - suggest installing plugin or creating component
                        if let Some(action) = self
                            .action_unknown_component(uri, diagnostic, &doc.content, &project)
                        {
                            actions.push(action);
                        }
                    }
                    "OUI002" => {
                        // Deprecated component - suggest migration
                        if let Some(action) =
                            self.action_migrate_deprecated(uri, diagnostic, &project)
                        {
                            actions.push(action);
                        }
                    }
                    "OUI003" => {
                        // Missing translation key - create stub
                        if let Some(action) =
                            self.action_create_translation(uri, diagnostic, &project)
                        {
                            actions.push(action);
                        }
                    }
                    "OUI004" => {
                        // Invalid prop value - suggest valid values
                        actions.extend(self.action_fix_prop_value(uri, diagnostic, &doc.content));
                    }
                    "OUI005" => {
                        // Unknown token - create in theme.toml
                        if let Some(action) = self.action_create_token(uri, diagnostic, &project) {
                            actions.push(action);
                        }
                    }
                    "OUI006" => {
                        // Missing required prop - add with default
                        if let Some(action) =
                            self.action_add_missing_prop(uri, diagnostic, range, &project)
                        {
                            actions.push(action);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Add general refactoring actions
        actions.extend(self.general_actions(uri, range, &doc.content, &project));

        actions
    }

    /// Action for unknown component
    fn action_unknown_component(
        &self,
        _uri: &Url,
        diagnostic: &Diagnostic,
        _content: &str,
        _project: &Option<ProjectContext>,
    ) -> Option<CodeActionOrCommand> {
        let component_name = diagnostic
            .message
            .strip_prefix("Unknown component: ")?
            .to_string();

        // Suggest creating a new component file
        Some(CodeActionOrCommand::Command(Command {
            title: format!("Create component '{}'", component_name),
            command: "oxide.createComponent".to_string(),
            arguments: Some(vec![serde_json::Value::String(component_name)]),
        }))
    }

    /// Action for deprecated component migration
    fn action_migrate_deprecated(
        &self,
        _uri: &Url,
        diagnostic: &Diagnostic,
        project: &Option<ProjectContext>,
    ) -> Option<CodeActionOrCommand> {
        // Extract component name from diagnostic
        let message = &diagnostic.message;
        let project = project.as_ref()?;

        // Find migration info in schema
        for (_id, schema) in &project.component_schemas {
            if schema.deprecated {
                if let Some(msg) = &schema.deprecation_message {
                    if message.contains(msg) || message.contains(&schema.id) {
                        // Return a command to open migration docs
                        return Some(CodeActionOrCommand::Command(Command {
                            title: format!("View migration guide for '{}'", schema.id),
                            command: "oxide.showMigrationGuide".to_string(),
                            arguments: Some(vec![serde_json::Value::String(schema.id.clone())]),
                        }));
                    }
                }
            }
        }

        None
    }

    /// Action to create missing translation key
    fn action_create_translation(
        &self,
        _uri: &Url,
        diagnostic: &Diagnostic,
        project: &Option<ProjectContext>,
    ) -> Option<CodeActionOrCommand> {
        let key = diagnostic
            .message
            .strip_prefix("Missing translation key: ")?
            .to_string();

        let project = project.as_ref()?;
        let i18n_file = project.root.join("i18n").join("en.toml");

        if !i18n_file.exists() {
            return None;
        }

        // Create the edit to add the key
        let key_parts: Vec<&str> = key.split('.').collect();
        let section = key_parts[..key_parts.len() - 1].join(".");
        let key_name = key_parts.last()?;

        let insert_text = if section.is_empty() {
            format!("{} = \"TODO\"\n", key_name)
        } else {
            format!("\n[{}]\n{} = \"TODO\"\n", section, key_name)
        };

        let file_uri = Url::from_file_path(&i18n_file).ok()?;

        Some(CodeActionOrCommand::CodeAction(CodeAction {
            title: format!("Create translation key '{}'", key),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(
                    [(
                        file_uri,
                        vec![TextEdit {
                            range: Range {
                                start: Position {
                                    line: u32::MAX,
                                    character: 0,
                                },
                                end: Position {
                                    line: u32::MAX,
                                    character: 0,
                                },
                            },
                            new_text: insert_text,
                        }],
                    )]
                    .into_iter()
                    .collect(),
                ),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        }))
    }

    /// Action to fix invalid prop value
    fn action_fix_prop_value(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        _content: &str,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = vec![];

        // Parse valid values from diagnostic message
        // Message format: "Invalid value for X: 'Y'. Expected one of: a, b, c"
        if let Some(expected_pos) = diagnostic.message.find("Expected one of: ") {
            let values_str = &diagnostic.message[expected_pos + "Expected one of: ".len()..];
            let valid_values: Vec<&str> = values_str.split(", ").collect();

            for value in valid_values {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Change to '{}'", value),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(
                            [(
                                uri.clone(),
                                vec![TextEdit {
                                    range: diagnostic.range,
                                    new_text: value.to_string(),
                                }],
                            )]
                            .into_iter()
                            .collect(),
                        ),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(false),
                    disabled: None,
                    data: None,
                }));
            }
        }

        actions
    }

    /// Action to create missing token
    fn action_create_token(
        &self,
        _uri: &Url,
        diagnostic: &Diagnostic,
        project: &Option<ProjectContext>,
    ) -> Option<CodeActionOrCommand> {
        let token = diagnostic
            .message
            .strip_prefix("Unknown design token: ")?
            .to_string();

        let project = project.as_ref()?;

        // Determine target file based on token type
        let (file_name, section) = if token.starts_with("colors.") {
            ("theme.toml", "colors")
        } else if token.starts_with("spacing.") {
            ("theme.toml", "spacing")
        } else if token.starts_with("radius.") {
            ("theme.toml", "radius")
        } else {
            return None;
        };

        let theme_file = project.root.join(file_name);
        let file_uri = Url::from_file_path(&theme_file).ok()?;

        let key = token.split('.').last()?;
        let insert_text = format!("{} = \"#000000\"\n", key);

        Some(CodeActionOrCommand::CodeAction(CodeAction {
            title: format!("Add '{}' to theme.toml", token),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(
                    [(
                        file_uri,
                        vec![TextEdit {
                            range: Range {
                                start: Position {
                                    line: u32::MAX,
                                    character: 0,
                                },
                                end: Position {
                                    line: u32::MAX,
                                    character: 0,
                                },
                            },
                            new_text: format!("\n[{}]\n{}", section, insert_text),
                        }],
                    )]
                    .into_iter()
                    .collect(),
                ),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        }))
    }

    /// Action to add missing required prop
    fn action_add_missing_prop(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        _range: Range,
        project: &Option<ProjectContext>,
    ) -> Option<CodeActionOrCommand> {
        // Extract prop name from message
        // Message format: "Missing required prop: propName"
        let prop_name = diagnostic
            .message
            .strip_prefix("Missing required prop: ")?
            .to_string();

        // Find default value from schema
        let default_value = if let Some(ctx) = project {
            ctx.component_schemas
                .values()
                .flat_map(|s| s.props.iter())
                .find(|p| p.name == prop_name)
                .and_then(|p| p.default.clone())
                .map(|v| format!("{}", v))
                .unwrap_or_else(|| "\"\"".to_string())
        } else {
            "\"\"".to_string()
        };

        let insert_text = format!("{}: {}\n", prop_name, default_value);

        Some(CodeActionOrCommand::CodeAction(CodeAction {
            title: format!("Add required prop '{}'", prop_name),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(
                    [(
                        uri.clone(),
                        vec![TextEdit {
                            range: Range {
                                start: Position {
                                    line: diagnostic.range.start.line + 1,
                                    character: 0,
                                },
                                end: Position {
                                    line: diagnostic.range.start.line + 1,
                                    character: 0,
                                },
                            },
                            new_text: format!("    {}", insert_text),
                        }],
                    )]
                    .into_iter()
                    .collect(),
                ),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        }))
    }

    /// General refactoring actions
    fn general_actions(
        &self,
        _uri: &Url,
        _range: Range,
        _content: &str,
        _project: &Option<ProjectContext>,
    ) -> Vec<CodeActionOrCommand> {
        vec![
            // Extract component
            CodeActionOrCommand::Command(Command {
                title: "Extract to component".to_string(),
                command: "oxide.extractComponent".to_string(),
                arguments: None,
            }),
            // Wrap with container
            CodeActionOrCommand::Command(Command {
                title: "Wrap with Container".to_string(),
                command: "oxide.wrapWithContainer".to_string(),
                arguments: None,
            }),
            // Wrap with Column
            CodeActionOrCommand::Command(Command {
                title: "Wrap with Column".to_string(),
                command: "oxide.wrapWithColumn".to_string(),
                arguments: None,
            }),
            // Wrap with Row
            CodeActionOrCommand::Command(Command {
                title: "Wrap with Row".to_string(),
                command: "oxide.wrapWithRow".to_string(),
                arguments: None,
            }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_action_fix_prop_value() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = CodeActionEngine::new(docs, project);

        let uri = Url::parse("file:///test.oui").unwrap();
        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 7 },
                end: Position { line: 0, character: 10 },
            },
            message: "Invalid value for align: 'bad'. Expected one of: start, center, end".to_string(),
            ..Default::default()
        };

        let actions = engine.action_fix_prop_value(&uri, &diagnostic, "align: bad");
        assert_eq!(actions.len(), 3);
    }
}
