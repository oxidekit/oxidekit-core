//! Documentation generator

use crate::bundler::MarkdownRenderer;
use crate::types::{DocCategory, DocPage, ContentFormat};
use crate::{DocsError, DocsResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

use super::parser::{ItemDoc, ItemKind, ModuleDoc, Visibility};

/// Documentation for an entire crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateDocumentation {
    /// Crate name
    pub name: String,
    /// Crate version
    pub version: String,
    /// Crate description
    pub description: Option<String>,
    /// Root module documentation
    pub root_module: ModuleDoc,
    /// All modules (flattened)
    pub all_modules: Vec<ModuleDoc>,
    /// Public items index
    pub public_items: Vec<ItemSummary>,
}

/// Summary of an item for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    /// Item name
    pub name: String,
    /// Full path
    pub path: String,
    /// Item kind
    pub kind: ItemKind,
    /// Short description
    pub description: String,
}

/// Code documentation generator
pub struct CodeDocGenerator {
    renderer: MarkdownRenderer,
}

impl CodeDocGenerator {
    /// Create a new generator
    pub fn new() -> Self {
        Self {
            renderer: MarkdownRenderer::new(),
        }
    }

    /// Generate documentation for a crate
    pub fn generate_crate(&self, crate_path: &Path) -> DocsResult<CrateDocumentation> {
        let cargo_toml = crate_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(DocsError::MissingContent(format!(
                "Cargo.toml not found at {:?}",
                cargo_toml
            )));
        }

        // Parse Cargo.toml for crate info
        let cargo_content = fs::read_to_string(&cargo_toml)?;
        let cargo: toml::Value = toml::from_str(&cargo_content)?;

        let package = cargo
            .get("package")
            .ok_or_else(|| DocsError::Config("Missing [package] in Cargo.toml".to_string()))?;

        let name = package
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DocsError::Config("Missing package name".to_string()))?
            .to_string();

        let version = package
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let description = package
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        info!("Generating docs for crate: {} v{}", name, version);

        // Find the lib.rs or main.rs
        let src_dir = crate_path.join("src");
        let root_file = if src_dir.join("lib.rs").exists() {
            src_dir.join("lib.rs")
        } else if src_dir.join("main.rs").exists() {
            src_dir.join("main.rs")
        } else {
            return Err(DocsError::MissingContent("No lib.rs or main.rs found".to_string()));
        };

        // Parse root module
        let root_module = ModuleDoc::parse_file(&root_file)?;

        // Recursively parse all modules
        let mut all_modules = Vec::new();
        self.collect_modules(&src_dir, &name, &mut all_modules)?;

        // Build public items index
        let public_items = self.build_items_index(&all_modules);

        Ok(CrateDocumentation {
            name,
            version,
            description,
            root_module,
            all_modules,
            public_items,
        })
    }

    /// Generate documentation for a single file
    pub fn generate_file(&self, file_path: &Path) -> DocsResult<ModuleDoc> {
        ModuleDoc::parse_file(file_path)
    }

    /// Collect all modules recursively
    fn collect_modules(
        &self,
        dir: &Path,
        parent_path: &str,
        modules: &mut Vec<ModuleDoc>,
    ) -> DocsResult<()> {
        for entry in WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "rs") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    // Skip mod.rs (handled separately)
                    if name == "mod" {
                        continue;
                    }

                    let module_path = format!("{}::{}", parent_path, name);

                    match ModuleDoc::parse_file(path) {
                        Ok(mut module) => {
                            module.path = module_path;
                            modules.push(module);
                        }
                        Err(e) => {
                            warn!("Failed to parse {:?}: {}", path, e);
                        }
                    }
                }
            } else if path.is_dir() && path != dir {
                // Check for mod.rs in subdirectory
                let mod_file = path.join("mod.rs");
                if mod_file.exists() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        let module_path = format!("{}::{}", parent_path, name);

                        match ModuleDoc::parse_file(&mod_file) {
                            Ok(mut module) => {
                                module.path = module_path.clone();
                                modules.push(module);
                            }
                            Err(e) => {
                                warn!("Failed to parse {:?}: {}", mod_file, e);
                            }
                        }

                        // Recursively process subdirectory
                        self.collect_modules(path, &module_path, modules)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Build an index of public items
    fn build_items_index(&self, modules: &[ModuleDoc]) -> Vec<ItemSummary> {
        let mut items = Vec::new();

        for module in modules {
            for item in &module.items {
                if item.visibility == Visibility::Public {
                    items.push(ItemSummary {
                        name: item.name.clone(),
                        path: format!("{}::{}", module.path, item.name),
                        kind: item.kind,
                        description: item
                            .doc
                            .as_ref()
                            .map(|d| d.summary())
                            .unwrap_or_default(),
                    });
                }
            }
        }

        // Sort by name
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }

    /// Generate markdown documentation
    pub fn generate_markdown(&self, crate_docs: &CrateDocumentation) -> DocsResult<Vec<DocPage>> {
        let mut pages = Vec::new();

        // Generate index page
        pages.push(self.generate_index_page(crate_docs)?);

        // Generate module pages
        for module in &crate_docs.all_modules {
            pages.push(self.generate_module_page(module, &crate_docs.name)?);
        }

        // Generate item pages
        for module in &crate_docs.all_modules {
            for item in &module.items {
                if item.visibility == Visibility::Public {
                    pages.push(self.generate_item_page(item, module)?);
                }
            }
        }

        Ok(pages)
    }

    /// Generate the index page
    fn generate_index_page(&self, crate_docs: &CrateDocumentation) -> DocsResult<DocPage> {
        let mut content = String::new();

        content.push_str(&format!("# {}\n\n", crate_docs.name));

        if let Some(ref desc) = crate_docs.description {
            content.push_str(desc);
            content.push_str("\n\n");
        }

        content.push_str(&format!("**Version:** {}\n\n", crate_docs.version));

        // Root module documentation
        if let Some(ref doc) = crate_docs.root_module.doc {
            content.push_str(&doc.text);
            content.push_str("\n\n");
        }

        // Modules list
        content.push_str("## Modules\n\n");
        for module in &crate_docs.all_modules {
            content.push_str(&format!(
                "- [`{}`]({})\n",
                module.name,
                module.path.replace("::", "/")
            ));
        }

        content.push_str("\n## Items\n\n");
        content.push_str("| Name | Kind | Description |\n");
        content.push_str("|------|------|-------------|\n");

        for item in &crate_docs.public_items {
            content.push_str(&format!(
                "| [`{}`]({}) | {} | {} |\n",
                item.name,
                item.path.replace("::", "/"),
                item.kind.display_name(),
                item.description.lines().next().unwrap_or(""),
            ));
        }

        Ok(DocPage {
            id: format!("{}-index", crate_docs.name),
            title: crate_docs.name.clone(),
            path: PathBuf::from("index.md"),
            content,
            format: ContentFormat::Markdown,
            category: DocCategory::ApiReference,
            tags: vec!["api".to_string(), "crate".to_string()],
            parent: None,
            children: crate_docs.all_modules.iter().map(|m| m.name.clone()).collect(),
            modified: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Generate a module page
    fn generate_module_page(&self, module: &ModuleDoc, crate_name: &str) -> DocsResult<DocPage> {
        let mut content = String::new();

        content.push_str(&format!("# Module `{}`\n\n", module.name));
        content.push_str(&format!("Full path: `{}`\n\n", module.path));

        if let Some(ref doc) = module.doc {
            content.push_str(&doc.text);
            content.push_str("\n\n");
        }

        // Items in module
        let public_items: Vec<_> = module
            .items
            .iter()
            .filter(|i| i.visibility == Visibility::Public)
            .collect();

        if !public_items.is_empty() {
            content.push_str("## Items\n\n");

            // Group by kind
            for kind in [
                ItemKind::Struct,
                ItemKind::Enum,
                ItemKind::Trait,
                ItemKind::Function,
                ItemKind::TypeAlias,
                ItemKind::Constant,
                ItemKind::Static,
                ItemKind::Macro,
            ] {
                let items: Vec<_> = public_items.iter().filter(|i| i.kind == kind).collect();
                if !items.is_empty() {
                    content.push_str(&format!("### {}s\n\n", kind.display_name()));
                    for item in items {
                        content.push_str(&format!("- [`{}`]({})\n", item.name, item.name));
                        if let Some(ref doc) = item.doc {
                            let summary = doc.summary();
                            if !summary.is_empty() {
                                content.push_str(&format!("  - {}\n", summary));
                            }
                        }
                    }
                    content.push_str("\n");
                }
            }
        }

        // Submodules
        if !module.submodules.is_empty() {
            content.push_str("## Submodules\n\n");
            for submod in &module.submodules {
                content.push_str(&format!("- [`{}`]({})\n", submod.name, submod.name));
            }
        }

        Ok(DocPage {
            id: module.name.clone(),
            title: format!("Module {}", module.name),
            path: PathBuf::from(format!("{}.md", module.name)),
            content,
            format: ContentFormat::Markdown,
            category: DocCategory::ApiReference,
            tags: vec!["api".to_string(), "module".to_string()],
            parent: Some(format!("{}-index", crate_name)),
            children: module.items.iter().map(|i| i.name.clone()).collect(),
            modified: Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Generate an item page
    fn generate_item_page(&self, item: &ItemDoc, module: &ModuleDoc) -> DocsResult<DocPage> {
        let mut content = String::new();

        content.push_str(&format!(
            "# {} `{}`\n\n",
            item.kind.display_name(),
            item.name
        ));

        if let Some(ref signature) = item.signature {
            content.push_str("```rust\n");
            content.push_str(signature);
            content.push_str("\n```\n\n");
        }

        if let Some(ref doc) = item.doc {
            content.push_str(&doc.text);
            content.push_str("\n\n");

            // Examples
            if !doc.examples.is_empty() {
                content.push_str("## Examples\n\n");
                for example in &doc.examples {
                    content.push_str(&format!("```{}\n{}\n```\n\n", example.language, example.code));
                }
            }
        }

        // Associated items
        if !item.associated.is_empty() {
            content.push_str("## Associated Items\n\n");
            for assoc in &item.associated {
                content.push_str(&format!("### `{}`\n\n", assoc.name));
                if let Some(ref doc) = assoc.doc {
                    content.push_str(&doc.summary());
                    content.push_str("\n\n");
                }
            }
        }

        Ok(DocPage {
            id: format!("{}-{}", module.name, item.name),
            title: format!("{} {}", item.kind.display_name(), item.name),
            path: PathBuf::from(format!("{}/{}.md", module.name, item.name)),
            content,
            format: ContentFormat::Markdown,
            category: DocCategory::ApiReference,
            tags: vec![
                "api".to_string(),
                item.kind.display_name().to_lowercase(),
            ],
            parent: Some(module.name.clone()),
            children: Vec::new(),
            modified: Utc::now(),
            metadata: HashMap::new(),
        })
    }
}

impl Default for CodeDocGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = CodeDocGenerator::new();
        // Just verify it creates successfully
        assert!(true);
    }
}
