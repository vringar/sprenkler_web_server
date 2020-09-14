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
            .and(warp::body::json())
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
    use tracing::{event, Level, instrument};

    #[instrument]
    pub async fn toggle_valve_status(
        index: usize,
        config: Arc<ServerConfig>,
        new_state: AutomationStatus,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut controller_config = config.as_ref().controller_configs[0].write();
        let v = &mut controller_config.valves[index];
        v.automation_status = new_state.clone();
        v.valve_status = match new_state {
            AutomationStatus::ForceClose => ValveStatus::Close,
            AutomationStatus::ForceOpen => ValveStatus::Open,
            AutomationStatus::Scheduled => match v.should_be_running(Local::now().naive_local()) {
                true => ValveStatus::Open,
                false => ValveStatus::Close,
            },
        };
        Ok(StatusCode::OK)
    }
    #[instrument]
    pub fn get_details_template(
        i: usize,
        config: Arc<ServerConfig>,
    ) -> WithTemplate<serde_json::Value> {
        let controller_config = config.as_ref().controller_configs[0].read();
        let valve = &controller_config.valves[i];
        let valve = json!(valve);
        event!(Level::INFO, "The json looks like this {:?} ",valve);

        WithTemplate {
            name: "timetable",
            value: valve,
        }
    }
}
