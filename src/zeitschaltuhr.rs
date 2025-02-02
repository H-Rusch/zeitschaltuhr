use crate::period::{Period, RealTimeProvider};
use chrono::{DateTime, Local};
use cron::Schedule;

pub enum Scheduling {
    Fixed(Schedule),
    Dynamic(Period),
}

pub trait Abc {
    fn lel(&self) -> Box<dyn Iterator<Item = DateTime<Local>> + '_>;
}

impl Abc for Scheduling {
    fn lel(&self) -> Box<dyn Iterator<Item = DateTime<Local>> + '_> {
        match &self {
            Scheduling::Fixed(schedule) => Box::new(schedule.upcoming(Local)),
            Scheduling::Dynamic(period) => Box::new(period.upcoming_fixed()),
        }
    }
}
