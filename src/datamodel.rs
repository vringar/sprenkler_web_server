use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Schedule {
    status: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Valve {
    pub name: String,
    pub index: i8,
    schedule: Schedule
}

impl Valve {
    pub fn new(name: &str, index: i8) -> Self {
        Valve{name: name.to_owned(), index: index, schedule: Schedule{status: false}}
    }
}