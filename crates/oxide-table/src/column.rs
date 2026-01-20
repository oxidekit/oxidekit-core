//! Column definitions and configuration for DataTable.
//!
//! This module provides the column definition system that allows specifying
//! column properties like name, type, width, sortability, and filterability.

use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::Ordering;
use std::fmt;

/// The type of data a column contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColumnType {
    /// Plain text data.
    #[default]
    Text,
    /// Numeric data (integers or floats).
    Number,
    /// Date values.
    Date,
    /// DateTime values.
    DateTime,
    /// Boolean values.
    Boolean,
    /// Currency values.
    Currency,
    /// Percentage values.
    Percentage,
    /// Custom type with specific handling.
    Custom(ColumnTypeId),
}

/// Identifier for custom column types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColumnTypeId(pub u32);

/// Alignment for column content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColumnAlign {
    /// Left-aligned (default for text).
    #[default]
    Left,
    /// Center-aligned.
    Center,
    /// Right-aligned (default for numbers).
    Right,
}

/// Column resize mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColumnResize {
    /// Column cannot be resized.
    #[default]
    None,
    /// Column can be resized freely.
    Free,
    /// Column can be resized within bounds.
    Bounded {
        min_width: u32,
        max_width: u32,
    },
}

/// Position for frozen/sticky columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrozenPosition {
    /// Frozen to the left edge.
    Left,
    /// Frozen to the right edge.
    Right,
}

/// Custom sort function type.
pub type SortFn<T> = Box<dyn Fn(&T, &T) -> Ordering + Send + Sync>;

/// Custom formatter function type.
pub type FormatFn<T> = Box<dyn Fn(&T) -> String + Send + Sync>;

/// Custom cell renderer function type.
pub type RenderFn<T> = Box<dyn Fn(&T) -> Box<dyn Any + Send + Sync> + Send + Sync>;

/// Column definition for a DataTable.
///
/// # Example
///
/// ```rust
/// use oxide_table::column::{Column, ColumnType};
///
/// let col = Column::<String>::new("name", "Name")
///     .sortable()
///     .filterable()
///     .width(200);
/// ```
pub struct Column<T: 'static> {
    /// Unique identifier for the column (used for data binding).
    pub id: String,
    /// Display header for the column.
    pub header: String,
    /// Data type of the column.
    pub column_type: ColumnType,
    /// Width in pixels (None for auto).
    pub width: Option<u32>,
    /// Minimum width in pixels.
    pub min_width: Option<u32>,
    /// Maximum width in pixels.
    pub max_width: Option<u32>,
    /// Whether the column is sortable.
    pub sortable: bool,
    /// Whether the column is filterable.
    pub filterable: bool,
    /// Filter options for select-style filters.
    pub filter_options: Option<Vec<String>>,
    /// Whether the column is visible.
    pub visible: bool,
    /// Whether the column can be reordered.
    pub reorderable: bool,
    /// Resize mode for the column.
    pub resize: ColumnResize,
    /// Content alignment.
    pub align: ColumnAlign,
    /// Frozen position (None for normal scrolling).
    pub frozen: Option<FrozenPosition>,
    /// Column group header (for grouped columns).
    pub group: Option<String>,
    /// Whether this column shows row selection checkboxes.
    pub is_checkbox: bool,
    /// Whether this column shows row actions.
    pub is_actions: bool,
    /// Custom sort function.
    pub sort_fn: Option<SortFn<T>>,
    /// Custom format function.
    pub format_fn: Option<FormatFn<T>>,
    /// Custom render function.
    pub render_fn: Option<RenderFn<T>>,
    /// Whether the column is editable.
    pub editable: bool,
    /// Tooltip text for the column header.
    pub tooltip: Option<String>,
    /// CSS class names for the column.
    pub class_names: Vec<String>,
}

impl<T: 'static> fmt::Debug for Column<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Column")
            .field("id", &self.id)
            .field("header", &self.header)
            .field("column_type", &self.column_type)
            .field("width", &self.width)
            .field("sortable", &self.sortable)
            .field("filterable", &self.filterable)
            .field("visible", &self.visible)
            .finish()
    }
}

impl<T: 'static> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            header: self.header.clone(),
            column_type: self.column_type,
            width: self.width,
            min_width: self.min_width,
            max_width: self.max_width,
            sortable: self.sortable,
            filterable: self.filterable,
            filter_options: self.filter_options.clone(),
            visible: self.visible,
            reorderable: self.reorderable,
            resize: self.resize,
            align: self.align,
            frozen: self.frozen,
            group: self.group.clone(),
            is_checkbox: self.is_checkbox,
            is_actions: self.is_actions,
            sort_fn: None, // Cannot clone function
            format_fn: None, // Cannot clone function
            render_fn: None, // Cannot clone function
            editable: self.editable,
            tooltip: self.tooltip.clone(),
            class_names: self.class_names.clone(),
        }
    }
}

impl<T: 'static> Column<T> {
    /// Create a new column with the given ID and header.
    pub fn new(id: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            column_type: ColumnType::Text,
            width: None,
            min_width: None,
            max_width: None,
            sortable: false,
            filterable: false,
            filter_options: None,
            visible: true,
            reorderable: true,
            resize: ColumnResize::None,
            align: ColumnAlign::Left,
            frozen: None,
            group: None,
            is_checkbox: false,
            is_actions: false,
            sort_fn: None,
            format_fn: None,
            render_fn: None,
            editable: false,
            tooltip: None,
            class_names: Vec::new(),
        }
    }

    /// Create a checkbox selection column.
    pub fn checkbox() -> Self {
        Self::new("__checkbox", "")
            .width(40)
            .frozen_left()
            .align(ColumnAlign::Center)
            .reorderable(false)
            .set_checkbox(true)
    }

    /// Create an actions column.
    pub fn actions(header: impl Into<String>) -> Self {
        Self::new("__actions", header)
            .frozen_right()
            .reorderable(false)
            .set_actions(true)
    }

    /// Set the column type.
    pub fn type_(mut self, column_type: ColumnType) -> Self {
        self.column_type = column_type;
        // Auto-set alignment based on type
        if matches!(column_type, ColumnType::Number | ColumnType::Currency | ColumnType::Percentage) {
            self.align = ColumnAlign::Right;
        }
        self
    }

    /// Set the column width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the minimum column width in pixels.
    pub fn min_width(mut self, width: u32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set the maximum column width in pixels.
    pub fn max_width(mut self, width: u32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Make the column sortable.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Make the column filterable with text filter.
    pub fn filterable(mut self) -> Self {
        self.filterable = true;
        self
    }

    /// Make the column filterable with select options.
    pub fn filterable_select(mut self, options: &[&str]) -> Self {
        self.filterable = true;
        self.filter_options = Some(options.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Set column visibility.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set whether the column can be reordered.
    pub fn reorderable(mut self, reorderable: bool) -> Self {
        self.reorderable = reorderable;
        self
    }

    /// Make the column resizable.
    pub fn resizable(mut self) -> Self {
        self.resize = ColumnResize::Free;
        self
    }

    /// Make the column resizable within bounds.
    pub fn resizable_bounded(mut self, min: u32, max: u32) -> Self {
        self.resize = ColumnResize::Bounded {
            min_width: min,
            max_width: max,
        };
        self
    }

    /// Set the content alignment.
    pub fn align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        self
    }

    /// Freeze the column to the left.
    pub fn frozen_left(mut self) -> Self {
        self.frozen = Some(FrozenPosition::Left);
        self
    }

    /// Freeze the column to the right.
    pub fn frozen_right(mut self) -> Self {
        self.frozen = Some(FrozenPosition::Right);
        self
    }

    /// Set the column group header.
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Mark this as a checkbox column.
    fn set_checkbox(mut self, is_checkbox: bool) -> Self {
        self.is_checkbox = is_checkbox;
        self
    }

    /// Mark this as an actions column.
    fn set_actions(mut self, is_actions: bool) -> Self {
        self.is_actions = is_actions;
        self
    }

    /// Set a custom sort function.
    pub fn with_sort_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
    {
        self.sort_fn = Some(Box::new(f));
        self
    }

    /// Set a custom format function.
    pub fn with_format_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> String + Send + Sync + 'static,
    {
        self.format_fn = Some(Box::new(f));
        self
    }

    /// Set a custom render function.
    pub fn with_render_fn<F, R>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> R + Send + Sync + 'static,
        R: Any + Send + Sync + 'static,
    {
        self.render_fn = Some(Box::new(move |t| Box::new(f(t))));
        self
    }

    /// Make the column editable.
    pub fn editable(mut self) -> Self {
        self.editable = true;
        self
    }

    /// Set a tooltip for the column header.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Add a CSS class to the column.
    pub fn class(mut self, class_name: impl Into<String>) -> Self {
        self.class_names.push(class_name.into());
        self
    }

    /// Get the effective width, falling back to default.
    pub fn effective_width(&self) -> u32 {
        self.width.unwrap_or(100)
    }
}

/// Column group for grouped column headers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnGroup {
    /// Group identifier.
    pub id: String,
    /// Display header for the group.
    pub header: String,
    /// Column IDs in this group.
    pub columns: Vec<String>,
}

impl ColumnGroup {
    /// Create a new column group.
    pub fn new(id: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            columns: Vec::new(),
        }
    }

    /// Add a column to the group.
    pub fn add_column(mut self, column_id: impl Into<String>) -> Self {
        self.columns.push(column_id.into());
        self
    }

    /// Add multiple columns to the group.
    pub fn add_columns(mut self, column_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.columns.extend(column_ids.into_iter().map(Into::into));
        self
    }
}

/// Column state for tracking runtime column configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnState {
    /// Column ID.
    pub id: String,
    /// Current width.
    pub width: u32,
    /// Current visibility.
    pub visible: bool,
    /// Current order index.
    pub order: usize,
}

impl ColumnState {
    /// Create a new column state from a column definition.
    pub fn from_column<T>(column: &Column<T>, order: usize) -> Self {
        Self {
            id: column.id.clone(),
            width: column.effective_width(),
            visible: column.visible,
            order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_creation() {
        let col = Column::<String>::new("name", "Name");
        assert_eq!(col.id, "name");
        assert_eq!(col.header, "Name");
        assert_eq!(col.column_type, ColumnType::Text);
        assert!(!col.sortable);
        assert!(!col.filterable);
        assert!(col.visible);
    }

    #[test]
    fn test_column_builder() {
        let col = Column::<String>::new("email", "Email")
            .sortable()
            .filterable()
            .width(200)
            .type_(ColumnType::Text);

        assert_eq!(col.id, "email");
        assert!(col.sortable);
        assert!(col.filterable);
        assert_eq!(col.width, Some(200));
    }

    #[test]
    fn test_column_auto_alignment() {
        let text_col = Column::<String>::new("name", "Name").type_(ColumnType::Text);
        assert_eq!(text_col.align, ColumnAlign::Left);

        let num_col = Column::<i32>::new("amount", "Amount").type_(ColumnType::Number);
        assert_eq!(num_col.align, ColumnAlign::Right);

        let currency_col = Column::<f64>::new("price", "Price").type_(ColumnType::Currency);
        assert_eq!(currency_col.align, ColumnAlign::Right);
    }

    #[test]
    fn test_checkbox_column() {
        let col = Column::<String>::checkbox();
        assert!(col.is_checkbox);
        assert_eq!(col.width, Some(40));
        assert!(matches!(col.frozen, Some(FrozenPosition::Left)));
    }

    #[test]
    fn test_actions_column() {
        let col = Column::<String>::actions("Actions");
        assert!(col.is_actions);
        assert!(matches!(col.frozen, Some(FrozenPosition::Right)));
    }

    #[test]
    fn test_column_resizable() {
        let col = Column::<String>::new("name", "Name").resizable();
        assert!(matches!(col.resize, ColumnResize::Free));

        let bounded_col = Column::<String>::new("name", "Name").resizable_bounded(50, 300);
        assert!(matches!(
            bounded_col.resize,
            ColumnResize::Bounded { min_width: 50, max_width: 300 }
        ));
    }

    #[test]
    fn test_column_frozen() {
        let left_col = Column::<String>::new("name", "Name").frozen_left();
        assert!(matches!(left_col.frozen, Some(FrozenPosition::Left)));

        let right_col = Column::<String>::new("actions", "Actions").frozen_right();
        assert!(matches!(right_col.frozen, Some(FrozenPosition::Right)));
    }

    #[test]
    fn test_filterable_select() {
        let col = Column::<String>::new("status", "Status")
            .filterable_select(&["Active", "Inactive", "Pending"]);

        assert!(col.filterable);
        assert!(col.filter_options.is_some());
        let options = col.filter_options.unwrap();
        assert_eq!(options.len(), 3);
        assert!(options.contains(&"Active".to_string()));
    }

    #[test]
    fn test_column_group() {
        let group = ColumnGroup::new("personal", "Personal Info")
            .add_column("name")
            .add_column("email")
            .add_columns(vec!["phone", "address"]);

        assert_eq!(group.id, "personal");
        assert_eq!(group.header, "Personal Info");
        assert_eq!(group.columns.len(), 4);
    }

    #[test]
    fn test_column_state() {
        let col = Column::<String>::new("name", "Name").width(150).visible(true);
        let state = ColumnState::from_column(&col, 0);

        assert_eq!(state.id, "name");
        assert_eq!(state.width, 150);
        assert!(state.visible);
        assert_eq!(state.order, 0);
    }

    #[test]
    fn test_column_effective_width() {
        let col_with_width = Column::<String>::new("name", "Name").width(200);
        assert_eq!(col_with_width.effective_width(), 200);

        let col_without_width = Column::<String>::new("name", "Name");
        assert_eq!(col_without_width.effective_width(), 100); // default
    }

    #[test]
    fn test_column_classes() {
        let col = Column::<String>::new("name", "Name")
            .class("highlight")
            .class("primary");

        assert_eq!(col.class_names.len(), 2);
        assert!(col.class_names.contains(&"highlight".to_string()));
        assert!(col.class_names.contains(&"primary".to_string()));
    }
}
