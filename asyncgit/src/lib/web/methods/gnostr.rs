#[allow(unused_imports)] //TODO
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};

use askama::Template;
use tracing::debug;

use crate::web::methods::filters;
#[allow(unused_imports)] //TODO
use crate::web::methods::repo::Error;

#[derive(Template)]
#[template(path = "gnostr.html")]
pub struct View {
    // No specific data needed for now, but can be added later
}

//pub async fn handle() -> Result<Html<String>, Error> {
//TODO debug!("Gnostr handler invoked");
//TODO Ok(Html(View {}.render().expect("Failed to render template")))
pub async fn handle() -> Result<impl IntoResponse, crate::web::methods::repo::Error> {
    debug!("Gnostr handler invoked, redirecting to root");
    Ok(Redirect::permanent("/"))
}

pub async fn handle_slash() -> Result<impl IntoResponse, crate::web::methods::repo::Error> {
    debug!("Gnostr handler with trailing slash invoked, redirecting to root");
    Ok(Redirect::permanent("/"))
}
