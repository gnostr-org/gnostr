use std::{collections::BTreeMap, sync::Arc};

use anyhow::Context;
use askama::Template;
use axum::{response::IntoResponse, Extension};

use super::filters;
use crate::web::{
    database::schema::repository::{Repository, YokedRepository},
    into_response,
    GLOBAL_CSS_HASH,
    GNOSTR_SVG_HASH,
    LOADER_FRAGMENT_SVG_HASH,
    LOGO_INVERTED_SVG_HASH,
    LOGO_SVG_HASH,
    HOME_SVG_HASH,
    HOME_ACTIVE_SVG_HASH,
    MESSAGES_SVG_HASH,
    MESSAGES_ACTIVE_SVG_HASH,
    NOTIFICATIONS_SVG_HASH,
    NOTIFICATIONS_ACTIVE_SVG_HASH,
    SETTINGS_SVG_HASH,
    SETTINGS_ACTIVE_SVG_HASH,
    NEW_NOTE_SVG_HASH,
    NO_USER_SVG_HASH,
    PROFILE_WEBSITE_SVG_HASH,
    PROFILE_ZAP_SVG_HASH,
    MESSAGE_USER_SVG_HASH,
    PUBKEY_SVG_HASH,
    ADD_RELAY_SVG_HASH,
    CLOSE_MODAL_SVG_HASH,
    CRATE_VERSION,
    EVENT_LIKE_SVG_HASH,
    EVENT_LIKED_SVG_HASH,
    EVENT_DELETE_SVG_HASH,
    EVENT_REPLY_SVG_HASH,
    EVENT_SHARE_SVG_HASH,
    EVENT_OPTIONS_SVG_HASH,
    GNOSTR_NOTIF_SVG_HASH,
    JS_BUNDLE_HASH,
    layers::logger,
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct View {
    pub repositories: BTreeMap<Option<String>, Vec<YokedRepository>>,
}

pub async fn handle(
    Extension(db): Extension<Arc<rocksdb::DB>>,
) -> Result<impl IntoResponse, super::repo::Error> {
    let mut repositories: BTreeMap<Option<String>, Vec<YokedRepository>> = BTreeMap::new();

    let fetched = tokio::task::spawn_blocking(move || Repository::fetch_all(&db))
        .await
        .context("Failed to join Tokio task")??;

    for (k, v) in fetched {
        // TODO: fixme
        let mut split: Vec<_> = k.split('/').collect();
        split.pop();
        let key = Some(split.join("/")).filter(|v| !v.is_empty());

        let k = repositories.entry(key).or_default();
        k.push(v);
    }

    Ok(into_response(View { repositories }))
}

// SPA handler - serves the main template for client-side routing

pub async fn handle_spa() -> impl IntoResponse {
    // For SPA routes, we don't need repositories data, just serve the template
    let empty_repositories: BTreeMap<
        Option<String>,
        Vec<crate::web::database::schema::repository::YokedRepository>,
    > = BTreeMap::new();
    into_response(View {
        repositories: empty_repositories,
    })
}
