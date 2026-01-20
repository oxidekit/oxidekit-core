//! OxideKit Figma Translator
//!
//! A first-class Figma translator that converts design artifacts into:
//! - Design tokens (colors, spacing, typography, shadows)
//! - Typography roles
//! - Component mappings
//! - Layout templates
//! - OxideKit-compatible UI files
//!
//! This is a **semantic design-to-system bridge**, not a pixel dump.
//! It translates design intent, not pixels.
//!
//! # Core Principles
//!
//! - Designers stay in Figma
//! - Developers do not manually recreate designs
//! - AI and tooling can reason about design intent
//! - Designs become reusable, versioned assets
//!
//! # Example
//!
//! ```no_run
//! use oxide_figma::{FigmaClient, FigmaConfig, Translator};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = FigmaConfig::from_env()?;
//! let client = FigmaClient::new(config);
//!
//! // Fetch a Figma file
//! let file = client.get_file("file_key").await?;
//!
//! // Translate to OxideKit
//! let translator = Translator::new();
//! let result = translator.translate(&file)?;
//!
//! // Export theme
//! result.export_theme("theme.generated.toml")?;
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod assets;
pub mod components;
pub mod diff;
pub mod error;
pub mod layout;
pub mod sync;
pub mod tokens;
pub mod translator;
pub mod types;
pub mod validation;

pub use api::{FigmaClient, FigmaConfig};
pub use error::{FigmaError, Result};
pub use translator::{TranslationResult, Translator};
pub use types::*;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::api::{FigmaClient, FigmaConfig};
    pub use crate::assets::{AssetDownloader, AssetDownloadBuilder};
    pub use crate::components::{ComponentMapper, ComponentMapping, MappingConfidence};
    pub use crate::diff::{DesignDiff, DiffResult};
    pub use crate::error::{FigmaError, Result};
    pub use crate::layout::LayoutTranslator;
    pub use crate::sync::{SyncConfig, SyncConfigBuilder, SyncEngine, SyncResult};
    pub use crate::tokens::{TokenExtractor, ExtractedTokens};
    pub use crate::translator::{TranslationResult, Translator, TranslatorBuilder, TranslatorConfig};
    pub use crate::types::*;
    pub use crate::validation::{ValidationReport, Validator};
}
