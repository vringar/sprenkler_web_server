use chrono::{NaiveTime, Weekday};
use handlebars::Handlebars;
use std::sync::Arc;
use warp::{Filter, Rejection};

use filters::{detail_view_filter, update_valve_status_filter};
use serde::{Deserialize, Serialize};

use crate::datamodel::{ServerConfig, ValveNumber};

use self::filters::{
    add_duration_filter, create_valve_filter, delete_duration_filter, delete_valve_filter,
    homepage_filter,
};

pub fn get_dynamic_paths(
    hb: Arc<Handlebars>,
    config: ServerConfig,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + '_ {
    let homepage = homepage_filter(config.clone(), hb.clone());

    let create_valve = create_valve_filter(config.clone());
    let delete_valve = delete_valve_filter(config.clone());

    let toggle_status = update_valve_status_filter(config.clone());

    let detail_view = detail_view_filter(config.clone(), hb.clone());

    let add_duration = add_duration_filter(config.clone());
    let delete_duration = delete_duration_filter(config);

    homepage.or(warp::path("valves").and(
        detail_view
            .or(toggle_status)
            .or(create_valve)
            .or(delete_valve)
            .or(add_duration)
            .or(delete_duration),
    ))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValveParams {
    pub valve_number: ValveNumber,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimetableParams {
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub day: Weekday,
}
mod filters {
    use super::handlers::{
        add_duration, create_valve, delete_duration, delete_valve, render_details, render_homepage,
        update_valve_status,
    };
    use crate::{datamodel::ServerConfig, hb::render};
    use handlebars::Handlebars;

    use std::sync::Arc;
    use warp::Filter;

    /// GET /
    pub fn homepage_filter(
        config: ServerConfig,
        hb: Arc<Handlebars>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + '_ {
        let render = move |t| render(t, hb.clone());
        warp::get()
            .and(warp::path::end())
            .and(with_server_config(config))
            .and_then(render_homepage)
            .and_then(render.clone())
    }

    /// POST /
    pub fn create_valve_filter(
        config: ServerConfig,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path::end())
            .and(warp::body::form())
            .and(with_server_config(config))
            .and_then(create_valve)
    }
    /// GET /:id/
    pub fn detail_view_filter(
        config: ServerConfig,
        hb: Arc<Handlebars>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + '_ {
        let render = move |t| render(t, hb.clone());
        warp::get()
            .and(warp::path::param())
            .and(warp::path::end())
            .and(with_server_config(config))
            .and_then(render_details)
            .and_then(render.clone())
    }

    /// DELETE /:id/
    pub fn delete_valve_filter(
        config: ServerConfig,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::delete()
            .and(warp::path::param())
            .and(warp::path::end())
            .and(with_server_config(config))
            .and_then(delete_valve)
    }
    /// POST /:id/status
    pub fn update_valve_status_filter(
        config: ServerConfig,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path::param())
            .and(warp::path("status"))
            .and(with_server_config(config))
            .and(warp::body::json())
            .and_then(update_valve_status)
    }

    /// POST /:id/timetable
    pub fn add_duration_filter(
        config: ServerConfig,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path::param())
            .and(warp::path("timetable"))
            .and(with_server_config(config))
            .and(warp::body::form())
            .and_then(add_duration)
    }
    /// DELETE /:id/timetable
    pub fn delete_duration_filter(
        config: ServerConfig,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::delete()
            .and(warp::path::param())
            .and(warp::path("timetable"))
            .and(with_server_config(config))
            .and(warp::body::json())
            .and_then(delete_duration)
    }

    pub fn with_server_config(
        config: ServerConfig,
    ) -> impl Filter<Extract = (ServerConfig,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || config.clone())
    }
}

mod handlers {
    use crate::datamodel::{
        AutomationStatus, ControllerConfig, Duration, Error::InvalidValveNumber, Schedule,
        ServerConfig, Valve, ValveNumber, ValveStatus,
    };

    use chrono::{Local, NaiveDateTime};
    use hyper::Uri;
    use reqwest::Url;

    use std::convert::{Infallible, TryFrom};
    use warp::http::StatusCode;

    use crate::hb::WithTemplate;

    use serde::Serialize;
    use serde_json::json;

    use super::{TimetableParams, ValveParams};

    #[derive(Serialize, Debug)]
    pub struct ValveData<'a> {
        name: &'a str,
        valve_number: ValveNumber,
        automation_status: AutomationStatus,
        schedule: &'a Schedule,
        valve_status: ValveStatus,
    }

    impl<'a> ValveData<'a> {
        pub fn from(valve: &'a Valve, time: NaiveDateTime) -> ValveData<'a> {
            ValveData {
                name: &valve.name,
                valve_number: valve.valve_number,
                automation_status: valve.automation_status.clone(),
                schedule: valve.schedule(),
                valve_status: valve.valve_status(time),
            }
        }
    }
    #[derive(Serialize, Debug)]
    struct HomepageData<'a> {
        valves: Vec<ValveData<'a>>,
        address: &'a Url,
    }

    impl<'a> HomepageData<'a> {
        pub fn from(config: &'a ControllerConfig, time: NaiveDateTime) -> HomepageData<'a> {
            HomepageData {
                valves: config
                    .iter()
                    .map(|valve| ValveData::from(valve, time))
                    .collect(),
                address: &config.address,
            }
        }
    }

    pub async fn update_valve_status(
        valve_number: ValveNumber,
        config: ServerConfig,
        new_state: AutomationStatus,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut controller_config = config.write().await;
        let v = &mut controller_config
            .get_mut(valve_number)
            .ok_or_else(|| warp::reject::custom(InvalidValveNumber {}))?;
        v.automation_status = new_state.clone();
        Ok(StatusCode::OK)
    }

    pub async fn render_details(
        valve_number: ValveNumber,
        config: ServerConfig,
    ) -> Result<WithTemplate<serde_json::Value>, warp::Rejection> {
        let controller_config = config.read().await;
        let valve = &controller_config.get(valve_number);
        valve
            .map(|valve| WithTemplate {
                name: "timetable",
                value: json!(ValveData::from(valve, Local::now().naive_local())),
            })
            .ok_or_else(|| warp::reject::custom(InvalidValveNumber {}))
    }

    pub async fn create_valve(
        params: ValveParams,
        config: ServerConfig,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut controller_config = config.write().await;
        if controller_config.get(params.valve_number).is_some() {
            return Err(warp::reject::custom(InvalidValveNumber {}));
        }
        controller_config.push(Valve::new(params.name, params.valve_number));
        Ok(warp::redirect(Uri::from_static("/")))
    }

    pub async fn render_homepage(
        config: ServerConfig,
    ) -> Result<WithTemplate<serde_json::Value>, Infallible> {
        let controller_config = config.read().await;
        let controller_config = &(*controller_config);

        Ok(WithTemplate {
            name: "index",
            value: json!(HomepageData::from(
                controller_config,
                Local::now().naive_local()
            )),
        })
    }

    pub async fn delete_valve(
        valve_number: ValveNumber,
        config: ServerConfig,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut config = config.write().await;
        if !config.remove_valve(valve_number) {
            return Err(warp::reject::custom(InvalidValveNumber {}));
        }
        Ok(warp::reply())
    }
    pub async fn add_duration(
        valve_number: ValveNumber,
        config: ServerConfig,
        params: TimetableParams,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut config = config.write().await;
        let duration = Duration::new(params.start_time, params.end_time)?;
        config
            .get_mut(valve_number)
            .ok_or_else(|| warp::reject::custom(InvalidValveNumber {}))
            .and_then(|valve| {
                valve
                    .add_duration(&params.day, duration)
                    .map_err(|_| warp::reject::custom(InvalidValveNumber {}))?;
                Ok(warp::redirect(
                    Uri::try_from(format!("/valves/{}", valve_number)).unwrap(),
                ))
            })
    }
    pub async fn delete_duration(
        valve_number: ValveNumber,
        config: ServerConfig,
        params: TimetableParams,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut config = config.write().await;
        let duration = Duration::new(params.start_time, params.end_time)?;
        config
            .get_mut(valve_number)
            .ok_or_else(|| warp::reject::custom(InvalidValveNumber {}))
            .and_then(|valve| {
                valve
                    .remove_duration(&params.day, duration)
                    .map_err(|_| warp::reject::custom(InvalidValveNumber {}))?;
                Ok(warp::reply())
            })
    }
}
