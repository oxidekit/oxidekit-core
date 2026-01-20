//! Translation workflow management for teams
//!
//! Provides tools for coordinating translation work across team members:
//! - Key locking to prevent conflicts
//! - String freeze workflow
//! - Review and approval processes
//! - Diff and change tracking

pub mod locking;
pub mod freeze;
pub mod review;
pub mod diff;
pub mod conventions;

pub use locking::{KeyLock, LockManager, LockStatus, LockError};
pub use freeze::{StringFreeze, FreezePhase, FreezeStatus};
pub use review::{ReviewWorkflow, ReviewStatus, ReviewComment, Reviewer, ReviewerRole};
pub use diff::{TranslationDiff, DiffEntry, ChangeType};
pub use conventions::{KeyNamingConvention, ConventionChecker, ConventionViolation};
