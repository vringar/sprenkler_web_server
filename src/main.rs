use std::sync::Arc;

use handlebars::Handlebars;

use serde_json::json;
use warp::Filter;

use reqwest::Url;

mod datamodel;
use datamodel::Config;
use datamodel::Valve;

mod valves;
use valves::get_valve_paths;

mod hb;
use hb::render;
use hb::WithTemplate;

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

    let url = Url::parse("https://localhost:4040").unwrap();
    let config: Config = Config {
        adress: url,
        valves: vec![Valve::new("blub", 0)],
    };
    let config = Arc::new(config);
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

    let route = warp::get().and(root.or(static_content).or(valve_paths));
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}
