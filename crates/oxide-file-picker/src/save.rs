//! File save dialog implementation.
//!
//! Provides a cross-platform file save dialog with support for
//! suggested filename, file type filtering, and overwrite confirmation.

use crate::types::FileFilter;
use crate::FilePickerError;
use std::path::PathBuf;

/// Builder for configuring and showing a file save dialog.
///
/// # Example
///
/// ```rust,ignore
/// use oxide_file_picker::{SaveDialog, FileFilter};
///
/// let path = SaveDialog::new()
///     .title("Save Document")
///     .default_name("untitled.txt")
///     .filter(FileFilter::documents())
///     .default_extension("txt")
///     .pick()
///     .await?;
///
/// if let Some(path) = path {
///     // Save the file to `path`
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SaveDialog {
    /// Dialog title
    pub(crate) title: Option<String>,
    /// File type filters
    pub(crate) filters: Vec<FileFilter>,
    /// Suggested filename
    pub(crate) default_name: Option<String>,
    /// Default file extension (without dot)
    pub(crate) default_extension: Option<String>,
    /// Initial directory to save in
    pub(crate) initial_dir: Option<PathBuf>,
    /// Show overwrite confirmation
    pub(crate) confirm_overwrite: bool,
    /// Whether to show hidden files
    pub(crate) show_hidden: bool,
    /// Custom button text
    pub(crate) button_text: Option<String>,
    /// Parent window handle (platform-specific)
    pub(crate) parent_window: Option<usize>,
    /// Allow creating new folders
    pub(crate) can_create_folders: bool,
    /// Show file format dropdown
    pub(crate) show_format_dropdown: bool,
}

impl Default for SaveDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveDialog {
    /// Create a new file save dialog builder.
    pub fn new() -> Self {
        Self {
            title: None,
            filters: Vec::new(),
            default_name: None,
            default_extension: None,
            initial_dir: None,
            confirm_overwrite: true,
            show_hidden: false,
            button_text: None,
            parent_window: None,
            can_create_folders: true,
            show_format_dropdown: true,
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

    /// Set the suggested filename.
    pub fn default_name(mut self, name: impl Into<String>) -> Self {
        self.default_name = Some(name.into());
        self
    }

    /// Set the default file extension (without the dot).
    ///
    /// This extension will be appended if the user doesn't specify one.
    pub fn default_extension(mut self, ext: impl Into<String>) -> Self {
        self.default_extension = Some(ext.into());
        self
    }

    /// Set the initial directory.
    pub fn initial_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.initial_dir = Some(dir.into());
        self
    }

    /// Enable or disable overwrite confirmation.
    ///
    /// When enabled (default), the dialog will prompt the user if the
    /// selected file already exists.
    pub fn confirm_overwrite(mut self, confirm: bool) -> Self {
        self.confirm_overwrite = confirm;
        self
    }

    /// Show hidden files in the dialog.
    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    /// Set custom button text (e.g., "Export" instead of "Save").
    pub fn button_text(mut self, text: impl Into<String>) -> Self {
        self.button_text = Some(text.into());
        self
    }

    /// Set the parent window handle for modal behavior.
    pub fn parent(mut self, handle: usize) -> Self {
        self.parent_window = Some(handle);
        self
    }

    /// Allow or disallow creating new folders.
    pub fn can_create_folders(mut self, allow: bool) -> Self {
        self.can_create_folders = allow;
        self
    }

    /// Show or hide the file format dropdown.
    pub fn show_format_dropdown(mut self, show: bool) -> Self {
        self.show_format_dropdown = show;
        self
    }

    /// Show the dialog and pick a save location.
    ///
    /// Returns `None` if the user cancelled the dialog.
    #[cfg(feature = "async-ops")]
    pub async fn pick(self) -> Result<Option<PathBuf>, FilePickerError> {
        let result = tokio::task::spawn_blocking(move || self.pick_sync()).await?;
        result
    }

    /// Synchronously show the dialog and pick a save location.
    pub fn pick_sync(self) -> Result<Option<PathBuf>, FilePickerError> {
        crate::platform::show_save_dialog(&self)
    }

    /// Ensure the path has the correct extension.
    ///
    /// If the path doesn't have an extension and a default is set,
    /// append the default extension.
    pub fn ensure_extension(&self, path: PathBuf) -> PathBuf {
        if path.extension().is_some() {
            return path;
        }

        if let Some(ref ext) = self.default_extension {
            let mut new_path = path.into_os_string();
            new_path.push(".");
            new_path.push(ext);
            PathBuf::from(new_path)
        } else {
            path
        }
    }
}

/// Result of a file save operation.
#[derive(Debug, Clone)]
pub struct SaveResult {
    /// Selected file path.
    pub path: Option<PathBuf>,
    /// The filter index that was selected (0-based).
    pub filter_index: Option<usize>,
}

impl SaveResult {
    /// Create a new save result with a path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            filter_index: None,
        }
    }

    /// Create a cancelled result.
    pub fn cancelled() -> Self {
        Self {
            path: None,
            filter_index: None,
        }
    }

    /// Create a result with a selected filter.
    pub fn with_filter(path: PathBuf, filter_index: usize) -> Self {
        Self {
            path: Some(path),
            filter_index: Some(filter_index),
        }
    }

    /// Check if the dialog was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.path.is_none()
    }

    /// Get the selected path.
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    /// Take ownership of the path.
    pub fn into_path(self) -> Option<PathBuf> {
        self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_dialog_builder() {
        let dialog = SaveDialog::new()
            .title("Save Document")
            .default_name("document.txt")
            .default_extension("txt")
            .confirm_overwrite(true)
            .initial_dir("/home/user/documents");

        assert_eq!(dialog.title.as_deref(), Some("Save Document"));
        assert_eq!(dialog.default_name.as_deref(), Some("document.txt"));
        assert_eq!(dialog.default_extension.as_deref(), Some("txt"));
        assert!(dialog.confirm_overwrite);
        assert_eq!(
            dialog.initial_dir,
            Some(PathBuf::from("/home/user/documents"))
        );
    }

    #[test]
    fn test_save_dialog_filters() {
        let dialog = SaveDialog::new()
            .filter(FileFilter::documents())
            .filters(&[FileFilter::images(), FileFilter::code()]);

        assert_eq!(dialog.filters.len(), 3);
    }

    #[test]
    fn test_ensure_extension() {
        let dialog = SaveDialog::new().default_extension("txt");

        // Path without extension gets default
        let path = dialog.ensure_extension(PathBuf::from("/doc/file"));
        assert_eq!(path, PathBuf::from("/doc/file.txt"));

        // Path with extension is unchanged
        let path = dialog.ensure_extension(PathBuf::from("/doc/file.md"));
        assert_eq!(path, PathBuf::from("/doc/file.md"));
    }

    #[test]
    fn test_ensure_extension_no_default() {
        let dialog = SaveDialog::new();

        // No default extension, path unchanged
        let path = dialog.ensure_extension(PathBuf::from("/doc/file"));
        assert_eq!(path, PathBuf::from("/doc/file"));
    }

    #[test]
    fn test_save_result() {
        let result = SaveResult::new(PathBuf::from("/doc/saved.txt"));
        assert!(!result.is_cancelled());
        assert_eq!(result.path(), Some(&PathBuf::from("/doc/saved.txt")));

        let cancelled = SaveResult::cancelled();
        assert!(cancelled.is_cancelled());
        assert!(cancelled.path().is_none());
    }

    #[test]
    fn test_save_result_with_filter() {
        let result = SaveResult::with_filter(PathBuf::from("/doc/file.pdf"), 2);
        assert_eq!(result.filter_index, Some(2));
    }

    #[test]
    fn test_default_values() {
        let dialog = SaveDialog::new();
        assert!(dialog.confirm_overwrite);
        assert!(dialog.can_create_folders);
        assert!(dialog.show_format_dropdown);
        assert!(!dialog.show_hidden);
    }
}
