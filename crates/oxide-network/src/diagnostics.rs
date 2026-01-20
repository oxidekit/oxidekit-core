//! CORS doctor and network diagnostics for OxideKit applications.
//!
//! This module provides comprehensive diagnostic tools to detect and fix
//! network misconfigurations, particularly CORS-related issues that commonly
//! plague web development.
//!
//! # Features
//!
//! - **Network Check**: Validates configuration against best practices
//! - **CORS Doctor**: Detects CORS misconfigurations and suggests fixes
//! - **Connectivity Testing**: Verify API endpoints are reachable
//! - **Configuration Audit**: Review settings for security issues
//!
//! # Example
//!
//! ```rust
//! use oxide_network::diagnostics::{NetworkDoctor, DiagnosticReport};
//! use oxide_network::network_mode::{NetworkConfig, TargetPlatform};
//!
//! let config = NetworkConfig::for_target(TargetPlatform::Web);
//! let doctor = NetworkDoctor::new();
//! let report = doctor.diagnose(&config);
//!
//! println!("{}", report.summary());
//! for issue in report.issues() {
//!     println!("  - {}: {}", issue.severity, issue.message);
//! }
//! ```
//!
//! # CLI Integration
//!
//! ```bash
//! oxide network check
//! oxide doctor
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::cors::CorsConfig;
use crate::network_mode::{NetworkConfig, NetworkMode, TargetPlatform};
use crate::proxy::ProxyConfig;

/// Severity level for diagnostic issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational message.
    Info,
    /// Warning that should be reviewed.
    Warning,
    /// Error that will likely cause problems.
    Error,
    /// Critical issue that must be fixed.
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Category of diagnostic issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    /// CORS-related issue.
    Cors,
    /// Proxy configuration issue.
    Proxy,
    /// Network mode mismatch.
    NetworkMode,
    /// Security concern.
    Security,
    /// Performance issue.
    Performance,
    /// Configuration error.
    Configuration,
    /// Connectivity problem.
    Connectivity,
}

impl fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueCategory::Cors => write!(f, "CORS"),
            IssueCategory::Proxy => write!(f, "Proxy"),
            IssueCategory::NetworkMode => write!(f, "Network Mode"),
            IssueCategory::Security => write!(f, "Security"),
            IssueCategory::Performance => write!(f, "Performance"),
            IssueCategory::Configuration => write!(f, "Configuration"),
            IssueCategory::Connectivity => write!(f, "Connectivity"),
        }
    }
}

/// A diagnostic issue found during analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    /// Unique identifier for this issue type.
    pub code: String,
    /// Severity of the issue.
    pub severity: Severity,
    /// Category of the issue.
    pub category: IssueCategory,
    /// Human-readable message describing the issue.
    pub message: String,
    /// Detailed explanation.
    pub details: Option<String>,
    /// Suggested fix.
    pub fix: Option<String>,
    /// Related documentation URL.
    pub doc_url: Option<String>,
    /// Related configuration keys.
    pub related_config: Vec<String>,
}

impl DiagnosticIssue {
    /// Create a new diagnostic issue.
    pub fn new(
        code: impl Into<String>,
        severity: Severity,
        category: IssueCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            category,
            message: message.into(),
            details: None,
            fix: None,
            doc_url: None,
            related_config: Vec::new(),
        }
    }

    /// Add details to the issue.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add a suggested fix.
    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }

    /// Add a documentation URL.
    pub fn with_doc_url(mut self, url: impl Into<String>) -> Self {
        self.doc_url = Some(url.into());
        self
    }

    /// Add a related configuration key.
    pub fn with_config(mut self, config_key: impl Into<String>) -> Self {
        self.related_config.push(config_key.into());
        self
    }

    /// Format the issue for display.
    pub fn format(&self, colorize: bool) -> String {
        let severity_str = if colorize {
            match self.severity {
                Severity::Info => format!("\x1b[34m{}\x1b[0m", self.severity),
                Severity::Warning => format!("\x1b[33m{}\x1b[0m", self.severity),
                Severity::Error => format!("\x1b[31m{}\x1b[0m", self.severity),
                Severity::Critical => format!("\x1b[31;1m{}\x1b[0m", self.severity),
            }
        } else {
            self.severity.to_string()
        };

        let mut output = format!(
            "[{}] {} ({}): {}",
            severity_str, self.code, self.category, self.message
        );

        if let Some(ref details) = self.details {
            output.push_str(&format!("\n    Details: {}", details));
        }

        if let Some(ref fix) = self.fix {
            output.push_str(&format!("\n    Fix: {}", fix));
        }

        output
    }
}

/// A complete diagnostic report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// All issues found.
    issues: Vec<DiagnosticIssue>,
    /// Summary statistics.
    stats: DiagnosticStats,
    /// Timestamp of the report.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Target platform analyzed.
    pub target: Option<TargetPlatform>,
    /// Network mode analyzed.
    pub network_mode: Option<NetworkMode>,
}

/// Statistics about the diagnostic report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticStats {
    /// Count of issues by severity.
    pub by_severity: HashMap<String, usize>,
    /// Count of issues by category.
    pub by_category: HashMap<String, usize>,
    /// Total number of issues.
    pub total: usize,
}

impl DiagnosticReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            ..Default::default()
        }
    }

    /// Add an issue to the report.
    pub fn add_issue(&mut self, issue: DiagnosticIssue) {
        *self
            .stats
            .by_severity
            .entry(issue.severity.to_string())
            .or_insert(0) += 1;
        *self
            .stats
            .by_category
            .entry(issue.category.to_string())
            .or_insert(0) += 1;
        self.stats.total += 1;
        self.issues.push(issue);
    }

    /// Set the target platform.
    pub fn with_target(mut self, target: TargetPlatform) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the network mode.
    pub fn with_mode(mut self, mode: NetworkMode) -> Self {
        self.network_mode = Some(mode);
        self
    }

    /// Get all issues.
    pub fn issues(&self) -> &[DiagnosticIssue] {
        &self.issues
    }

    /// Get issues filtered by severity.
    pub fn issues_by_severity(&self, severity: Severity) -> Vec<&DiagnosticIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .collect()
    }

    /// Get issues filtered by category.
    pub fn issues_by_category(&self, category: IssueCategory) -> Vec<&DiagnosticIssue> {
        self.issues
            .iter()
            .filter(|i| i.category == category)
            .collect()
    }

    /// Check if the report has any critical or error issues.
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity >= Severity::Error)
    }

    /// Check if the report has any issues at all.
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Get a summary of the report.
    pub fn summary(&self) -> String {
        let status = if self.has_errors() {
            "FAILED"
        } else if self.has_issues() {
            "PASSED with warnings"
        } else {
            "PASSED"
        };

        let mut summary = format!("Network Diagnostics: {}\n", status);
        summary.push_str(&format!("  Total issues: {}\n", self.stats.total));

        if let Some(count) = self.stats.by_severity.get("CRITICAL") {
            summary.push_str(&format!("  Critical: {}\n", count));
        }
        if let Some(count) = self.stats.by_severity.get("ERROR") {
            summary.push_str(&format!("  Errors: {}\n", count));
        }
        if let Some(count) = self.stats.by_severity.get("WARN") {
            summary.push_str(&format!("  Warnings: {}\n", count));
        }
        if let Some(count) = self.stats.by_severity.get("INFO") {
            summary.push_str(&format!("  Info: {}\n", count));
        }

        summary
    }

    /// Format the full report.
    pub fn format(&self, colorize: bool) -> String {
        let mut output = self.summary();
        output.push('\n');

        if self.issues.is_empty() {
            output.push_str("No issues found. Configuration looks good!\n");
        } else {
            output.push_str("Issues:\n\n");
            for issue in &self.issues {
                output.push_str(&issue.format(colorize));
                output.push_str("\n\n");
            }
        }

        output
    }
}

/// Network doctor for diagnosing configuration issues.
#[derive(Debug, Default)]
pub struct NetworkDoctor {
    /// Whether to check for dev-specific issues.
    check_dev_issues: bool,
    /// Whether to check for production-specific issues.
    check_prod_issues: bool,
    /// Custom rules to apply.
    custom_rules: Vec<Box<dyn DiagnosticRule>>,
}

impl NetworkDoctor {
    /// Create a new network doctor.
    pub fn new() -> Self {
        Self {
            check_dev_issues: true,
            check_prod_issues: true,
            custom_rules: Vec::new(),
        }
    }

    /// Only check development issues.
    pub fn dev_only(mut self) -> Self {
        self.check_dev_issues = true;
        self.check_prod_issues = false;
        self
    }

    /// Only check production issues.
    pub fn prod_only(mut self) -> Self {
        self.check_dev_issues = false;
        self.check_prod_issues = true;
        self
    }

    /// Diagnose a network configuration.
    pub fn diagnose(&self, config: &NetworkConfig) -> DiagnosticReport {
        let mut report = DiagnosticReport::new()
            .with_target(config.target)
            .with_mode(config.mode);

        // Check network mode
        self.check_network_mode(config, &mut report);

        // Check for CORS issues
        self.check_cors_issues(config, &mut report);

        // Check proxy configuration
        self.check_proxy_issues(config, &mut report);

        // Check endpoints
        self.check_endpoints(config, &mut report);

        // Check security
        self.check_security(config, &mut report);

        // Apply custom rules
        for rule in &self.custom_rules {
            if let Some(issue) = rule.check(config) {
                report.add_issue(issue);
            }
        }

        report
    }

    /// Diagnose a proxy configuration.
    pub fn diagnose_proxy(&self, config: &ProxyConfig) -> DiagnosticReport {
        let mut report = DiagnosticReport::new();

        // Check for empty targets
        if config.targets.is_empty() {
            report.add_issue(
                DiagnosticIssue::new(
                    "PROXY001",
                    Severity::Error,
                    IssueCategory::Proxy,
                    "No proxy targets configured",
                )
                .with_fix("Add at least one proxy target: oxide dev --proxy api=http://localhost:8000"),
            );
        }

        // Check each target
        for target in &config.targets {
            // Check URL format
            if !target.target_url.starts_with("http://")
                && !target.target_url.starts_with("https://")
                && !target.target_url.starts_with("ws://")
                && !target.target_url.starts_with("wss://")
            {
                report.add_issue(
                    DiagnosticIssue::new(
                        "PROXY002",
                        Severity::Error,
                        IssueCategory::Proxy,
                        format!("Invalid URL for target '{}': {}", target.name, target.target_url),
                    )
                    .with_fix("URLs must start with http://, https://, ws://, or wss://"),
                );
            }

            // Check for localhost in production
            if !config.verify_ssl
                && (target.target_url.contains("localhost") || target.target_url.contains("127.0.0.1"))
            {
                report.add_issue(
                    DiagnosticIssue::new(
                        "PROXY003",
                        Severity::Warning,
                        IssueCategory::Proxy,
                        format!("Target '{}' uses localhost URL", target.name),
                    )
                    .with_details("This is fine for development but should not be used in production"),
                );
            }

            // Check WebSocket configuration
            if target.websocket
                && !target.target_url.starts_with("ws://")
                && !target.target_url.starts_with("wss://")
            {
                report.add_issue(
                    DiagnosticIssue::new(
                        "PROXY004",
                        Severity::Info,
                        IssueCategory::Proxy,
                        format!(
                            "WebSocket enabled for '{}' but URL uses HTTP scheme",
                            target.name
                        ),
                    )
                    .with_details("HTTP URLs will be upgraded to WebSocket automatically"),
                );
            }
        }

        // Check for duplicate path prefixes
        let mut seen_prefixes = std::collections::HashSet::new();
        for target in &config.targets {
            if !seen_prefixes.insert(&target.path_prefix) {
                report.add_issue(
                    DiagnosticIssue::new(
                        "PROXY005",
                        Severity::Error,
                        IssueCategory::Proxy,
                        format!("Duplicate path prefix: {}", target.path_prefix),
                    )
                    .with_fix("Each target must have a unique path prefix"),
                );
            }
        }

        report
    }

    /// Diagnose a CORS configuration.
    pub fn diagnose_cors(&self, config: &CorsConfig) -> DiagnosticReport {
        let diagnostic = crate::cors::CorsDiagnostic::analyze(config);

        let mut report = DiagnosticReport::new();

        for error in diagnostic.errors {
            report.add_issue(DiagnosticIssue::new(
                "CORS001",
                Severity::Error,
                IssueCategory::Cors,
                error,
            ));
        }

        for warning in diagnostic.warnings {
            report.add_issue(DiagnosticIssue::new(
                "CORS002",
                Severity::Warning,
                IssueCategory::Cors,
                warning,
            ));
        }

        for rec in diagnostic.recommendations {
            report.add_issue(DiagnosticIssue::new(
                "CORS003",
                Severity::Info,
                IssueCategory::Cors,
                rec,
            ));
        }

        report
    }

    fn check_network_mode(&self, config: &NetworkConfig, report: &mut DiagnosticReport) {
        // Check for unnecessary cross-origin mode
        if config.mode == NetworkMode::CrossOrigin {
            if config.target.supports_direct_api_access() {
                report.add_issue(
                    DiagnosticIssue::new(
                        "MODE001",
                        Severity::Warning,
                        IssueCategory::NetworkMode,
                        format!(
                            "Cross-origin mode is unnecessary for {} target",
                            config.target
                        ),
                    )
                    .with_fix("Use direct mode for desktop/mobile apps")
                    .with_config("network.mode"),
                );
            }
        }

        // Check for missing proxy in web dev mode
        if config.dev_mode && config.target.is_browser_based() && config.mode != NetworkMode::Proxied {
            if !config.endpoints.is_empty() {
                report.add_issue(
                    DiagnosticIssue::new(
                        "MODE002",
                        Severity::Warning,
                        IssueCategory::NetworkMode,
                        "Web/static target in dev mode without proxy may cause CORS errors",
                    )
                    .with_fix("Run with --proxy option: oxide dev --proxy api=http://localhost:8000")
                    .with_config("network.mode"),
                );
            }
        }
    }

    fn check_cors_issues(&self, config: &NetworkConfig, report: &mut DiagnosticReport) {
        // Check for allowed origins in cross-origin mode
        if config.mode == NetworkMode::CrossOrigin && config.allowed_origins.is_empty() {
            report.add_issue(
                DiagnosticIssue::new(
                    "CORS001",
                    Severity::Error,
                    IssueCategory::Cors,
                    "Cross-origin mode requires allowed origins to be configured",
                )
                .with_fix("Add allowed origins in oxide.toml: [network] allowed_origins = [\"...\"]")
                .with_config("network.allowed_origins"),
            );
        }

        // Check for wildcard origins
        if config.allowed_origins.contains(&"*".to_string()) {
            report.add_issue(
                DiagnosticIssue::new(
                    "CORS002",
                    Severity::Warning,
                    IssueCategory::Cors,
                    "Wildcard origin (*) allows requests from any domain",
                )
                .with_details("This is a security risk in production. Use explicit origins instead.")
                .with_config("network.allowed_origins"),
            );
        }

        // Recommend same-origin for browser targets
        if config.target.is_browser_based() && config.mode == NetworkMode::CrossOrigin {
            report.add_issue(
                DiagnosticIssue::new(
                    "CORS003",
                    Severity::Info,
                    IssueCategory::Cors,
                    "Consider using same-origin deployment to eliminate CORS",
                )
                .with_fix("Use reverse proxy: oxide network generate-proxy --target nginx"),
            );
        }
    }

    fn check_proxy_issues(&self, config: &NetworkConfig, report: &mut DiagnosticReport) {
        // Check for proxy in production
        if !config.dev_mode && config.mode == NetworkMode::Proxied {
            report.add_issue(
                DiagnosticIssue::new(
                    "PROXY001",
                    Severity::Error,
                    IssueCategory::Proxy,
                    "Proxied mode should not be used in production builds",
                )
                .with_details("The dev proxy is not included in production builds")
                .with_fix("Use same-origin deployment with a reverse proxy"),
            );
        }

        // Check endpoint URLs
        for endpoint in &config.endpoints {
            if !endpoint.url.starts_with("http://") && !endpoint.url.starts_with("https://") {
                report.add_issue(
                    DiagnosticIssue::new(
                        "PROXY002",
                        Severity::Error,
                        IssueCategory::Configuration,
                        format!("Invalid URL for endpoint '{}': {}", endpoint.name, endpoint.url),
                    )
                    .with_fix("URLs must start with http:// or https://"),
                );
            }
        }
    }

    fn check_endpoints(&self, config: &NetworkConfig, report: &mut DiagnosticReport) {
        // Check for empty endpoints in browser mode
        if config.target.is_browser_based() && config.endpoints.is_empty() && self.check_dev_issues {
            report.add_issue(
                DiagnosticIssue::new(
                    "ENDPOINT001",
                    Severity::Info,
                    IssueCategory::Configuration,
                    "No API endpoints configured for browser-based app",
                )
                .with_details("If your app makes API calls, configure endpoints for proper routing"),
            );
        }

        // Check for duplicate endpoint names
        let mut seen_names = std::collections::HashSet::new();
        for endpoint in &config.endpoints {
            if !seen_names.insert(&endpoint.name) {
                report.add_issue(
                    DiagnosticIssue::new(
                        "ENDPOINT002",
                        Severity::Error,
                        IssueCategory::Configuration,
                        format!("Duplicate endpoint name: {}", endpoint.name),
                    )
                    .with_fix("Each endpoint must have a unique name"),
                );
            }
        }
    }

    fn check_security(&self, config: &NetworkConfig, report: &mut DiagnosticReport) {
        // Check for HTTP in production origins
        if !config.dev_mode {
            if let Some(ref origin) = config.production_origin {
                if origin.starts_with("http://") && !origin.contains("localhost") {
                    report.add_issue(
                        DiagnosticIssue::new(
                            "SEC001",
                            Severity::Warning,
                            IssueCategory::Security,
                            "Production origin uses HTTP instead of HTTPS",
                        )
                        .with_fix("Use HTTPS for production: https://...")
                        .with_config("network.production_origin"),
                    );
                }
            }

            // Check endpoints for HTTP
            for endpoint in &config.endpoints {
                if endpoint.url.starts_with("http://")
                    && !endpoint.url.contains("localhost")
                    && !endpoint.url.contains("127.0.0.1")
                {
                    report.add_issue(
                        DiagnosticIssue::new(
                            "SEC002",
                            Severity::Warning,
                            IssueCategory::Security,
                            format!("Endpoint '{}' uses HTTP instead of HTTPS", endpoint.name),
                        )
                        .with_fix("Use HTTPS for production API endpoints"),
                    );
                }
            }
        }
    }
}

/// Trait for custom diagnostic rules.
pub trait DiagnosticRule: Send + Sync + std::fmt::Debug {
    /// Check the configuration and return an issue if found.
    fn check(&self, config: &NetworkConfig) -> Option<DiagnosticIssue>;
}

/// Quick check functions for common scenarios.
pub mod quick_checks {
    use super::*;

    /// Check if a web app is correctly configured to avoid CORS issues.
    pub fn check_web_app(dev_mode: bool, has_proxy: bool, has_api_calls: bool) -> Vec<String> {
        let mut issues = Vec::new();

        if dev_mode && has_api_calls && !has_proxy {
            issues.push(
                "Web app in dev mode with API calls but no proxy configured. \
                You will likely encounter CORS errors. \
                Run: oxide dev --proxy api=http://localhost:8000"
                    .to_string(),
            );
        }

        if !dev_mode && has_api_calls {
            issues.push(
                "Ensure your production deployment uses same-origin (reverse proxy) \
                or has proper CORS headers on the backend."
                    .to_string(),
            );
        }

        issues
    }

    /// Check if CORS configuration is safe for production.
    pub fn check_cors_production_ready(config: &CorsConfig) -> Vec<String> {
        let mut issues = Vec::new();

        if config.allow_all_origins {
            issues.push("Wildcard origin (*) is not recommended for production.".to_string());
        }

        if config.origins.iter().any(|o| o.contains("localhost")) {
            issues.push("Localhost origins found - remove for production.".to_string());
        }

        if config.allow_credentials && config.origins.len() > 10 {
            issues.push(
                "Many origins with credentials enabled. Consider using a reverse proxy instead."
                    .to_string(),
            );
        }

        issues
    }

    /// Get a quick recommendation for a target platform.
    pub fn get_recommendation(target: TargetPlatform, dev_mode: bool) -> String {
        match (target, dev_mode) {
            (TargetPlatform::Desktop, _) => {
                "Desktop apps don't need CORS configuration. Use direct HTTP to APIs.".to_string()
            }
            (TargetPlatform::Mobile, _) => {
                "Mobile apps don't need CORS configuration. Use direct HTTP to APIs.".to_string()
            }
            (TargetPlatform::Web, true) => {
                "Use dev proxy: oxide dev --proxy api=http://localhost:8000".to_string()
            }
            (TargetPlatform::Web, false) => {
                "Use reverse proxy for same-origin deployment: oxide network generate-proxy --target nginx"
                    .to_string()
            }
            (TargetPlatform::Static, true) => {
                "Use dev proxy: oxide dev --proxy api=http://localhost:8000".to_string()
            }
            (TargetPlatform::Static, false) => {
                "Configure reverse proxy or edge routing for same-origin deployment.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_issue_creation() {
        let issue = DiagnosticIssue::new(
            "TEST001",
            Severity::Warning,
            IssueCategory::Cors,
            "Test message",
        )
        .with_details("Test details")
        .with_fix("Test fix");

        assert_eq!(issue.code, "TEST001");
        assert_eq!(issue.severity, Severity::Warning);
        assert_eq!(issue.category, IssueCategory::Cors);
        assert!(issue.details.is_some());
        assert!(issue.fix.is_some());
    }

    #[test]
    fn test_diagnostic_report() {
        let mut report = DiagnosticReport::new();

        report.add_issue(DiagnosticIssue::new(
            "TEST001",
            Severity::Error,
            IssueCategory::Cors,
            "Error message",
        ));

        report.add_issue(DiagnosticIssue::new(
            "TEST002",
            Severity::Warning,
            IssueCategory::Proxy,
            "Warning message",
        ));

        assert_eq!(report.issues().len(), 2);
        assert!(report.has_errors());
        assert!(report.has_issues());
    }

    #[test]
    fn test_report_filtering() {
        let mut report = DiagnosticReport::new();

        report.add_issue(DiagnosticIssue::new(
            "TEST001",
            Severity::Error,
            IssueCategory::Cors,
            "Error",
        ));

        report.add_issue(DiagnosticIssue::new(
            "TEST002",
            Severity::Warning,
            IssueCategory::Cors,
            "Warning",
        ));

        report.add_issue(DiagnosticIssue::new(
            "TEST003",
            Severity::Warning,
            IssueCategory::Proxy,
            "Proxy warning",
        ));

        assert_eq!(report.issues_by_severity(Severity::Error).len(), 1);
        assert_eq!(report.issues_by_severity(Severity::Warning).len(), 2);
        assert_eq!(report.issues_by_category(IssueCategory::Cors).len(), 2);
        assert_eq!(report.issues_by_category(IssueCategory::Proxy).len(), 1);
    }

    #[test]
    fn test_network_doctor_desktop() {
        let config = NetworkConfig::for_target(TargetPlatform::Desktop);
        let doctor = NetworkDoctor::new();
        let report = doctor.diagnose(&config);

        // Desktop should have no CORS issues
        assert!(report.issues_by_category(IssueCategory::Cors).is_empty());
    }

    #[test]
    fn test_network_doctor_web_dev_no_proxy() {
        let mut config = NetworkConfig::for_target(TargetPlatform::Web);
        config.set_dev_mode(true);
        config.mode = NetworkMode::SameOrigin; // Not using proxy
        config.add_endpoint(crate::network_mode::ApiEndpoint::new(
            "api",
            "http://localhost:8000",
        ));

        let doctor = NetworkDoctor::new();
        let report = doctor.diagnose(&config);

        // Should warn about missing proxy
        assert!(report.has_issues());
    }

    #[test]
    fn test_network_doctor_cross_origin_no_origins() {
        let mut config = NetworkConfig::for_target(TargetPlatform::Web);
        config.mode = NetworkMode::CrossOrigin;

        let doctor = NetworkDoctor::new();
        let report = doctor.diagnose(&config);

        // Should error about missing allowed origins
        assert!(report.has_errors());
    }

    #[test]
    fn test_proxy_diagnosis() {
        let config = ProxyConfig::new()
            .add_target(crate::proxy::ProxyTarget::new("api", "invalid-url").with_path("/api"));

        let doctor = NetworkDoctor::new();
        let report = doctor.diagnose_proxy(&config);

        // Should error about invalid URL
        assert!(report.has_errors());
    }

    #[test]
    fn test_quick_checks() {
        let issues = quick_checks::check_web_app(true, false, true);
        assert!(!issues.is_empty());

        let issues = quick_checks::check_web_app(true, true, true);
        assert!(issues.is_empty());

        let recommendation = quick_checks::get_recommendation(TargetPlatform::Desktop, false);
        assert!(recommendation.contains("don't need CORS"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn test_report_summary() {
        let mut report = DiagnosticReport::new();
        report.add_issue(DiagnosticIssue::new(
            "TEST001",
            Severity::Error,
            IssueCategory::Cors,
            "Error",
        ));

        let summary = report.summary();
        assert!(summary.contains("FAILED"));
        assert!(summary.contains("Errors: 1"));
    }
}
