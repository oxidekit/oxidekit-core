//! Date and time utility functions
//!
//! Provides helper functions for date calculations, formatting, and manipulation.

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};

/// Represents the first day of the week
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum WeekStart {
    /// Week starts on Sunday (US convention)
    #[default]
    Sunday,
    /// Week starts on Monday (ISO 8601 / European convention)
    Monday,
}

impl WeekStart {
    /// Get the weekday for the start of week
    pub fn weekday(&self) -> Weekday {
        match self {
            WeekStart::Sunday => Weekday::Sun,
            WeekStart::Monday => Weekday::Mon,
        }
    }
}

/// Get the first day of the month
pub fn first_day_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, 1)
}

/// Get the last day of the month
pub fn last_day_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    let first = first_day_of_month(year, month)?;
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    Some(next_month - Duration::days(1))
}

/// Get the number of days in a month
pub fn days_in_month(year: i32, month: u32) -> Option<u32> {
    let last = last_day_of_month(year, month)?;
    Some(last.day())
}

/// Check if a year is a leap year
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get all dates in a month, including padding days from adjacent months
/// Returns a grid suitable for a calendar view (6 weeks x 7 days = 42 cells)
pub fn calendar_grid(year: i32, month: u32, week_start: WeekStart) -> Option<Vec<NaiveDate>> {
    let first = first_day_of_month(year, month)?;
    let last = last_day_of_month(year, month)?;

    // Calculate how many days to show from the previous month
    let first_weekday = first.weekday();
    let days_from_prev = match week_start {
        WeekStart::Sunday => first_weekday.num_days_from_sunday(),
        WeekStart::Monday => first_weekday.num_days_from_monday(),
    };

    let start_date = first - Duration::days(days_from_prev as i64);

    // Generate 42 days (6 weeks)
    let mut dates = Vec::with_capacity(42);
    for i in 0..42 {
        dates.push(start_date + Duration::days(i));
    }

    Some(dates)
}

/// Get the week number for a date (ISO week)
pub fn week_number(date: NaiveDate) -> u32 {
    date.iso_week().week()
}

/// Get dates for a specific week
pub fn week_dates(year: i32, week: u32, week_start: WeekStart) -> Option<Vec<NaiveDate>> {
    // Find the first day of the year
    let jan_first = NaiveDate::from_ymd_opt(year, 1, 1)?;

    // Find the first occurrence of week_start day in or before jan_first
    let jan_first_weekday = jan_first.weekday();
    let target_weekday = week_start.weekday();

    let days_diff = match week_start {
        WeekStart::Sunday => {
            (jan_first_weekday.num_days_from_sunday() as i64)
        }
        WeekStart::Monday => {
            (jan_first_weekday.num_days_from_monday() as i64)
        }
    };

    let first_week_start = jan_first - Duration::days(days_diff);
    let target_week_start = first_week_start + Duration::weeks((week - 1) as i64);

    let mut dates = Vec::with_capacity(7);
    for i in 0..7 {
        dates.push(target_week_start + Duration::days(i));
    }

    Some(dates)
}

/// Check if a date is today
pub fn is_today(date: NaiveDate) -> bool {
    date == chrono::Local::now().date_naive()
}

/// Check if a date is in the past
pub fn is_past(date: NaiveDate) -> bool {
    date < chrono::Local::now().date_naive()
}

/// Check if a date is in the future
pub fn is_future(date: NaiveDate) -> bool {
    date > chrono::Local::now().date_naive()
}

/// Check if a date is within a range (inclusive)
pub fn is_in_range(date: NaiveDate, start: NaiveDate, end: NaiveDate) -> bool {
    date >= start && date <= end
}

/// Check if two date ranges overlap
pub fn ranges_overlap(
    start1: NaiveDate,
    end1: NaiveDate,
    start2: NaiveDate,
    end2: NaiveDate,
) -> bool {
    start1 <= end2 && end1 >= start2
}

/// Get the difference between two dates in days
pub fn days_between(start: NaiveDate, end: NaiveDate) -> i64 {
    (end - start).num_days()
}

/// Add months to a date, handling edge cases
pub fn add_months(date: NaiveDate, months: i32) -> Option<NaiveDate> {
    let total_months = (date.year() * 12 + date.month() as i32 - 1) + months;
    let new_year = total_months / 12;
    let new_month = (total_months % 12 + 1) as u32;

    // Try the same day, if not valid, use the last day of the month
    NaiveDate::from_ymd_opt(new_year, new_month, date.day())
        .or_else(|| last_day_of_month(new_year, new_month))
}

/// Add years to a date, handling leap year edge cases
pub fn add_years(date: NaiveDate, years: i32) -> Option<NaiveDate> {
    let new_year = date.year() + years;
    NaiveDate::from_ymd_opt(new_year, date.month(), date.day())
        .or_else(|| NaiveDate::from_ymd_opt(new_year, date.month(), 28))
}

/// Get the start of the day for a datetime
pub fn start_of_day(datetime: NaiveDateTime) -> NaiveDateTime {
    datetime.date().and_hms_opt(0, 0, 0).unwrap()
}

/// Get the end of the day for a datetime
pub fn end_of_day(datetime: NaiveDateTime) -> NaiveDateTime {
    datetime.date().and_hms_nano_opt(23, 59, 59, 999_999_999).unwrap()
}

/// Get the start of the month for a date
pub fn start_of_month(date: NaiveDate) -> Option<NaiveDate> {
    first_day_of_month(date.year(), date.month())
}

/// Get the end of the month for a date
pub fn end_of_month(date: NaiveDate) -> Option<NaiveDate> {
    last_day_of_month(date.year(), date.month())
}

/// Get the start of the year for a date
pub fn start_of_year(date: NaiveDate) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(date.year(), 1, 1)
}

/// Get the end of the year for a date
pub fn end_of_year(date: NaiveDate) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(date.year(), 12, 31)
}

/// Format time with configurable format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TimeFormat {
    /// 12-hour format with AM/PM (e.g., "2:30 PM")
    #[default]
    TwelveHour,
    /// 24-hour format (e.g., "14:30")
    TwentyFourHour,
}

/// Parse a time string in various formats
pub fn parse_time(input: &str) -> Option<NaiveTime> {
    // Try 24-hour format first
    if let Ok(time) = NaiveTime::parse_from_str(input, "%H:%M") {
        return Some(time);
    }
    if let Ok(time) = NaiveTime::parse_from_str(input, "%H:%M:%S") {
        return Some(time);
    }

    // Try 12-hour format
    let input_upper = input.to_uppercase();
    if let Ok(time) = NaiveTime::parse_from_str(&input_upper, "%I:%M %p") {
        return Some(time);
    }
    if let Ok(time) = NaiveTime::parse_from_str(&input_upper, "%I:%M:%S %p") {
        return Some(time);
    }
    if let Ok(time) = NaiveTime::parse_from_str(&input_upper, "%I:%M%p") {
        return Some(time);
    }

    None
}

/// Format a time according to the specified format
pub fn format_time(time: NaiveTime, format: TimeFormat) -> String {
    match format {
        TimeFormat::TwelveHour => time.format("%-I:%M %p").to_string(),
        TimeFormat::TwentyFourHour => time.format("%H:%M").to_string(),
    }
}

/// Round time to the nearest step (in minutes)
pub fn round_time_to_step(time: NaiveTime, step_minutes: u32) -> NaiveTime {
    let total_minutes = time.hour() * 60 + time.minute();
    let rounded = ((total_minutes as f64 / step_minutes as f64).round() * step_minutes as f64) as u32;
    let hours = (rounded / 60) % 24;
    let minutes = rounded % 60;
    NaiveTime::from_hms_opt(hours, minutes, 0).unwrap_or(time)
}

/// Generate time options for a time picker
pub fn generate_time_options(step_minutes: u32, format: TimeFormat) -> Vec<(NaiveTime, String)> {
    let mut options = Vec::new();
    let mut current = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(23, 59, 59).unwrap();
    let step = Duration::minutes(step_minutes as i64);

    while current <= end {
        let label = format_time(current, format);
        options.push((current, label));
        match current.overflowing_add_signed(step) {
            (new_time, 0) if new_time > current => current = new_time,
            _ => break,
        }
    }

    options
}

/// Get ISO 8601 formatted date string
pub fn to_iso_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Get ISO 8601 formatted datetime string
pub fn to_iso_datetime(datetime: NaiveDateTime) -> String {
    datetime.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Parse ISO 8601 date string
pub fn from_iso_date(input: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").ok()
}

/// Parse ISO 8601 datetime string
pub fn from_iso_datetime(input: &str) -> Option<NaiveDateTime> {
    // Try with time zone designator
    if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%SZ") {
        return Some(dt);
    }
    // Try with milliseconds
    if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(dt);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_day_of_month() {
        let date = first_day_of_month(2024, 2).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    }

    #[test]
    fn test_last_day_of_month() {
        // February in leap year
        let date = last_day_of_month(2024, 2).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());

        // February in non-leap year
        let date = last_day_of_month(2023, 2).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());

        // December
        let date = last_day_of_month(2024, 12).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 2), Some(29));
        assert_eq!(days_in_month(2023, 2), Some(28));
        assert_eq!(days_in_month(2024, 1), Some(31));
        assert_eq!(days_in_month(2024, 4), Some(30));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn test_calendar_grid() {
        let grid = calendar_grid(2024, 1, WeekStart::Sunday).unwrap();
        assert_eq!(grid.len(), 42);
        // January 2024 starts on Monday, so we need Dec 31, 2023 as the first cell
        // Actually with Sunday start, we need Sunday Dec 31, 2023
        assert_eq!(grid[0], NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
    }

    #[test]
    fn test_add_months() {
        // Normal case
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(
            add_months(date, 1),
            Some(NaiveDate::from_ymd_opt(2024, 2, 15).unwrap())
        );

        // Edge case: Jan 31 + 1 month = Feb 29 (leap year)
        let date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        assert_eq!(
            add_months(date, 1),
            Some(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap())
        );

        // Crossing year boundary
        let date = NaiveDate::from_ymd_opt(2024, 11, 15).unwrap();
        assert_eq!(
            add_months(date, 3),
            Some(NaiveDate::from_ymd_opt(2025, 2, 15).unwrap())
        );

        // Negative months
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        assert_eq!(
            add_months(date, -1),
            Some(NaiveDate::from_ymd_opt(2024, 2, 15).unwrap())
        );
    }

    #[test]
    fn test_parse_time() {
        // 24-hour format
        assert_eq!(
            parse_time("14:30"),
            Some(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
        );
        assert_eq!(
            parse_time("09:05:30"),
            Some(NaiveTime::from_hms_opt(9, 5, 30).unwrap())
        );

        // 12-hour format
        assert_eq!(
            parse_time("2:30 PM"),
            Some(NaiveTime::from_hms_opt(14, 30, 0).unwrap())
        );
        assert_eq!(
            parse_time("12:00 AM"),
            Some(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        );
    }

    #[test]
    fn test_format_time() {
        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
        assert_eq!(format_time(time, TimeFormat::TwentyFourHour), "14:30");
        assert_eq!(format_time(time, TimeFormat::TwelveHour), "2:30 PM");
    }

    #[test]
    fn test_round_time_to_step() {
        let time = NaiveTime::from_hms_opt(14, 32, 0).unwrap();
        assert_eq!(
            round_time_to_step(time, 15),
            NaiveTime::from_hms_opt(14, 30, 0).unwrap()
        );
        assert_eq!(
            round_time_to_step(time, 30),
            NaiveTime::from_hms_opt(14, 30, 0).unwrap()
        );

        let time = NaiveTime::from_hms_opt(14, 38, 0).unwrap();
        assert_eq!(
            round_time_to_step(time, 15),
            NaiveTime::from_hms_opt(14, 45, 0).unwrap()
        );
    }

    #[test]
    fn test_generate_time_options() {
        let options = generate_time_options(60, TimeFormat::TwentyFourHour);
        assert_eq!(options.len(), 24);
        assert_eq!(options[0].1, "00:00");
        assert_eq!(options[23].1, "23:00");
    }

    #[test]
    fn test_iso_formatting() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(to_iso_date(date), "2024-01-15");
        assert_eq!(from_iso_date("2024-01-15"), Some(date));

        let datetime = NaiveDateTime::new(date, NaiveTime::from_hms_opt(14, 30, 0).unwrap());
        assert_eq!(to_iso_datetime(datetime), "2024-01-15T14:30:00");
        assert_eq!(from_iso_datetime("2024-01-15T14:30:00"), Some(datetime));
    }

    #[test]
    fn test_is_in_range() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        assert!(is_in_range(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), start, end));
        assert!(is_in_range(start, start, end));
        assert!(is_in_range(end, start, end));
        assert!(!is_in_range(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(), start, end));
    }

    #[test]
    fn test_days_between() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        assert_eq!(days_between(start, end), 30);
    }
}
