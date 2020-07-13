use std::sync::Arc;
use warp::{Filter, Rejection};

use handlebars::Handlebars;
use serde_json::json;

use crate::hb::render;
use crate::hb::WithTemplate;

use crate::datamodel::Config;

pub fn get_valve_paths(
    hb: Arc<Handlebars>,
    config: Arc<Config>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + '_ {
    let hb = hb.clone();
    let handlebars = move |with_template| render(with_template, hb.clone());
    warp::path("valves")
        .and(warp::path::param())
        .and(warp::path::end())
        .map(move |i: usize| WithTemplate {
            name: "timetable",
            value: json!((*config).valves[i]),
        })
        .map(handlebars.clone())
}
