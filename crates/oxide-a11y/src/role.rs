//! Accessibility Roles (WAI-ARIA 1.2 compliant)
//!
//! This module defines all standard ARIA roles organized by category:
//! - Widget roles: Interactive UI elements
//! - Document structure roles: Non-interactive structural elements
//! - Landmark roles: Page regions for navigation
//! - Live region roles: Dynamic content areas
//! - Window roles: Dialog and window types

use serde::{Deserialize, Serialize};

/// Role category as defined by WAI-ARIA 1.2 specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleCategory {
    /// Abstract roles - base types not for direct use
    Abstract,
    /// Widget roles - interactive UI elements
    Widget,
    /// Composite widget roles - containers of interactive elements
    CompositeWidget,
    /// Document structure roles - organizational elements
    DocumentStructure,
    /// Landmark roles - navigational regions
    Landmark,
    /// Live region roles - dynamic content areas
    LiveRegion,
    /// Window roles - dialog and overlay elements
    Window,
}

/// Complete WAI-ARIA 1.2 role enumeration
///
/// Roles are the foundation of accessibility, describing the type
/// and purpose of each element to assistive technologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Role {
    // ========================================================================
    // Widget Roles - Interactive UI Elements
    // ========================================================================
    /// A clickable button that triggers an action
    Button,

    /// A checkable input with true/false/mixed states
    Checkbox,

    /// A cell containing data within a grid
    Gridcell,

    /// An interactive reference to a resource
    Link,

    /// An option in a menu
    Menuitem,

    /// A checkable menuitem with a checkbox
    Menuitemcheckbox,

    /// A checkable menuitem in a group where only one can be checked
    Menuitemradio,

    /// A selectable item in a listbox
    Option,

    /// Displays the progress of a task
    Progressbar,

    /// A checkable input in a group where only one can be checked
    Radio,

    /// A graphical object controlling the scrolling of content
    Scrollbar,

    /// A text input for entering search queries
    Searchbox,

    /// An input for selecting a value from a range
    Slider,

    /// A form of range with discrete values
    Spinbutton,

    /// A type of checkbox representing on/off values
    Switch,

    /// A grouping label for a tabpanel
    Tab,

    /// A container for the content of a tab
    Tabpanel,

    /// A single-line text input
    Textbox,

    /// An option item in a tree
    Treeitem,

    // ========================================================================
    // Composite Widget Roles - Containers of Interactive Elements
    // ========================================================================
    /// A composite widget with a text input and popup listbox
    Combobox,

    /// An interactive 2D tabular data structure
    Grid,

    /// A widget for selecting from a list of options
    Listbox,

    /// A widget offering a list of choices
    Menu,

    /// A presentation of menu as a horizontal bar
    Menubar,

    /// A group of radio buttons
    Radiogroup,

    /// A list of tab elements
    Tablist,

    /// A widget for hierarchical data
    Tree,

    /// A grid with expandable rows
    Treegrid,

    // ========================================================================
    // Document Structure Roles
    // ========================================================================
    /// A region declared as a web application
    Application,

    /// A section of content that forms an independent part of a document
    Article,

    /// A cell in a tabular container
    Cell,

    /// A cell containing header information for a column
    Columnheader,

    /// A definition of a term or concept
    Definition,

    /// Deprecated: A list of references to members of a group
    Directory,

    /// An element containing content that assistive technology users may want to browse
    Document,

    /// A scrollable list of articles
    Feed,

    /// A perceivable section of content representing a self-contained unit
    Figure,

    /// A set of user interface objects not in a list or tree
    #[default]
    Group,

    /// A heading for a section of the page
    Heading,

    /// A container for a collection of elements representing an image
    Img,

    /// An image represented by alternative text
    Image,

    /// A section containing listitem elements
    List,

    /// A single item in a list or directory
    Listitem,

    /// Content representing a mathematical expression
    Math,

    /// An element with no role semantics
    None,

    /// A section containing supplementary information
    Note,

    /// An element with implicit native role semantics removed
    Presentation,

    /// A row of cells in a tabular container
    Row,

    /// A group containing one or more row elements
    Rowgroup,

    /// A cell containing header information for a row
    Rowheader,

    /// A divider separating sections or groups
    Separator,

    /// A section containing data arranged in rows and columns
    Table,

    /// A word or phrase with a corresponding definition
    Term,

    /// A collection of commonly used function buttons
    Toolbar,

    /// A contextual popup for displaying a description
    Tooltip,

    // ========================================================================
    // Landmark Roles - Page Regions
    // ========================================================================
    /// Site-oriented content at the beginning of a page
    Banner,

    /// Supporting section related to the main content
    Complementary,

    /// Footer content with information about the page
    Contentinfo,

    /// A region containing a form
    Form,

    /// The main content of the document
    Main,

    /// A collection of navigational elements
    Navigation,

    /// A perceivable section with specific purpose
    Region,

    /// A landmark region for searching content
    Search,

    // ========================================================================
    // Live Region Roles
    // ========================================================================
    /// A message with important, time-sensitive information
    Alert,

    /// A type of live region with a log of messages
    Log,

    /// A type of live region containing non-essential scrolling text
    Marquee,

    /// A type of live region for advisory information
    Status,

    /// A type of live region displaying an elapsed time
    Timer,

    // ========================================================================
    // Window Roles
    // ========================================================================
    /// An alert dialog requiring user response
    Alertdialog,

    /// A dialog box or window
    Dialog,
}

impl Role {
    /// Get the category for this role
    pub const fn category(&self) -> RoleCategory {
        match self {
            // Widget roles
            Role::Button
            | Role::Checkbox
            | Role::Gridcell
            | Role::Link
            | Role::Menuitem
            | Role::Menuitemcheckbox
            | Role::Menuitemradio
            | Role::Option
            | Role::Progressbar
            | Role::Radio
            | Role::Scrollbar
            | Role::Searchbox
            | Role::Slider
            | Role::Spinbutton
            | Role::Switch
            | Role::Tab
            | Role::Tabpanel
            | Role::Textbox
            | Role::Treeitem => RoleCategory::Widget,

            // Composite widget roles
            Role::Combobox
            | Role::Grid
            | Role::Listbox
            | Role::Menu
            | Role::Menubar
            | Role::Radiogroup
            | Role::Tablist
            | Role::Tree
            | Role::Treegrid => RoleCategory::CompositeWidget,

            // Landmark roles
            Role::Banner
            | Role::Complementary
            | Role::Contentinfo
            | Role::Form
            | Role::Main
            | Role::Navigation
            | Role::Region
            | Role::Search => RoleCategory::Landmark,

            // Live region roles
            Role::Alert | Role::Log | Role::Marquee | Role::Status | Role::Timer => {
                RoleCategory::LiveRegion
            }

            // Window roles
            Role::Alertdialog | Role::Dialog => RoleCategory::Window,

            // Document structure (everything else)
            _ => RoleCategory::DocumentStructure,
        }
    }

    /// Check if this role is interactive/focusable by default
    pub const fn is_interactive(&self) -> bool {
        matches!(
            self,
            Role::Button
                | Role::Checkbox
                | Role::Link
                | Role::Menuitem
                | Role::Menuitemcheckbox
                | Role::Menuitemradio
                | Role::Option
                | Role::Radio
                | Role::Scrollbar
                | Role::Searchbox
                | Role::Slider
                | Role::Spinbutton
                | Role::Switch
                | Role::Tab
                | Role::Textbox
                | Role::Treeitem
                | Role::Combobox
                | Role::Gridcell
        )
    }

    /// Check if this role is a landmark
    pub const fn is_landmark(&self) -> bool {
        matches!(self.category(), RoleCategory::Landmark)
    }

    /// Check if this role is a live region
    pub const fn is_live_region(&self) -> bool {
        matches!(self.category(), RoleCategory::LiveRegion)
    }

    /// Check if this role can be named (has accessible name computation)
    pub const fn is_nameable(&self) -> bool {
        !matches!(self, Role::None | Role::Presentation)
    }

    /// Get required ARIA attributes for this role per WAI-ARIA spec
    pub const fn required_states(&self) -> &'static [&'static str] {
        match self {
            Role::Checkbox | Role::Switch => &["aria-checked"],
            Role::Combobox => &["aria-controls", "aria-expanded"],
            Role::Heading => &["aria-level"],
            Role::Menuitemcheckbox | Role::Menuitemradio => &["aria-checked"],
            Role::Option => &["aria-selected"],
            Role::Radio => &["aria-checked"],
            Role::Scrollbar => {
                &[
                    "aria-controls",
                    "aria-valuenow",
                    "aria-valuemax",
                    "aria-valuemin",
                ]
            }
            Role::Separator => &["aria-valuenow"], // When focusable
            Role::Slider | Role::Spinbutton => &["aria-valuenow", "aria-valuemax", "aria-valuemin"],
            Role::Tab => &["aria-selected"],
            _ => &[],
        }
    }

    /// Get supported ARIA states and properties for this role
    pub const fn supported_states(&self) -> &'static [&'static str] {
        match self {
            Role::Button => &[
                "aria-disabled",
                "aria-expanded",
                "aria-haspopup",
                "aria-pressed",
            ],
            Role::Checkbox => &["aria-checked", "aria-disabled", "aria-readonly", "aria-required"],
            Role::Combobox => &[
                "aria-autocomplete",
                "aria-controls",
                "aria-disabled",
                "aria-expanded",
                "aria-haspopup",
                "aria-readonly",
                "aria-required",
            ],
            Role::Dialog | Role::Alertdialog => &["aria-modal"],
            Role::Grid => &[
                "aria-disabled",
                "aria-multiselectable",
                "aria-readonly",
                "aria-colcount",
                "aria-rowcount",
            ],
            Role::Gridcell => &[
                "aria-disabled",
                "aria-readonly",
                "aria-selected",
                "aria-colindex",
                "aria-colspan",
                "aria-rowindex",
                "aria-rowspan",
            ],
            Role::Heading => &["aria-level"],
            Role::Link => &["aria-disabled", "aria-expanded"],
            Role::Listbox => &["aria-disabled", "aria-multiselectable", "aria-readonly", "aria-required"],
            Role::Menu | Role::Menubar => &["aria-disabled"],
            Role::Menuitem => &["aria-disabled", "aria-haspopup"],
            Role::Menuitemcheckbox | Role::Menuitemradio => &["aria-checked", "aria-disabled"],
            Role::Option => &["aria-checked", "aria-disabled", "aria-selected"],
            Role::Progressbar => &["aria-valuenow", "aria-valuemax", "aria-valuemin", "aria-valuetext"],
            Role::Radio => &["aria-checked", "aria-disabled"],
            Role::Row => &[
                "aria-disabled",
                "aria-expanded",
                "aria-level",
                "aria-selected",
                "aria-colindex",
                "aria-rowindex",
            ],
            Role::Scrollbar => &[
                "aria-controls",
                "aria-disabled",
                "aria-orientation",
                "aria-valuemax",
                "aria-valuemin",
                "aria-valuenow",
            ],
            Role::Searchbox | Role::Textbox => &[
                "aria-autocomplete",
                "aria-disabled",
                "aria-multiline",
                "aria-placeholder",
                "aria-readonly",
                "aria-required",
            ],
            Role::Slider => &[
                "aria-disabled",
                "aria-orientation",
                "aria-readonly",
                "aria-valuemax",
                "aria-valuemin",
                "aria-valuenow",
                "aria-valuetext",
            ],
            Role::Spinbutton => &[
                "aria-disabled",
                "aria-readonly",
                "aria-required",
                "aria-valuemax",
                "aria-valuemin",
                "aria-valuenow",
                "aria-valuetext",
            ],
            Role::Switch => &["aria-checked", "aria-disabled", "aria-readonly"],
            Role::Tab => &["aria-disabled", "aria-selected", "aria-controls"],
            Role::Tablist => &["aria-disabled", "aria-multiselectable", "aria-orientation"],
            Role::Tabpanel => &[],
            Role::Tree => &["aria-disabled", "aria-multiselectable", "aria-required"],
            Role::Treegrid => &[
                "aria-disabled",
                "aria-multiselectable",
                "aria-readonly",
                "aria-required",
            ],
            Role::Treeitem => &["aria-checked", "aria-disabled", "aria-expanded", "aria-selected"],
            _ => &[],
        }
    }

    /// Get the string representation for this role
    pub const fn as_str(&self) -> &'static str {
        match self {
            // Widget roles
            Role::Button => "button",
            Role::Checkbox => "checkbox",
            Role::Gridcell => "gridcell",
            Role::Link => "link",
            Role::Menuitem => "menuitem",
            Role::Menuitemcheckbox => "menuitemcheckbox",
            Role::Menuitemradio => "menuitemradio",
            Role::Option => "option",
            Role::Progressbar => "progressbar",
            Role::Radio => "radio",
            Role::Scrollbar => "scrollbar",
            Role::Searchbox => "searchbox",
            Role::Slider => "slider",
            Role::Spinbutton => "spinbutton",
            Role::Switch => "switch",
            Role::Tab => "tab",
            Role::Tabpanel => "tabpanel",
            Role::Textbox => "textbox",
            Role::Treeitem => "treeitem",

            // Composite roles
            Role::Combobox => "combobox",
            Role::Grid => "grid",
            Role::Listbox => "listbox",
            Role::Menu => "menu",
            Role::Menubar => "menubar",
            Role::Radiogroup => "radiogroup",
            Role::Tablist => "tablist",
            Role::Tree => "tree",
            Role::Treegrid => "treegrid",

            // Document structure
            Role::Application => "application",
            Role::Article => "article",
            Role::Cell => "cell",
            Role::Columnheader => "columnheader",
            Role::Definition => "definition",
            Role::Directory => "directory",
            Role::Document => "document",
            Role::Feed => "feed",
            Role::Figure => "figure",
            Role::Group => "group",
            Role::Heading => "heading",
            Role::Img => "img",
            Role::Image => "image",
            Role::List => "list",
            Role::Listitem => "listitem",
            Role::Math => "math",
            Role::None => "none",
            Role::Note => "note",
            Role::Presentation => "presentation",
            Role::Row => "row",
            Role::Rowgroup => "rowgroup",
            Role::Rowheader => "rowheader",
            Role::Separator => "separator",
            Role::Table => "table",
            Role::Term => "term",
            Role::Toolbar => "toolbar",
            Role::Tooltip => "tooltip",

            // Landmarks
            Role::Banner => "banner",
            Role::Complementary => "complementary",
            Role::Contentinfo => "contentinfo",
            Role::Form => "form",
            Role::Main => "main",
            Role::Navigation => "navigation",
            Role::Region => "region",
            Role::Search => "search",

            // Live regions
            Role::Alert => "alert",
            Role::Log => "log",
            Role::Marquee => "marquee",
            Role::Status => "status",
            Role::Timer => "timer",

            // Window
            Role::Alertdialog => "alertdialog",
            Role::Dialog => "dialog",
        }
    }

    /// Parse a role from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "button" => Some(Role::Button),
            "checkbox" => Some(Role::Checkbox),
            "gridcell" => Some(Role::Gridcell),
            "link" => Some(Role::Link),
            "menuitem" => Some(Role::Menuitem),
            "menuitemcheckbox" => Some(Role::Menuitemcheckbox),
            "menuitemradio" => Some(Role::Menuitemradio),
            "option" => Some(Role::Option),
            "progressbar" => Some(Role::Progressbar),
            "radio" => Some(Role::Radio),
            "scrollbar" => Some(Role::Scrollbar),
            "searchbox" => Some(Role::Searchbox),
            "slider" => Some(Role::Slider),
            "spinbutton" => Some(Role::Spinbutton),
            "switch" => Some(Role::Switch),
            "tab" => Some(Role::Tab),
            "tabpanel" => Some(Role::Tabpanel),
            "textbox" => Some(Role::Textbox),
            "treeitem" => Some(Role::Treeitem),
            "combobox" => Some(Role::Combobox),
            "grid" => Some(Role::Grid),
            "listbox" => Some(Role::Listbox),
            "menu" => Some(Role::Menu),
            "menubar" => Some(Role::Menubar),
            "radiogroup" => Some(Role::Radiogroup),
            "tablist" => Some(Role::Tablist),
            "tree" => Some(Role::Tree),
            "treegrid" => Some(Role::Treegrid),
            "application" => Some(Role::Application),
            "article" => Some(Role::Article),
            "cell" => Some(Role::Cell),
            "columnheader" => Some(Role::Columnheader),
            "definition" => Some(Role::Definition),
            "directory" => Some(Role::Directory),
            "document" => Some(Role::Document),
            "feed" => Some(Role::Feed),
            "figure" => Some(Role::Figure),
            "group" => Some(Role::Group),
            "heading" => Some(Role::Heading),
            "img" | "image" => Some(Role::Img),
            "list" => Some(Role::List),
            "listitem" => Some(Role::Listitem),
            "math" => Some(Role::Math),
            "none" => Some(Role::None),
            "note" => Some(Role::Note),
            "presentation" => Some(Role::Presentation),
            "row" => Some(Role::Row),
            "rowgroup" => Some(Role::Rowgroup),
            "rowheader" => Some(Role::Rowheader),
            "separator" => Some(Role::Separator),
            "table" => Some(Role::Table),
            "term" => Some(Role::Term),
            "toolbar" => Some(Role::Toolbar),
            "tooltip" => Some(Role::Tooltip),
            "banner" => Some(Role::Banner),
            "complementary" => Some(Role::Complementary),
            "contentinfo" => Some(Role::Contentinfo),
            "form" => Some(Role::Form),
            "main" => Some(Role::Main),
            "navigation" => Some(Role::Navigation),
            "region" => Some(Role::Region),
            "search" => Some(Role::Search),
            "alert" => Some(Role::Alert),
            "log" => Some(Role::Log),
            "marquee" => Some(Role::Marquee),
            "status" => Some(Role::Status),
            "timer" => Some(Role::Timer),
            "alertdialog" => Some(Role::Alertdialog),
            "dialog" => Some(Role::Dialog),
            _ => None,
        }
    }

    /// Get human-readable description for the role
    pub const fn description(&self) -> &'static str {
        match self {
            Role::Button => "A clickable button that triggers an action",
            Role::Checkbox => "A checkable input with true, false, or mixed states",
            Role::Link => "An interactive reference to a resource",
            Role::Textbox => "A text input field",
            Role::Searchbox => "A text input for search queries",
            Role::Slider => "An input for selecting a value from a range",
            Role::Spinbutton => "An input for selecting from a range with discrete values",
            Role::Switch => "A toggle switch for on/off states",
            Role::Progressbar => "Displays the progress of a task",
            Role::Tab => "A selectable tab in a tablist",
            Role::Tabpanel => "Content panel associated with a tab",
            Role::Tablist => "A container for tab elements",
            Role::Menu => "A list of choices or actions",
            Role::Menuitem => "An option in a menu",
            Role::Menubar => "A horizontal menu bar",
            Role::Dialog => "A dialog box or modal window",
            Role::Alertdialog => "A dialog requiring immediate user response",
            Role::Alert => "An important, time-sensitive message",
            Role::Combobox => "A text input with a popup list of options",
            Role::Listbox => "A list of selectable options",
            Role::Grid => "An interactive table-like structure",
            Role::Tree => "A hierarchical list of items",
            Role::Navigation => "A collection of navigational elements",
            Role::Main => "The main content of the document",
            Role::Banner => "Site-oriented content at the top of the page",
            Role::Contentinfo => "Footer information about the page",
            Role::Complementary => "Supporting content related to main content",
            Role::Search => "A region for searching content",
            Role::Form => "A region containing form elements",
            Role::Region => "A significant section of the document",
            Role::Heading => "A heading for a section",
            Role::List => "A list of items",
            Role::Listitem => "An item in a list",
            Role::Table => "A table with rows and columns",
            Role::Row => "A row in a table or grid",
            Role::Cell => "A cell in a table",
            Role::Img | Role::Image => "An image or graphic",
            Role::Figure => "A self-contained unit with optional caption",
            Role::Status => "Advisory information for the user",
            Role::Log => "A log of messages",
            _ => "An accessible element",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_categories() {
        assert_eq!(Role::Button.category(), RoleCategory::Widget);
        assert_eq!(Role::Combobox.category(), RoleCategory::CompositeWidget);
        assert_eq!(Role::Navigation.category(), RoleCategory::Landmark);
        assert_eq!(Role::Alert.category(), RoleCategory::LiveRegion);
        assert_eq!(Role::Dialog.category(), RoleCategory::Window);
        assert_eq!(Role::Article.category(), RoleCategory::DocumentStructure);
    }

    #[test]
    fn test_interactive_roles() {
        assert!(Role::Button.is_interactive());
        assert!(Role::Link.is_interactive());
        assert!(Role::Textbox.is_interactive());
        assert!(Role::Checkbox.is_interactive());
        assert!(Role::Slider.is_interactive());

        assert!(!Role::Article.is_interactive());
        assert!(!Role::Navigation.is_interactive());
        assert!(!Role::Heading.is_interactive());
    }

    #[test]
    fn test_landmark_roles() {
        assert!(Role::Main.is_landmark());
        assert!(Role::Navigation.is_landmark());
        assert!(Role::Banner.is_landmark());
        assert!(Role::Contentinfo.is_landmark());
        assert!(Role::Search.is_landmark());

        assert!(!Role::Button.is_landmark());
        assert!(!Role::Dialog.is_landmark());
    }

    #[test]
    fn test_live_region_roles() {
        assert!(Role::Alert.is_live_region());
        assert!(Role::Status.is_live_region());
        assert!(Role::Log.is_live_region());
        assert!(Role::Timer.is_live_region());

        assert!(!Role::Button.is_live_region());
        assert!(!Role::Navigation.is_live_region());
    }

    #[test]
    fn test_required_states() {
        assert!(Role::Checkbox
            .required_states()
            .contains(&"aria-checked"));
        assert!(Role::Slider.required_states().contains(&"aria-valuenow"));
        assert!(Role::Heading.required_states().contains(&"aria-level"));
        assert!(Role::Button.required_states().is_empty());
    }

    #[test]
    fn test_role_string_conversion() {
        assert_eq!(Role::Button.as_str(), "button");
        assert_eq!(Role::Navigation.as_str(), "navigation");
        assert_eq!(Role::Alertdialog.as_str(), "alertdialog");
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("button"), Some(Role::Button));
        assert_eq!(Role::from_str("BUTTON"), Some(Role::Button));
        assert_eq!(Role::from_str("navigation"), Some(Role::Navigation));
        assert_eq!(Role::from_str("invalid"), None);
    }

    #[test]
    fn test_role_display() {
        assert_eq!(format!("{}", Role::Button), "button");
        assert_eq!(format!("{}", Role::Navigation), "navigation");
    }

    #[test]
    fn test_nameable_roles() {
        assert!(Role::Button.is_nameable());
        assert!(Role::Link.is_nameable());

        assert!(!Role::None.is_nameable());
        assert!(!Role::Presentation.is_nameable());
    }

    #[test]
    fn test_role_serialization() {
        let role = Role::Button;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"button\"");

        let parsed: Role = serde_json::from_str("\"checkbox\"").unwrap();
        assert_eq!(parsed, Role::Checkbox);
    }
}
