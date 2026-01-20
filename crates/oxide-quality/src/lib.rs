//! OxideKit Quality Gates
//!
//! Platform-wide quality gates to prevent degradation over time.
//!
//! # Design Principles
//!
//! - **Quality gates are non-negotiable** - accessibility, performance, and security
//!   failures block releases
//! - **Machine-readable output** - all checks produce structured reports
//! - **CI/CD first** - designed for automated pipelines
//! - **Developer friendly** - clear error messages with actionable fixes
//!
//! # Features
//!
//! - `full` - Enable all quality gates
//! - `a11y` - Accessibility validation (WCAG AA compliance)
//! - `perf` - Performance budget checking
//! - `security` - Security audit and unsafe code detection
//! - `bundle` - Bundle size analysis
//! - `ci` - CI/CD integration helpers
//! - `hooks` - Pre-commit hook generation
//! - `visual-regression` - Visual regression testing with screenshot comparison
//!
//! # Quality Gates
//!
//! 1. **Lint** - Static analysis of .oui files
//! 2. **Accessibility** - Screen reader, keyboard, and contrast checks
//! 3. **Performance** - Frame time, allocation, and memory budgets
//! 4. **Security** - Dependency audit, unsafe code review
//! 5. **Bundle** - Size limits and dependency analysis
//! 6. **Visual Regression** - Screenshot comparison and diff generation

mod error;
pub mod lint;
mod report;
mod config;

#[cfg(feature = "a11y")]
pub mod a11y;

#[cfg(feature = "perf")]
pub mod perf;

#[cfg(feature = "security")]
pub mod security;

#[cfg(feature = "bundle")]
pub mod bundle;

#[cfg(feature = "ci")]
pub mod ci;

#[cfg(feature = "hooks")]
pub mod hooks;

#[cfg(feature = "visual-regression")]
pub mod visual_regression;

// Re-export types
pub use error::*;
pub use lint::{
    LintReport, LintViolation, LintSeverity, LintRule, RuleCategory, SeverityCounts,
    all_rules, get_enabled_rules, get_rule, get_rules_by_category,
};
pub use report::*;
pub use config::*;

#[cfg(feature = "a11y")]
pub use a11y::{A11yReport, A11yViolation, A11yImpact};

#[cfg(feature = "perf")]
pub use perf::{PerfReport, PerfViolation, PerfBudget, PerfCategory, ComplexityScore, ComplexityFactors, PerfSummary};

#[cfg(feature = "security")]
pub use security::{SecurityReport, SecurityViolation, SecurityVulnerability, VulnerabilitySeverity, SecurityCategory, UnsafeUsage, ForbiddenApiUsage, PermissionIssue};

#[cfg(feature = "bundle")]
pub use bundle::{BundleReport, BundleFile, BundleViolation, SizeChange, Sbom, SbomComponent, format_size, save_baseline, generate_sbom};

#[cfg(feature = "ci")]
pub use ci::{CiGenerationResult, CiFile, generate_ci_config, generate_minimal_ci, generate_badge_url};

#[cfg(feature = "hooks")]
pub use hooks::{HookConfig, generate_pre_commit_hook, generate_commit_msg_hook, generate_pre_push_hook, generate_husky_config, generate_lefthook_config, install_hook, uninstall_hook};

#[cfg(feature = "visual-regression")]
pub use visual_regression::{
    VisualRegressionReport, VisualRegressionResult, ComparisonResult, DiffStats,
    ScreenshotCapture, ImageComparison, BaselineManager, VisualReportGenerator,
    run_visual_regression, update_baselines, generate_visual_report,
};

use std::path::Path;

/// Run all quality gates on a project
pub fn check_all(project_path: &Path, config: &QualityConfig) -> QualityReport {
    let mut report = QualityReport::new(project_path);

    // Always run lint
    let lint_results = lint::check(project_path, &config.lint);
    report.add_section(QualitySection::Lint(lint_results));

    // Conditional checks based on features
    #[cfg(feature = "a11y")]
    {
        let a11y_results = a11y::check(project_path, &config.a11y);
        report.add_section(QualitySection::Accessibility(a11y_results));
    }

    #[cfg(feature = "perf")]
    {
        let perf_results = perf::check(project_path, &config.perf);
        report.add_section(QualitySection::Performance(perf_results));
    }

    #[cfg(feature = "security")]
    {
        let security_results = security::check(project_path, &config.security);
        report.add_section(QualitySection::Security(security_results));
    }

    #[cfg(feature = "bundle")]
    {
        let bundle_results = bundle::check(project_path, &config.bundle);
        report.add_section(QualitySection::Bundle(bundle_results));
    }

    report.finalize();
    report
}

/// Quick lint check only
pub fn run_lint(project_path: &Path) -> LintReport {
    lint::check(project_path, &LintConfig::default())
}

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        QualityConfig, QualityReport, QualitySection, QualityStatus,
        LintConfig, LintReport, LintRule, LintSeverity, LintViolation,
        check_all, run_lint,
    };

    #[cfg(feature = "a11y")]
    pub use crate::{A11yConfig, A11yReport, A11yViolation, WcagLevel};

    #[cfg(feature = "perf")]
    pub use crate::{PerfConfig, PerfReport, PerfBudget, PerfViolation};

    #[cfg(feature = "security")]
    pub use crate::{SecurityConfig, SecurityReport, SecurityViolation};

    #[cfg(feature = "bundle")]
    pub use crate::{BundleConfig, BundleReport, BundleViolation};

    #[cfg(feature = "ci")]
    pub use crate::{CiConfig, generate_ci_config};

    #[cfg(feature = "hooks")]
    pub use crate::{generate_pre_commit_hook, HookConfig};

    #[cfg(feature = "visual-regression")]
    pub use crate::{
        VisualRegressionConfig, VisualRegressionReport, run_visual_regression,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_quality_report_creation() {
        let report = QualityReport::new(&PathBuf::from("/test/project"));
        assert_eq!(report.status, QualityStatus::Pending);
    }
}
