use chrono::naive::{NaiveDateTime, NaiveTime};
use chrono::Datelike;
use chrono::Weekday;
use reqwest::Url;
use serde::{Deserialize, Serialize, Serializer};
use std::slice::{Iter, IterMut};
use std::{fmt, sync::Arc};
use tokio::sync::RwLock;

use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    BeginAfterEnd,
    OverlappingDurations,
    InvalidValveNumber,
    MissingDuration,
    Request(reqwest::Error),
}
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for Error {}

impl warp::reject::Reject for Error {}
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Duration {
    begin: NaiveTime,
    end: NaiveTime,
}

impl Duration {
    pub fn new(begin: NaiveTime, end: NaiveTime) -> Result<Duration, Error> {
        if begin >= end {
            return Err(Error::BeginAfterEnd);
        }
        Ok(Duration { begin, end })
    }

    pub fn is_overlapping(&self, other: &Self) -> bool {
        if self.end < other.begin {
            return false;
        }
        if other.end < self.begin {
            return false;
        }
        true
    }

    pub fn contains(&self, other: &NaiveTime) -> bool {
        &self.begin < other && other < &self.end
    }
}
#[derive(Serialize, Deserialize, Debug, Default)]
struct DailySchedule(Vec<Duration>);

impl DailySchedule {
    pub fn add_entry(&mut self, duration: Duration) -> Result<(), Error> {
        if self.0.iter().any(|v| v.is_overlapping(&duration)) {
            return Err(Error::OverlappingDurations);
        }
        self.0.push(duration);
        Ok(())
    }
    pub fn remove_entry(&mut self, duration: Duration) -> Result<(), Error> {
        self.0.retain(|d| duration != *d);
        Ok(())
    }

    pub fn should_be_running(&self, time: &NaiveTime) -> bool {
        self.0.iter().any(|d| d.contains(time))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Schedule(#[serde(serialize_with = "daymap")] HashMap<Weekday, DailySchedule>);

impl Schedule {
    fn empty() -> Self {
        let mut sched = Schedule(HashMap::with_capacity(7));
        sched.insert(Weekday::Mon, DailySchedule::default());
        sched.insert(Weekday::Tue, DailySchedule::default());
        sched.insert(Weekday::Wed, DailySchedule::default());
        sched.insert(Weekday::Thu, DailySchedule::default());
        sched.insert(Weekday::Fri, DailySchedule::default());
        sched.insert(Weekday::Sat, DailySchedule::default());
        sched.insert(Weekday::Sun, DailySchedule::default());
        sched
    }
    fn insert(&mut self, weekday: Weekday, daily_schedule: DailySchedule) {
        self.0.insert(weekday, daily_schedule);
    }
}

impl std::ops::Index<&Weekday> for Schedule {
    type Output = DailySchedule;

    fn index(&self, index: &Weekday) -> &Self::Output {
        self.0.get(index).unwrap()
    }
}

impl std::ops::IndexMut<&Weekday> for Schedule {
    fn index_mut(&mut self, index: &Weekday) -> &mut Self::Output {
        self.0.get_mut(index).unwrap()
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
pub type ValveNumber = u8;
#[derive(Serialize, Deserialize, Debug)]
pub struct Valve {
    pub name: String,
    pub valve_number: ValveNumber,
    pub automation_status: AutomationStatus,
    schedule: Schedule,
}

impl Valve {
    pub fn new(name: impl Into<String>, valve_number: ValveNumber) -> Self {
        Valve {
            name: name.into(),
            valve_number,
            automation_status: AutomationStatus::ForceClose,
            schedule: Schedule::empty(),
        }
    }

    pub fn valve_status(&self, current_time: NaiveDateTime) -> ValveStatus {
        match self.automation_status {
            AutomationStatus::ForceClose => ValveStatus::Close,
            AutomationStatus::ForceOpen => ValveStatus::Open,
            AutomationStatus::Scheduled => {
                let daily_schedule = &self.schedule[&current_time.weekday()];
                match daily_schedule.should_be_running(&current_time.time()) {
                    true => ValveStatus::Open,
                    false => ValveStatus::Close,
                }
            }
        }
    }

    pub fn add_duration(&mut self, day: &Weekday, duration: Duration) -> Result<(), Error> {
        self.schedule[day].add_entry(duration)
    }

    pub fn remove_duration(&mut self, day: &Weekday, duration: Duration) -> Result<(), Error> {
        self.schedule[day].remove_entry(duration)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ControllerConfig {
    valves: Vec<Valve>,
    pub address: Url,
}

impl ControllerConfig {
    pub fn new(address: Url) -> Self {
        ControllerConfig {
            valves: Default::default(),
            address,
        }
    }

    pub fn get(&self, valve_number: ValveNumber) -> Option<&Valve> {
        self.valves.iter().find(|v| v.valve_number == valve_number)
    }

    pub fn get_mut(&mut self, valve_number: ValveNumber) -> Option<&mut Valve> {
        self.valves
            .iter_mut()
            .find(|v| v.valve_number == valve_number)
    }

    pub fn push(&mut self, valve: Valve) {
        self.valves.push(valve)
    }

    pub fn remove_valve(&mut self, valve_number: ValveNumber) -> bool {
        let mut found_smt = false;
        self.valves.retain(|v| {
            let res = v.valve_number != valve_number;
            if !res {
                found_smt = true;
            }
            res
        });
        found_smt
    }

    pub fn iter(&self) -> Iter<Valve> {
        self.valves.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Valve> {
        self.valves.iter_mut()
    }
}

impl IntoIterator for ControllerConfig {
    type Item = Valve;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.valves.into_iter()
    }
}

pub type ServerConfig = Arc<RwLock<ControllerConfig>>;
