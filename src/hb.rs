use serde::Serialize;
use std::sync::Arc;

use handlebars::{Handlebars, handlebars_helper};

pub struct WithTemplate<T: Serialize> {
    pub name: &'static str,
    pub value: T,
}

pub fn render<T>(template: WithTemplate<T>, hbs: Arc<Handlebars>) -> impl warp::Reply
where
    T: Serialize,
{
    let render = hbs
        .render(template.name, &template.value)
        .unwrap_or_else(|err| err.to_string());
    warp::reply::html(render)
}

pub fn init() -> Handlebars<'static> {
    let mut hb = Handlebars::new();
    handlebars_helper!(ifeq: |this:str, other:str|  this.eq(other));
    hb.register_helper("ifeq", Box::new(ifeq));
    // register the template
    hb.register_templates_directory(".hbs", "./static/templates")
        .unwrap();
    hb
}



#[cfg(test)]
mod tests {
    use super::init;
    #[test]
    fn test_helper() {
        let hb = init();
        assert_eq!(2 + 2, 4);
    }
}