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
mod thread; // GEMINI: Add thread module
mod tree;

use crate::app::database::schema::repository::YokedRepository;

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
    thread::handle as handle_thread, // GEMINI: Import thread handler
    tree::handle as handle_tree,
};
use crate::app::database::schema::tag::YokedString;
use crate::app::{
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
        .expect("scan_path missing")
        .clone();

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

    let root_repo_exists_in_db =
        crate::app::database::schema::repository::Repository::exists(db, &PathBuf::from("."))
            .unwrap_or_default();

    // Try to find the repository name and handler segment
    let mut current_segment_index = 0;

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
        let is_working_tree = full_potential_repo_path.join(".git").is_file(); //<repo>/.git
        let is_working_tree_repo = full_potential_repo_path.join(".git").is_dir();
        let exists_in_db = crate::app::database::schema::repository::Repository::exists(
            db,
            &potential_repo_name,
        )
        .unwrap_or_default();
        debug!(
            "  Is Bare: {}, Is Working Tree: {}, Is Working Tree Repo:{}, Exists in DB: {}",
            is_bare_repo, is_working_tree, is_working_tree_repo, exists_in_db
        );

        // Only consider it a repository if it exists on disk *and* is in the database.
        // We keep scanning so that a nested repository (e.g. a SwiftPM checkout inside
        // another repo) is chosen over its parent.
        if (is_bare_repo || is_working_tree || is_working_tree_repo) && exists_in_db {
            repository_name = potential_repo_name;

            // If it's a working tree repo, but the URL *includes* .git (e.g. /repo/.git/tree)
            // we should treat the part before .git as the repository_name.
            let mut next_index = current_segment_index + 1;
            if (is_working_tree || is_working_tree_repo)
                && next_index < uri_segments.len()
                && uri_segments[next_index] == ".git"
            {
                // Adjust segments to skip ".git"
                next_index += 1;
            }

            handler_segment = None;
            child_path_segments = Vec::new();
            if next_index < uri_segments.len() {
                handler_segment = Some(uri_segments[next_index]);
                child_path_segments = uri_segments[next_index + 1..].to_vec();
            }
        }
        current_segment_index += 1;
    }

    if repository_name.as_os_str().is_empty() && root_repo_exists_in_db {
        repository_name = PathBuf::from(".");
        handler_segment = uri_segments.get(0).copied();
        child_path_segments = if uri_segments.len() > 1 {
            uri_segments[1..].to_vec()
        } else {
            Vec::new()
        };
    }

    debug!("Repository Name: {}", repository_name.display());
    debug!("Handler Segment: {:?}", handler_segment);
    debug!("Child Path Segments: {:?}", child_path_segments);

    if repository_name.as_os_str().is_empty() {
        let prefix = uri_segments.join("/");
        if prefix.is_empty() {
            return RepositoryNotFound.into_response();
        }
        let prefix_response = super::index::render_prefix(db.clone(), &prefix).await;
        if let Ok(response) = &prefix_response {
            if response.status() == StatusCode::NOT_FOUND {
                if let Some(tree_response) =
                    super::file_tree::render(scan_path.as_path(), &PathBuf::from(&prefix))
                {
                    return tree_response;
                }
            }
        }
        return match prefix_response {
            Ok(response) => response,
            Err(error) => error.into_response(),
        };
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
        Some("thread") => h!(handle_thread), // GEMINI: Add thread handler
        _ => {
            // Default to summary; preserve any trailing path segments so the
            // page title/header can reflect the requested path.
            if !child_path_segments.is_empty() {
                child_path = Some(child_path_segments.into_iter().collect::<PathBuf>().clean());
            }
            h!(handle_summary)
        } // Default to summary if no specific handler is found
    };

    debug!("Final Child Path: {:?}", child_path);

    let repository_abs_path = repository_abs_path(scan_path.as_path(), &repository_name);

    debug!("Repository Name: {}", repository_name.display());
    debug!("Repository Path: {}", repository_abs_path.display());

    // If the selected repository path is not a valid git repo but is a directory,
    // render a file tree landing page instead of letting the handler fail.
    if gix::open(&repository_abs_path).is_err() && repository_abs_path.is_dir() {
        return super::file_tree::render(scan_path.as_path(), &repository_name)
            .unwrap_or_else(|| RepositoryNotFound.into_response());
    }

    request.extensions_mut().insert(ChildPath(child_path));
    request.extensions_mut().insert(Repository(repository_name));
    request
        .extensions_mut()
        .insert(RepositoryPath(repository_abs_path));
    request.extensions_mut().insert(ScanPath(scan_path.to_path_buf()));

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

impl Repository {
    pub fn href_base(&self) -> String {
        if self.0.as_os_str().is_empty() || self.0 == Path::new(".") {
            "/".to_string()
        } else {
            format!("/{}/", self.0.display())
        }
    }

    pub fn title(&self) -> String {
        if self.0.as_os_str().is_empty() || self.0 == Path::new(".") {
            "gnostr.org".to_string()
        } else {
            format!("gnostr.org/{}", self.0.display())
        }
    }

    pub fn path_segments(&self) -> Vec<(String, String)> {
        if self.0.as_os_str().is_empty() || self.0 == Path::new(".") {
            return Vec::new();
        }
        let components: Vec<_> = self
            .0
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        let mut segments = Vec::new();
        let mut href = String::new();
        for component in components {
            if !href.is_empty() {
                href.push('/');
            }
            href.push_str(&component);
            segments.push((component.clone(), format!("/{href}")));
        }
        segments
    }

    pub fn display_name(&self) -> String {
        if self.0.as_os_str().is_empty() || self.0 == Path::new(".") {
            "root repo".to_string()
        } else {
            self.0.display().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::repository_abs_path;
    use std::path::Path;

    #[test]
    fn root_repository_uses_scan_path() {
        let scan_path = Path::new("/tmp/gnostr-worktree");
        assert_eq!(
            repository_abs_path(scan_path, Path::new(".")),
            scan_path
        );
    }

    #[test]
    fn nested_repository_keeps_repository_root() {
        let scan_path = Path::new("/tmp");
        assert_eq!(
            repository_abs_path(scan_path, Path::new("repo")),
            Path::new("/tmp/repo")
        );
    }
}

#[derive(Clone)]
pub struct RepositoryPath(pub PathBuf);

#[derive(Clone)]
pub struct ScanPath(pub PathBuf);

#[derive(Clone)]
pub struct ChildPath(pub Option<PathBuf>);

fn repository_abs_path(scan_path: &Path, repository_name: &Path) -> PathBuf {
    if repository_name.as_os_str().is_empty() || repository_name == Path::new(".") {
        scan_path.to_path_buf()
    } else {
        scan_path.join(repository_name)
    }
}

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
