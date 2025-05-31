use chrono::{Datelike, Days, Duration, TimeZone, Timelike, Utc};
use chrono_tz::Europe::Berlin;

use crate::period::*;

#[test]
fn that_period_can_be_created_with_non_utc_timezone() {
    let start = Berlin
        .with_ymd_and_hms(2020, 4, 15, 2, 0, 0)
        .single()
        .unwrap();
    let duration = Duration::seconds(15);
    let expected_utc_start = Utc.with_ymd_and_hms(2020, 4, 15, 0, 0, 0).single().unwrap();

    let period = Period::starting_at(start, duration);

    assert!(period.is_ok());
    assert_eq!(start, expected_utc_start);
    assert_eq!(expected_utc_start, period.unwrap().start);
}

#[test]
fn that_period_can_not_be_created_with_zero_duration() {
    let now = Utc::now();
    let duration = Duration::seconds(0);

    let result = Period::starting_at(now, duration);

    assert!(result.is_err());
}

#[test]
fn that_period_can_not_be_created_with_negative_duration() {
    let now = Utc::now();
    let duration = Duration::seconds(-1);

    let result = Period::starting_at(now, duration);

    assert!(result.is_err());
}

#[test]
fn that_upcoming_fixed_returns_iterator() {
    let start = Utc::now();
    let duration = Duration::minutes(12);
    let period = Period::starting_at(start, duration).unwrap();

    let mut period_iterator = period.upcoming_fixed();

    assert_eq!(period.duration, duration); // iterator did not take ownership
    assert!(period_iterator.next().is_some());
    assert!(period_iterator.next().is_some());
}

#[test]
fn that_upcoming_relative_returns_iterator() {
    let start = Utc::now();
    let duration = Duration::minutes(12);
    let period = Period::starting_at(start, duration).unwrap();

    let mut period_iterator = period.upcoming_relative();

    assert_eq!(period.duration, duration); // iterator did not take ownership
    assert!(period_iterator.next().is_some());
    assert!(period_iterator.next().is_some());
}

#[test]
fn that_upcoming_fixed_owned_returns_iterator() {
    let start = Utc::now();
    let duration = Duration::minutes(12);
    let period = Period::starting_at(start, duration).unwrap();

    let mut period_iterator = period.upcoming_fixed_owned();

    assert!(period_iterator.next().is_some());
    assert!(period_iterator.next().is_some());
}

#[test]
fn that_upcoming_relative_owned_returns_iterator() {
    let start = Utc::now();
    let duration = Duration::minutes(12);
    let period = Period::starting_at(start, duration).unwrap();

    let mut period_iterator = period.upcoming_relative_owned();

    assert!(period_iterator.next().is_some());
    assert!(period_iterator.next().is_some());
}

#[test]
fn that_relative_iterator_adjusts_initial_value_to_be_in_the_future_when_start_is_in_the_past() {
    let timestamp = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let duration = Duration::days(1);
    let period = Period::starting_at(timestamp, duration).unwrap();
    let iterator = period.upcoming_relative_owned();
    let now = Utc::now();

    let result = iterator.current.unwrap();

    assert!(result > now);
    assert_eq!(
        result.day(),
        now.checked_add_days(Days::new(1)).unwrap().day()
    );
    assert_eq!(result.hour(), 0);
    assert_eq!(result.minute(), 0);
    assert_eq!(result.second(), 0);
}

#[test]
fn that_relative_iterator_adjusts_initial_value_to_be_in_the_future_when_start_is_now() {
    let now = Utc::now();
    let duration = Duration::days(1);
    let period = Period::starting_at(now, duration).unwrap();
    let iterator = period.upcoming_relative_owned();

    let current = iterator.current.unwrap();

    assert_eq!(now + duration, current);
}

#[test]
fn that_relative_iterator_does_not_adjust_initial_value_when_start_is_in_the_future() {
    let timestamp = Utc::now().checked_add_days(Days::new(10)).unwrap();
    let duration = Duration::days(1);
    let period = Period::starting_at(timestamp, duration).unwrap();
    let iterator = period.upcoming_relative_owned();

    let current = iterator.current.unwrap();

    assert_eq!(timestamp, current);
}

#[test]
fn that_next_of_period_iterator_returns_increasing_timestamp_starting_with_start() {
    let start = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let duration = Duration::minutes(10);
    let period = Period::starting_at(start, duration).unwrap();
    let mut period_iterator = PeriodIterator::new_fixed(&period);

    let next = period_iterator.next().unwrap();
    assert_eq!(next, start);

    let next = period_iterator.next().unwrap();
    assert_eq!(next, start + duration);

    let next = period_iterator.next().unwrap();
    assert_eq!(next, start + duration + duration);
}

#[test]
fn that_next_of_owned_period_iterator_returns_increasing_timestamp_starting_with_start() {
    let start = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let duration = Duration::minutes(10);
    let period = Period::starting_at(start, duration).unwrap();
    let mut period_iterator = OwnedPeriodIterator::new_fixed(period);

    let next = period_iterator.next().unwrap();
    assert_eq!(next, start);

    let next = period_iterator.next().unwrap();
    assert_eq!(next, start + duration);

    let next = period_iterator.next().unwrap();
    assert_eq!(next, start + duration + duration);
}

#[test]
fn that_fixed_iterator_can_return_timestamps_in_the_past() {
    let timestamp = Utc.with_ymd_and_hms(1990, 1, 1, 0, 0, 0).single().unwrap();
    let duration = Duration::hours(1);
    let expected_result = Utc.with_ymd_and_hms(1990, 1, 1, 1, 0, 0).single().unwrap();
    let period = Period::starting_at(timestamp, duration).unwrap();
    let mut iterator = period.upcoming_fixed();

    let next = iterator.next().unwrap();
    assert_eq!(next, timestamp);

    let next = iterator.next().unwrap();
    assert_eq!(next, expected_result);
}

#[test]
fn that_duration_between_data_points_is_unaffected_by_start_of_daylight_savings() {
    let timezone = Berlin;
    let timestamp = timezone.with_ymd_and_hms(2025, 3, 30, 1, 0, 0).unwrap();
    let expected_result = timezone.with_ymd_and_hms(2025, 3, 30, 3, 0, 0).unwrap();
    let duration = Duration::hours(1);
    let period = Period::starting_at(timestamp.with_timezone(&Utc), duration).unwrap();
    let mut iterator = period.upcoming_fixed();

    let next = iterator.next().unwrap();
    assert_eq!(next, timestamp);

    let next = iterator.next().unwrap();
    assert_eq!(next, expected_result);
}

#[test]
fn that_duration_between_data_points_is_unaffected_by_end_of_daylight_savings() {
    let timezone = Berlin;
    let timestamp = timezone
        .with_ymd_and_hms(2025, 10, 26, 2, 0, 0)
        .earliest()
        .unwrap();
    let expected_result = timezone
        .with_ymd_and_hms(2025, 10, 26, 2, 0, 0)
        .latest()
        .unwrap();
    let duration = Duration::hours(1);
    let period = Period::starting_at(timestamp.with_timezone(&Utc), duration).unwrap();
    let mut iterator = period.upcoming_fixed();

    let next = iterator.next().unwrap();
    assert_eq!(next, timestamp);

    let next = iterator.next().unwrap();
    assert_eq!(next, expected_result);
}

#[test]
fn that_next_available_timestamp_returns_value_in_the_future_when_timestamp_is_now() {
    let timestamp = Utc::now();
    let duration = Duration::seconds(20);

    let result = next_available_timestamp(timestamp, &duration).unwrap();

    assert_eq!(result, timestamp + duration);
}

#[test]
fn that_next_available_timestamp_returns_adjusted_value_in_the_future_when_timestamp_is_in_the_past(
) {
    let timestamp = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
    let duration = Duration::days(1);

    let result = next_available_timestamp(timestamp, &duration).unwrap();

    assert!(result > timestamp);
    assert_eq!(
        result.day(),
        Utc::now().checked_add_days(Days::new(1)).unwrap().day()
    );
    assert_eq!(result.hour(), 0);
    assert_eq!(result.minute(), 0);
    assert_eq!(result.second(), 0);
}

#[test]
fn that_next_available_timestamp_returns_timestamp_in_the_future_when_timestamp_lies_in_the_future()
{
    let timestamp = Utc::now().checked_add_days(Days::new(10)).unwrap();
    let duration = Duration::days(1);

    let result = next_available_timestamp(timestamp, &duration).unwrap();

    assert!(result == timestamp);
}
