use std::{collections::BTreeMap, sync::Arc};

use anyhow::Context;
use askama::Template;
use axum::{response::IntoResponse, Extension};

use super::filters;
use crate::app::{
    database::schema::repository::{Repository, YokedRepository},
    into_response,
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct View {
    pub repositories: Vec<(Option<String>, Vec<YokedRepository>)>,
    pub base_href: String,
    pub title: String,
    pub segments: Vec<(String, String)>,
}

pub async fn handle(
    Extension(db): Extension<Arc<rocksdb::DB>>,
) -> Result<impl IntoResponse, super::repo::Error> {
    let mut grouped: BTreeMap<Option<String>, Vec<YokedRepository>> = BTreeMap::new();

    let fetched = tokio::task::spawn_blocking(move || Repository::fetch_all(&db))
        .await
        .context("Failed to join Tokio task")??;

    for (k, v) in fetched {
        // TODO: fixme
        if k == "." {
            grouped.entry(Some(".".to_string())).or_default().push(v);
            continue;
        }

        let mut split: Vec<_> = k.split('/').collect();
        split.pop();
        let key = Some(split.join("/")).filter(|v| !v.is_empty());

        let k = grouped.entry(key).or_default();
        k.push(v);
    }

    // Surface the repository root first, then the remaining groups in sorted order.
    let mut repositories: Vec<(Option<String>, Vec<YokedRepository>)> = Vec::new();
    if let Some(root) = grouped.remove(&Some(".".to_string())) {
        repositories.push((Some(".".to_string()), root));
    }
    for (key, repos) in grouped {
        repositories.push((key, repos));
    }

    Ok(into_response(View {
        repositories,
        base_href: "/".to_string(),
        title: "gnostr.org".to_string(),
        segments: Vec::new(),
    }))
}

/// Render a repo listing scoped to a directory prefix (e.g. `/xcode`).
pub async fn render_prefix(
    db: Arc<rocksdb::DB>,
    prefix: &str,
) -> Result<axum::response::Response, super::repo::Error> {
    let prefix = prefix.trim_matches('/');
    if prefix.is_empty() {
        return handle(Extension(db)).await.map(|r| r.into_response());
    }

    let mut grouped: BTreeMap<Option<String>, Vec<YokedRepository>> = BTreeMap::new();
    let fetched = tokio::task::spawn_blocking(move || Repository::fetch_all(&db))
        .await
        .context("Failed to join Tokio task")??;

    let prefix_with_slash = format!("{prefix}/");

    for (k, v) in fetched {
        if k == "." {
            continue;
        }
        let Some(relative) = k.strip_prefix(&prefix_with_slash) else {
            continue;
        };
        let mut split: Vec<_> = relative.split('/').collect();
        split.pop();
        let key = Some(split.join("/")).filter(|v| !v.is_empty());

        grouped.entry(key).or_default().push(v);
    }

    let repositories: Vec<(Option<String>, Vec<YokedRepository>)> = grouped.into_iter().collect();

    if repositories.is_empty() {
        return Ok(super::repo::RepositoryNotFound.into_response());
    }

    let segments = path_segments(prefix);

    Ok(into_response(View {
        repositories,
        base_href: format!("/{prefix}/"),
        title: format!("gnostr.org/{prefix}"),
        segments,
    })
    .into_response())
}

fn path_segments(prefix: &str) -> Vec<(String, String)> {
    let mut segments = Vec::new();
    let mut href = String::new();
    for component in prefix.split('/') {
        if component.is_empty() {
            continue;
        }
        if !href.is_empty() {
            href.push('/');
        }
        href.push_str(component);
        segments.push((component.to_string(), format!("/{href}")));
    }
    segments
}

// SPA handler - serves the main template for client-side routing

pub async fn handle_spa() -> impl IntoResponse {
    // For SPA routes, we don't need repositories data, just serve the template
    let empty_repositories: Vec<(Option<String>, Vec<crate::app::database::schema::repository::YokedRepository>)> =
        Vec::new();
    into_response(View {
        repositories: empty_repositories,
        base_href: "/".to_string(),
        title: "gnostr.org".to_string(),
        segments: Vec::new(),
    })
}
