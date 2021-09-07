use crate::datamodel::{Error, ServerConfig, Valve, ValveStatus};
use chrono::{Local, NaiveDateTime};
use reqwest::{Client, Url};
use tokio::time::{sleep, Duration};

pub async fn control_valves(config: ServerConfig) -> ! {
    let client = reqwest::Client::new();
    loop {
        sleep(Duration::from_secs(60)).await;
        let config = config.read().await;

        let time: NaiveDateTime = Local::now().naive_local();
        for valve in (*config).iter() {
            send_valve_status(&client, config.address.clone(), valve, time).await.unwrap();
        }
    }
}

async fn send_valve_status(client: &Client, url: Url, valve: &Valve, time: NaiveDateTime) -> Result<(), Error> {
    let url = url
        .join("/valves/")
        .and_then(|url| url.join(&valve.valve_number.to_string())).unwrap();
    let body = match valve.valve_status(time) {
        ValveStatus::Open => "open",
        ValveStatus::Close => "closed",
    };
    let builder = client.put(url).body(body);
    builder.build()?;
    Ok(())
}
