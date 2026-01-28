use std::sync::Arc;

use askama::Template;
use axum::{extract::Query, response::IntoResponse, Extension};
use serde::Deserialize;

use crate::methods::filters;
use crate::{
    git::{Commit, OpenRepository},
    into_response, Git,
};

use crate::methods::repo::Error;
use crate::methods::repo::{Repository, RepositoryPath};

#[derive(Template)]
#[template(path = "repo/commit.html")]
pub struct View {
    pub repo: Repository,
    pub commit: Arc<Commit>,
    pub branch: Option<Arc<str>>,
    pub dl_branch: Arc<str>,
    pub id: Option<String>,
    pub _highlight_css_hash: &'static str,
    pub _dark_highlight_css_hash: &'static str,
}

#[derive(Deserialize)]
pub struct UriQuery {
    pub id: Option<String>,
    #[serde(rename = "h")]
    pub branch: Option<Arc<str>>,
}

pub async fn handle(
    Extension(repo): Extension<Repository>,
    Extension(RepositoryPath(repository_path)): Extension<RepositoryPath>,
    Extension(git): Extension<Arc<Git>>,
    Query(query): Query<UriQuery>,
) -> Result<impl IntoResponse, Error> {
    let open_repo = git.repo(repository_path, query.branch.clone()).await?;

    let (dl_branch, commit) = tokio::try_join!(
        fetch_dl_branch(query.branch.clone(), open_repo.clone()),
        fetch_commit(query.id.as_deref(), open_repo),
    )?;

    Ok(into_response(View {
        repo,
        commit,
        branch: query.branch,
        id: query.id,
        dl_branch,
        _highlight_css_hash: crate::HIGHLIGHT_CSS_HASH.get().unwrap(),
        _dark_highlight_css_hash: crate::DARK_HIGHLIGHT_CSS_HASH.get().unwrap(),
    }))
}

async fn fetch_commit(
    commit_id: Option<&str>,
    open_repo: Arc<OpenRepository>,
) -> Result<Arc<Commit>, Error> {
    Ok(if let Some(commit) = commit_id {
        open_repo.commit(commit, true).await?
    } else {
        Arc::new(open_repo.latest_commit(true).await?)
    })
}

async fn fetch_dl_branch(
    branch: Option<Arc<str>>,
    open_repo: Arc<OpenRepository>,
) -> Result<Arc<str>, Error> {
    if let Some(branch) = branch.clone() {
        Ok(branch)
    } else {
        Ok(Arc::from(
            open_repo
                .clone()
                .default_branch()
                .await?
                .unwrap_or_else(|| "master".to_string()),
        ))
    }
}
