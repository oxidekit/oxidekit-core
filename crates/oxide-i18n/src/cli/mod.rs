//! CLI commands for i18n management
//!
//! Provides `oxide i18n` commands for key extraction, validation,
//! and team workflow support.

pub mod check;
pub mod extract;
pub mod team_workflow;

use clap::Subcommand;

/// I18n subcommands
#[derive(Subcommand)]
pub enum I18nCommands {
    /// Extract translation keys from source files
    Extract {
        /// Source directory to scan (default: current directory)
        #[arg(short, long)]
        source: Option<String>,

        /// Output file for extracted keys (JSON)
        #[arg(short, long, default_value = "i18n/keys.json")]
        output: String,

        /// File extensions to scan (comma-separated)
        #[arg(long, default_value = "oui,rs")]
        extensions: String,

        /// Generate human-readable keys file
        #[arg(long)]
        human_readable: bool,

        /// Export AI schema
        #[arg(long)]
        ai_schema: bool,
    },

    /// Check translations for completeness and consistency
    Check {
        /// Directory containing translation files
        #[arg(short = 'd', long, default_value = "i18n")]
        translations_dir: String,

        /// Source directory to scan for key extraction
        #[arg(short, long)]
        source: Option<String>,

        /// Base locale to compare against
        #[arg(short, long, default_value = "en")]
        base: String,

        /// Specific locale to check (all if not specified)
        #[arg(short, long)]
        locale: Option<String>,

        /// Strict mode (fail on warnings too)
        #[arg(long)]
        strict: bool,

        /// Output format (human, json)
        #[arg(long, default_value = "human")]
        format: String,

        /// Output file for report (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// In-progress locales (comma-separated, report as warnings)
        #[arg(long)]
        in_progress: Option<String>,

        /// Skip orphaned key check
        #[arg(long)]
        skip_orphaned: bool,

        /// Skip placeholder check
        #[arg(long)]
        skip_placeholders: bool,

        /// Skip plural forms check
        #[arg(long)]
        skip_plurals: bool,

        /// Length warning threshold (percentage, e.g., 50 for 50%)
        #[arg(long)]
        length_threshold: Option<u32>,
    },

    /// Initialize i18n for a project
    Init {
        /// Directory for translation files
        #[arg(short = 'd', long, default_value = "i18n")]
        translations_dir: String,

        /// Default locale
        #[arg(short, long, default_value = "en")]
        default_locale: String,

        /// Additional locales to create (comma-separated)
        #[arg(short, long)]
        locales: Option<String>,
    },

    /// Add a new locale
    Add {
        /// Locale to add (e.g., "fr", "de-DE")
        locale: String,

        /// Directory containing translation files
        #[arg(short = 'd', long, default_value = "i18n")]
        translations_dir: String,

        /// Copy translations from this locale as a starting point
        #[arg(long)]
        copy_from: Option<String>,
    },

    /// Show i18n status and statistics
    Status {
        /// Directory containing translation files
        #[arg(short = 'd', long, default_value = "i18n")]
        translations_dir: String,

        /// Show detailed key-by-key status
        #[arg(long)]
        detailed: bool,
    },

    // =========================================================================
    // Team Workflow Commands
    // =========================================================================

    /// Export translations for external translators
    Export {
        /// Target locale to export
        locale: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Export format (xliff, json, po)
        #[arg(short, long, default_value = "xliff")]
        format: String,

        /// Directory containing translation files
        #[arg(short = 'd', long, default_value = "i18n")]
        translations_dir: String,

        /// Include only untranslated keys
        #[arg(long)]
        untranslated_only: bool,

        /// Include context and notes for translators
        #[arg(long)]
        include_context: bool,
    },

    /// Import translations from external translators
    Import {
        /// Input file from translator
        file: String,

        /// Directory containing translation files
        #[arg(short = 'd', long, default_value = "i18n")]
        translations_dir: String,

        /// Mark imported translations as needing review
        #[arg(long)]
        needs_review: bool,

        /// Overwrite existing translations
        #[arg(long)]
        overwrite: bool,

        /// Dry run - show what would be imported
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate translation coverage report
    Coverage {
        /// Directory containing translation files
        #[arg(short = 'd', long, default_value = "i18n")]
        translations_dir: String,

        /// Output format (human, json, markdown, html)
        #[arg(short, long, default_value = "human")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Specific locale to report on
        #[arg(short, long)]
        locale: Option<String>,

        /// Show keys with issues
        #[arg(long)]
        show_keys: bool,
    },

    /// Run translation quality checks
    Quality {
        /// Target locale to check
        locale: String,

        /// Directory containing translation files
        #[arg(short = 'd', long, default_value = "i18n")]
        translations_dir: String,

        /// Output format (human, json, markdown)
        #[arg(short, long, default_value = "human")]
        format: String,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Fail on warnings (strict mode)
        #[arg(long)]
        strict: bool,
    },

    /// Lock translation keys to prevent conflicts
    Lock {
        #[command(subcommand)]
        action: LockCommands,
    },

    /// Manage string freeze workflow
    Freeze {
        #[command(subcommand)]
        action: FreezeCommands,
    },

    /// Translation diff between versions
    Diff {
        /// First file or version
        from: String,

        /// Second file or version
        to: String,

        /// Output format (human, json, markdown)
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Translation memory operations
    Memory {
        #[command(subcommand)]
        action: MemoryCommands,
    },
}

/// Lock subcommands
#[derive(Subcommand)]
pub enum LockCommands {
    /// Lock a key or key pattern
    Acquire {
        /// Key or pattern to lock (e.g., "auth.*")
        key: String,

        /// Lock duration in hours
        #[arg(short, long, default_value = "4")]
        duration: u32,

        /// Note about what you're working on
        #[arg(short, long)]
        note: Option<String>,
    },

    /// Release a lock
    Release {
        /// Key to release
        key: String,
    },

    /// Show current locks
    List {
        /// Show only your locks
        #[arg(long)]
        mine: bool,
    },

    /// Check lock status for a key
    Status {
        /// Key to check
        key: String,
    },

    /// Release all your locks
    ReleaseAll,
}

/// Freeze subcommands
#[derive(Subcommand)]
pub enum FreezeCommands {
    /// Start a string freeze for a version
    Start {
        /// Version being frozen (e.g., "1.0.0")
        version: String,
    },

    /// Advance to the next freeze phase
    Advance {
        /// Reason for advancing
        #[arg(short, long)]
        reason: Option<String>,
    },

    /// Show current freeze status
    Status,

    /// Add an exception for a key during freeze
    Exception {
        /// Key to add exception for
        key: String,
    },

    /// Reset freeze (back to development)
    Reset,
}

/// Memory subcommands
#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Add translations to memory
    Add {
        /// Source text
        source: String,

        /// Translation
        target: String,

        /// Source language
        #[arg(long, default_value = "en")]
        source_lang: String,

        /// Target language
        #[arg(long)]
        target_lang: String,
    },

    /// Search translation memory
    Search {
        /// Text to search for
        query: String,

        /// Source language
        #[arg(long, default_value = "en")]
        source_lang: String,

        /// Target language
        #[arg(long)]
        target_lang: String,

        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Import translations into memory
    Import {
        /// File to import
        file: String,
    },

    /// Export translation memory
    Export {
        /// Output file
        output: String,
    },

    /// Show memory statistics
    Stats,
}
