use serde::Serialize;
use std::sync::Arc;

use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
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
