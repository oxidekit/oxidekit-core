//! File open dialog implementation.
//!
//! Provides a cross-platform file open dialog with support for single/multiple
//! file selection, file type filtering, and initial directory configuration.

use crate::types::FileFilter;
use crate::FilePickerError;
use std::path::PathBuf;

/// Builder for configuring and showing a file open dialog.
///
/// # Example
///
/// ```rust,ignore
/// use oxide_file_picker::{OpenDialog, FileFilter};
///
/// // Single file selection
/// let file = OpenDialog::new()
///     .title("Select an image")
///     .filter(FileFilter::images())
///     .pick()
///     .await?;
///
/// // Multiple file selection
/// let files = OpenDialog::new()
///     .title("Select files")
///     .multiple(true)
///     .filters(&[
///         FileFilter::images(),
///         FileFilter::documents(),
///     ])
///     .pick_multiple()
///     .await?;
/// ```
#[derive(Debug, Clone)]
pub struct OpenDialog {
    /// Dialog title
    pub(crate) title: Option<String>,
    /// File type filters
    pub(crate) filters: Vec<FileFilter>,
    /// Allow multiple file selection
    pub(crate) multiple: bool,
    /// Initial directory to open in
    pub(crate) initial_dir: Option<PathBuf>,
    /// Initial filename (for display)
    pub(crate) initial_name: Option<String>,
    /// Whether to show hidden files
    pub(crate) show_hidden: bool,
    /// Custom button text
    pub(crate) button_text: Option<String>,
    /// Parent window handle (platform-specific)
    pub(crate) parent_window: Option<usize>,
    /// Whether the dialog can select packages/bundles on macOS
    pub(crate) can_select_packages: bool,
}

impl Default for OpenDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenDialog {
    /// Create a new file open dialog builder.
    pub fn new() -> Self {
        Self {
            title: None,
            filters: Vec::new(),
            multiple: false,
            initial_dir: None,
            initial_name: None,
            show_hidden: false,
            button_text: None,
            parent_window: None,
            can_select_packages: false,
        }
    }

    /// Set the dialog title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add a single file filter.
    pub fn filter(mut self, filter: FileFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Add multiple file filters.
    pub fn filters(mut self, filters: &[FileFilter]) -> Self {
        self.filters.extend(filters.iter().cloned());
        self
    }

    /// Enable or disable multiple file selection.
    pub fn multiple(mut self, allow: bool) -> Self {
        self.multiple = allow;
        self
    }

    /// Set the initial directory.
    pub fn initial_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.initial_dir = Some(dir.into());
        self
    }

    /// Set an initial filename to display.
    pub fn initial_name(mut self, name: impl Into<String>) -> Self {
        self.initial_name = Some(name.into());
        self
    }

    /// Show hidden files in the dialog.
    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    /// Set custom button text (e.g., "Select" instead of "Open").
    pub fn button_text(mut self, text: impl Into<String>) -> Self {
        self.button_text = Some(text.into());
        self
    }

    /// Set the parent window handle for modal behavior.
    pub fn parent(mut self, handle: usize) -> Self {
        self.parent_window = Some(handle);
        self
    }

    /// Allow selecting macOS packages/bundles as single files.
    pub fn can_select_packages(mut self, allow: bool) -> Self {
        self.can_select_packages = allow;
        self
    }

    /// Show the dialog and pick a single file.
    ///
    /// Returns `None` if the user cancelled the dialog.
    #[cfg(feature = "async-ops")]
    pub async fn pick(self) -> Result<Option<PathBuf>, FilePickerError> {
        let result = tokio::task::spawn_blocking(move || self.pick_sync()).await?;
        result
    }

    /// Show the dialog and pick multiple files.
    ///
    /// Returns an empty vector if the user cancelled the dialog.
    #[cfg(feature = "async-ops")]
    pub async fn pick_multiple(mut self) -> Result<Vec<PathBuf>, FilePickerError> {
        self.multiple = true;
        let result = tokio::task::spawn_blocking(move || self.pick_multiple_sync()).await?;
        result
    }

    /// Synchronously show the dialog and pick a single file.
    pub fn pick_sync(self) -> Result<Option<PathBuf>, FilePickerError> {
        crate::platform::show_open_dialog(&self, false)
            .map(|paths| paths.into_iter().next())
    }

    /// Synchronously show the dialog and pick multiple files.
    pub fn pick_multiple_sync(mut self) -> Result<Vec<PathBuf>, FilePickerError> {
        self.multiple = true;
        crate::platform::show_open_dialog(&self, true)
    }
}

/// Result of a file open operation.
#[derive(Debug, Clone)]
pub struct OpenResult {
    /// Selected file paths.
    pub paths: Vec<PathBuf>,
    /// The filter index that was selected (0-based).
    pub filter_index: Option<usize>,
}

impl OpenResult {
    /// Create a new open result.
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            paths,
            filter_index: None,
        }
    }

    /// Create a result with a selected filter.
    pub fn with_filter(paths: Vec<PathBuf>, filter_index: usize) -> Self {
        Self {
            paths,
            filter_index: Some(filter_index),
        }
    }

    /// Check if the dialog was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.paths.is_empty()
    }

    /// Get the first selected path.
    pub fn first(&self) -> Option<&PathBuf> {
        self.paths.first()
    }

    /// Get all selected paths.
    pub fn all(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Take ownership of the paths.
    pub fn into_paths(self) -> Vec<PathBuf> {
        self.paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_dialog_builder() {
        let dialog = OpenDialog::new()
            .title("Test Dialog")
            .filter(FileFilter::images())
            .multiple(true)
            .initial_dir("/home/user")
            .show_hidden(true);

        assert_eq!(dialog.title.as_deref(), Some("Test Dialog"));
        assert_eq!(dialog.filters.len(), 1);
        assert!(dialog.multiple);
        assert_eq!(
            dialog.initial_dir,
            Some(PathBuf::from("/home/user"))
        );
        assert!(dialog.show_hidden);
    }

    #[test]
    fn test_open_dialog_multiple_filters() {
        let dialog = OpenDialog::new()
            .filters(&[FileFilter::images(), FileFilter::documents()])
            .filter(FileFilter::code());

        assert_eq!(dialog.filters.len(), 3);
    }

    #[test]
    fn test_open_result() {
        let result = OpenResult::new(vec![PathBuf::from("/file1.txt")]);
        assert!(!result.is_cancelled());
        assert_eq!(result.first(), Some(&PathBuf::from("/file1.txt")));

        let cancelled = OpenResult::new(vec![]);
        assert!(cancelled.is_cancelled());
    }

    #[test]
    fn test_open_result_with_filter() {
        let result = OpenResult::with_filter(
            vec![PathBuf::from("/image.png")],
            1,
        );
        assert_eq!(result.filter_index, Some(1));
    }
}
