//! Design Token Governance
//!
//! Provides a system for locking and protecting design tokens at various levels.
//! This ensures brand consistency while allowing appropriate customization.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::brand_pack::BrandPack;
use crate::error::{BrandingError, BrandingResult, LockLevel};

/// Token governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenGovernance {
    /// Global governance settings
    #[serde(default)]
    pub settings: GovernanceSettings,

    /// Token locks by path
    #[serde(default)]
    pub locks: HashMap<String, TokenLock>,

    /// Governance rules
    #[serde(default)]
    pub rules: Vec<GovernanceRule>,

    /// Allowed override scopes
    #[serde(default)]
    pub allowed_scopes: HashMap<String, Vec<OverrideScope>>,
}

impl Default for TokenGovernance {
    fn default() -> Self {
        Self {
            settings: GovernanceSettings::default(),
            locks: HashMap::new(),
            rules: Vec::new(),
            allowed_scopes: HashMap::new(),
        }
    }
}

impl TokenGovernance {
    /// Create governance from a brand pack
    pub fn from_brand(brand: &BrandPack) -> Self {
        let mut governance = Self::default();

        // Apply brand's governance rules
        governance.rules = brand.governance.rules.clone();
        governance.settings = brand.governance.settings.clone();

        // Lock colors that are marked as locked
        if brand.colors.primary.locked {
            governance.lock_token("colors.primary", LockLevel::Brand);
        }
        if brand.colors.secondary.locked {
            governance.lock_token("colors.secondary", LockLevel::Brand);
        }
        if brand.colors.accent.locked {
            governance.lock_token("colors.accent", LockLevel::Brand);
        }

        for (name, color) in &brand.colors.custom {
            if color.locked {
                governance.lock_token(&format!("colors.{}", name), LockLevel::Brand);
            }
        }

        // Lock typography if specified
        if brand.typography.primary_family.locked {
            governance.lock_token("typography.primary_family", LockLevel::Brand);
        }

        governance
    }

    /// Lock a token at the specified level
    pub fn lock_token(&mut self, path: &str, level: LockLevel) {
        self.locks.insert(path.to_string(), TokenLock {
            level,
            reason: None,
            allowed_overrides: Vec::new(),
        });
    }

    /// Lock a token with a reason
    pub fn lock_token_with_reason(&mut self, path: &str, level: LockLevel, reason: impl Into<String>) {
        self.locks.insert(path.to_string(), TokenLock {
            level,
            reason: Some(reason.into()),
            allowed_overrides: Vec::new(),
        });
    }

    /// Check if a token can be overridden
    pub fn can_override(&self, token_path: &str) -> bool {
        // Check exact path
        if let Some(lock) = self.locks.get(token_path) {
            if lock.level != LockLevel::None {
                return false;
            }
        }

        // Check parent paths (e.g., "colors" locks all colors)
        for (locked_path, lock) in &self.locks {
            if token_path.starts_with(locked_path) && lock.level != LockLevel::None {
                // Check if there's a specific exception
                if !lock.allowed_overrides.contains(&token_path.to_string()) {
                    return false;
                }
            }
        }

        // Check governance rules
        for rule in &self.rules {
            if rule.matches(token_path) && !rule.allows_override() {
                return false;
            }
        }

        true
    }

    /// Get the lock for a token
    pub fn get_lock(&self, token_path: &str) -> Option<&TokenLock> {
        // Check exact match first
        if let Some(lock) = self.locks.get(token_path) {
            return Some(lock);
        }

        // Check parent paths
        for (locked_path, lock) in &self.locks {
            if token_path.starts_with(locked_path) {
                return Some(lock);
            }
        }

        None
    }

    /// Validate a set of overrides against governance rules
    pub fn validate_overrides(&self, overrides: &HashMap<String, serde_json::Value>) -> BrandingResult<()> {
        for (path, _value) in overrides {
            if !self.can_override(path) {
                let lock = self.get_lock(path);
                let level = lock.map(|l| l.level).unwrap_or(LockLevel::None);
                return Err(BrandingError::TokenLocked {
                    token: path.clone(),
                    level,
                });
            }
        }
        Ok(())
    }

    /// Get all locked token paths
    pub fn locked_tokens(&self) -> Vec<&str> {
        self.locks
            .iter()
            .filter(|(_, lock)| lock.level != LockLevel::None)
            .map(|(path, _)| path.as_str())
            .collect()
    }

    /// Get tokens locked at a specific level
    pub fn tokens_at_level(&self, level: LockLevel) -> Vec<&str> {
        self.locks
            .iter()
            .filter(|(_, lock)| lock.level == level)
            .map(|(path, _)| path.as_str())
            .collect()
    }

    /// Add a governance rule
    pub fn add_rule(&mut self, rule: GovernanceRule) {
        self.rules.push(rule);
    }

    /// Set allowed override scopes for a token
    pub fn set_allowed_scopes(&mut self, token_path: &str, scopes: Vec<OverrideScope>) {
        self.allowed_scopes.insert(token_path.to_string(), scopes);
    }
}

/// Token lock information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLock {
    /// Lock level
    pub level: LockLevel,

    /// Reason for the lock
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Specific paths that are allowed to override despite the lock
    #[serde(default)]
    pub allowed_overrides: Vec<String>,
}

impl Default for TokenLock {
    fn default() -> Self {
        Self {
            level: LockLevel::None,
            reason: None,
            allowed_overrides: Vec::new(),
        }
    }
}

impl Serialize for LockLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            LockLevel::Brand => "brand",
            LockLevel::Organization => "organization",
            LockLevel::App => "app",
            LockLevel::None => "none",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for LockLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "brand" => Ok(LockLevel::Brand),
            "organization" => Ok(LockLevel::Organization),
            "app" => Ok(LockLevel::App),
            "none" => Ok(LockLevel::None),
            _ => Err(serde::de::Error::custom(format!("Unknown lock level: {}", s))),
        }
    }
}

/// Governance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRule {
    /// Rule name
    pub name: String,

    /// Rule description
    #[serde(default)]
    pub description: String,

    /// Token paths this rule applies to (supports wildcards)
    pub patterns: Vec<String>,

    /// Rule type
    pub rule_type: RuleType,

    /// Rule action
    pub action: RuleAction,

    /// Conditions for the rule
    #[serde(default)]
    pub conditions: Vec<RuleCondition>,
}

impl GovernanceRule {
    /// Create a new lock rule
    pub fn lock(name: impl Into<String>, patterns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            patterns,
            rule_type: RuleType::Lock,
            action: RuleAction::Deny,
            conditions: Vec::new(),
        }
    }

    /// Create a new restrict rule
    pub fn restrict(name: impl Into<String>, patterns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            patterns,
            rule_type: RuleType::Restrict,
            action: RuleAction::Warn,
            conditions: Vec::new(),
        }
    }

    /// Check if this rule matches a token path
    pub fn matches(&self, token_path: &str) -> bool {
        for pattern in &self.patterns {
            if Self::pattern_matches(pattern, token_path) {
                return true;
            }
        }
        false
    }

    /// Check if this rule allows overrides
    pub fn allows_override(&self) -> bool {
        match (&self.rule_type, &self.action) {
            (RuleType::Lock, _) => false,
            (RuleType::Restrict, RuleAction::Deny) => false,
            _ => true,
        }
    }

    /// Simple pattern matching with * wildcard
    fn pattern_matches(pattern: &str, path: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            return path.starts_with(prefix);
        }

        if pattern.starts_with("*") {
            let suffix = &pattern[1..];
            return path.ends_with(suffix);
        }

        if pattern.contains("*") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }

        pattern == path
    }
}

/// Rule type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    /// Complete lock - no overrides allowed
    Lock,
    /// Restrict - overrides allowed with conditions
    Restrict,
    /// Require - token must be present
    Require,
    /// Validate - custom validation
    Validate,
}

/// Rule action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Warn but allow
    Warn,
    /// Require approval
    RequireApproval,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    /// Condition type
    pub condition_type: ConditionType,

    /// Condition value
    pub value: serde_json::Value,
}

/// Condition types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    /// Check environment
    Environment,
    /// Check user role
    Role,
    /// Check date/time
    DateTime,
    /// Check feature flag
    FeatureFlag,
    /// Custom condition
    Custom,
}

/// Override scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverrideScope {
    /// Theme-level overrides
    Theme,
    /// Component-level overrides
    Component,
    /// Page/view-level overrides
    Page,
    /// Instance-level overrides
    Instance,
}

/// Governance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSettings {
    /// Whether governance is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Strict mode - deny all unlisted overrides
    #[serde(default)]
    pub strict_mode: bool,

    /// Log override attempts
    #[serde(default = "default_true")]
    pub log_overrides: bool,

    /// Require approval for certain changes
    #[serde(default)]
    pub require_approval: bool,

    /// Default lock level for new tokens
    #[serde(default)]
    pub default_lock_level: LockLevel,

    /// Tokens that are always unlocked
    #[serde(default)]
    pub always_unlocked: HashSet<String>,

    /// Tokens that are always locked
    #[serde(default)]
    pub always_locked: HashSet<String>,
}

fn default_true() -> bool {
    true
}

impl Default for GovernanceSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            strict_mode: false,
            log_overrides: true,
            require_approval: false,
            default_lock_level: LockLevel::None,
            always_unlocked: HashSet::new(),
            always_locked: HashSet::new(),
        }
    }
}

/// Governance rules collection from brand pack
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GovernanceRules {
    /// General settings
    #[serde(default)]
    pub settings: GovernanceSettings,

    /// Governance rules
    #[serde(default)]
    pub rules: Vec<GovernanceRule>,
}

/// Builder for creating governance rules
#[derive(Debug, Default)]
pub struct GovernanceBuilder {
    governance: TokenGovernance,
}

impl GovernanceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lock brand colors
    pub fn lock_brand_colors(mut self) -> Self {
        self.governance.lock_token_with_reason(
            "colors.primary",
            LockLevel::Brand,
            "Primary brand color is protected",
        );
        self.governance.lock_token_with_reason(
            "colors.secondary",
            LockLevel::Brand,
            "Secondary brand color is protected",
        );
        self
    }

    /// Lock all colors
    pub fn lock_all_colors(mut self) -> Self {
        self.governance.lock_token_with_reason(
            "colors",
            LockLevel::Brand,
            "All colors are protected by brand guidelines",
        );
        self
    }

    /// Lock typography
    pub fn lock_typography(mut self) -> Self {
        self.governance.lock_token_with_reason(
            "typography",
            LockLevel::Brand,
            "Typography is protected by brand guidelines",
        );
        self
    }

    /// Lock a specific token
    pub fn lock(mut self, path: &str, level: LockLevel, reason: &str) -> Self {
        self.governance.lock_token_with_reason(path, level, reason);
        self
    }

    /// Add a rule
    pub fn rule(mut self, rule: GovernanceRule) -> Self {
        self.governance.add_rule(rule);
        self
    }

    /// Enable strict mode
    pub fn strict(mut self) -> Self {
        self.governance.settings.strict_mode = true;
        self
    }

    /// Build the governance
    pub fn build(self) -> TokenGovernance {
        self.governance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_lock() {
        let mut governance = TokenGovernance::default();
        governance.lock_token("colors.primary", LockLevel::Brand);

        assert!(!governance.can_override("colors.primary"));
        assert!(governance.can_override("colors.secondary"));
    }

    #[test]
    fn test_parent_lock() {
        let mut governance = TokenGovernance::default();
        governance.lock_token("colors", LockLevel::Brand);

        assert!(!governance.can_override("colors.primary"));
        assert!(!governance.can_override("colors.secondary"));
        assert!(!governance.can_override("colors.custom.brand-blue"));
        assert!(governance.can_override("typography.primary"));
    }

    #[test]
    fn test_governance_rule_matching() {
        let rule = GovernanceRule::lock("lock-colors", vec!["colors.*".into()]);

        assert!(rule.matches("colors.primary"));
        assert!(rule.matches("colors.custom.blue"));
        assert!(!rule.matches("typography.primary"));
    }

    #[test]
    fn test_governance_builder() {
        let governance = GovernanceBuilder::new()
            .lock_brand_colors()
            .lock("logo.primary", LockLevel::Brand, "Primary logo is protected")
            .strict()
            .build();

        assert!(!governance.can_override("colors.primary"));
        assert!(!governance.can_override("logo.primary"));
        assert!(governance.settings.strict_mode);
    }

    #[test]
    fn test_validate_overrides() {
        let mut governance = TokenGovernance::default();
        governance.lock_token("colors.primary", LockLevel::Brand);

        let mut overrides = HashMap::new();
        overrides.insert("colors.secondary".into(), serde_json::json!("#FF0000"));
        assert!(governance.validate_overrides(&overrides).is_ok());

        overrides.insert("colors.primary".into(), serde_json::json!("#00FF00"));
        assert!(governance.validate_overrides(&overrides).is_err());
    }

    #[test]
    fn test_allowed_overrides() {
        let mut governance = TokenGovernance::default();
        governance.locks.insert("colors".into(), TokenLock {
            level: LockLevel::Brand,
            reason: Some("Colors locked".into()),
            allowed_overrides: vec!["colors.accent".into()],
        });

        assert!(!governance.can_override("colors.primary"));
        assert!(governance.can_override("colors.accent"));
    }
}
