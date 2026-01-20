//! Patch System
//!
//! Provides structured patches for changes, undo/redo history,
//! and patch-to-source functionality for persisting changes to files.

use crate::editor::{PendingChange, StyleValueChange};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

/// Error types for patch operations
#[derive(Debug, Error)]
pub enum PatchError {
    #[error("Patch not found: {0}")]
    NotFound(String),
    #[error("Invalid patch format: {0}")]
    InvalidFormat(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// A patch representing a set of changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditPatch {
    /// Unique patch ID
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Individual operations in this patch
    pub operations: Vec<PatchOperation>,
    /// Timestamp when patch was created
    pub timestamp: DateTime<Utc>,
    /// Session ID that created this patch
    pub session_id: Option<String>,
    /// Source files affected
    pub affected_files: Vec<String>,
    /// Whether this patch has been applied to source
    pub persisted: bool,
}

impl EditPatch {
    /// Create a new empty patch
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            operations: Vec::new(),
            timestamp: Utc::now(),
            session_id: None,
            affected_files: Vec::new(),
            persisted: false,
        }
    }

    /// Create a patch from pending changes
    pub fn from_changes(changes: &[PendingChange], description: &str) -> Self {
        let operations: Vec<PatchOperation> = changes
            .iter()
            .map(|c| PatchOperation {
                component_id: c.handle.to_string(),
                property: c.property.clone(),
                old_value: None, // Would need to track old values
                new_value: Some(c.value.clone()),
                operation_type: OperationType::Modify,
            })
            .collect();

        Self {
            id: Uuid::new_v4().to_string(),
            description: description.to_string(),
            operations,
            timestamp: Utc::now(),
            session_id: None,
            affected_files: Vec::new(),
            persisted: false,
        }
    }

    /// Add an operation to the patch
    pub fn add_operation(&mut self, op: PatchOperation) {
        self.operations.push(op);
    }

    /// Check if patch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Get operation count
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Mark as persisted
    pub fn mark_persisted(&mut self) {
        self.persisted = true;
    }

    /// Create inverse patch (for undo)
    pub fn inverse(&self) -> Self {
        let operations: Vec<PatchOperation> = self
            .operations
            .iter()
            .map(|op| PatchOperation {
                component_id: op.component_id.clone(),
                property: op.property.clone(),
                old_value: op.new_value.clone(),
                new_value: op.old_value.clone(),
                operation_type: match op.operation_type {
                    OperationType::Add => OperationType::Remove,
                    OperationType::Remove => OperationType::Add,
                    OperationType::Modify => OperationType::Modify,
                },
            })
            .collect();

        Self {
            id: Uuid::new_v4().to_string(),
            description: format!("Undo: {}", self.description),
            operations,
            timestamp: Utc::now(),
            session_id: self.session_id.clone(),
            affected_files: self.affected_files.clone(),
            persisted: false,
        }
    }

    /// Export patch as JSON
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Import patch from JSON
    pub fn from_json(json: &serde_json::Value) -> Result<Self, PatchError> {
        serde_json::from_value(json.clone())
            .map_err(|e| PatchError::ParseError(e.to_string()))
    }
}

/// A single operation in a patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOperation {
    /// Component instance ID
    pub component_id: String,
    /// Property being changed
    pub property: String,
    /// Old value (for undo)
    pub old_value: Option<StyleValueChange>,
    /// New value
    pub new_value: Option<StyleValueChange>,
    /// Type of operation
    pub operation_type: OperationType,
}

impl PatchOperation {
    /// Create a modify operation
    pub fn modify(
        component_id: impl Into<String>,
        property: impl Into<String>,
        old_value: Option<StyleValueChange>,
        new_value: StyleValueChange,
    ) -> Self {
        Self {
            component_id: component_id.into(),
            property: property.into(),
            old_value,
            new_value: Some(new_value),
            operation_type: OperationType::Modify,
        }
    }

    /// Create an add operation
    pub fn add(
        component_id: impl Into<String>,
        property: impl Into<String>,
        value: StyleValueChange,
    ) -> Self {
        Self {
            component_id: component_id.into(),
            property: property.into(),
            old_value: None,
            new_value: Some(value),
            operation_type: OperationType::Add,
        }
    }

    /// Create a remove operation
    pub fn remove(
        component_id: impl Into<String>,
        property: impl Into<String>,
        old_value: StyleValueChange,
    ) -> Self {
        Self {
            component_id: component_id.into(),
            property: property.into(),
            old_value: Some(old_value),
            new_value: None,
            operation_type: OperationType::Remove,
        }
    }
}

/// Operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Add a new property
    Add,
    /// Remove a property
    Remove,
    /// Modify existing property
    Modify,
}

/// Undo/redo history manager
#[derive(Debug, Default)]
pub struct PatchHistory {
    /// Undo stack
    undo_stack: Vec<EditPatch>,
    /// Redo stack
    redo_stack: Vec<EditPatch>,
    /// Maximum history size
    max_size: usize,
}

impl PatchHistory {
    /// Create a new history manager
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size: 100,
        }
    }

    /// Create with custom max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_size),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// Push a new patch (clears redo stack)
    pub fn push(&mut self, patch: EditPatch) {
        self.undo_stack.push(patch);
        self.redo_stack.clear();

        // Trim if over max size
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last patch
    pub fn undo(&mut self) -> Option<EditPatch> {
        let patch = self.undo_stack.pop()?;
        self.redo_stack.push(patch.clone());
        Some(patch)
    }

    /// Redo the last undone patch
    pub fn redo(&mut self) -> Option<EditPatch> {
        let patch = self.redo_stack.pop()?;
        self.undo_stack.push(patch.clone());
        Some(patch)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get undo stack size
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get last patch (peek)
    pub fn last(&self) -> Option<&EditPatch> {
        self.undo_stack.last()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get all patches (for export)
    pub fn all_patches(&self) -> Vec<&EditPatch> {
        self.undo_stack.iter().collect()
    }
}

/// Source file patcher for writing changes back to .oui files
#[derive(Debug)]
pub struct SourcePatcher {
    /// Project root directory
    project_root: PathBuf,
    /// Pending file changes
    pending_writes: HashMap<PathBuf, String>,
    /// Backup of original files
    backups: HashMap<PathBuf, String>,
}

impl SourcePatcher {
    /// Create a new source patcher
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            pending_writes: HashMap::new(),
            backups: HashMap::new(),
        }
    }

    /// Apply a patch to source files
    pub fn apply_to_source(&mut self, patch: &EditPatch) -> Result<Vec<SourceChange>, PatchError> {
        let mut changes = Vec::new();

        // Group operations by file
        // In a real implementation, we'd need source location info to map
        // component IDs back to files

        for op in &patch.operations {
            // This is a simplified implementation
            // Real implementation would parse .oui files and apply changes
            let change = SourceChange {
                file: "ui/app.oui".to_string(), // Would be resolved from component
                line: 0,
                column: 0,
                old_text: String::new(),
                new_text: self.format_value_for_source(&op.new_value),
                property: op.property.clone(),
            };
            changes.push(change);
        }

        Ok(changes)
    }

    /// Format a style value for source file output
    fn format_value_for_source(&self, value: &Option<StyleValueChange>) -> String {
        match value {
            Some(StyleValueChange::Color(c)) => format!("\"{}\"", c),
            Some(StyleValueChange::Number { value, unit }) => {
                if let Some(u) = unit {
                    format!("{}{}", value, u)
                } else {
                    format!("{}", value)
                }
            }
            Some(StyleValueChange::String(s)) => format!("\"{}\"", s),
            Some(StyleValueChange::Bool(b)) => format!("{}", b),
            Some(StyleValueChange::Token(t)) => format!("${}", t),
            Some(StyleValueChange::Enum(e)) => format!("{}", e),
            Some(StyleValueChange::Unset) | None => String::new(),
        }
    }

    /// Preview changes without applying
    pub fn preview(&self, patch: &EditPatch) -> Vec<SourceChangePreview> {
        patch
            .operations
            .iter()
            .map(|op| SourceChangePreview {
                component_id: op.component_id.clone(),
                property: op.property.clone(),
                old_value: op.old_value.as_ref().map(|v| format!("{:?}", v)),
                new_value: op.new_value.as_ref().map(|v| format!("{:?}", v)),
            })
            .collect()
    }

    /// Write pending changes to files
    pub fn flush(&mut self) -> Result<(), PatchError> {
        for (path, content) in self.pending_writes.drain() {
            // Backup original
            if !self.backups.contains_key(&path) {
                if let Ok(original) = std::fs::read_to_string(&path) {
                    self.backups.insert(path.clone(), original);
                }
            }

            std::fs::write(&path, content)?;
        }
        Ok(())
    }

    /// Restore from backups
    pub fn restore(&mut self) -> Result<(), PatchError> {
        for (path, content) in &self.backups {
            std::fs::write(path, content)?;
        }
        self.pending_writes.clear();
        Ok(())
    }
}

/// A change to a source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceChange {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
    /// Original text
    pub old_text: String,
    /// New text
    pub new_text: String,
    /// Property being changed
    pub property: String,
}

/// Preview of a source change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceChangePreview {
    /// Component ID
    pub component_id: String,
    /// Property name
    pub property: String,
    /// Old value (formatted)
    pub old_value: Option<String>,
    /// New value (formatted)
    pub new_value: Option<String>,
}

/// Token-level change for theme modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenChange {
    /// Token path (e.g., "color.primary")
    pub path: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: String,
    /// Category (color, spacing, etc.)
    pub category: String,
}

/// Patch bundle for exporting/importing multiple patches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchBundle {
    /// Bundle version
    pub version: String,
    /// Patches in order
    pub patches: Vec<EditPatch>,
    /// Bundle metadata
    pub metadata: HashMap<String, String>,
}

impl PatchBundle {
    /// Create a new bundle
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            patches: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a patch
    pub fn add(&mut self, patch: EditPatch) {
        self.patches.push(patch);
    }

    /// Export to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Import from JSON
    pub fn from_json(json: &str) -> Result<Self, PatchError> {
        serde_json::from_str(json).map_err(|e| PatchError::ParseError(e.to_string()))
    }
}

impl Default for PatchBundle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_creation() {
        let mut patch = EditPatch::new("Test patch");
        patch.add_operation(PatchOperation::modify(
            "component-1",
            "color",
            None,
            StyleValueChange::color("#FF0000"),
        ));

        assert_eq!(patch.len(), 1);
        assert!(!patch.is_empty());
    }

    #[test]
    fn test_patch_inverse() {
        let mut patch = EditPatch::new("Original");
        patch.add_operation(PatchOperation::modify(
            "component-1",
            "color",
            Some(StyleValueChange::color("#000000")),
            StyleValueChange::color("#FF0000"),
        ));

        let inverse = patch.inverse();
        assert!(inverse.description.starts_with("Undo:"));

        let inv_op = &inverse.operations[0];
        assert!(matches!(&inv_op.new_value, Some(StyleValueChange::Color(c)) if c == "#000000"));
    }

    #[test]
    fn test_history_undo_redo() {
        let mut history = PatchHistory::new();

        let patch1 = EditPatch::new("Patch 1");
        let patch2 = EditPatch::new("Patch 2");

        history.push(patch1);
        history.push(patch2);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undone = history.undo().unwrap();
        assert_eq!(undone.description, "Patch 2");
        assert!(history.can_redo());

        let redone = history.redo().unwrap();
        assert_eq!(redone.description, "Patch 2");
    }

    #[test]
    fn test_history_push_clears_redo() {
        let mut history = PatchHistory::new();

        history.push(EditPatch::new("Patch 1"));
        history.push(EditPatch::new("Patch 2"));
        history.undo();
        assert!(history.can_redo());

        history.push(EditPatch::new("Patch 3"));
        assert!(!history.can_redo());
    }

    #[test]
    fn test_patch_bundle() {
        let mut bundle = PatchBundle::new();
        bundle.add(EditPatch::new("Patch 1"));
        bundle.add(EditPatch::new("Patch 2"));

        let json = bundle.to_json();
        assert!(json.contains("Patch 1"));
        assert!(json.contains("Patch 2"));

        let loaded = PatchBundle::from_json(&json).unwrap();
        assert_eq!(loaded.patches.len(), 2);
    }

    #[test]
    fn test_source_change_preview() {
        let patcher = SourcePatcher::new("/project");

        let mut patch = EditPatch::new("Test");
        patch.add_operation(PatchOperation::modify(
            "btn-1",
            "color",
            Some(StyleValueChange::color("#000")),
            StyleValueChange::color("#FFF"),
        ));

        let previews = patcher.preview(&patch);
        assert_eq!(previews.len(), 1);
        assert_eq!(previews[0].property, "color");
    }
}
