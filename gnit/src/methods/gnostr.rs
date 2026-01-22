use axum::{response::{IntoResponse, Html}, http::StatusCode};
use tracing::debug;
use askama::Template;

use crate::methods::repo::Error;
use crate::methods::filters;

#[derive(Template)]
#[template(path = "gnostr.html")]
pub struct View {
    // No specific data needed for now, but can be added later
}

pub async fn handle() -> Result<Html<String>, Error> {
    debug!("Gnostr handler invoked");
    Ok(Html(View {}.render().expect("Failed to render template")))
}
