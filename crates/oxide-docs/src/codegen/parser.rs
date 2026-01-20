//! Source code documentation parser

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::DocsResult;

/// A documentation comment extracted from source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocComment {
    /// The raw documentation text
    pub text: String,
    /// Parsed sections
    pub sections: Vec<DocSection>,
    /// Code examples
    pub examples: Vec<CodeExample>,
    /// Links to other items
    pub links: Vec<DocLink>,
    /// Whether this doc is deprecated
    pub deprecated: Option<String>,
    /// Stability attributes
    pub stability: Option<Stability>,
}

impl DocComment {
    /// Parse a documentation comment from text
    pub fn parse(text: &str) -> Self {
        let mut sections = Vec::new();
        let mut examples = Vec::new();
        let mut links = Vec::new();
        let mut deprecated = None;
        let mut stability = None;

        let mut current_section: Option<(String, String)> = None;
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_content = String::new();

        for line in text.lines() {
            let trimmed = line.trim();

            // Handle code blocks
            if trimmed.starts_with("```") {
                if in_code_block {
                    examples.push(CodeExample {
                        language: code_lang.clone(),
                        code: code_content.trim().to_string(),
                        runnable: code_lang == "rust" && !code_content.contains("no_run"),
                    });
                    in_code_block = false;
                    code_content.clear();
                } else {
                    in_code_block = true;
                    code_lang = trimmed[3..].to_string();
                    if code_lang.contains(',') {
                        code_lang = code_lang.split(',').next().unwrap_or("").to_string();
                    }
                }
                continue;
            }

            if in_code_block {
                code_content.push_str(line);
                code_content.push('\n');
                continue;
            }

            // Parse special sections
            if trimmed.starts_with("# ") {
                // Finish previous section
                if let Some((name, content)) = current_section.take() {
                    sections.push(DocSection { name, content });
                }
                current_section = Some((trimmed[2..].to_string(), String::new()));
                continue;
            }

            // Parse attributes
            if trimmed.starts_with("#[deprecated") {
                deprecated = Some(extract_attribute_value(trimmed));
                continue;
            }

            if trimmed.starts_with("#[stable") || trimmed.starts_with("#[unstable") {
                stability = Some(if trimmed.contains("unstable") {
                    Stability::Unstable
                } else {
                    Stability::Stable
                });
                continue;
            }

            // Parse links
            for link in extract_links(trimmed) {
                links.push(link);
            }

            // Add to current section or default content
            if let Some((_, ref mut content)) = current_section {
                content.push_str(line);
                content.push('\n');
            }
        }

        // Finish last section
        if let Some((name, content)) = current_section {
            sections.push(DocSection { name, content });
        }

        Self {
            text: text.to_string(),
            sections,
            examples,
            links,
            deprecated,
            stability,
        }
    }

    /// Get the first paragraph as a summary
    pub fn summary(&self) -> String {
        self.text
            .lines()
            .take_while(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Check if there are any examples
    pub fn has_examples(&self) -> bool {
        !self.examples.is_empty()
    }
}

/// A section in documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSection {
    /// Section name (e.g., "Examples", "Arguments", "Returns")
    pub name: String,
    /// Section content
    pub content: String,
}

/// A code example in documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Programming language
    pub language: String,
    /// Code content
    pub code: String,
    /// Whether this example can be run
    pub runnable: bool,
}

/// A link to another item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocLink {
    /// Link text
    pub text: String,
    /// Link target (path or URL)
    pub target: String,
}

/// Item stability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stability {
    Stable,
    Unstable,
}

/// Documentation for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDoc {
    /// Module name
    pub name: String,
    /// Module path
    pub path: String,
    /// Module documentation
    pub doc: Option<DocComment>,
    /// Items in this module
    pub items: Vec<ItemDoc>,
    /// Submodules
    pub submodules: Vec<ModuleDoc>,
    /// Re-exports
    pub reexports: Vec<String>,
}

impl ModuleDoc {
    /// Create a new module doc
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            doc: None,
            items: Vec::new(),
            submodules: Vec::new(),
            reexports: Vec::new(),
        }
    }

    /// Parse module documentation from a source file
    pub fn parse_file(file_path: &Path) -> DocsResult<Self> {
        let content = fs::read_to_string(file_path)?;
        let name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let mut module = Self::new(name, file_path.to_string_lossy());

        // Parse module-level docs (//! comments at the start)
        let mut module_doc_lines = Vec::new();
        for line in content.lines() {
            if let Some(stripped) = line.trim().strip_prefix("//!") {
                module_doc_lines.push(stripped.trim_start());
            } else if !line.trim().is_empty() && !line.trim().starts_with("//!") {
                break;
            }
        }

        if !module_doc_lines.is_empty() {
            module.doc = Some(DocComment::parse(&module_doc_lines.join("\n")));
        }

        // Parse items (simplified - real implementation would use syn)
        let items = parse_items(&content);
        module.items = items;

        Ok(module)
    }
}

/// Documentation for a single item (fn, struct, enum, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDoc {
    /// Item name
    pub name: String,
    /// Item kind
    pub kind: ItemKind,
    /// Item documentation
    pub doc: Option<DocComment>,
    /// Visibility
    pub visibility: Visibility,
    /// Type signature (for functions, methods)
    pub signature: Option<String>,
    /// Generic parameters
    pub generics: Vec<String>,
    /// Associated items (for structs, traits, enums)
    pub associated: Vec<ItemDoc>,
}

/// Kind of documented item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Function,
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Constant,
    Static,
    Module,
    Macro,
    Impl,
    Method,
    Field,
    Variant,
}

impl ItemKind {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Function => "Function",
            Self::Struct => "Struct",
            Self::Enum => "Enum",
            Self::Trait => "Trait",
            Self::TypeAlias => "Type Alias",
            Self::Constant => "Constant",
            Self::Static => "Static",
            Self::Module => "Module",
            Self::Macro => "Macro",
            Self::Impl => "Implementation",
            Self::Method => "Method",
            Self::Field => "Field",
            Self::Variant => "Variant",
        }
    }
}

/// Item visibility
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
    Crate,
    Super,
    Restricted(String),
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
}

// Helper functions

fn extract_attribute_value(line: &str) -> String {
    if let Some(start) = line.find('(') {
        if let Some(end) = line.rfind(')') {
            return line[start + 1..end].to_string();
        }
    }
    String::new()
}

fn extract_links(text: &str) -> Vec<DocLink> {
    let mut links = Vec::new();
    let link_re = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();

    for cap in link_re.captures_iter(text) {
        links.push(DocLink {
            text: cap[1].to_string(),
            target: cap[2].to_string(),
        });
    }

    // Also handle [`Type`] style links
    let type_link_re = regex::Regex::new(r"\[`([^`]+)`\]").unwrap();
    for cap in type_link_re.captures_iter(text) {
        links.push(DocLink {
            text: cap[1].to_string(),
            target: cap[1].to_string(), // Self-referential
        });
    }

    links
}

/// Parse items from source code (simplified implementation)
fn parse_items(content: &str) -> Vec<ItemDoc> {
    let mut items = Vec::new();
    let mut current_doc_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Collect doc comments
        if let Some(doc) = trimmed.strip_prefix("///") {
            current_doc_lines.push(doc.trim_start().to_string());
            continue;
        }

        // Skip attributes for now (simplified)
        if trimmed.starts_with("#[") {
            continue;
        }

        // Parse item definitions
        if let Some(item) = parse_item_definition(trimmed, &current_doc_lines) {
            items.push(item);
        }

        // Clear doc comments if we hit a non-doc, non-attribute line
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            current_doc_lines.clear();
        }
    }

    items
}

fn parse_item_definition(line: &str, doc_lines: &[String]) -> Option<ItemDoc> {
    let visibility = if line.starts_with("pub ") {
        Visibility::Public
    } else if line.starts_with("pub(crate)") {
        Visibility::Crate
    } else if line.starts_with("pub(super)") {
        Visibility::Super
    } else {
        Visibility::Private
    };

    // Strip visibility prefix
    let line = line
        .strip_prefix("pub(crate) ")
        .or_else(|| line.strip_prefix("pub(super) "))
        .or_else(|| line.strip_prefix("pub "))
        .unwrap_or(line);

    let (kind, name, signature) = if line.starts_with("fn ") {
        let name = extract_name(&line[3..], '(');
        let sig = extract_until_brace(line);
        (ItemKind::Function, name, Some(sig))
    } else if line.starts_with("async fn ") {
        let name = extract_name(&line[9..], '(');
        let sig = extract_until_brace(line);
        (ItemKind::Function, name, Some(sig))
    } else if line.starts_with("struct ") {
        let name = extract_name(&line[7..], '<');
        (ItemKind::Struct, name, None)
    } else if line.starts_with("enum ") {
        let name = extract_name(&line[5..], '<');
        (ItemKind::Enum, name, None)
    } else if line.starts_with("trait ") {
        let name = extract_name(&line[6..], '<');
        (ItemKind::Trait, name, None)
    } else if line.starts_with("type ") {
        let name = extract_name(&line[5..], '<');
        (ItemKind::TypeAlias, name, None)
    } else if line.starts_with("const ") {
        let name = extract_name(&line[6..], ':');
        (ItemKind::Constant, name, None)
    } else if line.starts_with("static ") {
        let name = extract_name(&line[7..], ':');
        (ItemKind::Static, name, None)
    } else if line.starts_with("mod ") {
        let name = extract_name(&line[4..], ' ');
        (ItemKind::Module, name, None)
    } else if line.starts_with("macro_rules! ") {
        let name = extract_name(&line[13..], ' ');
        (ItemKind::Macro, name, None)
    } else {
        return None;
    };

    let doc = if doc_lines.is_empty() {
        None
    } else {
        Some(DocComment::parse(&doc_lines.join("\n")))
    };

    Some(ItemDoc {
        name,
        kind,
        doc,
        visibility,
        signature,
        generics: Vec::new(),
        associated: Vec::new(),
    })
}

fn extract_name(text: &str, until: char) -> String {
    text.split(|c| c == until || c == ' ' || c == '{' || c == ';')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn extract_until_brace(text: &str) -> String {
    text.split('{')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_comment_parse() {
        let doc = DocComment::parse("This is a summary.\n\n# Examples\n\nSome example text.");
        assert!(!doc.summary().is_empty());
    }

    #[test]
    fn test_code_example_extraction() {
        let doc = DocComment::parse(
            "Summary\n\n```rust\nlet x = 1;\n```"
        );
        assert!(doc.has_examples());
        assert_eq!(doc.examples[0].language, "rust");
    }

    #[test]
    fn test_item_parsing() {
        let items = parse_items("/// Documentation\npub fn test_function() {}");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "test_function");
        assert_eq!(items[0].kind, ItemKind::Function);
        assert_eq!(items[0].visibility, Visibility::Public);
    }
}
