//! Lint rule definitions
//!
//! Defines all available lint rules for .oui files.

use super::{LintRule, LintSeverity, RuleCategory};
use crate::ErrorCode;

/// Get all available lint rules
pub fn all_rules() -> Vec<LintRule> {
    vec![
        // Correctness rules (L0xxx)
        LintRule {
            id: "parse-error".to_string(),
            description: "File failed to parse".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(1),
            enabled_by_default: true,
            category: RuleCategory::Correctness,
        },
        LintRule {
            id: "unknown-component".to_string(),
            description: "Component type is not registered".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(2),
            enabled_by_default: true,
            category: RuleCategory::Correctness,
        },
        LintRule {
            id: "unknown-prop".to_string(),
            description: "Property is not defined for this component".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(3),
            enabled_by_default: true,
            category: RuleCategory::Correctness,
        },
        LintRule {
            id: "invalid-prop-type".to_string(),
            description: "Property value has incorrect type".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(4),
            enabled_by_default: true,
            category: RuleCategory::Correctness,
        },
        LintRule {
            id: "missing-required-prop".to_string(),
            description: "Required property is missing".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(5),
            enabled_by_default: true,
            category: RuleCategory::Correctness,
        },
        LintRule {
            id: "duplicate-prop".to_string(),
            description: "Property is defined multiple times".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(6),
            enabled_by_default: true,
            category: RuleCategory::Correctness,
        },
        LintRule {
            id: "invalid-child".to_string(),
            description: "Component does not accept this child type".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(7),
            enabled_by_default: true,
            category: RuleCategory::Correctness,
        },

        // Style rules (L01xx)
        LintRule {
            id: "no-hardcoded-colors".to_string(),
            description: "Avoid hardcoded color values, use design tokens".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(100),
            enabled_by_default: true,
            category: RuleCategory::Style,
        },
        LintRule {
            id: "no-hardcoded-spacing".to_string(),
            description: "Avoid hardcoded spacing values, use design tokens".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(101),
            enabled_by_default: true,
            category: RuleCategory::Style,
        },
        LintRule {
            id: "no-inline-styles".to_string(),
            description: "Prefer style blocks over inline style properties".to_string(),
            default_severity: LintSeverity::Info,
            code: ErrorCode::lint(102),
            enabled_by_default: false,
            category: RuleCategory::Style,
        },
        LintRule {
            id: "consistent-naming".to_string(),
            description: "Component names should follow naming conventions".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(103),
            enabled_by_default: true,
            category: RuleCategory::Style,
        },
        LintRule {
            id: "max-nesting-depth".to_string(),
            description: "Component nesting should not exceed maximum depth".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(104),
            enabled_by_default: true,
            category: RuleCategory::Style,
        },
        LintRule {
            id: "ordered-props".to_string(),
            description: "Properties should be in consistent order".to_string(),
            default_severity: LintSeverity::Info,
            code: ErrorCode::lint(105),
            enabled_by_default: false,
            category: RuleCategory::Style,
        },

        // Best practices (L02xx)
        LintRule {
            id: "no-empty-component".to_string(),
            description: "Component has no content or children".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(200),
            enabled_by_default: true,
            category: RuleCategory::BestPractice,
        },
        LintRule {
            id: "prefer-semantic".to_string(),
            description: "Use semantic components instead of generic containers".to_string(),
            default_severity: LintSeverity::Info,
            code: ErrorCode::lint(201),
            enabled_by_default: true,
            category: RuleCategory::BestPractice,
        },
        LintRule {
            id: "no-deprecated-component".to_string(),
            description: "Component is deprecated".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(202),
            enabled_by_default: true,
            category: RuleCategory::BestPractice,
        },
        LintRule {
            id: "no-deprecated-prop".to_string(),
            description: "Property is deprecated".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(203),
            enabled_by_default: true,
            category: RuleCategory::BestPractice,
        },
        LintRule {
            id: "use-tokens".to_string(),
            description: "Use design tokens for consistency".to_string(),
            default_severity: LintSeverity::Info,
            code: ErrorCode::lint(204),
            enabled_by_default: true,
            category: RuleCategory::BestPractice,
        },
        LintRule {
            id: "no-magic-numbers".to_string(),
            description: "Avoid magic numbers, use named constants".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(205),
            enabled_by_default: true,
            category: RuleCategory::BestPractice,
        },

        // Accessibility rules (L03xx)
        LintRule {
            id: "require-alt-text".to_string(),
            description: "Images must have alt text".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(300),
            enabled_by_default: true,
            category: RuleCategory::Accessibility,
        },
        LintRule {
            id: "require-label".to_string(),
            description: "Interactive elements must have labels".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(301),
            enabled_by_default: true,
            category: RuleCategory::Accessibility,
        },
        LintRule {
            id: "no-redundant-role".to_string(),
            description: "Avoid redundant ARIA roles".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(302),
            enabled_by_default: true,
            category: RuleCategory::Accessibility,
        },
        LintRule {
            id: "valid-aria".to_string(),
            description: "ARIA attributes must be valid".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(303),
            enabled_by_default: true,
            category: RuleCategory::Accessibility,
        },
        LintRule {
            id: "heading-order".to_string(),
            description: "Heading levels should not skip".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(304),
            enabled_by_default: true,
            category: RuleCategory::Accessibility,
        },
        LintRule {
            id: "no-positive-tabindex".to_string(),
            description: "Avoid positive tabindex values".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(305),
            enabled_by_default: true,
            category: RuleCategory::Accessibility,
        },
        LintRule {
            id: "clickable-has-role".to_string(),
            description: "Clickable elements should have appropriate roles".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(306),
            enabled_by_default: true,
            category: RuleCategory::Accessibility,
        },
        LintRule {
            id: "no-autofocus".to_string(),
            description: "Avoid autofocus as it can disorient users".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(307),
            enabled_by_default: true,
            category: RuleCategory::Accessibility,
        },

        // Performance rules (L04xx)
        LintRule {
            id: "no-large-inline-data".to_string(),
            description: "Avoid large inline data, use external resources".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(400),
            enabled_by_default: true,
            category: RuleCategory::Performance,
        },
        LintRule {
            id: "limit-children".to_string(),
            description: "Too many children may impact performance".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(401),
            enabled_by_default: true,
            category: RuleCategory::Performance,
        },
        LintRule {
            id: "prefer-virtualized".to_string(),
            description: "Use virtualized lists for large datasets".to_string(),
            default_severity: LintSeverity::Info,
            code: ErrorCode::lint(402),
            enabled_by_default: true,
            category: RuleCategory::Performance,
        },
        LintRule {
            id: "no-expensive-calc".to_string(),
            description: "Avoid expensive calculations in render".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(403),
            enabled_by_default: true,
            category: RuleCategory::Performance,
        },

        // Security rules (L05xx)
        LintRule {
            id: "no-inline-scripts".to_string(),
            description: "Avoid inline scripts".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(500),
            enabled_by_default: true,
            category: RuleCategory::Security,
        },
        LintRule {
            id: "no-external-links-without-rel".to_string(),
            description: "External links should have rel='noopener'".to_string(),
            default_severity: LintSeverity::Warning,
            code: ErrorCode::lint(501),
            enabled_by_default: true,
            category: RuleCategory::Security,
        },
        LintRule {
            id: "no-dangerous-props".to_string(),
            description: "Avoid potentially dangerous properties".to_string(),
            default_severity: LintSeverity::Error,
            code: ErrorCode::lint(502),
            enabled_by_default: true,
            category: RuleCategory::Security,
        },
    ]
}

/// Get enabled rules based on configuration
pub fn get_enabled_rules(config: &crate::LintConfig) -> Vec<LintRule> {
    let all = all_rules();

    // If specific rules are requested, only return those
    if !config.rules.is_empty() {
        return all
            .into_iter()
            .filter(|r| config.rules.contains(&r.id) && !config.disable.contains(&r.id))
            .collect();
    }

    // Otherwise return all default-enabled rules minus disabled ones
    all.into_iter()
        .filter(|r| r.enabled_by_default && !config.disable.contains(&r.id))
        .collect()
}

/// Get rule by ID
pub fn get_rule(id: &str) -> Option<LintRule> {
    all_rules().into_iter().find(|r| r.id == id)
}

/// Get rules by category
pub fn get_rules_by_category(category: RuleCategory) -> Vec<LintRule> {
    all_rules()
        .into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules() {
        let rules = all_rules();
        assert!(!rules.is_empty());

        // Verify all rules have unique IDs
        let mut ids: Vec<_> = rules.iter().map(|r| r.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), rules.len());

        // Verify all rules have unique codes
        let mut codes: Vec<_> = rules.iter().map(|r| r.code.code).collect();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), rules.len());
    }

    #[test]
    fn test_get_rule() {
        let rule = get_rule("no-hardcoded-colors");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().category, RuleCategory::Style);
    }

    #[test]
    fn test_get_enabled_rules() {
        let config = crate::LintConfig::default();
        let enabled = get_enabled_rules(&config);
        assert!(!enabled.is_empty());

        // All enabled rules should have enabled_by_default = true
        for rule in &enabled {
            assert!(rule.enabled_by_default);
        }
    }

    #[test]
    fn test_rules_by_category() {
        let a11y_rules = get_rules_by_category(RuleCategory::Accessibility);
        assert!(!a11y_rules.is_empty());

        for rule in &a11y_rules {
            assert_eq!(rule.category, RuleCategory::Accessibility);
        }
    }
}
