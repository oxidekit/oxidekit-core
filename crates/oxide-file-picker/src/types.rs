//! File type definitions and associations.
//!
//! This module provides types for file filtering, MIME type handling,
//! and file type associations for "Open with" functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during file picker operations.
#[derive(Debug, Error)]
pub enum FilePickerError {
    /// User cancelled the dialog
    #[error("dialog cancelled by user")]
    Cancelled,

    /// Platform not supported
    #[error("file picker not supported on this platform")]
    PlatformNotSupported,

    /// No file selected
    #[error("no file was selected")]
    NoFileSelected,

    /// Invalid path provided
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// Permission denied
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// File access error
    #[error("file access error: {0}")]
    FileAccessError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// A file type filter for use in file dialogs.
///
/// # Example
///
/// ```
/// use oxide_file_picker::FileFilter;
///
/// let filter = FileFilter::new("Images", &["png", "jpg", "gif", "webp"]);
/// assert!(filter.matches("photo.png"));
/// assert!(!filter.matches("document.pdf"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFilter {
    /// Human-readable name for the filter (e.g., "Images", "Documents")
    pub name: String,
    /// List of file extensions (without the dot)
    pub extensions: Vec<String>,
}

impl FileFilter {
    /// Create a new file filter.
    pub fn new(name: impl Into<String>, extensions: &[&str]) -> Self {
        Self {
            name: name.into(),
            extensions: extensions.iter().map(|s| s.to_lowercase()).collect(),
        }
    }

    /// Create a filter that matches all files.
    pub fn all() -> Self {
        Self {
            name: "All Files".into(),
            extensions: vec!["*".into()],
        }
    }

    /// Check if a filename matches this filter.
    pub fn matches(&self, filename: &str) -> bool {
        if self.extensions.contains(&"*".to_string()) {
            return true;
        }

        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        self.extensions.contains(&ext)
    }

    /// Get the extensions as a pattern string (e.g., "*.png;*.jpg").
    pub fn as_pattern(&self) -> String {
        if self.extensions.contains(&"*".to_string()) {
            "*.*".to_string()
        } else {
            self.extensions
                .iter()
                .map(|ext| format!("*.{}", ext))
                .collect::<Vec<_>>()
                .join(";")
        }
    }
}

impl Default for FileFilter {
    fn default() -> Self {
        Self::all()
    }
}

/// Common file filter presets.
impl FileFilter {
    /// Images filter (png, jpg, jpeg, gif, webp, bmp, svg).
    pub fn images() -> Self {
        Self::new("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"])
    }

    /// Documents filter (pdf, doc, docx, txt, md, rtf).
    pub fn documents() -> Self {
        Self::new("Documents", &["pdf", "doc", "docx", "txt", "md", "rtf", "odt"])
    }

    /// Audio filter (mp3, wav, flac, aac, ogg, m4a).
    pub fn audio() -> Self {
        Self::new("Audio", &["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma"])
    }

    /// Video filter (mp4, mov, avi, mkv, webm).
    pub fn video() -> Self {
        Self::new("Video", &["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"])
    }

    /// Archives filter (zip, tar, gz, rar, 7z).
    pub fn archives() -> Self {
        Self::new("Archives", &["zip", "tar", "gz", "rar", "7z", "bz2", "xz"])
    }

    /// Code files filter.
    pub fn code() -> Self {
        Self::new(
            "Code",
            &[
                "rs", "js", "ts", "jsx", "tsx", "py", "go", "java", "c", "cpp", "h", "hpp", "cs",
                "rb", "php", "swift", "kt", "scala", "oui",
            ],
        )
    }

    /// OxideKit UI files filter.
    pub fn oui() -> Self {
        Self::new("OxideKit UI", &["oui"])
    }
}

/// MIME type information for a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MimeType {
    /// The MIME type string (e.g., "image/png").
    pub mime: String,
    /// Primary file extension for this MIME type.
    pub extension: Option<String>,
}

impl MimeType {
    /// Create a new MIME type.
    pub fn new(mime: impl Into<String>) -> Self {
        Self {
            mime: mime.into(),
            extension: None,
        }
    }

    /// Create a MIME type with an associated extension.
    pub fn with_extension(mime: impl Into<String>, ext: impl Into<String>) -> Self {
        Self {
            mime: mime.into(),
            extension: Some(ext.into()),
        }
    }

    /// Get the MIME type from a file path.
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let mime = mime_guess::from_path(path.as_ref())
            .first_or_octet_stream()
            .to_string();
        let extension = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());
        Self { mime, extension }
    }

    /// Check if this is an image type.
    pub fn is_image(&self) -> bool {
        self.mime.starts_with("image/")
    }

    /// Check if this is a video type.
    pub fn is_video(&self) -> bool {
        self.mime.starts_with("video/")
    }

    /// Check if this is an audio type.
    pub fn is_audio(&self) -> bool {
        self.mime.starts_with("audio/")
    }

    /// Check if this is a text type.
    pub fn is_text(&self) -> bool {
        self.mime.starts_with("text/") || self.mime == "application/json"
    }

    /// Check if this matches a MIME pattern (e.g., "image/*").
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        if pattern == "*/*" {
            return true;
        }
        if pattern.ends_with("/*") {
            let prefix = pattern.trim_end_matches("/*");
            return self.mime.starts_with(&format!("{}/", prefix));
        }
        self.mime == pattern
    }
}

/// File type category for icons and grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileCategory {
    /// Image files
    Image,
    /// Video files
    Video,
    /// Audio/music files
    Audio,
    /// Text documents
    Document,
    /// PDF files
    Pdf,
    /// Spreadsheets
    Spreadsheet,
    /// Presentations
    Presentation,
    /// Archive/compressed files
    Archive,
    /// Source code files
    Code,
    /// Executable files
    Executable,
    /// Font files
    Font,
    /// 3D model files
    Model3D,
    /// OxideKit UI files
    OxideKit,
    /// Folder/directory
    Folder,
    /// Unknown file type
    Unknown,
}

impl FileCategory {
    /// Determine the category from a file path.
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        Self::from_extension(&ext)
    }

    /// Determine the category from a file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Images
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" | "tiff" | "tif"
            | "heic" | "heif" | "avif" => Self::Image,

            // Video
            "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv" | "wmv" | "m4v" | "mpeg" | "mpg" => {
                Self::Video
            }

            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "aiff" | "alac" => Self::Audio,

            // Documents
            "doc" | "docx" | "txt" | "md" | "rtf" | "odt" | "pages" => Self::Document,

            // PDF
            "pdf" => Self::Pdf,

            // Spreadsheets
            "xls" | "xlsx" | "csv" | "ods" | "numbers" => Self::Spreadsheet,

            // Presentations
            "ppt" | "pptx" | "odp" | "key" => Self::Presentation,

            // Archives
            "zip" | "tar" | "gz" | "rar" | "7z" | "bz2" | "xz" | "tgz" | "dmg" | "iso" => {
                Self::Archive
            }

            // Code
            "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "go" | "java" | "c" | "cpp" | "h" | "hpp"
            | "cs" | "rb" | "php" | "swift" | "kt" | "scala" | "html" | "css" | "scss" | "sass"
            | "less" | "json" | "xml" | "yaml" | "yml" | "toml" | "sh" | "bash" | "zsh" | "sql" => {
                Self::Code
            }

            // Executables
            "exe" | "app" | "msi" | "deb" | "rpm" | "apk" | "ipa" | "appimage" => Self::Executable,

            // Fonts
            "ttf" | "otf" | "woff" | "woff2" | "eot" => Self::Font,

            // 3D Models
            "obj" | "fbx" | "gltf" | "glb" | "stl" | "dae" | "3ds" | "blend" => Self::Model3D,

            // OxideKit
            "oui" => Self::OxideKit,

            // Unknown
            _ => Self::Unknown,
        }
    }

    /// Get a human-readable name for this category.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Document => "Document",
            Self::Pdf => "PDF",
            Self::Spreadsheet => "Spreadsheet",
            Self::Presentation => "Presentation",
            Self::Archive => "Archive",
            Self::Code => "Source Code",
            Self::Executable => "Application",
            Self::Font => "Font",
            Self::Model3D => "3D Model",
            Self::OxideKit => "OxideKit UI",
            Self::Folder => "Folder",
            Self::Unknown => "File",
        }
    }

    /// Get the icon name for this category (for icon lookup).
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Image => "file-image",
            Self::Video => "file-video",
            Self::Audio => "file-audio",
            Self::Document => "file-text",
            Self::Pdf => "file-pdf",
            Self::Spreadsheet => "file-spreadsheet",
            Self::Presentation => "file-presentation",
            Self::Archive => "file-archive",
            Self::Code => "file-code",
            Self::Executable => "file-app",
            Self::Font => "file-font",
            Self::Model3D => "file-3d",
            Self::OxideKit => "file-oui",
            Self::Folder => "folder",
            Self::Unknown => "file",
        }
    }
}

/// Handler for a specific file type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeHandler {
    /// Unique identifier for this handler.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Path to the application executable.
    pub executable: String,
    /// Command-line arguments (use %f for file path placeholder).
    pub arguments: Vec<String>,
    /// Icon path or name.
    pub icon: Option<String>,
}

impl FileTypeHandler {
    /// Create a new file type handler.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        executable: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            executable: executable.into(),
            arguments: vec!["%f".to_string()],
            icon: None,
        }
    }

    /// Set custom arguments.
    pub fn with_arguments(mut self, args: &[&str]) -> Self {
        self.arguments = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set the icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Registry for file type associations.
#[derive(Debug, Default)]
pub struct FileTypeRegistry {
    /// Handlers by extension.
    handlers_by_ext: HashMap<String, Vec<FileTypeHandler>>,
    /// Default handlers by extension.
    default_handlers: HashMap<String, String>,
    /// Handlers by MIME type.
    handlers_by_mime: HashMap<String, Vec<FileTypeHandler>>,
}

impl FileTypeRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler for an extension.
    pub fn register_handler(&mut self, extension: &str, handler: FileTypeHandler) {
        let ext = extension.to_lowercase();
        self.handlers_by_ext
            .entry(ext)
            .or_default()
            .push(handler);
    }

    /// Register a handler for a MIME type.
    pub fn register_mime_handler(&mut self, mime: &str, handler: FileTypeHandler) {
        self.handlers_by_mime
            .entry(mime.to_string())
            .or_default()
            .push(handler);
    }

    /// Set the default handler for an extension.
    pub fn set_default(&mut self, extension: &str, handler_id: &str) {
        self.default_handlers
            .insert(extension.to_lowercase(), handler_id.to_string());
    }

    /// Get handlers for a file path.
    pub fn handlers_for_path(&self, path: impl AsRef<Path>) -> Vec<&FileTypeHandler> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        self.handlers_by_ext
            .get(&ext)
            .map(|h| h.iter().collect())
            .unwrap_or_default()
    }

    /// Get handlers for a MIME type.
    pub fn handlers_for_mime(&self, mime: &str) -> Vec<&FileTypeHandler> {
        self.handlers_by_mime
            .get(mime)
            .map(|h| h.iter().collect())
            .unwrap_or_default()
    }

    /// Get the default handler for a path.
    pub fn default_handler(&self, path: impl AsRef<Path>) -> Option<&FileTypeHandler> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())?;

        let handler_id = self.default_handlers.get(&ext)?;
        self.handlers_by_ext
            .get(&ext)?
            .iter()
            .find(|h| &h.id == handler_id)
    }

    /// Get all registered extensions.
    pub fn registered_extensions(&self) -> Vec<&str> {
        self.handlers_by_ext.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_filter_matches() {
        let filter = FileFilter::new("Images", &["png", "jpg"]);
        assert!(filter.matches("photo.png"));
        assert!(filter.matches("photo.PNG"));
        assert!(filter.matches("photo.jpg"));
        assert!(!filter.matches("document.pdf"));
        assert!(!filter.matches("noext"));
    }

    #[test]
    fn test_file_filter_all() {
        let filter = FileFilter::all();
        assert!(filter.matches("anything.xyz"));
        assert!(filter.matches("noext"));
    }

    #[test]
    fn test_file_filter_presets() {
        let images = FileFilter::images();
        assert!(images.matches("photo.png"));
        assert!(images.matches("photo.webp"));

        let docs = FileFilter::documents();
        assert!(docs.matches("doc.pdf"));
        assert!(docs.matches("readme.md"));

        let code = FileFilter::code();
        assert!(code.matches("main.rs"));
        assert!(code.matches("app.oui"));
    }

    #[test]
    fn test_mime_type_detection() {
        let mime = MimeType::from_path("image.png");
        assert_eq!(mime.mime, "image/png");
        assert!(mime.is_image());
        assert!(!mime.is_video());

        let video = MimeType::from_path("movie.mp4");
        assert!(video.is_video());

        let text = MimeType::from_path("readme.txt");
        assert!(text.is_text());
    }

    #[test]
    fn test_mime_pattern_matching() {
        let mime = MimeType::new("image/png");
        assert!(mime.matches_pattern("image/*"));
        assert!(mime.matches_pattern("image/png"));
        assert!(!mime.matches_pattern("video/*"));
        assert!(mime.matches_pattern("*/*"));
    }

    #[test]
    fn test_file_category() {
        assert_eq!(FileCategory::from_extension("png"), FileCategory::Image);
        assert_eq!(FileCategory::from_extension("mp4"), FileCategory::Video);
        assert_eq!(FileCategory::from_extension("rs"), FileCategory::Code);
        assert_eq!(FileCategory::from_extension("oui"), FileCategory::OxideKit);
        assert_eq!(FileCategory::from_extension("xyz"), FileCategory::Unknown);
    }

    #[test]
    fn test_file_category_from_path() {
        let path = std::path::Path::new("/home/user/photo.jpg");
        assert_eq!(FileCategory::from_path(path), FileCategory::Image);
    }

    #[test]
    fn test_file_type_registry() {
        let mut registry = FileTypeRegistry::new();

        let handler = FileTypeHandler::new("vscode", "VS Code", "/usr/bin/code");
        registry.register_handler("rs", handler.clone());
        registry.set_default("rs", "vscode");

        let handlers = registry.handlers_for_path("main.rs");
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].name, "VS Code");

        let default = registry.default_handler("main.rs");
        assert!(default.is_some());
        assert_eq!(default.unwrap().id, "vscode");
    }

    #[test]
    fn test_filter_as_pattern() {
        let filter = FileFilter::new("Images", &["png", "jpg"]);
        let pattern = filter.as_pattern();
        assert!(pattern.contains("*.png"));
        assert!(pattern.contains("*.jpg"));
    }
}
