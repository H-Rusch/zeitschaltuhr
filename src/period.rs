use chrono::{DateTime, Duration, TimeZone, Utc};
use cron::Schedule;
use mockall::automock;

pub struct Period<T>
where
    T: TimeZone,
{
    start: DateTime<T>,
    duration: Duration,
    time_provider: Box<dyn TimeProvider<T>>,
}

#[derive(Debug)]
pub enum PeriodError {
    NegativeDurationError,
    ZeroDurationError,
}

impl<T> Period<T>
where
    T: TimeZone + 'static,
{
    pub fn starting_now(duration: Duration, timezone: T) -> Result<Self, PeriodError> {
        Period::starting_at(now_as_timezone(&timezone), duration, timezone)
    }

    pub fn starting_at(
        start: DateTime<T>,
        duration: Duration,
        timezone: T,
    ) -> Result<Self, PeriodError> {
        Period::starting_at_with_time_provider(
            start,
            duration,
            Box::new(RealTimeProvider { timezone }),
        )
    }

    fn starting_at_with_time_provider(
        start: DateTime<T>,
        duration: Duration,
        time_provider: Box<dyn TimeProvider<T>>,
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

    pub fn upcoming_relative(&self) -> PeriodIterator<NextAvailableIntervalStrategy<T>, T> {
        PeriodIterator::new_relative(self)
    }

    pub fn upcoming_fixed(&self) -> PeriodIterator<FixedIntervalStrategy, T> {
        PeriodIterator::new_fixed(self)
    }
}

pub struct PeriodIterator<'a, S, T>
where
    S: IntervalStrategy<T>,
    T: TimeZone,
{
    period: &'a Period<T>,
    current: Option<DateTime<T>>,
    next_date_strategy: S,
}

impl<'a, S, T> PeriodIterator<'a, S, T>
where
    S: IntervalStrategy<T>,
    T: TimeZone,
{
    fn new(period: &'a Period<T>, next_date_stategy: S) -> Self {
        let start = period.start.clone();
        PeriodIterator {
            period,
            current: Some(start),
            next_date_strategy: next_date_stategy,
        }
    }
}

impl<'a, T> PeriodIterator<'a, FixedIntervalStrategy, T>
where
    T: TimeZone,
{
    /// Create an iterator for the period, which can generate values in the past.
    fn new_fixed(period: &'a Period<T>) -> Self {
        Self::new(period, FixedIntervalStrategy)
    }
}

impl<'a, T> PeriodIterator<'a, NextAvailableIntervalStrategy<'a, T>, T>
where
    T: TimeZone,
{
    /// Create an iterator for the period, which will only generate values after the current timestamp.
    fn new_relative(period: &'a Period<T>) -> Self {
        Self::new(
            period,
            NextAvailableIntervalStrategy {
                time_provider: &*period.time_provider,
            },
        )
    }
}

impl<'a, S, T> Iterator for PeriodIterator<'a, S, T>
where
    S: IntervalStrategy<T>,
    T: TimeZone,
{
    type Item = DateTime<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.clone() {
            Some(current) => {
                let next = self
                    .next_date_strategy
                    .next_timestamp(current.clone(), &self.period.duration);
                self.current = Some(next);
                Some(current)
            }
            None => None,
        }
    }
}

pub trait IntervalStrategy<T>
where
    T: TimeZone,
{
    /// Determine the next timestamp for the interval based on the given duration.
    fn next_timestamp(&self, current: DateTime<T>, duration: &Duration) -> DateTime<T>;
}

pub struct FixedIntervalStrategy;

impl<T> IntervalStrategy<T> for FixedIntervalStrategy
where
    T: TimeZone,
{
    /// Determine the next timestamp for the interval. This can return values from the past.
    fn next_timestamp(&self, timestamp: DateTime<T>, duration: &Duration) -> DateTime<T> {
        timestamp + *duration
    }
}

pub struct NextAvailableIntervalStrategy<'a, T>
where
    T: TimeZone,
{
    time_provider: &'a dyn TimeProvider<T>,
}

impl<'a, T> IntervalStrategy<T> for NextAvailableIntervalStrategy<'a, T>
where
    T: TimeZone,
{
    /// Determine the next timestamp for the interval. The returned value is the next available value which in the future.
    fn next_timestamp(&self, current: DateTime<T>, duration: &Duration) -> DateTime<T> {
        #[rustfmt::skip]
        let full_durations_till_present = ((self.time_provider.now().timestamp() - current.timestamp()).max(0) as u32)
            .div_ceil(duration.num_seconds() as u32) as i64;

        current + Duration::seconds(full_durations_till_present * duration.num_seconds())
    }
}

#[automock]
pub trait TimeProvider<T>
where
    T: TimeZone,
{
    fn now(&self) -> DateTime<T>;
}

pub struct RealTimeProvider<T>
where
    T: TimeZone,
{
    timezone: T,
}

impl<T> TimeProvider<T> for RealTimeProvider<T>
where
    T: TimeZone,
{
    fn now(&self) -> DateTime<T> {
        now_as_timezone(&self.timezone)
    }
}

fn now_as_timezone<T>(timezone: &T) -> DateTime<T>
where
    T: TimeZone,
{
    timezone.from_utc_datetime(&Utc::now().naive_utc())
}

// TODO do something with this
pub trait UpcomingDates<T>
where
    T: TimeZone + 'static,
{
    fn upcoming2(&self, timezone: T) -> Box<dyn Iterator<Item = DateTime<T>> + '_>;
}

impl<T> UpcomingDates<T> for Period<T>
where
    T: TimeZone + 'static,
{
    fn upcoming2(&self, _: T) -> Box<dyn Iterator<Item = DateTime<T>> + '_> {
        Box::new(self.upcoming_relative())
    }
}

impl<T> UpcomingDates<T> for Schedule
where
    T: TimeZone + 'static,
{
    fn upcoming2(&self, timezone: T) -> Box<dyn Iterator<Item = DateTime<T>> + '_> {
        Box::new(self.upcoming(timezone))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};
    use chrono_tz::Europe::Berlin;
    use std::str::FromStr;

    use super::*;

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
