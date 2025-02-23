use crate::period::Period;
use chrono::{DateTime, TimeZone};
use cron::Schedule;

pub trait TemporalIterator<T>
where
    T: TimeZone + 'static,
{
    fn iter_times(&self, timezone: &T) -> Box<dyn Iterator<Item = DateTime<T>>>;
}

impl<T> TemporalIterator<T> for Period<T>
where
    T: TimeZone + 'static,
{
    fn iter_times(&self, _: &T) -> Box<dyn Iterator<Item = DateTime<T>>> {
        Box::new(self.clone().upcoming_relative_owned())
    }
}

impl<T> TemporalIterator<T> for Schedule
where
    T: TimeZone + 'static,
{
    fn iter_times(&self, timezone: &T) -> Box<dyn Iterator<Item = DateTime<T>>> {
        Box::new(self.upcoming_owned(timezone.clone()))
    }
}

#[cfg(test)]
mod tests {

    use chrono::{Duration, Local};
    use std::str::FromStr;

    use super::*;

    #[test]
    fn that_iter_times_of_cron_schedule_returns_iterator_of_datetimes() {
        // '2100' is maximum year supported by cron-crate
        let expression = "0   30   12     1,15       May  *  2100";
        let schedule = Schedule::from_str(expression).unwrap();
        let mut dates = schedule.iter_times(&Local);

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

    #[test]
    fn that_iter_times_of_period_returns_iterator_of_datetimes() {
        let start = Local::now();
        let duration = Duration::minutes(12);
        let period = Period::starting_at(start, duration, Local).unwrap();

        let mut period_iterator = period.iter_times(&Local);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration);

        let next = period_iterator.next().unwrap();
        assert_eq!(next, start + duration + duration);
    }
}
