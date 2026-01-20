//! Server Capabilities Configuration
//!
//! Defines the LSP capabilities that oxide-lsp supports.

use tower_lsp::lsp_types::*;

/// Server capabilities configuration
pub struct ServerCapabilities;

impl ServerCapabilities {
    /// Returns the default server capabilities
    pub fn default_capabilities() -> tower_lsp::lsp_types::ServerCapabilities {
        tower_lsp::lsp_types::ServerCapabilities {
            // Text document sync
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    will_save: Some(false),
                    will_save_wait_until: Some(false),
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(true),
                    })),
                },
            )),

            // Completion
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![
                    ".".to_string(),  // Component access (ui.Button)
                    ":".to_string(),  // Prop values
                    "\"".to_string(), // String values
                    "{".to_string(),  // Object/block start
                    "t".to_string(),  // Translation key trigger
                    "(".to_string(),  // Function call start
                ]),
                all_commit_characters: None,
                work_done_progress_options: WorkDoneProgressOptions::default(),
                completion_item: None,
            }),

            // Hover
            hover_provider: Some(HoverProviderCapability::Simple(true)),

            // Go to definition
            definition_provider: Some(OneOf::Left(true)),

            // Go to type definition
            type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),

            // Go to references
            references_provider: Some(OneOf::Left(true)),

            // Document symbols (outline)
            document_symbol_provider: Some(OneOf::Left(true)),

            // Workspace symbols
            workspace_symbol_provider: Some(OneOf::Left(true)),

            // Code actions
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![
                    CodeActionKind::QUICKFIX,
                    CodeActionKind::REFACTOR,
                    CodeActionKind::SOURCE,
                ]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: Some(true),
            })),

            // Document formatting
            document_formatting_provider: Some(OneOf::Left(true)),

            // Document range formatting
            document_range_formatting_provider: Some(OneOf::Left(true)),

            // Rename
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            })),

            // Semantic tokens (for syntax highlighting)
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    legend: SemanticTokensLegend {
                        token_types: vec![
                            SemanticTokenType::KEYWORD,
                            SemanticTokenType::TYPE,
                            SemanticTokenType::PROPERTY,
                            SemanticTokenType::STRING,
                            SemanticTokenType::NUMBER,
                            SemanticTokenType::VARIABLE,
                            SemanticTokenType::FUNCTION,
                            SemanticTokenType::COMMENT,
                        ],
                        token_modifiers: vec![
                            SemanticTokenModifier::DECLARATION,
                            SemanticTokenModifier::DEFINITION,
                            SemanticTokenModifier::DEPRECATED,
                        ],
                    },
                    range: Some(true),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                }),
            ),

            // Workspace capabilities
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(true)),
                }),
                file_operations: None,
            }),

            // Signature help (for function calls)
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                retrigger_characters: None,
                work_done_progress_options: WorkDoneProgressOptions::default(),
            }),

            // Document highlights
            document_highlight_provider: Some(OneOf::Left(true)),

            // Code lens
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(true),
            }),

            // Folding range
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),

            // Color provider (for theme tokens)
            color_provider: Some(ColorProviderCapability::Simple(true)),

            // Other capabilities left as default
            ..Default::default()
        }
    }
}

/// Supported file extensions and their language IDs
pub const SUPPORTED_EXTENSIONS: &[(&str, &str)] = &[
    (".oui", "oui"),
    ("oxide.toml", "toml"),
    ("plugin.toml", "toml"),
    ("theme.toml", "toml"),
    ("typography.toml", "toml"),
    ("fonts.toml", "toml"),
];

/// Check if a file is supported by the LSP
pub fn is_supported_file(path: &str) -> bool {
    SUPPORTED_EXTENSIONS
        .iter()
        .any(|(ext, _)| path.ends_with(ext))
}

/// Get the language ID for a file path
pub fn get_language_id(path: &str) -> Option<&'static str> {
    SUPPORTED_EXTENSIONS
        .iter()
        .find(|(ext, _)| path.ends_with(ext))
        .map(|(_, lang)| *lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_file_detection() {
        assert!(is_supported_file("app.oui"));
        assert!(is_supported_file("oxide.toml"));
        assert!(is_supported_file("plugin.toml"));
        assert!(is_supported_file("theme.toml"));
        assert!(!is_supported_file("random.txt"));
        assert!(!is_supported_file("config.json"));
    }

    #[test]
    fn test_language_id() {
        assert_eq!(get_language_id("app.oui"), Some("oui"));
        assert_eq!(get_language_id("oxide.toml"), Some("toml"));
        assert_eq!(get_language_id("random.txt"), None);
    }
}
