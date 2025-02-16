use crate::period::Period;
use chrono::{DateTime, Utc};
use cron::Schedule;

pub trait TemporalIterator: Send + Sync + 'static {
    fn iter_times(&self) -> Box<dyn Iterator<Item = DateTime<Utc>> + Send>;
}

impl TemporalIterator for Period {
    fn iter_times(&self) -> Box<dyn Iterator<Item = DateTime<Utc>> + Send> {
        Box::new(self.clone().upcoming_relative_owned())
    }
}

impl TemporalIterator for Schedule {
    fn iter_times(&self) -> Box<dyn Iterator<Item = DateTime<Utc>> + Send> {
        Box::new(self.upcoming_owned(Utc))
    }
}

#[cfg(test)]
mod tests {

    use chrono::{Duration, TimeZone, Utc};
    use std::str::FromStr;

    use super::*;

    #[test]
    fn that_iter_times_of_cron_schedule_returns_iterator_of_datetimes() {
        // '2100' is maximum year supported by cron-crate
        let expression = "0   30   12     1,15       May  *  2100";
        let schedule = Schedule::from_str(expression).unwrap();
        let mut dates = schedule.iter_times();

        assert_eq!(
            dates.next(),
            Utc.with_ymd_and_hms(2100, 5, 1, 12, 30, 0).single()
        );
        assert_eq!(
            dates.next(),
            Utc.with_ymd_and_hms(2100, 5, 15, 12, 30, 0).single()
        );
        assert_eq!(dates.next(), None);
    }

    #[test]
    fn that_iter_times_of_period_returns_iterator_of_datetimes() {
        let start = Utc::now();
        let duration = Duration::minutes(12);
        let period = Period::starting_at(start, duration).unwrap();

        let mut period_iterator = period.iter_times();

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration + duration);
    }
}
