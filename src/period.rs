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
        if duration.is_zero() {
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
        }
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
    fn new(period: &'a Period<T>, next_date_strategy: S) -> Self {
        let start = period.start.clone();
        PeriodIterator {
            period,
            current: Some(start),
            next_date_strategy,
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

impl<S, T> Iterator for PeriodIterator<'_, S, T>
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
    /// Determine the next timestamp for the interval. This can return values which lie in the past.
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

impl<T> IntervalStrategy<T> for NextAvailableIntervalStrategy<'_, T>
where
    T: TimeZone,
{
    /// Determine the next timestamp for the interval. The returned value is the next available value which lies in the future.
    fn next_timestamp(&self, current: DateTime<T>, duration: &Duration) -> DateTime<T> {
        #[rustfmt::skip]
        // how often does the duration fit into the time period between "now" and "current"?
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
#[path = "./period/tests.rs"]
mod tests;
