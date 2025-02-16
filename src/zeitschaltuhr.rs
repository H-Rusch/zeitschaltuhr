use std::time::{Instant, SystemTime};

use crate::task::Task;
use crate::temporal_iterator::TemporalIterator;
use chrono::DateTime;
use tokio::time::sleep_until;

use chrono::Utc;

#[derive(Default)]
pub struct Zeitschaltuhr {
    tasks: Vec<ScheduledTask>,
}

impl Zeitschaltuhr {
    pub fn add_task(&mut self, task: Box<dyn Task>, temporal_iterator: Box<dyn TemporalIterator>) {
        let scheduled_task = ScheduledTask::new(temporal_iterator, task);
        self.tasks.push(scheduled_task);
    }

    pub fn run(self) {
        for scheduled_task in self.tasks {
            tokio::spawn(async move {
                execute_task(scheduled_task).await;
            });
        }
    }
}

async fn execute_task(scheduled_task: ScheduledTask) {
    for time in scheduled_task.original_iterator.iter_times() {
        sleep_until(to_instant(time)).await;
        scheduled_task.task.execute();
    }
}

fn to_instant(date_time: DateTime<Utc>) -> tokio::time::Instant {
    let offset_from_epoch = u64::try_from(date_time.timestamp()).unwrap_or_default();
    let target_system_time =
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(offset_from_epoch);

    // offset from now until target time
    let now = SystemTime::now();
    let target_instant = match target_system_time.duration_since(now) {
        Ok(duration) => Instant::now() + duration,
        Err(_) => Instant::now(),
    };

    tokio::time::Instant::from_std(target_instant)
}

struct ScheduledTask {
    original_iterator: Box<dyn TemporalIterator + Send + Sync>,
    task: Box<dyn Task>,
}

impl ScheduledTask {
    fn new(
        original_iterator: Box<dyn TemporalIterator + Send + Sync>,
        task: Box<dyn Task>,
    ) -> Self {
        Self {
            original_iterator,
            task,
        }
    }
}

#[cfg(test)]
mod tests {

    use chrono::Duration;

    use crate::{period::Period, task::PrintingTask};

    use super::*;

    #[test]
    fn that_zeitschaltuhr_can_be_created() {
        let zeitschaltuhr = Zeitschaltuhr::default();

        assert!(zeitschaltuhr.tasks.is_empty());
    }

    #[test]
    fn that_scheduled_task_can_be_created_added_to_zeitschaltuhr() {
        let mut zeitschaltuhr = Zeitschaltuhr::default();
        let period = Period::starting_at(Utc::now(), Duration::seconds(10)).unwrap();
        let task = PrintingTask::new("some text".to_string());

        zeitschaltuhr.add_task(Box::new(task), Box::new(period));

        assert_eq!(1, zeitschaltuhr.tasks.len());
    }
}
