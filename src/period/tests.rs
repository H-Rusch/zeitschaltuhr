pub mod tests {
    use chrono::{Duration, Local, TimeZone};
    use chrono_tz::Europe::Berlin;
    use cron::Schedule;
    use std::str::FromStr;

    use crate::period::*;

    #[test]
    fn that_period_can_be_created_without_time_provider() {
        let now = Local::now();
        let duration = Duration::hours(1);

        let result = Period::starting_at(now, duration, Local);

        assert!(result.is_ok());
        let period = result.unwrap();
        assert_eq!(period.duration, duration);
        assert_eq!(period.start, now);
    }

    #[test]
    fn that_period_can_be_created_with_time_provider() {
        let now = Local::now();
        let duration = Duration::hours(1);
        let time_provider = MockTimeProvider::new();

        let result = Period::starting_at_with_time_provider(now, duration, Box::new(time_provider));

        assert!(result.is_ok());
        let period = result.unwrap();
        assert_eq!(period.duration, duration);
        assert_eq!(period.start, now);
    }

    #[test]
    fn that_period_can_not_be_created_with_zero_duration() {
        let now = Local::now();
        let duration = Duration::seconds(0);

        let result = Period::starting_at(now, duration, Local);

        assert!(result.is_err());
    }

    #[test]
    fn that_period_can_not_be_created_with_negative_duration() {
        let now = Local::now();
        let duration = Duration::seconds(-1);

        let result = Period::starting_at(now, duration, Local);

        assert!(result.is_err());
    }

    #[test]
    fn that_upcoming_returns_iterator() {
        let start = Local::now();
        let duration = Duration::minutes(12);
        let period = Period::starting_at(start, duration, Local).unwrap();

        let mut period_iterator = period.upcoming_fixed();

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration);
    }

    #[test]
    fn that_next_returns_increasing_timestamp() {
        let start = Local::now();
        let duration = Duration::minutes(12);
        let period = Period::starting_at(start, duration, Local).unwrap();
        let mut period_iterator = PeriodIterator::new_fixed(&period);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration + duration);
    }

    #[test]
    fn that_next_available_returns_values_starting_from_current_timestamp() {
        let fake_timestamp = Local
            .with_ymd_and_hms(2020, 2, 1, 0, 0, 0)
            .single()
            .unwrap();
        let mut time_provider = MockTimeProvider::new();
        time_provider.expect_now().returning(move || fake_timestamp);
        let strategy = NextAvailableIntervalStrategy {
            time_provider: &time_provider,
        };

        let current = Local
            .with_ymd_and_hms(2020, 1, 1, 0, 0, 0)
            .single()
            .unwrap();
        let duration = Duration::minutes(7);

        let expected_result = Local
            .with_ymd_and_hms(2020, 2, 1, 0, 6, 0)
            .single()
            .unwrap();

        let result = strategy.next_timestamp(current, &duration);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn that_fixed_interval_returns_increasing_timestamp() {
        let strategy = FixedIntervalStrategy;
        let timestamp = Local::now();
        let duration = Duration::minutes(12);

        let result = strategy.next_timestamp(timestamp, &duration);

        assert_eq!(result, timestamp + duration);
    }

    #[test]
    fn that_fixed_interval_can_return_timestamps_in_the_past() {
        let strategy = FixedIntervalStrategy;
        let timestamp = Local
            .with_ymd_and_hms(1990, 1, 1, 0, 0, 0)
            .single()
            .unwrap();
        let duration = Duration::hours(1);
        let expected_result = Local
            .with_ymd_and_hms(1990, 1, 1, 1, 0, 0)
            .single()
            .unwrap();

        let result = strategy.next_timestamp(timestamp, &duration);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn that_duration_between_data_points_is_unaffected_by_start_of_daylight_savings() {
        let timezone = Berlin;
        let start = timezone.with_ymd_and_hms(2025, 3, 30, 1, 0, 0).unwrap();
        let expected_result = timezone.with_ymd_and_hms(2025, 3, 30, 3, 0, 0).unwrap();
        let duration = Duration::hours(1);
        let period = Period::starting_at(start, duration, timezone).unwrap();
        let mut period_iterator = PeriodIterator::new_fixed(&period);
        period_iterator.next().unwrap(); // first value is same as start

        let result = period_iterator.next().unwrap();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn that_duration_between_data_points_is_unaffected_by_end_of_daylight_savings() {
        let timezone = Berlin;
        let start = timezone
            .with_ymd_and_hms(2025, 10, 26, 2, 0, 0)
            .earliest()
            .unwrap();
        let expected_result = timezone
            .with_ymd_and_hms(2025, 10, 26, 2, 0, 0)
            .latest()
            .unwrap();
        let duration = Duration::hours(1);
        let period = Period::starting_at(start, duration, timezone).unwrap();
        let mut period_iterator = PeriodIterator::new_fixed(&period);
        period_iterator.next().unwrap(); // first value is same as start

        let result = period_iterator.next().unwrap();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn that_upcoming_of_cron_schedule_returns_iterator_of_datetimes() {
        // '2100' is maximum year supported by cron-crate
        let expression = "0   30   12     1,15       May  *  2100";
        let schedule = Schedule::from_str(expression).unwrap();
        let mut dates = schedule.upcoming2(Local);

        assert_eq!(
            dates.next(),
            Local.with_ymd_and_hms(2100, 5, 1, 12, 30, 0).single()
        );
        assert_eq!(
            dates.next(),
            Local.with_ymd_and_hms(2100, 5, 15, 12, 30, 0).single()
        );
        assert_eq!(dates.next(), None);
    }
}
