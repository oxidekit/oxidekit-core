//! Main code editor component
//!
//! This module provides the main `CodeEditor` component and the read-only `CodeBlock` widget.

use crate::features::complete::{AutoComplete, CompletionProvider};
use crate::features::cursors::{Cursor, CursorMode, MultiCursor, Selection};
use crate::features::folding::{FoldState, FoldingProvider};
use crate::features::search::SearchReplace;
use crate::syntax::highlighter::SyntaxHighlighter;
use crate::syntax::languages::{Language, LanguageRegistry};
use crate::syntax::themes::Theme;
use crate::view::gutter::GutterConfig;
use crate::view::line_numbers::LineNumberConfig;
use crate::view::minimap::MinimapConfig;
use crate::{EditorError, EditorResult, Position, Range};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Events emitted by the editor
#[derive(Debug, Clone)]
pub enum EditorEvent {
    /// Content changed
    Change { content: String },
    /// Cursor position changed
    CursorMove { positions: Vec<Position> },
    /// Selection changed
    SelectionChange { selections: Vec<Selection> },
    /// Save requested (Cmd+S)
    SaveRequested,
    /// Focus gained
    Focus,
    /// Focus lost
    Blur,
    /// Line clicked
    LineClick { line: usize },
    /// Gutter clicked
    GutterClick { line: usize },
    /// Fold toggled
    FoldToggle { line: usize, folded: bool },
}

/// Configuration for the code editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    /// Enable line numbers
    pub line_numbers: bool,
    /// Enable code folding
    pub folding: bool,
    /// Enable minimap
    pub minimap: bool,
    /// Enable bracket matching
    pub bracket_matching: bool,
    /// Enable auto-indent
    pub auto_indent: bool,
    /// Enable auto-close brackets
    pub auto_close_brackets: bool,
    /// Enable auto-close quotes
    pub auto_close_quotes: bool,
    /// Tab size in spaces
    pub tab_size: usize,
    /// Use soft tabs (spaces instead of tabs)
    pub soft_tabs: bool,
    /// Word wrap mode
    pub word_wrap: WordWrap,
    /// Show whitespace
    pub show_whitespace: bool,
    /// Highlight current line
    pub highlight_current_line: bool,
    /// Enable autocomplete
    pub autocomplete: bool,
    /// Font size in pixels
    pub font_size: f32,
    /// Font family
    pub font_family: String,
    /// Line height multiplier
    pub line_height: f32,
    /// Read-only mode
    pub read_only: bool,
    /// Enable virtual scrolling for large files
    pub virtual_scrolling: bool,
    /// Number of visible lines before triggering virtual scroll
    pub virtual_scroll_threshold: usize,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            line_numbers: true,
            folding: true,
            minimap: true,
            bracket_matching: true,
            auto_indent: true,
            auto_close_brackets: true,
            auto_close_quotes: true,
            tab_size: 4,
            soft_tabs: true,
            word_wrap: WordWrap::Off,
            show_whitespace: false,
            highlight_current_line: true,
            autocomplete: true,
            font_size: 14.0,
            font_family: "JetBrains Mono".to_string(),
            line_height: 1.5,
            read_only: false,
            virtual_scrolling: true,
            virtual_scroll_threshold: 10000,
        }
    }
}

/// Word wrap mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WordWrap {
    /// No word wrap
    #[default]
    Off,
    /// Wrap at editor width
    On,
    /// Wrap at specific column
    Column(usize),
    /// Wrap at word boundaries only
    WordBoundary,
}

/// The internal state of the editor
#[derive(Debug, Clone)]
pub struct EditorState {
    /// The document content as a rope (efficient for large files)
    pub document: Rope,
    /// Multiple cursors
    pub cursors: MultiCursor,
    /// Current selections
    pub selections: Vec<Selection>,
    /// Undo history
    pub undo_stack: Vec<EditOperation>,
    /// Redo history
    pub redo_stack: Vec<EditOperation>,
    /// Current fold state
    pub fold_state: FoldState,
    /// Currently visible line range
    pub visible_range: Range,
    /// Scroll position
    pub scroll_offset: (f32, f32),
    /// Highlighted lines
    pub highlighted_lines: HashSet<usize>,
    /// Diagnostic markers (errors, warnings)
    pub diagnostics: Vec<Diagnostic>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            document: Rope::new(),
            cursors: MultiCursor::new(),
            selections: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            fold_state: FoldState::default(),
            visible_range: Range::default(),
            scroll_offset: (0.0, 0.0),
            highlighted_lines: HashSet::new(),
            diagnostics: Vec::new(),
        }
    }
}

/// An edit operation for undo/redo
#[derive(Debug, Clone)]
pub struct EditOperation {
    /// The range that was affected
    pub range: Range,
    /// The old text that was replaced
    pub old_text: String,
    /// The new text that replaced it
    pub new_text: String,
    /// Cursor positions before the edit
    pub old_cursors: Vec<Position>,
    /// Cursor positions after the edit
    pub new_cursors: Vec<Position>,
}

/// A diagnostic marker (error, warning, info)
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The range of the diagnostic
    pub range: Range,
    /// The severity level
    pub severity: DiagnosticSeverity,
    /// The message
    pub message: String,
    /// Optional source (e.g., "rustc", "eslint")
    pub source: Option<String>,
    /// Optional error code
    pub code: Option<String>,
}

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// The main code editor component
#[derive(Debug)]
pub struct CodeEditor {
    /// Editor configuration
    config: EditorConfig,
    /// Current language
    language: Language,
    /// Current theme
    theme: Theme,
    /// Editor state
    state: EditorState,
    /// Syntax highlighter
    highlighter: SyntaxHighlighter,
    /// Search/replace functionality
    search: SearchReplace,
    /// Autocomplete functionality
    autocomplete: AutoComplete,
    /// Language registry
    language_registry: LanguageRegistry,
    /// Folding provider
    folding_provider: Option<Box<dyn FoldingProvider>>,
    /// Completion providers
    completion_providers: Vec<Box<dyn CompletionProvider>>,
    /// Event callback
    on_change: Option<Box<dyn Fn(&str)>>,
    /// Event handler
    on_event: Option<Box<dyn Fn(EditorEvent)>>,
    /// Line number config
    line_number_config: LineNumberConfig,
    /// Gutter config
    gutter_config: GutterConfig,
    /// Minimap config
    minimap_config: MinimapConfig,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeEditor {
    /// Create a new code editor
    pub fn new() -> Self {
        Self {
            config: EditorConfig::default(),
            language: Language::PlainText,
            theme: Theme::dark(),
            state: EditorState::default(),
            highlighter: SyntaxHighlighter::new(),
            search: SearchReplace::new(),
            autocomplete: AutoComplete::new(),
            language_registry: LanguageRegistry::default(),
            folding_provider: None,
            completion_providers: Vec::new(),
            on_change: None,
            on_event: None,
            line_number_config: LineNumberConfig::default(),
            gutter_config: GutterConfig::default(),
            minimap_config: MinimapConfig::default(),
        }
    }

    /// Set the language for syntax highlighting
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self.highlighter.set_language(language);
        self
    }

    /// Set the color theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the initial value/content
    pub fn value(mut self, content: &str) -> Self {
        self.state.document = Rope::from_str(content);
        self.highlighter.highlight_all(&self.state.document);
        self
    }

    /// Enable or disable line numbers
    pub fn line_numbers(mut self, enabled: bool) -> Self {
        self.config.line_numbers = enabled;
        self
    }

    /// Enable or disable code folding
    pub fn folding(mut self, enabled: bool) -> Self {
        self.config.folding = enabled;
        self
    }

    /// Enable or disable minimap
    pub fn minimap(mut self, enabled: bool) -> Self {
        self.config.minimap = enabled;
        self
    }

    /// Set the tab size
    pub fn tab_size(mut self, size: usize) -> Self {
        self.config.tab_size = size;
        self
    }

    /// Use soft tabs (spaces)
    pub fn soft_tabs(mut self, enabled: bool) -> Self {
        self.config.soft_tabs = enabled;
        self
    }

    /// Set word wrap mode
    pub fn word_wrap(mut self, mode: WordWrap) -> Self {
        self.config.word_wrap = mode;
        self
    }

    /// Set read-only mode
    pub fn read_only(mut self, enabled: bool) -> Self {
        self.config.read_only = enabled;
        self
    }

    /// Set the font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.config.font_size = size;
        self
    }

    /// Set the font family
    pub fn font_family(mut self, family: &str) -> Self {
        self.config.font_family = family.to_string();
        self
    }

    /// Set the line height multiplier
    pub fn line_height(mut self, height: f32) -> Self {
        self.config.line_height = height;
        self
    }

    /// Set the change callback
    pub fn on_change<F: Fn(&str) + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Set the event handler
    pub fn on_event<F: Fn(EditorEvent) + 'static>(mut self, handler: F) -> Self {
        self.on_event = Some(Box::new(handler));
        self
    }

    /// Enable or disable bracket matching
    pub fn bracket_matching(mut self, enabled: bool) -> Self {
        self.config.bracket_matching = enabled;
        self
    }

    /// Enable or disable auto-indent
    pub fn auto_indent(mut self, enabled: bool) -> Self {
        self.config.auto_indent = enabled;
        self
    }

    /// Enable or disable auto-close brackets
    pub fn auto_close_brackets(mut self, enabled: bool) -> Self {
        self.config.auto_close_brackets = enabled;
        self
    }

    /// Highlight the current line
    pub fn highlight_current_line(mut self, enabled: bool) -> Self {
        self.config.highlight_current_line = enabled;
        self
    }

    /// Show whitespace characters
    pub fn show_whitespace(mut self, enabled: bool) -> Self {
        self.config.show_whitespace = enabled;
        self
    }

    /// Enable or disable autocomplete
    pub fn autocomplete_enabled(mut self, enabled: bool) -> Self {
        self.config.autocomplete = enabled;
        self
    }

    /// Add a completion provider
    pub fn add_completion_provider(mut self, provider: Box<dyn CompletionProvider>) -> Self {
        self.completion_providers.push(provider);
        self
    }

    /// Set the folding provider
    pub fn folding_provider(mut self, provider: Box<dyn FoldingProvider>) -> Self {
        self.folding_provider = Some(provider);
        self
    }

    /// Get the current content
    pub fn get_content(&self) -> String {
        self.state.document.to_string()
    }

    /// Set the content
    pub fn set_content(&mut self, content: &str) {
        self.state.document = Rope::from_str(content);
        self.highlighter.highlight_all(&self.state.document);
        self.emit_change();
    }

    /// Get the current language
    pub fn get_language(&self) -> Language {
        self.language
    }

    /// Get the current theme
    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    /// Get the configuration
    pub fn get_config(&self) -> &EditorConfig {
        &self.config
    }

    /// Get the current state
    pub fn get_state(&self) -> &EditorState {
        &self.state
    }

    /// Get a mutable reference to the state
    pub fn get_state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.state.document.len_lines()
    }

    /// Get the text of a specific line
    pub fn get_line(&self, line: usize) -> EditorResult<String> {
        let line_count = self.line_count();
        if line >= line_count {
            return Err(EditorError::LineOutOfRange(line, line_count));
        }
        Ok(self.state.document.line(line).to_string())
    }

    /// Get the length of a specific line
    pub fn get_line_length(&self, line: usize) -> EditorResult<usize> {
        let line_count = self.line_count();
        if line >= line_count {
            return Err(EditorError::LineOutOfRange(line, line_count));
        }
        let line_text = self.state.document.line(line);
        // Don't count the newline character
        let len = line_text.len_chars();
        if len > 0 && line_text.char(len - 1) == '\n' {
            Ok(len - 1)
        } else {
            Ok(len)
        }
    }

    /// Insert text at the current cursor position
    pub fn insert(&mut self, text: &str) -> EditorResult<()> {
        if self.config.read_only {
            return Ok(());
        }

        let cursor = self.state.cursors.primary();
        let char_idx = self.position_to_char_index(cursor.position)?;

        // Record for undo
        let old_cursors: Vec<_> = self.state.cursors.all().iter().map(|c| c.position).collect();

        self.state.document.insert(char_idx, text);

        // Update cursor position
        let new_pos = self.char_index_to_position(char_idx + text.chars().count())?;
        self.state.cursors.set_primary_position(new_pos);

        // Update highlighting incrementally
        self.highlighter
            .highlight_range(&self.state.document, cursor.position.line, new_pos.line + 1);

        let new_cursors: Vec<_> = self.state.cursors.all().iter().map(|c| c.position).collect();

        // Push undo operation
        self.state.undo_stack.push(EditOperation {
            range: Range::new(cursor.position, new_pos),
            old_text: String::new(),
            new_text: text.to_string(),
            old_cursors,
            new_cursors,
        });
        self.state.redo_stack.clear();

        self.emit_change();
        Ok(())
    }

    /// Delete text in a range
    pub fn delete(&mut self, range: Range) -> EditorResult<String> {
        if self.config.read_only {
            return Ok(String::new());
        }

        let range = range.normalize();
        let start_idx = self.position_to_char_index(range.start)?;
        let end_idx = self.position_to_char_index(range.end)?;

        let old_text: String = self.state.document.slice(start_idx..end_idx).to_string();
        let old_cursors: Vec<_> = self.state.cursors.all().iter().map(|c| c.position).collect();

        self.state.document.remove(start_idx..end_idx);

        // Update cursor
        self.state.cursors.set_primary_position(range.start);

        // Update highlighting
        self.highlighter
            .highlight_range(&self.state.document, range.start.line, range.start.line + 1);

        let new_cursors: Vec<_> = self.state.cursors.all().iter().map(|c| c.position).collect();

        // Push undo operation
        self.state.undo_stack.push(EditOperation {
            range,
            old_text: old_text.clone(),
            new_text: String::new(),
            old_cursors,
            new_cursors,
        });
        self.state.redo_stack.clear();

        self.emit_change();
        Ok(old_text)
    }

    /// Replace text in a range
    pub fn replace(&mut self, range: Range, new_text: &str) -> EditorResult<String> {
        if self.config.read_only {
            return Ok(String::new());
        }

        let range = range.normalize();
        let start_idx = self.position_to_char_index(range.start)?;
        let end_idx = self.position_to_char_index(range.end)?;

        let old_text: String = self.state.document.slice(start_idx..end_idx).to_string();
        let old_cursors: Vec<_> = self.state.cursors.all().iter().map(|c| c.position).collect();

        self.state.document.remove(start_idx..end_idx);
        self.state.document.insert(start_idx, new_text);

        // Update cursor
        let new_end = self.char_index_to_position(start_idx + new_text.chars().count())?;
        self.state.cursors.set_primary_position(new_end);

        // Update highlighting
        self.highlighter
            .highlight_range(&self.state.document, range.start.line, new_end.line + 1);

        let new_cursors: Vec<_> = self.state.cursors.all().iter().map(|c| c.position).collect();

        // Push undo operation
        self.state.undo_stack.push(EditOperation {
            range,
            old_text: old_text.clone(),
            new_text: new_text.to_string(),
            old_cursors,
            new_cursors,
        });
        self.state.redo_stack.clear();

        self.emit_change();
        Ok(old_text)
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> EditorResult<bool> {
        if let Some(op) = self.state.undo_stack.pop() {
            let start_idx = self.position_to_char_index(op.range.start)?;
            let new_text_len = op.new_text.chars().count();

            // Remove the new text and insert the old text
            if new_text_len > 0 {
                self.state.document.remove(start_idx..start_idx + new_text_len);
            }
            if !op.old_text.is_empty() {
                self.state.document.insert(start_idx, &op.old_text);
            }

            // Restore cursors
            self.state.cursors.restore_positions(&op.old_cursors);

            // Update highlighting
            self.highlighter.highlight_all(&self.state.document);

            // Move to redo stack
            self.state.redo_stack.push(op);

            self.emit_change();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redo the last undone operation
    pub fn redo(&mut self) -> EditorResult<bool> {
        if let Some(op) = self.state.redo_stack.pop() {
            let start_idx = self.position_to_char_index(op.range.start)?;
            let old_text_len = op.old_text.chars().count();

            // Remove the old text and insert the new text
            if old_text_len > 0 {
                self.state.document.remove(start_idx..start_idx + old_text_len);
            }
            if !op.new_text.is_empty() {
                self.state.document.insert(start_idx, &op.new_text);
            }

            // Restore cursors
            self.state.cursors.restore_positions(&op.new_cursors);

            // Update highlighting
            self.highlighter.highlight_all(&self.state.document);

            // Move to undo stack
            self.state.undo_stack.push(op);

            self.emit_change();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Move cursor to a position
    pub fn go_to(&mut self, position: Position) -> EditorResult<()> {
        self.validate_position(&position)?;
        self.state.cursors.set_primary_position(position);
        self.emit_cursor_move();
        Ok(())
    }

    /// Move cursor to a line number (1-indexed for user input)
    pub fn go_to_line(&mut self, line: usize) -> EditorResult<()> {
        let line = line.saturating_sub(1); // Convert to 0-indexed
        let line_count = self.line_count();
        if line >= line_count {
            return Err(EditorError::LineOutOfRange(line + 1, line_count));
        }
        self.state.cursors.set_primary_position(Position::line_start(line));
        self.emit_cursor_move();
        Ok(())
    }

    /// Select a range
    pub fn select(&mut self, range: Range) -> EditorResult<()> {
        self.validate_position(&range.start)?;
        self.validate_position(&range.end)?;
        self.state.cursors.primary_mut().set_selection(Selection {
            anchor: range.start,
            head: range.end,
        });
        self.emit_selection_change();
        Ok(())
    }

    /// Select all content
    pub fn select_all(&mut self) {
        let end_line = self.line_count().saturating_sub(1);
        let end_col = self.get_line_length(end_line).unwrap_or(0);
        self.state.cursors.primary_mut().set_selection(Selection {
            anchor: Position::new(0, 0),
            head: Position::new(end_line, end_col),
        });
        self.emit_selection_change();
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.state.cursors.primary_mut().clear_selection();
        self.emit_selection_change();
    }

    /// Toggle line comment
    pub fn toggle_comment(&mut self) -> EditorResult<()> {
        let comment_prefix = self.language.comment_prefix();
        let cursor = self.state.cursors.primary();

        // Determine affected lines
        let (start_line, end_line) = if let Some(sel) = cursor.selection.as_ref() {
            let range = sel.to_range();
            (range.start.line, range.end.line)
        } else {
            (cursor.position.line, cursor.position.line)
        };

        // Check if all lines are commented
        let all_commented = (start_line..=end_line).all(|line| {
            self.get_line(line)
                .map(|text| text.trim_start().starts_with(comment_prefix))
                .unwrap_or(false)
        });

        // Toggle comments
        for line in start_line..=end_line {
            let line_text = self.get_line(line)?;
            let trimmed = line_text.trim_start();
            let indent = line_text.len() - trimmed.len();

            if all_commented {
                // Remove comment
                if let Some(stripped) = trimmed.strip_prefix(comment_prefix) {
                    let stripped = stripped.strip_prefix(' ').unwrap_or(stripped);
                    let new_text = format!("{}{}", &line_text[..indent], stripped);
                    let range = Range::single_line(line, 0, line_text.len());
                    self.replace(range, &new_text)?;
                }
            } else {
                // Add comment
                let new_text = format!("{}{} {}", &line_text[..indent], comment_prefix, trimmed);
                let range = Range::single_line(line, 0, line_text.len());
                self.replace(range, &new_text)?;
            }
        }

        Ok(())
    }

    /// Indent selected lines
    pub fn indent(&mut self) -> EditorResult<()> {
        let indent_str = if self.config.soft_tabs {
            " ".repeat(self.config.tab_size)
        } else {
            "\t".to_string()
        };

        let cursor = self.state.cursors.primary();
        let (start_line, end_line) = if let Some(sel) = cursor.selection.as_ref() {
            let range = sel.to_range();
            (range.start.line, range.end.line)
        } else {
            (cursor.position.line, cursor.position.line)
        };

        for line in start_line..=end_line {
            let pos = Position::line_start(line);
            let char_idx = self.position_to_char_index(pos)?;
            self.state.document.insert(char_idx, &indent_str);
        }

        self.highlighter.highlight_all(&self.state.document);
        self.emit_change();
        Ok(())
    }

    /// Outdent selected lines
    pub fn outdent(&mut self) -> EditorResult<()> {
        let cursor = self.state.cursors.primary();
        let (start_line, end_line) = if let Some(sel) = cursor.selection.as_ref() {
            let range = sel.to_range();
            (range.start.line, range.end.line)
        } else {
            (cursor.position.line, cursor.position.line)
        };

        for line in start_line..=end_line {
            let line_text = self.get_line(line)?;
            let chars: Vec<char> = line_text.chars().collect();

            let remove_count = if chars.first() == Some(&'\t') {
                1
            } else {
                // Count leading spaces up to tab_size
                chars
                    .iter()
                    .take(self.config.tab_size)
                    .take_while(|c| **c == ' ')
                    .count()
            };

            if remove_count > 0 {
                let pos = Position::line_start(line);
                let char_idx = self.position_to_char_index(pos)?;
                self.state.document.remove(char_idx..char_idx + remove_count);
            }
        }

        self.highlighter.highlight_all(&self.state.document);
        self.emit_change();
        Ok(())
    }

    /// Fold code at a line
    pub fn fold_at(&mut self, line: usize) -> EditorResult<()> {
        if !self.config.folding {
            return Ok(());
        }

        if let Some(provider) = &self.folding_provider {
            if let Some(range) = provider.get_fold_range(&self.state.document, line) {
                self.state.fold_state.fold(range);
                self.emit_event(EditorEvent::FoldToggle { line, folded: true });
            } else {
                return Err(EditorError::CannotFold(line));
            }
        }
        Ok(())
    }

    /// Unfold code at a line
    pub fn unfold_at(&mut self, line: usize) -> EditorResult<()> {
        if self.state.fold_state.unfold_at(line) {
            self.emit_event(EditorEvent::FoldToggle {
                line,
                folded: false,
            });
        }
        Ok(())
    }

    /// Toggle fold at a line
    pub fn toggle_fold_at(&mut self, line: usize) -> EditorResult<()> {
        if self.state.fold_state.is_folded(line) {
            self.unfold_at(line)
        } else {
            self.fold_at(line)
        }
    }

    /// Fold all foldable regions
    pub fn fold_all(&mut self) -> EditorResult<()> {
        if let Some(provider) = &self.folding_provider {
            let ranges = provider.get_all_fold_ranges(&self.state.document);
            for range in ranges {
                self.state.fold_state.fold(range);
            }
        }
        Ok(())
    }

    /// Unfold all regions
    pub fn unfold_all(&mut self) {
        self.state.fold_state.unfold_all();
    }

    /// Get matching bracket position
    pub fn find_matching_bracket(&self, position: Position) -> Option<Position> {
        if !self.config.bracket_matching {
            return None;
        }

        let char_idx = self.position_to_char_index(position).ok()?;
        let ch = self.state.document.char(char_idx);

        let (open, close, forward) = match ch {
            '(' => ('(', ')', true),
            ')' => ('(', ')', false),
            '[' => ('[', ']', true),
            ']' => ('[', ']', false),
            '{' => ('{', '}', true),
            '}' => ('{', '}', false),
            '<' => ('<', '>', true),
            '>' => ('<', '>', false),
            _ => return None,
        };

        let mut depth = 1;
        let total_chars = self.state.document.len_chars();

        if forward {
            for i in (char_idx + 1)..total_chars {
                let c = self.state.document.char(i);
                if c == close {
                    depth -= 1;
                    if depth == 0 {
                        return self.char_index_to_position(i).ok();
                    }
                } else if c == open {
                    depth += 1;
                }
            }
        } else {
            for i in (0..char_idx).rev() {
                let c = self.state.document.char(i);
                if c == open {
                    depth -= 1;
                    if depth == 0 {
                        return self.char_index_to_position(i).ok();
                    }
                } else if c == close {
                    depth += 1;
                }
            }
        }

        None
    }

    /// Find text in the document
    pub fn find(&mut self, query: &str) -> Vec<Range> {
        self.search.find(&self.state.document, query)
    }

    /// Find with options
    pub fn find_with_options(
        &mut self,
        query: &str,
        options: &crate::features::search::FindOptions,
    ) -> EditorResult<Vec<Range>> {
        self.search
            .find_with_options(&self.state.document, query, options)
    }

    /// Replace all occurrences
    pub fn replace_all(&mut self, query: &str, replacement: &str) -> EditorResult<usize> {
        let matches = self.find(query);
        let count = matches.len();

        // Replace from end to start to preserve positions
        for range in matches.into_iter().rev() {
            self.replace(range, replacement)?;
        }

        Ok(count)
    }

    /// Add a diagnostic marker
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.state.diagnostics.push(diagnostic);
    }

    /// Clear all diagnostics
    pub fn clear_diagnostics(&mut self) {
        self.state.diagnostics.clear();
    }

    /// Get diagnostics for a line
    pub fn get_diagnostics_at_line(&self, line: usize) -> Vec<&Diagnostic> {
        self.state
            .diagnostics
            .iter()
            .filter(|d| d.range.start.line <= line && d.range.end.line >= line)
            .collect()
    }

    /// Highlight specific lines
    pub fn highlight_lines(&mut self, lines: &[usize]) {
        self.state.highlighted_lines.clear();
        self.state.highlighted_lines.extend(lines.iter());
    }

    /// Clear line highlights
    pub fn clear_line_highlights(&mut self) {
        self.state.highlighted_lines.clear();
    }

    // Helper methods

    fn position_to_char_index(&self, pos: Position) -> EditorResult<usize> {
        let line_count = self.state.document.len_lines();
        if pos.line >= line_count {
            return Err(EditorError::LineOutOfRange(pos.line, line_count));
        }

        let line_start = self.state.document.line_to_char(pos.line);
        let line = self.state.document.line(pos.line);
        let line_len = line.len_chars();

        // Allow position at end of line (after last char)
        let max_col = if line_len > 0 && line.char(line_len - 1) == '\n' {
            line_len - 1
        } else {
            line_len
        };

        if pos.column > max_col {
            return Err(EditorError::ColumnOutOfRange(pos.column, pos.line, max_col));
        }

        Ok(line_start + pos.column)
    }

    fn char_index_to_position(&self, char_idx: usize) -> EditorResult<Position> {
        let line = self.state.document.char_to_line(char_idx);
        let line_start = self.state.document.line_to_char(line);
        let column = char_idx - line_start;
        Ok(Position::new(line, column))
    }

    fn validate_position(&self, pos: &Position) -> EditorResult<()> {
        let line_count = self.line_count();
        if pos.line >= line_count {
            return Err(EditorError::LineOutOfRange(pos.line, line_count));
        }
        let line_len = self.get_line_length(pos.line)?;
        if pos.column > line_len {
            return Err(EditorError::ColumnOutOfRange(pos.column, pos.line, line_len));
        }
        Ok(())
    }

    fn emit_change(&self) {
        if let Some(callback) = &self.on_change {
            callback(&self.state.document.to_string());
        }
        if let Some(handler) = &self.on_event {
            handler(EditorEvent::Change {
                content: self.state.document.to_string(),
            });
        }
    }

    fn emit_cursor_move(&self) {
        if let Some(handler) = &self.on_event {
            handler(EditorEvent::CursorMove {
                positions: self.state.cursors.all().iter().map(|c| c.position).collect(),
            });
        }
    }

    fn emit_selection_change(&self) {
        if let Some(handler) = &self.on_event {
            let selections: Vec<_> = self
                .state
                .cursors
                .all()
                .iter()
                .filter_map(|c| c.selection.clone())
                .collect();
            handler(EditorEvent::SelectionChange { selections });
        }
    }

    fn emit_event(&self, event: EditorEvent) {
        if let Some(handler) = &self.on_event {
            handler(event);
        }
    }
}

/// A read-only code block for displaying code
#[derive(Debug)]
pub struct CodeBlock {
    /// The code content
    code: String,
    /// Language for syntax highlighting
    language: Language,
    /// Theme
    theme: Theme,
    /// Show line numbers
    line_numbers: bool,
    /// Show copy button
    copy_button: bool,
    /// Lines to highlight
    highlighted_lines: Vec<usize>,
    /// First visible line (for virtual scrolling)
    start_line: usize,
    /// Maximum height in lines (0 = unlimited)
    max_lines: usize,
    /// Show language badge
    show_language: bool,
    /// Caption/title
    caption: Option<String>,
}

impl Default for CodeBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeBlock {
    /// Create a new code block
    pub fn new() -> Self {
        Self {
            code: String::new(),
            language: Language::PlainText,
            theme: Theme::dark(),
            line_numbers: true,
            copy_button: true,
            highlighted_lines: Vec::new(),
            start_line: 1,
            max_lines: 0,
            show_language: true,
            caption: None,
        }
    }

    /// Set the code content
    pub fn code(mut self, code: &str) -> Self {
        self.code = code.to_string();
        self
    }

    /// Set the language
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Set the theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Enable or disable line numbers
    pub fn line_numbers(mut self, enabled: bool) -> Self {
        self.line_numbers = enabled;
        self
    }

    /// Enable or disable copy button
    pub fn copy_button(mut self, enabled: bool) -> Self {
        self.copy_button = enabled;
        self
    }

    /// Set lines to highlight (1-indexed)
    pub fn highlight_lines(mut self, lines: &[usize]) -> Self {
        self.highlighted_lines = lines.to_vec();
        self
    }

    /// Set the starting line number
    pub fn start_line(mut self, line: usize) -> Self {
        self.start_line = line;
        self
    }

    /// Set maximum visible lines
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = lines;
        self
    }

    /// Show or hide language badge
    pub fn show_language(mut self, show: bool) -> Self {
        self.show_language = show;
        self
    }

    /// Set a caption/title
    pub fn caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    /// Get the code content
    pub fn get_code(&self) -> &str {
        &self.code
    }

    /// Get the language
    pub fn get_language(&self) -> Language {
        self.language
    }

    /// Get highlighted lines
    pub fn get_highlighted_lines(&self) -> &[usize] {
        &self.highlighted_lines
    }

    /// Check if line numbers are enabled
    pub fn has_line_numbers(&self) -> bool {
        self.line_numbers
    }

    /// Check if copy button is enabled
    pub fn has_copy_button(&self) -> bool {
        self.copy_button
    }

    /// Get the caption
    pub fn get_caption(&self) -> Option<&str> {
        self.caption.as_deref()
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.code.lines().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = CodeEditor::new()
            .language(Language::Rust)
            .theme(Theme::dark())
            .value("fn main() {}")
            .line_numbers(true)
            .folding(true);

        assert_eq!(editor.get_language(), Language::Rust);
        assert!(editor.get_config().line_numbers);
        assert!(editor.get_config().folding);
        assert_eq!(editor.get_content(), "fn main() {}");
    }

    #[test]
    fn test_editor_insert() {
        let mut editor = CodeEditor::new().value("Hello");

        editor.go_to(Position::new(0, 5)).unwrap();
        editor.insert(" World").unwrap();

        assert_eq!(editor.get_content(), "Hello World");
    }

    #[test]
    fn test_editor_delete() {
        let mut editor = CodeEditor::new().value("Hello World");

        let range = Range::single_line(0, 5, 11);
        editor.delete(range).unwrap();

        assert_eq!(editor.get_content(), "Hello");
    }

    #[test]
    fn test_editor_replace() {
        let mut editor = CodeEditor::new().value("Hello World");

        let range = Range::single_line(0, 6, 11);
        editor.replace(range, "Rust").unwrap();

        assert_eq!(editor.get_content(), "Hello Rust");
    }

    #[test]
    fn test_editor_undo_redo() {
        let mut editor = CodeEditor::new().value("Hello");

        editor.go_to(Position::new(0, 5)).unwrap();
        editor.insert(" World").unwrap();
        assert_eq!(editor.get_content(), "Hello World");

        editor.undo().unwrap();
        assert_eq!(editor.get_content(), "Hello");

        editor.redo().unwrap();
        assert_eq!(editor.get_content(), "Hello World");
    }

    #[test]
    fn test_editor_line_operations() {
        let editor = CodeEditor::new().value("Line 1\nLine 2\nLine 3");

        assert_eq!(editor.line_count(), 3);
        assert_eq!(editor.get_line(0).unwrap(), "Line 1\n");
        assert_eq!(editor.get_line(1).unwrap(), "Line 2\n");
        assert_eq!(editor.get_line(2).unwrap(), "Line 3");
    }

    #[test]
    fn test_editor_read_only() {
        let mut editor = CodeEditor::new().value("Hello").read_only(true);

        editor.go_to(Position::new(0, 5)).unwrap();
        editor.insert(" World").unwrap();

        // Content should be unchanged in read-only mode
        assert_eq!(editor.get_content(), "Hello");
    }

    #[test]
    fn test_code_block_creation() {
        let block = CodeBlock::new()
            .code("const x = 1;")
            .language(Language::JavaScript)
            .line_numbers(true)
            .copy_button(true)
            .highlight_lines(&[1]);

        assert_eq!(block.get_code(), "const x = 1;");
        assert_eq!(block.get_language(), Language::JavaScript);
        assert!(block.has_line_numbers());
        assert!(block.has_copy_button());
        assert_eq!(block.get_highlighted_lines(), &[1]);
    }

    #[test]
    fn test_go_to_line() {
        let mut editor = CodeEditor::new().value("Line 1\nLine 2\nLine 3");

        editor.go_to_line(2).unwrap();
        assert_eq!(editor.get_state().cursors.primary().position.line, 1);

        editor.go_to_line(3).unwrap();
        assert_eq!(editor.get_state().cursors.primary().position.line, 2);
    }

    #[test]
    fn test_select_all() {
        let mut editor = CodeEditor::new().value("Hello\nWorld");

        editor.select_all();

        let selection = editor.get_state().cursors.primary().selection.as_ref().unwrap();
        assert_eq!(selection.anchor, Position::new(0, 0));
        assert_eq!(selection.head, Position::new(1, 5));
    }
}
