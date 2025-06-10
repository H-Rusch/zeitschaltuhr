use chrono::{DateTime, Duration, TimeZone, Utc};
use std::cmp::Ordering;

#[derive(Clone)]
pub struct Period {
    start: DateTime<Utc>,
    duration: Duration,
}

#[derive(Debug)]
pub enum PeriodError {
    NegativeDurationError,
    ZeroDurationError,
}

impl Period {
    /// Create a Period where the starting timestamp and the duration are adjusted to the nearest second
    /// Fails if the duration is zero or negative.
    pub fn starting_at<T: TimeZone>(
        start: DateTime<T>,
        duration: Duration,
    ) -> Result<Self, PeriodError> {
        let start = start.to_utc();
        let duration = adjust_duration(duration);

        // TODO dont support sub second accuracy
        if duration.is_zero() {
            Err(PeriodError::ZeroDurationError)
        } else if duration.num_seconds().is_negative()
            || duration.num_nanoseconds().unwrap_or(0).is_negative()
        {
            Err(PeriodError::NegativeDurationError)
        } else {
            Ok(Period { start, duration })
        }
    }

    pub fn upcoming_relative(&self) -> PeriodIterator {
        PeriodIterator::new_relative(self)
    }

    pub fn upcoming_fixed(&self) -> PeriodIterator {
        PeriodIterator::new_fixed(self)
    }

    /// Return an iterator of DateTimes that takes ownership of the Period. That iterator will only generate values in the future.
    pub fn upcoming_relative_owned(self) -> OwnedPeriodIterator {
        OwnedPeriodIterator::new_relative(self)
    }

    /// Return an iterator of DateTimes that takes ownership of the Period. The iterator can generate values in the past.
    pub fn upcoming_fixed_owned(self) -> OwnedPeriodIterator {
        OwnedPeriodIterator::new_fixed(self)
    }
}

pub struct PeriodIterator<'a> {
    period: &'a Period,
    current: Option<DateTime<Utc>>,
}

impl<'a> PeriodIterator<'a> {
    fn new(period: &'a Period, start: DateTime<Utc>) -> Self {
        PeriodIterator {
            period,
            current: Some(start),
        }
    }

    /// Create an iterator for the period, which can generate values in the past.
    fn new_fixed(period: &'a Period) -> Self {
        Self::new(period, period.start)
    }

    /// Create an iterator for the period, which will only generate values after the current timestamp.
    fn new_relative(period: &'a Period) -> Self {
        let start = next_available_timestamp(period.start, &period.duration).unwrap();
        Self::new(period, start)
    }
}

impl Iterator for PeriodIterator<'_> {
    type Item = DateTime<Utc>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().inspect(|current| {
            self.current = Some(*current + self.period.duration);
        })
    }
}

pub struct OwnedPeriodIterator {
    period: Period,
    current: Option<DateTime<Utc>>,
}

impl OwnedPeriodIterator {
    fn new(period: Period, start: DateTime<Utc>) -> Self {
        OwnedPeriodIterator {
            period,
            current: Some(start),
        }
    }

    /// Create an iterator for the period, which can generate values in the past.
    fn new_fixed(period: Period) -> Self {
        let start = period.start;
        Self::new(period, start)
    }

    /// Create an iterator for the period, which will only generate values after the current timestamp.
    fn new_relative(period: Period) -> Self {
        let start = next_available_timestamp(period.start, &period.duration).unwrap();
        Self::new(period, start)
    }
}

impl Iterator for OwnedPeriodIterator {
    type Item = DateTime<Utc>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().inspect(|current| {
            self.current = Some(*current + self.period.duration);
        })
    }
}

fn adjust_duration(duration: Duration) -> Duration {
    Duration::seconds(duration.as_seconds_f64().round() as i64)
}

fn next_available_timestamp<T>(timestamp: DateTime<T>, duration: &Duration) -> Option<DateTime<T>>
where
    T: TimeZone,
{
    let seconds_from_timestamp = Utc::now().timestamp() - timestamp.timestamp();

    Some(match seconds_from_timestamp.cmp(&0) {
        Ordering::Less => timestamp.clone(),
        Ordering::Equal => timestamp.clone() + *duration,
        Ordering::Greater => {
            let elapsed_durations =
                (seconds_from_timestamp as u32).div_ceil(duration.num_seconds() as u32) as i32;
            timestamp.clone() + duration.checked_mul(elapsed_durations).unwrap()
        }
    })
}

#[cfg(test)]
#[path = "./period/tests.rs"]
mod tests;
