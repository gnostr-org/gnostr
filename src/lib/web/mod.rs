pub mod database;
pub mod git;
pub mod layers;
pub mod methods;
pub mod syntax_highlight;
pub mod theme;
pub mod unified_diff_builder;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub fn into_response<T: Template>(template: T) -> impl IntoResponse {
    let render_result = template.render();
    match render_result {
        Ok(html) => html.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render template: {}", err),
        )
            .into_response(),
    }
}

#[derive(Clone)]
pub enum ResponseEither<A, B> {
    Left(A),
    Right(B),
}

impl<A: IntoResponse, B: IntoResponse> IntoResponse for ResponseEither<A, B> {
    fn into_response(self) -> Response {
        match self {
            ResponseEither::Left(a) => a.into_response(),
            ResponseEither::Right(b) => b.into_response(),
        }
    }
}
