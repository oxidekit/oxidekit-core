//! Semantic Versioning implementation
//!
//! Implements SemVer 2.0.0 specification: https://semver.org/

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::error::VersionError;

/// A semantic version number
///
/// Follows SemVer 2.0.0 specification:
/// - MAJOR: incompatible API changes
/// - MINOR: backwards-compatible functionality additions
/// - PATCH: backwards-compatible bug fixes
///
/// # Examples
///
/// ```
/// use oxide_version::Version;
///
/// let v = Version::parse("1.2.3").unwrap();
/// assert_eq!(v.major, 1);
/// assert_eq!(v.minor, 2);
/// assert_eq!(v.patch, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Major version (incompatible changes)
    pub major: u64,
    /// Minor version (backwards-compatible additions)
    pub minor: u64,
    /// Patch version (backwards-compatible fixes)
    pub patch: u64,
    /// Pre-release identifier (e.g., "alpha.1", "beta.2", "rc.1")
    #[serde(default)]
    pub pre: PreRelease,
    /// Build metadata (e.g., "build.123", "20240115")
    #[serde(default)]
    pub build: BuildMetadata,
}

impl Version {
    /// Create a new version
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: PreRelease::empty(),
            build: BuildMetadata::empty(),
        }
    }

    /// Create a version with pre-release identifier
    pub fn with_prerelease(mut self, pre: PreRelease) -> Self {
        self.pre = pre;
        self
    }

    /// Create a version with build metadata
    pub fn with_build(mut self, build: BuildMetadata) -> Self {
        self.build = build;
        self
    }

    /// Parse a version string
    ///
    /// # Examples
    ///
    /// ```
    /// use oxide_version::Version;
    ///
    /// let v = Version::parse("1.2.3-alpha.1+build.123").unwrap();
    /// assert_eq!(v.major, 1);
    /// assert_eq!(v.minor, 2);
    /// assert_eq!(v.patch, 3);
    /// ```
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        let s = s.trim();

        // Strip leading 'v' if present
        let s = s.strip_prefix('v').unwrap_or(s);

        // Split off build metadata first (after +)
        let (version_pre, build) = match s.split_once('+') {
            Some((v, b)) => (v, BuildMetadata::new(b)),
            None => (s, BuildMetadata::empty()),
        };

        // Split off pre-release (after -)
        let (version, pre) = match version_pre.split_once('-') {
            Some((v, p)) => (v, PreRelease::new(p)?),
            None => (version_pre, PreRelease::empty()),
        };

        // Parse major.minor.patch
        let parts: Vec<&str> = version.split('.').collect();

        if parts.len() < 2 {
            return Err(VersionError::InvalidFormat(format!(
                "Version must have at least major.minor: {}",
                s
            )));
        }

        let major = parts[0].parse().map_err(|_| {
            VersionError::InvalidFormat(format!("Invalid major version: {}", parts[0]))
        })?;

        let minor = parts[1].parse().map_err(|_| {
            VersionError::InvalidFormat(format!("Invalid minor version: {}", parts[1]))
        })?;

        let patch = if parts.len() > 2 {
            parts[2].parse().map_err(|_| {
                VersionError::InvalidFormat(format!("Invalid patch version: {}", parts[2]))
            })?
        } else {
            0
        };

        Ok(Self {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }

    /// Check if this is a stable release (no pre-release identifier)
    pub fn is_stable(&self) -> bool {
        self.pre.is_empty()
    }

    /// Check if this is a pre-release version
    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }

    /// Check if this is version 0.x.x (initial development)
    pub fn is_initial_development(&self) -> bool {
        self.major == 0
    }

    /// Get the next major version (e.g., 1.2.3 -> 2.0.0)
    pub fn next_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    /// Get the next minor version (e.g., 1.2.3 -> 1.3.0)
    pub fn next_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    /// Get the next patch version (e.g., 1.2.3 -> 1.2.4)
    pub fn next_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }

    /// Check if two versions are compatible according to semver
    ///
    /// Two versions are compatible if:
    /// - Major versions match (for stable versions)
    /// - For 0.x.x, minor versions must also match
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        if self.major == 0 && other.major == 0 {
            // Initial development: minor version must match
            self.minor == other.minor
        } else {
            // Stable: major version must match
            self.major == other.major
        }
    }

    /// Return base version without pre-release or build metadata
    pub fn base_version(&self) -> Self {
        Self::new(self.major, self.minor, self.patch)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre)?;
        }

        if !self.build.is_empty() {
            write!(f, "+{}", self.build)?;
        }

        Ok(())
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        // Build metadata is ignored for equality
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.pre == other.pre
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Build metadata is ignored for ordering
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Pre-release versions have lower precedence than normal versions
        match (self.pre.is_empty(), other.pre.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater, // 1.0.0 > 1.0.0-alpha
            (false, true) => Ordering::Less,    // 1.0.0-alpha < 1.0.0
            (false, false) => self.pre.cmp(&other.pre),
        }
    }
}

impl std::hash::Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.major.hash(state);
        self.minor.hash(state);
        self.patch.hash(state);
        self.pre.hash(state);
        // Build metadata is not included in hash
    }
}

/// Pre-release identifier
///
/// Examples: "alpha", "alpha.1", "beta.2", "rc.1"
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PreRelease {
    identifiers: Vec<PreReleaseIdentifier>,
}

impl PreRelease {
    /// Create an empty pre-release identifier
    pub fn empty() -> Self {
        Self { identifiers: Vec::new() }
    }

    /// Create a new pre-release identifier
    pub fn new(s: &str) -> Result<Self, VersionError> {
        if s.is_empty() {
            return Ok(Self::empty());
        }

        let identifiers: Result<Vec<_>, _> = s
            .split('.')
            .map(|part| PreReleaseIdentifier::parse(part))
            .collect();

        Ok(Self { identifiers: identifiers? })
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.identifiers.is_empty()
    }
}

impl fmt::Display for PreRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.identifiers.iter().map(|i| i.to_string()).collect();
        write!(f, "{}", parts.join("."))
    }
}

impl PartialOrd for PreRelease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreRelease {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare identifier by identifier
        for (a, b) in self.identifiers.iter().zip(&other.identifiers) {
            match a.cmp(b) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }

        // If all compared identifiers are equal, longer is greater
        self.identifiers.len().cmp(&other.identifiers.len())
    }
}

/// A single pre-release identifier (either numeric or alphanumeric)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PreReleaseIdentifier {
    Numeric(u64),
    Alphanumeric(String),
}

impl PreReleaseIdentifier {
    fn parse(s: &str) -> Result<Self, VersionError> {
        if s.is_empty() {
            return Err(VersionError::InvalidFormat(
                "Empty pre-release identifier".to_string(),
            ));
        }

        // Check if all digits
        if s.chars().all(|c| c.is_ascii_digit()) {
            // Leading zeros not allowed for numeric identifiers
            if s.len() > 1 && s.starts_with('0') {
                return Err(VersionError::InvalidFormat(format!(
                    "Numeric pre-release identifier cannot have leading zeros: {}",
                    s
                )));
            }

            let n = s.parse().map_err(|_| {
                VersionError::InvalidFormat(format!("Invalid numeric identifier: {}", s))
            })?;

            Ok(PreReleaseIdentifier::Numeric(n))
        } else {
            // Alphanumeric: must contain only [0-9A-Za-z-]
            if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return Err(VersionError::InvalidFormat(format!(
                    "Invalid characters in pre-release identifier: {}",
                    s
                )));
            }

            Ok(PreReleaseIdentifier::Alphanumeric(s.to_string()))
        }
    }
}

impl fmt::Display for PreReleaseIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreReleaseIdentifier::Numeric(n) => write!(f, "{}", n),
            PreReleaseIdentifier::Alphanumeric(s) => write!(f, "{}", s),
        }
    }
}

impl PartialOrd for PreReleaseIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreReleaseIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Numeric identifiers are compared as integers
            (PreReleaseIdentifier::Numeric(a), PreReleaseIdentifier::Numeric(b)) => a.cmp(b),
            // Alphanumeric identifiers are compared lexically
            (PreReleaseIdentifier::Alphanumeric(a), PreReleaseIdentifier::Alphanumeric(b)) => {
                a.cmp(b)
            }
            // Numeric has lower precedence than alphanumeric
            (PreReleaseIdentifier::Numeric(_), PreReleaseIdentifier::Alphanumeric(_)) => {
                Ordering::Less
            }
            (PreReleaseIdentifier::Alphanumeric(_), PreReleaseIdentifier::Numeric(_)) => {
                Ordering::Greater
            }
        }
    }
}

/// Build metadata
///
/// Build metadata has no semantic meaning in semver comparisons
/// but is preserved for informational purposes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BuildMetadata {
    value: String,
}

impl BuildMetadata {
    /// Create empty build metadata
    pub fn empty() -> Self {
        Self { value: String::new() }
    }

    /// Create new build metadata
    pub fn new(s: &str) -> Self {
        Self { value: s.to_string() }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Get the raw value
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for BuildMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Version bump type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionBump {
    /// Major version bump (breaking changes)
    Major,
    /// Minor version bump (new features)
    Minor,
    /// Patch version bump (bug fixes)
    Patch,
}

impl Version {
    /// Apply a version bump
    pub fn bump(&self, bump: VersionBump) -> Self {
        match bump {
            VersionBump::Major => self.next_major(),
            VersionBump::Minor => self.next_minor(),
            VersionBump::Patch => self.next_patch(),
        }
    }

    /// Determine the bump type from one version to another
    pub fn bump_type_to(&self, to: &Version) -> Option<VersionBump> {
        if to.major > self.major {
            Some(VersionBump::Major)
        } else if to.major == self.major && to.minor > self.minor {
            Some(VersionBump::Minor)
        } else if to.major == self.major && to.minor == self.minor && to.patch > self.patch {
            Some(VersionBump::Patch)
        } else {
            None // Downgrade or same version
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre.is_empty());
        assert!(v.build.is_empty());
    }

    #[test]
    fn test_parse_with_v_prefix() {
        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
    }

    #[test]
    fn test_parse_prerelease() {
        let v = Version::parse("1.0.0-alpha.1").unwrap();
        assert!(!v.pre.is_empty());
        assert_eq!(v.pre.to_string(), "alpha.1");
    }

    #[test]
    fn test_parse_build_metadata() {
        let v = Version::parse("1.0.0+build.123").unwrap();
        assert!(!v.build.is_empty());
        assert_eq!(v.build.as_str(), "build.123");
    }

    #[test]
    fn test_parse_full() {
        let v = Version::parse("1.0.0-beta.2+build.456").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.pre.to_string(), "beta.2");
        assert_eq!(v.build.as_str(), "build.456");
    }

    #[test]
    fn test_ordering_basic() {
        assert!(Version::parse("1.0.0").unwrap() < Version::parse("2.0.0").unwrap());
        assert!(Version::parse("1.0.0").unwrap() < Version::parse("1.1.0").unwrap());
        assert!(Version::parse("1.0.0").unwrap() < Version::parse("1.0.1").unwrap());
    }

    #[test]
    fn test_ordering_prerelease() {
        // Pre-release has lower precedence
        assert!(Version::parse("1.0.0-alpha").unwrap() < Version::parse("1.0.0").unwrap());
        assert!(Version::parse("1.0.0-alpha").unwrap() < Version::parse("1.0.0-beta").unwrap());
        assert!(Version::parse("1.0.0-alpha.1").unwrap() < Version::parse("1.0.0-alpha.2").unwrap());
    }

    #[test]
    fn test_ordering_numeric_vs_alpha() {
        // Numeric identifiers have lower precedence
        assert!(Version::parse("1.0.0-1").unwrap() < Version::parse("1.0.0-alpha").unwrap());
    }

    #[test]
    fn test_equality_ignores_build() {
        let v1 = Version::parse("1.0.0+build1").unwrap();
        let v2 = Version::parse("1.0.0+build2").unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_display() {
        let v = Version::parse("1.2.3-alpha.1+build.123").unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha.1+build.123");
    }

    #[test]
    fn test_bump() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.bump(VersionBump::Major), Version::new(2, 0, 0));
        assert_eq!(v.bump(VersionBump::Minor), Version::new(1, 3, 0));
        assert_eq!(v.bump(VersionBump::Patch), Version::new(1, 2, 4));
    }

    #[test]
    fn test_is_compatible() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.5.0").unwrap();
        let v3 = Version::parse("2.0.0").unwrap();

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_initial_development_compat() {
        let v1 = Version::parse("0.1.0").unwrap();
        let v2 = Version::parse("0.1.5").unwrap();
        let v3 = Version::parse("0.2.0").unwrap();

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }
}
