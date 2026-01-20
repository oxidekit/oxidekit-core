//! Drag and drop functionality.
//!
//! Provides components and event handling for file and directory drag-and-drop
//! operations with visual feedback support.

use crate::types::{FileCategory, FileFilter, MimeType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// State of a drag-and-drop operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropState {
    /// No drag operation active
    Idle,
    /// Files are being dragged over the zone
    DragOver,
    /// Files are being dragged but not over the zone
    DragLeave,
    /// Files have been dropped
    Dropped,
    /// Drop operation was rejected
    Rejected,
}

impl Default for DropState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Information about a dropped file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroppedFile {
    /// Path to the file or directory
    pub path: PathBuf,
    /// Whether this is a directory
    pub is_directory: bool,
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// MIME type (if determinable)
    pub mime_type: Option<String>,
    /// File category
    pub category: FileCategory,
}

impl DroppedFile {
    /// Create a new dropped file from a path.
    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&path)?;
        let is_directory = metadata.is_dir();
        let size = if is_directory { 0 } else { metadata.len() };

        let mime_type = if is_directory {
            None
        } else {
            Some(MimeType::from_path(&path).mime)
        };

        let category = if is_directory {
            FileCategory::Folder
        } else {
            FileCategory::from_path(&path)
        };

        Ok(Self {
            path,
            is_directory,
            size,
            mime_type,
            category,
        })
    }

    /// Get the filename.
    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
    }

    /// Get the file extension.
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }

    /// Check if this file matches a filter.
    pub fn matches_filter(&self, filter: &FileFilter) -> bool {
        if self.is_directory {
            return false;
        }
        filter.matches(self.name())
    }

    /// Check if this file matches a MIME pattern.
    pub fn matches_mime_pattern(&self, pattern: &str) -> bool {
        if let Some(ref mime) = self.mime_type {
            let mime_type = MimeType::new(mime);
            mime_type.matches_pattern(pattern)
        } else {
            false
        }
    }
}

/// Event triggered during drag-and-drop operations.
#[derive(Debug, Clone)]
pub enum DropEvent {
    /// Drag entered the drop zone
    DragEnter {
        /// Position in the drop zone
        position: (f32, f32),
        /// Number of items being dragged
        item_count: usize,
    },
    /// Drag is moving over the drop zone
    DragOver {
        /// Current position
        position: (f32, f32),
    },
    /// Drag left the drop zone
    DragLeave,
    /// Files were dropped
    Drop {
        /// Dropped files
        files: Vec<DroppedFile>,
        /// Position where the drop occurred
        position: (f32, f32),
    },
}

/// Configuration for a drop zone.
#[derive(Debug, Clone)]
pub struct DropZoneConfig {
    /// Accepted MIME type patterns (e.g., "image/*", "application/pdf")
    pub accept_mime: Vec<String>,
    /// Accepted file filters
    pub accept_filters: Vec<FileFilter>,
    /// Whether to accept directories
    pub accept_directories: bool,
    /// Whether to accept multiple files
    pub accept_multiple: bool,
    /// Maximum file size in bytes (0 = no limit)
    pub max_file_size: u64,
    /// Maximum total size in bytes (0 = no limit)
    pub max_total_size: u64,
    /// Maximum number of files (0 = no limit)
    pub max_file_count: usize,
}

impl Default for DropZoneConfig {
    fn default() -> Self {
        Self {
            accept_mime: Vec::new(),
            accept_filters: Vec::new(),
            accept_directories: false,
            accept_multiple: true,
            max_file_size: 0,
            max_total_size: 0,
            max_file_count: 0,
        }
    }
}

impl DropZoneConfig {
    /// Create a new drop zone configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Accept files matching MIME patterns.
    pub fn accept(mut self, patterns: &[&str]) -> Self {
        self.accept_mime = patterns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Accept files matching filters.
    pub fn accept_filters(mut self, filters: &[FileFilter]) -> Self {
        self.accept_filters = filters.to_vec();
        self
    }

    /// Accept directories.
    pub fn accept_directories(mut self, accept: bool) -> Self {
        self.accept_directories = accept;
        self
    }

    /// Accept multiple files.
    pub fn accept_multiple(mut self, accept: bool) -> Self {
        self.accept_multiple = accept;
        self
    }

    /// Set maximum file size.
    pub fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set maximum total size.
    pub fn max_total_size(mut self, size: u64) -> Self {
        self.max_total_size = size;
        self
    }

    /// Set maximum file count.
    pub fn max_file_count(mut self, count: usize) -> Self {
        self.max_file_count = count;
        self
    }

    /// Check if a file is acceptable.
    pub fn accepts(&self, file: &DroppedFile) -> bool {
        // Check directory
        if file.is_directory {
            return self.accept_directories;
        }

        // Check file size
        if self.max_file_size > 0 && file.size > self.max_file_size {
            return false;
        }

        // Check MIME types
        if !self.accept_mime.is_empty() {
            let matches_mime = self.accept_mime.iter().any(|pattern| {
                file.matches_mime_pattern(pattern)
            });
            if !matches_mime {
                return false;
            }
        }

        // Check filters
        if !self.accept_filters.is_empty() {
            let matches_filter = self
                .accept_filters
                .iter()
                .any(|filter| file.matches_filter(filter));
            if !matches_filter {
                return false;
            }
        }

        true
    }

    /// Validate a list of dropped files.
    pub fn validate(&self, files: &[DroppedFile]) -> DropValidation {
        let mut result = DropValidation::default();

        // Check multiple files
        if !self.accept_multiple && files.len() > 1 {
            result.reject("Multiple files not allowed");
            return result;
        }

        // Check file count
        if self.max_file_count > 0 && files.len() > self.max_file_count {
            result.reject(&format!(
                "Too many files (max {})",
                self.max_file_count
            ));
            return result;
        }

        // Calculate total size
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        if self.max_total_size > 0 && total_size > self.max_total_size {
            result.reject("Total size exceeds limit");
            return result;
        }

        // Check each file
        for file in files {
            if self.accepts(file) {
                result.accepted.push(file.clone());
            } else {
                result.rejected.push(file.clone());
            }
        }

        if result.accepted.is_empty() && !files.is_empty() {
            result.reject("No acceptable files");
        }

        result
    }
}

/// Result of drop validation.
#[derive(Debug, Clone, Default)]
pub struct DropValidation {
    /// Accepted files
    pub accepted: Vec<DroppedFile>,
    /// Rejected files
    pub rejected: Vec<DroppedFile>,
    /// Rejection reason (if all rejected)
    pub rejection_reason: Option<String>,
}

impl DropValidation {
    /// Check if all files were accepted.
    pub fn is_fully_accepted(&self) -> bool {
        self.rejected.is_empty() && !self.accepted.is_empty()
    }

    /// Check if all files were rejected.
    pub fn is_fully_rejected(&self) -> bool {
        self.accepted.is_empty()
    }

    /// Check if some files were rejected.
    pub fn has_rejections(&self) -> bool {
        !self.rejected.is_empty()
    }

    /// Set rejection reason.
    fn reject(&mut self, reason: &str) {
        self.rejection_reason = Some(reason.to_string());
    }
}

/// A drop zone component that handles file drag-and-drop.
#[derive(Debug, Clone)]
pub struct DropZone {
    /// Unique identifier
    pub id: String,
    /// Configuration
    pub config: DropZoneConfig,
    /// Current state
    pub state: DropState,
    /// Label text for the drop zone
    pub label: Option<String>,
    /// Hint text shown when dragging
    pub hint: Option<String>,
    /// Icon name for the drop zone
    pub icon: Option<String>,
}

impl DropZone {
    /// Create a new drop zone.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            config: DropZoneConfig::default(),
            state: DropState::Idle,
            label: None,
            hint: None,
            icon: None,
        }
    }

    /// Set the configuration.
    pub fn config(mut self, config: DropZoneConfig) -> Self {
        self.config = config;
        self
    }

    /// Accept files matching MIME patterns.
    pub fn accept(mut self, patterns: &[&str]) -> Self {
        self.config.accept_mime = patterns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Accept files matching filters.
    pub fn accept_filters(mut self, filters: &[FileFilter]) -> Self {
        self.config.accept_filters = filters.to_vec();
        self
    }

    /// Accept directories.
    pub fn accept_directories(mut self, accept: bool) -> Self {
        self.config.accept_directories = accept;
        self
    }

    /// Set label text.
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set hint text.
    pub fn hint(mut self, text: impl Into<String>) -> Self {
        self.hint = Some(text.into());
        self
    }

    /// Set icon.
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }

    /// Handle a drag enter event.
    pub fn on_drag_enter(&mut self, position: (f32, f32), item_count: usize) -> DropEvent {
        self.state = DropState::DragOver;
        DropEvent::DragEnter {
            position,
            item_count,
        }
    }

    /// Handle a drag over event.
    pub fn on_drag_over(&mut self, position: (f32, f32)) -> DropEvent {
        self.state = DropState::DragOver;
        DropEvent::DragOver { position }
    }

    /// Handle a drag leave event.
    pub fn on_drag_leave(&mut self) -> DropEvent {
        self.state = DropState::DragLeave;
        DropEvent::DragLeave
    }

    /// Handle a drop event.
    pub fn on_drop(
        &mut self,
        paths: &[PathBuf],
        position: (f32, f32),
    ) -> Result<DropEvent, std::io::Error> {
        let files: Result<Vec<_>, _> = paths
            .iter()
            .map(|p| DroppedFile::from_path(p.clone()))
            .collect();
        let files = files?;

        let validation = self.config.validate(&files);

        if validation.is_fully_rejected() {
            self.state = DropState::Rejected;
        } else {
            self.state = DropState::Dropped;
        }

        Ok(DropEvent::Drop {
            files: validation.accepted,
            position,
        })
    }

    /// Reset the drop zone state.
    pub fn reset(&mut self) {
        self.state = DropState::Idle;
    }

    /// Check if the drop zone is active (being dragged over).
    pub fn is_active(&self) -> bool {
        self.state == DropState::DragOver
    }

    /// Get the current state.
    pub fn state(&self) -> DropState {
        self.state
    }
}

/// Style configuration for drop zone visual feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropZoneStyle {
    /// Background color in idle state
    pub idle_background: String,
    /// Border color in idle state
    pub idle_border: String,
    /// Background color when dragging over
    pub active_background: String,
    /// Border color when dragging over
    pub active_border: String,
    /// Background color for rejected drops
    pub rejected_background: String,
    /// Border color for rejected drops
    pub rejected_border: String,
    /// Border width
    pub border_width: f32,
    /// Border radius
    pub border_radius: f32,
    /// Border style (solid, dashed)
    pub border_style: String,
}

impl Default for DropZoneStyle {
    fn default() -> Self {
        Self {
            idle_background: "#f5f5f5".into(),
            idle_border: "#cccccc".into(),
            active_background: "#e3f2fd".into(),
            active_border: "#2196f3".into(),
            rejected_background: "#ffebee".into(),
            rejected_border: "#f44336".into(),
            border_width: 2.0,
            border_radius: 8.0,
            border_style: "dashed".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file() -> DroppedFile {
        DroppedFile {
            path: PathBuf::from("/test/image.png"),
            is_directory: false,
            size: 1024,
            mime_type: Some("image/png".to_string()),
            category: FileCategory::Image,
        }
    }

    #[test]
    fn test_dropped_file_name() {
        let file = create_test_file();
        assert_eq!(file.name(), "image.png");
        assert_eq!(file.extension(), Some("png"));
    }

    #[test]
    fn test_dropped_file_matches_filter() {
        let file = create_test_file();
        let images = FileFilter::images();
        let docs = FileFilter::documents();

        assert!(file.matches_filter(&images));
        assert!(!file.matches_filter(&docs));
    }

    #[test]
    fn test_dropped_file_matches_mime() {
        let file = create_test_file();
        assert!(file.matches_mime_pattern("image/*"));
        assert!(file.matches_mime_pattern("image/png"));
        assert!(!file.matches_mime_pattern("video/*"));
    }

    #[test]
    fn test_drop_zone_config_accepts() {
        let config = DropZoneConfig::new()
            .accept(&["image/*"])
            .max_file_size(10 * 1024);

        let file = create_test_file();
        assert!(config.accepts(&file));

        let large_file = DroppedFile {
            size: 20 * 1024,
            ..file.clone()
        };
        assert!(!config.accepts(&large_file));
    }

    #[test]
    fn test_drop_zone_config_directories() {
        let config = DropZoneConfig::new().accept_directories(true);

        let dir = DroppedFile {
            path: PathBuf::from("/test/dir"),
            is_directory: true,
            size: 0,
            mime_type: None,
            category: FileCategory::Folder,
        };
        assert!(config.accepts(&dir));

        let no_dir_config = DropZoneConfig::new().accept_directories(false);
        assert!(!no_dir_config.accepts(&dir));
    }

    #[test]
    fn test_drop_zone_validate() {
        let config = DropZoneConfig::new()
            .accept(&["image/*"])
            .max_file_count(2);

        let file = create_test_file();
        let validation = config.validate(&[file.clone()]);
        assert!(validation.is_fully_accepted());

        let files = vec![file.clone(), file.clone(), file.clone()];
        let validation = config.validate(&files);
        assert!(validation.is_fully_rejected());
    }

    #[test]
    fn test_drop_zone_events() {
        let mut zone = DropZone::new("test");

        zone.on_drag_enter((10.0, 10.0), 1);
        assert_eq!(zone.state(), DropState::DragOver);

        zone.on_drag_leave();
        assert_eq!(zone.state(), DropState::DragLeave);

        zone.reset();
        assert_eq!(zone.state(), DropState::Idle);
    }

    #[test]
    fn test_drop_zone_builder() {
        let zone = DropZone::new("upload")
            .accept(&["image/*", "application/pdf"])
            .accept_directories(false)
            .label("Drop files here")
            .hint("PNG, JPG, or PDF")
            .icon("upload");

        assert_eq!(zone.id, "upload");
        assert_eq!(zone.label.as_deref(), Some("Drop files here"));
        assert_eq!(zone.config.accept_mime.len(), 2);
    }

    #[test]
    fn test_drop_state_default() {
        let state = DropState::default();
        assert_eq!(state, DropState::Idle);
    }

    #[test]
    fn test_drop_zone_style_default() {
        let style = DropZoneStyle::default();
        assert_eq!(style.border_style, "dashed");
        assert!(style.border_width > 0.0);
    }

    #[test]
    fn test_drop_validation_partial() {
        let config = DropZoneConfig::new().accept(&["image/*"]);

        let image = create_test_file();
        let doc = DroppedFile {
            path: PathBuf::from("/test/doc.pdf"),
            is_directory: false,
            size: 1024,
            mime_type: Some("application/pdf".to_string()),
            category: FileCategory::Pdf,
        };

        let validation = config.validate(&[image, doc]);
        assert!(validation.has_rejections());
        assert!(!validation.is_fully_accepted());
        assert!(!validation.is_fully_rejected());
        assert_eq!(validation.accepted.len(), 1);
        assert_eq!(validation.rejected.len(), 1);
    }
}
