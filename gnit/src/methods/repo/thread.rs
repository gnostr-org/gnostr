use axum::{body::Body, extract::Extension, http::Request, response::{Response, IntoResponse}};
use std::{path::PathBuf, sync::Arc};
use tracing::debug;

use crate::{database::schema::repository::YokedRepository, git::Git};

pub async fn handle(
    request: Request<Body>,
) -> Response {
    debug!("Thread handler invoked");

    // Extract necessary extensions
    let db = request.extensions().get::<Arc<rocksdb::DB>>().expect("db extension missing").clone();
    let git = request.extensions().get::<Arc<Git>>().expect("git extension missing").clone();
    let scan_path = request.extensions().get::<Arc<PathBuf>>().expect("scan_path extension missing").clone();
    let repo = request.extensions().get::<crate::methods::repo::Repository>().expect("repository extension missing").clone();
    let repo_path = request.extensions().get::<crate::methods::repo::RepositoryPath>().expect("repository path extension missing").clone();
    let child_path = request.extensions().get::<crate::methods::repo::ChildPath>().expect("child path extension missing").clone();

    // Get the thread hash from child_path
    let thread_hash = match child_path.0.and_then(|p: PathBuf| p.to_str().map(String::from)) {
        Some(hash) => hash,
        None => {
            debug!("No thread hash provided");
            return (axum::http::StatusCode::BAD_REQUEST, "Missing thread hash").into_response();
        }
    };

    // TODO: Implement actual logic to fetch and render the thread page using the thread_hash, db, git, etc.
    debug!("Attempting to render thread: {}", thread_hash);

    (axum::http::StatusCode::OK, format!("Thread page for: {}", thread_hash)).into_response()
}
