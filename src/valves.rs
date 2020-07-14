use handlebars::Handlebars;
use std::sync::Arc;
use warp::{Filter, Rejection};

use crate::datamodel::ServerConfig;
use filters::{valve_overview, valve_toggle};

pub fn get_valve_paths(
    hb: Arc<Handlebars>,
    config: Arc<ServerConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + '_ {
    let hb = hb.clone();
    let root = {
        let config = config.clone();
        valve_overview(config, hb)
    };

    let toggle_status = valve_toggle(config);

    warp::path("valves").and(root.or(toggle_status))
}

mod filters {
    use super::handlers::{get_details_template, toggle_valve_status};
    use crate::hb::render;
    use handlebars::Handlebars;

    use crate::datamodel::ServerConfig;
    use std::sync::Arc;
    use warp::Filter;

    // POST /:id/toggle
    pub fn valve_toggle(
        config: Arc<ServerConfig>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path::param())
            .and(warp::path("toggle"))
            .and(with_server_config(config))
            .and_then(toggle_valve_status)
    }

    pub fn valve_overview(
        config: Arc<ServerConfig>,
        hb: Arc<Handlebars>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + '_ {
        let render = move |t| render(t, hb.clone());
        warp::get()
            .and(warp::path::param())
            .and(warp::path::end())
            .and(with_server_config(config))
            .map(get_details_template)
            .map(render.clone())
    }
    pub fn with_server_config(
        config: Arc<ServerConfig>,
    ) -> impl Filter<Extract = (Arc<ServerConfig>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || config.clone())
    }
}

mod handlers {
    use crate::datamodel::{AutomationStatus, ServerConfig, ValveStatus};
    use chrono::Local;
    use std::convert::Infallible;
    use std::sync::Arc;
    use warp::http::StatusCode;

    use crate::hb::WithTemplate;

    use serde_json::json;

    pub async fn toggle_valve_status(
        index: usize,
        config: Arc<ServerConfig>,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut controller_config = config.as_ref().controller_configs[0].write();
        let v = &mut controller_config.valves[index];
        let status = (
            &v.automation_status,
            &v.valve_status,
            &v.should_be_running(Local::now().naive_local()),
        );
        let target_state = match status {
            (AutomationStatus::Scheduled, ValveStatus::Close, _) => {
                (AutomationStatus::Manual, ValveStatus::Open)
            }
            (AutomationStatus::Scheduled, ValveStatus::Open, _) => {
                (AutomationStatus::Manual, ValveStatus::Close)
            }
            (AutomationStatus::Manual, _, true) => (AutomationStatus::Scheduled, ValveStatus::Open),
            (AutomationStatus::Manual, _, false) => {
                (AutomationStatus::Scheduled, ValveStatus::Close)
            }
        };
        v.automation_status = target_state.0;
        v.valve_status = target_state.1;
        Ok(StatusCode::OK)
    }

    pub fn get_details_template(
        i: usize,
        config: Arc<ServerConfig>,
    ) -> WithTemplate<serde_json::Value> {
        let controller_config = config.as_ref().controller_configs[0].read();
        let valve = &controller_config.valves[i];
        let valve = json!(valve);
        println!("{}", valve);
        WithTemplate {
            name: "timetable",
            value: valve,
        }
    }
}
