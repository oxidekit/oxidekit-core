//! Accessibility System for OxideKit
//!
//! Provides WCAG 2.1 AA compliant accessibility features including:
//! - ARIA role mappings (WAI-ARIA 1.2)
//! - Focus management (focus trap, focus ring, skip links)
//! - Keyboard navigation handlers
//! - Screen reader announcements
//! - Reduced motion preferences
//! - High contrast mode support
//! - Label associations

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// ARIA Roles (WAI-ARIA 1.2)
// ============================================================================

/// ARIA role categories as defined by WAI-ARIA 1.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AriaRoleCategory {
    /// Abstract roles (not to be used directly)
    Abstract,
    /// Widget roles (interactive UI elements)
    Widget,
    /// Document structure roles
    DocumentStructure,
    /// Landmark roles (page regions)
    Landmark,
    /// Live region roles
    LiveRegion,
    /// Window roles
    Window,
}

/// Complete ARIA roles as defined by WAI-ARIA 1.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AriaRole {
    // === Widget Roles ===
    /// A clickable button
    Button,
    /// A checkable input
    Checkbox,
    /// A grid cell
    Gridcell,
    /// A hyperlink
    Link,
    /// An item in a menu
    Menuitem,
    /// A checkable menu item
    Menuitemcheckbox,
    /// A radio-style menu item
    Menuitemradio,
    /// A selectable option
    Option,
    /// Shows task progress
    Progressbar,
    /// A radio button
    Radio,
    /// A scrollbar
    Scrollbar,
    /// A search input
    Searchbox,
    /// A value selector
    Slider,
    /// A numeric spinner
    Spinbutton,
    /// A switch toggle
    Switch,
    /// A tab in a tablist
    Tab,
    /// A panel for a tab
    Tabpanel,
    /// Text input field
    Textbox,
    /// A tree item
    Treeitem,

    // === Composite Widget Roles ===
    /// A combo box
    Combobox,
    /// A data grid
    Grid,
    /// A list box
    Listbox,
    /// A menu
    Menu,
    /// A menu bar
    Menubar,
    /// A group of radio buttons
    Radiogroup,
    /// A tab list
    Tablist,
    /// A tree view
    Tree,
    /// A tree grid
    Treegrid,

    // === Document Structure Roles ===
    /// An application region
    Application,
    /// An article
    Article,
    /// A cell in a table
    Cell,
    /// A column header
    Columnheader,
    /// A definition
    Definition,
    /// A directory
    Directory,
    /// A document
    Document,
    /// A feed of articles
    Feed,
    /// A figure with caption
    Figure,
    /// A generic group
    Group,
    /// A heading
    Heading,
    /// An image
    Img,
    /// A list
    List,
    /// A list item
    Listitem,
    /// Math content
    Math,
    /// No role
    None,
    /// A note
    Note,
    /// Presentation only
    Presentation,
    /// A row in a table/grid
    Row,
    /// A row group
    Rowgroup,
    /// A row header
    Rowheader,
    /// A separator
    Separator,
    /// A table
    Table,
    /// A term
    Term,
    /// A toolbar
    Toolbar,
    /// A tooltip
    Tooltip,

    // === Landmark Roles ===
    /// A banner landmark
    Banner,
    /// Complementary content
    Complementary,
    /// Content info (footer)
    Contentinfo,
    /// A form landmark
    Form,
    /// Main content
    Main,
    /// Navigation
    Navigation,
    /// A region
    Region,
    /// A search landmark
    Search,

    // === Live Region Roles ===
    /// An alert message
    Alert,
    /// A log of messages
    Log,
    /// A marquee
    Marquee,
    /// Status information
    Status,
    /// A timer
    Timer,

    // === Window Roles ===
    /// An alert dialog
    Alertdialog,
    /// A dialog
    Dialog,
}

impl AriaRole {
    /// Get the category for this role
    pub fn category(&self) -> AriaRoleCategory {
        match self {
            // Widget roles
            AriaRole::Button
            | AriaRole::Checkbox
            | AriaRole::Gridcell
            | AriaRole::Link
            | AriaRole::Menuitem
            | AriaRole::Menuitemcheckbox
            | AriaRole::Menuitemradio
            | AriaRole::Option
            | AriaRole::Progressbar
            | AriaRole::Radio
            | AriaRole::Scrollbar
            | AriaRole::Searchbox
            | AriaRole::Slider
            | AriaRole::Spinbutton
            | AriaRole::Switch
            | AriaRole::Tab
            | AriaRole::Tabpanel
            | AriaRole::Textbox
            | AriaRole::Treeitem
            | AriaRole::Combobox
            | AriaRole::Grid
            | AriaRole::Listbox
            | AriaRole::Menu
            | AriaRole::Menubar
            | AriaRole::Radiogroup
            | AriaRole::Tablist
            | AriaRole::Tree
            | AriaRole::Treegrid => AriaRoleCategory::Widget,

            // Landmark roles
            AriaRole::Banner
            | AriaRole::Complementary
            | AriaRole::Contentinfo
            | AriaRole::Form
            | AriaRole::Main
            | AriaRole::Navigation
            | AriaRole::Region
            | AriaRole::Search => AriaRoleCategory::Landmark,

            // Live region roles
            AriaRole::Alert
            | AriaRole::Log
            | AriaRole::Marquee
            | AriaRole::Status
            | AriaRole::Timer => AriaRoleCategory::LiveRegion,

            // Window roles
            AriaRole::Alertdialog | AriaRole::Dialog => AriaRoleCategory::Window,

            // Document structure roles (everything else)
            _ => AriaRoleCategory::DocumentStructure,
        }
    }

    /// Check if this role is focusable by default
    pub fn is_focusable(&self) -> bool {
        matches!(
            self,
            AriaRole::Button
                | AriaRole::Checkbox
                | AriaRole::Link
                | AriaRole::Menuitem
                | AriaRole::Menuitemcheckbox
                | AriaRole::Menuitemradio
                | AriaRole::Option
                | AriaRole::Radio
                | AriaRole::Searchbox
                | AriaRole::Slider
                | AriaRole::Spinbutton
                | AriaRole::Switch
                | AriaRole::Tab
                | AriaRole::Textbox
                | AriaRole::Treeitem
                | AriaRole::Combobox
        )
    }

    /// Get required ARIA attributes for this role
    pub fn required_attributes(&self) -> &'static [&'static str] {
        match self {
            AriaRole::Checkbox | AriaRole::Switch => &["aria-checked"],
            AriaRole::Combobox => &["aria-expanded", "aria-controls"],
            AriaRole::Heading => &["aria-level"],
            AriaRole::Progressbar => &["aria-valuenow"],
            AriaRole::Option => &["aria-selected"],
            AriaRole::Radio => &["aria-checked"],
            AriaRole::Scrollbar => {
                &["aria-controls", "aria-valuenow", "aria-valuemin", "aria-valuemax"]
            }
            AriaRole::Slider => &["aria-valuenow", "aria-valuemin", "aria-valuemax"],
            AriaRole::Spinbutton => &["aria-valuenow", "aria-valuemin", "aria-valuemax"],
            AriaRole::Tab => &["aria-selected"],
            _ => &[],
        }
    }

    /// Get the string representation for this role
    pub fn as_str(&self) -> &'static str {
        match self {
            AriaRole::Button => "button",
            AriaRole::Checkbox => "checkbox",
            AriaRole::Gridcell => "gridcell",
            AriaRole::Link => "link",
            AriaRole::Menuitem => "menuitem",
            AriaRole::Menuitemcheckbox => "menuitemcheckbox",
            AriaRole::Menuitemradio => "menuitemradio",
            AriaRole::Option => "option",
            AriaRole::Progressbar => "progressbar",
            AriaRole::Radio => "radio",
            AriaRole::Scrollbar => "scrollbar",
            AriaRole::Searchbox => "searchbox",
            AriaRole::Slider => "slider",
            AriaRole::Spinbutton => "spinbutton",
            AriaRole::Switch => "switch",
            AriaRole::Tab => "tab",
            AriaRole::Tabpanel => "tabpanel",
            AriaRole::Textbox => "textbox",
            AriaRole::Treeitem => "treeitem",
            AriaRole::Combobox => "combobox",
            AriaRole::Grid => "grid",
            AriaRole::Listbox => "listbox",
            AriaRole::Menu => "menu",
            AriaRole::Menubar => "menubar",
            AriaRole::Radiogroup => "radiogroup",
            AriaRole::Tablist => "tablist",
            AriaRole::Tree => "tree",
            AriaRole::Treegrid => "treegrid",
            AriaRole::Application => "application",
            AriaRole::Article => "article",
            AriaRole::Cell => "cell",
            AriaRole::Columnheader => "columnheader",
            AriaRole::Definition => "definition",
            AriaRole::Directory => "directory",
            AriaRole::Document => "document",
            AriaRole::Feed => "feed",
            AriaRole::Figure => "figure",
            AriaRole::Group => "group",
            AriaRole::Heading => "heading",
            AriaRole::Img => "img",
            AriaRole::List => "list",
            AriaRole::Listitem => "listitem",
            AriaRole::Math => "math",
            AriaRole::None => "none",
            AriaRole::Note => "note",
            AriaRole::Presentation => "presentation",
            AriaRole::Row => "row",
            AriaRole::Rowgroup => "rowgroup",
            AriaRole::Rowheader => "rowheader",
            AriaRole::Separator => "separator",
            AriaRole::Table => "table",
            AriaRole::Term => "term",
            AriaRole::Toolbar => "toolbar",
            AriaRole::Tooltip => "tooltip",
            AriaRole::Banner => "banner",
            AriaRole::Complementary => "complementary",
            AriaRole::Contentinfo => "contentinfo",
            AriaRole::Form => "form",
            AriaRole::Main => "main",
            AriaRole::Navigation => "navigation",
            AriaRole::Region => "region",
            AriaRole::Search => "search",
            AriaRole::Alert => "alert",
            AriaRole::Log => "log",
            AriaRole::Marquee => "marquee",
            AriaRole::Status => "status",
            AriaRole::Timer => "timer",
            AriaRole::Alertdialog => "alertdialog",
            AriaRole::Dialog => "dialog",
        }
    }
}

/// Meter role (separate due to match exhaustiveness)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Meter;

// ============================================================================
// ARIA Attributes
// ============================================================================

/// Common ARIA attributes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AriaAttributes {
    /// The accessible label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// ID of element that labels this one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelled_by: Option<String>,

    /// ID of element that describes this one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub described_by: Option<String>,

    /// Whether the element is expanded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded: Option<bool>,

    /// Whether the element is selected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,

    /// Whether the element is checked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<TriState>,

    /// Whether the element is pressed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressed: Option<TriState>,

    /// Whether the element is disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    /// Whether the element is hidden
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,

    /// Current value (for sliders, progress bars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_now: Option<f64>,

    /// Minimum value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_min: Option<f64>,

    /// Maximum value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_max: Option<f64>,

    /// Text representation of value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_text: Option<String>,

    /// Level (for headings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,

    /// Position in set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos_in_set: Option<u32>,

    /// Size of set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_size: Option<u32>,

    /// ID of controlled element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controls: Option<String>,

    /// ID of element this owns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owns: Option<String>,

    /// Live region politeness
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<LiveRegionPoliteness>,

    /// Atomic update for live regions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atomic: Option<bool>,

    /// Relevant changes for live regions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevant: Option<Vec<LiveRegionRelevant>>,

    /// Whether element is busy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub busy: Option<bool>,

    /// Current item in a set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<CurrentType>,

    /// Error message ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Invalid state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid: Option<InvalidState>,

    /// Has popup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_popup: Option<PopupType>,

    /// Autocomplete type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autocomplete: Option<AutocompleteType>,

    /// Sort direction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<SortDirection>,

    /// Column index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_index: Option<u32>,

    /// Column span
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_span: Option<u32>,

    /// Row index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_index: Option<u32>,

    /// Row span
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_span: Option<u32>,

    /// Modal dialog
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modal: Option<bool>,

    /// Orientation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<AriaOrientation>,

    /// Read-only state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Required field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// Multiline input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiline: Option<bool>,

    /// Multi-selectable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_selectable: Option<bool>,

    /// Placeholder text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Keyboard shortcut
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_shortcuts: Option<String>,

    /// Role description override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_description: Option<String>,
}

/// Tri-state value (true, false, or mixed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriState {
    True,
    False,
    Mixed,
}

/// Live region politeness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveRegionPoliteness {
    Off,
    Polite,
    Assertive,
}

/// Live region relevant changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveRegionRelevant {
    Additions,
    Removals,
    Text,
    All,
}

/// Current item type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CurrentType {
    Page,
    Step,
    Location,
    Date,
    Time,
    True,
    False,
}

/// Invalid state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvalidState {
    False,
    True,
    Grammar,
    Spelling,
}

/// Popup type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PopupType {
    False,
    True,
    Menu,
    Listbox,
    Tree,
    Grid,
    Dialog,
}

/// Autocomplete type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutocompleteType {
    None,
    Inline,
    List,
    Both,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    None,
    Ascending,
    Descending,
    Other,
}

/// Widget orientation for ARIA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AriaOrientation {
    Horizontal,
    Vertical,
}

// ============================================================================
// Accessible Node
// ============================================================================

/// An accessible node in the accessibility tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibleNode {
    /// Unique identifier
    pub id: String,

    /// ARIA role
    pub role: AriaRole,

    /// ARIA attributes
    #[serde(default)]
    pub attributes: AriaAttributes,

    /// Child node IDs
    #[serde(default)]
    pub children: Vec<String>,

    /// Parent node ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// Tab index (-1 = not focusable, 0+ = focusable)
    #[serde(default)]
    pub tab_index: i32,

    /// Bounding rectangle (x, y, width, height)
    #[serde(default)]
    pub bounds: (f32, f32, f32, f32),

    /// Whether node is in the accessibility tree
    #[serde(default = "default_true")]
    pub in_tree: bool,
}

fn default_true() -> bool {
    true
}

impl AccessibleNode {
    /// Create a new accessible node
    pub fn new(id: impl Into<String>, role: AriaRole) -> Self {
        Self {
            id: id.into(),
            role,
            attributes: AriaAttributes::default(),
            children: Vec::new(),
            parent: None,
            tab_index: if role.is_focusable() { 0 } else { -1 },
            bounds: (0.0, 0.0, 0.0, 0.0),
            in_tree: true,
        }
    }

    /// Set the accessible label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.attributes.label = Some(label.into());
        self
    }

    /// Set the tab index
    pub fn with_tab_index(mut self, index: i32) -> Self {
        self.tab_index = index;
        self
    }

    /// Set bounds
    pub fn with_bounds(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.bounds = (x, y, w, h);
        self
    }

    /// Add a child node ID
    pub fn add_child(&mut self, child_id: impl Into<String>) {
        self.children.push(child_id.into());
    }

    /// Check if node is focusable
    pub fn is_focusable(&self) -> bool {
        self.tab_index >= 0 && !self.attributes.disabled.unwrap_or(false)
    }

    /// Get the accessible name
    pub fn accessible_name(&self) -> Option<&str> {
        self.attributes.label.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aria_role_categories() {
        assert_eq!(AriaRole::Button.category(), AriaRoleCategory::Widget);
        assert_eq!(AriaRole::Navigation.category(), AriaRoleCategory::Landmark);
        assert_eq!(AriaRole::Alert.category(), AriaRoleCategory::LiveRegion);
        assert_eq!(AriaRole::Dialog.category(), AriaRoleCategory::Window);
        assert_eq!(AriaRole::Article.category(), AriaRoleCategory::DocumentStructure);
    }

    #[test]
    fn test_aria_role_focusable() {
        assert!(AriaRole::Button.is_focusable());
        assert!(AriaRole::Link.is_focusable());
        assert!(AriaRole::Textbox.is_focusable());
        assert!(!AriaRole::Article.is_focusable());
        assert!(!AriaRole::Navigation.is_focusable());
    }

    #[test]
    fn test_aria_role_required_attributes() {
        assert!(AriaRole::Checkbox.required_attributes().contains(&"aria-checked"));
        assert!(AriaRole::Slider.required_attributes().contains(&"aria-valuenow"));
        assert!(AriaRole::Button.required_attributes().is_empty());
    }

    #[test]
    fn test_accessible_node_creation() {
        let node = AccessibleNode::new("btn-1", AriaRole::Button)
            .with_label("Submit")
            .with_bounds(10.0, 20.0, 100.0, 40.0);

        assert_eq!(node.id, "btn-1");
        assert_eq!(node.role, AriaRole::Button);
        assert_eq!(node.accessible_name(), Some("Submit"));
        assert!(node.is_focusable());
        assert_eq!(node.tab_index, 0);
    }

    #[test]
    fn test_accessible_node_disabled() {
        let mut node = AccessibleNode::new("btn-2", AriaRole::Button);
        node.attributes.disabled = Some(true);
        assert!(!node.is_focusable());
    }

    #[test]
    fn test_tri_state_serialization() {
        let checked = TriState::Mixed;
        let json = serde_json::to_string(&checked).unwrap();
        assert_eq!(json, "\"mixed\"");
    }
}

// ============================================================================
// Focus Management
// ============================================================================

/// Focus ring style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusRingStyle {
    /// Ring color (hex or rgba)
    pub color: String,
    /// Ring width in pixels
    pub width: f32,
    /// Ring offset from element
    pub offset: f32,
    /// Ring border radius (None = inherit from element)
    pub radius: Option<f32>,
    /// Ring style
    pub style: FocusRingType,
}

impl Default for FocusRingStyle {
    fn default() -> Self {
        Self {
            color: "rgba(59, 130, 246, 0.5)".into(), // Blue with 50% opacity
            width: 2.0,
            offset: 2.0,
            radius: None,
            style: FocusRingType::Outline,
        }
    }
}

/// Focus ring rendering type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FocusRingType {
    /// Standard outline
    Outline,
    /// Inset shadow
    Inset,
    /// Outer glow
    Glow,
    /// Solid border
    Solid,
}

/// Focus trap configuration for modal dialogs
#[derive(Debug, Clone)]
pub struct FocusTrap {
    /// ID of the container element
    pub container_id: String,
    /// IDs of focusable elements within the trap
    pub focusable_ids: Vec<String>,
    /// Current focus index
    pub current_index: usize,
    /// Whether the trap is active
    pub active: bool,
    /// ID of element to restore focus to when trap is released
    pub restore_focus_to: Option<String>,
    /// Whether to auto-focus first element when activated
    pub auto_focus_first: bool,
}

impl FocusTrap {
    /// Create a new focus trap
    pub fn new(container_id: impl Into<String>) -> Self {
        Self {
            container_id: container_id.into(),
            focusable_ids: Vec::new(),
            current_index: 0,
            active: false,
            restore_focus_to: None,
            auto_focus_first: true,
        }
    }

    /// Add a focusable element to the trap
    pub fn add_focusable(&mut self, id: impl Into<String>) {
        self.focusable_ids.push(id.into());
    }

    /// Activate the trap and optionally store the current focus for restoration
    pub fn activate(&mut self, current_focus: Option<String>) {
        self.restore_focus_to = current_focus;
        self.active = true;
        if self.auto_focus_first && !self.focusable_ids.is_empty() {
            self.current_index = 0;
        }
    }

    /// Deactivate the trap and return the ID to restore focus to
    pub fn deactivate(&mut self) -> Option<String> {
        self.active = false;
        self.restore_focus_to.take()
    }

    /// Move focus to the next element (wrapping)
    pub fn focus_next(&mut self) -> Option<&str> {
        if self.focusable_ids.is_empty() || !self.active {
            return None;
        }
        self.current_index = (self.current_index + 1) % self.focusable_ids.len();
        Some(&self.focusable_ids[self.current_index])
    }

    /// Move focus to the previous element (wrapping)
    pub fn focus_previous(&mut self) -> Option<&str> {
        if self.focusable_ids.is_empty() || !self.active {
            return None;
        }
        self.current_index = if self.current_index == 0 {
            self.focusable_ids.len() - 1
        } else {
            self.current_index - 1
        };
        Some(&self.focusable_ids[self.current_index])
    }

    /// Get the currently focused element ID
    pub fn current_focus(&self) -> Option<&str> {
        if self.active && !self.focusable_ids.is_empty() {
            Some(&self.focusable_ids[self.current_index])
        } else {
            None
        }
    }

    /// Focus a specific element by ID
    pub fn focus_element(&mut self, id: &str) -> bool {
        if !self.active {
            return false;
        }
        if let Some(idx) = self.focusable_ids.iter().position(|i| i == id) {
            self.current_index = idx;
            true
        } else {
            false
        }
    }
}

/// Skip link for keyboard navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipLink {
    /// Link ID
    pub id: String,
    /// Display text
    pub label: String,
    /// Target element ID to skip to
    pub target_id: String,
    /// Whether link is currently visible
    pub visible: bool,
}

impl SkipLink {
    /// Create a skip to main content link
    pub fn to_main_content() -> Self {
        Self {
            id: "skip-to-main".into(),
            label: "Skip to main content".into(),
            target_id: "main-content".into(),
            visible: false,
        }
    }

    /// Create a skip to navigation link
    pub fn to_navigation() -> Self {
        Self {
            id: "skip-to-nav".into(),
            label: "Skip to navigation".into(),
            target_id: "main-navigation".into(),
            visible: false,
        }
    }

    /// Create a custom skip link
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            target_id: target_id.into(),
            visible: false,
        }
    }

    /// Show the skip link (typically on first Tab press)
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the skip link
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

/// Focus manager for handling application-wide focus
#[derive(Debug, Default)]
pub struct FocusManager {
    /// Currently focused element ID
    current_focus: Option<String>,
    /// Focus history stack
    focus_history: Vec<String>,
    /// Active focus traps
    focus_traps: Vec<FocusTrap>,
    /// Skip links
    skip_links: Vec<SkipLink>,
    /// Focus ring style
    focus_ring_style: FocusRingStyle,
    /// Whether focus ring should be visible
    focus_visible: bool,
    /// Last interaction was keyboard
    keyboard_interaction: bool,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set focus to an element
    pub fn focus(&mut self, id: impl Into<String>) {
        let id = id.into();
        if let Some(prev) = self.current_focus.take() {
            self.focus_history.push(prev);
        }
        self.current_focus = Some(id);
    }

    /// Clear focus
    pub fn blur(&mut self) {
        self.current_focus = None;
    }

    /// Get the currently focused element ID
    pub fn current(&self) -> Option<&str> {
        // Check active focus traps first
        for trap in self.focus_traps.iter().rev() {
            if trap.active {
                return trap.current_focus();
            }
        }
        self.current_focus.as_deref()
    }

    /// Restore focus to the previous element
    pub fn restore(&mut self) -> Option<String> {
        let prev = self.focus_history.pop();
        if prev.is_some() {
            self.current_focus = prev.clone();
        }
        prev
    }

    /// Push a focus trap
    pub fn push_trap(&mut self, mut trap: FocusTrap) {
        trap.activate(self.current_focus.clone());
        self.focus_traps.push(trap);
    }

    /// Pop the active focus trap and restore focus
    pub fn pop_trap(&mut self) -> Option<FocusTrap> {
        if let Some(mut trap) = self.focus_traps.pop() {
            if let Some(restore_id) = trap.deactivate() {
                self.focus(restore_id);
            }
            Some(trap)
        } else {
            None
        }
    }

    /// Add a skip link
    pub fn add_skip_link(&mut self, link: SkipLink) {
        self.skip_links.push(link);
    }

    /// Get skip links
    pub fn skip_links(&self) -> &[SkipLink] {
        &self.skip_links
    }

    /// Show skip links (on first Tab press)
    pub fn show_skip_links(&mut self) {
        for link in &mut self.skip_links {
            link.show();
        }
    }

    /// Hide skip links
    pub fn hide_skip_links(&mut self) {
        for link in &mut self.skip_links {
            link.hide();
        }
    }

    /// Set whether last interaction was keyboard
    pub fn set_keyboard_interaction(&mut self, keyboard: bool) {
        self.keyboard_interaction = keyboard;
        self.focus_visible = keyboard;
    }

    /// Check if focus ring should be visible
    pub fn is_focus_visible(&self) -> bool {
        self.focus_visible
    }

    /// Get focus ring style
    pub fn focus_ring_style(&self) -> &FocusRingStyle {
        &self.focus_ring_style
    }

    /// Set focus ring style
    pub fn set_focus_ring_style(&mut self, style: FocusRingStyle) {
        self.focus_ring_style = style;
    }

    /// Handle Tab key press (move to next focusable)
    pub fn handle_tab(&mut self, shift: bool) -> Option<&str> {
        self.set_keyboard_interaction(true);

        // Check for active focus trap
        for trap in self.focus_traps.iter_mut().rev() {
            if trap.active {
                return if shift {
                    trap.focus_previous()
                } else {
                    trap.focus_next()
                };
            }
        }
        None
    }
}

#[cfg(test)]
mod focus_tests {
    use super::*;

    #[test]
    fn test_focus_trap_navigation() {
        let mut trap = FocusTrap::new("dialog");
        trap.add_focusable("btn-1");
        trap.add_focusable("btn-2");
        trap.add_focusable("btn-3");
        trap.activate(None);

        assert_eq!(trap.current_focus(), Some("btn-1"));
        assert_eq!(trap.focus_next(), Some("btn-2"));
        assert_eq!(trap.focus_next(), Some("btn-3"));
        assert_eq!(trap.focus_next(), Some("btn-1")); // wraps

        assert_eq!(trap.focus_previous(), Some("btn-3")); // wraps back
    }

    #[test]
    fn test_focus_trap_restore() {
        let mut trap = FocusTrap::new("modal");
        trap.add_focusable("ok-btn");
        trap.activate(Some("trigger-btn".into()));

        assert_eq!(trap.current_focus(), Some("ok-btn"));
        let restore = trap.deactivate();
        assert_eq!(restore, Some("trigger-btn".into()));
        assert!(!trap.active);
    }

    #[test]
    fn test_focus_manager() {
        let mut manager = FocusManager::new();
        manager.focus("input-1");
        assert_eq!(manager.current(), Some("input-1"));

        manager.focus("input-2");
        assert_eq!(manager.current(), Some("input-2"));

        manager.restore();
        assert_eq!(manager.current(), Some("input-1"));
    }

    #[test]
    fn test_focus_manager_with_trap() {
        let mut manager = FocusManager::new();
        manager.focus("page-element");

        let mut trap = FocusTrap::new("dialog");
        trap.add_focusable("dialog-btn-1");
        trap.add_focusable("dialog-btn-2");
        manager.push_trap(trap);

        assert_eq!(manager.current(), Some("dialog-btn-1"));
        manager.handle_tab(false);
        assert_eq!(manager.current(), Some("dialog-btn-2"));

        manager.pop_trap();
        assert_eq!(manager.current(), Some("page-element"));
    }

    #[test]
    fn test_skip_links() {
        let mut manager = FocusManager::new();
        manager.add_skip_link(SkipLink::to_main_content());
        manager.add_skip_link(SkipLink::to_navigation());

        assert_eq!(manager.skip_links().len(), 2);
        assert!(!manager.skip_links()[0].visible);

        manager.show_skip_links();
        assert!(manager.skip_links()[0].visible);
    }
}

// ============================================================================
// Keyboard Navigation
// ============================================================================

/// Keyboard key representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Key {
    // Navigation
    Tab,
    Enter,
    Space,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Letters (for shortcuts)
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers
    Digit0, Digit1, Digit2, Digit3, Digit4,
    Digit5, Digit6, Digit7, Digit8, Digit9,

    // Other
    Backspace,
    Delete,
    Insert,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Cmd on Mac, Win on Windows
}

impl KeyModifiers {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn ctrl() -> Self {
        Self { ctrl: true, ..Self::default() }
    }

    pub fn alt() -> Self {
        Self { alt: true, ..Self::default() }
    }

    pub fn shift() -> Self {
        Self { shift: true, ..Self::default() }
    }

    pub fn meta() -> Self {
        Self { meta: true, ..Self::default() }
    }

    pub fn ctrl_shift() -> Self {
        Self { ctrl: true, shift: true, ..Self::default() }
    }
}

/// A keyboard event
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardEvent {
    pub key: Key,
    pub modifiers: KeyModifiers,
    /// Prevents default handling
    pub default_prevented: bool,
    /// Stops event propagation
    pub propagation_stopped: bool,
}

impl KeyboardEvent {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::none(),
            default_prevented: false,
            propagation_stopped: false,
        }
    }

    pub fn with_modifiers(key: Key, modifiers: KeyModifiers) -> Self {
        Self {
            key,
            modifiers,
            default_prevented: false,
            propagation_stopped: false,
        }
    }

    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Check if this matches a key binding
    pub fn matches(&self, key: Key, modifiers: KeyModifiers) -> bool {
        self.key == key && self.modifiers == modifiers
    }
}

/// Keyboard navigation pattern for different widget types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationPattern {
    /// Single focusable element (button, link)
    Single,
    /// Linear navigation (toolbar, menubar)
    Linear,
    /// Grid navigation (data grid, calendar)
    Grid { columns: usize },
    /// Tree navigation (tree view)
    Tree,
    /// Tab navigation (tab list)
    Tabs,
    /// Composite widget (combobox)
    Composite,
}

/// Result of handling a keyboard event
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardResult {
    /// No action taken
    Ignored,
    /// Focus moved to element with given ID
    FocusMoved(String),
    /// Element activated (clicked)
    Activated(String),
    /// Value changed
    ValueChanged { id: String, delta: i32 },
    /// Selection changed
    SelectionChanged { id: String, selected: bool },
    /// Expansion toggled
    ExpansionToggled { id: String, expanded: bool },
    /// Navigation requested (for composite widgets)
    NavigateOut { direction: NavigationDirection },
    /// Custom action
    Custom(String),
}

/// Direction for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
    First,
    Last,
    Next,
    Previous,
}

/// Keyboard navigation handler for widgets
pub struct KeyboardNavigator {
    /// Current pattern
    pattern: NavigationPattern,
    /// Focusable element IDs in order
    elements: Vec<String>,
    /// Current focus index
    current_index: usize,
    /// Whether wrapping is enabled
    wrap: bool,
    /// Whether Home/End keys are enabled
    home_end_enabled: bool,
    /// Custom key bindings
    bindings: HashMap<(Key, KeyModifiers), String>,
}

impl KeyboardNavigator {
    /// Create a new navigator with the given pattern
    pub fn new(pattern: NavigationPattern) -> Self {
        Self {
            pattern,
            elements: Vec::new(),
            current_index: 0,
            wrap: true,
            home_end_enabled: true,
            bindings: HashMap::new(),
        }
    }

    /// Add an element to the navigation order
    pub fn add_element(&mut self, id: impl Into<String>) {
        self.elements.push(id.into());
    }

    /// Set elements from a slice
    pub fn set_elements(&mut self, elements: Vec<String>) {
        self.elements = elements;
        self.current_index = 0;
    }

    /// Add a custom key binding
    pub fn bind(&mut self, key: Key, modifiers: KeyModifiers, action: impl Into<String>) {
        self.bindings.insert((key, modifiers), action.into());
    }

    /// Set wrapping behavior
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    /// Handle a keyboard event
    pub fn handle(&mut self, event: &KeyboardEvent) -> KeyboardResult {
        // Check custom bindings first
        if let Some(action) = self.bindings.get(&(event.key, event.modifiers)) {
            return KeyboardResult::Custom(action.clone());
        }

        match self.pattern {
            NavigationPattern::Single => self.handle_single(event),
            NavigationPattern::Linear => self.handle_linear(event),
            NavigationPattern::Grid { columns } => self.handle_grid(event, columns),
            NavigationPattern::Tree => self.handle_tree(event),
            NavigationPattern::Tabs => self.handle_tabs(event),
            NavigationPattern::Composite => self.handle_composite(event),
        }
    }

    fn handle_single(&mut self, event: &KeyboardEvent) -> KeyboardResult {
        match event.key {
            Key::Enter | Key::Space if event.modifiers == KeyModifiers::none() => {
                if let Some(id) = self.elements.first() {
                    KeyboardResult::Activated(id.clone())
                } else {
                    KeyboardResult::Ignored
                }
            }
            _ => KeyboardResult::Ignored,
        }
    }

    fn handle_linear(&mut self, event: &KeyboardEvent) -> KeyboardResult {
        if event.modifiers != KeyModifiers::none() {
            return KeyboardResult::Ignored;
        }

        match event.key {
            Key::ArrowRight | Key::ArrowDown => self.move_next(),
            Key::ArrowLeft | Key::ArrowUp => self.move_previous(),
            Key::Home if self.home_end_enabled => self.move_first(),
            Key::End if self.home_end_enabled => self.move_last(),
            Key::Enter | Key::Space => self.activate_current(),
            _ => KeyboardResult::Ignored,
        }
    }

    fn handle_grid(&mut self, event: &KeyboardEvent, columns: usize) -> KeyboardResult {
        if event.modifiers != KeyModifiers::none() {
            return KeyboardResult::Ignored;
        }

        match event.key {
            Key::ArrowRight => self.move_next(),
            Key::ArrowLeft => self.move_previous(),
            Key::ArrowDown => self.move_by(columns as i32),
            Key::ArrowUp => self.move_by(-(columns as i32)),
            Key::Home if self.home_end_enabled => {
                // Move to start of row
                let row_start = (self.current_index / columns) * columns;
                self.move_to(row_start)
            }
            Key::End if self.home_end_enabled => {
                // Move to end of row
                let row_end = ((self.current_index / columns) + 1) * columns - 1;
                let row_end = row_end.min(self.elements.len() - 1);
                self.move_to(row_end)
            }
            Key::Enter | Key::Space => self.activate_current(),
            _ => KeyboardResult::Ignored,
        }
    }

    fn handle_tree(&mut self, event: &KeyboardEvent) -> KeyboardResult {
        if event.modifiers != KeyModifiers::none() {
            return KeyboardResult::Ignored;
        }

        match event.key {
            Key::ArrowDown => self.move_next(),
            Key::ArrowUp => self.move_previous(),
            Key::ArrowRight => {
                // Expand or move to first child
                if let Some(id) = self.current_element() {
                    KeyboardResult::ExpansionToggled { id, expanded: true }
                } else {
                    KeyboardResult::Ignored
                }
            }
            Key::ArrowLeft => {
                // Collapse or move to parent
                if let Some(id) = self.current_element() {
                    KeyboardResult::ExpansionToggled { id, expanded: false }
                } else {
                    KeyboardResult::Ignored
                }
            }
            Key::Home if self.home_end_enabled => self.move_first(),
            Key::End if self.home_end_enabled => self.move_last(),
            Key::Enter | Key::Space => self.activate_current(),
            _ => KeyboardResult::Ignored,
        }
    }

    fn handle_tabs(&mut self, event: &KeyboardEvent) -> KeyboardResult {
        if event.modifiers != KeyModifiers::none() {
            return KeyboardResult::Ignored;
        }

        match event.key {
            Key::ArrowRight | Key::ArrowDown => self.move_next(),
            Key::ArrowLeft | Key::ArrowUp => self.move_previous(),
            Key::Home if self.home_end_enabled => self.move_first(),
            Key::End if self.home_end_enabled => self.move_last(),
            Key::Enter | Key::Space => {
                if let Some(id) = self.current_element() {
                    KeyboardResult::SelectionChanged { id, selected: true }
                } else {
                    KeyboardResult::Ignored
                }
            }
            _ => KeyboardResult::Ignored,
        }
    }

    fn handle_composite(&mut self, event: &KeyboardEvent) -> KeyboardResult {
        if event.modifiers != KeyModifiers::none() {
            return KeyboardResult::Ignored;
        }

        match event.key {
            Key::ArrowDown => KeyboardResult::NavigateOut {
                direction: NavigationDirection::Down,
            },
            Key::ArrowUp => KeyboardResult::NavigateOut {
                direction: NavigationDirection::Up,
            },
            Key::Escape => KeyboardResult::NavigateOut {
                direction: NavigationDirection::Previous,
            },
            Key::Enter | Key::Space => self.activate_current(),
            _ => KeyboardResult::Ignored,
        }
    }

    fn move_next(&mut self) -> KeyboardResult {
        self.move_by(1)
    }

    fn move_previous(&mut self) -> KeyboardResult {
        self.move_by(-1)
    }

    fn move_first(&mut self) -> KeyboardResult {
        self.move_to(0)
    }

    fn move_last(&mut self) -> KeyboardResult {
        if self.elements.is_empty() {
            return KeyboardResult::Ignored;
        }
        self.move_to(self.elements.len() - 1)
    }

    fn move_by(&mut self, delta: i32) -> KeyboardResult {
        if self.elements.is_empty() {
            return KeyboardResult::Ignored;
        }

        let len = self.elements.len() as i32;
        let new_index = if self.wrap {
            ((self.current_index as i32 + delta) % len + len) % len
        } else {
            (self.current_index as i32 + delta).clamp(0, len - 1)
        };

        self.move_to(new_index as usize)
    }

    fn move_to(&mut self, index: usize) -> KeyboardResult {
        if index >= self.elements.len() {
            return KeyboardResult::Ignored;
        }

        self.current_index = index;
        KeyboardResult::FocusMoved(self.elements[index].clone())
    }

    fn activate_current(&self) -> KeyboardResult {
        if let Some(id) = self.current_element() {
            KeyboardResult::Activated(id)
        } else {
            KeyboardResult::Ignored
        }
    }

    fn current_element(&self) -> Option<String> {
        self.elements.get(self.current_index).cloned()
    }

    /// Get current focus index
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Set current focus index
    pub fn set_current_index(&mut self, index: usize) {
        if index < self.elements.len() {
            self.current_index = index;
        }
    }
}

#[cfg(test)]
mod keyboard_tests {
    use super::*;

    #[test]
    fn test_linear_navigation() {
        let mut nav = KeyboardNavigator::new(NavigationPattern::Linear);
        nav.set_elements(vec!["a".into(), "b".into(), "c".into()]);

        let event = KeyboardEvent::new(Key::ArrowRight);
        assert_eq!(nav.handle(&event), KeyboardResult::FocusMoved("b".into()));
        assert_eq!(nav.handle(&event), KeyboardResult::FocusMoved("c".into()));
        assert_eq!(nav.handle(&event), KeyboardResult::FocusMoved("a".into())); // wraps
    }

    #[test]
    fn test_grid_navigation() {
        let mut nav = KeyboardNavigator::new(NavigationPattern::Grid { columns: 3 });
        nav.set_elements(vec![
            "a".into(), "b".into(), "c".into(),
            "d".into(), "e".into(), "f".into(),
        ]);

        // Move right
        let right = KeyboardEvent::new(Key::ArrowRight);
        assert_eq!(nav.handle(&right), KeyboardResult::FocusMoved("b".into()));

        // Move down
        let down = KeyboardEvent::new(Key::ArrowDown);
        assert_eq!(nav.handle(&down), KeyboardResult::FocusMoved("e".into()));
    }

    #[test]
    fn test_activation() {
        let mut nav = KeyboardNavigator::new(NavigationPattern::Single);
        nav.add_element("button");

        let enter = KeyboardEvent::new(Key::Enter);
        assert_eq!(nav.handle(&enter), KeyboardResult::Activated("button".into()));

        let space = KeyboardEvent::new(Key::Space);
        assert_eq!(nav.handle(&space), KeyboardResult::Activated("button".into()));
    }

    #[test]
    fn test_custom_binding() {
        let mut nav = KeyboardNavigator::new(NavigationPattern::Linear);
        nav.add_element("item");
        nav.bind(Key::Delete, KeyModifiers::none(), "delete-item");

        let delete = KeyboardEvent::new(Key::Delete);
        assert_eq!(nav.handle(&delete), KeyboardResult::Custom("delete-item".into()));
    }

    #[test]
    fn test_no_wrap() {
        let mut nav = KeyboardNavigator::new(NavigationPattern::Linear);
        nav.set_elements(vec!["a".into(), "b".into()]);
        nav.set_wrap(false);

        let left = KeyboardEvent::new(Key::ArrowLeft);
        assert_eq!(nav.handle(&left), KeyboardResult::FocusMoved("a".into())); // stays at 0
    }
}

// ============================================================================
// Screen Reader Announcements
// ============================================================================

/// Live region announcement for screen readers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    /// The message to announce
    pub message: String,
    /// Politeness level
    pub politeness: LiveRegionPoliteness,
    /// Whether to clear previous announcements
    pub clear_previous: bool,
    /// Delay before announcement (ms)
    pub delay_ms: u32,
    /// Unique ID for deduplication
    pub id: Option<String>,
}

impl Announcement {
    /// Create a polite announcement
    pub fn polite(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            politeness: LiveRegionPoliteness::Polite,
            clear_previous: false,
            delay_ms: 0,
            id: None,
        }
    }

    /// Create an assertive announcement (interrupts current speech)
    pub fn assertive(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            politeness: LiveRegionPoliteness::Assertive,
            clear_previous: true,
            delay_ms: 0,
            id: None,
        }
    }

    /// Set a delay before announcing
    pub fn with_delay(mut self, ms: u32) -> Self {
        self.delay_ms = ms;
        self
    }

    /// Set an ID for deduplication
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Screen reader announcer - manages live region announcements
#[derive(Debug, Default)]
pub struct ScreenReaderAnnouncer {
    /// Pending announcements
    queue: Vec<Announcement>,
    /// IDs of recent announcements (for deduplication)
    recent_ids: HashSet<String>,
    /// Maximum queue size
    max_queue_size: usize,
}

impl ScreenReaderAnnouncer {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            recent_ids: HashSet::new(),
            max_queue_size: 10,
        }
    }

    /// Queue an announcement
    pub fn announce(&mut self, announcement: Announcement) {
        // Deduplicate by ID
        if let Some(ref id) = announcement.id {
            if self.recent_ids.contains(id) {
                return;
            }
            self.recent_ids.insert(id.clone());
        }

        // Clear previous if requested
        if announcement.clear_previous {
            self.queue.clear();
        }

        // Respect max queue size
        if self.queue.len() >= self.max_queue_size {
            self.queue.remove(0);
        }

        self.queue.push(announcement);
    }

    /// Get next announcement to speak
    pub fn next(&mut self) -> Option<Announcement> {
        if self.queue.is_empty() {
            return None;
        }
        Some(self.queue.remove(0))
    }

    /// Check if there are pending announcements
    pub fn has_pending(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Clear recent IDs (for deduplication reset)
    pub fn clear_recent_ids(&mut self) {
        self.recent_ids.clear();
    }

    /// Announce a status change
    pub fn announce_status(&mut self, status: impl Into<String>) {
        self.announce(Announcement::polite(status));
    }

    /// Announce an error
    pub fn announce_error(&mut self, error: impl Into<String>) {
        self.announce(Announcement::assertive(format!("Error: {}", error.into())));
    }

    /// Announce a success
    pub fn announce_success(&mut self, message: impl Into<String>) {
        self.announce(Announcement::polite(format!("Success: {}", message.into())));
    }

    /// Announce focus change
    pub fn announce_focus(&mut self, label: impl Into<String>, role: AriaRole) {
        let message = format!("{}, {}", label.into(), role.as_str());
        self.announce(Announcement::polite(message));
    }
}

// ============================================================================
// User Preferences (Motion, Contrast, etc.)
// ============================================================================

/// User accessibility preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityPreferences {
    /// Reduce motion (prefers-reduced-motion)
    pub reduce_motion: bool,
    /// High contrast mode
    pub high_contrast: HighContrastMode,
    /// Force colors (Windows high contrast)
    pub forced_colors: bool,
    /// Prefer reduced transparency
    pub reduce_transparency: bool,
    /// Invert colors
    pub invert_colors: bool,
    /// Prefer larger text
    pub large_text: bool,
    /// Text scaling factor (1.0 = 100%)
    pub text_scale: f32,
    /// Screen reader active
    pub screen_reader_active: bool,
    /// Keyboard navigation only
    pub keyboard_only: bool,
    /// Show focus indicators always
    pub always_show_focus: bool,
    /// Animation duration multiplier (0.0 = instant, 1.0 = normal)
    pub animation_scale: f32,
}

impl Default for AccessibilityPreferences {
    fn default() -> Self {
        Self {
            reduce_motion: false,
            high_contrast: HighContrastMode::None,
            forced_colors: false,
            reduce_transparency: false,
            invert_colors: false,
            large_text: false,
            text_scale: 1.0,
            screen_reader_active: false,
            keyboard_only: false,
            always_show_focus: false,
            animation_scale: 1.0,
        }
    }
}

impl AccessibilityPreferences {
    /// Check if animations should be disabled or instant
    pub fn should_reduce_motion(&self) -> bool {
        self.reduce_motion || self.animation_scale == 0.0
    }

    /// Get adjusted animation duration
    pub fn adjust_duration(&self, duration_ms: u32) -> u32 {
        if self.reduce_motion {
            0
        } else {
            (duration_ms as f32 * self.animation_scale) as u32
        }
    }

    /// Check if high contrast adjustments are needed
    pub fn needs_high_contrast(&self) -> bool {
        self.high_contrast != HighContrastMode::None || self.forced_colors
    }

    /// Get scaled text size
    pub fn scale_text(&self, size: f32) -> f32 {
        let base = size * self.text_scale;
        if self.large_text {
            base * 1.25
        } else {
            base
        }
    }
}

/// High contrast mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HighContrastMode {
    #[default]
    None,
    /// Dark background, light text
    Dark,
    /// Light background, dark text
    Light,
    /// Custom colors (Windows high contrast)
    Custom,
}

/// High contrast color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighContrastColors {
    /// Background color
    pub background: String,
    /// Foreground/text color
    pub foreground: String,
    /// Link color
    pub link: String,
    /// Visited link color
    pub link_visited: String,
    /// Button background
    pub button_background: String,
    /// Button text
    pub button_text: String,
    /// Disabled text
    pub disabled: String,
    /// Highlight/selection background
    pub highlight: String,
    /// Highlight/selection text
    pub highlight_text: String,
}

impl HighContrastColors {
    /// Standard dark high contrast scheme
    pub fn dark() -> Self {
        Self {
            background: "#000000".into(),
            foreground: "#FFFFFF".into(),
            link: "#FFFF00".into(),
            link_visited: "#FF00FF".into(),
            button_background: "#000000".into(),
            button_text: "#FFFFFF".into(),
            disabled: "#808080".into(),
            highlight: "#FFFF00".into(),
            highlight_text: "#000000".into(),
        }
    }

    /// Standard light high contrast scheme
    pub fn light() -> Self {
        Self {
            background: "#FFFFFF".into(),
            foreground: "#000000".into(),
            link: "#0000FF".into(),
            link_visited: "#800080".into(),
            button_background: "#FFFFFF".into(),
            button_text: "#000000".into(),
            disabled: "#808080".into(),
            highlight: "#000080".into(),
            highlight_text: "#FFFFFF".into(),
        }
    }
}

// ============================================================================
// Label Associations
// ============================================================================

/// Label association for form controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelAssociation {
    /// ID of the control being labeled
    pub control_id: String,
    /// ID of the labeling element
    pub label_id: String,
    /// The label text
    pub label_text: String,
    /// Association type
    pub association_type: LabelAssociationType,
}

/// How the label is associated with the control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelAssociationType {
    /// Using aria-labelledby
    AriaLabelledBy,
    /// Using aria-label attribute
    AriaLabel,
    /// Using HTML label element with for attribute
    HtmlFor,
    /// Label wraps the control
    Wrapper,
    /// Using aria-describedby
    AriaDescribedBy,
    /// Implicit from content
    Implicit,
}

/// Manages label associations for accessibility
#[derive(Debug, Default)]
pub struct LabelManager {
    /// Map of control ID to its label associations
    associations: HashMap<String, Vec<LabelAssociation>>,
    /// Map of label ID to the control it labels
    reverse_map: HashMap<String, String>,
}

impl LabelManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Associate a label with a control
    pub fn associate(
        &mut self,
        control_id: impl Into<String>,
        label_id: impl Into<String>,
        label_text: impl Into<String>,
        association_type: LabelAssociationType,
    ) {
        let control_id = control_id.into();
        let label_id = label_id.into();

        let association = LabelAssociation {
            control_id: control_id.clone(),
            label_id: label_id.clone(),
            label_text: label_text.into(),
            association_type,
        };

        self.associations
            .entry(control_id.clone())
            .or_default()
            .push(association);
        self.reverse_map.insert(label_id, control_id);
    }

    /// Get labels for a control
    pub fn get_labels(&self, control_id: &str) -> Option<&[LabelAssociation]> {
        self.associations.get(control_id).map(|v| v.as_slice())
    }

    /// Get the primary accessible name for a control
    pub fn get_accessible_name(&self, control_id: &str) -> Option<String> {
        let labels = self.associations.get(control_id)?;

        // Priority: aria-labelledby > aria-label > html for > wrapper > implicit
        let priority = [
            LabelAssociationType::AriaLabelledBy,
            LabelAssociationType::AriaLabel,
            LabelAssociationType::HtmlFor,
            LabelAssociationType::Wrapper,
            LabelAssociationType::Implicit,
        ];

        for assoc_type in priority {
            if let Some(label) = labels.iter().find(|l| l.association_type == assoc_type) {
                return Some(label.label_text.clone());
            }
        }

        None
    }

    /// Get the accessible description for a control
    pub fn get_accessible_description(&self, control_id: &str) -> Option<String> {
        self.associations.get(control_id).and_then(|labels| {
            labels
                .iter()
                .find(|l| l.association_type == LabelAssociationType::AriaDescribedBy)
                .map(|l| l.label_text.clone())
        })
    }

    /// Get the control ID that a label element labels
    pub fn get_labeled_control(&self, label_id: &str) -> Option<&str> {
        self.reverse_map.get(label_id).map(|s| s.as_str())
    }

    /// Remove all associations for a control
    pub fn remove_control(&mut self, control_id: &str) {
        if let Some(associations) = self.associations.remove(control_id) {
            for assoc in associations {
                self.reverse_map.remove(&assoc.label_id);
            }
        }
    }

    /// Check if a control has a label (WCAG 2.1 requirement)
    pub fn has_label(&self, control_id: &str) -> bool {
        self.associations.get(control_id).map_or(false, |v| !v.is_empty())
    }
}

// ============================================================================
// Accessibility Tree
// ============================================================================

/// The accessibility tree - represents the accessible structure of the UI
#[derive(Debug, Default)]
pub struct AccessibilityTree {
    /// All nodes indexed by ID
    nodes: HashMap<String, AccessibleNode>,
    /// Root node ID
    root_id: Option<String>,
    /// Landmark nodes for quick navigation
    landmarks: Vec<String>,
    /// Heading nodes for quick navigation
    headings: Vec<String>,
    /// Focus order (tab sequence)
    focus_order: Vec<String>,
}

impl AccessibilityTree {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the root node
    pub fn set_root(&mut self, node: AccessibleNode) {
        self.root_id = Some(node.id.clone());
        self.add_node(node);
    }

    /// Add a node to the tree
    pub fn add_node(&mut self, node: AccessibleNode) {
        // Track landmarks
        if node.role.category() == AriaRoleCategory::Landmark {
            self.landmarks.push(node.id.clone());
        }

        // Track headings
        if node.role == AriaRole::Heading {
            self.headings.push(node.id.clone());
        }

        // Track focusable elements
        if node.is_focusable() {
            self.focus_order.push(node.id.clone());
        }

        self.nodes.insert(node.id.clone(), node);
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&AccessibleNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut AccessibleNode> {
        self.nodes.get_mut(id)
    }

    /// Remove a node by ID
    pub fn remove_node(&mut self, id: &str) -> Option<AccessibleNode> {
        self.landmarks.retain(|l| l != id);
        self.headings.retain(|h| h != id);
        self.focus_order.retain(|f| f != id);
        self.nodes.remove(id)
    }

    /// Get all landmark nodes
    pub fn get_landmarks(&self) -> Vec<&AccessibleNode> {
        self.landmarks
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Get all heading nodes
    pub fn get_headings(&self) -> Vec<&AccessibleNode> {
        self.headings
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Get headings at a specific level
    pub fn get_headings_at_level(&self, level: u8) -> Vec<&AccessibleNode> {
        self.get_headings()
            .into_iter()
            .filter(|node| node.attributes.level == Some(level))
            .collect()
    }

    /// Get the focus order
    pub fn focus_order(&self) -> &[String] {
        &self.focus_order
    }

    /// Update the focus order based on tab indices and DOM order
    pub fn rebuild_focus_order(&mut self) {
        self.focus_order.clear();

        let mut focusable: Vec<_> = self
            .nodes
            .values()
            .filter(|n| n.is_focusable())
            .collect();

        // Sort by tab index, then by order added (preserved in nodes iteration)
        // Sort by tab index, then by y position, then by x position
        focusable.sort_by(|a, b| {
            let a_key = (a.tab_index.max(0), a.bounds.1 as i32, a.bounds.0 as i32);
            let b_key = (b.tab_index.max(0), b.bounds.1 as i32, b.bounds.0 as i32);
            a_key.cmp(&b_key)
        });

        self.focus_order = focusable.into_iter().map(|n| n.id.clone()).collect();
    }

    /// Find the next focusable element after the given ID
    pub fn next_focusable(&self, current_id: &str) -> Option<&str> {
        let current_idx = self.focus_order.iter().position(|id| id == current_id)?;
        let next_idx = (current_idx + 1) % self.focus_order.len();
        self.focus_order.get(next_idx).map(|s| s.as_str())
    }

    /// Find the previous focusable element before the given ID
    pub fn prev_focusable(&self, current_id: &str) -> Option<&str> {
        let current_idx = self.focus_order.iter().position(|id| id == current_id)?;
        let prev_idx = if current_idx == 0 {
            self.focus_order.len().saturating_sub(1)
        } else {
            current_idx - 1
        };
        self.focus_order.get(prev_idx).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod announcer_tests {
    use super::*;

    #[test]
    fn test_announcement_queue() {
        let mut announcer = ScreenReaderAnnouncer::new();
        announcer.announce(Announcement::polite("First"));
        announcer.announce(Announcement::polite("Second"));

        assert!(announcer.has_pending());
        assert_eq!(announcer.next().unwrap().message, "First");
        assert_eq!(announcer.next().unwrap().message, "Second");
        assert!(!announcer.has_pending());
    }

    #[test]
    fn test_assertive_clears_queue() {
        let mut announcer = ScreenReaderAnnouncer::new();
        announcer.announce(Announcement::polite("Ignored"));
        announcer.announce(Announcement::assertive("Important!"));

        assert_eq!(announcer.next().unwrap().message, "Important!");
        assert!(!announcer.has_pending());
    }

    #[test]
    fn test_deduplication() {
        let mut announcer = ScreenReaderAnnouncer::new();
        announcer.announce(Announcement::polite("Loading...").with_id("loading"));
        announcer.announce(Announcement::polite("Loading...").with_id("loading"));

        assert_eq!(announcer.next().unwrap().message, "Loading...");
        assert!(!announcer.has_pending());
    }
}

#[cfg(test)]
mod preferences_tests {
    use super::*;

    #[test]
    fn test_reduce_motion() {
        let mut prefs = AccessibilityPreferences::default();
        assert!(!prefs.should_reduce_motion());
        assert_eq!(prefs.adjust_duration(300), 300);

        prefs.reduce_motion = true;
        assert!(prefs.should_reduce_motion());
        assert_eq!(prefs.adjust_duration(300), 0);
    }

    #[test]
    fn test_animation_scale() {
        let mut prefs = AccessibilityPreferences::default();
        prefs.animation_scale = 0.5;
        assert_eq!(prefs.adjust_duration(300), 150);

        prefs.animation_scale = 2.0;
        assert_eq!(prefs.adjust_duration(300), 600);
    }

    #[test]
    fn test_text_scaling() {
        let mut prefs = AccessibilityPreferences::default();
        assert_eq!(prefs.scale_text(16.0), 16.0);

        prefs.text_scale = 1.5;
        assert_eq!(prefs.scale_text(16.0), 24.0);

        prefs.large_text = true;
        assert_eq!(prefs.scale_text(16.0), 30.0); // 16 * 1.5 * 1.25
    }

    #[test]
    fn test_high_contrast() {
        let mut prefs = AccessibilityPreferences::default();
        assert!(!prefs.needs_high_contrast());

        prefs.high_contrast = HighContrastMode::Dark;
        assert!(prefs.needs_high_contrast());
    }
}

#[cfg(test)]
mod label_tests {
    use super::*;

    #[test]
    fn test_label_association() {
        let mut manager = LabelManager::new();
        manager.associate(
            "email-input",
            "email-label",
            "Email Address",
            LabelAssociationType::HtmlFor,
        );

        assert!(manager.has_label("email-input"));
        assert_eq!(
            manager.get_accessible_name("email-input"),
            Some("Email Address".into())
        );
        assert_eq!(
            manager.get_labeled_control("email-label"),
            Some("email-input")
        );
    }

    #[test]
    fn test_label_priority() {
        let mut manager = LabelManager::new();

        // Add multiple labels with different types
        manager.associate(
            "input",
            "label1",
            "Implicit Label",
            LabelAssociationType::Implicit,
        );
        manager.associate(
            "input",
            "label2",
            "ARIA Label",
            LabelAssociationType::AriaLabelledBy,
        );

        // aria-labelledby takes priority
        assert_eq!(
            manager.get_accessible_name("input"),
            Some("ARIA Label".into())
        );
    }

    #[test]
    fn test_accessible_description() {
        let mut manager = LabelManager::new();
        manager.associate(
            "password",
            "pw-label",
            "Password",
            LabelAssociationType::HtmlFor,
        );
        manager.associate(
            "password",
            "pw-desc",
            "Must be at least 8 characters",
            LabelAssociationType::AriaDescribedBy,
        );

        assert_eq!(
            manager.get_accessible_name("password"),
            Some("Password".into())
        );
        assert_eq!(
            manager.get_accessible_description("password"),
            Some("Must be at least 8 characters".into())
        );
    }
}

#[cfg(test)]
mod tree_tests {
    use super::*;

    #[test]
    fn test_accessibility_tree() {
        let mut tree = AccessibilityTree::new();

        let main = AccessibleNode::new("main", AriaRole::Main).with_label("Main Content");
        let nav = AccessibleNode::new("nav", AriaRole::Navigation).with_label("Primary Navigation");
        let h1 = AccessibleNode::new("h1", AriaRole::Heading).with_label("Welcome");
        let btn = AccessibleNode::new("btn", AriaRole::Button).with_label("Submit");

        tree.add_node(main);
        tree.add_node(nav);
        tree.add_node(h1);
        tree.add_node(btn);

        // Check landmarks
        let landmarks = tree.get_landmarks();
        assert_eq!(landmarks.len(), 2);

        // Check headings
        let headings = tree.get_headings();
        assert_eq!(headings.len(), 1);

        // Check focus order
        assert_eq!(tree.focus_order().len(), 1); // Only button is focusable
    }

    #[test]
    fn test_focus_navigation() {
        let mut tree = AccessibilityTree::new();

        tree.add_node(AccessibleNode::new("btn1", AriaRole::Button));
        tree.add_node(AccessibleNode::new("btn2", AriaRole::Button));
        tree.add_node(AccessibleNode::new("btn3", AriaRole::Button));

        assert_eq!(tree.next_focusable("btn1"), Some("btn2"));
        assert_eq!(tree.next_focusable("btn3"), Some("btn1")); // wraps
        assert_eq!(tree.prev_focusable("btn1"), Some("btn3")); // wraps
    }
}
