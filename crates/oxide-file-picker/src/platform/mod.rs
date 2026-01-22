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

// macOS implementation
#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use objc2_app_kit::{NSModalResponseOK, NSOpenPanel, NSSavePanel};
    use objc2_foundation::{MainThreadMarker, NSString, NSURL};

    pub fn show_open_dialog(dialog: &OpenDialog, multiple: bool) -> PickerResult<Vec<PathBuf>> {
        // SAFETY: File dialogs must be called from main thread - this is enforced by macOS
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        unsafe {
            let panel = NSOpenPanel::openPanel(mtm);

            // Set title
            if let Some(title) = &dialog.title {
                panel.setTitle(Some(&NSString::from_str(title)));
            }

            // Set initial directory
            if let Some(dir) = &dialog.initial_dir {
                let url = NSURL::fileURLWithPath(&NSString::from_str(dir.to_string_lossy().as_ref()));
                panel.setDirectoryURL(Some(&url));
            }

            // Set initial filename
            if let Some(name) = &dialog.initial_name {
                panel.setNameFieldStringValue(&NSString::from_str(name));
            }

            // Configure options
            panel.setCanChooseFiles(true);
            panel.setCanChooseDirectories(false);
            panel.setAllowsMultipleSelection(multiple);
            panel.setShowsHiddenFiles(dialog.show_hidden);
            panel.setTreatsFilePackagesAsDirectories(!dialog.can_select_packages);

            // Set button text
            if let Some(text) = &dialog.button_text {
                panel.setPrompt(Some(&NSString::from_str(text)));
            }

            // Show dialog
            let response = panel.runModal();

            if response == NSModalResponseOK {
                let urls = panel.URLs();
                let mut paths = Vec::new();
                for i in 0..urls.count() {
                    if let Some(path) = urls.objectAtIndex(i).path() {
                        paths.push(PathBuf::from(path.to_string()));
                    }
                }
                Ok(paths)
            } else {
                Ok(Vec::new())
            }
        }
    }

    pub fn show_save_dialog(dialog: &SaveDialog) -> PickerResult<Option<PathBuf>> {
        // SAFETY: File dialogs must be called from main thread - this is enforced by macOS
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        unsafe {
            let panel = NSSavePanel::savePanel(mtm);

            // Set title
            if let Some(title) = &dialog.title {
                panel.setTitle(Some(&NSString::from_str(title)));
            }

            // Set initial directory
            if let Some(dir) = &dialog.initial_dir {
                let url = NSURL::fileURLWithPath(&NSString::from_str(dir.to_string_lossy().as_ref()));
                panel.setDirectoryURL(Some(&url));
            }

            // Set default filename
            if let Some(name) = &dialog.default_name {
                panel.setNameFieldStringValue(&NSString::from_str(name));
            }

            // Configure options
            panel.setShowsHiddenFiles(dialog.show_hidden);
            panel.setCanCreateDirectories(dialog.can_create_folders);

            // Set button text
            if let Some(text) = &dialog.button_text {
                panel.setPrompt(Some(&NSString::from_str(text)));
            }

            // Show dialog
            let response = panel.runModal();

            if response == NSModalResponseOK {
                if let Some(url) = panel.URL() {
                    if let Some(path) = url.path() {
                        let path = PathBuf::from(path.to_string());
                        return Ok(Some(dialog.ensure_extension(path)));
                    }
                }
            }

            Ok(None)
        }
    }

    pub fn show_directory_dialog(
        dialog: &DirectoryDialog,
        multiple: bool,
    ) -> PickerResult<Vec<PathBuf>> {
        // SAFETY: File dialogs must be called from main thread - this is enforced by macOS
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        unsafe {
            let panel = NSOpenPanel::openPanel(mtm);

            // Set title
            if let Some(title) = &dialog.title {
                panel.setTitle(Some(&NSString::from_str(title)));
            }

            // Set initial directory
            if let Some(dir) = &dialog.initial_dir {
                let url = NSURL::fileURLWithPath(&NSString::from_str(dir.to_string_lossy().as_ref()));
                panel.setDirectoryURL(Some(&url));
            }

            // Configure for directory selection
            panel.setCanChooseFiles(false);
            panel.setCanChooseDirectories(true);
            panel.setAllowsMultipleSelection(multiple);
            panel.setShowsHiddenFiles(dialog.show_hidden);
            panel.setCanCreateDirectories(dialog.can_create_folders);

            // Set button text
            if let Some(text) = &dialog.button_text {
                panel.setPrompt(Some(&NSString::from_str(text)));
            }

            // Set prompt message
            if let Some(prompt) = &dialog.prompt {
                panel.setMessage(Some(&NSString::from_str(prompt)));
            }

            // Show dialog
            let response = panel.runModal();

            if response == NSModalResponseOK {
                let urls = panel.URLs();
                let mut paths = Vec::new();
                for i in 0..urls.count() {
                    if let Some(path) = urls.objectAtIndex(i).path() {
                        paths.push(PathBuf::from(path.to_string()));
                    }
                }
                Ok(paths)
            } else {
                Ok(Vec::new())
            }
        }
    }
}

// Windows implementation
#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{
        FileOpenDialog, FileSaveDialog, IFileDialog, IFileOpenDialog, IFileSaveDialog,
        IShellItem, SHCreateItemFromParsingName, SIGDN_FILESYSPATH, FOS_ALLOWMULTISELECT,
        FOS_PICKFOLDERS, FOS_FORCEFILESYSTEM,
    };

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn show_open_dialog(dialog: &OpenDialog, multiple: bool) -> PickerResult<Vec<PathBuf>> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

            let file_dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            // Set title
            if let Some(title) = &dialog.title {
                let wide = to_wide(title);
                file_dialog
                    .SetTitle(PCWSTR(wide.as_ptr()))
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;
            }

            // Set options
            let mut options = FOS_FORCEFILESYSTEM;
            if multiple {
                options |= FOS_ALLOWMULTISELECT;
            }
            file_dialog
                .SetOptions(options)
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            // Set initial directory
            if let Some(dir) = &dialog.initial_dir {
                let wide = to_wide(&dir.to_string_lossy());
                if let Ok(item) = SHCreateItemFromParsingName::<IShellItem>(PCWSTR(wide.as_ptr()), None) {
                    let _ = file_dialog.SetFolder(&item);
                }
            }

            // Show dialog
            if file_dialog.Show(None).is_ok() {
                let results = file_dialog
                    .GetResults()
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;

                let count = results
                    .GetCount()
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;

                let mut paths = Vec::new();
                for i in 0..count {
                    if let Ok(item) = results.GetItemAt(i) {
                        if let Ok(path) = item.GetDisplayName(SIGDN_FILESYSPATH) {
                            let path_str = path.to_string().map_err(|e| FilePickerError::Platform(e.to_string()))?;
                            paths.push(PathBuf::from(path_str));
                        }
                    }
                }

                CoUninitialize();
                Ok(paths)
            } else {
                CoUninitialize();
                Ok(Vec::new())
            }
        }
    }

    pub fn show_save_dialog(dialog: &SaveDialog) -> PickerResult<Option<PathBuf>> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

            let file_dialog: IFileSaveDialog =
                CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            // Set title
            if let Some(title) = &dialog.title {
                let wide = to_wide(title);
                file_dialog
                    .SetTitle(PCWSTR(wide.as_ptr()))
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;
            }

            // Set default filename
            if let Some(name) = &dialog.default_name {
                let wide = to_wide(name);
                file_dialog
                    .SetFileName(PCWSTR(wide.as_ptr()))
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;
            }

            // Set options
            file_dialog
                .SetOptions(FOS_FORCEFILESYSTEM)
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            // Set initial directory
            if let Some(dir) = &dialog.initial_dir {
                let wide = to_wide(&dir.to_string_lossy());
                if let Ok(item) = SHCreateItemFromParsingName::<IShellItem>(PCWSTR(wide.as_ptr()), None) {
                    let _ = file_dialog.SetFolder(&item);
                }
            }

            // Show dialog
            if file_dialog.Show(None).is_ok() {
                if let Ok(result) = file_dialog.GetResult() {
                    if let Ok(path) = result.GetDisplayName(SIGDN_FILESYSPATH) {
                        let path_str = path.to_string().map_err(|e| FilePickerError::Platform(e.to_string()))?;
                        let path = PathBuf::from(path_str);
                        CoUninitialize();
                        return Ok(Some(dialog.ensure_extension(path)));
                    }
                }
            }

            CoUninitialize();
            Ok(None)
        }
    }

    pub fn show_directory_dialog(
        dialog: &DirectoryDialog,
        multiple: bool,
    ) -> PickerResult<Vec<PathBuf>> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

            let file_dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            // Set title
            if let Some(title) = &dialog.title {
                let wide = to_wide(title);
                file_dialog
                    .SetTitle(PCWSTR(wide.as_ptr()))
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;
            }

            // Set options for directory picking
            let mut options = FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM;
            if multiple {
                options |= FOS_ALLOWMULTISELECT;
            }
            file_dialog
                .SetOptions(options)
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            // Set initial directory
            if let Some(dir) = &dialog.initial_dir {
                let wide = to_wide(&dir.to_string_lossy());
                if let Ok(item) = SHCreateItemFromParsingName::<IShellItem>(PCWSTR(wide.as_ptr()), None) {
                    let _ = file_dialog.SetFolder(&item);
                }
            }

            // Show dialog
            if file_dialog.Show(None).is_ok() {
                let results = file_dialog
                    .GetResults()
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;

                let count = results
                    .GetCount()
                    .map_err(|e| FilePickerError::Platform(e.to_string()))?;

                let mut paths = Vec::new();
                for i in 0..count {
                    if let Ok(item) = results.GetItemAt(i) {
                        if let Ok(path) = item.GetDisplayName(SIGDN_FILESYSPATH) {
                            let path_str = path.to_string().map_err(|e| FilePickerError::Platform(e.to_string()))?;
                            paths.push(PathBuf::from(path_str));
                        }
                    }
                }

                CoUninitialize();
                Ok(paths)
            } else {
                CoUninitialize();
                Ok(Vec::new())
            }
        }
    }
}

// Linux implementation using XDG Desktop Portal (ashpd)
#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use ashpd::desktop::file_chooser::{Choice, FileChooserProxy, FileFilter as AshpdFilter, OpenFileRequest, SaveFileRequest};
    use ashpd::WindowIdentifier;

    fn to_ashpd_filter(filter: &crate::types::FileFilter) -> AshpdFilter {
        let mut af = AshpdFilter::new(&filter.name);
        for ext in &filter.extensions {
            if ext != "*" {
                af = af.glob(&format!("*.{}", ext));
            }
        }
        af
    }

    pub fn show_open_dialog(dialog: &OpenDialog, multiple: bool) -> PickerResult<Vec<PathBuf>> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| FilePickerError::Platform(e.to_string()))?;

        runtime.block_on(async {
            let proxy = FileChooserProxy::new()
                .await
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            let mut request = OpenFileRequest::default()
                .accept_label(dialog.button_text.as_deref().unwrap_or("Open"))
                .multiple(multiple);

            if let Some(title) = &dialog.title {
                request = request.title(title.as_str());
            }

            if let Some(dir) = &dialog.initial_dir {
                request = request.current_folder(dir.clone());
            }

            for filter in &dialog.filters {
                request = request.filter(to_ashpd_filter(filter));
            }

            let response = proxy
                .open_file(WindowIdentifier::default(), request)
                .await
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            let paths: Vec<PathBuf> = response
                .uris()
                .iter()
                .filter_map(|uri| {
                    if uri.scheme() == "file" {
                        uri.to_file_path().ok()
                    } else {
                        None
                    }
                })
                .collect();

            Ok(paths)
        })
    }

    pub fn show_save_dialog(dialog: &SaveDialog) -> PickerResult<Option<PathBuf>> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| FilePickerError::Platform(e.to_string()))?;

        runtime.block_on(async {
            let proxy = FileChooserProxy::new()
                .await
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            let mut request = SaveFileRequest::default()
                .accept_label(dialog.button_text.as_deref().unwrap_or("Save"));

            if let Some(title) = &dialog.title {
                request = request.title(title.as_str());
            }

            if let Some(dir) = &dialog.initial_dir {
                request = request.current_folder(dir.clone());
            }

            if let Some(name) = &dialog.default_name {
                request = request.current_name(name.as_str());
            }

            for filter in &dialog.filters {
                request = request.filter(to_ashpd_filter(filter));
            }

            let response = proxy
                .save_file(WindowIdentifier::default(), request)
                .await
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            let path = response
                .uris()
                .first()
                .and_then(|uri| {
                    if uri.scheme() == "file" {
                        uri.to_file_path().ok()
                    } else {
                        None
                    }
                })
                .map(|p| dialog.ensure_extension(p));

            Ok(path)
        })
    }

    pub fn show_directory_dialog(
        dialog: &DirectoryDialog,
        multiple: bool,
    ) -> PickerResult<Vec<PathBuf>> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| FilePickerError::Platform(e.to_string()))?;

        runtime.block_on(async {
            let proxy = FileChooserProxy::new()
                .await
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            let mut request = OpenFileRequest::default()
                .accept_label(dialog.button_text.as_deref().unwrap_or("Select"))
                .multiple(multiple)
                .directory(true);

            if let Some(title) = &dialog.title {
                request = request.title(title.as_str());
            }

            if let Some(dir) = &dialog.initial_dir {
                request = request.current_folder(dir.clone());
            }

            let response = proxy
                .open_file(WindowIdentifier::default(), request)
                .await
                .map_err(|e| FilePickerError::Platform(e.to_string()))?;

            let paths: Vec<PathBuf> = response
                .uris()
                .iter()
                .filter_map(|uri| {
                    if uri.scheme() == "file" {
                        uri.to_file_path().ok()
                    } else {
                        None
                    }
                })
                .collect();

            Ok(paths)
        })
    }
}

/// Show the native open file dialog.
///
/// # Arguments
/// * `dialog` - Configuration for the file dialog
/// * `multiple` - Whether to allow selecting multiple files
///
/// # Returns
/// A vector of selected file paths, or an error
pub fn show_open_dialog(
    dialog: &OpenDialog,
    multiple: bool,
) -> PickerResult<Vec<PathBuf>> {
    #[cfg(target_os = "macos")]
    {
        macos::show_open_dialog(dialog, multiple)
    }

    #[cfg(target_os = "windows")]
    {
        windows::show_open_dialog(dialog, multiple)
    }

    #[cfg(target_os = "linux")]
    {
        linux::show_open_dialog(dialog, multiple)
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
pub fn show_save_dialog(dialog: &SaveDialog) -> PickerResult<Option<PathBuf>> {
    #[cfg(target_os = "macos")]
    {
        macos::show_save_dialog(dialog)
    }

    #[cfg(target_os = "windows")]
    {
        windows::show_save_dialog(dialog)
    }

    #[cfg(target_os = "linux")]
    {
        linux::show_save_dialog(dialog)
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
    dialog: &DirectoryDialog,
    multiple: bool,
) -> PickerResult<Vec<PathBuf>> {
    #[cfg(target_os = "macos")]
    {
        macos::show_directory_dialog(dialog, multiple)
    }

    #[cfg(target_os = "windows")]
    {
        windows::show_directory_dialog(dialog, multiple)
    }

    #[cfg(target_os = "linux")]
    {
        linux::show_directory_dialog(dialog, multiple)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(FilePickerError::PlatformNotSupported)
    }
}
