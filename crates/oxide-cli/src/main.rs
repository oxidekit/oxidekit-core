//! OxideKit CLI
//!
//! The `oxide` command-line tool for creating, developing, and building OxideKit apps.

use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;

#[derive(Parser)]
#[command(name = "oxide")]
#[command(version, about = "OxideKit - A Rust-native application platform")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new OxideKit project (interactive wizard)
    New {
        /// Name of the project (prompts if not provided)
        name: Option<String>,

        /// Project template (default, minimal) - deprecated, use --starter
        #[arg(short, long, default_value = "default", hide = true)]
        template: String,

        /// Use a starter template (e.g., admin-panel, desktop-wallet)
        #[arg(short, long)]
        starter: Option<String>,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,

        /// Skip interactive prompts (use defaults)
        #[arg(short, long)]
        yes: bool,

        /// Plugin presets to include (core, desktop, web, native, network, storage, crypto, full)
        #[arg(short, long, value_delimiter = ',')]
        plugins: Vec<String>,

        /// Theme (dark, light, system, high-contrast, custom)
        #[arg(long)]
        theme: Option<String>,

        /// Initialize a git repository
        #[arg(long)]
        git: Option<bool>,

        /// Skip git initialization
        #[arg(long, conflicts_with = "git")]
        no_git: bool,
    },

    /// Initialize current directory as OxideKit project (interactive wizard)
    Init {
        /// Use a starter template (e.g., admin-panel, desktop-wallet)
        #[arg(short, long)]
        starter: Option<String>,

        /// Skip interactive prompts (use defaults)
        #[arg(short, long)]
        yes: bool,

        /// Plugin presets to include (core, desktop, web, native, network, storage, crypto, full)
        #[arg(short, long, value_delimiter = ',')]
        plugins: Vec<String>,

        /// Theme (dark, light, system, high-contrast, custom)
        #[arg(long)]
        theme: Option<String>,
    },

    /// Start development server with hot reload
    Dev {
        /// Port for dev server
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Open in browser
        #[arg(long)]
        open: bool,
    },

    /// Build the application
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Target platform
        #[arg(short, long)]
        target: Option<String>,
    },

    /// Run the application
    Run {
        /// Run in release mode
        #[arg(short, long)]
        release: bool,
    },

    /// Format code
    Fmt {
        /// Check only, don't modify files
        #[arg(long)]
        check: bool,
    },

    /// Lint code
    Lint,

    /// Diagnose project issues
    Doctor,

    /// Check code for various issues
    Check {
        #[command(subcommand)]
        action: CheckCommands,
    },

    /// Export project artifacts
    Export {
        #[command(subcommand)]
        what: ExportCommands,
    },

    /// Diagnostics management
    Diagnostics {
        #[command(subcommand)]
        action: DiagnosticsCommands,
    },

    /// Metrics and observability
    Metrics {
        #[command(subcommand)]
        action: MetricsCommands,
    },

    /// Starter templates
    Starters {
        #[command(subcommand)]
        action: StartersCommands,
    },

    /// Internationalization (i18n) management
    I18n {
        #[command(subcommand)]
        action: I18nCommands,
    },

    /// Interactive learning tutorials
    Learn {
        /// Specific topic to learn about
        topic: Option<String>,

        /// List all available tutorials
        #[arg(short, long)]
        list: bool,
    },

    /// Scan and display dependency licenses
    Licenses {
        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,

        /// Check against a license policy (permissive, copyleft, commercial, or path to policy file)
        #[arg(short, long)]
        policy: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Generate Software Bill of Materials (SBOM)
    Sbom {
        /// SBOM format (spdx-json, spdx-tv, cyclonedx, cyclonedx-xml, oxidekit)
        #[arg(short, long, default_value = "spdx-json")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Check license compliance
    Compliance {
        /// License policy (permissive, copyleft, commercial, or path to policy file)
        #[arg(short, long, default_value = "permissive")]
        policy: String,

        /// Report format (text, json, html, csv, markdown)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Generate third-party NOTICE file
    Notice {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Legal and compliance tools
    Legal {
        #[command(subcommand)]
        action: LegalCommands,
    },

    /// Plugin management
    Plugin {
        #[command(subcommand)]
        action: PluginCommands,
    },

    /// Add a plugin (shorthand for `plugin add`)
    Add {
        /// Plugin specifier (e.g., ui.tables, git github.com/acme/plugin@v1.0.0, path ../my-plugin)
        specifier: String,
    },

    /// Documentation and learning
    Docs {
        #[command(subcommand)]
        action: DocsCommands,
    },

    /// Version checking and compatibility enforcement
    Version {
        #[command(subcommand)]
        action: VersionCommands,
    },

    /// Check compatibility between components (shorthand for `version compat`)
    CompatCheck {
        /// Component name to check
        component: String,

        /// Version to check
        version: String,
    },

    /// Migration tools for Electron/Tauri projects
    Migrate {
        #[command(subcommand)]
        action: MigrateCommands,
    },

    /// Compatibility layer management (WebView, JS runtime, NPM bundling)
    CompatLayer {
        #[command(subcommand)]
        action: CompatLayerCommands,
    },

    /// Figma design translator (design-to-code bridge)
    Figma {
        #[command(subcommand)]
        action: FigmaCommands,
    },

    /// Mobile development commands (iOS/Android)
    Mobile(commands::mobile::MobileArgs),
}

/// Figma commands for design translation
#[derive(Subcommand)]
enum FigmaCommands {
    /// Pull and translate a Figma file
    Pull {
        /// Figma file URL or file key
        url: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,

        /// Generate dark theme (default)
        #[arg(long, default_value = "true")]
        dark: bool,

        /// Theme name (defaults to Figma file name)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Start continuous sync with Figma
    Sync {
        /// Figma file URL or file key
        url: String,

        /// Poll interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,

        /// Auto-apply safe (non-breaking) changes
        #[arg(long)]
        auto_apply: bool,
    },

    /// Show diff between Figma and local files
    Diff {
        /// Figma file URL or file key
        url: String,
    },

    /// Import assets from Figma
    Import {
        /// Figma file URL or file key
        url: String,

        /// Output directory for assets
        #[arg(short, long)]
        output: Option<String>,

        /// Export format (svg, png, jpg, pdf)
        #[arg(short, long, default_value = "svg")]
        format: String,

        /// Export scale
        #[arg(short, long, default_value = "1.0")]
        scale: f32,
    },

    /// Get Figma file info
    Info {
        /// Figma file URL or file key
        url: String,
    },

    /// Export full design pack from Figma
    ExportDesign {
        /// Figma file URL or file key
        url: String,

        /// Output directory for design pack
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum ExportCommands {
    /// Export AI schema (oxide.ai.json)
    AiSchema {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Export typography roles (typography.toml)
    Typography {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Export theme (theme.toml)
    Theme {
        /// Theme name (dark, light)
        #[arg(default_value = "dark")]
        name: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum DiagnosticsCommands {
    /// Export diagnostics bundle to file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Preview diagnostics without exporting
    Preview,

    /// Run diagnostics system tests
    Test,

    /// Verify endpoint connectivity
    VerifyEndpoint {
        /// Endpoint URL to verify
        endpoint: String,
    },
}

#[derive(Subcommand)]
enum MetricsCommands {
    /// Show current metrics status
    Status,

    /// Export metrics snapshot
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Export format (json, json-compact)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Run health check
    Health,

    /// Run metrics system tests
    Test,

    /// Show Prometheus exporter info
    Prometheus {
        /// Port for Prometheus exporter
        #[arg(short, long, default_value = "9090")]
        port: u16,
    },

    /// Show metrics configuration help
    Config,

    /// Reset all metrics
    Reset,
}

#[derive(Subcommand)]
enum StartersCommands {
    /// List available starters
    List {
        /// Filter by category (admin, docs, website, wallet, monitoring, app)
        #[arg(short, long)]
        category: Option<String>,

        /// Filter by target (desktop, web, static)
        #[arg(short, long)]
        target: Option<String>,
    },

    /// Show starter details
    Info {
        /// Starter ID
        starter_id: String,
    },

    /// Search starters
    Search {
        /// Search query
        query: String,
    },
}

#[derive(Subcommand)]
enum LegalCommands {
    /// Check CLA status for a contributor
    Cla {
        /// Contributor email to check
        email: String,

        /// Path to CLA database file
        #[arg(short, long)]
        database: Option<String>,
    },

    /// Check export control status
    ExportControl {
        /// Check distribution to specific country (ISO 3166-1 alpha-2 code)
        #[arg(short, long)]
        country: Option<String>,
    },

    /// Create a license policy file
    CreatePolicy {
        /// Policy type (permissive, copyleft, commercial)
        #[arg(short, long, default_value = "permissive")]
        policy_type: String,

        /// Output file path
        #[arg(short, long, default_value = "license-policy.toml")]
        output: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// Create a new plugin project
    New {
        /// Plugin ID (e.g., ui.tables, native.keychain, theme.modern)
        plugin_id: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,

        /// Publisher name
        #[arg(short, long)]
        publisher: Option<String>,

        /// Plugin description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Install a plugin
    Add {
        /// Plugin specifier (e.g., ui.tables, git github.com/acme/plugin@v1.0.0, path ../my-plugin)
        specifier: String,
    },

    /// Uninstall a plugin
    Remove {
        /// Plugin ID
        plugin_id: String,
    },

    /// List installed plugins
    List {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Filter by namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },

    /// Verify a plugin's security
    Verify {
        /// Plugin ID
        plugin_id: String,
    },

    /// Show plugin information
    Info {
        /// Plugin ID
        plugin_id: String,
    },

    /// Search the registry for plugins
    Search {
        /// Search query
        query: String,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },
}

#[derive(Subcommand)]
enum DocsCommands {
    /// Open documentation (default: in browser)
    Open {
        /// Open offline documentation
        #[arg(long)]
        offline: bool,

        /// Specific topic to open
        topic: Option<String>,
    },

    /// Start local documentation server
    Serve {
        /// Port for documentation server
        #[arg(short, long, default_value = "3030")]
        port: u16,

        /// Open in browser after starting
        #[arg(short, long)]
        open: bool,
    },

    /// Build documentation bundle
    Build {
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Export documentation as archive
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Search documentation
    Search {
        /// Search query
        query: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Generate API documentation from code
    Generate {
        /// Crate path (default: current directory)
        #[arg(short, long)]
        path: Option<String>,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },

    /// List available tutorials
    Tutorials,

    /// Run a specific tutorial
    Tutorial {
        /// Tutorial ID
        tutorial_id: String,
    },
}

#[derive(Subcommand)]
enum VersionCommands {
    /// Check current versions and compatibility
    Check,

    /// Check compatibility between a component and core
    Compat {
        /// Component name
        component: String,

        /// Component version
        version: String,
    },

    /// Explain compatibility between two versions
    Explain {
        /// From version
        from: String,

        /// To version
        to: String,
    },

    /// Show upgrade path between versions
    UpgradePath {
        /// From version
        from: String,

        /// To version
        to: String,
    },

    /// Show deprecation warnings
    Deprecations {
        /// Target version (defaults to current)
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Show compatibility matrix
    Matrix {
        /// Core version (defaults to current)
        #[arg(short, long)]
        core: Option<String>,
    },

    /// Validate the lockfile
    ValidateLock,

    /// Update the lockfile
    UpdateLock,
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Analyze a project for migration
    Analyze {
        /// Path to the source project
        path: String,
    },

    /// Generate a migration plan
    Plan {
        /// Path to the source project
        path: String,

        /// Output file for the plan (JSON)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Run the migration
    Run {
        /// Path to the source project
        path: String,

        /// Dry run (don't make changes)
        #[arg(long)]
        dry_run: bool,
    },

    /// Scaffold a compatibility shim
    Scaffold {
        /// Name for the scaffold
        name: String,

        /// Kind of scaffold (plugin, component, widget, shim)
        #[arg(short, long, default_value = "plugin")]
        kind: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum CompatLayerCommands {
    /// Add a compatibility package (webview, js-runtime)
    Add {
        /// Package to add (webview, js-runtime)
        package: String,
    },

    /// Remove a compatibility package
    Remove {
        /// Package to remove
        package: String,
    },

    /// NPM build-time bundling commands
    Npm {
        #[command(subcommand)]
        action: NpmCommands,
    },

    /// Show compatibility layer status
    Status,
}

#[derive(Subcommand)]
enum NpmCommands {
    /// Add an NPM package for bundling
    Add {
        /// Package specifier (package@version)
        package: String,
    },

    /// Build NPM bundles
    Build,

    /// List NPM packages
    List,
}

#[derive(Subcommand)]
enum CheckCommands {
    /// Check code for cross-platform portability
    Portability {
        /// Target platform to check against
        #[arg(short, long)]
        target: Option<String>,

        /// Treat warnings as errors
        #[arg(short, long)]
        strict: bool,

        /// Check against all supported targets
        #[arg(long)]
        all_targets: bool,
    },

    /// Show portability info for a plugin
    Plugin {
        /// Path to plugin manifest
        #[arg(short, long)]
        manifest: Option<String>,
    },

    /// List all known targets and their capabilities
    Targets,
}

#[derive(Subcommand)]
enum I18nCommands {
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
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, template, starter, output, yes, plugins, theme, git, no_git } => {
            // Determine git setting: --no-git takes precedence, then --git, then None (prompt)
            let git_setting = if no_git { Some(false) } else { git };

            commands::new::run(
                name.as_deref(),
                &template,
                starter.as_deref(),
                output.as_deref(),
                yes,
                &plugins,
                theme.as_deref(),
                git_setting,
            )
        }
        Commands::Init { starter, yes, plugins, theme } => {
            commands::init::run(
                starter.as_deref(),
                yes,
                &plugins,
                theme.as_deref(),
            )
        }
        Commands::Dev { port, open } => commands::dev::run(port, open),
        Commands::Build { release, target } => commands::build::run(release, target),
        Commands::Run { release } => commands::run::run(release),
        Commands::Fmt { check } => commands::fmt::run(check),
        Commands::Lint => commands::lint::run(),
        Commands::Doctor => commands::doctor::run(),
        Commands::Check { action } => match action {
            CheckCommands::Portability { target, strict, all_targets } => {
                commands::check::run_portability(target, strict, all_targets)
            }
            CheckCommands::Plugin { manifest } => {
                commands::check::run_plugin_portability(manifest)
            }
            CheckCommands::Targets => {
                commands::check::run_list_targets()
            }
        },
        Commands::Export { what } => match what {
            ExportCommands::AiSchema { output } => {
                commands::export::run_ai_schema(output.as_deref())
            }
            ExportCommands::Typography { output } => {
                commands::export::run_typography(output.as_deref())
            }
            ExportCommands::Theme { name, output } => {
                commands::export::run_theme(&name, output.as_deref())
            }
        },
        Commands::Diagnostics { action } => match action {
            DiagnosticsCommands::Export { output } => {
                commands::diagnostics::run_export(output.as_deref())
            }
            DiagnosticsCommands::Preview => commands::diagnostics::run_preview(),
            DiagnosticsCommands::Test => commands::diagnostics::run_test(),
            DiagnosticsCommands::VerifyEndpoint { endpoint } => {
                commands::diagnostics::run_verify_endpoint(&endpoint)
            }
        },
        Commands::Metrics { action } => match action {
            MetricsCommands::Status => commands::metrics::run_status(),
            MetricsCommands::Export { output, format } => {
                commands::metrics::run_export(output.as_deref(), &format)
            }
            MetricsCommands::Health => commands::metrics::run_health(),
            MetricsCommands::Test => commands::metrics::run_test(),
            MetricsCommands::Prometheus { port } => {
                commands::metrics::run_prometheus_info(port)
            }
            MetricsCommands::Config => commands::metrics::run_config(),
            MetricsCommands::Reset => commands::metrics::run_reset(),
        },
        Commands::Starters { action } => match action {
            StartersCommands::List { category, target } => {
                commands::starters::run_list(category.as_deref(), target.as_deref())
            }
            StartersCommands::Info { starter_id } => {
                commands::starters::run_info(&starter_id)
            }
            StartersCommands::Search { query } => {
                commands::starters::run_search(&query)
            }
        },
        Commands::I18n { action } => match action {
            I18nCommands::Extract { source, output, extensions, human_readable, ai_schema } => {
                commands::i18n::run_extract(source, &output, &extensions, human_readable, ai_schema)
            }
            I18nCommands::Check {
                translations_dir, source, base, locale, strict, format, output,
                in_progress, skip_orphaned, skip_placeholders, skip_plurals, length_threshold
            } => {
                commands::i18n::run_check(
                    &translations_dir, source, &base, locale, strict, &format, output,
                    in_progress, skip_orphaned, skip_placeholders, skip_plurals, length_threshold
                )
            }
            I18nCommands::Init { translations_dir, default_locale, locales } => {
                commands::i18n::run_init(&translations_dir, &default_locale, locales)
            }
            I18nCommands::Add { locale, translations_dir, copy_from } => {
                commands::i18n::run_add(&locale, &translations_dir, copy_from)
            }
            I18nCommands::Status { translations_dir, detailed } => {
                commands::i18n::run_status(&translations_dir, detailed)
            }
        },
        Commands::Learn { topic, list } => {
            commands::learn::run(topic.as_deref(), list)
        }
        Commands::Licenses { format, policy, output } => {
            commands::legal::run_licenses(
                Some(format.as_str()),
                policy.as_deref(),
                output.as_deref(),
            )
        }
        Commands::Sbom { format, output } => {
            commands::legal::run_sbom(Some(format.as_str()), output.as_deref())
        }
        Commands::Compliance { policy, format, output } => {
            commands::legal::run_compliance(
                Some(policy.as_str()),
                Some(format.as_str()),
                output.as_deref(),
            )
        }
        Commands::Notice { output } => {
            commands::legal::run_notice(output.as_deref())
        }
        Commands::Legal { action } => match action {
            LegalCommands::Cla { email, database } => {
                commands::legal::run_cla_check(&email, database.as_deref())
            }
            LegalCommands::ExportControl { country } => {
                commands::legal::run_export_control(country.as_deref())
            }
            LegalCommands::CreatePolicy { policy_type, output } => {
                commands::legal::run_create_policy(&policy_type, &output)
            }
        },
        Commands::Plugin { action } => match action {
            PluginCommands::New { plugin_id, output, publisher, description } => {
                commands::plugin::run_new(
                    &plugin_id,
                    output.as_deref(),
                    publisher.as_deref(),
                    description.as_deref(),
                )
            }
            PluginCommands::Add { specifier } => {
                commands::plugin::run_add(&specifier)
            }
            PluginCommands::Remove { plugin_id } => {
                commands::plugin::run_remove(&plugin_id)
            }
            PluginCommands::List { category, namespace } => {
                commands::plugin::run_list(category.as_deref(), namespace.as_deref())
            }
            PluginCommands::Verify { plugin_id } => {
                commands::plugin::run_verify(&plugin_id)
            }
            PluginCommands::Info { plugin_id } => {
                commands::plugin::run_info(&plugin_id)
            }
            PluginCommands::Search { query, category } => {
                commands::plugin::run_search(&query, category.as_deref())
            }
        },
        Commands::Add { specifier } => {
            // Shorthand for `plugin add`
            commands::plugin::run_add(&specifier)
        },
        Commands::Docs { action } => match action {
            DocsCommands::Open { offline, topic } => {
                commands::docs::run_open(offline, topic.as_deref())
            }
            DocsCommands::Serve { port, open } => {
                commands::docs::run_serve(port, open, None)
            }
            DocsCommands::Build { output } => {
                commands::docs::run_build(output.as_deref())
            }
            DocsCommands::Export { output } => {
                commands::docs::run_export(output.as_deref())
            }
            DocsCommands::Search { query, limit } => {
                commands::docs::run_search(&query, limit)
            }
            DocsCommands::Generate { path, output } => {
                commands::docs::run_generate(path.as_deref(), output.as_deref())
            }
            DocsCommands::Tutorials => {
                commands::docs::run_tutorials_list()
            }
            DocsCommands::Tutorial { tutorial_id } => {
                commands::docs::run_tutorial(&tutorial_id)
            }
        },
        Commands::Version { action } => match action {
            VersionCommands::Check => commands::version::run_check(),
            VersionCommands::Compat { component, version } => {
                commands::version::run_compat(&component, &version)
            }
            VersionCommands::Explain { from, to } => {
                commands::version::run_explain(&from, &to)
            }
            VersionCommands::UpgradePath { from, to } => {
                commands::version::run_upgrade_path(&from, &to)
            }
            VersionCommands::Deprecations { version } => {
                commands::version::run_deprecations(version.as_deref())
            }
            VersionCommands::Matrix { core } => {
                commands::version::run_matrix(core.as_deref())
            }
            VersionCommands::ValidateLock => {
                commands::version::run_validate_lockfile()
            }
            VersionCommands::UpdateLock => {
                commands::version::run_update_lockfile()
            }
        },
        Commands::CompatCheck { component, version } => {
            // Shorthand for `version compat`
            commands::version::run_compat(&component, &version)
        },
        Commands::Migrate { action } => match action {
            MigrateCommands::Analyze { path } => {
                commands::migrate::run_analyze(&path)
            }
            MigrateCommands::Plan { path, output } => {
                commands::migrate::run_plan(&path, output.as_deref())
            }
            MigrateCommands::Run { path, dry_run } => {
                commands::migrate::run_migrate(&path, dry_run)
            }
            MigrateCommands::Scaffold { name, kind, output } => {
                commands::migrate::run_compat_scaffold(&name, &kind, output.as_deref())
            }
        },
        Commands::CompatLayer { action } => match action {
            CompatLayerCommands::Add { package } => {
                commands::compat::run_add(&package)
            }
            CompatLayerCommands::Remove { package } => {
                commands::compat::run_remove(&package)
            }
            CompatLayerCommands::Npm { action: npm_action } => match npm_action {
                NpmCommands::Add { package } => {
                    commands::compat::run_npm_add(&package)
                }
                NpmCommands::Build => {
                    commands::compat::run_npm_build()
                }
                NpmCommands::List => {
                    commands::compat::run_npm_list()
                }
            },
            CompatLayerCommands::Status => {
                commands::compat::run_status()
            }
        },
        Commands::Figma { action } => match action {
            FigmaCommands::Pull { url, output, dark, name } => {
                commands::figma::run_pull(&url, output.as_deref(), dark, name.as_deref())
            }
            FigmaCommands::Sync { url, interval, auto_apply } => {
                commands::figma::run_sync(&url, interval, auto_apply)
            }
            FigmaCommands::Diff { url } => {
                commands::figma::run_diff(&url)
            }
            FigmaCommands::Import { url, output, format, scale } => {
                commands::figma::run_import(&url, output.as_deref(), &format, scale)
            }
            FigmaCommands::Info { url } => {
                commands::figma::run_info(&url)
            }
            FigmaCommands::ExportDesign { url, output } => {
                commands::figma::run_export_design(&url, output.as_deref())
            }
        },
        Commands::Mobile(args) => commands::mobile::run(args),
    }
}
