use tracing::debug;

mod about;
mod commit;
mod diff;
mod log;
mod refs;
mod smart_git;
mod snapshot;
mod summary;
mod tag;
mod tree;

use std::{
    collections::BTreeMap,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    body::Body,
    handler::HandlerWithoutStateExt,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use path_clean::PathClean;
use tower::{util::BoxCloneService, Service};

use self::{
    about::handle as handle_about,
    commit::handle as handle_commit,
    diff::{handle as handle_diff, handle_plain as handle_patch},
    log::handle as handle_log,
    refs::handle as handle_refs,
    smart_git::handle as handle_smart_git,
    snapshot::handle as handle_snapshot,
    summary::handle as handle_summary,
    tag::handle as handle_tag,
    tree::handle as handle_tree,
};
use crate::database::schema::tag::YokedString;
use crate::{
    database::schema::{commit::YokedCommit, tag::YokedTag},
    layers::UnwrapInfallible,
};

pub const DEFAULT_BRANCHES: [&str; 2] = ["refs/heads/master", "refs/heads/main"];

// this is some wicked, wicked abuse of axum right here...
#[allow(clippy::trait_duplication_in_bounds)] // clippy seems a bit.. lost
pub async fn service(mut request: Request<Body>) -> Response {
    let scan_path = request
        .extensions()
        .get::<Arc<PathBuf>>()
        .expect("scan_path missing");

    let uri_segments: Vec<&str> = request
        .uri()
        .path()
        .trim_start_matches('/')
        .trim_end_matches('/')
        .split('/')
        .collect();

    debug!("URI Segments: {:?}", uri_segments);

    let mut repository_name = PathBuf::new();
    let mut handler_segment = None;
    let mut child_path_segments = Vec::new();

    let db = request
        .extensions()
        .get::<Arc<rocksdb::DB>>()
        .expect("db extension missing");

    // Check if root path should be treated as a repository
    let root_repo_path = scan_path.join(".git");
    let is_root_repo = root_repo_path.is_dir() || root_repo_path.is_file();
    let root_repo_exists_in_db =
        crate::database::schema::repository::Repository::exists(db, &PathBuf::from(""))
            .unwrap_or_default();

    // Try to find the repository name and handler segment
    let mut current_segment_index = 0;

    // Handle root repository case (URI segments could be ["summary"], ["about"], etc. for root repo)
    if is_root_repo && root_repo_exists_in_db {
        repository_name = PathBuf::new();
        handler_segment = uri_segments.get(0).copied();
        child_path_segments = if uri_segments.len() > 1 {
            uri_segments[1..].to_vec()
        } else {
            Vec::new()
        };
    }

    // If not root repository, continue with normal detection
    if repository_name.as_os_str().is_empty() {
        while current_segment_index < uri_segments.len() {
            let potential_repo_name_segments = &uri_segments[0..=current_segment_index];
            debug!("Looping URI Segments: {:?}", potential_repo_name_segments);
            let potential_repo_name = potential_repo_name_segments
                .iter()
                .collect::<PathBuf>()
                .clean();
            debug!("Potential Repo Name: {}", potential_repo_name.display());
            let full_potential_repo_path = scan_path.join(&potential_repo_name);
            debug!(
                "Full Potential Repo Path: {}",
                full_potential_repo_path.display()
            );

            //We detect repo types
            let is_bare_repo = full_potential_repo_path.join("HEAD").is_file() //<repo>.git/HEAD
            && full_potential_repo_path.join("objects").is_dir(); //<repo>.git/objects/
            let is_working_tree = full_potential_repo_path.join("/.git").is_file(); //<repo>/.git
            let is_working_tree_repo = full_potential_repo_path.join(".git").is_dir();
            let exists_in_db =
                crate::database::schema::repository::Repository::exists(db, &potential_repo_name)
                    .unwrap_or_default();
            debug!(
                "  Is Bare: {}, Is Working Tree: {}, Is Working Tree Repo:{}, Exists in DB: {}",
                is_bare_repo, is_working_tree, is_working_tree_repo, exists_in_db
            );

            // Only consider it a repository if it exists on disk *and* is in the database
            if (is_bare_repo || is_working_tree || is_working_tree_repo) && exists_in_db {
                repository_name = potential_repo_name;

                // If it's a working tree repo, but the URL *includes* .git (e.g., /repo/.git/tree)
                // we should treat the part before .git as the repository_name
                if (is_working_tree || is_working_tree_repo)
                    && current_segment_index + 1 < uri_segments.len()
                    && uri_segments[current_segment_index + 1] == ".git"
                {
                    // Adjust segments to skip ".git"
                    current_segment_index += 1; // Skip the .git segment
                }

                if current_segment_index + 1 < uri_segments.len() {
                    handler_segment = Some(uri_segments[current_segment_index + 1]);
                    child_path_segments = uri_segments[current_segment_index + 2..].to_vec();
                }
                break;
            }
            current_segment_index += 1;
        }
    }

    debug!("Repository Name: {}", repository_name.display());
    debug!("Handler Segment: {:?}", handler_segment);
    debug!("Child Path Segments: {:?}", child_path_segments);

    if repository_name.as_os_str().is_empty() && !(is_root_repo && root_repo_exists_in_db) {
        return RepositoryNotFound.into_response();
    }

    let mut child_path = None;
    macro_rules! h {
        ($handler:ident) => {
            BoxCloneService::new($handler.into_service())
        };
    }

    let mut service = match handler_segment {
        Some("about") => h!(handle_about),
        Some("refs") if child_path_segments.last() == Some(&"info") => {
            h!(handle_smart_git)
        }
        Some("git-upload-pack") => h!(handle_smart_git),
        Some("refs") => h!(handle_refs),
        Some("log") => h!(handle_log),
        Some("tree") => {
            if child_path_segments.is_empty() {
                child_path = None;
            } else {
                child_path = Some(child_path_segments.into_iter().collect::<PathBuf>().clean());
            }
            h!(handle_tree)
        }
        Some("commit") => h!(handle_commit),
        Some("diff") => h!(handle_diff),
        Some("patch") => h!(handle_patch),
        Some("tag") => h!(handle_tag),
        Some("snapshot") => h!(handle_snapshot),
        _ => h!(handle_summary), // Default to summary if no specific handler is found
    };

    debug!("Final Child Path: {:?}", child_path);

    let repository_abs_path = if repository_name.as_os_str().is_empty() {
        // Root repository - use .git directory
        scan_path.join(".git")
    } else {
        // Check if this is a working tree repo and use .git subdirectory
        let repo_path = scan_path.join(&repository_name);
        if repo_path.join(".git").is_dir() {
            repo_path.join(".git")
        } else {
            repo_path
        }
    };

    debug!("Repository Name: {}", repository_name.display());
    debug!("Repository Path: {}", repository_abs_path.display());

    request.extensions_mut().insert(ChildPath(child_path));
    request.extensions_mut().insert(Repository(repository_name));
    request
        .extensions_mut()
        .insert(RepositoryPath(repository_abs_path));

    service
        .call(request)
        .await
        .unwrap_infallible()
        .into_response()
}

#[derive(Clone)]
pub struct Repository(pub PathBuf);

impl Deref for Repository {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct RepositoryPath(pub PathBuf);

#[derive(Clone)]
pub struct ChildPath(pub Option<PathBuf>);

impl Deref for RepositoryPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct RepositoryNotFound;

impl IntoResponse for RepositoryNotFound {
    fn into_response(self) -> Response {
        (StatusCode::NOT_FOUND, "Repository not found").into_response()
    }
}

pub struct Error(anyhow::Error);

impl From<Arc<anyhow::Error>> for Error {
    fn from(e: Arc<anyhow::Error>) -> Self {
        Self(anyhow::Error::msg(format!("{e:?}")))
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", self.0)).into_response()
    }
}

pub struct Refs {
    heads: BTreeMap<String, YokedCommit>,
    tags: Vec<(YokedString, YokedTag)>,
}
