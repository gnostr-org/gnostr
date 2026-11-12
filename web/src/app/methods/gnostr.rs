use axum::response::{IntoResponse, Redirect};

#[allow(unused_imports)]
use crate::app::methods::filters;
use crate::app::methods::repo::Error;
use askama::Template;
use tracing::debug;

#[derive(Template)]
#[template(path = "gnostr.html")]
pub struct View {
    // No specific data needed for now, but can be added later
}

//pub async fn handle() -> Result<Html<String>, Error> {
//TODO debug!("Gnostr handler invoked");
//TODO Ok(Html(View {}.render().expect("Failed to render template")))
pub async fn handle() -> Result<impl IntoResponse, Error> {
    debug!("Gnostr handler invoked, redirecting to root");
    Ok(Redirect::permanent("/"))
}

pub async fn handle_slash() -> Result<impl IntoResponse, Error> {
    debug!("Gnostr handler with trailing slash invoked, redirecting to root");
    Ok(Redirect::permanent("/"))
}
