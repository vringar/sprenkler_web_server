use hyper::server::Server;
use listenfd::ListenFd;
use std::convert::Infallible;

use std::sync::Arc;

use reqwest::Url;

use handlebars::Handlebars;

use serde_json::json;
use warp::Filter;

mod valves;
use valves::get_valve_paths;

mod hb;
use hb::render;
use hb::WithTemplate;

mod datamodel;
use datamodel::{Config, Valve};

#[tokio::main]
async fn main() {
    let mut hb = Handlebars::new();
    // register the template
    hb.register_templates_directory(".hbs", "./static/templates")
        .unwrap();

    // Turn Handlebars instance into a Filter so we can combine it
    // easily with others...
    let hb = Arc::new(hb);

    // Create a reusable closure to render template
    let handlebars = {
        let hb = hb.clone();
        move |with_template| render(with_template, hb.clone())
    };

    let config = get_sample_config();
    //GET /
    let root = {
        let config = config.clone();
        warp::path::end()
            .map(move || WithTemplate {
                name: "index",
                value: json!(*config),
            })
            .map(handlebars.clone())
    };
    let valve_paths = get_valve_paths(hb.clone(), config.clone());
    let static_content = warp::path("static").and(warp::fs::dir("./static/"));

    let routes = warp::get().and(root.or(static_content).or(valve_paths));
    // hyper let's us build a server from a TcpListener (which will be
    // useful shortly). Thus, we'll need to convert our `warp::Filter` into
    // a `hyper::service::MakeService` for use with a `hyper::server::Server`.
    let svc = warp::service(routes);

    let make_svc = hyper::service::make_service_fn(|_: _| {
        // the clone is there because not all warp filters impl Copy
        let svc = svc.clone();
        async move { Ok::<_, Infallible>(svc) }
    });

    let mut listenfd = ListenFd::from_env();
    // if listenfd doesn't take a TcpListener (i.e. we're not running via
    // the command above), we fall back to explicitly binding to a given
    // host:port.
    let server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        Server::from_tcp(l).unwrap()
    } else {
        Server::bind(&([127, 0, 0, 1], 3030).into())
    };

    server.serve(make_svc).await.unwrap();
}

pub fn get_sample_config() -> Arc<Config> {
    let url = Url::parse("https://localhost:4040").unwrap();
    let config: Config = Config {
        adress: url,
        valves: vec![Valve::new("blub", 0)],
    };
    Arc::new(config)
}
