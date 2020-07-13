use std::sync::Arc;
use warp::{Filter, Rejection};
use warp::http::StatusCode;

use handlebars::Handlebars;
use serde_json::json;

use crate::hb::render;
use crate::hb::WithTemplate;

use crate::datamodel::{ServerConfig, AutomationStatus, ValveStatus};
use chrono::Local;

pub fn get_valve_paths(
    hb: Arc<Handlebars>,
    config: Arc<ServerConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + '_ {
    let hb = hb.clone();
    let handlebars = move |with_template| render(with_template, hb.clone());
    let root = {   let config = config.clone();
         warp::get()
        .and(warp::path::param())
        .and(warp::path::end())
        .map(move |i: usize| {
            let controller_config = config.as_ref().controller_configs[0].read();
            let valve = &controller_config.valves[i];
            let valve = json!(valve);
            println!("{}",valve);
            WithTemplate {
            name: "timetable",
            value: valve,
        }})
        .map(handlebars.clone())
    };

    let toggle_status = warp::post()
    .and(warp::path::param())
    .and(warp::path("toggle"))
    .map(move |i: usize| {
        let mut controller_config = config.as_ref().controller_configs[0].write();
        let v =  & mut controller_config.valves[i];
        let status = (&v.automation_status, &v.valve_status, &v.should_be_running(Local::now().naive_local()));
        let target_state = match status {
            (AutomationStatus::Scheduled, ValveStatus::Close, _) => (AutomationStatus::Manual, ValveStatus::Open),
            (AutomationStatus::Scheduled, ValveStatus::Open, _) => (AutomationStatus::Manual, ValveStatus::Close),
            (AutomationStatus::Manual, _, true) => (AutomationStatus::Scheduled, ValveStatus::Open ),
            (AutomationStatus::Manual, _, false) => (AutomationStatus::Scheduled, ValveStatus::Close)
        };
        v.automation_status = target_state.0;
        v.valve_status = target_state.1;
        Ok(StatusCode::OK)
    } );

    warp::path("valves").and(root.or(toggle_status))
}
