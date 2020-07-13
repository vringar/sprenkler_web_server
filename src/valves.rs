use std::sync::Arc;
use warp::{Filter, Rejection};

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

    warp::path("valves").and(root)
}
