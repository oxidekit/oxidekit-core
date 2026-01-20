//! OxideKit Language Server Protocol (LSP) Implementation
//!
//! Provides IDE support for OxideKit through the Language Server Protocol:
//! - Autocomplete for components, props, tokens, translation keys
//! - Diagnostics for invalid props, missing keys, deprecated APIs
//! - Jump-to-definition for components, tokens, plugins
//! - Hover information with descriptions
//! - Code actions for quick fixes
//!
//! # Supported File Types
//!
//! - `.oui` - Oxide UI language files
//! - `oxide.toml` - Project configuration
//! - `plugin.toml` - Plugin configuration
//! - `theme.toml` - Theme configuration
//! - `typography.toml` - Typography configuration
//! - `fonts.toml` - Font configuration
//!
//! # Architecture
//!
//! The LSP server is stateless where possible, reading from:
//! - `oxide.ai.json` - AI-generated component metadata
//! - `extensions.lock` - Locked dependency versions
//! - Local project manifests
//! - Installed plugin schemas

pub mod analysis;
pub mod capabilities;
pub mod completion;
pub mod diagnostics;
pub mod document;
pub mod hover;
pub mod jump;
pub mod project;
pub mod protocol;
pub mod server;

use thiserror::Error;

pub use analysis::*;
pub use capabilities::*;
pub use completion::*;
pub use diagnostics::LspDiagnostic;
pub use document::*;
pub use hover::*;
pub use jump::*;
pub use project::*;
pub use protocol::*;
pub use server::*;

/// LSP-specific errors
#[derive(Debug, Error)]
pub enum LspError {
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Invalid document URI: {0}")]
    InvalidUri(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Project not found at: {0}")]
    ProjectNotFound(String),

    #[error("Schema error: {0}")]
    SchemaError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    TomlError(#[from] toml::de::Error),
}

/// Result type for LSP operations
pub type LspResult<T> = Result<T, LspError>;

/// Common prelude for using oxide-lsp
pub mod prelude {
    pub use crate::completion::{CompletionEngine, CompletionKind, OxideCompletionItem};
    pub use crate::diagnostics::DiagnosticsEngine;
    pub use crate::hover::{HoverEngine, HoverInfo};
    pub use crate::jump::{JumpEngine, JumpTarget};
    pub use crate::project::ProjectContext;
    pub use crate::server::OxideLspServer;
    pub use crate::{LspError, LspResult};
}
