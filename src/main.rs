use executor::control_valves;
use hyper::server::Server;
use listenfd::ListenFd;
use std::convert::Infallible;
use tokio::sync::RwLock;

use std::sync::Arc;

use reqwest::Url;

use warp::Filter;

mod paths;
use paths::get_dynamic_paths;

mod hb;

mod datamodel;
use datamodel::{ControllerConfig, ServerConfig, Valve};

mod executor;

use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    // Filter traces based on the RUST_LOG env var, or, if it's not set,
    // default to show the output of the example.
    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "tracing=info,warp=debug,web_server=debug".to_owned());

    // Configure the default `tracing` subscriber.
    // The `fmt` subscriber from the `tracing-subscriber` crate logs `tracing`
    // events to stdout. Other subscribers are available for integrating with
    // distributed tracing systems such as OpenTelemetry.
    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(filter)
        // Record an event when each span closes. This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let hb = hb::init();
    // Turn Handlebars instance into a Filter so we can combine it
    // easily with others...
    let hb = Arc::new(hb);
    let config = get_sample_config();
    let dynamic_paths = get_dynamic_paths(hb.clone(), config.clone());
    let static_content = warp::get()
        .and(warp::path("static"))
        .and(warp::fs::dir("./static/"));

    let routes = dynamic_paths
        .or(static_content)
        .with(warp::trace::request());
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
    let bg_task = tokio::spawn(control_valves(config.clone()));
    server.serve(make_svc).await.unwrap();
    bg_task.await.unwrap();
}

pub fn get_sample_config() -> ServerConfig {
    let url = Url::parse("https://localhost:4040").unwrap();
    let mut config: ControllerConfig = ControllerConfig::new(url);
    for valve in vec![
        Valve::new("blub", 0),
        Valve::new("test", 1),
        Valve::new("new", 2),
    ] {
        config.push(valve)
    }

    Arc::new(RwLock::new(config))
}
