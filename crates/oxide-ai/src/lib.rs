//! OxideKit AI Connector
//!
//! This crate provides an AI-native interface for OxideKit, enabling AI assistants
//! to reliably discover, validate, and compose UI components without hallucination.
//!
//! ## Features
//!
//! - **Machine-Readable Catalog** (`oxide.ai.json`): Structured component specs, extensions,
//!   themes, and recipes for AI consumption.
//!
//! - **MCP Server**: Model Context Protocol server exposing OxideKit knowledge as tools
//!   that AI assistants can call directly.
//!
//! - **Template Extractor**: Extract tagged parts from design packs with full dependency graphs.
//!
//! - **Validation**: Validate UI files against the component registry with machine-readable errors.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use oxide_ai::prelude::*;
//!
//! // Create AI schema catalog
//! let catalog = AiCatalog::with_core();
//!
//! // Export to JSON
//! let json = catalog.export_json()?;
//!
//! // Start MCP server
//! let server = McpServer::new(catalog);
//! server.serve("127.0.0.1:9090").await?;
//! ```

mod schema;
mod extensions;
mod recipes;
mod design_pack;
mod mcp;
mod context;
mod prompts;
mod export;
mod validation;

pub use schema::*;
pub use extensions::*;
pub use recipes::*;
pub use design_pack::*;
pub use mcp::*;
pub use context::*;
pub use prompts::*;
pub use export::*;
pub use validation::*;

/// Prelude for common imports
pub mod prelude {
    pub use crate::schema::{AiCatalog, AiSchema, SchemaVersion};
    pub use crate::extensions::{ExtensionSpec, ExtensionKind, Permission, Capability};
    pub use crate::recipes::{Recipe, RecipeStep, PatchPlan};
    pub use crate::design_pack::{DesignPack, TemplatePart, CompositionGraph};
    pub use crate::mcp::{McpServer, McpMethod, McpRequest, McpResponse};
    pub use crate::context::{AiContext, ProjectContext};
    pub use crate::prompts::{PromptTemplate, PromptLibrary};
    pub use crate::export::{AiExporter, ExportOptions};
    pub use crate::validation::{AiValidator, ValidationReport};
}

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// AI schema format version
pub const SCHEMA_VERSION: &str = "1.0";
