use chrono::{DateTime, Duration, Local, TimeZone, Utc};

#[derive(Clone)]
pub struct Period<T>
where
    T: TimeZone,
{
    timezone: T,
    start: DateTime<T>,
    duration: Duration,
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
    pub fn starting_at(
        start: DateTime<T>,
        duration: Duration,
        timezone: T,
    ) -> Result<Self, PeriodError> {
        // TODO dont support sub second accuracy
        if duration.is_zero() {
            Err(PeriodError::ZeroDurationError)
        } else if duration.num_seconds().is_negative()
            || duration.num_nanoseconds().unwrap_or(0).is_negative()
        {
            Err(PeriodError::NegativeDurationError)
        } else {
            Ok(Period {
                timezone,
                start,
                duration,
            })
        }
    }

    pub fn upcoming_relative(&self) -> PeriodIterator<T> {
        PeriodIterator::new_relative(self)
    }

    pub fn upcoming_fixed(&self) -> PeriodIterator<T> {
        PeriodIterator::new_fixed(self)
    }

    /// Return an iterator of DateTimes that takes ownership of the Period. That iterator will only generate values in the future.
    pub fn upcoming_relative_owned(self) -> OwnedPeriodIterator<T> {
        OwnedPeriodIterator::new_relative(self)
    }

    /// Return an iterator of DateTimes that takes ownership of the Period. The iterator can generate values in the past.
    pub fn upcoming_fixed_owned(self) -> OwnedPeriodIterator<T> {
        OwnedPeriodIterator::new_fixed(self)
    }
}

pub struct PeriodIterator<'a, T>
where
    T: TimeZone,
{
    period: &'a Period<T>,
    current: Option<DateTime<T>>,
}

impl<'a, T> PeriodIterator<'a, T>
where
    T: TimeZone,
{
    fn new(period: &'a Period<T>, start: DateTime<T>) -> Self {
        PeriodIterator {
            period,
            current: Some(start),
        }
    }

    /// Create an iterator for the period, which can generate values in the past.
    fn new_fixed(period: &'a Period<T>) -> Self {
        Self::new(period, period.start.clone())
    }

    /// Create an iterator for the period, which will only generate values after the current timestamp.
    fn new_relative(period: &'a Period<T>) -> Self {
        let start =
            next_available_timestamp(period.start.clone(), &period.duration, &period.timezone)
                .unwrap();
        Self::new(period, start)
    }
}

impl<T> Iterator for PeriodIterator<'_, T>
where
    T: TimeZone,
{
    type Item = DateTime<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().inspect(|current| {
            self.current = Some(current.clone() + self.period.duration);
        })
    }
}

pub struct OwnedPeriodIterator<T>
where
    T: TimeZone,
{
    period: Period<T>,
    current: Option<DateTime<T>>,
}

impl<T> OwnedPeriodIterator<T>
where
    T: TimeZone,
{
    fn new(period: Period<T>, start: DateTime<T>) -> Self {
        OwnedPeriodIterator {
            period,
            current: Some(start),
        }
    }

    /// Create an iterator for the period, which can generate values in the past.
    fn new_fixed(period: Period<T>) -> Self {
        let start = period.start.clone();
        Self::new(period, start)
    }

    /// Create an iterator for the period, which will only generate values after the current timestamp.
    fn new_relative(period: Period<T>) -> Self {
        let start =
            next_available_timestamp(period.start.clone(), &period.duration, &period.timezone)
                .unwrap();
        Self::new(period, start)
    }
}

impl<T> Iterator for OwnedPeriodIterator<T>
where
    T: TimeZone,
{
    type Item = DateTime<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().inspect(|current| {
            self.current = Some(current.clone() + self.period.duration);
        })
    }
}

fn next_available_timestamp<T>(
    timestamp: DateTime<T>,
    duration: &Duration,
    timezone: &T,
) -> Option<DateTime<T>>
where
    T: TimeZone,
{
    let now = now_as_timezone(timezone);

    println!("duration {:?}", duration);
    println!("input  {:?}", timestamp.clone());
    println!("now    {:?}", now.clone());
    if now.timestamp() == timestamp.timestamp() {
        return Some(timestamp.clone() + *duration);
    }

    // calculate how often the duration fits in the time_difference between "now" and "current"
    let time_difference = (now.timestamp() - timestamp.timestamp()).max(0) as u32;
    let elapsed_duration_count = (time_difference).div_ceil(duration.num_seconds() as u32) as i64;

    println!("{:?}", elapsed_duration_count);

    // TODO remove clone
    // TODO use Duration.mult
    let result =
        timestamp.clone() + Duration::seconds(elapsed_duration_count * duration.num_seconds());

    println!("result {:?}", result.clone());

    Some(result)
}

fn now_as_timezone<T>(timezone: &T) -> DateTime<T>
where
    T: TimeZone,
{
    timezone.from_utc_datetime(&Utc::now().naive_utc())
}

#[cfg(test)]
#[path = "./period/tests.rs"]
mod tests;
