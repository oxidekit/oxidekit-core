//! Compatibility Policy Management
//!
//! Controls which compatibility features are allowed in an OxideKit project.
//! Policies are defined in `oxide.toml` under the `[policy]` section.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during policy operations
#[derive(Error, Debug)]
pub enum PolicyError {
    /// Failed to read config file
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse config
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Policy violation
    #[error("Policy violation: {0}")]
    Violation(PolicyViolation),
}

/// Represents a policy violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    /// The feature that was violated
    pub feature: String,
    /// Human-readable description of the violation
    pub message: String,
    /// Severity level
    pub severity: ViolationSeverity,
    /// Suggestions for resolution
    pub suggestions: Vec<String>,
}

impl std::fmt::Display for PolicyViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.feature, self.message)
    }
}

/// Severity of a policy violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViolationSeverity {
    /// Warning - can proceed but should address
    Warning,
    /// Error - cannot proceed in release builds
    Error,
    /// Critical - cannot proceed at all
    Critical,
}

impl std::fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationSeverity::Warning => write!(f, "WARNING"),
            ViolationSeverity::Error => write!(f, "ERROR"),
            ViolationSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Compatibility policy configuration
///
/// Defines which compatibility features are allowed in the project.
/// By default, all compatibility features are disabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CompatPolicy {
    /// Allow WebView embedding (NOT RECOMMENDED)
    pub allow_webview: bool,

    /// Allow JavaScript runtime (NOT RECOMMENDED)
    pub allow_js_runtime: bool,

    /// Allow NPM package bundling at build time
    pub allow_npm_bundling: bool,

    /// Allow remote content in WebView (DANGEROUS)
    pub allow_remote_webview: bool,

    /// Allowed remote origins for WebView (if allow_remote_webview is true)
    pub allowed_origins: Vec<String>,

    /// Allow eval() in JavaScript runtime (DANGEROUS)
    pub allow_js_eval: bool,

    /// Maximum memory for JS runtime in MB
    pub js_memory_limit_mb: u32,

    /// Maximum execution time for JS in milliseconds
    pub js_timeout_ms: u32,

    /// Allow devtools in release builds (NOT RECOMMENDED)
    pub allow_devtools_in_release: bool,

    /// Strict mode - fail on any policy warning
    pub strict_mode: bool,

    /// Custom CSP (Content Security Policy) for WebView
    pub webview_csp: Option<String>,
}

impl Default for CompatPolicy {
    fn default() -> Self {
        Self {
            // All compatibility features disabled by default
            allow_webview: false,
            allow_js_runtime: false,
            allow_npm_bundling: false,
            allow_remote_webview: false,
            allowed_origins: Vec::new(),
            allow_js_eval: false,
            js_memory_limit_mb: 64,
            js_timeout_ms: 5000,
            allow_devtools_in_release: false,
            strict_mode: false,
            // Strict default CSP
            webview_csp: Some(
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'none'".to_string()
            ),
        }
    }
}

impl CompatPolicy {
    /// Create a new policy with all features disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a policy allowing all features (DANGEROUS - only for development)
    pub fn allow_all() -> Self {
        Self {
            allow_webview: true,
            allow_js_runtime: true,
            allow_npm_bundling: true,
            allow_remote_webview: true,
            allowed_origins: vec!["*".to_string()],
            allow_js_eval: true,
            js_memory_limit_mb: 256,
            js_timeout_ms: 30000,
            allow_devtools_in_release: true,
            strict_mode: false,
            webview_csp: None,
        }
    }

    /// Load policy from oxide.toml configuration file
    pub fn from_config<P: AsRef<Path>>(path: P) -> Result<Self, PolicyError> {
        let content = std::fs::read_to_string(path)?;
        let config: ConfigFile = toml::from_str(&content)?;
        Ok(config.policy.unwrap_or_default())
    }

    /// Load policy from oxide.toml in current directory
    pub fn from_current_dir() -> Result<Self, PolicyError> {
        Self::from_config("oxide.toml")
    }

    /// Check if a feature is allowed and return violations
    pub fn check_feature(&self, feature: CompatFeature) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();

        match feature {
            CompatFeature::WebView => {
                if !self.allow_webview {
                    violations.push(PolicyViolation {
                        feature: "webview".to_string(),
                        message: "WebView embedding is not allowed by policy".to_string(),
                        severity: ViolationSeverity::Error,
                        suggestions: vec![
                            "Add `allow_webview = true` to [policy] in oxide.toml".to_string(),
                            "Consider using native OxideKit components instead".to_string(),
                        ],
                    });
                }
            }
            CompatFeature::RemoteWebView => {
                if !self.allow_remote_webview {
                    violations.push(PolicyViolation {
                        feature: "remote_webview".to_string(),
                        message: "Remote content in WebView is not allowed".to_string(),
                        severity: ViolationSeverity::Critical,
                        suggestions: vec![
                            "Add `allow_remote_webview = true` to [policy] in oxide.toml".to_string(),
                            "Bundle web assets locally instead".to_string(),
                        ],
                    });
                }
            }
            CompatFeature::JsRuntime => {
                if !self.allow_js_runtime {
                    violations.push(PolicyViolation {
                        feature: "js_runtime".to_string(),
                        message: "JavaScript runtime is not allowed by policy".to_string(),
                        severity: ViolationSeverity::Error,
                        suggestions: vec![
                            "Add `allow_js_runtime = true` to [policy] in oxide.toml".to_string(),
                            "Consider porting JS logic to Rust".to_string(),
                        ],
                    });
                }
            }
            CompatFeature::JsEval => {
                if !self.allow_js_eval {
                    violations.push(PolicyViolation {
                        feature: "js_eval".to_string(),
                        message: "eval() is disabled in JavaScript runtime".to_string(),
                        severity: ViolationSeverity::Critical,
                        suggestions: vec![
                            "Add `allow_js_eval = true` to [policy] in oxide.toml (DANGEROUS)".to_string(),
                            "Refactor code to avoid eval()".to_string(),
                        ],
                    });
                }
            }
            CompatFeature::NpmBundling => {
                if !self.allow_npm_bundling {
                    violations.push(PolicyViolation {
                        feature: "npm_bundling".to_string(),
                        message: "NPM bundling is not allowed by policy".to_string(),
                        severity: ViolationSeverity::Warning,
                        suggestions: vec![
                            "Add `allow_npm_bundling = true` to [policy] in oxide.toml".to_string(),
                        ],
                    });
                }
            }
            CompatFeature::DevtoolsInRelease => {
                if !self.allow_devtools_in_release {
                    violations.push(PolicyViolation {
                        feature: "devtools".to_string(),
                        message: "Devtools are disabled in release builds".to_string(),
                        severity: ViolationSeverity::Warning,
                        suggestions: vec![
                            "Use debug builds for development".to_string(),
                            "Add `allow_devtools_in_release = true` to [policy]".to_string(),
                        ],
                    });
                }
            }
        }

        violations
    }

    /// Check if an origin is allowed for remote WebView content
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        if !self.allow_remote_webview {
            return false;
        }

        if self.allowed_origins.contains(&"*".to_string()) {
            return true;
        }

        self.allowed_origins.iter().any(|allowed| {
            if allowed.starts_with("*.") {
                // Wildcard subdomain match
                let domain = &allowed[2..];
                origin.ends_with(domain) || origin == domain
            } else {
                origin == allowed
            }
        })
    }

    /// Get the effective CSP for WebView
    pub fn effective_csp(&self) -> &str {
        self.webview_csp.as_deref().unwrap_or(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'none'"
        )
    }

    /// Validate the entire policy for consistency
    pub fn validate(&self) -> Vec<PolicyViolation> {
        let mut violations = Vec::new();

        // Warn if remote webview is allowed without specific origins
        if self.allow_remote_webview && self.allowed_origins.is_empty() {
            violations.push(PolicyViolation {
                feature: "remote_webview".to_string(),
                message: "Remote WebView allowed but no origins specified".to_string(),
                severity: ViolationSeverity::Warning,
                suggestions: vec![
                    "Add specific origins to `allowed_origins`".to_string(),
                ],
            });
        }

        // Warn if wildcard origin is used
        if self.allowed_origins.contains(&"*".to_string()) {
            violations.push(PolicyViolation {
                feature: "remote_webview".to_string(),
                message: "Wildcard origin (*) allows any remote content".to_string(),
                severity: ViolationSeverity::Warning,
                suggestions: vec![
                    "Specify exact origins instead of wildcard".to_string(),
                ],
            });
        }

        // Warn if eval is enabled
        if self.allow_js_eval {
            violations.push(PolicyViolation {
                feature: "js_eval".to_string(),
                message: "eval() is enabled which can execute arbitrary code".to_string(),
                severity: ViolationSeverity::Warning,
                suggestions: vec![
                    "Disable eval() if not strictly necessary".to_string(),
                ],
            });
        }

        // Warn if no CSP is set
        if self.allow_webview && self.webview_csp.is_none() {
            violations.push(PolicyViolation {
                feature: "webview".to_string(),
                message: "No Content Security Policy configured for WebView".to_string(),
                severity: ViolationSeverity::Warning,
                suggestions: vec![
                    "Add a webview_csp to restrict script execution".to_string(),
                ],
            });
        }

        violations
    }

    /// Export policy to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

/// Compatibility features that can be checked against policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatFeature {
    /// WebView embedding
    WebView,
    /// Remote content in WebView
    RemoteWebView,
    /// JavaScript runtime
    JsRuntime,
    /// eval() in JavaScript
    JsEval,
    /// NPM bundling
    NpmBundling,
    /// Devtools in release builds
    DevtoolsInRelease,
}

/// Partial config file structure for parsing policy
#[derive(Debug, Deserialize)]
struct ConfigFile {
    policy: Option<CompatPolicy>,
}

/// Policy enforcement context
pub struct PolicyEnforcer {
    policy: CompatPolicy,
    is_release: bool,
    violations: Vec<PolicyViolation>,
}

impl PolicyEnforcer {
    /// Create a new policy enforcer
    pub fn new(policy: CompatPolicy, is_release: bool) -> Self {
        Self {
            policy,
            is_release,
            violations: Vec::new(),
        }
    }

    /// Enforce a feature check, collecting violations
    pub fn enforce(&mut self, feature: CompatFeature) -> bool {
        let feature_violations = self.policy.check_feature(feature);
        let allowed = feature_violations.is_empty();
        self.violations.extend(feature_violations);
        allowed
    }

    /// Get all collected violations
    pub fn violations(&self) -> &[PolicyViolation] {
        &self.violations
    }

    /// Check if any critical violations occurred
    pub fn has_critical_violations(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Critical)
    }

    /// Check if any error violations occurred
    pub fn has_error_violations(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Error)
    }

    /// Check if should fail based on policy and build type
    pub fn should_fail(&self) -> bool {
        if self.policy.strict_mode {
            return !self.violations.is_empty();
        }

        if self.is_release {
            self.has_error_violations() || self.has_critical_violations()
        } else {
            self.has_critical_violations()
        }
    }

    /// Print violations to stderr
    pub fn print_violations(&self) {
        for violation in &self.violations {
            eprintln!("{}", violation);
            for suggestion in &violation.suggestions {
                eprintln!("  -> {}", suggestion);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy_blocks_all() {
        let policy = CompatPolicy::default();
        assert!(!policy.allow_webview);
        assert!(!policy.allow_js_runtime);
        assert!(!policy.allow_remote_webview);
        assert!(!policy.allow_js_eval);
    }

    #[test]
    fn test_check_feature_webview() {
        let policy = CompatPolicy::default();
        let violations = policy.check_feature(CompatFeature::WebView);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].feature, "webview");
    }

    #[test]
    fn test_allowed_policy() {
        let mut policy = CompatPolicy::default();
        policy.allow_webview = true;
        let violations = policy.check_feature(CompatFeature::WebView);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_origin_allowed() {
        let mut policy = CompatPolicy::default();
        policy.allow_remote_webview = true;
        policy.allowed_origins = vec!["example.com".to_string(), "*.trusted.com".to_string()];

        assert!(policy.is_origin_allowed("example.com"));
        assert!(policy.is_origin_allowed("sub.trusted.com"));
        assert!(policy.is_origin_allowed("trusted.com"));
        assert!(!policy.is_origin_allowed("evil.com"));
    }

    #[test]
    fn test_wildcard_origin() {
        let mut policy = CompatPolicy::default();
        policy.allow_remote_webview = true;
        policy.allowed_origins = vec!["*".to_string()];

        assert!(policy.is_origin_allowed("any.domain.com"));
    }

    #[test]
    fn test_policy_validation() {
        let mut policy = CompatPolicy::default();
        policy.allow_remote_webview = true;
        // No origins specified

        let violations = policy.validate();
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_enforcer() {
        let policy = CompatPolicy::default();
        let mut enforcer = PolicyEnforcer::new(policy, true);

        assert!(!enforcer.enforce(CompatFeature::WebView));
        assert!(enforcer.has_error_violations());
        assert!(enforcer.should_fail());
    }
}
