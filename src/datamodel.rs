use reqwest::Url;
use serde::{Deserialize, Serialize};
use chrono::Weekday;
#[derive(Serialize, Deserialize, Debug)]
struct DailySchedule {

}

#[derive(Serialize, Deserialize, Debug)]
struct Schedule {
    daily_schedule: [DailySchedule; 7],
    status: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum ValveStatus {
    Open,
    Close,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum AutomationStatus {
    Scheduled,
    Manual,
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
            automation_status: AutomationStatus::Manual,
            schedule: Schedule { status: false },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub valves: Vec<Valve>,
    pub adress: Url,
}
