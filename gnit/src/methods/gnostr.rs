#[allow(unused_imports)] //TODO
use axum::{response::{IntoResponse, Html, Redirect}, http::StatusCode};

use tracing::debug;
use askama::Template;

#[allow(unused_imports)] //TODO
use crate::methods::repo::Error;
use crate::methods::filters;

#[derive(Template)]
#[template(path = "gnostr.html")]
pub struct View {
    // No specific data needed for now, but can be added later
}

//pub async fn handle() -> Result<Html<String>, Error> {
    //TODO debug!("Gnostr handler invoked");
    //TODO Ok(Html(View {}.render().expect("Failed to render template")))
pub async fn handle() -> Result<impl IntoResponse, crate::methods::repo::Error> {

    debug!("Gnostr handler invoked, redirecting to root");
    Ok(Redirect::permanent("/"))
}
