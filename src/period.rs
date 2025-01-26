use chrono::{DateTime, Duration, Local};
use cron::Schedule;

pub struct Period {
    start: DateTime<Local>,
    duration: Duration,
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
        return if duration.is_zero() {
            Err(PeriodError::ZeroDurationError)
        } else if duration.num_seconds().is_negative()
            || duration.num_nanoseconds().unwrap_or(0).is_negative()
        {
            Err(PeriodError::NegativeDurationError)
        } else {
            Ok(Period { start, duration })
        };
    }

    pub fn upcoming(&self) -> PeriodIterator {
        PeriodIterator::new(self)
    }
}

pub struct PeriodIterator {
    duration: Duration,
    current: Option<DateTime<Local>>,
}

impl PeriodIterator {
    fn new(period: &Period) -> Self {
        PeriodIterator {
            duration: period.duration,
            current: Some(period.start),
        }
    }
}

impl Iterator for PeriodIterator {
    type Item = DateTime<Local>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            let next = current + self.duration;
            self.current = Some(next);
        }

        self.current
    }
}

pub trait UpcomingDates {
    fn upcoming2(&self) -> Box<dyn Iterator<Item = DateTime<Local>> + '_>;
}

impl UpcomingDates for Period {
    fn upcoming2(&self) -> Box<dyn Iterator<Item = DateTime<Local>> + '_> {
        Box::new(self.upcoming())
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
    use std::{iter, str::FromStr};

    use super::*;

    #[test]
    fn that_period_can_be_created() {
        let now = Local::now();
        let duration = Duration::hours(1);

        let result = Period::starting_at(now, duration);

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

        let mut period_iterator = period.upcoming();

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration);
    }

    #[test]
    fn that_next_returns_increasing_timestamp() {
        let start = Local::now();
        let duration = Duration::minutes(12);
        let period = Period::starting_at(start, duration).unwrap();
        let mut period_iterator = PeriodIterator::new(&period);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration + duration);
    }

    #[test]
    fn that_next_returns_values_starting_from_current_timestamp() {
        todo!()
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
        let mut period_iterator = PeriodIterator::new(&period);

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
        let mut period_iterator = PeriodIterator::new(&period);

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
