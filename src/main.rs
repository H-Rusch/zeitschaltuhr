// mod zeitschaltuhr;
mod period;
mod zeitschaltuhr;

use std::str::FromStr;

use chrono::{Duration, TimeZone};
use chrono::{Local, Utc};
use cron::Schedule;
use period::Period;
use zeitschaltuhr::{Abc, Scheduling};

fn main() {
    let expression = "0   30   9,12,15     1,15       May-Aug  Mon,Wed,Fri  2018/2";
    let schedule = Schedule::from_str(expression).unwrap();

    println!("{:?}", schedule.upcoming(Local).take(2).collect::<Vec<_>>());

    let duration = Duration::days(7);
    let now = Local::now();
    let new_time = now + duration;

    println!("{} + {} = {}", now, duration, new_time);

    println!(
        "{:?}",
        Local
            .with_ymd_and_hms(2025, 3, 30, 1, 0, 0)
            .single()
            .unwrap()
    );

    let period = Period::starting_now(Duration::weeks(100)).unwrap();
    println!("{:?}", period.upcoming().next());

    let a_period = Scheduling::Dynamic(period);
    let a_schedule = Scheduling::Fixed(schedule);

    let a = a_period.lel().take(2).collect::<Vec<_>>();
    let b = a_schedule.lel().take(2).collect::<Vec<_>>();

    let mut together = a.iter().chain(b.iter()).collect::<Vec<_>>();
    together.sort();
    println!("{:?}", together);

    let scheduled_time = together.iter().next().unwrap();
    let duration_until = (**scheduled_time - now).to_std().unwrap();

    println!("{:?}", duration_until);
}
