//! Markdown Parser
//!
//! Provides markdown parsing using pulldown-cmark with CommonMark and GFM support.

use crate::{HeadingLevel, MarkdownError, MarkdownResult, Position, Range};
use pulldown_cmark::{
    Alignment, CodeBlockKind, Event, HeadingLevel as CmarkHeadingLevel, Options, Parser, Tag,
    TagEnd,
};
use std::collections::HashMap;

/// Error during parsing
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Location of the error
    pub location: Option<Position>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = &self.location {
            write!(f, "Parse error at {}:{}: {}", loc.line, loc.column, self.message)
        } else {
            write!(f, "Parse error: {}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// Table column alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

impl From<Alignment> for TableAlignment {
    fn from(align: Alignment) -> Self {
        match align {
            Alignment::None => TableAlignment::None,
            Alignment::Left => TableAlignment::Left,
            Alignment::Center => TableAlignment::Center,
            Alignment::Right => TableAlignment::Right,
        }
    }
}

/// List type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListType {
    /// Unordered list (bullet points)
    Unordered,
    /// Ordered list with starting number
    Ordered(u64),
}

/// A table cell
#[derive(Debug, Clone, Default)]
pub struct TableCell {
    /// Cell content
    pub content: Vec<InlineElement>,
    /// Whether this is a header cell
    pub is_header: bool,
    /// Column alignment
    pub alignment: TableAlignment,
}

/// A table row
#[derive(Debug, Clone, Default)]
pub struct TableRow {
    /// Cells in this row
    pub cells: Vec<TableCell>,
    /// Whether this is the header row
    pub is_header: bool,
}

/// A list item
#[derive(Debug, Clone)]
pub struct ListItem {
    /// Item content (blocks)
    pub content: Vec<BlockElement>,
    /// Task checkbox state (None if not a task item)
    pub task_checked: Option<bool>,
    /// Nested list (if any)
    pub nested_list: Option<Box<BlockElement>>,
}

/// Inline markdown elements
#[derive(Debug, Clone)]
pub enum InlineElement {
    /// Plain text
    Text(String),
    /// Soft line break
    SoftBreak,
    /// Hard line break
    HardBreak,
    /// Bold/strong text
    Strong(Vec<InlineElement>),
    /// Italic/emphasis text
    Emphasis(Vec<InlineElement>),
    /// Strikethrough text (GFM)
    Strikethrough(Vec<InlineElement>),
    /// Inline code
    Code(String),
    /// Link with URL and optional title
    Link {
        url: String,
        title: Option<String>,
        content: Vec<InlineElement>,
    },
    /// Image with URL and optional title
    Image {
        url: String,
        title: Option<String>,
        alt: String,
    },
    /// HTML inline content
    Html(String),
    /// Footnote reference
    FootnoteReference(String),
}

/// Block-level markdown elements
#[derive(Debug, Clone)]
pub enum BlockElement {
    /// Heading with level and content
    Heading {
        level: HeadingLevel,
        content: Vec<InlineElement>,
        id: Option<String>,
    },
    /// Paragraph
    Paragraph(Vec<InlineElement>),
    /// Code block with optional language
    CodeBlock {
        language: Option<String>,
        content: String,
        filename: Option<String>,
    },
    /// Blockquote
    Blockquote(Vec<BlockElement>),
    /// Unordered or ordered list
    List {
        list_type: ListType,
        items: Vec<ListItem>,
    },
    /// Table (GFM)
    Table {
        alignments: Vec<TableAlignment>,
        header: TableRow,
        rows: Vec<TableRow>,
    },
    /// Horizontal rule
    HorizontalRule,
    /// Raw HTML block
    Html(String),
    /// Footnote definition
    FootnoteDefinition {
        label: String,
        content: Vec<BlockElement>,
    },
    /// Definition list (extension)
    DefinitionList {
        items: Vec<DefinitionItem>,
    },
}

/// Definition list item
#[derive(Debug, Clone)]
pub struct DefinitionItem {
    /// Term being defined
    pub term: Vec<InlineElement>,
    /// Definitions for the term
    pub definitions: Vec<Vec<BlockElement>>,
}

/// Parsed markdown document
#[derive(Debug, Clone, Default)]
pub struct MarkdownDocument {
    /// Block elements in the document
    pub blocks: Vec<BlockElement>,
    /// Footnote definitions found
    pub footnotes: HashMap<String, Vec<BlockElement>>,
    /// Link references
    pub link_references: HashMap<String, (String, Option<String>)>,
}

impl MarkdownDocument {
    /// Create an empty document
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all headings in the document
    pub fn headings(&self) -> Vec<(&HeadingLevel, &[InlineElement], Option<&str>)> {
        let mut headings = Vec::new();
        self.collect_headings(&self.blocks, &mut headings);
        headings
    }

    fn collect_headings<'a>(
        &'a self,
        blocks: &'a [BlockElement],
        headings: &mut Vec<(&'a HeadingLevel, &'a [InlineElement], Option<&'a str>)>,
    ) {
        for block in blocks {
            match block {
                BlockElement::Heading { level, content, id } => {
                    headings.push((level, content, id.as_deref()));
                }
                BlockElement::Blockquote(inner) => {
                    self.collect_headings(inner, headings);
                }
                _ => {}
            }
        }
    }

    /// Get plain text content of the document
    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        self.extract_text(&self.blocks, &mut text);
        text
    }

    fn extract_text(&self, blocks: &[BlockElement], text: &mut String) {
        for block in blocks {
            match block {
                BlockElement::Heading { content, .. } => {
                    self.extract_inline_text(content, text);
                    text.push('\n');
                }
                BlockElement::Paragraph(content) => {
                    self.extract_inline_text(content, text);
                    text.push('\n');
                }
                BlockElement::CodeBlock { content, .. } => {
                    text.push_str(content);
                    text.push('\n');
                }
                BlockElement::Blockquote(inner) => {
                    self.extract_text(inner, text);
                }
                BlockElement::List { items, .. } => {
                    for item in items {
                        self.extract_text(&item.content, text);
                    }
                }
                BlockElement::Table { header, rows, .. } => {
                    for cell in &header.cells {
                        self.extract_inline_text(&cell.content, text);
                        text.push('\t');
                    }
                    text.push('\n');
                    for row in rows {
                        for cell in &row.cells {
                            self.extract_inline_text(&cell.content, text);
                            text.push('\t');
                        }
                        text.push('\n');
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_inline_text(&self, inlines: &[InlineElement], text: &mut String) {
        for inline in inlines {
            match inline {
                InlineElement::Text(t) => text.push_str(t),
                InlineElement::SoftBreak | InlineElement::HardBreak => text.push(' '),
                InlineElement::Strong(inner) | InlineElement::Emphasis(inner) | InlineElement::Strikethrough(inner) => {
                    self.extract_inline_text(inner, text);
                }
                InlineElement::Code(c) => text.push_str(c),
                InlineElement::Link { content, .. } => {
                    self.extract_inline_text(content, text);
                }
                InlineElement::Image { alt, .. } => {
                    text.push_str(alt);
                }
                _ => {}
            }
        }
    }
}

/// Markdown parser with CommonMark and GFM support
pub struct MarkdownParser {
    /// Enable GFM extensions
    gfm: bool,
    /// Enable footnotes
    footnotes: bool,
    /// Enable smart punctuation
    smart_punctuation: bool,
    /// Enable heading attributes
    heading_attributes: bool,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    /// Create a new parser with default options
    pub fn new() -> Self {
        Self {
            gfm: true,
            footnotes: true,
            smart_punctuation: false,
            heading_attributes: true,
        }
    }

    /// Enable or disable GFM extensions
    pub fn gfm(mut self, enable: bool) -> Self {
        self.gfm = enable;
        self
    }

    /// Enable or disable footnotes
    pub fn footnotes(mut self, enable: bool) -> Self {
        self.footnotes = enable;
        self
    }

    /// Enable or disable smart punctuation
    pub fn smart_punctuation(mut self, enable: bool) -> Self {
        self.smart_punctuation = enable;
        self
    }

    /// Enable or disable heading attributes
    pub fn heading_attributes(mut self, enable: bool) -> Self {
        self.heading_attributes = enable;
        self
    }

    /// Build parser options
    fn build_options(&self) -> Options {
        let mut options = Options::empty();

        if self.gfm {
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TASKLISTS);
        }

        if self.footnotes {
            options.insert(Options::ENABLE_FOOTNOTES);
        }

        if self.smart_punctuation {
            options.insert(Options::ENABLE_SMART_PUNCTUATION);
        }

        if self.heading_attributes {
            options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        }

        options
    }

    /// Parse markdown content into a document
    pub fn parse(&self, content: &str) -> MarkdownResult<MarkdownDocument> {
        let options = self.build_options();
        let parser = Parser::new_ext(content, options);

        let mut document = MarkdownDocument::new();
        let mut block_stack: Vec<Vec<BlockElement>> = vec![Vec::new()];
        let mut inline_stack: Vec<Vec<InlineElement>> = Vec::new();
        let mut current_list_items: Vec<Vec<ListItem>> = Vec::new();
        let mut current_list_types: Vec<ListType> = Vec::new();
        let mut table_state: Option<TableParseState> = None;
        let mut current_footnote: Option<String> = None;

        for event in parser {
            match event {
                Event::Start(tag) => {
                    self.handle_start_tag(
                        tag,
                        &mut block_stack,
                        &mut inline_stack,
                        &mut current_list_items,
                        &mut current_list_types,
                        &mut table_state,
                        &mut current_footnote,
                    )?;
                }
                Event::End(tag_end) => {
                    self.handle_end_tag(
                        tag_end,
                        &mut document,
                        &mut block_stack,
                        &mut inline_stack,
                        &mut current_list_items,
                        &mut current_list_types,
                        &mut table_state,
                        &mut current_footnote,
                    )?;
                }
                Event::Text(text) => {
                    if let Some(inlines) = inline_stack.last_mut() {
                        inlines.push(InlineElement::Text(text.to_string()));
                    } else if let Some(ref mut state) = table_state {
                        if let Some(cell) = state.current_row.cells.last_mut() {
                            cell.content.push(InlineElement::Text(text.to_string()));
                        }
                    }
                }
                Event::Code(code) => {
                    if let Some(inlines) = inline_stack.last_mut() {
                        inlines.push(InlineElement::Code(code.to_string()));
                    } else if let Some(ref mut state) = table_state {
                        if let Some(cell) = state.current_row.cells.last_mut() {
                            cell.content.push(InlineElement::Code(code.to_string()));
                        }
                    }
                }
                Event::SoftBreak => {
                    if let Some(inlines) = inline_stack.last_mut() {
                        inlines.push(InlineElement::SoftBreak);
                    }
                }
                Event::HardBreak => {
                    if let Some(inlines) = inline_stack.last_mut() {
                        inlines.push(InlineElement::HardBreak);
                    }
                }
                Event::Html(html) => {
                    let html_str = html.to_string();
                    if inline_stack.is_empty() {
                        if let Some(blocks) = block_stack.last_mut() {
                            blocks.push(BlockElement::Html(html_str));
                        }
                    } else if let Some(inlines) = inline_stack.last_mut() {
                        inlines.push(InlineElement::Html(html_str));
                    }
                }
                Event::InlineHtml(html) => {
                    if let Some(inlines) = inline_stack.last_mut() {
                        inlines.push(InlineElement::Html(html.to_string()));
                    }
                }
                Event::Rule => {
                    if let Some(blocks) = block_stack.last_mut() {
                        blocks.push(BlockElement::HorizontalRule);
                    }
                }
                Event::FootnoteReference(label) => {
                    if let Some(inlines) = inline_stack.last_mut() {
                        inlines.push(InlineElement::FootnoteReference(label.to_string()));
                    }
                }
                Event::TaskListMarker(checked) => {
                    if let Some(items) = current_list_items.last_mut() {
                        if let Some(item) = items.last_mut() {
                            item.task_checked = Some(checked);
                        }
                    }
                }
                Event::InlineMath(math) => {
                    // Inline math (e.g., $x^2$)
                    if let Some(inlines) = inline_stack.last_mut() {
                        inlines.push(InlineElement::Code(format!("${}$", math)));
                    }
                }
                Event::DisplayMath(math) => {
                    // Display math (e.g., $$x^2$$)
                    if let Some(blocks) = block_stack.last_mut() {
                        blocks.push(BlockElement::CodeBlock {
                            language: Some("math".to_string()),
                            content: math.to_string(),
                            filename: None,
                        });
                    }
                }
            }
        }

        // Final blocks should be in the first level
        if let Some(blocks) = block_stack.pop() {
            document.blocks = blocks;
        }

        Ok(document)
    }

    fn handle_start_tag(
        &self,
        tag: Tag<'_>,
        block_stack: &mut Vec<Vec<BlockElement>>,
        inline_stack: &mut Vec<Vec<InlineElement>>,
        current_list_items: &mut Vec<Vec<ListItem>>,
        current_list_types: &mut Vec<ListType>,
        table_state: &mut Option<TableParseState>,
        current_footnote: &mut Option<String>,
    ) -> MarkdownResult<()> {
        match tag {
            Tag::Paragraph => {
                inline_stack.push(Vec::new());
            }
            Tag::Heading { level, id, .. } => {
                inline_stack.push(Vec::new());
                // Store heading info for later
                let _ = (level, id);
            }
            Tag::BlockQuote(_kind) => {
                block_stack.push(Vec::new());
            }
            Tag::CodeBlock(kind) => {
                // Code blocks are handled differently - we collect text directly
                let (language, filename) = match kind {
                    CodeBlockKind::Fenced(info) => {
                        let info_str = info.to_string();
                        let mut parts = info_str.split_whitespace();
                        let lang = parts.next().map(|s| s.to_string()).filter(|s| !s.is_empty());
                        let file = parts.next().map(|s| s.to_string());
                        (lang, file)
                    }
                    CodeBlockKind::Indented => (None, None),
                };
                // Push placeholder
                if let Some(blocks) = block_stack.last_mut() {
                    blocks.push(BlockElement::CodeBlock {
                        language,
                        content: String::new(),
                        filename,
                    });
                }
            }
            Tag::List(start_num) => {
                let list_type = match start_num {
                    Some(n) => ListType::Ordered(n),
                    None => ListType::Unordered,
                };
                current_list_types.push(list_type);
                current_list_items.push(Vec::new());
            }
            Tag::Item => {
                block_stack.push(Vec::new());
            }
            Tag::FootnoteDefinition(label) => {
                *current_footnote = Some(label.to_string());
                block_stack.push(Vec::new());
            }
            Tag::Table(alignments) => {
                *table_state = Some(TableParseState {
                    alignments: alignments.iter().map(|a| TableAlignment::from(*a)).collect(),
                    header: TableRow::default(),
                    rows: Vec::new(),
                    current_row: TableRow::default(),
                    in_header: true,
                });
            }
            Tag::TableHead => {
                if let Some(ref mut state) = table_state {
                    state.in_header = true;
                    state.current_row = TableRow {
                        cells: Vec::new(),
                        is_header: true,
                    };
                }
            }
            Tag::TableRow => {
                if let Some(ref mut state) = table_state {
                    state.current_row = TableRow {
                        cells: Vec::new(),
                        is_header: state.in_header,
                    };
                }
            }
            Tag::TableCell => {
                if let Some(ref mut state) = table_state {
                    let col_idx = state.current_row.cells.len();
                    let alignment = state.alignments.get(col_idx).copied().unwrap_or_default();
                    state.current_row.cells.push(TableCell {
                        content: Vec::new(),
                        is_header: state.in_header,
                        alignment,
                    });
                }
            }
            Tag::Emphasis => {
                inline_stack.push(Vec::new());
            }
            Tag::Strong => {
                inline_stack.push(Vec::new());
            }
            Tag::Strikethrough => {
                inline_stack.push(Vec::new());
            }
            Tag::Link { dest_url, title, .. } => {
                inline_stack.push(Vec::new());
                // Store link info for later
                let _ = (dest_url, title);
            }
            Tag::Image { dest_url, title, .. } => {
                inline_stack.push(Vec::new());
                // Store image info for later
                let _ = (dest_url, title);
            }
            _ => {}
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_end_tag(
        &self,
        tag_end: TagEnd,
        document: &mut MarkdownDocument,
        block_stack: &mut Vec<Vec<BlockElement>>,
        inline_stack: &mut Vec<Vec<InlineElement>>,
        current_list_items: &mut Vec<Vec<ListItem>>,
        current_list_types: &mut Vec<ListType>,
        table_state: &mut Option<TableParseState>,
        current_footnote: &mut Option<String>,
    ) -> MarkdownResult<()> {
        match tag_end {
            TagEnd::Paragraph => {
                let content = inline_stack.pop().unwrap_or_default();
                if let Some(blocks) = block_stack.last_mut() {
                    blocks.push(BlockElement::Paragraph(content));
                }
            }
            TagEnd::Heading(level) => {
                let content = inline_stack.pop().unwrap_or_default();
                let heading_level = match level {
                    CmarkHeadingLevel::H1 => HeadingLevel::H1,
                    CmarkHeadingLevel::H2 => HeadingLevel::H2,
                    CmarkHeadingLevel::H3 => HeadingLevel::H3,
                    CmarkHeadingLevel::H4 => HeadingLevel::H4,
                    CmarkHeadingLevel::H5 => HeadingLevel::H5,
                    CmarkHeadingLevel::H6 => HeadingLevel::H6,
                };
                if let Some(blocks) = block_stack.last_mut() {
                    blocks.push(BlockElement::Heading {
                        level: heading_level,
                        content,
                        id: None,
                    });
                }
            }
            TagEnd::BlockQuote(_kind) => {
                let inner_blocks = block_stack.pop().unwrap_or_default();
                if let Some(blocks) = block_stack.last_mut() {
                    blocks.push(BlockElement::Blockquote(inner_blocks));
                }
            }
            TagEnd::CodeBlock => {
                // Code block content was accumulated as Text events
                // Update the last code block with accumulated content
            }
            TagEnd::List(_ordered) => {
                let items = current_list_items.pop().unwrap_or_default();
                let list_type = current_list_types.pop().unwrap_or(ListType::Unordered);
                if let Some(blocks) = block_stack.last_mut() {
                    blocks.push(BlockElement::List { list_type, items });
                }
            }
            TagEnd::Item => {
                let item_blocks = block_stack.pop().unwrap_or_default();
                if let Some(items) = current_list_items.last_mut() {
                    items.push(ListItem {
                        content: item_blocks,
                        task_checked: None,
                        nested_list: None,
                    });
                }
            }
            TagEnd::FootnoteDefinition => {
                let content = block_stack.pop().unwrap_or_default();
                if let Some(label) = current_footnote.take() {
                    document.footnotes.insert(label.clone(), content.clone());
                    if let Some(blocks) = block_stack.last_mut() {
                        blocks.push(BlockElement::FootnoteDefinition { label, content });
                    }
                }
            }
            TagEnd::Table => {
                if let Some(state) = table_state.take() {
                    if let Some(blocks) = block_stack.last_mut() {
                        blocks.push(BlockElement::Table {
                            alignments: state.alignments,
                            header: state.header,
                            rows: state.rows,
                        });
                    }
                }
            }
            TagEnd::TableHead => {
                if let Some(ref mut state) = table_state {
                    state.header = std::mem::take(&mut state.current_row);
                    state.in_header = false;
                }
            }
            TagEnd::TableRow => {
                if let Some(ref mut state) = table_state {
                    if !state.in_header {
                        let row = std::mem::take(&mut state.current_row);
                        state.rows.push(row);
                    }
                }
            }
            TagEnd::TableCell => {
                // Cell is already in current_row
            }
            TagEnd::Emphasis => {
                let content = inline_stack.pop().unwrap_or_default();
                if let Some(parent) = inline_stack.last_mut() {
                    parent.push(InlineElement::Emphasis(content));
                } else if let Some(ref mut state) = table_state {
                    if let Some(cell) = state.current_row.cells.last_mut() {
                        cell.content.push(InlineElement::Emphasis(content));
                    }
                }
            }
            TagEnd::Strong => {
                let content = inline_stack.pop().unwrap_or_default();
                if let Some(parent) = inline_stack.last_mut() {
                    parent.push(InlineElement::Strong(content));
                } else if let Some(ref mut state) = table_state {
                    if let Some(cell) = state.current_row.cells.last_mut() {
                        cell.content.push(InlineElement::Strong(content));
                    }
                }
            }
            TagEnd::Strikethrough => {
                let content = inline_stack.pop().unwrap_or_default();
                if let Some(parent) = inline_stack.last_mut() {
                    parent.push(InlineElement::Strikethrough(content));
                } else if let Some(ref mut state) = table_state {
                    if let Some(cell) = state.current_row.cells.last_mut() {
                        cell.content.push(InlineElement::Strikethrough(content));
                    }
                }
            }
            TagEnd::Link => {
                let content = inline_stack.pop().unwrap_or_default();
                if let Some(parent) = inline_stack.last_mut() {
                    // Link URL/title would need to be tracked from start tag
                    parent.push(InlineElement::Link {
                        url: String::new(),
                        title: None,
                        content,
                    });
                }
            }
            TagEnd::Image => {
                let content = inline_stack.pop().unwrap_or_default();
                let alt = content
                    .iter()
                    .filter_map(|i| match i {
                        InlineElement::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("");
                if let Some(parent) = inline_stack.last_mut() {
                    // Image URL/title would need to be tracked from start tag
                    parent.push(InlineElement::Image {
                        url: String::new(),
                        title: None,
                        alt,
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Internal state for parsing tables
struct TableParseState {
    alignments: Vec<TableAlignment>,
    header: TableRow,
    rows: Vec<TableRow>,
    current_row: TableRow,
    in_header: bool,
}

/// Parse markdown content with default options
pub fn parse(content: &str) -> MarkdownResult<MarkdownDocument> {
    MarkdownParser::new().parse(content)
}

/// Generate a slug from heading text for anchor links
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let doc = parse("# Hello World").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            BlockElement::Heading { level, content, .. } => {
                assert_eq!(*level, HeadingLevel::H1);
                assert_eq!(content.len(), 1);
            }
            _ => panic!("Expected heading"),
        }
    }

    #[test]
    fn test_parse_paragraph() {
        let doc = parse("This is a paragraph.").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            BlockElement::Paragraph(content) => {
                assert_eq!(content.len(), 1);
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_parse_code_block() {
        let doc = parse("```rust\nfn main() {}\n```").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            BlockElement::CodeBlock { language, .. } => {
                assert_eq!(language.as_deref(), Some("rust"));
            }
            _ => panic!("Expected code block"),
        }
    }

    #[test]
    fn test_parse_list() {
        let doc = parse("- Item 1\n- Item 2\n- Item 3").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            BlockElement::List { list_type, items } => {
                assert_eq!(*list_type, ListType::Unordered);
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_ordered_list() {
        let doc = parse("1. First\n2. Second\n3. Third").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            BlockElement::List { list_type, items } => {
                assert!(matches!(list_type, ListType::Ordered(1)));
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected ordered list"),
        }
    }

    #[test]
    fn test_parse_blockquote() {
        let doc = parse("> This is a quote").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            BlockElement::Blockquote(inner) => {
                assert_eq!(inner.len(), 1);
            }
            _ => panic!("Expected blockquote"),
        }
    }

    #[test]
    fn test_parse_horizontal_rule() {
        let doc = parse("---").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(doc.blocks[0], BlockElement::HorizontalRule));
    }

    #[test]
    fn test_parse_inline_styles() {
        let doc = parse("**bold** *italic* ~~strike~~").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            BlockElement::Paragraph(content) => {
                assert!(content.len() >= 3);
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("Special! Characters?"), "special-characters");
        assert_eq!(slugify("CamelCase"), "camelcase");
    }

    #[test]
    fn test_document_headings() {
        let doc = parse("# H1\n## H2\n### H3").unwrap();
        let headings = doc.headings();
        assert_eq!(headings.len(), 3);
        assert_eq!(*headings[0].0, HeadingLevel::H1);
        assert_eq!(*headings[1].0, HeadingLevel::H2);
        assert_eq!(*headings[2].0, HeadingLevel::H3);
    }

    #[test]
    fn test_document_plain_text() {
        let doc = parse("# Title\n\nSome **bold** text.").unwrap();
        let text = doc.plain_text();
        assert!(text.contains("Title"));
        assert!(text.contains("bold"));
    }

    #[test]
    fn test_table_alignment() {
        assert_eq!(TableAlignment::from(Alignment::Left), TableAlignment::Left);
        assert_eq!(TableAlignment::from(Alignment::Center), TableAlignment::Center);
        assert_eq!(TableAlignment::from(Alignment::Right), TableAlignment::Right);
        assert_eq!(TableAlignment::from(Alignment::None), TableAlignment::None);
    }
}
