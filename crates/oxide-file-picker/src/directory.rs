//! Directory picker dialog implementation.
//!
//! Provides a cross-platform directory selection dialog with support for
//! creating new folders and initial directory configuration.

use crate::FilePickerError;
use std::path::PathBuf;

/// Builder for configuring and showing a directory picker dialog.
///
/// # Example
///
/// ```rust,ignore
/// use oxide_file_picker::DirectoryDialog;
///
/// // Pick a folder
/// let folder = DirectoryDialog::new()
///     .title("Select Project Folder")
///     .initial_dir(home_dir())
///     .can_create_folders(true)
///     .pick()
///     .await?;
///
/// if let Some(folder) = folder {
///     println!("Selected: {}", folder.display());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DirectoryDialog {
    /// Dialog title
    pub(crate) title: Option<String>,
    /// Initial directory to open in
    pub(crate) initial_dir: Option<PathBuf>,
    /// Allow creating new folders
    pub(crate) can_create_folders: bool,
    /// Whether to show hidden files/folders
    pub(crate) show_hidden: bool,
    /// Custom button text
    pub(crate) button_text: Option<String>,
    /// Parent window handle (platform-specific)
    pub(crate) parent_window: Option<usize>,
    /// Prompt message (shown above the file list)
    pub(crate) prompt: Option<String>,
    /// Allow selecting multiple directories
    pub(crate) multiple: bool,
}

impl Default for DirectoryDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectoryDialog {
    /// Create a new directory picker dialog builder.
    pub fn new() -> Self {
        Self {
            title: None,
            initial_dir: None,
            can_create_folders: true,
            show_hidden: false,
            button_text: None,
            parent_window: None,
            prompt: None,
            multiple: false,
        }
    }

    /// Set the dialog title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the initial directory.
    pub fn initial_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.initial_dir = Some(dir.into());
        self
    }

    /// Allow or disallow creating new folders.
    pub fn can_create_folders(mut self, allow: bool) -> Self {
        self.can_create_folders = allow;
        self
    }

    /// Show hidden files/folders in the dialog.
    pub fn show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    /// Set custom button text (e.g., "Choose" instead of "Select").
    pub fn button_text(mut self, text: impl Into<String>) -> Self {
        self.button_text = Some(text.into());
        self
    }

    /// Set the parent window handle for modal behavior.
    pub fn parent(mut self, handle: usize) -> Self {
        self.parent_window = Some(handle);
        self
    }

    /// Set a prompt message shown in the dialog.
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Enable or disable multiple directory selection.
    pub fn multiple(mut self, allow: bool) -> Self {
        self.multiple = allow;
        self
    }

    /// Show the dialog and pick a single directory.
    ///
    /// Returns `None` if the user cancelled the dialog.
    #[cfg(feature = "async-ops")]
    pub async fn pick(self) -> Result<Option<PathBuf>, FilePickerError> {
        let result = tokio::task::spawn_blocking(move || self.pick_sync()).await?;
        result
    }

    /// Show the dialog and pick multiple directories.
    ///
    /// Returns an empty vector if the user cancelled the dialog.
    #[cfg(feature = "async-ops")]
    pub async fn pick_multiple(mut self) -> Result<Vec<PathBuf>, FilePickerError> {
        self.multiple = true;
        let result = tokio::task::spawn_blocking(move || self.pick_multiple_sync()).await?;
        result
    }

    /// Synchronously show the dialog and pick a single directory.
    pub fn pick_sync(self) -> Result<Option<PathBuf>, FilePickerError> {
        crate::platform::show_directory_dialog(&self, false)
            .map(|paths| paths.into_iter().next())
    }

    /// Synchronously show the dialog and pick multiple directories.
    pub fn pick_multiple_sync(mut self) -> Result<Vec<PathBuf>, FilePickerError> {
        self.multiple = true;
        crate::platform::show_directory_dialog(&self, true)
    }
}

/// Common directory locations for easy access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonLocation {
    /// User's home directory
    Home,
    /// User's documents folder
    Documents,
    /// User's downloads folder
    Downloads,
    /// User's desktop folder
    Desktop,
    /// User's pictures/photos folder
    Pictures,
    /// User's music folder
    Music,
    /// User's videos folder
    Videos,
    /// Application configuration folder
    Config,
    /// Application data folder
    Data,
    /// Application cache folder
    Cache,
    /// System temporary folder
    Temp,
}

impl CommonLocation {
    /// Get the path for this common location.
    pub fn path(&self) -> Option<PathBuf> {
        match self {
            Self::Home => dirs::home_dir(),
            Self::Documents => dirs::document_dir(),
            Self::Downloads => dirs::download_dir(),
            Self::Desktop => dirs::desktop_dir(),
            Self::Pictures => dirs::picture_dir(),
            Self::Music => dirs::audio_dir(),
            Self::Videos => dirs::video_dir(),
            Self::Config => dirs::config_dir(),
            Self::Data => dirs::data_dir(),
            Self::Cache => dirs::cache_dir(),
            Self::Temp => Some(std::env::temp_dir()),
        }
    }

    /// Get the display name for this location.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Documents => "Documents",
            Self::Downloads => "Downloads",
            Self::Desktop => "Desktop",
            Self::Pictures => "Pictures",
            Self::Music => "Music",
            Self::Videos => "Videos",
            Self::Config => "Configuration",
            Self::Data => "Application Data",
            Self::Cache => "Cache",
            Self::Temp => "Temporary",
        }
    }

    /// Get all common locations.
    pub fn all() -> &'static [CommonLocation] {
        &[
            Self::Home,
            Self::Documents,
            Self::Downloads,
            Self::Desktop,
            Self::Pictures,
            Self::Music,
            Self::Videos,
        ]
    }

    /// Get common locations for file dialogs (user-facing).
    pub fn for_file_dialogs() -> &'static [CommonLocation] {
        &[
            Self::Home,
            Self::Documents,
            Self::Downloads,
            Self::Desktop,
            Self::Pictures,
        ]
    }
}

/// Result of a directory selection operation.
#[derive(Debug, Clone)]
pub struct DirectoryResult {
    /// Selected directory paths.
    pub paths: Vec<PathBuf>,
}

impl DirectoryResult {
    /// Create a new directory result.
    pub fn new(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }

    /// Create a result with a single path.
    pub fn single(path: PathBuf) -> Self {
        Self { paths: vec![path] }
    }

    /// Create a cancelled result.
    pub fn cancelled() -> Self {
        Self { paths: Vec::new() }
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
    fn test_directory_dialog_builder() {
        let dialog = DirectoryDialog::new()
            .title("Select Folder")
            .initial_dir("/home/user")
            .can_create_folders(true)
            .show_hidden(false)
            .prompt("Choose a project folder");

        assert_eq!(dialog.title.as_deref(), Some("Select Folder"));
        assert_eq!(dialog.initial_dir, Some(PathBuf::from("/home/user")));
        assert!(dialog.can_create_folders);
        assert!(!dialog.show_hidden);
        assert_eq!(
            dialog.prompt.as_deref(),
            Some("Choose a project folder")
        );
    }

    #[test]
    fn test_directory_dialog_multiple() {
        let dialog = DirectoryDialog::new().multiple(true);
        assert!(dialog.multiple);
    }

    #[test]
    fn test_common_location_home() {
        let home = CommonLocation::Home;
        assert_eq!(home.display_name(), "Home");
        assert!(home.path().is_some());
    }

    #[test]
    fn test_common_location_all() {
        let all = CommonLocation::all();
        assert!(!all.is_empty());
        assert!(all.contains(&CommonLocation::Home));
        assert!(all.contains(&CommonLocation::Documents));
    }

    #[test]
    fn test_directory_result() {
        let result = DirectoryResult::single(PathBuf::from("/projects"));
        assert!(!result.is_cancelled());
        assert_eq!(result.first(), Some(&PathBuf::from("/projects")));

        let cancelled = DirectoryResult::cancelled();
        assert!(cancelled.is_cancelled());
    }

    #[test]
    fn test_directory_result_multiple() {
        let result = DirectoryResult::new(vec![
            PathBuf::from("/dir1"),
            PathBuf::from("/dir2"),
        ]);
        assert_eq!(result.all().len(), 2);
    }

    #[test]
    fn test_default_values() {
        let dialog = DirectoryDialog::new();
        assert!(dialog.can_create_folders);
        assert!(!dialog.show_hidden);
        assert!(!dialog.multiple);
    }

    #[test]
    fn test_common_location_temp() {
        let temp = CommonLocation::Temp;
        let path = temp.path();
        assert!(path.is_some());
        // Temp directory should always exist
        assert!(path.unwrap().exists() || cfg!(not(test)));
    }
}
