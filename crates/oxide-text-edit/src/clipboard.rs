//! Clipboard integration for text editing
//!
//! Provides platform-agnostic clipboard operations with hooks for platform-specific implementations.

use crate::operations::{OperationResult, TextOperations};
use crate::selection::SelectionRange;

/// Clipboard content types
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardContent {
    /// Plain text content
    Text(String),
    /// Rich text content (HTML or RTF)
    RichText {
        /// Plain text representation
        plain: String,
        /// Rich text markup (HTML/RTF)
        markup: String,
        /// Format of the markup
        format: RichTextFormat,
    },
}

/// Rich text format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RichTextFormat {
    /// HTML format
    Html,
    /// RTF format
    Rtf,
}

impl ClipboardContent {
    /// Create plain text clipboard content
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Create rich text clipboard content
    pub fn rich_text(plain: impl Into<String>, markup: impl Into<String>, format: RichTextFormat) -> Self {
        Self::RichText {
            plain: plain.into(),
            markup: markup.into(),
            format,
        }
    }

    /// Get the plain text representation
    pub fn as_plain_text(&self) -> &str {
        match self {
            Self::Text(text) => text,
            Self::RichText { plain, .. } => plain,
        }
    }

    /// Check if this is plain text
    pub fn is_plain_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Check if this is rich text
    pub fn is_rich_text(&self) -> bool {
        matches!(self, Self::RichText { .. })
    }
}

/// Platform clipboard provider trait
///
/// Implement this trait to provide platform-specific clipboard access.
pub trait ClipboardProvider: Send + Sync {
    /// Read text from the clipboard
    fn read_text(&self) -> Result<Option<String>, ClipboardError>;

    /// Write text to the clipboard
    fn write_text(&self, text: &str) -> Result<(), ClipboardError>;

    /// Read rich text from the clipboard (optional)
    fn read_rich_text(&self) -> Result<Option<ClipboardContent>, ClipboardError> {
        self.read_text().map(|opt| opt.map(ClipboardContent::Text))
    }

    /// Write rich text to the clipboard (optional)
    fn write_rich_text(&self, content: &ClipboardContent) -> Result<(), ClipboardError> {
        self.write_text(content.as_plain_text())
    }

    /// Check if the clipboard contains text
    fn has_text(&self) -> bool {
        self.read_text().map(|opt| opt.is_some()).unwrap_or(false)
    }

    /// Clear the clipboard
    fn clear(&self) -> Result<(), ClipboardError> {
        self.write_text("")
    }
}

/// Clipboard errors
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardError {
    /// Clipboard is not available
    NotAvailable,
    /// Access denied
    AccessDenied,
    /// Content format not supported
    UnsupportedFormat,
    /// Failed to read from clipboard
    ReadFailed(String),
    /// Failed to write to clipboard
    WriteFailed(String),
    /// Clipboard is locked by another process
    Locked,
    /// Platform-specific error
    Platform(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAvailable => write!(f, "Clipboard not available"),
            Self::AccessDenied => write!(f, "Clipboard access denied"),
            Self::UnsupportedFormat => write!(f, "Clipboard format not supported"),
            Self::ReadFailed(msg) => write!(f, "Failed to read clipboard: {}", msg),
            Self::WriteFailed(msg) => write!(f, "Failed to write clipboard: {}", msg),
            Self::Locked => write!(f, "Clipboard is locked"),
            Self::Platform(msg) => write!(f, "Platform clipboard error: {}", msg),
        }
    }
}

impl std::error::Error for ClipboardError {}

/// In-memory clipboard provider for testing and platforms without system clipboard
#[derive(Debug, Default)]
pub struct MemoryClipboard {
    content: std::sync::Mutex<Option<ClipboardContent>>,
}

/// System clipboard provider using the native OS clipboard
#[cfg(feature = "system-clipboard")]
pub struct SystemClipboard {
    clipboard: std::sync::Mutex<arboard::Clipboard>,
}

#[cfg(feature = "system-clipboard")]
impl SystemClipboard {
    /// Create a new system clipboard provider
    pub fn new() -> Result<Self, ClipboardError> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|e| ClipboardError::Platform(e.to_string()))?;
        Ok(Self {
            clipboard: std::sync::Mutex::new(clipboard),
        })
    }
}

#[cfg(feature = "system-clipboard")]
impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new().expect("Failed to initialize system clipboard")
    }
}

#[cfg(feature = "system-clipboard")]
impl std::fmt::Debug for SystemClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemClipboard").finish()
    }
}

#[cfg(feature = "system-clipboard")]
impl ClipboardProvider for SystemClipboard {
    fn read_text(&self) -> Result<Option<String>, ClipboardError> {
        let mut clipboard = self.clipboard.lock().map_err(|_| ClipboardError::Locked)?;
        match clipboard.get_text() {
            Ok(text) if text.is_empty() => Ok(None),
            Ok(text) => Ok(Some(text)),
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(e) => Err(ClipboardError::ReadFailed(e.to_string())),
        }
    }

    fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        let mut clipboard = self.clipboard.lock().map_err(|_| ClipboardError::Locked)?;
        clipboard
            .set_text(text.to_string())
            .map_err(|e| ClipboardError::WriteFailed(e.to_string()))
    }

    fn has_text(&self) -> bool {
        self.read_text().map(|opt| opt.is_some()).unwrap_or(false)
    }

    fn clear(&self) -> Result<(), ClipboardError> {
        let mut clipboard = self.clipboard.lock().map_err(|_| ClipboardError::Locked)?;
        clipboard
            .clear()
            .map_err(|e| ClipboardError::WriteFailed(e.to_string()))
    }
}

impl MemoryClipboard {
    /// Create a new in-memory clipboard
    pub fn new() -> Self {
        Self::default()
    }
}

impl ClipboardProvider for MemoryClipboard {
    fn read_text(&self) -> Result<Option<String>, ClipboardError> {
        let guard = self.content.lock().map_err(|_| ClipboardError::Locked)?;
        Ok(guard.as_ref().map(|c| c.as_plain_text().to_string()))
    }

    fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        let mut guard = self.content.lock().map_err(|_| ClipboardError::Locked)?;
        *guard = Some(ClipboardContent::Text(text.to_string()));
        Ok(())
    }

    fn read_rich_text(&self) -> Result<Option<ClipboardContent>, ClipboardError> {
        let guard = self.content.lock().map_err(|_| ClipboardError::Locked)?;
        Ok(guard.clone())
    }

    fn write_rich_text(&self, content: &ClipboardContent) -> Result<(), ClipboardError> {
        let mut guard = self.content.lock().map_err(|_| ClipboardError::Locked)?;
        *guard = Some(content.clone());
        Ok(())
    }

    fn clear(&self) -> Result<(), ClipboardError> {
        let mut guard = self.content.lock().map_err(|_| ClipboardError::Locked)?;
        *guard = None;
        Ok(())
    }
}

/// Clipboard operations for text editing
pub struct ClipboardOperations<P: ClipboardProvider> {
    provider: P,
}

impl<P: ClipboardProvider> ClipboardOperations<P> {
    /// Create clipboard operations with a provider
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    /// Copy selected text to clipboard
    ///
    /// Returns the copied text if successful, None if selection is empty
    pub fn copy(&self, text: &str, selection: &SelectionRange) -> Result<Option<String>, ClipboardError> {
        if selection.is_collapsed() {
            return Ok(None);
        }

        let selected = selection.extract_text(text).to_string();
        self.provider.write_text(&selected)?;
        Ok(Some(selected))
    }

    /// Cut selected text to clipboard
    ///
    /// Returns the operation result and the cut text if successful
    pub fn cut(
        &self,
        text: &str,
        selection: &SelectionRange,
    ) -> Result<Option<(OperationResult, String)>, ClipboardError> {
        if selection.is_collapsed() {
            return Ok(None);
        }

        let selected = selection.extract_text(text).to_string();
        self.provider.write_text(&selected)?;

        let result = TextOperations::delete_selection(text, selection);
        Ok(Some((result, selected)))
    }

    /// Paste text from clipboard
    ///
    /// Returns the operation result if there was text to paste
    pub fn paste(&self, text: &str, selection: &SelectionRange) -> Result<Option<OperationResult>, ClipboardError> {
        let clipboard_text = self.provider.read_text()?;

        match clipboard_text {
            Some(paste_text) if !paste_text.is_empty() => {
                let result = TextOperations::insert_at_selection(text, selection, &paste_text);
                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    /// Paste text without formatting (strips rich text to plain text)
    pub fn paste_plain(&self, text: &str, selection: &SelectionRange) -> Result<Option<OperationResult>, ClipboardError> {
        // Read as rich text and extract plain text
        let content = self.provider.read_rich_text()?;

        match content {
            Some(content) => {
                let plain_text = content.as_plain_text();
                if plain_text.is_empty() {
                    return Ok(None);
                }
                let result = TextOperations::insert_at_selection(text, selection, plain_text);
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// Check if there is text available to paste
    pub fn can_paste(&self) -> bool {
        self.provider.has_text()
    }

    /// Get the clipboard provider
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// Get mutable access to the clipboard provider
    pub fn provider_mut(&mut self) -> &mut P {
        &mut self.provider
    }
}

/// Result of a clipboard operation on text
#[derive(Debug, Clone)]
pub struct ClipboardResult {
    /// The operation result (new text and selection)
    pub operation: Option<OperationResult>,
    /// The text that was copied/cut (if any)
    pub clipboard_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::TextPosition;

    fn create_clipboard() -> ClipboardOperations<MemoryClipboard> {
        ClipboardOperations::new(MemoryClipboard::new())
    }

    #[test]
    fn test_copy() {
        let clipboard = create_clipboard();
        let text = "Hello World";
        let selection = SelectionRange::new(
            TextPosition::from_offset(0),
            TextPosition::from_offset(5),
        );

        let result = clipboard.copy(text, &selection).unwrap();
        assert_eq!(result, Some("Hello".to_string()));

        // Verify clipboard contains the text
        let pasted = clipboard.provider().read_text().unwrap();
        assert_eq!(pasted, Some("Hello".to_string()));
    }

    #[test]
    fn test_copy_empty_selection() {
        let clipboard = create_clipboard();
        let text = "Hello World";
        let selection = SelectionRange::collapsed_at(5);

        let result = clipboard.copy(text, &selection).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cut() {
        let clipboard = create_clipboard();
        let text = "Hello World";
        let selection = SelectionRange::new(
            TextPosition::from_offset(0),
            TextPosition::from_offset(6),
        );

        let result = clipboard.cut(text, &selection).unwrap().unwrap();
        assert_eq!(result.0.text, "World");
        assert_eq!(result.1, "Hello ");

        // Verify clipboard contains the cut text
        let pasted = clipboard.provider().read_text().unwrap();
        assert_eq!(pasted, Some("Hello ".to_string()));
    }

    #[test]
    fn test_cut_empty_selection() {
        let clipboard = create_clipboard();
        let text = "Hello World";
        let selection = SelectionRange::collapsed_at(5);

        let result = clipboard.cut(text, &selection).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_paste() {
        let clipboard = create_clipboard();
        let text = "Hello World";
        let selection = SelectionRange::collapsed_at(5);

        // Write something to clipboard first
        clipboard.provider().write_text(" Beautiful").unwrap();

        let result = clipboard.paste(text, &selection).unwrap().unwrap();
        assert_eq!(result.text, "Hello Beautiful World");
    }

    #[test]
    fn test_paste_replacing_selection() {
        let clipboard = create_clipboard();
        let text = "Hello World";
        let selection = SelectionRange::new(
            TextPosition::from_offset(6),
            TextPosition::from_offset(11),
        );

        clipboard.provider().write_text("Universe").unwrap();

        let result = clipboard.paste(text, &selection).unwrap().unwrap();
        assert_eq!(result.text, "Hello Universe");
    }

    #[test]
    fn test_paste_empty_clipboard() {
        let clipboard = create_clipboard();
        let text = "Hello World";
        let selection = SelectionRange::collapsed_at(5);

        let result = clipboard.paste(text, &selection).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_can_paste() {
        let clipboard = create_clipboard();

        assert!(!clipboard.can_paste());

        clipboard.provider().write_text("Test").unwrap();
        assert!(clipboard.can_paste());

        clipboard.provider().clear().unwrap();
        assert!(!clipboard.can_paste());
    }

    #[test]
    fn test_clipboard_content_plain() {
        let content = ClipboardContent::text("Hello");
        assert!(content.is_plain_text());
        assert!(!content.is_rich_text());
        assert_eq!(content.as_plain_text(), "Hello");
    }

    #[test]
    fn test_clipboard_content_rich() {
        let content = ClipboardContent::rich_text(
            "Hello",
            "<b>Hello</b>",
            RichTextFormat::Html,
        );
        assert!(!content.is_plain_text());
        assert!(content.is_rich_text());
        assert_eq!(content.as_plain_text(), "Hello");
    }

    #[test]
    fn test_memory_clipboard_rich_text() {
        let clipboard = MemoryClipboard::new();
        let content = ClipboardContent::rich_text(
            "Hello",
            "<b>Hello</b>",
            RichTextFormat::Html,
        );

        clipboard.write_rich_text(&content).unwrap();
        let read = clipboard.read_rich_text().unwrap().unwrap();

        assert!(read.is_rich_text());
        if let ClipboardContent::RichText { plain, markup, format } = read {
            assert_eq!(plain, "Hello");
            assert_eq!(markup, "<b>Hello</b>");
            assert_eq!(format, RichTextFormat::Html);
        }
    }

    #[test]
    fn test_paste_plain() {
        let clipboard = create_clipboard();
        let text = "Hello World";
        let selection = SelectionRange::collapsed_at(5);

        // Write rich text to clipboard
        let content = ClipboardContent::rich_text(
            " Plain",
            "<b> Bold</b>",
            RichTextFormat::Html,
        );
        clipboard.provider().write_rich_text(&content).unwrap();

        // Paste as plain text
        let result = clipboard.paste_plain(text, &selection).unwrap().unwrap();
        assert_eq!(result.text, "Hello Plain World");
    }
}
