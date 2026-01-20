//! OxideKit Internationalization (i18n) System
//!
//! A comprehensive i18n solution for OxideKit that provides:
//! - Runtime translation lookup with `t!()` macro
//! - Multiple file formats (TOML, JSON, XLIFF, PO)
//! - Locale detection and fallback chains
//! - Pluralization support
//! - RTL language support
//! - Translation key extraction and validation
//! - Team workflow support (locking, freeze, review)
//! - Translation memory and suggestions
//! - Coverage and quality reports
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use oxide_i18n::{I18n, t};
//!
//! // Initialize with translations directory
//! let i18n = I18n::load("i18n/").unwrap();
//!
//! // Set the current locale
//! i18n.set_locale("en-US").unwrap();
//!
//! // Translate a key
//! let greeting = t!("welcome.message");
//!
//! // With interpolation
//! let personalized = t!("welcome.user", name = "Alice");
//!
//! // With pluralization
//! let items = t!("cart.items", count = 5);
//! ```
//!
//! # Translation File Format
//!
//! Translation files use TOML format with nested namespaces:
//!
//! ```toml
//! # i18n/en.toml
//! [auth.login]
//! title = "Sign in"
//! button = "Continue"
//! welcome = "Welcome, {name}!"
//!
//! [cart]
//! items = { one = "{count} item", other = "{count} items" }
//! ```
//!
//! # Team Workflow
//!
//! For teams, oxide-i18n provides:
//! - Key locking to prevent conflicts
//! - String freeze workflow for releases
//! - Review and approval process
//! - Translation memory for consistency
//! - Export/import for external translators (XLIFF, PO)
//!
//! ```bash
//! # Export for translators
//! oxide i18n export de --format xliff
//!
//! # Import translations
//! oxide i18n import de.xliff --needs-review
//!
//! # Check coverage
//! oxide i18n coverage
//!
//! # Run quality checks
//! oxide i18n quality de
//! ```

// Core modules
pub mod cli;
pub mod error;
pub mod extractor;
pub mod format;
pub mod locale;
pub mod macros;
pub mod pluralization;
pub mod rtl;
pub mod runtime;
pub mod validation;

// Team workflow modules
pub mod formats;
pub mod memory;
pub mod reports;
pub mod services;
pub mod workflow;

// Re-exports for convenience
pub use error::{I18nError, I18nResult};
pub use extractor::KeyExtractor;
pub use format::{TranslationFile, TranslationValue};
pub use locale::{Locale, LocaleRegistry};
pub use pluralization::{PluralCategory, PluralRules};
pub use rtl::{Direction, RtlSupport};
pub use runtime::I18n;
pub use validation::{ValidationIssue, ValidationReport, Validator};

// Re-exports for team workflow
pub use formats::{
    CoverageStats, Format, TranslationEntry, TranslationState,
};
pub use memory::{extract_placeholders, FuzzyMatch, FuzzyMatcher, MemoryEntry, MemoryStats, Suggestion, SuggestionEngine, SuggestionSource, TranslationMemory};
pub use reports::{
    CoverageReport, KeyCoverage, LocaleCoverage, QualityConfig, QualityIssue, QualityLevel,
    QualityReport, ReportExporter, ReportFormat,
};
pub use workflow::{
    ChangeType, ConventionChecker, ConventionViolation, DiffEntry, FreezePhase, FreezeStatus,
    KeyLock, KeyNamingConvention, LockManager, LockStatus, ReviewComment, ReviewStatus,
    ReviewWorkflow, ReviewerRole, StringFreeze, TranslationDiff,
};

/// Common prelude for using oxide-i18n
pub mod prelude {
    pub use crate::error::{I18nError, I18nResult};
    pub use crate::formats::{TranslationEntry, TranslationState};
    pub use crate::reports::{CoverageReport, QualityReport};
    pub use crate::runtime::I18n;
    pub use crate::t;
    pub use crate::workflow::{FreezePhase, LockManager, StringFreeze};
}
