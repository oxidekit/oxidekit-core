//! Markdown editor toolbar.

use serde::{Deserialize, Serialize};

/// Toolbar item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolbarItem {
    /// Bold text
    Bold,
    /// Italic text
    Italic,
    /// Strikethrough
    Strikethrough,
    /// Heading
    Heading,
    /// Link
    Link,
    /// Image
    Image,
    /// Code
    Code,
    /// Code block
    CodeBlock,
    /// Quote
    Quote,
    /// Unordered list
    BulletList,
    /// Ordered list
    NumberedList,
    /// Task list
    TaskList,
    /// Horizontal rule
    HorizontalRule,
    /// Table
    Table,
    /// Separator
    Separator,
    /// Undo
    Undo,
    /// Redo
    Redo,
    /// Preview toggle
    Preview,
}

impl ToolbarItem {
    /// Get icon name
    pub fn icon(&self) -> &'static str {
        match self {
            ToolbarItem::Bold => "format_bold",
            ToolbarItem::Italic => "format_italic",
            ToolbarItem::Strikethrough => "strikethrough_s",
            ToolbarItem::Heading => "title",
            ToolbarItem::Link => "link",
            ToolbarItem::Image => "image",
            ToolbarItem::Code => "code",
            ToolbarItem::CodeBlock => "code_blocks",
            ToolbarItem::Quote => "format_quote",
            ToolbarItem::BulletList => "format_list_bulleted",
            ToolbarItem::NumberedList => "format_list_numbered",
            ToolbarItem::TaskList => "checklist",
            ToolbarItem::HorizontalRule => "horizontal_rule",
            ToolbarItem::Table => "table_chart",
            ToolbarItem::Separator => "divider",
            ToolbarItem::Undo => "undo",
            ToolbarItem::Redo => "redo",
            ToolbarItem::Preview => "preview",
        }
    }

    /// Get tooltip
    pub fn tooltip(&self) -> &'static str {
        match self {
            ToolbarItem::Bold => "Bold (Ctrl+B)",
            ToolbarItem::Italic => "Italic (Ctrl+I)",
            ToolbarItem::Strikethrough => "Strikethrough",
            ToolbarItem::Heading => "Heading",
            ToolbarItem::Link => "Link (Ctrl+K)",
            ToolbarItem::Image => "Image",
            ToolbarItem::Code => "Inline Code",
            ToolbarItem::CodeBlock => "Code Block",
            ToolbarItem::Quote => "Quote",
            ToolbarItem::BulletList => "Bullet List",
            ToolbarItem::NumberedList => "Numbered List",
            ToolbarItem::TaskList => "Task List",
            ToolbarItem::HorizontalRule => "Horizontal Rule",
            ToolbarItem::Table => "Table",
            ToolbarItem::Separator => "",
            ToolbarItem::Undo => "Undo (Ctrl+Z)",
            ToolbarItem::Redo => "Redo (Ctrl+Y)",
            ToolbarItem::Preview => "Toggle Preview",
        }
    }
}

/// Toolbar action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolbarAction {
    /// Insert text at cursor
    Insert(String),
    /// Wrap selection
    Wrap { prefix: String, suffix: String },
    /// Toggle mode
    TogglePreview,
    /// Undo
    Undo,
    /// Redo
    Redo,
}

/// Toolbar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarConfig {
    /// Items to show
    pub items: Vec<ToolbarItem>,
    /// Show at top
    pub position_top: bool,
    /// Sticky toolbar
    pub sticky: bool,
}

impl Default for ToolbarConfig {
    fn default() -> Self {
        Self {
            items: vec![
                ToolbarItem::Bold,
                ToolbarItem::Italic,
                ToolbarItem::Strikethrough,
                ToolbarItem::Separator,
                ToolbarItem::Heading,
                ToolbarItem::Link,
                ToolbarItem::Image,
                ToolbarItem::Separator,
                ToolbarItem::Code,
                ToolbarItem::CodeBlock,
                ToolbarItem::Quote,
                ToolbarItem::Separator,
                ToolbarItem::BulletList,
                ToolbarItem::NumberedList,
                ToolbarItem::TaskList,
                ToolbarItem::Separator,
                ToolbarItem::Preview,
            ],
            position_top: true,
            sticky: true,
        }
    }
}

/// Toolbar component
#[derive(Debug, Clone)]
pub struct Toolbar {
    /// Configuration
    pub config: ToolbarConfig,
}

impl Toolbar {
    /// Create new toolbar
    pub fn new() -> Self {
        Self {
            config: ToolbarConfig::default(),
        }
    }

    /// Set items
    pub fn items(mut self, items: Vec<ToolbarItem>) -> Self {
        self.config.items = items;
        self
    }

    /// Set sticky
    pub fn sticky(mut self, sticky: bool) -> Self {
        self.config.sticky = sticky;
        self
    }

    /// Get action for item
    pub fn action_for(&self, item: ToolbarItem) -> ToolbarAction {
        match item {
            ToolbarItem::Bold => ToolbarAction::Wrap {
                prefix: "**".to_string(),
                suffix: "**".to_string(),
            },
            ToolbarItem::Italic => ToolbarAction::Wrap {
                prefix: "_".to_string(),
                suffix: "_".to_string(),
            },
            ToolbarItem::Strikethrough => ToolbarAction::Wrap {
                prefix: "~~".to_string(),
                suffix: "~~".to_string(),
            },
            ToolbarItem::Code => ToolbarAction::Wrap {
                prefix: "`".to_string(),
                suffix: "`".to_string(),
            },
            ToolbarItem::Link => ToolbarAction::Wrap {
                prefix: "[".to_string(),
                suffix: "](url)".to_string(),
            },
            ToolbarItem::Preview => ToolbarAction::TogglePreview,
            ToolbarItem::Undo => ToolbarAction::Undo,
            ToolbarItem::Redo => ToolbarAction::Redo,
            _ => ToolbarAction::Insert(String::new()),
        }
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}
