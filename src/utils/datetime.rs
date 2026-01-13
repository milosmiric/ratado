//! Date and time utility functions.
//!
//! This module provides functions for formatting dates in a human-readable way
//! and for performing date comparisons. All dates are handled in UTC internally
//! but displayed in the local timezone.

use chrono::{DateTime, Duration, Local, Utc};

#[cfg(test)]
use chrono::TimeZone;

/// Formats a date relative to today.
///
/// Returns a human-readable string like "Today", "Tomorrow", "Yesterday",
/// or a formatted date for dates further away.
///
/// # Arguments
///
/// * `date` - The UTC datetime to format
///
/// # Returns
///
/// A string representation of the date:
/// - "Today" for the current day
/// - "Tomorrow" for the next day
/// - "Yesterday" for the previous day
/// - "Mon 15" for dates within the next week
/// - "Jan 15" for dates further away
///
/// # Examples
///
/// ```
/// use ratado::utils::format_relative_date;
/// use chrono::Utc;
///
/// let today = format_relative_date(Utc::now());
/// assert_eq!(today, "Today");
/// ```
pub fn format_relative_date(date: DateTime<Utc>) -> String {
    let local_date = date.with_timezone(&Local).date_naive();
    let today = Local::now().date_naive();
    let tomorrow = today + Duration::days(1);
    let yesterday = today - Duration::days(1);

    if local_date == today {
        "Today".to_string()
    } else if local_date == tomorrow {
        "Tomorrow".to_string()
    } else if local_date == yesterday {
        "Yesterday".to_string()
    } else if local_date > today && local_date <= today + Duration::days(7) {
        // Within next week, show day name and date
        local_date.format("%a %d").to_string()
    } else {
        // Further out, show month and day
        local_date.format("%b %d").to_string()
    }
}

/// Formats an optional due date.
///
/// Convenience wrapper around [`format_relative_date`] that handles `Option`.
///
/// # Arguments
///
/// * `date` - Optional UTC datetime to format
///
/// # Returns
///
/// The formatted date string, or an empty string if `date` is `None`
///
/// # Examples
///
/// ```
/// use ratado::utils::format_due_date;
/// use chrono::Utc;
///
/// assert_eq!(format_due_date(Some(Utc::now())), "Today");
/// assert_eq!(format_due_date(None), "");
/// ```
pub fn format_due_date(date: Option<DateTime<Utc>>) -> String {
    match date {
        Some(d) => format_relative_date(d),
        None => String::new(),
    }
}

/// Checks if two datetimes fall on the same calendar day.
///
/// Comparison is done in the local timezone.
///
/// # Arguments
///
/// * `a` - First datetime
/// * `b` - Second datetime
///
/// # Returns
///
/// `true` if both datetimes are on the same calendar day
///
/// # Examples
///
/// ```
/// use ratado::utils::is_same_day;
/// use chrono::{Utc, Duration, TimeZone};
///
/// // Use a fixed time to avoid test failures at day boundaries
/// let base = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
/// let same_day = base + Duration::hours(1);
/// assert!(is_same_day(base, same_day));
/// ```
pub fn is_same_day(a: DateTime<Utc>, b: DateTime<Utc>) -> bool {
    let a_local = a.with_timezone(&Local).date_naive();
    let b_local = b.with_timezone(&Local).date_naive();
    a_local == b_local
}

/// Checks if a datetime is today.
///
/// # Arguments
///
/// * `date` - The datetime to check
///
/// # Returns
///
/// `true` if the datetime falls on today's date in the local timezone
///
/// # Examples
///
/// ```
/// use ratado::utils::is_today;
/// use chrono::Utc;
///
/// assert!(is_today(Utc::now()));
/// ```
pub fn is_today(date: DateTime<Utc>) -> bool {
    let local_date = date.with_timezone(&Local).date_naive();
    let today = Local::now().date_naive();
    local_date == today
}

/// Checks if a datetime is within the next 7 days.
///
/// # Arguments
///
/// * `date` - The datetime to check
///
/// # Returns
///
/// `true` if the datetime is between now and 7 days from now
///
/// # Examples
///
/// ```
/// use ratado::utils::is_this_week;
/// use chrono::{Utc, Duration};
///
/// let in_3_days = Utc::now() + Duration::days(3);
/// assert!(is_this_week(in_3_days));
///
/// let in_10_days = Utc::now() + Duration::days(10);
/// assert!(!is_this_week(in_10_days));
/// ```
pub fn is_this_week(date: DateTime<Utc>) -> bool {
    let now = Utc::now();
    let week_from_now = now + Duration::days(7);
    date >= now && date <= week_from_now
}

/// Calculates the number of days until a date.
///
/// # Arguments
///
/// * `date` - The target datetime
///
/// # Returns
///
/// Number of days until the date:
/// - Positive for future dates
/// - Negative for past dates
/// - Zero for today
///
/// # Examples
///
/// ```
/// use ratado::utils::days_until;
/// use chrono::{Utc, Duration};
///
/// let tomorrow = Utc::now() + Duration::days(1);
/// assert!(days_until(tomorrow) >= 0 && days_until(tomorrow) <= 2);
///
/// let yesterday = Utc::now() - Duration::days(1);
/// assert!(days_until(yesterday) >= -2 && days_until(yesterday) <= 0);
/// ```
pub fn days_until(date: DateTime<Utc>) -> i64 {
    let local_date = date.with_timezone(&Local).date_naive();
    let today = Local::now().date_naive();
    (local_date - today).num_days()
}

/// Returns the current UTC time.
///
/// This is a wrapper around `Utc::now()` that can be useful for testing
/// or when you need a consistent way to get the current time throughout
/// the application.
///
/// # Returns
///
/// The current UTC datetime
///
/// # Examples
///
/// ```
/// use ratado::utils::now;
///
/// let current = now();
/// // current is the current UTC time
/// ```
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_relative_date_today() {
        let today = Utc::now();
        assert_eq!(format_relative_date(today), "Today");
    }

    #[test]
    fn test_format_relative_date_tomorrow() {
        let tomorrow = Utc::now() + Duration::days(1);
        assert_eq!(format_relative_date(tomorrow), "Tomorrow");
    }

    #[test]
    fn test_format_relative_date_yesterday() {
        let yesterday = Utc::now() - Duration::days(1);
        assert_eq!(format_relative_date(yesterday), "Yesterday");
    }

    #[test]
    fn test_format_due_date_none() {
        assert_eq!(format_due_date(None), "");
    }

    #[test]
    fn test_format_due_date_some() {
        let today = Utc::now();
        assert_eq!(format_due_date(Some(today)), "Today");
    }

    #[test]
    fn test_is_same_day() {
        // Use a fixed time to avoid flaky tests at day boundaries
        let base = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let same_day = base + Duration::hours(1);
        let next_day = base + Duration::days(1);

        assert!(is_same_day(base, same_day));
        assert!(!is_same_day(base, next_day));
    }

    #[test]
    fn test_is_today() {
        let now = Utc::now();
        let tomorrow = now + Duration::days(1);

        assert!(is_today(now));
        assert!(!is_today(tomorrow));
    }

    #[test]
    fn test_is_this_week() {
        let now = Utc::now();
        let in_3_days = now + Duration::days(3);
        let in_10_days = now + Duration::days(10);
        let yesterday = now - Duration::days(1);

        assert!(is_this_week(in_3_days));
        assert!(!is_this_week(in_10_days));
        assert!(!is_this_week(yesterday));
    }

    #[test]
    fn test_days_until_positive() {
        let future = Utc::now() + Duration::days(5);
        let days = days_until(future);
        // Allow for timezone edge cases
        assert!(days >= 4 && days <= 6);
    }

    #[test]
    fn test_days_until_negative() {
        let past = Utc::now() - Duration::days(3);
        let days = days_until(past);
        // Allow for timezone edge cases
        assert!(days >= -4 && days <= -2);
    }

    #[test]
    fn test_days_until_today() {
        let today = Utc::now();
        let days = days_until(today);
        assert_eq!(days, 0);
    }
}
