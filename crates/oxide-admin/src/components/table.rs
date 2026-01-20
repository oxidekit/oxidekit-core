//! Table component
//!
//! Advanced data table with sorting, filtering, pagination, and virtualization support.

use oxide_layout::{NodeId, LayoutTree, Style, StyleBuilder, NodeVisual};
use oxide_render::Color;
use std::cmp::Ordering;

/// Column definition for table
#[derive(Debug, Clone)]
pub struct TableColumn {
    /// Column key/id
    pub key: String,

    /// Column header text
    pub header: String,

    /// Column width (fixed or flex)
    pub width: ColumnWidth,

    /// Whether column is sortable
    pub sortable: bool,

    /// Text alignment
    pub align: TextAlign,

    /// Whether column is visible
    pub visible: bool,

    /// Cell renderer type
    pub cell_type: CellType,
}

/// Column width specification
#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    /// Fixed width in pixels
    Fixed(f32),
    /// Flexible width (weight)
    Flex(f32),
    /// Auto width based on content
    Auto,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Cell content type
#[derive(Debug, Clone, Default)]
pub enum CellType {
    #[default]
    Text,
    Number,
    Badge,
    Status,
    Avatar,
    Actions,
    Custom(String),
}

impl TableColumn {
    /// Create a new text column
    pub fn text(key: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            width: ColumnWidth::Flex(1.0),
            sortable: true,
            align: TextAlign::Left,
            visible: true,
            cell_type: CellType::Text,
        }
    }

    /// Create a number column
    pub fn number(key: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            width: ColumnWidth::Fixed(100.0),
            sortable: true,
            align: TextAlign::Right,
            visible: true,
            cell_type: CellType::Number,
        }
    }

    /// Create a status column
    pub fn status(key: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            width: ColumnWidth::Fixed(120.0),
            sortable: true,
            align: TextAlign::Center,
            visible: true,
            cell_type: CellType::Status,
        }
    }

    /// Create an actions column
    pub fn actions(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            header: "Actions".to_string(),
            width: ColumnWidth::Fixed(100.0),
            sortable: false,
            align: TextAlign::Right,
            visible: true,
            cell_type: CellType::Actions,
        }
    }

    /// Set column width
    pub fn width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }

    /// Set sortable
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Set alignment
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SortDirection {
    #[default]
    None,
    Ascending,
    Descending,
}

/// Sort state for table
#[derive(Debug, Clone, Default)]
pub struct SortState {
    pub column: Option<String>,
    pub direction: SortDirection,
}

impl SortState {
    /// Toggle sort for a column
    pub fn toggle(&mut self, column: &str) {
        if self.column.as_deref() == Some(column) {
            self.direction = match self.direction {
                SortDirection::None => SortDirection::Ascending,
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::None,
            };
            if self.direction == SortDirection::None {
                self.column = None;
            }
        } else {
            self.column = Some(column.to_string());
            self.direction = SortDirection::Ascending;
        }
    }
}

/// Pagination state
#[derive(Debug, Clone)]
pub struct PaginationState {
    pub page: usize,
    pub page_size: usize,
    pub total_items: usize,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 25,
            total_items: 0,
        }
    }
}

impl PaginationState {
    /// Get total number of pages
    pub fn total_pages(&self) -> usize {
        if self.total_items == 0 {
            1
        } else {
            (self.total_items + self.page_size - 1) / self.page_size
        }
    }

    /// Get start index for current page
    pub fn start_index(&self) -> usize {
        (self.page - 1) * self.page_size
    }

    /// Get end index for current page
    pub fn end_index(&self) -> usize {
        (self.start_index() + self.page_size).min(self.total_items)
    }

    /// Go to next page
    pub fn next(&mut self) {
        if self.page < self.total_pages() {
            self.page += 1;
        }
    }

    /// Go to previous page
    pub fn prev(&mut self) {
        if self.page > 1 {
            self.page -= 1;
        }
    }

    /// Go to specific page
    pub fn go_to(&mut self, page: usize) {
        self.page = page.clamp(1, self.total_pages());
    }
}

/// Table row selection state
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub selected_rows: Vec<String>,
    pub select_all: bool,
}

impl SelectionState {
    /// Toggle selection of a row
    pub fn toggle(&mut self, row_id: &str) {
        if let Some(pos) = self.selected_rows.iter().position(|id| id == row_id) {
            self.selected_rows.remove(pos);
            self.select_all = false;
        } else {
            self.selected_rows.push(row_id.to_string());
        }
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.selected_rows.clear();
        self.select_all = false;
    }

    /// Check if row is selected
    pub fn is_selected(&self, row_id: &str) -> bool {
        self.select_all || self.selected_rows.contains(&row_id.to_string())
    }
}

/// Table properties
#[derive(Debug, Clone)]
pub struct TableProps {
    /// Table columns
    pub columns: Vec<TableColumn>,

    /// Enable row selection
    pub selectable: bool,

    /// Enable multi-select
    pub multi_select: bool,

    /// Show header
    pub show_header: bool,

    /// Enable striped rows
    pub striped: bool,

    /// Enable hover effect
    pub hoverable: bool,

    /// Compact mode
    pub compact: bool,

    /// Show border
    pub bordered: bool,

    /// Enable virtualization
    pub virtualized: bool,

    /// Estimated row height (for virtualization)
    pub row_height: f32,
}

impl Default for TableProps {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            selectable: false,
            multi_select: false,
            show_header: true,
            striped: true,
            hoverable: true,
            compact: false,
            bordered: true,
            virtualized: false,
            row_height: 48.0,
        }
    }
}

impl TableProps {
    /// Create new table props with columns
    pub fn new(columns: Vec<TableColumn>) -> Self {
        Self {
            columns,
            ..Default::default()
        }
    }

    /// Enable row selection
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Enable multi-select
    pub fn multi_select(mut self, multi_select: bool) -> Self {
        self.multi_select = multi_select;
        self
    }

    /// Set striped
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Set compact
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Enable virtualization
    pub fn virtualized(mut self, virtualized: bool) -> Self {
        self.virtualized = virtualized;
        self
    }
}

/// Table component
pub struct Table;

impl Table {
    /// Build table layout
    pub fn build(
        tree: &mut LayoutTree,
        props: TableProps,
        rows: &[TableRow],
        sort_state: &SortState,
        pagination: Option<&PaginationState>,
    ) -> NodeId {
        let mut children = Vec::new();

        // Build header
        if props.show_header {
            let header = Self::build_header(tree, &props, sort_state);
            children.push(header);
        }

        // Build body
        let body = Self::build_body(tree, &props, rows, pagination);
        children.push(body);

        // Build pagination if provided
        if let Some(pag) = pagination {
            let pag_node = Self::build_pagination(tree, pag);
            children.push(pag_node);
        }

        // Table container
        let table_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .build();

        let table_visual = if props.bordered {
            NodeVisual::default()
                .with_border(hex_to_rgba("#374151"), 1.0)
                .with_radius(8.0)
        } else {
            NodeVisual::default()
        };

        tree.new_visual_node_with_children(table_style, table_visual, &children)
    }

    /// Build table header
    fn build_header(tree: &mut LayoutTree, props: &TableProps, sort_state: &SortState) -> NodeId {
        let row_height = if props.compact { 36.0 } else { 48.0 };

        let header_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(row_height)
            .padding_xy(0.0, 16.0)
            .build();

        let header_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#111827"))
            .with_border(hex_to_rgba("#374151"), 1.0);

        // Build header cells
        let cells: Vec<NodeId> = props
            .columns
            .iter()
            .filter(|c| c.visible)
            .map(|col| Self::build_header_cell(tree, col, sort_state))
            .collect();

        tree.new_visual_node_with_children(header_style, header_visual, &cells)
    }

    /// Build header cell
    fn build_header_cell(tree: &mut LayoutTree, column: &TableColumn, sort_state: &SortState) -> NodeId {
        let width_style = match column.width {
            ColumnWidth::Fixed(w) => StyleBuilder::new().width(w),
            ColumnWidth::Flex(f) => StyleBuilder::new().flex_grow(f),
            ColumnWidth::Auto => StyleBuilder::new(),
        };

        let cell_style = width_style
            .flex_row()
            .align_center()
            .padding_xy(8.0, 12.0)
            .gap(4.0)
            .build();

        tree.new_node(cell_style)
    }

    /// Build table body
    fn build_body(
        tree: &mut LayoutTree,
        props: &TableProps,
        rows: &[TableRow],
        pagination: Option<&PaginationState>,
    ) -> NodeId {
        let body_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .build();

        // Apply pagination
        let visible_rows: Vec<&TableRow> = if let Some(pag) = pagination {
            rows.iter()
                .skip(pag.start_index())
                .take(pag.page_size)
                .collect()
        } else {
            rows.iter().collect()
        };

        // Build rows
        let row_nodes: Vec<NodeId> = visible_rows
            .iter()
            .enumerate()
            .map(|(idx, row)| Self::build_row(tree, props, row, idx))
            .collect();

        tree.new_node_with_children(body_style, &row_nodes)
    }

    /// Build table row
    fn build_row(tree: &mut LayoutTree, props: &TableProps, row: &TableRow, index: usize) -> NodeId {
        let row_height = if props.compact { 36.0 } else { 48.0 };

        let row_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(row_height)
            .padding_xy(0.0, 16.0)
            .build();

        let bg_color = if props.striped && index % 2 == 1 {
            hex_to_rgba("#111827")
        } else {
            hex_to_rgba("#0000")
        };

        let row_visual = NodeVisual::default()
            .with_background(bg_color)
            .with_border(hex_to_rgba("#374151"), 1.0);

        // Build cells
        let cells: Vec<NodeId> = props
            .columns
            .iter()
            .filter(|c| c.visible)
            .map(|col| {
                let value = row.cells.get(&col.key).map(|v| v.as_str()).unwrap_or("");
                Self::build_cell(tree, col, value)
            })
            .collect();

        tree.new_visual_node_with_children(row_style, row_visual, &cells)
    }

    /// Build table cell
    fn build_cell(tree: &mut LayoutTree, column: &TableColumn, _value: &str) -> NodeId {
        let width_style = match column.width {
            ColumnWidth::Fixed(w) => StyleBuilder::new().width(w),
            ColumnWidth::Flex(f) => StyleBuilder::new().flex_grow(f),
            ColumnWidth::Auto => StyleBuilder::new(),
        };

        let cell_style = width_style
            .flex_row()
            .align_center()
            .padding_xy(8.0, 12.0)
            .build();

        tree.new_node(cell_style)
    }

    /// Build pagination controls
    fn build_pagination(tree: &mut LayoutTree, state: &PaginationState) -> NodeId {
        let pag_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .height(52.0)
            .padding_xy(12.0, 16.0)
            .build();

        let pag_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#111827"))
            .with_border(hex_to_rgba("#374151"), 1.0);

        tree.new_visual_node(pag_style, pag_visual)
    }
}

/// Table row data
#[derive(Debug, Clone, Default)]
pub struct TableRow {
    /// Row ID
    pub id: String,
    /// Cell values by column key
    pub cells: std::collections::HashMap<String, String>,
}

impl TableRow {
    /// Create a new row with ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            cells: std::collections::HashMap::new(),
        }
    }

    /// Add cell value
    pub fn cell(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.cells.insert(key.into(), value.into());
        self
    }
}

/// Convert hex color to RGBA array
fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex)
        .map(|c| c.to_array())
        .unwrap_or([1.0, 1.0, 1.0, 1.0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_column() {
        let col = TableColumn::text("name", "Name")
            .width(ColumnWidth::Fixed(200.0))
            .sortable(true);

        assert_eq!(col.key, "name");
        assert_eq!(col.header, "Name");
    }

    #[test]
    fn test_pagination() {
        let mut pag = PaginationState {
            page: 1,
            page_size: 25,
            total_items: 100,
        };

        assert_eq!(pag.total_pages(), 4);
        assert_eq!(pag.start_index(), 0);
        assert_eq!(pag.end_index(), 25);

        pag.next();
        assert_eq!(pag.page, 2);
        assert_eq!(pag.start_index(), 25);
    }

    #[test]
    fn test_sort_state() {
        let mut sort = SortState::default();

        sort.toggle("name");
        assert_eq!(sort.column, Some("name".to_string()));
        assert_eq!(sort.direction, SortDirection::Ascending);

        sort.toggle("name");
        assert_eq!(sort.direction, SortDirection::Descending);

        sort.toggle("name");
        assert_eq!(sort.direction, SortDirection::None);
        assert_eq!(sort.column, None);
    }
}
