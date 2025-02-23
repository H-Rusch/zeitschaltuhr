use std::str::FromStr;
use std::time::{Instant, SystemTime};

use crate::task::{PrintingTask, Task};
use crate::temporal_iterator::TemporalIterator;
use crate::{period::Period, temporal_iterator};
use chrono::{DateTime, Duration, Local, TimeZone, Timelike};
use cron::Schedule;
use tokio::time::sleep_until;

pub struct Zeitschaltuhr<T>
where
    T: TimeZone,
{
    timezone: T,
    tasks: Vec<Box<dyn TemporalIterator<T>>>,
}

impl<T> Zeitschaltuhr<T>
where
    T: TimeZone + 'static,
{
    pub fn new(timezone: T) -> Self {
        Self {
            timezone,
            tasks: vec![],
        }
    }

    pub fn add_task(
        &mut self,
        temporal_iterator: Box<dyn TemporalIterator<T>>,
        task: PrintingTask,
    ) {
        println!("add_task");
        let iterator = temporal_iterator.iter_times(&self.timezone);
        println!("iterator_created");
        let scheduled_task = ScheduledTask {
            temporal_iterator,
            iterator,
            task,
        };
        println!("scheduled_task_created");

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            self.execute_scheduled_task(scheduled_task).await;
        });
    }

    async fn execute_scheduled_task(&self, mut scheduled_task: ScheduledTask<T>) {
        println!("execute_scheduled_task");
        while let Some(time) = scheduled_task.iterator.next() {
            println!();
            println!("{:?} next value of iterator ", time);
            println!("{:?} now", Local::now());

            let wait_until = to_instant(time);
            sleep_until(wait_until).await;
            scheduled_task.task.execute();
        }
    }
}

fn to_instant<T>(date_time: DateTime<T>) -> tokio::time::Instant
where
    T: TimeZone,
{
    let offset_from_epoch = u64::try_from(date_time.timestamp()).unwrap_or_default();
    let target_system_time =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(offset_from_epoch);

    // offset from now until target time
    let now = SystemTime::now();
    let target_instant = match target_system_time.duration_since(now) {
        Ok(duration) => Instant::now() + duration,
        Err(_) => Instant::now(),
    };

    println!("target instant: {:?}", target_instant);

    tokio::time::Instant::from_std(target_instant)
}

struct ScheduledTask<T>
where
    T: TimeZone,
{
    temporal_iterator: Box<dyn TemporalIterator<T>>,
    iterator: Box<dyn Iterator<Item = DateTime<T>>>,
    task: PrintingTask,
}

/*impl<T> ScheduledTask<T>
where
    T: TimeZone,
{
    fn new(temporal_iterator: Box<dyn Iterator<Item = DateTime<T>>>, task: PrintingTask) -> Self {
        Self {
            temporal_iterator,
            task,
        }
    }
}*/
