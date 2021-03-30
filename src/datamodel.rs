use chrono::naive::{NaiveDateTime, NaiveTime};
use chrono::Datelike;
use chrono::Weekday;
use reqwest::Url;
use serde::{Deserialize, Serialize, Serializer};
use std::{fmt, sync::{Arc, Mutex}};
use tokio::sync::RwLock;

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    BeginAfterEnd,
    OverlappingDurations
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for Error {

}
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Duration {
    begin: NaiveTime,
    end: NaiveTime,
}


impl Duration {
    fn sample() -> Self {
        let begin = NaiveTime::from_hms(12, 30, 00);
        let end = NaiveTime::from_hms(13, 00, 00);
        Self { begin, end }
    }

    pub fn new(begin: NaiveTime, end: NaiveTime) -> Result<Duration, Error> {
        if begin >= end {
            return Err(Error::BeginAfterEnd);
        }
        Ok(Duration{begin, end})
    }

    pub fn is_overlapping(&self, other: &Self) -> bool{
        if self.end < other.begin {
            return false;
        }
        if other.end < self.begin {
            return false;
        }
        return true;
    }
}
#[derive(Serialize, Deserialize, Debug, Default)]
struct DailySchedule(Vec<Duration>);

impl DailySchedule {
    fn new() -> Self {
        DailySchedule(vec![
            Duration::sample(),
            Duration::sample(),
            Duration::sample(),
        ])
    }
    pub fn add_entry(&mut self, duration: Duration) -> Result<(), Error> {
        if self.0.iter().any(|v| v.is_overlapping(&duration)) {
            return  Err(Error::OverlappingDurations);
        }
        Ok(self.0.push(duration))
    }
    pub fn remove_entry(&mut self, duration: Duration) -> Result<(), Error> {
        Ok(self.0.retain(|d| duration != *d))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Schedule(#[serde(serialize_with = "daymap")] HashMap<Weekday, DailySchedule>);

impl Schedule {
    fn empty() -> Self {
        let mut sched = Schedule(HashMap::with_capacity(7));
        sched.insert(Weekday::Mon, DailySchedule::new());
        sched.insert(Weekday::Tue, DailySchedule::new());
        sched.insert(Weekday::Wed, DailySchedule::new());
        sched.insert(Weekday::Thu, DailySchedule::new());
        sched.insert(Weekday::Fri, DailySchedule::new());
        sched.insert(Weekday::Sat, DailySchedule::new());
        sched.insert(Weekday::Sun, DailySchedule::new());
        sched
    }
    fn insert(&mut self, weekday: Weekday, daily_schedule: DailySchedule) {
        self.0.insert(weekday, daily_schedule);
    }
    fn get(&self, weekday: &Weekday) -> &DailySchedule {
        self.0.get(weekday).unwrap()
    }
}

fn daymap<S>(value: &HashMap<Weekday, DailySchedule>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ordered: Vec<_> = value.iter().collect();
    ordered.sort_by(|a, b| a.0.num_days_from_monday().cmp(&b.0.num_days_from_monday()));
    ordered.serialize(serializer)
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ValveStatus {
    Open,
    Close,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AutomationStatus {
    ForceOpen,
    Scheduled,
    ForceClose,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Valve {
    pub name: String,
    pub index: u8,
    pub automation_status: AutomationStatus,
    pub valve_status: ValveStatus,
    schedule: Schedule,
}

impl Valve {
    pub fn new(name: &str, index: u8) -> Self {
        Valve {
            name: name.to_owned(),
            index,
            valve_status: ValveStatus::Close,
            automation_status: AutomationStatus::ForceClose,
            schedule: Schedule::empty(),
        }
    }

    pub fn should_be_running(&self, current_time: NaiveDateTime) -> bool {
        let daily_schedule = self.schedule.get(&current_time.weekday());
        daily_schedule
            .0
            .iter()
            .any(|d| d.begin < current_time.time() && current_time.time() < d.end)
    }

    pub fn add_duration(&mut self, day: &Weekday, duration: Duration) -> Result<(), Error> {
        self.schedule.0.get_mut(day).unwrap().add_entry(duration)
    }

    pub fn remove_duration(&mut self, day: &Weekday, duration: Duration) -> Result<(), Error> {
        self.schedule.0.get_mut(day).unwrap().remove_entry(duration)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ControllerConfig {
    pub valves: Vec<Valve>,
    pub adress: Url,
}

pub type ServerConfig = Arc<RwLock<ControllerConfig>>;
