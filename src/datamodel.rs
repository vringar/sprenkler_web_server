use serde::{Serialize, Deserialize};
use reqwest::Url;

#[derive(Serialize, Deserialize, Debug)]
struct Schedule {
    status: bool
}
#[derive(Serialize, Deserialize,Debug)]
pub enum Status {
    ScheduledOpen,
    ScheduledClosed,
    OverrideOn,
    OverrideOff
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Valve {
    pub name: String,
    pub index: i8,
    pub status: Status,
    schedule: Schedule
}

impl Valve {
    pub fn new(name: &str, index: i8) -> Self {
        Valve{name: name.to_owned(), index: index, status: Status::OverrideOff, schedule: Schedule{status: false}}
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub valves: Vec<Valve>,
    pub adress: Url
}