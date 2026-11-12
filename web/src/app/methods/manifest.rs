use std::sync::{Mutex, OnceLock};

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManifestEntry {
    pub repo: String,
    pub cid: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    pub repos: Vec<ManifestEntry>,
}

static MANIFEST: OnceLock<Mutex<Option<Manifest>>> = OnceLock::new();

fn manifest_store() -> &'static Mutex<Option<Manifest>> {
    MANIFEST.get_or_init(|| Mutex::new(None))
}

pub async fn get_manifest() -> impl IntoResponse {
    let guard = manifest_store().lock().unwrap();
    match guard.as_ref() {
        Some(m) => Json(m.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, "no manifest posted yet").into_response(),
    }
}

pub async fn post_manifest(Json(payload): Json<Manifest>) -> impl IntoResponse {
    *manifest_store().lock().unwrap() = Some(payload);
    StatusCode::OK
}
