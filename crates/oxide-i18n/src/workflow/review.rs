//! Translation review workflow
//!
//! Manages the review and approval process for translations:
//! - Submit translations for review
//! - Reviewer comments and feedback
//! - Approval workflow
//! - Quality checks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

use crate::formats::TranslationState;

/// Error type for review operations
#[derive(Debug, Error)]
pub enum ReviewError {
    /// Review not found
    #[error("Review not found: {0}")]
    NotFound(Uuid),

    /// Invalid state transition
    #[error("Cannot transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: ReviewStatus,
        to: ReviewStatus,
    },

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type for review operations
pub type ReviewResult<T> = Result<T, ReviewError>;

/// Status of a review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    /// Waiting for review
    Pending,
    /// Review in progress
    InReview,
    /// Changes requested
    ChangesRequested,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
}

impl ReviewStatus {
    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, ReviewStatus::Approved | ReviewStatus::Rejected)
    }

    /// Get valid next states
    pub fn valid_transitions(&self) -> Vec<ReviewStatus> {
        match self {
            ReviewStatus::Pending => vec![ReviewStatus::InReview, ReviewStatus::Rejected],
            ReviewStatus::InReview => vec![
                ReviewStatus::Approved,
                ReviewStatus::ChangesRequested,
                ReviewStatus::Rejected,
            ],
            ReviewStatus::ChangesRequested => vec![
                ReviewStatus::InReview,
                ReviewStatus::Approved,
                ReviewStatus::Rejected,
            ],
            ReviewStatus::Approved => vec![ReviewStatus::InReview], // Re-open if needed
            ReviewStatus::Rejected => vec![ReviewStatus::Pending],  // Resubmit
        }
    }
}

/// A reviewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reviewer {
    /// Reviewer's username
    pub username: String,
    /// Reviewer's email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Languages the reviewer can review
    pub languages: Vec<String>,
    /// Reviewer's role (translator, reviewer, admin)
    pub role: ReviewerRole,
}

/// Role of a reviewer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerRole {
    /// Can submit translations
    Translator,
    /// Can review and approve translations
    Reviewer,
    /// Can do everything including admin tasks
    Admin,
}

impl ReviewerRole {
    /// Check if this role can approve reviews
    pub fn can_approve(&self) -> bool {
        matches!(self, ReviewerRole::Reviewer | ReviewerRole::Admin)
    }

    /// Check if this role can submit translations
    pub fn can_translate(&self) -> bool {
        true // All roles can translate
    }
}

/// A comment on a review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    /// Comment ID
    pub id: Uuid,
    /// Who made the comment
    pub author: String,
    /// When the comment was made
    pub timestamp: DateTime<Utc>,
    /// The comment text
    pub content: String,
    /// Reference to specific key (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Is this resolved?
    #[serde(default)]
    pub resolved: bool,
}

impl ReviewComment {
    /// Create a new comment
    pub fn new(author: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            author: author.into(),
            timestamp: Utc::now(),
            content: content.into(),
            key: None,
            resolved: false,
        }
    }

    /// Create a comment for a specific key
    pub fn for_key(
        author: impl Into<String>,
        key: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            author: author.into(),
            timestamp: Utc::now(),
            content: content.into(),
            key: Some(key.into()),
            resolved: false,
        }
    }
}

/// A translation review request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationReview {
    /// Review ID
    pub id: Uuid,
    /// Target locale
    pub locale: String,
    /// Keys included in this review
    pub keys: Vec<String>,
    /// Who submitted the review
    pub submitter: String,
    /// When the review was submitted
    pub submitted_at: DateTime<Utc>,
    /// Current status
    pub status: ReviewStatus,
    /// Assigned reviewer (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<String>,
    /// Review comments
    #[serde(default)]
    pub comments: Vec<ReviewComment>,
    /// When the review was completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    /// Status history
    #[serde(default)]
    pub history: Vec<StatusChange>,
}

/// A status change in the review history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChange {
    /// Previous status
    pub from: ReviewStatus,
    /// New status
    pub to: ReviewStatus,
    /// When the change occurred
    pub timestamp: DateTime<Utc>,
    /// Who made the change
    pub changed_by: String,
    /// Optional note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl TranslationReview {
    /// Create a new review request
    pub fn new(
        locale: impl Into<String>,
        keys: Vec<String>,
        submitter: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            locale: locale.into(),
            keys,
            submitter: submitter.into(),
            submitted_at: Utc::now(),
            status: ReviewStatus::Pending,
            reviewer: None,
            comments: Vec::new(),
            completed_at: None,
            history: Vec::new(),
        }
    }

    /// Add a comment
    pub fn add_comment(&mut self, comment: ReviewComment) {
        self.comments.push(comment);
    }

    /// Get unresolved comments
    pub fn unresolved_comments(&self) -> Vec<&ReviewComment> {
        self.comments.iter().filter(|c| !c.resolved).collect()
    }

    /// Resolve a comment
    pub fn resolve_comment(&mut self, comment_id: Uuid) -> bool {
        if let Some(comment) = self.comments.iter_mut().find(|c| c.id == comment_id) {
            comment.resolved = true;
            true
        } else {
            false
        }
    }
}

/// Manager for translation reviews
#[derive(Debug)]
pub struct ReviewWorkflow {
    /// Path to reviews directory
    reviews_dir: PathBuf,
    /// Current user
    current_user: String,
    /// User's role
    user_role: ReviewerRole,
    /// Cached reviews
    reviews: HashMap<Uuid, TranslationReview>,
}

impl ReviewWorkflow {
    /// Create a new review workflow manager
    pub fn new(
        reviews_dir: impl Into<PathBuf>,
        current_user: impl Into<String>,
        user_role: ReviewerRole,
    ) -> ReviewResult<Self> {
        let reviews_dir = reviews_dir.into();
        fs::create_dir_all(&reviews_dir)?;

        let mut workflow = Self {
            reviews_dir,
            current_user: current_user.into(),
            user_role,
            reviews: HashMap::new(),
        };

        workflow.load_all()?;
        Ok(workflow)
    }

    /// Submit a new review request
    pub fn submit_review(
        &mut self,
        locale: impl Into<String>,
        keys: Vec<String>,
    ) -> ReviewResult<Uuid> {
        let review = TranslationReview::new(locale, keys, &self.current_user);
        let id = review.id;

        self.save_review(&review)?;
        self.reviews.insert(id, review);

        Ok(id)
    }

    /// Get a review by ID
    pub fn get_review(&self, id: Uuid) -> Option<&TranslationReview> {
        self.reviews.get(&id)
    }

    /// Get a mutable review by ID
    pub fn get_review_mut(&mut self, id: Uuid) -> Option<&mut TranslationReview> {
        self.reviews.get_mut(&id)
    }

    /// Start reviewing (claim the review)
    pub fn start_review(&mut self, id: Uuid) -> ReviewResult<()> {
        if !self.user_role.can_approve() {
            return Err(ReviewError::PermissionDenied(
                "Only reviewers can claim reviews".to_string(),
            ));
        }

        // First, perform validation and mutation
        {
            let review = self
                .reviews
                .get_mut(&id)
                .ok_or(ReviewError::NotFound(id))?;

            Self::do_transition_status(review, ReviewStatus::InReview, &self.current_user)?;
            review.reviewer = Some(self.current_user.clone());
        }

        // Then save (requires separate borrow)
        let review = self.reviews.get(&id).unwrap();
        self.save_review(review)
    }

    /// Add a comment to a review
    pub fn add_comment(&mut self, id: Uuid, comment: ReviewComment) -> ReviewResult<()> {
        {
            let review = self
                .reviews
                .get_mut(&id)
                .ok_or(ReviewError::NotFound(id))?;

            review.add_comment(comment);
        }

        let review = self.reviews.get(&id).unwrap();
        self.save_review(review)
    }

    /// Request changes
    pub fn request_changes(&mut self, id: Uuid, comment: String) -> ReviewResult<()> {
        if !self.user_role.can_approve() {
            return Err(ReviewError::PermissionDenied(
                "Only reviewers can request changes".to_string(),
            ));
        }

        {
            let current_user = self.current_user.clone();
            let review = self
                .reviews
                .get_mut(&id)
                .ok_or(ReviewError::NotFound(id))?;

            Self::do_transition_status(review, ReviewStatus::ChangesRequested, &current_user)?;
            review.add_comment(ReviewComment::new(&current_user, comment));
        }

        let review = self.reviews.get(&id).unwrap();
        self.save_review(review)
    }

    /// Approve the review
    pub fn approve(&mut self, id: Uuid, comment: Option<String>) -> ReviewResult<()> {
        if !self.user_role.can_approve() {
            return Err(ReviewError::PermissionDenied(
                "Only reviewers can approve".to_string(),
            ));
        }

        {
            let current_user = self.current_user.clone();
            let review = self
                .reviews
                .get_mut(&id)
                .ok_or(ReviewError::NotFound(id))?;

            Self::do_transition_status(review, ReviewStatus::Approved, &current_user)?;
            review.completed_at = Some(Utc::now());

            if let Some(comment) = comment {
                review.add_comment(ReviewComment::new(&current_user, comment));
            }
        }

        let review = self.reviews.get(&id).unwrap();
        self.save_review(review)
    }

    /// Reject the review
    pub fn reject(&mut self, id: Uuid, reason: String) -> ReviewResult<()> {
        if !self.user_role.can_approve() {
            return Err(ReviewError::PermissionDenied(
                "Only reviewers can reject".to_string(),
            ));
        }

        {
            let current_user = self.current_user.clone();
            let review = self
                .reviews
                .get_mut(&id)
                .ok_or(ReviewError::NotFound(id))?;

            Self::do_transition_status(review, ReviewStatus::Rejected, &current_user)?;
            review.completed_at = Some(Utc::now());
            review.add_comment(ReviewComment::new(&current_user, reason));
        }

        let review = self.reviews.get(&id).unwrap();
        self.save_review(review)
    }

    /// List pending reviews
    pub fn pending_reviews(&self) -> Vec<&TranslationReview> {
        self.reviews
            .values()
            .filter(|r| r.status == ReviewStatus::Pending)
            .collect()
    }

    /// List reviews assigned to current user
    pub fn my_reviews(&self) -> Vec<&TranslationReview> {
        self.reviews
            .values()
            .filter(|r| r.reviewer.as_ref() == Some(&self.current_user))
            .collect()
    }

    /// List reviews submitted by current user
    pub fn my_submissions(&self) -> Vec<&TranslationReview> {
        self.reviews
            .values()
            .filter(|r| r.submitter == self.current_user)
            .collect()
    }

    /// List reviews for a locale
    pub fn reviews_for_locale(&self, locale: &str) -> Vec<&TranslationReview> {
        self.reviews
            .values()
            .filter(|r| r.locale == locale)
            .collect()
    }

    /// Transition status with validation (static method to avoid borrow issues)
    fn do_transition_status(
        review: &mut TranslationReview,
        new_status: ReviewStatus,
        changed_by: &str,
    ) -> ReviewResult<()> {
        let valid = review.status.valid_transitions();
        if !valid.contains(&new_status) {
            return Err(ReviewError::InvalidTransition {
                from: review.status,
                to: new_status,
            });
        }

        review.history.push(StatusChange {
            from: review.status,
            to: new_status,
            timestamp: Utc::now(),
            changed_by: changed_by.to_string(),
            note: None,
        });

        review.status = new_status;
        Ok(())
    }

    /// Save a review to file
    fn save_review(&self, review: &TranslationReview) -> ReviewResult<()> {
        let path = self.reviews_dir.join(format!("{}.json", review.id));
        let content = serde_json::to_string_pretty(review)
            .map_err(|e| ReviewError::Serialization(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load all reviews from directory
    fn load_all(&mut self) -> ReviewResult<()> {
        for entry in fs::read_dir(&self.reviews_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "json") {
                let content = fs::read_to_string(&path)?;
                let review: TranslationReview = serde_json::from_str(&content)
                    .map_err(|e| ReviewError::Serialization(e.to_string()))?;
                self.reviews.insert(review.id, review);
            }
        }
        Ok(())
    }
}

/// Conversion to translation state
impl From<ReviewStatus> for TranslationState {
    fn from(status: ReviewStatus) -> Self {
        match status {
            ReviewStatus::Pending | ReviewStatus::InReview => TranslationState::NeedsReview,
            ReviewStatus::ChangesRequested => TranslationState::InProgress,
            ReviewStatus::Approved => TranslationState::Approved,
            ReviewStatus::Rejected => TranslationState::New,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_review_workflow() {
        let dir = tempdir().unwrap();
        let reviews_dir = dir.path().join("reviews");

        // Translator submits review
        let mut translator = ReviewWorkflow::new(&reviews_dir, "alice", ReviewerRole::Translator).unwrap();
        let review_id = translator.submit_review("de", vec!["auth.login".to_string()]).unwrap();

        // Reviewer claims and reviews
        let mut reviewer = ReviewWorkflow::new(&reviews_dir, "bob", ReviewerRole::Reviewer).unwrap();
        reviewer.start_review(review_id).unwrap();

        let review = reviewer.get_review(review_id).unwrap();
        assert_eq!(review.status, ReviewStatus::InReview);
        assert_eq!(review.reviewer, Some("bob".to_string()));

        // Reviewer approves
        reviewer.approve(review_id, Some("Looks good!".to_string())).unwrap();

        let review = reviewer.get_review(review_id).unwrap();
        assert_eq!(review.status, ReviewStatus::Approved);
        assert!(review.completed_at.is_some());
    }

    #[test]
    fn test_review_status_transitions() {
        assert!(ReviewStatus::Pending.valid_transitions().contains(&ReviewStatus::InReview));
        assert!(!ReviewStatus::Pending.valid_transitions().contains(&ReviewStatus::Approved));
        assert!(ReviewStatus::InReview.valid_transitions().contains(&ReviewStatus::Approved));
    }
}
