use handlebars::Handlebars;
use std::sync::Arc;
use warp::{Filter, Rejection};

use crate::datamodel::ServerConfig;
use filters::{detail_view_filter, valve_update_status};
use serde::{Deserialize, Serialize};

use self::filters::{create_valve_filter, delete_valve_filter, homepage_filter};

pub fn get_dynamic_paths(
    hb: Arc<Handlebars>,
    config: Arc<ServerConfig>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + '_ {
    let homepage = homepage_filter(config.clone(), hb.clone());
    let create_valve = create_valve_filter(config.clone());

    let detail_view = detail_view_filter(config.clone(), hb.clone());
    let toggle_status = valve_update_status(config.clone());
    let delete_valve = delete_valve_filter(config.clone());

    homepage.or(warp::path("valves").and(
        detail_view
            .or(toggle_status)
            .or(create_valve)
            .or(delete_valve),
    ))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateValveParams {
    pub index: u8,
    pub name: String,
}

mod filters {
    use super::{
        handlers::{
            create_valve, delete_valve, render_details, render_homepage, update_valve_status,
        },
        CreateValveParams,
    };
    use crate::hb::render;
    use handlebars::Handlebars;

    use crate::datamodel::ServerConfig;
    use std::sync::Arc;
    use warp::Filter;

    // GET /
    pub fn homepage_filter(
        config: Arc<ServerConfig>,
        hb: Arc<Handlebars>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + '_ {
        let render = move |t| render(t, hb.clone());
        warp::get()
            .and(warp::path::end())
            .and(with_server_config(config))
            .map(render_homepage)
            .map(render.clone())
    }

    // POST /
    pub fn create_valve_filter(
        config: Arc<ServerConfig>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path::end())
            .and(warp::body::form::<CreateValveParams>())
            .and(with_server_config(config))
            .and_then(create_valve)
    }
    // GET /:id/
    pub fn detail_view_filter(
        config: Arc<ServerConfig>,
        hb: Arc<Handlebars>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + '_ {
        let render = move |t| render(t, hb.clone());
        warp::get()
            .and(warp::path::param())
            .and(warp::path::end())
            .and(with_server_config(config))
            .map(render_details)
            .map(render.clone())
    }

    // DELETE /:id/
    pub fn delete_valve_filter(
        config: Arc<ServerConfig>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::delete()
            .and(warp::path::param())
            .and(warp::path::end())
            .and(with_server_config(config))
            .and_then(delete_valve)
    }
    // POST /:id/status
    pub fn valve_update_status(
        config: Arc<ServerConfig>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path::param())
            .and(warp::path("status"))
            .and(with_server_config(config))
            .and(warp::body::json())
            .and_then(update_valve_status)
    }

    pub fn with_server_config(
        config: Arc<ServerConfig>,
    ) -> impl Filter<Extract = (Arc<ServerConfig>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || config.clone())
    }
}

mod handlers {
    use crate::datamodel::{AutomationStatus, ServerConfig, Valve, ValveStatus};
    use chrono::Local;
    use hyper::Uri;
    use std::convert::TryInto;
    use std::sync::Arc;
    use warp::{http::StatusCode, reject::Reject};

    use crate::hb::WithTemplate;

    use serde_json::json;

    use super::CreateValveParams;

    pub async fn update_valve_status(
        index: u8,
        config: Arc<ServerConfig>,
        new_state: AutomationStatus,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut controller_config = config.as_ref().controller_configs[0].write();
        let v = &mut controller_config
            .valves
            .iter_mut()
            .filter(|v| v.index == index)
            .last()
            .ok_or(warp::reject::custom(InvalidIndex {}))?;
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

    pub fn render_details(i: usize, config: Arc<ServerConfig>) -> WithTemplate<serde_json::Value> {
        let controller_config = config.as_ref().controller_configs[0].read();
        let valve = &controller_config.valves[i];
        let valve = json!(valve);
        println!("{}", valve);
        WithTemplate {
            name: "timetable",
            value: valve,
        }
    }
    #[derive(Debug)]
    pub struct InvalidIndex {}
    impl Reject for InvalidIndex {}

    pub async fn create_valve(
        params: CreateValveParams,
        config: Arc<ServerConfig>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut controller_config = config.controller_configs[0].write();
        if controller_config
            .valves
            .iter()
            .any(|v| v.index == params.index)
        {
            return Err(warp::reject::custom(InvalidIndex {}));
        }
        controller_config
            .valves
            .push(Valve::new(&params.name, params.index.try_into().unwrap()));
        Ok(warp::redirect(Uri::from_static("/")))
    }

    pub fn render_homepage(config: Arc<ServerConfig>) -> WithTemplate<serde_json::Value> {
        let controller_config = config.as_ref().controller_configs[0].read();
        let controller_config = &(*controller_config);
        return WithTemplate {
            name: "index",
            value: json!(controller_config),
        };
    }

    pub async fn delete_valve(
        index: u8,
        config: Arc<ServerConfig>,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut config = config.controller_configs[0].write();
        config.valves.retain(|v| v.index != index);
        Ok(warp::reply())
    }
}
