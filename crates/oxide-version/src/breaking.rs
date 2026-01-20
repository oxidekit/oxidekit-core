//! Breaking change detection
//!
//! Analyzes changes between versions to detect breaking changes.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::semver::{Version, VersionBump};

/// Types of breaking changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakingChangeType {
    /// Function/method removed
    FunctionRemoved,
    /// Function signature changed
    SignatureChanged,
    /// Struct field removed
    FieldRemoved,
    /// Struct field type changed
    FieldTypeChanged,
    /// Enum variant removed
    VariantRemoved,
    /// Enum variant fields changed
    VariantChanged,
    /// Trait method added (breaks implementors)
    TraitMethodAdded,
    /// Trait removed
    TraitRemoved,
    /// Type alias changed
    TypeChanged,
    /// Module removed
    ModuleRemoved,
    /// Feature removed
    FeatureRemoved,
    /// Behavior change
    BehaviorChanged,
    /// Configuration format changed
    ConfigChanged,
    /// Default value changed
    DefaultChanged,
    /// Dependency requirement changed
    DependencyChanged,
}

impl std::fmt::Display for BreakingChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakingChangeType::FunctionRemoved => write!(f, "function removed"),
            BreakingChangeType::SignatureChanged => write!(f, "signature changed"),
            BreakingChangeType::FieldRemoved => write!(f, "field removed"),
            BreakingChangeType::FieldTypeChanged => write!(f, "field type changed"),
            BreakingChangeType::VariantRemoved => write!(f, "variant removed"),
            BreakingChangeType::VariantChanged => write!(f, "variant changed"),
            BreakingChangeType::TraitMethodAdded => write!(f, "trait method added"),
            BreakingChangeType::TraitRemoved => write!(f, "trait removed"),
            BreakingChangeType::TypeChanged => write!(f, "type changed"),
            BreakingChangeType::ModuleRemoved => write!(f, "module removed"),
            BreakingChangeType::FeatureRemoved => write!(f, "feature removed"),
            BreakingChangeType::BehaviorChanged => write!(f, "behavior changed"),
            BreakingChangeType::ConfigChanged => write!(f, "config format changed"),
            BreakingChangeType::DefaultChanged => write!(f, "default value changed"),
            BreakingChangeType::DependencyChanged => write!(f, "dependency changed"),
        }
    }
}

/// Severity of a breaking change
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BreakingSeverity {
    /// Minor breaking change (usually easy to fix)
    Minor,
    /// Moderate breaking change (requires some work)
    Moderate,
    /// Major breaking change (significant refactoring needed)
    Major,
    /// Critical breaking change (fundamental API change)
    Critical,
}

impl Default for BreakingSeverity {
    fn default() -> Self {
        BreakingSeverity::Moderate
    }
}

/// A detected breaking change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    /// Type of change
    pub change_type: BreakingChangeType,
    /// Severity
    pub severity: BreakingSeverity,
    /// What was affected (function name, type name, etc.)
    pub subject: String,
    /// Description of the change
    pub description: String,
    /// Old value/signature (if applicable)
    pub old_value: Option<String>,
    /// New value/signature (if applicable)
    pub new_value: Option<String>,
    /// Migration instructions
    pub migration: Option<String>,
    /// Related documentation
    pub docs_url: Option<String>,
}

impl BreakingChange {
    /// Create a new breaking change
    pub fn new(
        change_type: BreakingChangeType,
        subject: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            change_type,
            severity: BreakingSeverity::default(),
            subject: subject.into(),
            description: description.into(),
            old_value: None,
            new_value: None,
            migration: None,
            docs_url: None,
        }
    }

    /// Set severity
    pub fn with_severity(mut self, severity: BreakingSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set old/new values
    pub fn with_change(mut self, old: impl Into<String>, new: impl Into<String>) -> Self {
        self.old_value = Some(old.into());
        self.new_value = Some(new.into());
        self
    }

    /// Set migration instructions
    pub fn with_migration(mut self, migration: impl Into<String>) -> Self {
        self.migration = Some(migration.into());
        self
    }

    /// Set documentation URL
    pub fn with_docs(mut self, url: impl Into<String>) -> Self {
        self.docs_url = Some(url.into());
        self
    }

    /// Format as a report entry
    pub fn to_report_entry(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "[{:?}] {} - {}\n",
            self.severity, self.change_type, self.subject
        ));
        output.push_str(&format!("    {}\n", self.description));

        if let (Some(old), Some(new)) = (&self.old_value, &self.new_value) {
            output.push_str(&format!("    Before: {}\n", old));
            output.push_str(&format!("    After:  {}\n", new));
        }

        if let Some(ref migration) = self.migration {
            output.push_str(&format!("    Migration: {}\n", migration));
        }

        output
    }
}

/// Analysis result for version changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeAnalysis {
    /// Source version
    pub from_version: Version,
    /// Target version
    pub to_version: Version,
    /// Detected breaking changes
    pub breaking_changes: Vec<BreakingChange>,
    /// Non-breaking additions
    pub additions: Vec<String>,
    /// Non-breaking deprecations
    pub deprecations: Vec<String>,
    /// Required version bump based on changes
    pub required_bump: VersionBump,
    /// Whether the actual bump matches the required bump
    pub bump_matches: bool,
}

impl ChangeAnalysis {
    /// Create a new analysis
    pub fn new(from: Version, to: Version) -> Self {
        let required_bump = VersionBump::Patch;
        let actual_bump = from.bump_type_to(&to);
        let bump_matches = actual_bump.map(|b| b as u8 >= required_bump as u8).unwrap_or(false);

        Self {
            from_version: from,
            to_version: to,
            breaking_changes: Vec::new(),
            additions: Vec::new(),
            deprecations: Vec::new(),
            required_bump,
            bump_matches,
        }
    }

    /// Add a breaking change
    pub fn add_breaking_change(&mut self, change: BreakingChange) {
        self.breaking_changes.push(change);
        self.update_required_bump();
    }

    /// Add an addition
    pub fn add_addition(&mut self, item: impl Into<String>) {
        self.additions.push(item.into());
        // Additions require at least minor bump
        if self.required_bump == VersionBump::Patch {
            self.required_bump = VersionBump::Minor;
        }
        self.update_bump_matches();
    }

    /// Add a deprecation
    pub fn add_deprecation(&mut self, item: impl Into<String>) {
        self.deprecations.push(item.into());
    }

    /// Update required bump based on breaking changes
    fn update_required_bump(&mut self) {
        if !self.breaking_changes.is_empty() {
            // Breaking changes require major bump (or minor in 0.x)
            self.required_bump = if self.from_version.major == 0 {
                VersionBump::Minor
            } else {
                VersionBump::Major
            };
        }
        self.update_bump_matches();
    }

    /// Update whether actual bump matches required
    fn update_bump_matches(&mut self) {
        let actual_bump = self.from_version.bump_type_to(&self.to_version);
        self.bump_matches = actual_bump
            .map(|b| b as u8 >= self.required_bump as u8)
            .unwrap_or(false);
    }

    /// Check if there are any breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        !self.breaking_changes.is_empty()
    }

    /// Get the most severe breaking change
    pub fn most_severe(&self) -> Option<&BreakingChange> {
        self.breaking_changes.iter().max_by_key(|c| c.severity)
    }

    /// Group breaking changes by type
    pub fn by_type(&self) -> HashMap<BreakingChangeType, Vec<&BreakingChange>> {
        let mut groups: HashMap<BreakingChangeType, Vec<&BreakingChange>> = HashMap::new();
        for change in &self.breaking_changes {
            groups.entry(change.change_type).or_default().push(change);
        }
        groups
    }

    /// Generate a changelog entry
    pub fn to_changelog(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "## {} -> {}\n\n",
            self.from_version, self.to_version
        ));

        if self.has_breaking_changes() {
            output.push_str("### Breaking Changes\n\n");
            for change in &self.breaking_changes {
                output.push_str(&format!("- **{}**: {}\n", change.subject, change.description));
                if let Some(ref migration) = change.migration {
                    output.push_str(&format!("  - Migration: {}\n", migration));
                }
            }
            output.push('\n');
        }

        if !self.additions.is_empty() {
            output.push_str("### Added\n\n");
            for item in &self.additions {
                output.push_str(&format!("- {}\n", item));
            }
            output.push('\n');
        }

        if !self.deprecations.is_empty() {
            output.push_str("### Deprecated\n\n");
            for item in &self.deprecations {
                output.push_str(&format!("- {}\n", item));
            }
            output.push('\n');
        }

        output
    }

    /// Generate a full analysis report
    pub fn to_report(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "Change Analysis: {} -> {}\n",
            self.from_version, self.to_version
        ));
        output.push_str(&format!("{}\n\n", "=".repeat(50)));

        // Summary
        output.push_str("Summary:\n");
        output.push_str(&format!("  Breaking changes: {}\n", self.breaking_changes.len()));
        output.push_str(&format!("  Additions: {}\n", self.additions.len()));
        output.push_str(&format!("  Deprecations: {}\n", self.deprecations.len()));
        output.push_str(&format!("  Required bump: {:?}\n", self.required_bump));
        output.push_str(&format!(
            "  Bump validation: {}\n\n",
            if self.bump_matches { "PASS" } else { "FAIL" }
        ));

        // Breaking changes detail
        if !self.breaking_changes.is_empty() {
            output.push_str("Breaking Changes:\n");
            output.push_str(&format!("{}\n", "-".repeat(30)));

            let by_type = self.by_type();
            for (change_type, changes) in by_type {
                output.push_str(&format!("\n{}:\n", change_type));
                for change in changes {
                    output.push_str(&format!("  - {}\n", change.subject));
                    if let Some(ref migration) = change.migration {
                        output.push_str(&format!("    Migration: {}\n", migration));
                    }
                }
            }
        }

        output
    }
}

/// Breaking change detector
pub struct BreakingChangeDetector {
    /// Known patterns that indicate breaking changes
    #[allow(dead_code)]
    patterns: Vec<BreakingPattern>,
}

/// A pattern for detecting breaking changes
#[derive(Debug, Clone)]
pub struct BreakingPattern {
    /// Pattern name
    pub name: String,
    /// Change type this detects
    pub change_type: BreakingChangeType,
    /// Default severity
    pub severity: BreakingSeverity,
    /// Description
    pub description: String,
}

impl BreakingChangeDetector {
    /// Create a new detector with default patterns
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    /// Default breaking change patterns
    fn default_patterns() -> Vec<BreakingPattern> {
        vec![
            BreakingPattern {
                name: "function_removal".to_string(),
                change_type: BreakingChangeType::FunctionRemoved,
                severity: BreakingSeverity::Major,
                description: "Public function was removed".to_string(),
            },
            BreakingPattern {
                name: "signature_change".to_string(),
                change_type: BreakingChangeType::SignatureChanged,
                severity: BreakingSeverity::Major,
                description: "Function signature changed".to_string(),
            },
            BreakingPattern {
                name: "field_removal".to_string(),
                change_type: BreakingChangeType::FieldRemoved,
                severity: BreakingSeverity::Moderate,
                description: "Struct field was removed".to_string(),
            },
            BreakingPattern {
                name: "type_change".to_string(),
                change_type: BreakingChangeType::TypeChanged,
                severity: BreakingSeverity::Major,
                description: "Type definition changed".to_string(),
            },
            BreakingPattern {
                name: "trait_method_added".to_string(),
                change_type: BreakingChangeType::TraitMethodAdded,
                severity: BreakingSeverity::Minor,
                description: "Non-default trait method added".to_string(),
            },
            BreakingPattern {
                name: "behavior_change".to_string(),
                change_type: BreakingChangeType::BehaviorChanged,
                severity: BreakingSeverity::Moderate,
                description: "Behavior changed".to_string(),
            },
        ]
    }

    /// Analyze changes between two API definitions
    pub fn analyze(
        &self,
        from_version: Version,
        to_version: Version,
        changes: Vec<DetectedChange>,
    ) -> ChangeAnalysis {
        let mut analysis = ChangeAnalysis::new(from_version, to_version);

        for change in changes {
            match change.kind {
                ChangeKind::Removed => {
                    let breaking = BreakingChange::new(
                        BreakingChangeType::FunctionRemoved,
                        &change.item,
                        format!("{} was removed", change.item),
                    )
                    .with_severity(BreakingSeverity::Major);
                    analysis.add_breaking_change(breaking);
                }
                ChangeKind::Modified => {
                    if change.is_breaking {
                        let breaking = BreakingChange::new(
                            BreakingChangeType::SignatureChanged,
                            &change.item,
                            format!("{} signature changed", change.item),
                        )
                        .with_severity(BreakingSeverity::Moderate);
                        analysis.add_breaking_change(breaking);
                    }
                }
                ChangeKind::Added => {
                    analysis.add_addition(&change.item);
                }
                ChangeKind::Deprecated => {
                    analysis.add_deprecation(&change.item);
                }
            }
        }

        analysis
    }
}

impl Default for BreakingChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// A detected change between versions
#[derive(Debug, Clone)]
pub struct DetectedChange {
    /// Item that changed (function name, type name, etc.)
    pub item: String,
    /// Kind of change
    pub kind: ChangeKind,
    /// Whether this is a breaking change
    pub is_breaking: bool,
    /// Optional details
    pub details: Option<String>,
}

/// Kind of change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// Item was added
    Added,
    /// Item was removed
    Removed,
    /// Item was modified
    Modified,
    /// Item was deprecated
    Deprecated,
}

impl DetectedChange {
    /// Create a removal change (always breaking)
    pub fn removed(item: impl Into<String>) -> Self {
        Self {
            item: item.into(),
            kind: ChangeKind::Removed,
            is_breaking: true,
            details: None,
        }
    }

    /// Create an addition change (never breaking)
    pub fn added(item: impl Into<String>) -> Self {
        Self {
            item: item.into(),
            kind: ChangeKind::Added,
            is_breaking: false,
            details: None,
        }
    }

    /// Create a modification change
    pub fn modified(item: impl Into<String>, is_breaking: bool) -> Self {
        Self {
            item: item.into(),
            kind: ChangeKind::Modified,
            is_breaking,
            details: None,
        }
    }

    /// Create a deprecation change
    pub fn deprecated(item: impl Into<String>) -> Self {
        Self {
            item: item.into(),
            kind: ChangeKind::Deprecated,
            is_breaking: false,
            details: None,
        }
    }

    /// Add details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaking_change_creation() {
        let change = BreakingChange::new(
            BreakingChangeType::FunctionRemoved,
            "Widget::render",
            "Method was removed",
        )
        .with_severity(BreakingSeverity::Major)
        .with_migration("Use Widget::draw instead");

        assert_eq!(change.change_type, BreakingChangeType::FunctionRemoved);
        assert_eq!(change.severity, BreakingSeverity::Major);
        assert!(change.migration.is_some());
    }

    #[test]
    fn test_change_analysis() {
        let from = Version::parse("1.0.0").unwrap();
        let to = Version::parse("2.0.0").unwrap();
        let mut analysis = ChangeAnalysis::new(from, to);

        analysis.add_breaking_change(BreakingChange::new(
            BreakingChangeType::FunctionRemoved,
            "old_function",
            "Removed",
        ));

        assert!(analysis.has_breaking_changes());
        assert_eq!(analysis.required_bump, VersionBump::Major);
        assert!(analysis.bump_matches); // 1.0.0 -> 2.0.0 is major
    }

    #[test]
    fn test_detector() {
        let detector = BreakingChangeDetector::new();

        let changes = vec![
            DetectedChange::removed("old_api"),
            DetectedChange::added("new_api"),
            DetectedChange::deprecated("deprecated_api"),
        ];

        let analysis = detector.analyze(
            Version::parse("1.0.0").unwrap(),
            Version::parse("2.0.0").unwrap(),
            changes,
        );

        assert_eq!(analysis.breaking_changes.len(), 1);
        assert_eq!(analysis.additions.len(), 1);
        assert_eq!(analysis.deprecations.len(), 1);
    }

    #[test]
    fn test_changelog_generation() {
        let mut analysis = ChangeAnalysis::new(
            Version::parse("1.0.0").unwrap(),
            Version::parse("2.0.0").unwrap(),
        );

        analysis.add_breaking_change(
            BreakingChange::new(
                BreakingChangeType::FunctionRemoved,
                "Widget::render",
                "Method removed",
            )
            .with_migration("Use Widget::draw"),
        );
        analysis.add_addition("Widget::draw");
        analysis.add_deprecation("Widget::update");

        let changelog = analysis.to_changelog();
        assert!(changelog.contains("Breaking Changes"));
        assert!(changelog.contains("Added"));
        assert!(changelog.contains("Deprecated"));
    }
}
