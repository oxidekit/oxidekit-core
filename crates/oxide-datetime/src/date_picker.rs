//! DatePicker Component
//!
//! A calendar-based date picker with month navigation, today button, and customizable constraints.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::locale::Locale;
use crate::utils::{calendar_grid, WeekStart};

/// Configuration for the DatePicker component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatePickerConfig {
    /// Initial selected date (optional)
    pub selected_date: Option<NaiveDate>,
    /// Minimum selectable date
    pub min_date: Option<NaiveDate>,
    /// Maximum selectable date
    pub max_date: Option<NaiveDate>,
    /// Dates that should be disabled
    pub disabled_dates: Vec<NaiveDate>,
    /// First day of the week
    pub week_start: WeekStart,
    /// Locale for formatting
    pub locale: Locale,
    /// Whether to show the today button
    pub show_today_button: bool,
    /// Whether to show the clear button
    pub show_clear_button: bool,
    /// Whether to close picker on date selection
    pub close_on_select: bool,
    /// Whether the picker is disabled
    pub disabled: bool,
    /// Whether the picker is read-only
    pub read_only: bool,
    /// Placeholder text when no date is selected
    pub placeholder: Option<String>,
}

impl Default for DatePickerConfig {
    fn default() -> Self {
        Self {
            selected_date: None,
            min_date: None,
            max_date: None,
            disabled_dates: Vec::new(),
            week_start: WeekStart::default(),
            locale: Locale::default(),
            show_today_button: true,
            show_clear_button: true,
            close_on_select: true,
            disabled: false,
            read_only: false,
            placeholder: None,
        }
    }
}

/// Represents a single day cell in the calendar grid
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayCell {
    /// The date for this cell
    pub date: NaiveDate,
    /// Whether this date is in the current display month
    pub is_current_month: bool,
    /// Whether this date is today
    pub is_today: bool,
    /// Whether this date is selected
    pub is_selected: bool,
    /// Whether this date is disabled
    pub is_disabled: bool,
    /// Whether this date is within the allowed range
    pub is_in_range: bool,
}

/// State for the DatePicker component
#[derive(Debug, Clone)]
pub struct DatePickerState {
    /// The currently selected date
    pub selected_date: Option<NaiveDate>,
    /// The month currently being displayed
    pub display_month: u32,
    /// The year currently being displayed
    pub display_year: i32,
    /// Whether the picker popover is open
    pub is_open: bool,
    /// The currently focused date (for keyboard navigation)
    pub focused_date: Option<NaiveDate>,
    /// Configuration
    config: DatePickerConfig,
}

impl DatePickerState {
    /// Create a new DatePickerState with the given configuration
    pub fn new(config: DatePickerConfig) -> Self {
        let today = chrono::Local::now().date_naive();
        let (display_year, display_month) = config
            .selected_date
            .map(|d| (d.year(), d.month()))
            .unwrap_or((today.year(), today.month()));

        Self {
            selected_date: config.selected_date,
            display_month,
            display_year,
            is_open: false,
            focused_date: config.selected_date,
            config,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &DatePickerConfig {
        &self.config
    }

    /// Check if a date is disabled
    pub fn is_date_disabled(&self, date: NaiveDate) -> bool {
        // Check if explicitly disabled
        if self.config.disabled_dates.contains(&date) {
            return true;
        }

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

        false
    }

    /// Get the calendar grid for the current display month
    pub fn calendar_grid(&self) -> Vec<DayCell> {
        let today = chrono::Local::now().date_naive();
        let dates = calendar_grid(self.display_year, self.display_month, self.config.week_start)
            .unwrap_or_default();

        dates
            .into_iter()
            .map(|date| {
                let is_disabled = self.is_date_disabled(date);
                DayCell {
                    date,
                    is_current_month: date.month() == self.display_month
                        && date.year() == self.display_year,
                    is_today: date == today,
                    is_selected: self.selected_date == Some(date),
                    is_disabled,
                    is_in_range: !is_disabled,
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

    /// Get the display title (e.g., "January 2024")
    pub fn display_title(&self) -> String {
        let month_name = self.config.locale.month_name(self.display_month, false);
        format!("{} {}", month_name, self.display_year)
    }

    /// Navigate to the previous month
    pub fn previous_month(&mut self) {
        if self.display_month == 1 {
            self.display_month = 12;
            self.display_year -= 1;
        } else {
            self.display_month -= 1;
        }
    }

    /// Navigate to the next month
    pub fn next_month(&mut self) {
        if self.display_month == 12 {
            self.display_month = 1;
            self.display_year += 1;
        } else {
            self.display_month += 1;
        }
    }

    /// Navigate to the previous year
    pub fn previous_year(&mut self) {
        self.display_year -= 1;
    }

    /// Navigate to the next year
    pub fn next_year(&mut self) {
        self.display_year += 1;
    }

    /// Navigate to today
    pub fn go_to_today(&mut self) {
        let today = chrono::Local::now().date_naive();
        self.display_year = today.year();
        self.display_month = today.month();
        self.focused_date = Some(today);
    }

    /// Select today's date
    pub fn select_today(&mut self) -> Option<NaiveDate> {
        let today = chrono::Local::now().date_naive();
        if !self.is_date_disabled(today) {
            self.select_date(today)
        } else {
            None
        }
    }

    /// Select a date
    pub fn select_date(&mut self, date: NaiveDate) -> Option<NaiveDate> {
        if self.config.disabled || self.config.read_only {
            return None;
        }

        if self.is_date_disabled(date) {
            return None;
        }

        self.selected_date = Some(date);
        self.focused_date = Some(date);

        if self.config.close_on_select {
            self.is_open = false;
        }

        Some(date)
    }

    /// Clear the selected date
    pub fn clear(&mut self) {
        if self.config.disabled || self.config.read_only {
            return;
        }
        self.selected_date = None;
        self.focused_date = None;
    }

    /// Open the picker
    pub fn open(&mut self) {
        if self.config.disabled {
            return;
        }
        self.is_open = true;

        // Set initial focus
        if self.focused_date.is_none() {
            self.focused_date = self.selected_date.or(Some(chrono::Local::now().date_naive()));
        }
    }

    /// Close the picker
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Toggle the picker open/closed state
    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Get the formatted display value
    pub fn display_value(&self) -> String {
        match self.selected_date {
            Some(date) => self.config.locale.format_date(date, false),
            None => self
                .config
                .placeholder
                .clone()
                .unwrap_or_else(|| self.config.locale.labels().select_date.to_string()),
        }
    }

    /// Handle keyboard navigation
    pub fn handle_key(&mut self, key: KeyAction) -> Option<DatePickerEvent> {
        if !self.is_open {
            if matches!(key, KeyAction::Enter | KeyAction::Space | KeyAction::ArrowDown) {
                self.open();
                return Some(DatePickerEvent::Opened);
            }
            return None;
        }

        match key {
            KeyAction::Escape => {
                self.close();
                Some(DatePickerEvent::Closed)
            }
            KeyAction::Enter | KeyAction::Space => {
                if let Some(focused) = self.focused_date {
                    if !self.is_date_disabled(focused) {
                        self.select_date(focused);
                        return Some(DatePickerEvent::DateSelected(focused));
                    }
                }
                None
            }
            KeyAction::ArrowLeft => {
                self.move_focus(-1, 0);
                Some(DatePickerEvent::FocusChanged(self.focused_date))
            }
            KeyAction::ArrowRight => {
                self.move_focus(1, 0);
                Some(DatePickerEvent::FocusChanged(self.focused_date))
            }
            KeyAction::ArrowUp => {
                self.move_focus(0, -7);
                Some(DatePickerEvent::FocusChanged(self.focused_date))
            }
            KeyAction::ArrowDown => {
                self.move_focus(0, 7);
                Some(DatePickerEvent::FocusChanged(self.focused_date))
            }
            KeyAction::PageUp => {
                self.previous_month();
                Some(DatePickerEvent::MonthChanged(
                    self.display_year,
                    self.display_month,
                ))
            }
            KeyAction::PageDown => {
                self.next_month();
                Some(DatePickerEvent::MonthChanged(
                    self.display_year,
                    self.display_month,
                ))
            }
            KeyAction::Home => {
                // Go to first day of month
                if let Some(first) =
                    NaiveDate::from_ymd_opt(self.display_year, self.display_month, 1)
                {
                    self.focused_date = Some(first);
                }
                Some(DatePickerEvent::FocusChanged(self.focused_date))
            }
            KeyAction::End => {
                // Go to last day of month
                if let Some(last) =
                    crate::utils::last_day_of_month(self.display_year, self.display_month)
                {
                    self.focused_date = Some(last);
                }
                Some(DatePickerEvent::FocusChanged(self.focused_date))
            }
            KeyAction::Tab => None, // Let default tab behavior happen
        }
    }

    /// Move focus by a number of days
    fn move_focus(&mut self, days: i64, extra_days: i64) {
        let current = self
            .focused_date
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        let new_date = current + chrono::Duration::days(days + extra_days);
        self.focused_date = Some(new_date);

        // Update display month if needed
        if new_date.month() != self.display_month || new_date.year() != self.display_year {
            self.display_month = new_date.month();
            self.display_year = new_date.year();
        }
    }
}

/// Keyboard actions for the date picker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Enter,
    Space,
    Escape,
    Tab,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Events emitted by the date picker
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatePickerEvent {
    /// Picker was opened
    Opened,
    /// Picker was closed
    Closed,
    /// A date was selected
    DateSelected(NaiveDate),
    /// The date was cleared
    DateCleared,
    /// Display month/year changed
    MonthChanged(i32, u32),
    /// Focus changed to a new date
    FocusChanged(Option<NaiveDate>),
}

/// Builder for DatePickerConfig
#[derive(Debug, Clone, Default)]
pub struct DatePickerBuilder {
    config: DatePickerConfig,
}

impl DatePickerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial selected date
    pub fn selected_date(mut self, date: NaiveDate) -> Self {
        self.config.selected_date = Some(date);
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

    /// Set the date range (min and max)
    pub fn date_range(mut self, min: NaiveDate, max: NaiveDate) -> Self {
        self.config.min_date = Some(min);
        self.config.max_date = Some(max);
        self
    }

    /// Add a disabled date
    pub fn disable_date(mut self, date: NaiveDate) -> Self {
        self.config.disabled_dates.push(date);
        self
    }

    /// Add multiple disabled dates
    pub fn disable_dates(mut self, dates: Vec<NaiveDate>) -> Self {
        self.config.disabled_dates.extend(dates);
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
        // Also update week start to match locale default
        self.config.week_start = locale.default_week_start();
        self
    }

    /// Show or hide the today button
    pub fn show_today_button(mut self, show: bool) -> Self {
        self.config.show_today_button = show;
        self
    }

    /// Show or hide the clear button
    pub fn show_clear_button(mut self, show: bool) -> Self {
        self.config.show_clear_button = show;
        self
    }

    /// Set whether to close on selection
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.config.close_on_select = close;
        self
    }

    /// Set the picker as disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// Set the picker as read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.config.read_only = read_only;
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.config.placeholder = Some(placeholder.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> DatePickerConfig {
        self.config
    }

    /// Build and create the state
    pub fn build_state(self) -> DatePickerState {
        DatePickerState::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_picker_creation() {
        let state = DatePickerBuilder::new().build_state();
        assert!(state.selected_date.is_none());
        assert!(!state.is_open);
    }

    #[test]
    fn test_date_picker_with_initial_date() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let state = DatePickerBuilder::new().selected_date(date).build_state();
        assert_eq!(state.selected_date, Some(date));
        assert_eq!(state.display_year, 2024);
        assert_eq!(state.display_month, 6);
    }

    #[test]
    fn test_date_picker_navigation() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let mut state = DatePickerBuilder::new().selected_date(date).build_state();

        state.previous_month();
        assert_eq!(state.display_month, 12);
        assert_eq!(state.display_year, 2023);

        state.next_month();
        assert_eq!(state.display_month, 1);
        assert_eq!(state.display_year, 2024);

        state.next_month();
        assert_eq!(state.display_month, 2);
        assert_eq!(state.display_year, 2024);
    }

    #[test]
    fn test_date_picker_select() {
        let mut state = DatePickerBuilder::new().build_state();
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        let result = state.select_date(date);
        assert_eq!(result, Some(date));
        assert_eq!(state.selected_date, Some(date));
    }

    #[test]
    fn test_date_picker_disabled_dates() {
        let disabled = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mut state = DatePickerBuilder::new().disable_date(disabled).build_state();

        assert!(state.is_date_disabled(disabled));

        let result = state.select_date(disabled);
        assert_eq!(result, None);
    }

    #[test]
    fn test_date_picker_min_max() {
        let min = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let max = NaiveDate::from_ymd_opt(2024, 6, 20).unwrap();
        let state = DatePickerBuilder::new().date_range(min, max).build_state();

        assert!(state.is_date_disabled(NaiveDate::from_ymd_opt(2024, 6, 9).unwrap()));
        assert!(!state.is_date_disabled(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()));
        assert!(state.is_date_disabled(NaiveDate::from_ymd_opt(2024, 6, 21).unwrap()));
    }

    #[test]
    fn test_date_picker_calendar_grid() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let state = DatePickerBuilder::new().selected_date(date).build_state();

        let grid = state.calendar_grid();
        assert_eq!(grid.len(), 42); // 6 weeks * 7 days

        // Find the selected date in the grid
        let selected_cell = grid.iter().find(|c| c.is_selected);
        assert!(selected_cell.is_some());
        assert_eq!(selected_cell.unwrap().date, date);
    }

    #[test]
    fn test_date_picker_clear() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mut state = DatePickerBuilder::new().selected_date(date).build_state();

        state.clear();
        assert!(state.selected_date.is_none());
    }

    #[test]
    fn test_date_picker_open_close() {
        let mut state = DatePickerBuilder::new().build_state();

        assert!(!state.is_open);

        state.open();
        assert!(state.is_open);

        state.close();
        assert!(!state.is_open);

        state.toggle();
        assert!(state.is_open);

        state.toggle();
        assert!(!state.is_open);
    }

    #[test]
    fn test_date_picker_disabled_state() {
        let mut state = DatePickerBuilder::new().disabled(true).build_state();
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        state.open();
        assert!(!state.is_open); // Should not open when disabled

        let result = state.select_date(date);
        assert_eq!(result, None);
    }

    #[test]
    fn test_date_picker_keyboard_navigation() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mut state = DatePickerBuilder::new().selected_date(date).build_state();
        state.focused_date = Some(date);
        state.is_open = true;

        // Test arrow keys
        let event = state.handle_key(KeyAction::ArrowRight);
        assert!(matches!(event, Some(DatePickerEvent::FocusChanged(_))));

        // Test escape
        let event = state.handle_key(KeyAction::Escape);
        assert!(matches!(event, Some(DatePickerEvent::Closed)));
        assert!(!state.is_open);
    }
}
