//! Contributor License Agreement (CLA) checker
//!
//! Verifies that contributors have signed required agreements
//! before accepting contributions.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::{LegalError, LegalResult};

/// CLA/DCO status for a contributor
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaStatus {
    /// CLA signed and valid
    Signed,
    /// CLA not signed
    NotSigned,
    /// CLA expired
    Expired,
    /// Exempt from CLA (e.g., trivial changes)
    Exempt,
    /// CLA signature pending verification
    Pending,
}

impl ClaStatus {
    /// Check if the status allows contribution
    pub fn allows_contribution(&self) -> bool {
        matches!(self, ClaStatus::Signed | ClaStatus::Exempt)
    }
}

/// Type of contributor agreement
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgreementType {
    /// Individual Contributor License Agreement
    IndividualCla,
    /// Corporate Contributor License Agreement
    CorporateCla,
    /// Developer Certificate of Origin (sign-off in commit)
    Dco,
}

impl AgreementType {
    /// Get a human-readable name
    pub fn display_name(&self) -> &str {
        match self {
            AgreementType::IndividualCla => "Individual CLA",
            AgreementType::CorporateCla => "Corporate CLA",
            AgreementType::Dco => "Developer Certificate of Origin",
        }
    }
}

/// CLA checker for verifying contributor agreements
#[derive(Debug, Clone)]
pub struct ClaChecker {
    /// Signed agreements database
    signed_agreements: HashMap<String, ContributorAgreement>,
    /// Organizations with corporate CLAs
    corporate_cla_orgs: HashSet<String>,
    /// Required agreement type
    required_type: AgreementType,
    /// Whether to allow DCO as alternative
    allow_dco_fallback: bool,
    /// Exempt email patterns (e.g., bots)
    exempt_patterns: Vec<String>,
    /// Exempt contribution patterns (e.g., typo fixes)
    trivial_patterns: Vec<String>,
}

impl Default for ClaChecker {
    fn default() -> Self {
        Self::new(AgreementType::IndividualCla)
    }
}

impl ClaChecker {
    /// Create a new CLA checker
    pub fn new(required_type: AgreementType) -> Self {
        Self {
            signed_agreements: HashMap::new(),
            corporate_cla_orgs: HashSet::new(),
            required_type,
            allow_dco_fallback: true,
            exempt_patterns: vec![
                "*[bot]@users.noreply.github.com".to_string(),
                "dependabot*".to_string(),
                "renovate*".to_string(),
            ],
            trivial_patterns: vec![
                "*.md".to_string(),
                "typo".to_string(),
                "docs:".to_string(),
            ],
        }
    }

    /// Allow DCO sign-off as fallback
    pub fn with_dco_fallback(mut self, allow: bool) -> Self {
        self.allow_dco_fallback = allow;
        self
    }

    /// Add exempt email pattern
    pub fn add_exempt_pattern(&mut self, pattern: &str) {
        self.exempt_patterns.push(pattern.to_string());
    }

    /// Add corporate CLA organization
    pub fn add_corporate_org(&mut self, org: &str) {
        self.corporate_cla_orgs.insert(org.to_string());
    }

    /// Register a signed agreement
    pub fn register_agreement(&mut self, agreement: ContributorAgreement) {
        self.signed_agreements.insert(agreement.email.clone(), agreement);
    }

    /// Load agreements from a JSON file
    pub fn load_from_file(&mut self, path: impl AsRef<Path>) -> LegalResult<()> {
        let content = std::fs::read_to_string(path)?;
        let database: ClaDatabase = serde_json::from_str(&content)?;

        for agreement in database.agreements {
            self.register_agreement(agreement);
        }

        for org in database.corporate_orgs {
            self.add_corporate_org(&org);
        }

        Ok(())
    }

    /// Save agreements to a JSON file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> LegalResult<()> {
        let database = ClaDatabase {
            agreements: self.signed_agreements.values().cloned().collect(),
            corporate_orgs: self.corporate_cla_orgs.iter().cloned().collect(),
        };

        let content = serde_json::to_string_pretty(&database)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Check CLA status for a contributor
    pub fn check(&self, email: &str, name: Option<&str>) -> ClaCheckResult {
        // Check exempt patterns
        if self.is_exempt(email) {
            return ClaCheckResult {
                email: email.to_string(),
                name: name.map(String::from),
                status: ClaStatus::Exempt,
                agreement_type: None,
                signed_at: None,
                expires_at: None,
                message: "Contributor is exempt from CLA requirement".to_string(),
            };
        }

        // Check for signed individual CLA
        if let Some(agreement) = self.signed_agreements.get(email) {
            if let Some(expires) = agreement.expires_at {
                if expires < Utc::now() {
                    return ClaCheckResult {
                        email: email.to_string(),
                        name: name.map(String::from),
                        status: ClaStatus::Expired,
                        agreement_type: Some(agreement.agreement_type),
                        signed_at: Some(agreement.signed_at),
                        expires_at: Some(expires),
                        message: "CLA signature has expired".to_string(),
                    };
                }
            }

            return ClaCheckResult {
                email: email.to_string(),
                name: name.map(String::from),
                status: ClaStatus::Signed,
                agreement_type: Some(agreement.agreement_type),
                signed_at: Some(agreement.signed_at),
                expires_at: agreement.expires_at,
                message: "CLA is signed and valid".to_string(),
            };
        }

        // Check for corporate CLA
        if let Some(org) = self.extract_org(email) {
            if self.corporate_cla_orgs.contains(&org) {
                return ClaCheckResult {
                    email: email.to_string(),
                    name: name.map(String::from),
                    status: ClaStatus::Signed,
                    agreement_type: Some(AgreementType::CorporateCla),
                    signed_at: None,
                    expires_at: None,
                    message: format!("Covered by corporate CLA for {}", org),
                };
            }
        }

        // Not signed
        ClaCheckResult {
            email: email.to_string(),
            name: name.map(String::from),
            status: ClaStatus::NotSigned,
            agreement_type: None,
            signed_at: None,
            expires_at: None,
            message: format!("Please sign the {} before contributing", self.required_type.display_name()),
        }
    }

    /// Check CLA status for a commit
    pub fn check_commit(&self, commit: &CommitInfo) -> ClaCheckResult {
        let mut result = self.check(&commit.author_email, Some(&commit.author_name));

        // Check for DCO sign-off if allowed and not signed
        if result.status == ClaStatus::NotSigned && self.allow_dco_fallback {
            if self.has_dco_signoff(&commit.message, &commit.author_email) {
                result.status = ClaStatus::Signed;
                result.agreement_type = Some(AgreementType::Dco);
                result.message = "Commit has DCO sign-off".to_string();
            }
        }

        // Check for trivial contribution exemption
        if result.status == ClaStatus::NotSigned && self.is_trivial_contribution(commit) {
            result.status = ClaStatus::Exempt;
            result.message = "Trivial contribution exempt from CLA".to_string();
        }

        result
    }

    /// Check multiple commits
    pub fn check_commits(&self, commits: &[CommitInfo]) -> Vec<ClaCheckResult> {
        commits.iter().map(|c| self.check_commit(c)).collect()
    }

    /// Check if all commits pass CLA requirements
    pub fn verify_all(&self, commits: &[CommitInfo]) -> LegalResult<()> {
        let results = self.check_commits(commits);

        let failures: Vec<_> = results
            .iter()
            .filter(|r| !r.status.allows_contribution())
            .collect();

        if failures.is_empty() {
            Ok(())
        } else {
            let emails: Vec<_> = failures.iter().map(|r| r.email.as_str()).collect();
            Err(LegalError::ClaNotSigned(emails.join(", ")))
        }
    }

    /// Check if email matches exempt pattern
    fn is_exempt(&self, email: &str) -> bool {
        for pattern in &self.exempt_patterns {
            if self.matches_pattern(email, pattern) {
                return true;
            }
        }
        false
    }

    /// Simple pattern matching (supports * wildcard)
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        let text = text.to_lowercase();
        let pattern = pattern.to_lowercase();

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                if parts[0].is_empty() {
                    return text.ends_with(parts[1]);
                } else if parts[1].is_empty() {
                    return text.starts_with(parts[0]);
                } else {
                    return text.starts_with(parts[0]) && text.ends_with(parts[1]);
                }
            }
        }

        text == pattern
    }

    /// Extract organization from email
    fn extract_org(&self, email: &str) -> Option<String> {
        email.split('@').nth(1).map(|s| s.to_lowercase())
    }

    /// Check for DCO sign-off in commit message
    fn has_dco_signoff(&self, message: &str, email: &str) -> bool {
        let signoff_pattern = format!("Signed-off-by:.*<{}>", regex_lite::escape(email));
        regex_lite::Regex::new(&signoff_pattern)
            .map(|re| re.is_match(message))
            .unwrap_or(false)
    }

    /// Check if contribution is trivial
    fn is_trivial_contribution(&self, commit: &CommitInfo) -> bool {
        let message = commit.message.to_lowercase();

        for pattern in &self.trivial_patterns {
            if message.contains(pattern) {
                return true;
            }
        }

        // Check if only documentation files changed
        if !commit.changed_files.is_empty() {
            let all_docs = commit.changed_files.iter().all(|f| {
                f.ends_with(".md") || f.ends_with(".txt") || f.starts_with("docs/")
            });
            if all_docs {
                return true;
            }
        }

        false
    }

    /// Generate CLA signing instructions
    pub fn signing_instructions(&self) -> String {
        match self.required_type {
            AgreementType::IndividualCla => {
                r#"To contribute, please sign the Individual Contributor License Agreement:

1. Visit https://oxidekit.com/cla
2. Sign in with your GitHub account
3. Review and accept the CLA
4. Your contributions will be automatically verified

For questions, contact legal@oxidekit.com
"#.to_string()
            }
            AgreementType::CorporateCla => {
                r#"To contribute on behalf of your organization, a Corporate CLA is required:

1. Download the Corporate CLA from https://oxidekit.com/corporate-cla
2. Have an authorized signatory complete and sign the agreement
3. Send the signed agreement to legal@oxidekit.com
4. Once processed, all employees will be able to contribute

For questions, contact legal@oxidekit.com
"#.to_string()
            }
            AgreementType::Dco => {
                r#"To contribute, please sign off your commits with the Developer Certificate of Origin:

Add the following line to your commit messages:

    Signed-off-by: Your Name <your.email@example.com>

Or use `git commit -s` to automatically add the sign-off.

By signing off, you certify that you wrote the code or have the right to submit it.

Full DCO text: https://developercertificate.org/
"#.to_string()
            }
        }
    }
}

/// Contributor agreement record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorAgreement {
    /// Contributor email
    pub email: String,
    /// Contributor name
    pub name: String,
    /// GitHub username (if applicable)
    pub github_username: Option<String>,
    /// Agreement type
    pub agreement_type: AgreementType,
    /// When the agreement was signed
    pub signed_at: DateTime<Utc>,
    /// When the agreement expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Agreement version signed
    pub version: String,
    /// Organization (for corporate CLAs)
    pub organization: Option<String>,
}

impl ContributorAgreement {
    /// Create a new individual CLA record
    pub fn individual(email: &str, name: &str, version: &str) -> Self {
        Self {
            email: email.to_string(),
            name: name.to_string(),
            github_username: None,
            agreement_type: AgreementType::IndividualCla,
            signed_at: Utc::now(),
            expires_at: None,
            version: version.to_string(),
            organization: None,
        }
    }

    /// Create a new corporate CLA record
    pub fn corporate(email: &str, name: &str, organization: &str, version: &str) -> Self {
        Self {
            email: email.to_string(),
            name: name.to_string(),
            github_username: None,
            agreement_type: AgreementType::CorporateCla,
            signed_at: Utc::now(),
            expires_at: None,
            version: version.to_string(),
            organization: Some(organization.to_string()),
        }
    }

    /// Set GitHub username
    pub fn with_github(mut self, username: &str) -> Self {
        self.github_username = Some(username.to_string());
        self
    }

    /// Set expiration
    pub fn with_expiration(mut self, expires: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires);
        self
    }
}

/// Result of CLA check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaCheckResult {
    /// Contributor email
    pub email: String,
    /// Contributor name (if known)
    pub name: Option<String>,
    /// CLA status
    pub status: ClaStatus,
    /// Type of agreement (if signed)
    pub agreement_type: Option<AgreementType>,
    /// When signed (if applicable)
    pub signed_at: Option<DateTime<Utc>>,
    /// When expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Human-readable message
    pub message: String,
}

/// Commit information for CLA checking
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit SHA
    pub sha: String,
    /// Author name
    pub author_name: String,
    /// Author email
    pub author_email: String,
    /// Commit message
    pub message: String,
    /// Changed files
    pub changed_files: Vec<String>,
}

/// CLA database structure
#[derive(Debug, Serialize, Deserialize)]
struct ClaDatabase {
    agreements: Vec<ContributorAgreement>,
    corporate_orgs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cla_check_not_signed() {
        let checker = ClaChecker::new(AgreementType::IndividualCla);
        let result = checker.check("unsigned@example.com", Some("Test User"));

        assert_eq!(result.status, ClaStatus::NotSigned);
    }

    #[test]
    fn test_cla_check_signed() {
        let mut checker = ClaChecker::new(AgreementType::IndividualCla);
        checker.register_agreement(ContributorAgreement::individual(
            "signed@example.com",
            "Signed User",
            "1.0",
        ));

        let result = checker.check("signed@example.com", None);
        assert_eq!(result.status, ClaStatus::Signed);
    }

    #[test]
    fn test_cla_exempt_bot() {
        let checker = ClaChecker::new(AgreementType::IndividualCla);
        let result = checker.check("dependabot[bot]@users.noreply.github.com", None);

        assert_eq!(result.status, ClaStatus::Exempt);
    }

    #[test]
    fn test_cla_corporate() {
        let mut checker = ClaChecker::new(AgreementType::IndividualCla);
        checker.add_corporate_org("bigcorp.com");

        let result = checker.check("employee@bigcorp.com", Some("Corporate Employee"));
        assert_eq!(result.status, ClaStatus::Signed);
        assert_eq!(result.agreement_type, Some(AgreementType::CorporateCla));
    }

    #[test]
    fn test_dco_signoff() {
        let checker = ClaChecker::new(AgreementType::IndividualCla).with_dco_fallback(true);

        let commit = CommitInfo {
            sha: "abc123".to_string(),
            author_name: "Test User".to_string(),
            author_email: "test@example.com".to_string(),
            message: "Fix bug\n\nSigned-off-by: Test User <test@example.com>".to_string(),
            changed_files: vec!["src/main.rs".to_string()],
        };

        let result = checker.check_commit(&commit);
        assert_eq!(result.status, ClaStatus::Signed);
        assert_eq!(result.agreement_type, Some(AgreementType::Dco));
    }

    #[test]
    fn test_trivial_docs_contribution() {
        let checker = ClaChecker::new(AgreementType::IndividualCla);

        let commit = CommitInfo {
            sha: "abc123".to_string(),
            author_name: "Doc Fixer".to_string(),
            author_email: "docs@example.com".to_string(),
            message: "Fix typo in README".to_string(),
            changed_files: vec!["README.md".to_string()],
        };

        let result = checker.check_commit(&commit);
        assert_eq!(result.status, ClaStatus::Exempt);
    }
}
