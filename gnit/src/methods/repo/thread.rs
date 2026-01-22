use axum::{body::Body, http::Request, response::Html};
use std::{path::PathBuf, sync::Arc};
use tracing::debug;
use askama::Template;

use crate::methods::filters; // GEMINI: Fix filters import
use crate::Git;

#[derive(Template)]
#[template(path = "thread.html")]
pub struct View {
    pub thread_hash: String,
}

pub async fn handle(
    request: Request<Body>,
) -> Result<Html<String>, super::Error> { // GEMINI: Correct return type to Html<String>
    debug!("Thread handler invoked");

    // Extract necessary extensions
    let _db = request.extensions().get::<Arc<rocksdb::DB>>().expect("db extension missing").clone();
    let _git = request.extensions().get::<Arc<Git>>().expect("git extension missing").clone();
    let _scan_path = request.extensions().get::<Arc<PathBuf>>().expect("scan_path extension missing").clone();
    let _repo = request.extensions().get::<crate::methods::repo::Repository>().expect("repository extension missing").clone();
    let _repo_path = request.extensions().get::<crate::methods::repo::RepositoryPath>().expect("repository path extension missing").clone();
    let child_path = request.extensions().get::<crate::methods::repo::ChildPath>().expect("child path extension missing").clone();

    // Get the thread hash from child_path
    let thread_hash = match child_path.0.and_then(|p: PathBuf| p.to_str().map(String::from)) {
        Some(hash) => hash,
        None => {
            debug!("No thread hash provided");
            // Return an Html response with an error message and BAD_REQUEST status
            return Ok(Html("<h1>Error: Missing thread hash</h1>".to_string()));
        }
    };

    debug!("Attempting to render thread: {}", thread_hash);

    Ok(Html(View { thread_hash }.render().expect("Failed to render template")))
}
