use std::rc::Rc;

use chrono::{DateTime, Duration, Local};
use cron::Schedule;
use mockall::automock;

pub struct Period {
    start: DateTime<Local>,
    duration: Duration,
    time_provider: Box<dyn TimeProvider>,
}

#[derive(Debug)]
pub enum PeriodError {
    NegativeDurationError,
    ZeroDurationError,
}

impl Period {
    pub fn starting_now(duration: Duration) -> Result<Self, PeriodError> {
        Period::starting_at(Local::now(), duration)
    }

    pub fn starting_at(start: DateTime<Local>, duration: Duration) -> Result<Self, PeriodError> {
        Period::starting_at_with_time_provider(start, duration, Box::new(RealTimeProvider))
    }

    fn starting_now_with_time_provider(
        duration: Duration,
        time_provider: Box<dyn TimeProvider>,
    ) -> Result<Self, PeriodError> {
        Period::starting_at_with_time_provider(Local::now(), duration, time_provider)
    }

    fn starting_at_with_time_provider(
        start: DateTime<Local>,
        duration: Duration,
        time_provider: Box<dyn TimeProvider>,
    ) -> Result<Self, PeriodError> {
        return if duration.is_zero() {
            Err(PeriodError::ZeroDurationError)
        } else if duration.num_seconds().is_negative()
            || duration.num_nanoseconds().unwrap_or(0).is_negative()
        {
            Err(PeriodError::NegativeDurationError)
        } else {
            Ok(Period {
                start,
                duration,
                time_provider,
            })
        };
    }

    pub fn upcoming_relative(&self) -> PeriodIterator<NextAvailableIntervalStrategy> {
        PeriodIterator::new_relative(self)
    }

    pub fn upcoming_fixed(&self) -> PeriodIterator<FixedIntervalStrategy> {
        PeriodIterator::new_fixed(self)
    }
}

pub struct PeriodIterator<'a, S: IntervalStrategy> {
    period: &'a Period,
    current: Option<DateTime<Local>>,
    next_date_strategy: S,
}

impl<'a, S: IntervalStrategy> PeriodIterator<'a, S> {
    fn new(period: &'a Period, next_date_stategy: S) -> Self {
        PeriodIterator {
            period,
            current: Some(period.start),
            next_date_strategy: next_date_stategy,
        }
    }
}

impl<'a> PeriodIterator<'a, FixedIntervalStrategy> {
    /// Create an iterator for the period, which can generate values in the past.
    fn new_fixed(period: &'a Period) -> Self {
        Self::new(period, FixedIntervalStrategy)
    }
}

impl<'a> PeriodIterator<'a, NextAvailableIntervalStrategy<'a>> {
    /// Create an iterator for the period, which will only generate values after the current timestamp.
    fn new_relative(period: &'a Period) -> Self {
        Self::new(
            period,
            NextAvailableIntervalStrategy {
                time_provider: &*period.time_provider,
            },
        )
    }
}

impl<'a, S: IntervalStrategy> Iterator for PeriodIterator<'a, S> {
    type Item = DateTime<Local>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(current) => {
                let next = self
                    .next_date_strategy
                    .next_timestamp(current, &self.period.duration);
                self.current = Some(next);
                Some(current)
            }
            None => None,
        }
    }
}

pub trait IntervalStrategy {
    /// Determine the next timestamp for the interval based on the given duration.
    fn next_timestamp(&self, current: DateTime<Local>, duration: &Duration) -> DateTime<Local>;
}

pub struct FixedIntervalStrategy;

impl IntervalStrategy for FixedIntervalStrategy {
    /// Determine the next timestamp for the interval. This can return values from the past.
    fn next_timestamp(&self, timestamp: DateTime<Local>, duration: &Duration) -> DateTime<Local> {
        timestamp + *duration
    }
}

pub struct NextAvailableIntervalStrategy<'a> {
    time_provider: &'a dyn TimeProvider,
}

impl<'a> IntervalStrategy for NextAvailableIntervalStrategy<'a> {
    /// Determine the next timestamp for the interval. The returned value is the next available value which in the future.
    fn next_timestamp(&self, current: DateTime<Local>, duration: &Duration) -> DateTime<Local> {
        #[rustfmt::skip]
        let full_durations_till_present = ((self.time_provider.now().timestamp() - current.timestamp()).max(0) as u32)
            .div_ceil(duration.num_seconds() as u32) as i64;

        current + Duration::seconds(full_durations_till_present * duration.num_seconds())
    }
}

#[automock]
pub trait TimeProvider {
    fn now(&self) -> DateTime<Local>;
}

pub struct RealTimeProvider;

impl TimeProvider for RealTimeProvider {
    fn now(&self) -> DateTime<Local> {
        Local::now()
    }
}

pub trait UpcomingDates {
    fn upcoming2(&self) -> Box<dyn Iterator<Item = DateTime<Local>> + '_>;
}

impl UpcomingDates for Period {
    fn upcoming2(&self) -> Box<dyn Iterator<Item = DateTime<Local>> + '_> {
        Box::new(self.upcoming_relative())
    }
}

impl UpcomingDates for Schedule {
    fn upcoming2(&self) -> Box<dyn Iterator<Item = DateTime<Local>> + '_> {
        Box::new(self.upcoming(Local))
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use core::time;
    use std::{iter, str::FromStr};

    use super::*;

    #[test]
    fn that_period_can_be_created_without_time_provider() {
        let now = Local::now();
        let duration = Duration::hours(1);

        let result = Period::starting_at(now, duration);

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

        let result = Period::starting_at(now, duration);

        assert!(result.is_err());
    }

    #[test]
    fn that_period_can_not_be_created_with_negative_duration() {
        let now = Local::now();
        let duration = Duration::seconds(-1);

        let result = Period::starting_at(now, duration);

        assert!(result.is_err());
    }

    #[test]
    fn that_upcoming_returns_iterator() {
        let start = Local::now();
        let duration = Duration::minutes(12);
        let period = Period::starting_at(start, duration).unwrap();

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

    // TODO make immune to local time zone
    #[test]
    fn that_duration_between_data_points_is_unaffected_by_start_of_daylight_savings() {
        let start = Local
            .with_ymd_and_hms(2025, 3, 30, 1, 0, 0)
            .single()
            .unwrap();
        let expected_result = Local
            .with_ymd_and_hms(2025, 3, 30, 3, 0, 0)
            .earliest()
            .unwrap();
        let duration = Duration::hours(1);
        let period = Period::starting_at(start, duration).unwrap();
        let mut period_iterator = PeriodIterator::new_fixed(&period);

        let next = period_iterator.next().unwrap();

        assert_eq!(next, expected_result);
    }

    #[test]
    fn that_duration_between_data_points_is_unaffected_by_end_of_daylight_savings() {
        let start = Local
            .with_ymd_and_hms(2025, 10, 26, 3, 0, 0)
            .latest()
            .unwrap();
        let expected_result = Local
            .with_ymd_and_hms(2025, 10, 26, 3, 0, 0)
            .earliest()
            .unwrap();
        let duration = Duration::hours(1);
        let period = Period::starting_at(start, duration).unwrap();
        let mut period_iterator = PeriodIterator::new_fixed(&period);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, expected_result);
    }

    #[test]
    fn that_upcoming_of_cron_schedule_returns_iterator_of_datetimes() {
        // '2100' is maximum year supported by cron-crate
        let expression = "0   30   12     1,15       May  *  2100";
        let schedule = Schedule::from_str(expression).unwrap();
        let mut dates = schedule.upcoming2();

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
