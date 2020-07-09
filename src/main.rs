use std::sync::Arc;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::json;
use warp::Filter;

mod datamodel;
use datamodel::Valve;
static INDEX_HBS: &'static str = include_str!("templates/index.hbs");

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
    let template = INDEX_HBS;

    let mut hb = Handlebars::new();
    // register the template
    hb.register_template_string("index.html", template)
        .unwrap();

    // Turn Handlebars instance into a Filter so we can combine it
    // easily with others...
    let hb = Arc::new(hb);

    // Create a reusable closure to render template
    let handlebars = move |with_template| render(with_template, hb.clone());

    //GET /
    let root = warp::path::end()
        .map(|| {
            let test = vec!{Valve::new("blub", 0)};
            let test = json!({"Valves":test});
            println!("{}",test);
            WithTemplate {
                name: "index.html",
                value: test,
            }
        }
           )
        .map(handlebars);

    let static_content = warp::path("static").and(warp::fs::dir("./static/"));


    let route = warp::get().and(root.or(static_content));
    warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
}