use chrono::naive::{NaiveDateTime, NaiveTime};
use chrono::Datelike;
use chrono::Weekday;
use parking_lot::RwLock;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Duration {
    begin: NaiveTime,
    end: NaiveTime,
}

impl Duration {
    fn sample() -> Self {
        let begin = NaiveTime::from_hms(12, 30, 00);
        let end = NaiveTime::from_hms(13, 00, 00);
        Self { begin, end }
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct DailySchedule(Vec<Duration>);

impl DailySchedule {
    fn new() -> Self {
        DailySchedule(vec![Duration::sample(), Duration::sample(), Duration::sample()])
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Schedule(HashMap<Weekday, DailySchedule>);

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
    pub index: i8,
    pub automation_status: AutomationStatus,
    pub valve_status: ValveStatus,
    schedule: Schedule,
}

impl Valve {
    pub fn new(name: &str, index: i8) -> Self {
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ControllerConfig {
    pub valves: Vec<Valve>,
    pub adress: Url,
}
#[derive(Debug)]
pub struct ServerConfig {
    pub controller_configs: Vec<RwLock<ControllerConfig>>,
}
impl ServerConfig {
    pub fn new(cc: ControllerConfig) -> Self {
        let cc = RwLock::from(cc);
        ServerConfig {
            controller_configs: vec![cc],
        }
    }
}
