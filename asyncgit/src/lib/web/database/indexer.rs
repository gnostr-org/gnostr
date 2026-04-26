use std::{
    collections::HashSet,
    ffi::OsStr,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use git2::{build::RepoBuilder, RepositoryInitOptions, Signature};
use gix::{bstr::ByteSlice, refs::Category, Reference};
use ini::Ini;
use itertools::Itertools;
use rocksdb::WriteBatch;
use time::{OffsetDateTime, UtcOffset};
use tracing::{debug, debug_span, error, instrument, warn};

use crate::web::database::schema::{
    commit::Commit,
    repository::{ArchivedRepository, Repository, RepositoryId},
    tag::{Tag, TagTree},
};

fn is_bare_repository(path: &Path) -> bool {
    path.join("HEAD").is_file() && path.join("objects").is_dir()
}

fn is_help_material_fixture(path: &Path) -> bool {
    if !is_bare_repository(path) {
        return false;
    }

    fs::read_to_string(path.join("description"))
        .map(|description| description.contains("gnostr learning repo for browsing commits, trees, and patches"))
        .unwrap_or(false)
}

fn write_fixture_file(root: &Path, relative_path: &str, content: &[u8]) -> anyhow::Result<()> {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create fixture directory {}", parent.display()))?;
    }

    fs::write(&path, content)
        .with_context(|| format!("failed to write fixture file {}", path.display()))?;
    Ok(())
}

fn stage_paths(repo: &git2::Repository, paths: &[&str]) -> anyhow::Result<()> {
    let mut index = repo.index()?;
    for path in paths {
        index.add_path(Path::new(path))?;
    }
    index.write()?;
    Ok(())
}

fn commit_fixture(
    repo: &git2::Repository,
    message: &str,
    paths: &[&str],
) -> anyhow::Result<()> {
    stage_paths(repo, paths)?;

    let tree_id = repo.index()?.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let sig = Signature::now("Copilot", "copilot@users.noreply.github.com")?;
    let parents = match repo.head() {
        Ok(head) => vec![head.peel_to_commit()?],
        Err(_) => Vec::new(),
    };
    let parent_refs: Vec<_> = parents.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)?;
    Ok(())
}

fn create_help_material_source_repo(source_repo: &Path) -> anyhow::Result<git2::Repository> {
    let mut init_opts = RepositoryInitOptions::new();
    init_opts.initial_head("main");
    let repo = git2::Repository::init_opts(source_repo, &init_opts)?;

    write_fixture_file(
        source_repo,
        "README.md",
        b"# gnostr learning repo\n\nThis fixture is a guided tour through gnostr and git history.\n\n## Start here\n\n- Open the tree view to browse the help files.\n- Open the log view to follow the commits in order.\n- Open the patch view to see how CRLF changes are rendered.\n- Visit the repo summary to read the description and recent history.\n",
    )?;
    write_fixture_file(
        source_repo,
        "docs/quickstart.md",
        b"# Quickstart\n\n1. Start gnostr.\n2. Open this repository in the browser.\n3. Click into `README.md` or `docs/` to explore the tree.\n4. Open the latest commit to read the release notes.\n5. Open the patch view to inspect the CRLF demo.\n\nThe goal is to make each page teach one small thing about the UI.\n",
    )?;

    commit_fixture(
        &repo,
        "Add gnostr learning guide",
        &["README.md", "docs/quickstart.md"],
    )?;

    write_fixture_file(
        source_repo,
        "docs/nostr-basics.md",
        b"# Nostr basics\n\n- `npub` is a public key people can share.\n- `nsec` is a private key and should stay secret.\n- Relays move events between clients.\n- Events are the packets the network exchanges.\n- Repo metadata becomes browseable history in gnostr.\n\nUse this file to explain the Nostr terms that appear in the UI.\n",
    )?;
    write_fixture_file(
        source_repo,
        "docs/repo-tour.md",
        b"# Repo tour\n\n- `README.md` gives the overview.\n- `docs/quickstart.md` shows the first clicks.\n- `docs/nostr-basics.md` explains the protocol terms.\n- `examples/crlf-demo.txt` exists so the patch view has a line-ending example.\n\nTry the tree, commit, and patch pages to see each file in context.\n",
    )?;
    write_fixture_file(
        source_repo,
        "examples/crlf-demo.txt",
        b"gnostr keeps the history readable.\r\nThis file intentionally uses CRLF line endings.\r\nOpen the patch view to see how the line endings are shown.\r\n",
    )?;

    commit_fixture(
        &repo,
        "Add nostr basics and repo tour",
        &["docs/nostr-basics.md", "docs/repo-tour.md"],
    )?;

    commit_fixture(
        &repo,
        "Add CRLF patch demo",
        &["examples/crlf-demo.txt"],
    )?;

    Ok(repo)
}

pub fn ensure_bare_repo_fixture(fixture_root: &Path, fixture_name: &str) -> anyhow::Result<PathBuf> {
    let fixture_path = fixture_root.join(format!("{fixture_name}.git"));

    if is_help_material_fixture(&fixture_path) {
        return Ok(fixture_path);
    }

    fs::create_dir_all(fixture_root)
        .with_context(|| format!("failed to create fixture directory {}", fixture_root.display()))?;

    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path)
            .with_context(|| format!("failed to remove stale fixture {}", fixture_path.display()))?;
    }

    let source_repo_dir = tempfile::tempdir_in(fixture_root)
        .with_context(|| format!("failed to create temp repo under {}", fixture_root.display()))?;
    create_help_material_source_repo(source_repo_dir.path())?;

    RepoBuilder::new()
        .bare(true)
        .clone(
            source_repo_dir
                .path()
                .to_str()
                .context("source repository path must be valid UTF-8")?,
            &fixture_path,
        )
        .with_context(|| {
            format!(
                "failed to create bare fixture {} from {}",
                fixture_path.display(),
                source_repo_dir.path().display()
            )
        })?;

    let fixture_repo = git2::Repository::open_bare(&fixture_path)
        .with_context(|| format!("failed to open bare fixture {}", fixture_path.display()))?;
    let head_commit = fixture_repo
        .head()
        .context("failed to read fixture HEAD")?
        .peel_to_commit()
        .context("failed to peel fixture HEAD to commit")?
        .id();
    fixture_repo
        .reference("refs/heads/main", head_commit, true, "ensure fixture main ref")
        .context("failed to write fixture main ref")?;

    fs::write(
        fixture_path.join("description"),
        b"Interactive gnostr learning repo for browsing commits, trees, and patches\n",
    )
    .with_context(|| format!("failed to write {}", fixture_path.join("description").display()))?;

    Ok(fixture_path)
}

pub fn run(scan_path: &Path, db: &Arc<rocksdb::DB>) {
    let span = debug_span!("index_update");
    let _entered = span.enter();

    debug!("Starting index update");

    update_repository_metadata(scan_path, db);
    update_repository_reflog(scan_path, db.clone());
    update_repository_tags(scan_path, db.clone());

    debug!("Flushing to disk");

    if let Err(error) = db.flush() {
        error!(%error, "Failed to flush database to disk");
    }

    debug!("Finished index update");
}

#[instrument(skip(db))]
fn update_repository_metadata(scan_path: &Path, db: &rocksdb::DB) {
    let mut discovered = Vec::new();

    // discover_repositories
    discover_repositories(scan_path, &mut discovered);

    for repository in discovered {
        // get_relative_path
        let Some(relative) = get_relative_path(scan_path, &repository) else {
            continue;
        };
        println!(
            "Processing repository: relative={}, repository={}",
            relative.display(),
            repository.display()
        );

        let id = match Repository::open(db, relative) {
            Ok(v) => v.map_or_else(RepositoryId::new, |v| {
                RepositoryId(v.get().id.0.to_native())
            }),
            Err(error) => {
                // maybe we could nuke it ourselves, but we need to instantly trigger
                // a reindex and we could enter into an infinite loop if there's a bug
                // or something
                error!(%error, "Failed to open repository index {}, please consider nuking database", relative.display());
                continue;
            }
        };

        let name = if relative == Path::new(".") {
            scan_path
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or(".")
        } else {
            relative
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap_or(".")
        };
        // Read description from correct location based on repository type
        let description_path = if std::process::Command::new("git")
            .args(["rev-parse", "--is-bare-repository"])
            .current_dir(&repository)
            .output()
            .map(|output| output.status.success() && output.stdout.starts_with(b"true"))
            .unwrap_or(false)
        {
            // Bare repository: description is in repo root
            repository.join("description")
        } else {
            // Working tree: description is in .git directory
            repository.join(".git").join("description")
        };

        let description = std::fs::read(&description_path).unwrap_or_default();
        let description = String::from_utf8(description)
            .ok()
            .filter(|v| !v.is_empty());

        let repository_path = scan_path.join(relative);

        let mut git_repository = match gix::open(repository_path.clone()) {
            Ok(v) => v,
            Err(error) => {
                warn!(%error, "Failed to open repository {} to update metadata, skipping", relative.display());
                continue;
            }
        };

        git_repository.object_cache_size(10 * 1024 * 1024);

        let res = Repository {
            id,
            name: name.to_string(),
            description,
            owner: find_gitweb_owner(repository_path.as_path()),
            last_modified: {
                let r =
                    find_last_committed_time(&git_repository).unwrap_or(OffsetDateTime::UNIX_EPOCH);
                (r.unix_timestamp(), r.offset().whole_seconds())
            },
            default_branch: find_default_branch(&git_repository).ok().flatten(),
        }
        .insert(db, relative);

        if let Err(error) = res {
            warn!(%error, "Failed to insert repository");
        }
    }
}

fn find_default_branch(repo: &gix::Repository) -> Result<Option<String>, anyhow::Error> {
    Ok(Some(repo.head()?.name().as_bstr().to_string()))
}

fn find_last_committed_time(repo: &gix::Repository) -> Result<OffsetDateTime, anyhow::Error> {
    let mut timestamp = OffsetDateTime::UNIX_EPOCH;

    for reference in repo.references()?.all()? {
        let Ok(mut reference) = reference else {
            warn!("Skipping unreadable reference while finding last committed time");
            continue;
        };

        let Ok(commit) = reference.peel_to_commit() else {
            continue;
        };

        let committer = commit.committer()?;
        let mut committed_time = OffsetDateTime::from_unix_timestamp(committer.time.seconds)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH);

        if let Ok(offset) = UtcOffset::from_whole_seconds(committer.time.offset) {
            committed_time = committed_time.to_offset(offset);
        }

        if committed_time > timestamp {
            timestamp = committed_time;
        }
    }

    Ok(timestamp)
}

#[instrument(skip(db))]
fn update_repository_reflog(scan_path: &Path, db: Arc<rocksdb::DB>) {
    let repos = match Repository::fetch_all(&db) {
        Ok(v) => v,
        Err(error) => {
            error!(%error, "Failed to read repository index to update reflog, consider deleting database directory");
            return;
        }
    };

    for (relative_path, db_repository) in repos {
        let Some(git_repository) = open_repo(scan_path, &relative_path, db_repository.get(), &db)
        else {
            continue;
        };

        let references = match git_repository.references() {
            Ok(v) => v,
            Err(error) => {
                error!(%error, "Failed to read references for {relative_path}");
                continue;
            }
        };

        let references = match references.all() {
            Ok(v) => v,
            Err(error) => {
                error!(%error, "Failed to read references for {relative_path}");
                continue;
            }
        };

        let mut valid_references = Vec::new();

        for reference in references {
            let mut reference = match reference {
                Ok(v) => v,
                Err(error) => {
                    error!(%error, "Failed to read reference for {relative_path}");
                    continue;
                }
            };

            let reference_name = reference.name();
            if reference_name.category() != Some(Category::LocalBranch) {
                continue;
            }

            valid_references.push(reference_name.as_bstr().to_string());

            if let Err(error) = branch_index_update(
                &mut reference,
                &relative_path,
                db_repository.get(),
                db.clone(),
                &git_repository,
                false,
            ) {
                error!(%error, "Failed to update reflog for {relative_path}@{:?}", valid_references.last());
            }
        }

        if let Err(error) = db_repository.get().replace_heads(&db, &valid_references) {
            error!(%error, "Failed to update heads");
        }
    }
}

#[instrument(skip(reference, db_repository, db, git_repository))]
fn branch_index_update(
    reference: &mut Reference<'_>,
    relative_path: &str,
    db_repository: &ArchivedRepository,
    db: Arc<rocksdb::DB>,
    git_repository: &gix::Repository,
    force_reindex: bool,
) -> Result<(), anyhow::Error> {
    debug!("Refreshing indexes");

    let commit_tree = db_repository.commit_tree(db.clone(), reference.name().as_bstr().to_str()?);

    if force_reindex {
        commit_tree.drop_commits()?;
    }

    let commit = reference.peel_to_commit()?;

    let latest_indexed = if let Some(latest_indexed) = commit_tree.fetch_latest_one()? {
        if commit.id().as_bytes() == latest_indexed.get().hash.as_slice() {
            debug!("No commits since last index");
            return Ok(());
        }

        Some(latest_indexed)
    } else {
        None
    };

    // TODO: stop collecting into a vec
    let revwalk = git_repository
        .rev_walk([commit.id().detach()])
        .all()?
        .collect::<Vec<_>>()
        .into_iter()
        .rev();

    let tree_len = commit_tree.len()?;
    let mut seen = false;
    let mut i = 0;
    for revs in &revwalk.chunks(250) {
        let mut batch = WriteBatch::default();

        for rev in revs {
            let rev = rev?;

            if let (false, Some(latest_indexed)) = (seen, &latest_indexed) {
                if rev.id.as_bytes() == latest_indexed.get().hash.as_slice() {
                    seen = true;
                }

                continue;
            }

            seen = true;

            if ((i + 1) % 25_000) == 0 {
                debug!("{} commits ingested", i + 1);
            }

            let commit = rev.object()?;
            let author = commit.author()?;
            let committer = commit.committer()?;

            Commit::new(&commit, author, committer)?.insert(
                &commit_tree,
                tree_len + i,
                &mut batch,
            )?;
            i += 1;
        }

        commit_tree.update_counter(tree_len + i, &mut batch)?;
        db.write_without_wal(batch)?;
    }

    if !seen && !force_reindex {
        warn!("Detected converged history, forcing reindex");

        return branch_index_update(
            reference,
            relative_path,
            db_repository,
            db,
            git_repository,
            true,
        );
    }

    Ok(())
}

#[instrument(skip(db))]
fn update_repository_tags(scan_path: &Path, db: Arc<rocksdb::DB>) {
    let repos = match Repository::fetch_all(&db) {
        Ok(v) => v,
        Err(error) => {
            error!(%error, "Failed to read repository index to update tags, consider deleting database directory");
            return;
        }
    };

    for (relative_path, db_repository) in repos {
        let Some(git_repository) = open_repo(scan_path, &relative_path, db_repository.get(), &db)
        else {
            continue;
        };

        if let Err(error) = tag_index_scan(
            &relative_path,
            db_repository.get(),
            db.clone(),
            &git_repository,
        ) {
            error!(%error, "Failed to update tags for {relative_path}");
        }
    }
}

#[instrument(skip(db_repository, db, git_repository))]
fn tag_index_scan(
    relative_path: &str,
    db_repository: &ArchivedRepository,
    db: Arc<rocksdb::DB>,
    git_repository: &gix::Repository,
) -> Result<(), anyhow::Error> {
    let tag_tree = db_repository.tag_tree(db);

    let git_tags: HashSet<_> = git_repository
        .references()
        .context("Failed to scan indexes on git repository")?
        .all()?
        .filter_map(Result::ok)
        .filter(|v| v.name().category() == Some(Category::Tag))
        .map(|v| v.name().as_bstr().to_string())
        .collect();
    let indexed_tags: HashSet<String> = tag_tree.list()?.into_iter().collect();

    // insert any git tags that are missing from the index
    for tag_name in git_tags.difference(&indexed_tags) {
        tag_index_update(tag_name, git_repository, &tag_tree)?;
    }

    // remove any extra tags that the index has
    // TODO: this also needs to check peel_to_tag
    for tag_name in indexed_tags.difference(&git_tags) {
        tag_index_delete(tag_name, &tag_tree)?;
    }

    Ok(())
}

#[instrument(skip(git_repository, tag_tree))]
fn tag_index_update(
    tag_name: &str,
    git_repository: &gix::Repository,
    tag_tree: &TagTree,
) -> Result<(), anyhow::Error> {
    let mut reference = git_repository
        .find_reference(tag_name)
        .context("Failed to read newly discovered tag")?;

    if let Ok(tag) = reference.peel_to_tag() {
        debug!("Inserting newly discovered tag to index");

        Tag::new(tag.tagger()?)?.insert(tag_tree, tag_name)?;
    }

    Ok(())
}

#[instrument(skip(tag_tree))]
fn tag_index_delete(tag_name: &str, tag_tree: &TagTree) -> Result<(), anyhow::Error> {
    debug!("Removing stale tag from index");
    tag_tree.remove(tag_name)?;

    Ok(())
}

#[instrument(skip(scan_path, db_repository, db))]
fn open_repo<P: AsRef<Path> + Debug>(
    scan_path: &Path,
    relative_path: P,
    db_repository: &ArchivedRepository,
    db: &rocksdb::DB,
) -> Option<gix::Repository> {
    let full_path = scan_path.join(relative_path.as_ref());
    debug!(
        "Attempting to open repository: {} (relative: {:?})",
        full_path.display(),
        relative_path
    );
    match gix::open(&full_path) {
        Ok(mut v) => {
            v.object_cache_size(10 * 1024 * 1024);
            Some(v)
        }
        Err(gix::open::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            warn!("Repository gone from disk, removing from db");

            if let Err(error) = db_repository.delete(db, relative_path) {
                warn!(%error, "Failed to delete dangling index");
            }

            None
        }
        Err(error) => {
            warn!(%error, "Failed to reindex, skipping");
            None
        }
    }
}

fn get_relative_path<'a>(relative_to: &Path, full_path: &'a Path) -> Option<&'a Path> {
    println!(
        "get_relative_path:relative_to:{} (scan_path)",
        &relative_to.display()
    ); //scan_path
    println!(
        "get_relative_path:full_path:{} (repository)",
        &full_path.display()
    ); //repository
    let relative = full_path.strip_prefix(relative_to).ok()?;

    if relative.as_os_str().is_empty() {
        Some(Path::new("."))
    } else {
        Some(relative)
    }
}

fn discover_repositories(current: &Path, discovered_repos: &mut Vec<PathBuf>) {
    // First check if current path is itself a bare repository
    // is_bare_repository
    if is_bare_repository(current) {
        debug!("Discovered bare Git repository at: {}", current.display());
        discovered_repos.push(current.to_path_buf());
        return; // Stop recursion for this path
    }

    // Existing check for working tree or normal repository
    let is_repo = gix::open(current).is_ok();
    if is_repo {
        debug!("Discovered Git repository at: {}", current.display());
        discovered_repos.push(current.to_path_buf());
    }

    // Check if current is a directory that contains .git
    if current.is_dir() && !is_repo {
        let git_path = current.join(".git");

        // Check if .git exists as either file or directory
        if git_path.exists() {
            // Try to open as repository (handles both .git file (worktree) and .git directory)
            if gix::open(current).is_ok() {
                debug!("Discovered Git repository at: {}", current.display());
                discovered_repos.push(current.to_path_buf());
                return; // Stop recursion for this path
            } else {
                debug!(
                    "Found .git at {} but not a valid repository",
                    current.display()
                );
            }
        }
    } else {
        println!("current.is_file()={}", current.is_file());
    }

    // If it's not a repository, and it's a directory, then we can recurse.
    let current_dir_entries = match std::fs::read_dir(current) {
        Ok(v) => v,
        Err(error) => {
            // Don't log an error if we just can't read a directory, it might be permissions, etc.
            // Only log if it's unexpected.
            if error.kind() != std::io::ErrorKind::NotFound
                && error.kind() != std::io::ErrorKind::PermissionDenied
            {
                error!(%error, "Failed to read directory {}", current.display());
            }
            return;
        }
    };

    for entry in current_dir_entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_dir() {
            if path.file_name() == Some(OsStr::new(".git")) {
                continue;
            }
            // Skip directories under `target/`
            if path.components().any(|c| c.as_os_str() == "target") {
                debug!("Skipping target directory: {}", path.display());
                continue;
            }
            discover_repositories(&path, discovered_repos);
        }
    }
}

fn find_gitweb_owner(repository_path: &Path) -> Option<String> {
    // Load the Git config file and attempt to extract the owner from the "gitweb" section.
    // If the owner is not found, an empty string is returned.
    Ini::load_from_file(repository_path.join("config"))
        .ok()?
        .section_mut(Some("gitweb"))
        .and_then(|section| section.remove("owner"))
}
