//! DateRangePicker Component
//!
//! A component for selecting a range of dates with preset options and two-calendar view.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::locale::Locale;
use crate::utils::{
    add_months, calendar_grid, end_of_month, first_day_of_month, last_day_of_month, start_of_month,
    WeekStart,
};

/// A date range with start and end dates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date of the range (inclusive)
    pub start: NaiveDate,
    /// End date of the range (inclusive)
    pub end: NaiveDate,
}

impl DateRange {
    /// Create a new date range
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        // Ensure start <= end
        if start <= end {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
    }

    /// Get the number of days in the range (inclusive)
    pub fn days(&self) -> i64 {
        (self.end - self.start).num_days() + 1
    }

    /// Check if a date is within this range
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &DateRange) -> bool {
        self.start <= other.end && self.end >= other.start
    }

    /// Get all dates in this range
    pub fn dates(&self) -> Vec<NaiveDate> {
        let mut dates = Vec::with_capacity(self.days() as usize);
        let mut current = self.start;
        while current <= self.end {
            dates.push(current);
            current = current + Duration::days(1);
        }
        dates
    }
}

/// Preset date range options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateRangePreset {
    /// Today only
    Today,
    /// Yesterday only
    Yesterday,
    /// Last 7 days including today
    Last7Days,
    /// Last 30 days including today
    Last30Days,
    /// Current month (1st to today)
    ThisMonth,
    /// Previous month (entire month)
    LastMonth,
    /// Last 3 months
    Last3Months,
    /// Last 6 months
    Last6Months,
    /// This year (Jan 1 to today)
    ThisYear,
    /// Last year (entire year)
    LastYear,
    /// Custom range (user-selected)
    Custom,
}

impl DateRangePreset {
    /// Get all available presets
    pub fn all() -> &'static [DateRangePreset] {
        &[
            DateRangePreset::Today,
            DateRangePreset::Yesterday,
            DateRangePreset::Last7Days,
            DateRangePreset::Last30Days,
            DateRangePreset::ThisMonth,
            DateRangePreset::LastMonth,
            DateRangePreset::Last3Months,
            DateRangePreset::Last6Months,
            DateRangePreset::ThisYear,
            DateRangePreset::LastYear,
            DateRangePreset::Custom,
        ]
    }

    /// Calculate the date range for this preset
    pub fn to_range(&self, today: NaiveDate) -> Option<DateRange> {
        match self {
            DateRangePreset::Today => Some(DateRange::new(today, today)),
            DateRangePreset::Yesterday => {
                let yesterday = today - Duration::days(1);
                Some(DateRange::new(yesterday, yesterday))
            }
            DateRangePreset::Last7Days => {
                let start = today - Duration::days(6);
                Some(DateRange::new(start, today))
            }
            DateRangePreset::Last30Days => {
                let start = today - Duration::days(29);
                Some(DateRange::new(start, today))
            }
            DateRangePreset::ThisMonth => {
                let start = first_day_of_month(today.year(), today.month())?;
                Some(DateRange::new(start, today))
            }
            DateRangePreset::LastMonth => {
                let last_month = add_months(today, -1)?;
                let start = first_day_of_month(last_month.year(), last_month.month())?;
                let end = last_day_of_month(last_month.year(), last_month.month())?;
                Some(DateRange::new(start, end))
            }
            DateRangePreset::Last3Months => {
                let three_months_ago = add_months(today, -2)?;
                let start = first_day_of_month(three_months_ago.year(), three_months_ago.month())?;
                Some(DateRange::new(start, today))
            }
            DateRangePreset::Last6Months => {
                let six_months_ago = add_months(today, -5)?;
                let start = first_day_of_month(six_months_ago.year(), six_months_ago.month())?;
                Some(DateRange::new(start, today))
            }
            DateRangePreset::ThisYear => {
                let start = NaiveDate::from_ymd_opt(today.year(), 1, 1)?;
                Some(DateRange::new(start, today))
            }
            DateRangePreset::LastYear => {
                let start = NaiveDate::from_ymd_opt(today.year() - 1, 1, 1)?;
                let end = NaiveDate::from_ymd_opt(today.year() - 1, 12, 31)?;
                Some(DateRange::new(start, end))
            }
            DateRangePreset::Custom => None,
        }
    }

    /// Get the label for this preset
    pub fn label(&self, locale: Locale) -> &'static str {
        let labels = locale.labels();
        match self {
            DateRangePreset::Today => labels.preset_today,
            DateRangePreset::Yesterday => labels.preset_yesterday,
            DateRangePreset::Last7Days => labels.preset_last_7_days,
            DateRangePreset::Last30Days => labels.preset_last_30_days,
            DateRangePreset::ThisMonth => labels.preset_this_month,
            DateRangePreset::LastMonth => labels.preset_last_month,
            DateRangePreset::Last3Months => "Last 3 months",
            DateRangePreset::Last6Months => "Last 6 months",
            DateRangePreset::ThisYear => "This year",
            DateRangePreset::LastYear => "Last year",
            DateRangePreset::Custom => labels.preset_custom,
        }
    }
}

/// Configuration for the DateRangePicker component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangePickerConfig {
    /// Initial selected range (optional)
    pub selected_range: Option<DateRange>,
    /// Minimum selectable date
    pub min_date: Option<NaiveDate>,
    /// Maximum selectable date
    pub max_date: Option<NaiveDate>,
    /// Available preset options
    pub presets: Vec<DateRangePreset>,
    /// First day of the week
    pub week_start: WeekStart,
    /// Locale for formatting
    pub locale: Locale,
    /// Whether to show two calendars side by side
    pub two_calendars: bool,
    /// Whether the picker is disabled
    pub disabled: bool,
    /// Maximum range length in days (0 = unlimited)
    pub max_range_days: u32,
    /// Placeholder text when no range is selected
    pub placeholder: Option<String>,
}

impl Default for DateRangePickerConfig {
    fn default() -> Self {
        Self {
            selected_range: None,
            min_date: None,
            max_date: None,
            presets: vec![
                DateRangePreset::Today,
                DateRangePreset::Yesterday,
                DateRangePreset::Last7Days,
                DateRangePreset::Last30Days,
                DateRangePreset::ThisMonth,
                DateRangePreset::LastMonth,
                DateRangePreset::Custom,
            ],
            week_start: WeekStart::default(),
            locale: Locale::default(),
            two_calendars: true,
            disabled: false,
            max_range_days: 0,
            placeholder: None,
        }
    }
}

/// Represents a single day cell in the date range calendar grid
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRangeCell {
    /// The date for this cell
    pub date: NaiveDate,
    /// Whether this date is in the current display month
    pub is_current_month: bool,
    /// Whether this date is today
    pub is_today: bool,
    /// Whether this date is the range start
    pub is_range_start: bool,
    /// Whether this date is the range end
    pub is_range_end: bool,
    /// Whether this date is within the selected range
    pub is_in_range: bool,
    /// Whether this date is being hovered (for preview)
    pub is_hover_preview: bool,
    /// Whether this date is disabled
    pub is_disabled: bool,
}

/// Selection state for range picking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeSelectionState {
    /// No selection in progress
    None,
    /// Start date selected, waiting for end date
    StartSelected(NaiveDate),
    /// Range fully selected
    Complete(DateRange),
}

/// State for the DateRangePicker component
#[derive(Debug, Clone)]
pub struct DateRangePickerState {
    /// The currently selected range
    pub selected_range: Option<DateRange>,
    /// Current selection state
    pub selection_state: RangeSelectionState,
    /// Currently selected preset
    pub selected_preset: Option<DateRangePreset>,
    /// The month currently being displayed (left calendar)
    pub display_month_left: u32,
    /// The year currently being displayed (left calendar)
    pub display_year_left: i32,
    /// Whether the picker popover is open
    pub is_open: bool,
    /// The currently hovered date (for range preview)
    pub hover_date: Option<NaiveDate>,
    /// Configuration
    config: DateRangePickerConfig,
}

impl DateRangePickerState {
    /// Create a new DateRangePickerState with the given configuration
    pub fn new(config: DateRangePickerConfig) -> Self {
        let today = chrono::Local::now().date_naive();
        let (display_year_left, display_month_left) = config
            .selected_range
            .map(|r| (r.start.year(), r.start.month()))
            .unwrap_or((today.year(), today.month()));

        let selection_state = config
            .selected_range
            .map(RangeSelectionState::Complete)
            .unwrap_or(RangeSelectionState::None);

        Self {
            selected_range: config.selected_range,
            selection_state,
            selected_preset: None,
            display_month_left,
            display_year_left,
            is_open: false,
            hover_date: None,
            config,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &DateRangePickerConfig {
        &self.config
    }

    /// Get the right calendar's display month and year
    pub fn right_calendar_month_year(&self) -> (i32, u32) {
        if self.display_month_left == 12 {
            (self.display_year_left + 1, 1)
        } else {
            (self.display_year_left, self.display_month_left + 1)
        }
    }

    /// Check if a date is disabled
    pub fn is_date_disabled(&self, date: NaiveDate) -> bool {
        // Check min date
        if let Some(min) = self.config.min_date {
            if date < min {
                return true;
            }
        }

        // Check max date
        if let Some(max) = self.config.max_date {
            if date > max {
                return true;
            }
        }

        // Check max range days if start is selected
        if self.config.max_range_days > 0 {
            if let RangeSelectionState::StartSelected(start) = self.selection_state {
                let days = (date - start).num_days().abs();
                if days >= self.config.max_range_days as i64 {
                    return true;
                }
            }
        }

        false
    }

    /// Get the calendar grid for the left calendar
    pub fn calendar_grid_left(&self) -> Vec<DateRangeCell> {
        self.calendar_grid_for_month(self.display_year_left, self.display_month_left)
    }

    /// Get the calendar grid for the right calendar
    pub fn calendar_grid_right(&self) -> Vec<DateRangeCell> {
        let (year, month) = self.right_calendar_month_year();
        self.calendar_grid_for_month(year, month)
    }

    /// Generate calendar grid for a specific month
    fn calendar_grid_for_month(&self, year: i32, month: u32) -> Vec<DateRangeCell> {
        let today = chrono::Local::now().date_naive();
        let dates = calendar_grid(year, month, self.config.week_start).unwrap_or_default();

        // Calculate the preview range (for hover effect)
        let preview_range = match self.selection_state {
            RangeSelectionState::StartSelected(start) => {
                self.hover_date.map(|end| DateRange::new(start, end))
            }
            _ => None,
        };

        dates
            .into_iter()
            .map(|date| {
                let is_disabled = self.is_date_disabled(date);
                let is_in_selected_range =
                    self.selected_range.map(|r| r.contains(date)).unwrap_or(false);
                let is_in_preview = preview_range.map(|r| r.contains(date)).unwrap_or(false);

                DateRangeCell {
                    date,
                    is_current_month: date.month() == month && date.year() == year,
                    is_today: date == today,
                    is_range_start: self
                        .selected_range
                        .map(|r| r.start == date)
                        .unwrap_or(false)
                        || matches!(self.selection_state, RangeSelectionState::StartSelected(d) if d == date),
                    is_range_end: self
                        .selected_range
                        .map(|r| r.end == date)
                        .unwrap_or(false),
                    is_in_range: is_in_selected_range || is_in_preview,
                    is_hover_preview: is_in_preview && !is_in_selected_range,
                    is_disabled,
                }
            })
            .collect()
    }

    /// Get weekday headers for the calendar
    pub fn weekday_headers(&self) -> Vec<&'static str> {
        self.config
            .locale
            .weekday_names_for_calendar(self.config.week_start)
    }

    /// Get the display title for the left calendar
    pub fn display_title_left(&self) -> String {
        let month_name = self
            .config
            .locale
            .month_name(self.display_month_left, false);
        format!("{} {}", month_name, self.display_year_left)
    }

    /// Get the display title for the right calendar
    pub fn display_title_right(&self) -> String {
        let (year, month) = self.right_calendar_month_year();
        let month_name = self.config.locale.month_name(month, false);
        format!("{} {}", month_name, year)
    }

    /// Navigate to the previous month
    pub fn previous_month(&mut self) {
        if self.display_month_left == 1 {
            self.display_month_left = 12;
            self.display_year_left -= 1;
        } else {
            self.display_month_left -= 1;
        }
    }

    /// Navigate to the next month
    pub fn next_month(&mut self) {
        if self.display_month_left == 12 {
            self.display_month_left = 1;
            self.display_year_left += 1;
        } else {
            self.display_month_left += 1;
        }
    }

    /// Handle date click
    pub fn click_date(&mut self, date: NaiveDate) -> Option<DateRangePickerEvent> {
        if self.config.disabled || self.is_date_disabled(date) {
            return None;
        }

        match self.selection_state {
            RangeSelectionState::None | RangeSelectionState::Complete(_) => {
                // Start a new selection
                self.selection_state = RangeSelectionState::StartSelected(date);
                self.selected_range = None;
                self.selected_preset = Some(DateRangePreset::Custom);
                Some(DateRangePickerEvent::SelectionStarted(date))
            }
            RangeSelectionState::StartSelected(start) => {
                // Complete the selection
                let range = DateRange::new(start, date);
                self.selection_state = RangeSelectionState::Complete(range);
                self.selected_range = Some(range);
                self.hover_date = None;
                Some(DateRangePickerEvent::RangeSelected(range))
            }
        }
    }

    /// Handle date hover
    pub fn hover_date(&mut self, date: Option<NaiveDate>) {
        self.hover_date = date;
    }

    /// Select a preset range
    pub fn select_preset(&mut self, preset: DateRangePreset) -> Option<DateRangePickerEvent> {
        if self.config.disabled {
            return None;
        }

        let today = chrono::Local::now().date_naive();
        if let Some(range) = preset.to_range(today) {
            self.selected_range = Some(range);
            self.selection_state = RangeSelectionState::Complete(range);
            self.selected_preset = Some(preset);

            // Update display to show the range
            self.display_year_left = range.start.year();
            self.display_month_left = range.start.month();

            Some(DateRangePickerEvent::RangeSelected(range))
        } else {
            // Custom preset - just clear and wait for selection
            self.selected_preset = Some(preset);
            self.selection_state = RangeSelectionState::None;
            self.selected_range = None;
            Some(DateRangePickerEvent::SelectionCleared)
        }
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        if self.config.disabled {
            return;
        }
        self.selected_range = None;
        self.selection_state = RangeSelectionState::None;
        self.selected_preset = None;
        self.hover_date = None;
    }

    /// Open the picker
    pub fn open(&mut self) {
        if self.config.disabled {
            return;
        }
        self.is_open = true;
    }

    /// Close the picker
    pub fn close(&mut self) {
        self.is_open = false;
        self.hover_date = None;

        // If selection was incomplete, clear it
        if matches!(self.selection_state, RangeSelectionState::StartSelected(_)) {
            self.selection_state = RangeSelectionState::None;
        }
    }

    /// Get the formatted display value
    pub fn display_value(&self) -> String {
        match self.selected_range {
            Some(range) => {
                let start_str = self.config.locale.format_date(range.start, true);
                let end_str = self.config.locale.format_date(range.end, true);
                format!("{} - {}", start_str, end_str)
            }
            None => self
                .config
                .placeholder
                .clone()
                .unwrap_or_else(|| "Select date range".to_string()),
        }
    }

    /// Get available presets
    pub fn presets(&self) -> &[DateRangePreset] {
        &self.config.presets
    }
}

/// Events emitted by the date range picker
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateRangePickerEvent {
    /// Picker was opened
    Opened,
    /// Picker was closed
    Closed,
    /// Start date was selected
    SelectionStarted(NaiveDate),
    /// A complete range was selected
    RangeSelected(DateRange),
    /// Selection was cleared
    SelectionCleared,
    /// Display month changed
    MonthChanged(i32, u32),
}

/// Builder for DateRangePickerConfig
#[derive(Debug, Clone, Default)]
pub struct DateRangePickerBuilder {
    config: DateRangePickerConfig,
}

impl DateRangePickerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial selected range
    pub fn selected_range(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.config.selected_range = Some(DateRange::new(start, end));
        self
    }

    /// Set the minimum selectable date
    pub fn min_date(mut self, date: NaiveDate) -> Self {
        self.config.min_date = Some(date);
        self
    }

    /// Set the maximum selectable date
    pub fn max_date(mut self, date: NaiveDate) -> Self {
        self.config.max_date = Some(date);
        self
    }

    /// Set the available presets
    pub fn presets(mut self, presets: Vec<DateRangePreset>) -> Self {
        self.config.presets = presets;
        self
    }

    /// Set the first day of the week
    pub fn week_start(mut self, start: WeekStart) -> Self {
        self.config.week_start = start;
        self
    }

    /// Set the locale
    pub fn locale(mut self, locale: Locale) -> Self {
        self.config.locale = locale;
        self.config.week_start = locale.default_week_start();
        self
    }

    /// Enable or disable two calendar view
    pub fn two_calendars(mut self, enabled: bool) -> Self {
        self.config.two_calendars = enabled;
        self
    }

    /// Set the maximum range length in days
    pub fn max_range_days(mut self, days: u32) -> Self {
        self.config.max_range_days = days;
        self
    }

    /// Set the picker as disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.config.placeholder = Some(placeholder.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> DateRangePickerConfig {
        self.config
    }

    /// Build and create the state
    pub fn build_state(self) -> DateRangePickerState {
        DateRangePickerState::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_range_creation() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let range = DateRange::new(start, end);

        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
        assert_eq!(range.days(), 11);
    }

    #[test]
    fn test_date_range_reversed() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let range = DateRange::new(start, end);

        // Should swap start and end
        assert_eq!(range.start, end);
        assert_eq!(range.end, start);
    }

    #[test]
    fn test_date_range_contains() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let range = DateRange::new(start, end);

        assert!(range.contains(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert!(range.contains(start));
        assert!(range.contains(end));
        assert!(!range.contains(NaiveDate::from_ymd_opt(2024, 1, 9).unwrap()));
        assert!(!range.contains(NaiveDate::from_ymd_opt(2024, 1, 21).unwrap()));
    }

    #[test]
    fn test_preset_today() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let range = DateRangePreset::Today.to_range(today).unwrap();

        assert_eq!(range.start, today);
        assert_eq!(range.end, today);
        assert_eq!(range.days(), 1);
    }

    #[test]
    fn test_preset_last_7_days() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let range = DateRangePreset::Last7Days.to_range(today).unwrap();

        assert_eq!(range.end, today);
        assert_eq!(range.days(), 7);
    }

    #[test]
    fn test_preset_this_month() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let range = DateRangePreset::ThisMonth.to_range(today).unwrap();

        assert_eq!(range.start, NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
        assert_eq!(range.end, today);
    }

    #[test]
    fn test_preset_last_month() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let range = DateRangePreset::LastMonth.to_range(today).unwrap();

        assert_eq!(range.start, NaiveDate::from_ymd_opt(2024, 5, 1).unwrap());
        assert_eq!(range.end, NaiveDate::from_ymd_opt(2024, 5, 31).unwrap());
    }

    #[test]
    fn test_date_range_picker_creation() {
        let state = DateRangePickerBuilder::new().build_state();
        assert!(state.selected_range.is_none());
        assert!(!state.is_open);
    }

    #[test]
    fn test_date_range_picker_selection() {
        let mut state = DateRangePickerBuilder::new().build_state();

        let start = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        // Click start date
        let event = state.click_date(start);
        assert!(matches!(
            event,
            Some(DateRangePickerEvent::SelectionStarted(_))
        ));
        assert!(matches!(
            state.selection_state,
            RangeSelectionState::StartSelected(_)
        ));

        // Click end date
        let event = state.click_date(end);
        assert!(matches!(
            event,
            Some(DateRangePickerEvent::RangeSelected(_))
        ));
        assert!(state.selected_range.is_some());
        assert_eq!(state.selected_range.unwrap().start, start);
        assert_eq!(state.selected_range.unwrap().end, end);
    }

    #[test]
    fn test_date_range_picker_preset_selection() {
        let mut state = DateRangePickerBuilder::new().build_state();
        let today = chrono::Local::now().date_naive();

        let event = state.select_preset(DateRangePreset::Today);
        assert!(matches!(
            event,
            Some(DateRangePickerEvent::RangeSelected(_))
        ));
        assert_eq!(state.selected_range.unwrap().start, today);
        assert_eq!(state.selected_range.unwrap().end, today);
    }

    #[test]
    fn test_date_range_picker_navigation() {
        let mut state = DateRangePickerBuilder::new().build_state();
        let initial_month = state.display_month_left;
        let initial_year = state.display_year_left;

        state.next_month();
        if initial_month == 12 {
            assert_eq!(state.display_month_left, 1);
            assert_eq!(state.display_year_left, initial_year + 1);
        } else {
            assert_eq!(state.display_month_left, initial_month + 1);
        }

        state.previous_month();
        assert_eq!(state.display_month_left, initial_month);
        assert_eq!(state.display_year_left, initial_year);
    }

    #[test]
    fn test_date_range_picker_clear() {
        let start = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mut state = DateRangePickerBuilder::new()
            .selected_range(start, end)
            .build_state();

        assert!(state.selected_range.is_some());
        state.clear();
        assert!(state.selected_range.is_none());
    }

    #[test]
    fn test_date_range_overlaps() {
        let range1 = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
        );
        let range2 = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
        );
        let range3 = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
        );

        assert!(range1.overlaps(&range2));
        assert!(!range1.overlaps(&range3));
    }
}
