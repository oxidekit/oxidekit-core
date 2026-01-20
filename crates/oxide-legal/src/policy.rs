//! License policy enforcement
//!
//! Define and enforce license policies for projects.

use std::collections::HashSet;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::error::{LegalError, LegalResult};
use crate::license::{License, LicenseCategory, LicenseType};
use crate::scanner::ScanResult;

/// License policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePolicy {
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: String,
    /// Allowed license categories
    pub allowed_categories: Vec<LicenseCategory>,
    /// Explicitly allowed licenses (SPDX IDs)
    pub allowed_licenses: HashSet<String>,
    /// Explicitly denied licenses (SPDX IDs)
    pub denied_licenses: HashSet<String>,
    /// Allowed packages with otherwise denied licenses
    pub exceptions: Vec<PolicyException>,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
    /// Require OSI-approved licenses
    pub require_osi_approved: bool,
    /// Action when policy is violated
    pub violation_action: PolicyAction,
}

impl Default for LicensePolicy {
    fn default() -> Self {
        Self::permissive()
    }
}

impl LicensePolicy {
    /// Create a permissive policy (MIT, Apache, BSD, etc.)
    pub fn permissive() -> Self {
        Self {
            name: "Permissive".to_string(),
            description: "Only allow permissive open source licenses".to_string(),
            allowed_categories: vec![
                LicenseCategory::PublicDomain,
                LicenseCategory::Permissive,
            ],
            allowed_licenses: HashSet::from([
                "MIT".to_string(),
                "MIT-0".to_string(),
                "Apache-2.0".to_string(),
                "BSD-2-Clause".to_string(),
                "BSD-3-Clause".to_string(),
                "ISC".to_string(),
                "Unlicense".to_string(),
                "CC0-1.0".to_string(),
                "0BSD".to_string(),
                "Zlib".to_string(),
                "BSL-1.0".to_string(),
            ]),
            denied_licenses: HashSet::from([
                "GPL-2.0".to_string(),
                "GPL-3.0".to_string(),
                "AGPL-3.0".to_string(),
                "Proprietary".to_string(),
            ]),
            exceptions: vec![],
            rules: vec![
                PolicyRule {
                    name: "No copyleft".to_string(),
                    description: "Copyleft licenses not allowed".to_string(),
                    condition: RuleCondition::CategoryDenied(LicenseCategory::StrongCopyleft),
                    action: PolicyAction::Deny,
                },
                PolicyRule {
                    name: "No AGPL".to_string(),
                    description: "Network copyleft not allowed".to_string(),
                    condition: RuleCondition::CategoryDenied(LicenseCategory::NetworkCopyleft),
                    action: PolicyAction::Deny,
                },
            ],
            require_osi_approved: true,
            violation_action: PolicyAction::Deny,
        }
    }

    /// Create a copyleft-friendly policy
    pub fn copyleft_friendly() -> Self {
        Self {
            name: "Copyleft Friendly".to_string(),
            description: "Allow copyleft licenses (GPL-compatible project)".to_string(),
            allowed_categories: vec![
                LicenseCategory::PublicDomain,
                LicenseCategory::Permissive,
                LicenseCategory::WeakCopyleft,
                LicenseCategory::StrongCopyleft,
            ],
            allowed_licenses: HashSet::new(),
            denied_licenses: HashSet::from([
                "AGPL-3.0".to_string(),
                "Proprietary".to_string(),
            ]),
            exceptions: vec![],
            rules: vec![
                PolicyRule {
                    name: "No AGPL".to_string(),
                    description: "Network copyleft not allowed".to_string(),
                    condition: RuleCondition::CategoryDenied(LicenseCategory::NetworkCopyleft),
                    action: PolicyAction::Deny,
                },
            ],
            require_osi_approved: false,
            violation_action: PolicyAction::Deny,
        }
    }

    /// Create a strict commercial policy
    pub fn commercial() -> Self {
        Self {
            name: "Commercial".to_string(),
            description: "Strict policy for commercial projects".to_string(),
            allowed_categories: vec![
                LicenseCategory::PublicDomain,
                LicenseCategory::Permissive,
            ],
            allowed_licenses: HashSet::from([
                "MIT".to_string(),
                "Apache-2.0".to_string(),
                "BSD-2-Clause".to_string(),
                "BSD-3-Clause".to_string(),
                "ISC".to_string(),
                "Zlib".to_string(),
            ]),
            denied_licenses: HashSet::new(),
            exceptions: vec![],
            rules: vec![
                PolicyRule {
                    name: "No weak copyleft".to_string(),
                    description: "Even weak copyleft requires review".to_string(),
                    condition: RuleCondition::CategoryDenied(LicenseCategory::WeakCopyleft),
                    action: PolicyAction::Warn,
                },
                PolicyRule {
                    name: "No unknown".to_string(),
                    description: "Unknown licenses not allowed".to_string(),
                    condition: RuleCondition::CategoryDenied(LicenseCategory::Unknown),
                    action: PolicyAction::Deny,
                },
            ],
            require_osi_approved: true,
            violation_action: PolicyAction::Deny,
        }
    }

    /// Load policy from a TOML file
    pub fn from_file(path: impl AsRef<Path>) -> LegalResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Save policy to a TOML file
    pub fn to_file(&self, path: impl AsRef<Path>) -> LegalResult<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add an exception for a specific package
    pub fn add_exception(&mut self, package: &str, reason: &str) {
        self.exceptions.push(PolicyException {
            package: package.to_string(),
            reason: reason.to_string(),
            approved_by: None,
            approved_at: None,
        });
    }

    /// Check if a license is allowed by this policy
    pub fn is_license_allowed(&self, license: &License, package: &str) -> bool {
        // Check exceptions first
        if self.exceptions.iter().any(|e| e.package == package) {
            return true;
        }

        // Check explicitly denied
        if self.denied_licenses.contains(&license.spdx_id) {
            return false;
        }

        // Check explicitly allowed
        if self.allowed_licenses.contains(&license.spdx_id) {
            return true;
        }

        // Check OSI requirement
        if self.require_osi_approved && !license.osi_approved {
            return false;
        }

        // Check category
        self.allowed_categories.contains(&license.category)
    }

    /// Validate a scan result against this policy
    pub fn validate(&self, scan: &ScanResult) -> PolicyValidationResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        for dep in &scan.dependencies {
            // Check rules first
            for rule in &self.rules {
                if rule.matches(&dep.license) {
                    let issue = PolicyIssue {
                        package: dep.name.clone(),
                        version: dep.version.clone(),
                        license: dep.license.spdx_id.clone(),
                        rule: rule.name.clone(),
                        message: rule.description.clone(),
                        action: rule.action.clone(),
                    };

                    match rule.action {
                        PolicyAction::Deny => violations.push(issue),
                        PolicyAction::Warn => warnings.push(issue),
                        PolicyAction::Allow => {}
                    }
                }
            }

            // General policy check
            if !self.is_license_allowed(&dep.license, &dep.name) {
                let issue = PolicyIssue {
                    package: dep.name.clone(),
                    version: dep.version.clone(),
                    license: dep.license.spdx_id.clone(),
                    rule: "License Policy".to_string(),
                    message: format!(
                        "License {} is not allowed by policy '{}'",
                        dep.license.spdx_id, self.name
                    ),
                    action: self.violation_action.clone(),
                };

                match self.violation_action {
                    PolicyAction::Deny => violations.push(issue),
                    PolicyAction::Warn => warnings.push(issue),
                    PolicyAction::Allow => {}
                }
            }
        }

        // Deduplicate violations
        violations.sort_by(|a, b| a.package.cmp(&b.package));
        violations.dedup_by(|a, b| a.package == b.package && a.license == b.license);

        warnings.sort_by(|a, b| a.package.cmp(&b.package));
        warnings.dedup_by(|a, b| a.package == b.package && a.license == b.license);

        PolicyValidationResult {
            policy_name: self.name.clone(),
            passed: violations.is_empty(),
            violations,
            warnings,
            total_checked: scan.dependencies.len(),
        }
    }
}

/// A policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Condition that triggers the rule
    pub condition: RuleCondition,
    /// Action to take when rule matches
    pub action: PolicyAction,
}

impl PolicyRule {
    /// Check if this rule matches a license
    pub fn matches(&self, license: &License) -> bool {
        self.condition.matches(license)
    }
}

/// Condition for a policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleCondition {
    /// Deny a specific license category
    CategoryDenied(LicenseCategory),
    /// Deny a specific license type
    TypeDenied(LicenseType),
    /// Deny a specific SPDX ID
    SpdxDenied(String),
    /// Require OSI approval
    RequireOsiApproved,
    /// Require FSF free
    RequireFsfFree,
    /// Any condition matches
    Any(Vec<RuleCondition>),
    /// All conditions match
    All(Vec<RuleCondition>),
}

impl RuleCondition {
    /// Check if the condition matches a license
    pub fn matches(&self, license: &License) -> bool {
        match self {
            RuleCondition::CategoryDenied(cat) => &license.category == cat,
            RuleCondition::TypeDenied(lt) => &license.license_type == lt,
            RuleCondition::SpdxDenied(spdx) => license.spdx_id.eq_ignore_ascii_case(spdx),
            RuleCondition::RequireOsiApproved => !license.osi_approved,
            RuleCondition::RequireFsfFree => !license.fsf_free,
            RuleCondition::Any(conditions) => conditions.iter().any(|c| c.matches(license)),
            RuleCondition::All(conditions) => conditions.iter().all(|c| c.matches(license)),
        }
    }
}

/// Action to take when a policy rule matches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    /// Allow the license
    Allow,
    /// Warn but allow
    Warn,
    /// Deny the license
    Deny,
}

/// Exception to policy for a specific package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyException {
    /// Package name
    pub package: String,
    /// Reason for exception
    pub reason: String,
    /// Who approved the exception
    pub approved_by: Option<String>,
    /// When the exception was approved
    pub approved_at: Option<String>,
}

/// Result of policy validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyValidationResult {
    /// Policy name
    pub policy_name: String,
    /// Whether validation passed
    pub passed: bool,
    /// Policy violations
    pub violations: Vec<PolicyIssue>,
    /// Policy warnings
    pub warnings: Vec<PolicyIssue>,
    /// Total packages checked
    pub total_checked: usize,
}

impl PolicyValidationResult {
    /// Convert to a LegalError if there are violations
    pub fn into_result(self) -> LegalResult<Self> {
        if self.passed {
            Ok(self)
        } else {
            Err(LegalError::MultiplePolicyViolations {
                count: self.violations.len(),
            })
        }
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        if self.passed {
            format!(
                "Policy '{}' passed: {} packages checked, {} warnings",
                self.policy_name,
                self.total_checked,
                self.warnings.len()
            )
        } else {
            format!(
                "Policy '{}' FAILED: {} violations, {} warnings ({} packages)",
                self.policy_name,
                self.violations.len(),
                self.warnings.len(),
                self.total_checked
            )
        }
    }
}

/// A policy issue (violation or warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyIssue {
    /// Package name
    pub package: String,
    /// Package version
    pub version: String,
    /// License identifier
    pub license: String,
    /// Rule that was violated
    pub rule: String,
    /// Issue message
    pub message: String,
    /// Action associated with this issue
    pub action: PolicyAction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissive_policy() {
        let policy = LicensePolicy::permissive();
        let mit = License::parse("MIT").unwrap();
        let gpl = License::parse("GPL-3.0").unwrap();

        assert!(policy.is_license_allowed(&mit, "test-package"));
        assert!(!policy.is_license_allowed(&gpl, "test-package"));
    }

    #[test]
    fn test_policy_exception() {
        let mut policy = LicensePolicy::permissive();
        let gpl = License::parse("GPL-3.0").unwrap();

        assert!(!policy.is_license_allowed(&gpl, "gpl-package"));

        policy.add_exception("gpl-package", "Legacy dependency");
        assert!(policy.is_license_allowed(&gpl, "gpl-package"));
    }

    #[test]
    fn test_commercial_policy() {
        let policy = LicensePolicy::commercial();
        let mit = License::parse("MIT").unwrap();
        let unknown = License::parse("CustomLicense").unwrap();

        assert!(policy.is_license_allowed(&mit, "test"));
        assert!(!policy.is_license_allowed(&unknown, "test"));
    }
}
