//! Formatting utilities

use chrono::{DateTime, Utc, Local};

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format duration in human-readable format
pub fn format_duration(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;

    if seconds >= DAY {
        let days = seconds / DAY;
        let hours = (seconds % DAY) / HOUR;
        format!("{}d {}h", days, hours)
    } else if seconds >= HOUR {
        let hours = seconds / HOUR;
        let minutes = (seconds % HOUR) / MINUTE;
        format!("{}h {}m", hours, minutes)
    } else if seconds >= MINUTE {
        let minutes = seconds / MINUTE;
        let secs = seconds % MINUTE;
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", seconds)
    }
}

/// Format relative time (e.g., "2 hours ago")
pub fn format_relative_time(datetime: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(datetime);

    if duration.num_seconds() < 0 {
        return "in the future".to_string();
    }

    let seconds = duration.num_seconds() as u64;

    if seconds < 60 {
        "just now".to_string()
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", minutes)
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else if seconds < 604800 {
        let days = seconds / 86400;
        if days == 1 {
            "yesterday".to_string()
        } else {
            format!("{} days ago", days)
        }
    } else if seconds < 2592000 {
        let weeks = seconds / 604800;
        if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{} weeks ago", weeks)
        }
    } else if seconds < 31536000 {
        let months = seconds / 2592000;
        if months == 1 {
            "1 month ago".to_string()
        } else {
            format!("{} months ago", months)
        }
    } else {
        let years = seconds / 31536000;
        if years == 1 {
            "1 year ago".to_string()
        } else {
            format!("{} years ago", years)
        }
    }
}

/// Format date/time for display
pub fn format_datetime(datetime: DateTime<Utc>) -> String {
    let local: DateTime<Local> = DateTime::from(datetime);
    local.format("%Y-%m-%d %H:%M").to_string()
}

/// Format date only
pub fn format_date(datetime: DateTime<Utc>) -> String {
    let local: DateTime<Local> = DateTime::from(datetime);
    local.format("%Y-%m-%d").to_string()
}

/// Format time only
pub fn format_time(datetime: DateTime<Utc>) -> String {
    let local: DateTime<Local> = DateTime::from(datetime);
    local.format("%H:%M:%S").to_string()
}

/// Truncate text with ellipsis
pub fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else if max_len <= 3 {
        text.chars().take(max_len).collect()
    } else {
        let truncated: String = text.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

/// Format number with thousands separator
pub fn format_number(num: u64) -> String {
    let s = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

/// Pluralize a word based on count
pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}

/// Generate initials from a name
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90061), "1d 1h");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello", 10), "Hello");
        assert_eq!(truncate("Hello World", 8), "Hello...");
        assert_eq!(truncate("Hi", 2), "Hi");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_initials() {
        assert_eq!(initials("John Doe"), "JD");
        assert_eq!(initials("Alice"), "A");
        assert_eq!(initials("John Michael Doe"), "JM");
    }
}
