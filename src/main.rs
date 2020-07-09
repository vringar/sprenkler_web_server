use std::sync::Arc;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::json;
use warp::Filter;

use reqwest::Url;

mod datamodel;
use datamodel::Config;
use datamodel::Valve;

struct WithTemplate<T: Serialize> {
    name: &'static str,
    value: T,
}

fn render<T>(template: WithTemplate<T>, hbs: Arc<Handlebars>) -> impl warp::Reply
where
    T: Serialize,
{
    let render = hbs
        .render(template.name, &template.value)
        .unwrap_or_else(|err| err.to_string());
    warp::reply::html(render)
}

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
    let root = warp::path::end()
        .map(|| WithTemplate {
            name: "index",
            value: json!({ "Valves": vec![Valve::new("blub", 0)] }),
        })
        .map(handlebars.clone());

    let static_content = warp::path("static").and(warp::fs::dir("./static/"));

    let valve = warp::path("valves")
        .and(warp::path::param())
        .and(warp::path::end())
        .map(|_: i16| WithTemplate {
            name: "details",
            value: json!(Valve::new("blub", 0)),
        })
        .map(handlebars.clone());

    let route = warp::get().and(root.or(static_content).or(valve));
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}
