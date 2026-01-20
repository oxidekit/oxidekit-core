//! Sorting logic for DataTable.
//!
//! This module provides sorting functionality including single-column and
//! multi-column sorting, with support for custom sort functions.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SortDirection {
    /// Ascending order (A-Z, 0-9).
    #[default]
    Ascending,
    /// Descending order (Z-A, 9-0).
    Descending,
}

impl SortDirection {
    /// Toggle between ascending and descending.
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    /// Apply the direction to an ordering.
    pub fn apply(&self, ordering: Ordering) -> Ordering {
        match self {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    }
}

/// Sort specification for a single column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortSpec {
    /// Column ID to sort by.
    pub column_id: String,
    /// Sort direction.
    pub direction: SortDirection,
}

impl SortSpec {
    /// Create a new sort specification.
    pub fn new(column_id: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column_id: column_id.into(),
            direction,
        }
    }

    /// Create an ascending sort specification.
    pub fn asc(column_id: impl Into<String>) -> Self {
        Self::new(column_id, SortDirection::Ascending)
    }

    /// Create a descending sort specification.
    pub fn desc(column_id: impl Into<String>) -> Self {
        Self::new(column_id, SortDirection::Descending)
    }

    /// Toggle the sort direction.
    pub fn toggle(&mut self) {
        self.direction = self.direction.toggle();
    }
}

/// Sort state for the table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SortState {
    /// Active sort specifications (for multi-column sort).
    /// First item is primary sort, subsequent items are secondary sorts.
    pub specs: Vec<SortSpec>,
    /// Maximum number of columns that can be sorted simultaneously.
    pub max_sort_columns: usize,
}

impl SortState {
    /// Create a new sort state.
    pub fn new() -> Self {
        Self {
            specs: Vec::new(),
            max_sort_columns: 3, // Default to 3 levels of sorting
        }
    }

    /// Create a sort state with a maximum number of sort columns.
    pub fn with_max_columns(max: usize) -> Self {
        Self {
            specs: Vec::new(),
            max_sort_columns: max,
        }
    }

    /// Check if any sorting is active.
    pub fn is_sorted(&self) -> bool {
        !self.specs.is_empty()
    }

    /// Get the sort direction for a column, if it's being sorted.
    pub fn get_direction(&self, column_id: &str) -> Option<SortDirection> {
        self.specs
            .iter()
            .find(|s| s.column_id == column_id)
            .map(|s| s.direction)
    }

    /// Get the sort priority for a column (1-based), if it's being sorted.
    pub fn get_priority(&self, column_id: &str) -> Option<usize> {
        self.specs
            .iter()
            .position(|s| s.column_id == column_id)
            .map(|i| i + 1)
    }

    /// Toggle sorting on a column.
    /// - If column is not sorted, add ascending sort.
    /// - If column is sorted ascending, change to descending.
    /// - If column is sorted descending, remove the sort.
    pub fn toggle_column(&mut self, column_id: &str) {
        if let Some(idx) = self.specs.iter().position(|s| s.column_id == column_id) {
            let spec = &mut self.specs[idx];
            match spec.direction {
                SortDirection::Ascending => {
                    spec.direction = SortDirection::Descending;
                }
                SortDirection::Descending => {
                    self.specs.remove(idx);
                }
            }
        } else {
            // Add new sort
            if self.specs.len() < self.max_sort_columns {
                self.specs.push(SortSpec::asc(column_id));
            } else {
                // Replace the last sort
                self.specs.pop();
                self.specs.push(SortSpec::asc(column_id));
            }
        }
    }

    /// Set a column as the primary sort, replacing all other sorts.
    pub fn set_primary(&mut self, column_id: &str, direction: SortDirection) {
        self.specs.clear();
        self.specs.push(SortSpec::new(column_id, direction));
    }

    /// Add a secondary sort (for multi-column sorting).
    /// If the column is already being sorted, move it to the end.
    pub fn add_secondary(&mut self, column_id: &str, direction: SortDirection) {
        // Remove existing sort for this column
        self.specs.retain(|s| s.column_id != column_id);

        // Add as secondary sort
        if self.specs.len() < self.max_sort_columns {
            self.specs.push(SortSpec::new(column_id, direction));
        }
    }

    /// Clear all sorts.
    pub fn clear(&mut self) {
        self.specs.clear();
    }
}

/// Trait for sortable values.
pub trait Sortable {
    /// Compare two values for sorting.
    fn compare(&self, other: &Self) -> Ordering;
}

impl Sortable for String {
    fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl Sortable for &str {
    fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl Sortable for i32 {
    fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl Sortable for i64 {
    fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl Sortable for u32 {
    fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl Sortable for u64 {
    fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl Sortable for f32 {
    fn compare(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Sortable for f64 {
    fn compare(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Sortable for bool {
    fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl<T: Sortable> Sortable for Option<T> {
    fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Some(a), Some(b)) => a.compare(b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

/// Sorter for sorting table data.
pub struct Sorter<T> {
    /// Value extractor functions by column ID.
    extractors: Vec<(String, Box<dyn Fn(&T) -> Box<dyn Sortable + '_> + Send + Sync>)>,
}

impl<T> Default for Sorter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Sorter<T> {
    /// Create a new sorter.
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    /// Add a value extractor for a column.
    pub fn add_extractor<F, V>(mut self, column_id: impl Into<String>, extractor: F) -> Self
    where
        F: Fn(&T) -> V + Send + Sync + 'static,
        V: Sortable + 'static,
    {
        let column_id = column_id.into();
        self.extractors.push((
            column_id,
            Box::new(move |item| Box::new(extractor(item))),
        ));
        self
    }

    /// Sort data according to the sort state.
    pub fn sort(&self, data: &mut [T], state: &SortState) {
        if state.specs.is_empty() {
            return;
        }

        data.sort_by(|a, b| {
            for spec in &state.specs {
                if let Some((_, extractor)) = self.extractors.iter().find(|(id, _)| id == &spec.column_id) {
                    let val_a = extractor(a);
                    let val_b = extractor(b);
                    let ordering = val_a.compare(&*val_b);
                    let directed = spec.direction.apply(ordering);
                    if directed != Ordering::Equal {
                        return directed;
                    }
                }
            }
            Ordering::Equal
        });
    }

    /// Get sorted indices without modifying the original data.
    pub fn sorted_indices(&self, data: &[T], state: &SortState) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..data.len()).collect();

        if state.specs.is_empty() {
            return indices;
        }

        indices.sort_by(|&i, &j| {
            let a = &data[i];
            let b = &data[j];
            for spec in &state.specs {
                if let Some((_, extractor)) = self.extractors.iter().find(|(id, _)| id == &spec.column_id) {
                    let val_a = extractor(a);
                    let val_b = extractor(b);
                    let ordering = val_a.compare(&*val_b);
                    let directed = spec.direction.apply(ordering);
                    if directed != Ordering::Equal {
                        return directed;
                    }
                }
            }
            Ordering::Equal
        });

        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_direction_toggle() {
        assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
        assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_sort_direction_apply() {
        assert_eq!(SortDirection::Ascending.apply(Ordering::Less), Ordering::Less);
        assert_eq!(SortDirection::Descending.apply(Ordering::Less), Ordering::Greater);
    }

    #[test]
    fn test_sort_spec() {
        let spec = SortSpec::asc("name");
        assert_eq!(spec.column_id, "name");
        assert_eq!(spec.direction, SortDirection::Ascending);

        let spec = SortSpec::desc("age");
        assert_eq!(spec.direction, SortDirection::Descending);
    }

    #[test]
    fn test_sort_state_toggle() {
        let mut state = SortState::new();

        // First click: add ascending
        state.toggle_column("name");
        assert_eq!(state.get_direction("name"), Some(SortDirection::Ascending));
        assert_eq!(state.get_priority("name"), Some(1));

        // Second click: change to descending
        state.toggle_column("name");
        assert_eq!(state.get_direction("name"), Some(SortDirection::Descending));

        // Third click: remove sort
        state.toggle_column("name");
        assert_eq!(state.get_direction("name"), None);
    }

    #[test]
    fn test_sort_state_multi_column() {
        let mut state = SortState::new();

        state.toggle_column("name");
        state.toggle_column("email");

        assert_eq!(state.get_priority("name"), Some(1));
        assert_eq!(state.get_priority("email"), Some(2));
    }

    #[test]
    fn test_sort_state_max_columns() {
        let mut state = SortState::with_max_columns(2);

        state.toggle_column("col1");
        state.toggle_column("col2");
        state.toggle_column("col3"); // Should replace col2

        assert_eq!(state.specs.len(), 2);
        assert_eq!(state.get_priority("col1"), Some(1));
        assert_eq!(state.get_priority("col3"), Some(2));
        assert_eq!(state.get_priority("col2"), None);
    }

    #[test]
    fn test_sort_state_set_primary() {
        let mut state = SortState::new();
        state.toggle_column("name");
        state.toggle_column("email");

        state.set_primary("age", SortDirection::Descending);

        assert_eq!(state.specs.len(), 1);
        assert_eq!(state.get_direction("age"), Some(SortDirection::Descending));
    }

    #[test]
    fn test_sort_state_add_secondary() {
        let mut state = SortState::new();
        state.set_primary("name", SortDirection::Ascending);
        state.add_secondary("email", SortDirection::Descending);

        assert_eq!(state.specs.len(), 2);
        assert_eq!(state.get_priority("name"), Some(1));
        assert_eq!(state.get_priority("email"), Some(2));
    }

    #[test]
    fn test_sorter_basic() {
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32,
        }

        let mut users = vec![
            User { name: "Charlie".into(), age: 30 },
            User { name: "Alice".into(), age: 25 },
            User { name: "Bob".into(), age: 35 },
        ];

        let sorter = Sorter::new()
            .add_extractor("name", |u: &User| u.name.clone())
            .add_extractor("age", |u: &User| u.age);

        let mut state = SortState::new();
        state.set_primary("name", SortDirection::Ascending);

        sorter.sort(&mut users, &state);

        assert_eq!(users[0].name, "Alice");
        assert_eq!(users[1].name, "Bob");
        assert_eq!(users[2].name, "Charlie");
    }

    #[test]
    fn test_sorter_descending() {
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32,
        }

        let mut users = vec![
            User { name: "Alice".into(), age: 25 },
            User { name: "Bob".into(), age: 35 },
            User { name: "Charlie".into(), age: 30 },
        ];

        let sorter = Sorter::new()
            .add_extractor("age", |u: &User| u.age);

        let mut state = SortState::new();
        state.set_primary("age", SortDirection::Descending);

        sorter.sort(&mut users, &state);

        assert_eq!(users[0].age, 35);
        assert_eq!(users[1].age, 30);
        assert_eq!(users[2].age, 25);
    }

    #[test]
    fn test_sorter_multi_column() {
        #[derive(Debug)]
        struct User {
            name: String,
            department: String,
        }

        let mut users = vec![
            User { name: "Charlie".into(), department: "Engineering".into() },
            User { name: "Alice".into(), department: "Marketing".into() },
            User { name: "Bob".into(), department: "Engineering".into() },
            User { name: "Diana".into(), department: "Engineering".into() },
        ];

        let sorter = Sorter::new()
            .add_extractor("name", |u: &User| u.name.clone())
            .add_extractor("department", |u: &User| u.department.clone());

        let mut state = SortState::new();
        state.set_primary("department", SortDirection::Ascending);
        state.add_secondary("name", SortDirection::Ascending);

        sorter.sort(&mut users, &state);

        // Engineering first (sorted by name: Bob, Charlie, Diana), then Marketing
        assert_eq!(users[0].name, "Bob");
        assert_eq!(users[1].name, "Charlie");
        assert_eq!(users[2].name, "Diana");
        assert_eq!(users[3].name, "Alice");
    }

    #[test]
    fn test_sorter_sorted_indices() {
        #[derive(Debug)]
        struct User {
            name: String,
        }

        let users = vec![
            User { name: "Charlie".into() },
            User { name: "Alice".into() },
            User { name: "Bob".into() },
        ];

        let sorter = Sorter::new()
            .add_extractor("name", |u: &User| u.name.clone());

        let mut state = SortState::new();
        state.set_primary("name", SortDirection::Ascending);

        let indices = sorter.sorted_indices(&users, &state);

        assert_eq!(indices, vec![1, 2, 0]); // Alice (1), Bob (2), Charlie (0)
    }

    #[test]
    fn test_sortable_option() {
        let a: Option<i32> = Some(5);
        let b: Option<i32> = Some(10);
        let c: Option<i32> = None;

        assert_eq!(a.compare(&b), Ordering::Less);
        assert_eq!(b.compare(&a), Ordering::Greater);
        assert_eq!(a.compare(&c), Ordering::Less);
        assert_eq!(c.compare(&a), Ordering::Greater);
        assert_eq!(c.compare(&c), Ordering::Equal);
    }
}
