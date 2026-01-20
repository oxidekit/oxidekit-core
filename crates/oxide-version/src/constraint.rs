//! Version constraints and requirements
//!
//! Implements version constraints similar to Cargo's version requirements.

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::semver::Version;
use crate::error::VersionError;

/// A version requirement (e.g., ">=1.0.0, <2.0.0")
///
/// Supports the following constraint operators:
/// - `^` (caret): Compatible with version (default for Cargo)
/// - `~` (tilde): Approximately equal
/// - `=` (exact): Exactly this version
/// - `>=`, `>`, `<=`, `<`: Comparisons
/// - `*`: Any version
///
/// # Examples
///
/// ```
/// use oxide_version::{VersionReq, Version};
///
/// let req = VersionReq::parse(">=1.0.0, <2.0.0").unwrap();
/// assert!(req.matches(&Version::parse("1.5.0").unwrap()));
/// assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionReq {
    constraints: Vec<VersionConstraint>,
}

impl VersionReq {
    /// Create a new version requirement
    pub fn new(constraints: Vec<VersionConstraint>) -> Self {
        Self { constraints }
    }

    /// Parse a version requirement string
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        let s = s.trim();

        // Handle wildcard
        if s == "*" {
            return Ok(Self {
                constraints: vec![VersionConstraint::Any],
            });
        }

        // Split on commas for multiple constraints
        let constraints: Result<Vec<_>, _> = s
            .split(',')
            .map(|part| VersionConstraint::parse(part.trim()))
            .collect();

        Ok(Self {
            constraints: constraints?,
        })
    }

    /// Create a requirement that matches any version
    pub fn any() -> Self {
        Self {
            constraints: vec![VersionConstraint::Any],
        }
    }

    /// Create a requirement for an exact version
    pub fn exact(version: Version) -> Self {
        Self {
            constraints: vec![VersionConstraint::Exact(version)],
        }
    }

    /// Create a caret requirement (^x.y.z)
    pub fn caret(version: Version) -> Self {
        Self {
            constraints: vec![VersionConstraint::Caret(version)],
        }
    }

    /// Create a tilde requirement (~x.y.z)
    pub fn tilde(version: Version) -> Self {
        Self {
            constraints: vec![VersionConstraint::Tilde(version)],
        }
    }

    /// Check if a version matches this requirement
    pub fn matches(&self, version: &Version) -> bool {
        self.constraints.iter().all(|c| c.matches(version))
    }

    /// Get the minimum version that could satisfy this requirement
    pub fn minimum_version(&self) -> Option<Version> {
        // Find the highest minimum across all constraints
        let mut min: Option<Version> = None;

        for constraint in &self.constraints {
            match constraint.minimum_version() {
                Some(v) => match &min {
                    Some(current) if &v > current => min = Some(v),
                    None => min = Some(v),
                    _ => {}
                },
                None => {}
            }
        }

        min
    }

    /// Get the maximum version that could satisfy this requirement
    pub fn maximum_version(&self) -> Option<Version> {
        // Find the lowest maximum across all constraints
        let mut max: Option<Version> = None;

        for constraint in &self.constraints {
            match constraint.maximum_version() {
                Some(v) => match &max {
                    Some(current) if &v < current => max = Some(v),
                    None => max = Some(v),
                    _ => {}
                },
                None => {}
            }
        }

        max
    }

    /// Check if this requirement can be satisfied
    pub fn is_satisfiable(&self) -> bool {
        // Check if minimum <= maximum
        match (self.minimum_version(), self.maximum_version()) {
            (Some(min), Some(max)) => min <= max,
            _ => true,
        }
    }

    /// Check if two requirements could have overlapping versions
    pub fn overlaps(&self, other: &VersionReq) -> bool {
        // Check if either's minimum is >= the other's maximum (exclusive bounds)
        // For <X constraints, the maximum is exclusive, so we need >= not >
        match (self.minimum_version(), other.maximum_version()) {
            (Some(min), Some(max)) if min >= max => return false,
            _ => {}
        }

        match (other.minimum_version(), self.maximum_version()) {
            (Some(min), Some(max)) if min >= max => return false,
            _ => {}
        }

        true
    }

    /// Get the constraints
    pub fn constraints(&self) -> &[VersionConstraint] {
        &self.constraints
    }
}

impl fmt::Display for VersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.constraints.iter().map(|c| c.to_string()).collect();
        write!(f, "{}", parts.join(", "))
    }
}

impl FromStr for VersionReq {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Default for VersionReq {
    fn default() -> Self {
        Self::any()
    }
}

/// A single version constraint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VersionConstraint {
    /// Any version (*)
    Any,
    /// Exact match (=x.y.z)
    Exact(Version),
    /// Greater than (>x.y.z)
    Greater(Version),
    /// Greater than or equal (>=x.y.z)
    GreaterEq(Version),
    /// Less than (<x.y.z)
    Less(Version),
    /// Less than or equal (<=x.y.z)
    LessEq(Version),
    /// Caret (^x.y.z) - compatible updates
    Caret(Version),
    /// Tilde (~x.y.z) - approximately equal
    Tilde(Version),
}

impl VersionConstraint {
    /// Parse a constraint string
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        let s = s.trim();

        if s == "*" {
            return Ok(VersionConstraint::Any);
        }

        // Check for operators
        if let Some(version_str) = s.strip_prefix(">=") {
            let version = Version::parse(version_str.trim())?;
            return Ok(VersionConstraint::GreaterEq(version));
        }

        if let Some(version_str) = s.strip_prefix("<=") {
            let version = Version::parse(version_str.trim())?;
            return Ok(VersionConstraint::LessEq(version));
        }

        if let Some(version_str) = s.strip_prefix('>') {
            let version = Version::parse(version_str.trim())?;
            return Ok(VersionConstraint::Greater(version));
        }

        if let Some(version_str) = s.strip_prefix('<') {
            let version = Version::parse(version_str.trim())?;
            return Ok(VersionConstraint::Less(version));
        }

        if let Some(version_str) = s.strip_prefix('=') {
            let version = Version::parse(version_str.trim())?;
            return Ok(VersionConstraint::Exact(version));
        }

        if let Some(version_str) = s.strip_prefix('^') {
            let version = Version::parse(version_str.trim())?;
            return Ok(VersionConstraint::Caret(version));
        }

        if let Some(version_str) = s.strip_prefix('~') {
            let version = Version::parse(version_str.trim())?;
            return Ok(VersionConstraint::Tilde(version));
        }

        // Default: treat as caret requirement (like Cargo)
        let version = Version::parse(s)?;
        Ok(VersionConstraint::Caret(version))
    }

    /// Check if a version matches this constraint
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Any => true,
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::Greater(v) => version > v,
            VersionConstraint::GreaterEq(v) => version >= v,
            VersionConstraint::Less(v) => version < v,
            VersionConstraint::LessEq(v) => version <= v,
            VersionConstraint::Caret(v) => self.matches_caret(version, v),
            VersionConstraint::Tilde(v) => self.matches_tilde(version, v),
        }
    }

    /// Match caret requirement (^x.y.z)
    ///
    /// Allows changes that do not modify the left-most non-zero digit:
    /// - ^1.2.3 := >=1.2.3, <2.0.0
    /// - ^0.2.3 := >=0.2.3, <0.3.0
    /// - ^0.0.3 := >=0.0.3, <0.0.4
    fn matches_caret(&self, version: &Version, base: &Version) -> bool {
        if version < base {
            return false;
        }

        if base.major != 0 {
            // ^1.2.3 := >=1.2.3, <2.0.0
            version.major == base.major
        } else if base.minor != 0 {
            // ^0.2.3 := >=0.2.3, <0.3.0
            version.major == 0 && version.minor == base.minor
        } else {
            // ^0.0.3 := >=0.0.3, <0.0.4
            version.major == 0 && version.minor == 0 && version.patch == base.patch
        }
    }

    /// Match tilde requirement (~x.y.z)
    ///
    /// Allows patch-level changes:
    /// - ~1.2.3 := >=1.2.3, <1.3.0
    /// - ~1.2   := >=1.2.0, <1.3.0
    /// - ~1     := >=1.0.0, <2.0.0
    fn matches_tilde(&self, version: &Version, base: &Version) -> bool {
        if version < base {
            return false;
        }

        version.major == base.major && version.minor == base.minor
    }

    /// Get the minimum version that satisfies this constraint
    pub fn minimum_version(&self) -> Option<Version> {
        match self {
            VersionConstraint::Any => None,
            VersionConstraint::Exact(v) => Some(v.clone()),
            VersionConstraint::Greater(v) => Some(v.next_patch()),
            VersionConstraint::GreaterEq(v) => Some(v.clone()),
            VersionConstraint::Less(_) => Some(Version::new(0, 0, 0)),
            VersionConstraint::LessEq(_) => Some(Version::new(0, 0, 0)),
            VersionConstraint::Caret(v) => Some(v.clone()),
            VersionConstraint::Tilde(v) => Some(v.clone()),
        }
    }

    /// Get the maximum version that satisfies this constraint
    pub fn maximum_version(&self) -> Option<Version> {
        match self {
            VersionConstraint::Any => None,
            VersionConstraint::Exact(v) => Some(v.clone()),
            VersionConstraint::Greater(_) => None,
            VersionConstraint::GreaterEq(_) => None,
            VersionConstraint::Less(v) => Some(v.clone()),
            VersionConstraint::LessEq(v) => Some(v.clone()),
            VersionConstraint::Caret(v) => {
                if v.major != 0 {
                    Some(v.next_major())
                } else if v.minor != 0 {
                    Some(Version::new(0, v.minor + 1, 0))
                } else {
                    Some(Version::new(0, 0, v.patch + 1))
                }
            }
            VersionConstraint::Tilde(v) => Some(Version::new(v.major, v.minor + 1, 0)),
        }
    }
}

impl fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionConstraint::Any => write!(f, "*"),
            VersionConstraint::Exact(v) => write!(f, "={}", v),
            VersionConstraint::Greater(v) => write!(f, ">{}", v),
            VersionConstraint::GreaterEq(v) => write!(f, ">={}", v),
            VersionConstraint::Less(v) => write!(f, "<{}", v),
            VersionConstraint::LessEq(v) => write!(f, "<={}", v),
            VersionConstraint::Caret(v) => write!(f, "^{}", v),
            VersionConstraint::Tilde(v) => write!(f, "~{}", v),
        }
    }
}

/// Solver for version constraints
///
/// Given a set of available versions and requirements, find compatible versions.
#[derive(Debug, Clone)]
pub struct ConstraintSolver {
    available_versions: Vec<Version>,
}

impl ConstraintSolver {
    /// Create a new solver with available versions
    pub fn new(mut available_versions: Vec<Version>) -> Self {
        // Sort versions in descending order (newest first)
        available_versions.sort_by(|a, b| b.cmp(a));
        Self { available_versions }
    }

    /// Find the best (highest) version that satisfies the requirement
    pub fn solve(&self, req: &VersionReq) -> Option<&Version> {
        self.available_versions.iter().find(|v| req.matches(v))
    }

    /// Find all versions that satisfy the requirement
    pub fn solve_all(&self, req: &VersionReq) -> Vec<&Version> {
        self.available_versions.iter().filter(|v| req.matches(v)).collect()
    }

    /// Solve for multiple requirements (find a version that satisfies all)
    pub fn solve_multi(&self, reqs: &[&VersionReq]) -> Option<&Version> {
        self.available_versions.iter().find(|v| {
            reqs.iter().all(|req| req.matches(v))
        })
    }

    /// Check if a solution exists for the given requirements
    pub fn has_solution(&self, reqs: &[&VersionReq]) -> bool {
        self.solve_multi(reqs).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exact() {
        let req = VersionReq::parse("=1.0.0").unwrap();
        assert!(req.matches(&Version::parse("1.0.0").unwrap()));
        assert!(!req.matches(&Version::parse("1.0.1").unwrap()));
    }

    #[test]
    fn test_parse_range() {
        let req = VersionReq::parse(">=1.0.0, <2.0.0").unwrap();
        assert!(req.matches(&Version::parse("1.0.0").unwrap()));
        assert!(req.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!req.matches(&Version::parse("0.9.9").unwrap()));
    }

    #[test]
    fn test_caret() {
        // ^1.2.3 := >=1.2.3, <2.0.0
        let req = VersionReq::parse("^1.2.3").unwrap();
        assert!(req.matches(&Version::parse("1.2.3").unwrap()));
        assert!(req.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!req.matches(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_caret_zero_major() {
        // ^0.2.3 := >=0.2.3, <0.3.0
        let req = VersionReq::parse("^0.2.3").unwrap();
        assert!(req.matches(&Version::parse("0.2.3").unwrap()));
        assert!(req.matches(&Version::parse("0.2.9").unwrap()));
        assert!(!req.matches(&Version::parse("0.3.0").unwrap()));
    }

    #[test]
    fn test_caret_zero_minor() {
        // ^0.0.3 := >=0.0.3, <0.0.4
        let req = VersionReq::parse("^0.0.3").unwrap();
        assert!(req.matches(&Version::parse("0.0.3").unwrap()));
        assert!(!req.matches(&Version::parse("0.0.4").unwrap()));
    }

    #[test]
    fn test_tilde() {
        // ~1.2.3 := >=1.2.3, <1.3.0
        let req = VersionReq::parse("~1.2.3").unwrap();
        assert!(req.matches(&Version::parse("1.2.3").unwrap()));
        assert!(req.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!req.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn test_wildcard() {
        let req = VersionReq::parse("*").unwrap();
        assert!(req.matches(&Version::parse("0.0.0").unwrap()));
        assert!(req.matches(&Version::parse("999.999.999").unwrap()));
    }

    #[test]
    fn test_default_is_caret() {
        // Version without operator defaults to caret
        let req = VersionReq::parse("1.2.3").unwrap();
        let req_caret = VersionReq::parse("^1.2.3").unwrap();

        let v1 = Version::parse("1.2.3").unwrap();
        let v2 = Version::parse("1.9.9").unwrap();
        let v3 = Version::parse("2.0.0").unwrap();

        assert_eq!(req.matches(&v1), req_caret.matches(&v1));
        assert_eq!(req.matches(&v2), req_caret.matches(&v2));
        assert_eq!(req.matches(&v3), req_caret.matches(&v3));
    }

    #[test]
    fn test_solver() {
        let versions = vec![
            Version::parse("1.0.0").unwrap(),
            Version::parse("1.1.0").unwrap(),
            Version::parse("1.2.0").unwrap(),
            Version::parse("2.0.0").unwrap(),
        ];

        let solver = ConstraintSolver::new(versions);

        let req = VersionReq::parse("^1.0.0").unwrap();
        let solution = solver.solve(&req);
        assert_eq!(solution, Some(&Version::parse("1.2.0").unwrap()));
    }

    #[test]
    fn test_solver_multi() {
        let versions = vec![
            Version::parse("1.0.0").unwrap(),
            Version::parse("1.1.0").unwrap(),
            Version::parse("1.2.0").unwrap(),
            Version::parse("2.0.0").unwrap(),
        ];

        let solver = ConstraintSolver::new(versions);

        let req1 = VersionReq::parse(">=1.0.0").unwrap();
        let req2 = VersionReq::parse("<1.2.0").unwrap();

        let solution = solver.solve_multi(&[&req1, &req2]);
        assert_eq!(solution, Some(&Version::parse("1.1.0").unwrap()));
    }

    #[test]
    fn test_overlaps() {
        let req1 = VersionReq::parse(">=1.0.0, <2.0.0").unwrap();
        let req2 = VersionReq::parse(">=1.5.0, <3.0.0").unwrap();
        let req3 = VersionReq::parse(">=2.0.0").unwrap();

        assert!(req1.overlaps(&req2)); // 1.5.0-2.0.0 overlap
        assert!(!req1.overlaps(&req3)); // No overlap
    }
}
