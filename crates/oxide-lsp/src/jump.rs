//! Jump-to-Definition Engine
//!
//! Provides navigation to:
//! - Component definitions
//! - Token definitions in theme files
//! - Typography role definitions
//! - Translation key definitions
//! - Plugin definitions

use crate::document::DocumentStore;
use crate::project::ProjectContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

/// Jump-to-definition engine
pub struct JumpEngine {
    documents: Arc<RwLock<DocumentStore>>,
    project: Arc<RwLock<Option<ProjectContext>>>,
}

impl JumpEngine {
    /// Create a new jump engine
    pub fn new(
        documents: Arc<RwLock<DocumentStore>>,
        project: Arc<RwLock<Option<ProjectContext>>>,
    ) -> Self {
        Self { documents, project }
    }

    /// Get jump-to-definition target
    pub async fn goto_definition(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        let docs = self.documents.read().await;
        let doc = docs.get(uri)?;

        let word_info = doc.word_at(position.line as usize, position.character as usize)?;
        let line = doc.get_line(position.line as usize)?;

        let project = self.project.read().await;

        // Try different jump providers
        if let Some(target) = self.jump_to_component(&word_info.word, &project).await {
            return Some(target);
        }

        if let Some(target) = self.jump_to_token(&word_info.word, &project).await {
            return Some(target);
        }

        if let Some(target) = self.jump_to_translation(line, &word_info.word, &project).await {
            return Some(target);
        }

        None
    }

    /// Jump to component definition
    async fn jump_to_component(
        &self,
        word: &str,
        project: &Option<ProjectContext>,
    ) -> Option<GotoDefinitionResponse> {
        let project = project.as_ref()?;

        // Check if it's a component from a plugin
        for plugin in &project.plugins {
            if plugin.components.contains(&word.to_string()) {
                // Jump to plugin definition
                let plugin_path = project.root.join("plugins").join(&plugin.name).join("plugin.toml");
                if plugin_path.exists() {
                    let uri = Url::from_file_path(&plugin_path).ok()?;
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri,
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 0 },
                        },
                    }));
                }
            }
        }

        // Check for local component definition
        // Look in common locations: src/components/, components/
        let search_paths = [
            project.root.join("src").join("components"),
            project.root.join("components"),
        ];

        for search_dir in search_paths {
            if !search_dir.exists() {
                continue;
            }

            // Look for files matching the component name
            let patterns = [
                format!("{}.oui", word),
                format!("{}.oui", word.to_lowercase()),
            ];

            for pattern in patterns {
                let component_path = search_dir.join(&pattern);
                if component_path.exists() {
                    let uri = Url::from_file_path(&component_path).ok()?;
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri,
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 0 },
                        },
                    }));
                }
            }
        }

        None
    }

    /// Jump to token definition
    async fn jump_to_token(
        &self,
        word: &str,
        project: &Option<ProjectContext>,
    ) -> Option<GotoDefinitionResponse> {
        // Check if word looks like a token reference
        if !word.contains('.') {
            return None;
        }

        let project = project.as_ref()?;

        // Determine the appropriate file based on token category
        let (file_name, key_path) = if word.starts_with("colors.") {
            ("theme.toml", word.strip_prefix("colors.")?)
        } else if word.starts_with("spacing.") {
            ("theme.toml", word.strip_prefix("spacing.")?)
        } else if word.starts_with("radius.") {
            ("theme.toml", word.strip_prefix("radius.")?)
        } else if word.starts_with("typography.") {
            ("typography.toml", word.strip_prefix("typography.")?)
        } else if word.starts_with("fonts.") {
            ("fonts.toml", word.strip_prefix("fonts.")?)
        } else {
            return None;
        };

        let file_path = project.root.join(file_name);
        if !file_path.exists() {
            return None;
        }

        // Find the line containing the key definition
        if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
            for (line_num, line) in content.lines().enumerate() {
                // Look for key definition like "primary = " or "[colors]\nprimary = "
                if line.trim().starts_with(key_path) && line.contains('=') {
                    let uri = Url::from_file_path(&file_path).ok()?;
                    let col = line.find(key_path).unwrap_or(0);
                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri,
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: col as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: (col + key_path.len()) as u32,
                            },
                        },
                    }));
                }
            }
        }

        // Jump to file even if key not found
        let uri = Url::from_file_path(&file_path).ok()?;
        Some(GotoDefinitionResponse::Scalar(Location {
            uri,
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        }))
    }

    /// Jump to translation key definition
    async fn jump_to_translation(
        &self,
        line: &str,
        word: &str,
        project: &Option<ProjectContext>,
    ) -> Option<GotoDefinitionResponse> {
        // Check if we're in a translation context
        if !line.contains("t(\"") && !line.contains("t('") {
            return None;
        }

        let project = project.as_ref()?;
        let i18n_dir = project.root.join("i18n");

        if !i18n_dir.exists() {
            return None;
        }

        // Use the word directly if it looks like a key, or try to extract from context
        let key = if word.contains('.') {
            word.to_string()
        } else {
            // Try to extract full key from line
            self.extract_translation_key(line)?
        };

        // Split key into parts: auth.login.title -> ["auth", "login", "title"]
        let key_parts: Vec<&str> = key.split('.').collect();

        // Find the translation file and line
        for locale in &["en", "en-US"] {
            let locale_file = i18n_dir.join(format!("{}.toml", locale));
            if !locale_file.exists() {
                continue;
            }

            if let Ok(content) = tokio::fs::read_to_string(&locale_file).await {
                // Look for the key in the file
                // For a key like "auth.login.title", we might find:
                // [auth.login]
                // title = "..."
                // or just:
                // auth.login.title = "..."

                let last_part = key_parts.last()?;
                let section = key_parts[..key_parts.len() - 1].join(".");

                // First, try to find the exact key
                for (line_num, line_content) in content.lines().enumerate() {
                    let trimmed = line_content.trim();

                    // Check for key = value
                    if trimmed.starts_with(last_part) && trimmed.contains('=') {
                        // Verify we're in the right section by checking previous lines
                        let section_pattern = format!("[{}]", section);
                        let lines_before: Vec<&str> = content.lines().take(line_num).collect();
                        let in_section = lines_before
                            .iter()
                            .rev()
                            .find(|l| l.trim().starts_with('['))
                            .map(|l| l.contains(&section_pattern))
                            .unwrap_or(false);

                        if in_section || section.is_empty() {
                            let uri = Url::from_file_path(&locale_file).ok()?;
                            let col = line_content.find(last_part).unwrap_or(0);
                            return Some(GotoDefinitionResponse::Scalar(Location {
                                uri,
                                range: Range {
                                    start: Position {
                                        line: line_num as u32,
                                        character: col as u32,
                                    },
                                    end: Position {
                                        line: line_num as u32,
                                        character: (col + last_part.len()) as u32,
                                    },
                                },
                            }));
                        }
                    }

                    // Check for full key path
                    if trimmed.starts_with(&key) && trimmed.contains('=') {
                        let uri = Url::from_file_path(&locale_file).ok()?;
                        return Some(GotoDefinitionResponse::Scalar(Location {
                            uri,
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: key.len() as u32,
                                },
                            },
                        }));
                    }
                }
            }
        }

        None
    }

    /// Extract translation key from line context
    fn extract_translation_key(&self, line: &str) -> Option<String> {
        let patterns = ["t(\"", "t('"];
        for pattern in patterns {
            if let Some(start) = line.find(pattern) {
                let after = &line[start + pattern.len()..];
                let quote_char = pattern.chars().last().unwrap();
                if let Some(end) = after.find(quote_char) {
                    return Some(after[..end].to_string());
                }
            }
        }
        None
    }
}

/// Jump target with metadata
#[derive(Debug, Clone)]
pub struct JumpTarget {
    pub location: Location,
    pub kind: JumpTargetKind,
}

/// Types of jump targets
#[derive(Debug, Clone, Copy)]
pub enum JumpTargetKind {
    Component,
    Token,
    TranslationKey,
    Plugin,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_translation_key() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = JumpEngine::new(docs, project);

        assert_eq!(
            engine.extract_translation_key("content: t(\"auth.login.title\")"),
            Some("auth.login.title".to_string())
        );

        assert_eq!(
            engine.extract_translation_key("t('key.here')"),
            Some("key.here".to_string())
        );

        assert_eq!(engine.extract_translation_key("no translation here"), None);
    }
}
