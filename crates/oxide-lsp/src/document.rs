//! Document Store
//!
//! Manages open documents and their state for the LSP server.

use std::collections::HashMap;
use tower_lsp::lsp_types::Url;

/// An open document in the editor
#[derive(Debug, Clone)]
pub struct Document {
    /// Document URI
    pub uri: Url,
    /// Document content
    pub content: String,
    /// Language ID (oui, toml, etc.)
    pub language_id: String,
    /// Document version
    pub version: i32,
    /// Parsed lines for quick access
    lines: Vec<String>,
}

impl Document {
    /// Create a new document
    pub fn new(uri: Url, content: String, language_id: String, version: i32) -> Self {
        let lines = content.lines().map(String::from).collect();
        Self {
            uri,
            content,
            language_id,
            version,
            lines,
        }
    }

    /// Update document content
    pub fn update(&mut self, content: String, version: i32) {
        self.lines = content.lines().map(String::from).collect();
        self.content = content;
        self.version = version;
    }

    /// Get a specific line (0-indexed)
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    /// Get the total number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get the character at a position
    pub fn char_at(&self, line: usize, column: usize) -> Option<char> {
        self.get_line(line)?.chars().nth(column)
    }

    /// Get the word at a position
    pub fn word_at(&self, line: usize, column: usize) -> Option<WordInfo> {
        let line_text = self.get_line(line)?;
        let chars: Vec<char> = line_text.chars().collect();

        if column >= chars.len() {
            return None;
        }

        // Find word boundaries
        let mut start = column;
        let mut end = column;

        // Scan backwards to find start
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

        // Scan forwards to find end
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }

        if start == end {
            return None;
        }

        let word: String = chars[start..end].iter().collect();
        Some(WordInfo {
            word,
            start_column: start,
            end_column: end,
        })
    }

    /// Get text in a range
    pub fn text_in_range(&self, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> String {
        if start_line == end_line {
            if let Some(line) = self.get_line(start_line) {
                return line.chars().skip(start_col).take(end_col - start_col).collect();
            }
        }

        let mut result = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i < start_line || i > end_line {
                continue;
            }
            if i == start_line {
                result.push_str(&line.chars().skip(start_col).collect::<String>());
                result.push('\n');
            } else if i == end_line {
                result.push_str(&line.chars().take(end_col).collect::<String>());
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }

    /// Check if this is an OUI file
    pub fn is_oui(&self) -> bool {
        self.language_id == "oui" || self.uri.path().ends_with(".oui")
    }

    /// Check if this is a TOML config file
    pub fn is_toml_config(&self) -> bool {
        self.language_id == "toml" && (
            self.uri.path().ends_with("oxide.toml") ||
            self.uri.path().ends_with("plugin.toml") ||
            self.uri.path().ends_with("theme.toml") ||
            self.uri.path().ends_with("typography.toml") ||
            self.uri.path().ends_with("fonts.toml")
        )
    }
}

/// Word information at a position
#[derive(Debug, Clone)]
pub struct WordInfo {
    pub word: String,
    pub start_column: usize,
    pub end_column: usize,
}

/// Check if a character is part of a word
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.'
}

/// Store for all open documents
#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<Url, Document>,
}

impl DocumentStore {
    /// Create a new document store
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Open a document
    pub fn open(&mut self, uri: Url, content: String, language_id: String, version: i32) {
        let doc = Document::new(uri.clone(), content, language_id, version);
        self.documents.insert(uri, doc);
    }

    /// Update a document
    pub fn update(&mut self, uri: &Url, content: String, version: i32) {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.update(content, version);
        }
    }

    /// Close a document
    pub fn close(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// Get a document
    pub fn get(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(uri)
    }

    /// Get all open documents
    pub fn all(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }

    /// Get all OUI documents
    pub fn oui_documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values().filter(|d| d.is_oui())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_lines() {
        let uri = Url::parse("file:///test.oui").unwrap();
        let doc = Document::new(
            uri,
            "line1\nline2\nline3".to_string(),
            "oui".to_string(),
            1,
        );

        assert_eq!(doc.line_count(), 3);
        assert_eq!(doc.get_line(0), Some("line1"));
        assert_eq!(doc.get_line(1), Some("line2"));
        assert_eq!(doc.get_line(2), Some("line3"));
        assert_eq!(doc.get_line(3), None);
    }

    #[test]
    fn test_word_at() {
        let uri = Url::parse("file:///test.oui").unwrap();
        let doc = Document::new(
            uri,
            "Text { content: \"hello\" }".to_string(),
            "oui".to_string(),
            1,
        );

        let word = doc.word_at(0, 0).unwrap();
        assert_eq!(word.word, "Text");
        assert_eq!(word.start_column, 0);
        assert_eq!(word.end_column, 4);

        let word = doc.word_at(0, 7).unwrap();
        assert_eq!(word.word, "content");
    }

    #[test]
    fn test_document_store() {
        let mut store = DocumentStore::new();
        let uri = Url::parse("file:///test.oui").unwrap();

        store.open(uri.clone(), "content".to_string(), "oui".to_string(), 1);
        assert!(store.get(&uri).is_some());

        store.update(&uri, "new content".to_string(), 2);
        assert_eq!(store.get(&uri).unwrap().content, "new content");

        store.close(&uri);
        assert!(store.get(&uri).is_none());
    }
}
