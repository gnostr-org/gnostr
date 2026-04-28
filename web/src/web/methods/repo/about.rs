use std::sync::Arc;

use askama::Template;
use axum::{extract::Query, response::IntoResponse, Extension};
use serde::Deserialize;

use crate::web::{
    git::ReadmeFormat,
    into_response,
    methods::{
        filters,
        repo::{Repository, RepositoryPath, Result},
    },
    Git,
};

#[derive(Deserialize)]
pub struct UriQuery {
    #[serde(rename = "h")]
    pub branch: Option<Arc<str>>,
}

#[derive(Template)]
#[template(path = "repo/about.html")]
pub struct View {
    repo: Repository,
    readme: Option<(ReadmeFormat, Arc<str>)>,
    branch: Option<Arc<str>>,
    pub _highlight_css_hash: &'static str,
    pub _dark_highlight_css_hash: &'static str,
}

pub async fn handle(
    Extension(repo): Extension<Repository>,
    Extension(RepositoryPath(repository_path)): Extension<RepositoryPath>,
    Extension(git): Extension<Arc<Git>>,
    Query(query): Query<UriQuery>,
) -> Result<impl IntoResponse> {
    let open_repo = git
        .clone()
        .repo(repository_path, query.branch.clone())
        .await?;
    let readme = open_repo.readme().await?;

    Ok(into_response(View {
        repo,
        readme,
        branch: query.branch,
        _highlight_css_hash: crate::web::HIGHLIGHT_CSS_HASH.get().unwrap(),
        _dark_highlight_css_hash: crate::web::DARK_HIGHLIGHT_CSS_HASH.get().unwrap(),
    }))
}
