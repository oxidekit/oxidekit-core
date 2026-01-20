//! Documentation generation from source code
//!
//! This module provides tools to extract documentation from Rust source files
//! and generate API reference documentation.

mod generator;
mod parser;

pub use generator::CodeDocGenerator;
pub use parser::{DocComment, ModuleDoc, ItemDoc, ItemKind};

use crate::DocsResult;
use std::path::Path;

/// Generate documentation from a Rust crate
pub fn generate_crate_docs(crate_path: &Path) -> DocsResult<generator::CrateDocumentation> {
    let generator = CodeDocGenerator::new();
    generator.generate_crate(crate_path)
}

/// Generate documentation from a single file
pub fn generate_file_docs(file_path: &Path) -> DocsResult<ModuleDoc> {
    let generator = CodeDocGenerator::new();
    generator.generate_file(file_path)
}
