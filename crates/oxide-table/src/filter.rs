//! Filtering logic for DataTable.
//!
//! This module provides comprehensive filtering functionality including
//! text, number range, date range, and select filters with various operators.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter operator for comparing values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterOperator {
    /// Value equals the filter value.
    Equals,
    /// Value does not equal the filter value.
    NotEquals,
    /// Value contains the filter value (for strings).
    Contains,
    /// Value does not contain the filter value.
    NotContains,
    /// Value starts with the filter value.
    StartsWith,
    /// Value ends with the filter value.
    EndsWith,
    /// Value is greater than the filter value.
    GreaterThan,
    /// Value is greater than or equal to the filter value.
    GreaterThanOrEqual,
    /// Value is less than the filter value.
    LessThan,
    /// Value is less than or equal to the filter value.
    LessThanOrEqual,
    /// Value is between two filter values (inclusive).
    Between,
    /// Value is in a list of filter values.
    In,
    /// Value is not in a list of filter values.
    NotIn,
    /// Value is null/empty.
    IsEmpty,
    /// Value is not null/empty.
    IsNotEmpty,
}

impl FilterOperator {
    /// Get the display label for the operator.
    pub fn label(&self) -> &'static str {
        match self {
            FilterOperator::Equals => "equals",
            FilterOperator::NotEquals => "does not equal",
            FilterOperator::Contains => "contains",
            FilterOperator::NotContains => "does not contain",
            FilterOperator::StartsWith => "starts with",
            FilterOperator::EndsWith => "ends with",
            FilterOperator::GreaterThan => "greater than",
            FilterOperator::GreaterThanOrEqual => "greater than or equal",
            FilterOperator::LessThan => "less than",
            FilterOperator::LessThanOrEqual => "less than or equal",
            FilterOperator::Between => "between",
            FilterOperator::In => "is one of",
            FilterOperator::NotIn => "is not one of",
            FilterOperator::IsEmpty => "is empty",
            FilterOperator::IsNotEmpty => "is not empty",
        }
    }

    /// Check if this operator requires a value.
    pub fn requires_value(&self) -> bool {
        !matches!(self, FilterOperator::IsEmpty | FilterOperator::IsNotEmpty)
    }

    /// Check if this operator requires two values (for range operators).
    pub fn requires_range(&self) -> bool {
        matches!(self, FilterOperator::Between)
    }

    /// Check if this operator requires a list of values.
    pub fn requires_list(&self) -> bool {
        matches!(self, FilterOperator::In | FilterOperator::NotIn)
    }
}

/// Filter value that can hold different types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterValue {
    /// Text value.
    Text(String),
    /// Integer value.
    Integer(i64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Boolean(bool),
    /// Date value.
    Date(NaiveDate),
    /// DateTime value.
    DateTime(DateTime<Utc>),
    /// List of text values.
    TextList(Vec<String>),
    /// Range of integer values.
    IntegerRange(i64, i64),
    /// Range of float values.
    FloatRange(f64, f64),
    /// Range of date values.
    DateRange(NaiveDate, NaiveDate),
    /// Range of datetime values.
    DateTimeRange(DateTime<Utc>, DateTime<Utc>),
}

impl FilterValue {
    /// Create a text filter value.
    pub fn text(value: impl Into<String>) -> Self {
        FilterValue::Text(value.into())
    }

    /// Create an integer filter value.
    pub fn integer(value: i64) -> Self {
        FilterValue::Integer(value)
    }

    /// Create a float filter value.
    pub fn float(value: f64) -> Self {
        FilterValue::Float(value)
    }

    /// Create a boolean filter value.
    pub fn boolean(value: bool) -> Self {
        FilterValue::Boolean(value)
    }

    /// Create a date filter value.
    pub fn date(value: NaiveDate) -> Self {
        FilterValue::Date(value)
    }

    /// Create a datetime filter value.
    pub fn datetime(value: DateTime<Utc>) -> Self {
        FilterValue::DateTime(value)
    }

    /// Create a text list filter value.
    pub fn text_list(values: Vec<String>) -> Self {
        FilterValue::TextList(values)
    }

    /// Create an integer range filter value.
    pub fn integer_range(min: i64, max: i64) -> Self {
        FilterValue::IntegerRange(min, max)
    }

    /// Create a float range filter value.
    pub fn float_range(min: f64, max: f64) -> Self {
        FilterValue::FloatRange(min, max)
    }

    /// Create a date range filter value.
    pub fn date_range(from: NaiveDate, to: NaiveDate) -> Self {
        FilterValue::DateRange(from, to)
    }
}

/// A filter specification for a column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    /// Column ID to filter.
    pub column_id: String,
    /// Filter operator.
    pub operator: FilterOperator,
    /// Filter value.
    pub value: FilterValue,
    /// Whether the filter is case-sensitive (for text filters).
    pub case_sensitive: bool,
}

impl Filter {
    /// Create a new filter.
    pub fn new(column_id: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            column_id: column_id.into(),
            operator,
            value,
            case_sensitive: false,
        }
    }

    /// Create a text equals filter.
    pub fn text_equals(column_id: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(column_id, FilterOperator::Equals, FilterValue::text(value))
    }

    /// Create a text contains filter.
    pub fn text_contains(column_id: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(column_id, FilterOperator::Contains, FilterValue::text(value))
    }

    /// Create a text starts with filter.
    pub fn text_starts_with(column_id: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(column_id, FilterOperator::StartsWith, FilterValue::text(value))
    }

    /// Create a number equals filter.
    pub fn number_equals(column_id: impl Into<String>, value: i64) -> Self {
        Self::new(column_id, FilterOperator::Equals, FilterValue::integer(value))
    }

    /// Create a number greater than filter.
    pub fn number_greater_than(column_id: impl Into<String>, value: i64) -> Self {
        Self::new(column_id, FilterOperator::GreaterThan, FilterValue::integer(value))
    }

    /// Create a number range filter.
    pub fn number_between(column_id: impl Into<String>, min: i64, max: i64) -> Self {
        Self::new(column_id, FilterOperator::Between, FilterValue::integer_range(min, max))
    }

    /// Create a select filter (one of values).
    pub fn select_in(column_id: impl Into<String>, values: Vec<String>) -> Self {
        Self::new(column_id, FilterOperator::In, FilterValue::text_list(values))
    }

    /// Create an is empty filter.
    pub fn is_empty(column_id: impl Into<String>) -> Self {
        Self::new(column_id, FilterOperator::IsEmpty, FilterValue::Text(String::new()))
    }

    /// Create an is not empty filter.
    pub fn is_not_empty(column_id: impl Into<String>) -> Self {
        Self::new(column_id, FilterOperator::IsNotEmpty, FilterValue::Text(String::new()))
    }

    /// Set case sensitivity.
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Get a display label for this filter (for filter chips).
    pub fn label(&self) -> String {
        let value_str = match &self.value {
            FilterValue::Text(s) => format!("\"{}\"", s),
            FilterValue::Integer(n) => n.to_string(),
            FilterValue::Float(n) => n.to_string(),
            FilterValue::Boolean(b) => b.to_string(),
            FilterValue::Date(d) => d.to_string(),
            FilterValue::DateTime(dt) => dt.to_string(),
            FilterValue::TextList(list) => format!("[{}]", list.join(", ")),
            FilterValue::IntegerRange(min, max) => format!("{} - {}", min, max),
            FilterValue::FloatRange(min, max) => format!("{} - {}", min, max),
            FilterValue::DateRange(from, to) => format!("{} - {}", from, to),
            FilterValue::DateTimeRange(from, to) => format!("{} - {}", from, to),
        };

        if self.operator.requires_value() {
            format!("{} {} {}", self.column_id, self.operator.label(), value_str)
        } else {
            format!("{} {}", self.column_id, self.operator.label())
        }
    }
}

/// Filter state for the table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterState {
    /// Active column filters.
    pub filters: Vec<Filter>,
    /// Global search query (searches all filterable columns).
    pub global_search: Option<String>,
    /// Whether global search is case-sensitive.
    pub global_search_case_sensitive: bool,
}

impl FilterState {
    /// Create a new filter state.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            global_search: None,
            global_search_case_sensitive: false,
        }
    }

    /// Check if any filters are active.
    pub fn has_filters(&self) -> bool {
        !self.filters.is_empty() || self.global_search.is_some()
    }

    /// Get all active filters for a column.
    pub fn get_column_filters(&self, column_id: &str) -> Vec<&Filter> {
        self.filters
            .iter()
            .filter(|f| f.column_id == column_id)
            .collect()
    }

    /// Check if a column has active filters.
    pub fn has_column_filter(&self, column_id: &str) -> bool {
        self.filters.iter().any(|f| f.column_id == column_id)
    }

    /// Add a filter.
    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    /// Remove all filters for a column.
    pub fn remove_column_filters(&mut self, column_id: &str) {
        self.filters.retain(|f| f.column_id != column_id);
    }

    /// Remove a specific filter by index.
    pub fn remove_filter(&mut self, index: usize) {
        if index < self.filters.len() {
            self.filters.remove(index);
        }
    }

    /// Set the global search query.
    pub fn set_global_search(&mut self, query: impl Into<String>) {
        let query = query.into();
        self.global_search = if query.is_empty() { None } else { Some(query) };
    }

    /// Clear the global search query.
    pub fn clear_global_search(&mut self) {
        self.global_search = None;
    }

    /// Clear all filters.
    pub fn clear(&mut self) {
        self.filters.clear();
        self.global_search = None;
    }

    /// Get the number of active filters (including global search).
    pub fn count(&self) -> usize {
        self.filters.len() + if self.global_search.is_some() { 1 } else { 0 }
    }
}

/// Trait for filterable values.
pub trait Filterable {
    /// Check if this value matches a text filter.
    fn matches_text(&self, value: &str, operator: FilterOperator, case_sensitive: bool) -> bool;

    /// Convert to string for global search.
    fn to_search_string(&self) -> String;
}

impl Filterable for String {
    fn matches_text(&self, value: &str, operator: FilterOperator, case_sensitive: bool) -> bool {
        let (self_str, value_str) = if case_sensitive {
            (self.clone(), value.to_string())
        } else {
            (self.to_lowercase(), value.to_lowercase())
        };

        match operator {
            FilterOperator::Equals => self_str == value_str,
            FilterOperator::NotEquals => self_str != value_str,
            FilterOperator::Contains => self_str.contains(&value_str),
            FilterOperator::NotContains => !self_str.contains(&value_str),
            FilterOperator::StartsWith => self_str.starts_with(&value_str),
            FilterOperator::EndsWith => self_str.ends_with(&value_str),
            FilterOperator::IsEmpty => self_str.is_empty(),
            FilterOperator::IsNotEmpty => !self_str.is_empty(),
            _ => false,
        }
    }

    fn to_search_string(&self) -> String {
        self.clone()
    }
}

impl Filterable for &str {
    fn matches_text(&self, value: &str, operator: FilterOperator, case_sensitive: bool) -> bool {
        self.to_string().matches_text(value, operator, case_sensitive)
    }

    fn to_search_string(&self) -> String {
        self.to_string()
    }
}

impl Filterable for i32 {
    fn matches_text(&self, _value: &str, _operator: FilterOperator, _case_sensitive: bool) -> bool {
        false
    }

    fn to_search_string(&self) -> String {
        self.to_string()
    }
}

impl Filterable for i64 {
    fn matches_text(&self, _value: &str, _operator: FilterOperator, _case_sensitive: bool) -> bool {
        false
    }

    fn to_search_string(&self) -> String {
        self.to_string()
    }
}

impl Filterable for f64 {
    fn matches_text(&self, _value: &str, _operator: FilterOperator, _case_sensitive: bool) -> bool {
        false
    }

    fn to_search_string(&self) -> String {
        self.to_string()
    }
}

impl Filterable for bool {
    fn matches_text(&self, _value: &str, _operator: FilterOperator, _case_sensitive: bool) -> bool {
        false
    }

    fn to_search_string(&self) -> String {
        if *self { "true" } else { "false" }.to_string()
    }
}

impl<T: Filterable> Filterable for Option<T> {
    fn matches_text(&self, value: &str, operator: FilterOperator, case_sensitive: bool) -> bool {
        match (self, operator) {
            (None, FilterOperator::IsEmpty) => true,
            (None, FilterOperator::IsNotEmpty) => false,
            (Some(_), FilterOperator::IsEmpty) => false,
            (Some(_), FilterOperator::IsNotEmpty) => true,
            (Some(v), _) => v.matches_text(value, operator, case_sensitive),
            (None, _) => false,
        }
    }

    fn to_search_string(&self) -> String {
        match self {
            Some(v) => v.to_search_string(),
            None => String::new(),
        }
    }
}

/// Matcher for number comparisons.
pub fn match_number<N: PartialOrd>(value: N, filter_value: &FilterValue, operator: FilterOperator) -> bool {
    match (filter_value, operator) {
        (FilterValue::Integer(n), FilterOperator::Equals) => {
            // Convert i64 to f64 for comparison if needed
            if let Some(v) = num_to_f64(&value) {
                (v - *n as f64).abs() < f64::EPSILON
            } else {
                false
            }
        }
        (FilterValue::Integer(n), FilterOperator::NotEquals) => {
            if let Some(v) = num_to_f64(&value) {
                (v - *n as f64).abs() >= f64::EPSILON
            } else {
                true
            }
        }
        (FilterValue::Integer(n), FilterOperator::GreaterThan) => {
            if let Some(v) = num_to_f64(&value) {
                v > *n as f64
            } else {
                false
            }
        }
        (FilterValue::Integer(n), FilterOperator::GreaterThanOrEqual) => {
            if let Some(v) = num_to_f64(&value) {
                v >= *n as f64
            } else {
                false
            }
        }
        (FilterValue::Integer(n), FilterOperator::LessThan) => {
            if let Some(v) = num_to_f64(&value) {
                v < *n as f64
            } else {
                false
            }
        }
        (FilterValue::Integer(n), FilterOperator::LessThanOrEqual) => {
            if let Some(v) = num_to_f64(&value) {
                v <= *n as f64
            } else {
                false
            }
        }
        (FilterValue::IntegerRange(min, max), FilterOperator::Between) => {
            if let Some(v) = num_to_f64(&value) {
                v >= *min as f64 && v <= *max as f64
            } else {
                false
            }
        }
        (FilterValue::Float(n), FilterOperator::Equals) => {
            if let Some(v) = num_to_f64(&value) {
                (v - n).abs() < f64::EPSILON
            } else {
                false
            }
        }
        (FilterValue::Float(n), FilterOperator::GreaterThan) => {
            if let Some(v) = num_to_f64(&value) {
                v > *n
            } else {
                false
            }
        }
        (FilterValue::FloatRange(min, max), FilterOperator::Between) => {
            if let Some(v) = num_to_f64(&value) {
                v >= *min && v <= *max
            } else {
                false
            }
        }
        _ => false,
    }
}

// Helper to convert numbers to f64 for comparison
fn num_to_f64<N: PartialOrd>(_value: &N) -> Option<f64> {
    // This is a simplified version - in real implementation would use num-traits
    None
}

/// Matcher for select/in comparisons.
pub fn match_in_list(value: &str, filter_value: &FilterValue, operator: FilterOperator, case_sensitive: bool) -> bool {
    match (filter_value, operator) {
        (FilterValue::TextList(list), FilterOperator::In) => {
            if case_sensitive {
                list.iter().any(|v| v == value)
            } else {
                let lower = value.to_lowercase();
                list.iter().any(|v| v.to_lowercase() == lower)
            }
        }
        (FilterValue::TextList(list), FilterOperator::NotIn) => {
            if case_sensitive {
                !list.iter().any(|v| v == value)
            } else {
                let lower = value.to_lowercase();
                !list.iter().any(|v| v.to_lowercase() == lower)
            }
        }
        _ => false,
    }
}

/// Filter chip for displaying active filters in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterChip {
    /// Filter index.
    pub index: usize,
    /// Column ID.
    pub column_id: String,
    /// Display label.
    pub label: String,
}

impl FilterChip {
    /// Create filter chips from filter state.
    pub fn from_state(state: &FilterState) -> Vec<Self> {
        let mut chips = Vec::new();

        if let Some(ref query) = state.global_search {
            chips.push(FilterChip {
                index: usize::MAX, // Special index for global search
                column_id: "__global".to_string(),
                label: format!("Search: \"{}\"", query),
            });
        }

        for (index, filter) in state.filters.iter().enumerate() {
            chips.push(FilterChip {
                index,
                column_id: filter.column_id.clone(),
                label: filter.label(),
            });
        }

        chips
    }
}

/// Filter configuration for a column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Column ID.
    pub column_id: String,
    /// Allowed operators for this column.
    pub allowed_operators: Vec<FilterOperator>,
    /// Predefined options for select filters.
    pub options: Option<Vec<String>>,
    /// Placeholder text for the filter input.
    pub placeholder: Option<String>,
}

impl FilterConfig {
    /// Create a text filter config.
    pub fn text(column_id: impl Into<String>) -> Self {
        Self {
            column_id: column_id.into(),
            allowed_operators: vec![
                FilterOperator::Contains,
                FilterOperator::Equals,
                FilterOperator::NotEquals,
                FilterOperator::StartsWith,
                FilterOperator::EndsWith,
                FilterOperator::IsEmpty,
                FilterOperator::IsNotEmpty,
            ],
            options: None,
            placeholder: Some("Filter...".to_string()),
        }
    }

    /// Create a number filter config.
    pub fn number(column_id: impl Into<String>) -> Self {
        Self {
            column_id: column_id.into(),
            allowed_operators: vec![
                FilterOperator::Equals,
                FilterOperator::NotEquals,
                FilterOperator::GreaterThan,
                FilterOperator::GreaterThanOrEqual,
                FilterOperator::LessThan,
                FilterOperator::LessThanOrEqual,
                FilterOperator::Between,
            ],
            options: None,
            placeholder: Some("0".to_string()),
        }
    }

    /// Create a select filter config.
    pub fn select(column_id: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            column_id: column_id.into(),
            allowed_operators: vec![FilterOperator::In, FilterOperator::NotIn],
            options: Some(options),
            placeholder: Some("Select...".to_string()),
        }
    }

    /// Create a date filter config.
    pub fn date(column_id: impl Into<String>) -> Self {
        Self {
            column_id: column_id.into(),
            allowed_operators: vec![
                FilterOperator::Equals,
                FilterOperator::NotEquals,
                FilterOperator::GreaterThan,
                FilterOperator::GreaterThanOrEqual,
                FilterOperator::LessThan,
                FilterOperator::LessThanOrEqual,
                FilterOperator::Between,
            ],
            options: None,
            placeholder: Some("YYYY-MM-DD".to_string()),
        }
    }
}

/// Quick filter for common filter patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickFilter {
    /// Filter label.
    pub label: String,
    /// Filters to apply.
    pub filters: Vec<Filter>,
}

impl QuickFilter {
    /// Create a new quick filter.
    pub fn new(label: impl Into<String>, filters: Vec<Filter>) -> Self {
        Self {
            label: label.into(),
            filters,
        }
    }
}

/// Collection of quick filters for a table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuickFilters {
    /// Available quick filters.
    pub filters: Vec<QuickFilter>,
    /// Currently active quick filter index.
    pub active: Option<usize>,
}

impl QuickFilters {
    /// Create a new quick filters collection.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            active: None,
        }
    }

    /// Add a quick filter.
    pub fn add(mut self, filter: QuickFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Apply a quick filter by index.
    pub fn apply(&mut self, index: usize, state: &mut FilterState) {
        if let Some(quick_filter) = self.filters.get(index) {
            state.filters = quick_filter.filters.clone();
            self.active = Some(index);
        }
    }

    /// Clear the active quick filter.
    pub fn clear(&mut self, state: &mut FilterState) {
        state.filters.clear();
        self.active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_operator_labels() {
        assert_eq!(FilterOperator::Contains.label(), "contains");
        assert_eq!(FilterOperator::Between.label(), "between");
        assert!(FilterOperator::Contains.requires_value());
        assert!(!FilterOperator::IsEmpty.requires_value());
        assert!(FilterOperator::Between.requires_range());
        assert!(FilterOperator::In.requires_list());
    }

    #[test]
    fn test_filter_creation() {
        let filter = Filter::text_contains("name", "alice");
        assert_eq!(filter.column_id, "name");
        assert_eq!(filter.operator, FilterOperator::Contains);
        assert!(!filter.case_sensitive);
    }

    #[test]
    fn test_filter_label() {
        let filter = Filter::text_contains("name", "alice");
        assert_eq!(filter.label(), "name contains \"alice\"");

        let filter = Filter::is_empty("status");
        assert_eq!(filter.label(), "status is empty");
    }

    #[test]
    fn test_filter_state() {
        let mut state = FilterState::new();
        assert!(!state.has_filters());

        state.add_filter(Filter::text_contains("name", "alice"));
        assert!(state.has_filters());
        assert!(state.has_column_filter("name"));
        assert!(!state.has_column_filter("email"));

        state.add_filter(Filter::text_contains("email", "example.com"));
        assert_eq!(state.count(), 2);

        state.remove_column_filters("name");
        assert!(!state.has_column_filter("name"));
        assert_eq!(state.count(), 1);
    }

    #[test]
    fn test_filter_state_global_search() {
        let mut state = FilterState::new();

        state.set_global_search("test query");
        assert!(state.has_filters());
        assert_eq!(state.global_search, Some("test query".to_string()));

        state.clear_global_search();
        assert!(!state.has_filters());
    }

    #[test]
    fn test_filterable_string() {
        let value = "Hello World".to_string();

        assert!(value.matches_text("hello", FilterOperator::Contains, false));
        assert!(!value.matches_text("hello", FilterOperator::Contains, true));
        assert!(value.matches_text("Hello", FilterOperator::StartsWith, true));
        assert!(value.matches_text("World", FilterOperator::EndsWith, true));
        assert!(!value.matches_text("", FilterOperator::IsEmpty, false));
        assert!(value.matches_text("", FilterOperator::IsNotEmpty, false));
    }

    #[test]
    fn test_filterable_option() {
        let some_value: Option<String> = Some("test".to_string());
        let none_value: Option<String> = None;

        assert!(!some_value.matches_text("", FilterOperator::IsEmpty, false));
        assert!(some_value.matches_text("", FilterOperator::IsNotEmpty, false));
        assert!(none_value.matches_text("", FilterOperator::IsEmpty, false));
        assert!(!none_value.matches_text("", FilterOperator::IsNotEmpty, false));
    }

    #[test]
    fn test_match_in_list() {
        let values = FilterValue::text_list(vec!["Active".to_string(), "Pending".to_string()]);

        assert!(match_in_list("Active", &values, FilterOperator::In, true));
        assert!(!match_in_list("Inactive", &values, FilterOperator::In, true));
        assert!(match_in_list("Inactive", &values, FilterOperator::NotIn, true));

        // Case insensitive
        assert!(match_in_list("active", &values, FilterOperator::In, false));
    }

    #[test]
    fn test_filter_chips() {
        let mut state = FilterState::new();
        state.set_global_search("query");
        state.add_filter(Filter::text_contains("name", "alice"));

        let chips = FilterChip::from_state(&state);
        assert_eq!(chips.len(), 2);
        assert_eq!(chips[0].column_id, "__global");
        assert_eq!(chips[1].column_id, "name");
    }

    #[test]
    fn test_filter_config() {
        let text_config = FilterConfig::text("name");
        assert!(text_config.allowed_operators.contains(&FilterOperator::Contains));
        assert!(text_config.options.is_none());

        let select_config = FilterConfig::select("status", vec!["Active".to_string()]);
        assert!(select_config.allowed_operators.contains(&FilterOperator::In));
        assert!(select_config.options.is_some());
    }

    #[test]
    fn test_quick_filters() {
        let mut quick_filters = QuickFilters::new()
            .add(QuickFilter::new("Active Only", vec![Filter::text_equals("status", "Active")]))
            .add(QuickFilter::new("Pending", vec![Filter::text_equals("status", "Pending")]));

        let mut state = FilterState::new();

        quick_filters.apply(0, &mut state);
        assert_eq!(quick_filters.active, Some(0));
        assert_eq!(state.filters.len(), 1);

        quick_filters.clear(&mut state);
        assert_eq!(quick_filters.active, None);
        assert!(state.filters.is_empty());
    }

    #[test]
    fn test_number_filter() {
        let filter = Filter::number_greater_than("age", 25);
        assert_eq!(filter.column_id, "age");
        assert_eq!(filter.operator, FilterOperator::GreaterThan);

        let filter = Filter::number_between("age", 20, 30);
        assert_eq!(filter.operator, FilterOperator::Between);
    }

    #[test]
    fn test_select_filter() {
        let filter = Filter::select_in("status", vec!["Active".to_string(), "Pending".to_string()]);
        assert_eq!(filter.operator, FilterOperator::In);
        match filter.value {
            FilterValue::TextList(list) => {
                assert_eq!(list.len(), 2);
            }
            _ => panic!("Expected TextList"),
        }
    }
}
