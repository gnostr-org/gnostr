use std::{fmt::Write, sync::Arc};

use askama::Template;
use axum::{
    extract::Query,
    http,
    http::HeaderValue,
    response::{IntoResponse, Response},
    Extension,
};
use bytes::{BufMut, BytesMut};
use clap::crate_version;
use time::format_description::well_known::Rfc2822;

use crate::{
    git::Commit,
    into_response,
    methods::{
        filters,
        repo::{commit::UriQuery, Repository, RepositoryPath, Result},
    },
    Git,
};

#[derive(Template)]
#[template(path = "repo/diff.html")]
pub struct View {
    pub repo: Repository,
    pub commit: Arc<Commit>,
    pub branch: Option<Arc<str>>,
    pub _highlight_css_hash: &'static str,
    pub _dark_highlight_css_hash: &'static str,
}

pub async fn handle(
    Extension(repo): Extension<Repository>,
    Extension(RepositoryPath(repository_path)): Extension<RepositoryPath>,
    Extension(git): Extension<Arc<Git>>,
    Query(query): Query<UriQuery>,
) -> Result<impl IntoResponse> {
    let open_repo = git.repo(repository_path, query.branch.clone()).await?;
    let commit = if let Some(commit) = query.id {
        open_repo.commit(&commit, true).await?
    } else {
        Arc::new(open_repo.latest_commit(true).await?)
    };

    Ok(into_response(View {
        repo,
        commit,
        branch: query.branch,
        _highlight_css_hash: crate::HIGHLIGHT_CSS_HASH.get().unwrap(),
        _dark_highlight_css_hash: crate::DARK_HIGHLIGHT_CSS_HASH.get().unwrap(),
    }))
}

pub async fn handle_plain(
    Extension(RepositoryPath(repository_path)): Extension<RepositoryPath>,
    Extension(git): Extension<Arc<Git>>,
    Query(query): Query<UriQuery>,
) -> Result<Response> {
    let open_repo = git.repo(repository_path, query.branch).await?;
    let commit = if let Some(commit) = query.id {
        open_repo.commit(&commit, false).await?
    } else {
        Arc::new(open_repo.latest_commit(false).await?)
    };

    let headers = [(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain"),
    )];

    let mut data = BytesMut::new();

    writeln!(data, "From {} Mon Sep 17 00:00:00 2001", commit.oid()).unwrap();
    writeln!(
        data,
        "From: {} <{}>",
        commit.author().name(),
        commit.author().email()
    )
    .unwrap();

    write!(data, "Date: ").unwrap();
    let mut writer = data.writer();
    commit
        .author()
        .time()
        .format_into(&mut writer, &Rfc2822)
        .unwrap();
    let mut data = writer.into_inner();
    writeln!(data).unwrap();

    writeln!(data, "Subject: [PATCH] {}\n", commit.summary()).unwrap();

    write!(data, "{}", commit.body()).unwrap();

    writeln!(data, "---").unwrap();

    data.extend_from_slice(commit.diff_stats.as_bytes());
    data.extend_from_slice(b"\n");
    data.extend_from_slice(commit.diff.as_bytes());

    writeln!(data, "--\nrgit {}", crate_version!()).unwrap();

    Ok((headers, data.freeze()).into_response())
}
