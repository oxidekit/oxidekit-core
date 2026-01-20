//! Platform-specific file picker implementations.
//!
//! This module provides the native file dialog implementations for each platform.

use crate::directory::DirectoryDialog;
use crate::open::OpenDialog;
use crate::save::SaveDialog;
use crate::FilePickerError;
use std::path::PathBuf;

/// Result type for file picker operations.
pub type PickerResult<T> = Result<T, FilePickerError>;

/// Show the native open file dialog.
///
/// # Arguments
/// * `dialog` - Configuration for the file dialog
/// * `multiple` - Whether to allow selecting multiple files
///
/// # Returns
/// A vector of selected file paths, or an error
pub fn show_open_dialog(
    _dialog: &OpenDialog,
    _multiple: bool,
) -> PickerResult<Vec<PathBuf>> {
    #[cfg(target_os = "macos")]
    {
        // TODO: Implement macOS file dialog using NSOpenPanel
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(target_os = "windows")]
    {
        // TODO: Implement Windows file dialog using IFileOpenDialog
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(target_os = "linux")]
    {
        // TODO: Implement Linux file dialog using GTK or portal
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(FilePickerError::PlatformNotSupported)
    }
}

/// Show the native save file dialog.
///
/// # Arguments
/// * `dialog` - Configuration for the file dialog
///
/// # Returns
/// The selected file path (Some) or None if cancelled, or an error
pub fn show_save_dialog(_dialog: &SaveDialog) -> PickerResult<Option<PathBuf>> {
    #[cfg(target_os = "macos")]
    {
        // TODO: Implement macOS save dialog using NSSavePanel
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(target_os = "windows")]
    {
        // TODO: Implement Windows save dialog using IFileSaveDialog
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(target_os = "linux")]
    {
        // TODO: Implement Linux save dialog using GTK or portal
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(FilePickerError::PlatformNotSupported)
    }
}

/// Show the native directory picker dialog.
///
/// # Arguments
/// * `dialog` - Configuration for the directory dialog
/// * `multiple` - Whether to allow selecting multiple directories
///
/// # Returns
/// A vector of selected directory paths, or an error
pub fn show_directory_dialog(
    _dialog: &DirectoryDialog,
    _multiple: bool,
) -> PickerResult<Vec<PathBuf>> {
    #[cfg(target_os = "macos")]
    {
        // TODO: Implement macOS directory picker using NSOpenPanel
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(target_os = "windows")]
    {
        // TODO: Implement Windows directory picker using IFileOpenDialog
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(target_os = "linux")]
    {
        // TODO: Implement Linux directory picker using GTK or portal
        Err(FilePickerError::PlatformNotSupported)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(FilePickerError::PlatformNotSupported)
    }
}
