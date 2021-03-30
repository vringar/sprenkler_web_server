use serde::Serialize;
use std::{convert::Infallible, sync::Arc};

use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError, Renderable};

pub struct WithTemplate<T: Serialize> {
    pub name: &'static str,
    pub value: T,
}

pub fn ifeq_helper<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    registery: &'reg Handlebars<'reg>,
    context: &'rc Context,
    render_context: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let param1 = h
        .param(0)
        .ok_or(RenderError::new("Param 0 is required for ifeq helper."))?;
    let param2 = h
        .param(1)
        .ok_or(RenderError::new("Param 1 is required for ifeq helper."))?;
    let param1 = param1.render();
    let param2 = param2.render();
    if param1 == param2 {
        h.template()
            .map(|t| t.render(registery, context, render_context, out))
            .ok_or(RenderError::new("ifeq helper failed to render template"))??;
    }
    Ok(())
}
pub async fn render<'reg, T>(template: WithTemplate<T>, hbs: Arc<Handlebars<'reg>>) ->  Result<impl warp::Reply, Infallible>
where
    T: Serialize,
{
    let render = hbs
        .render(template.name, &template.value)
        .unwrap_or_else(|err| err.to_string());
    Ok(warp::reply::html(render))
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
        let _hb = init();
    }
}
