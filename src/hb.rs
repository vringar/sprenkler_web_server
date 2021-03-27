use serde::Serialize;
use std::sync::Arc;

use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use handlebars::{handlebars_helper};

pub struct WithTemplate<T: Serialize> {
    pub name: &'static str,
    pub value: T,
}


pub fn ifeq_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    if !h.is_block() {
       return Err(RenderError::new("ifeq needs to be a block helper"));
    }
    let param1 =  h.param(0).ok_or(RenderError::new("Param 0 is required for ifeq helper."))?;
    let param2 = h.param(1).ok_or(RenderError::new("Param 1 is required for ifeq helper."))?;
    if param1.render() == param2.render() {
        out.write(h.block_param().unwrap())?;
    }
    Ok(())
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
    hb.register_helper("ifeq", Box::new(ifeq_helper));
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