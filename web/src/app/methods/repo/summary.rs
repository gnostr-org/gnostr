use std::{collections::BTreeMap, sync::Arc};

use anyhow::Context;
use askama::Template;
use axum::{response::IntoResponse, Extension};
use rkyv::string::ArchivedString;

use crate::app::into_response;
use crate::app::methods::filters;
use crate::app::methods::repo::Refs;
use crate::app::methods::repo::Repository;
use crate::app::methods::repo::YokedCommit;
use crate::app::methods::repo::YokedRepository;
use crate::app::methods::repo::{ChildPath, Error, DEFAULT_BRANCHES};
use std::path::PathBuf;

#[derive(Template)]
#[template(path = "repo/summary.html")]
pub struct View {
    repo: Repository,
    refs: Refs,
    commit_list: Vec<YokedCommit>,
    branch: Option<Arc<str>>,
    child_path: Option<PathBuf>,
}

pub async fn handle(
    Extension(repo): Extension<Repository>,
    Extension(ChildPath(child_path)): Extension<ChildPath>,
    Extension(db): Extension<Arc<rocksdb::DB>>,
) -> Result<impl IntoResponse, Error> {
    tokio::task::spawn_blocking(move || {
        let repository = crate::app::database::schema::repository::Repository::open(&db, &*repo)?
            .context("Repository does not exist")?;
        let commits = get_default_branch_commits(&repository, &db)?;

        let mut heads = BTreeMap::new();
        if let Some(heads_db) = repository.get().heads(&db)? {
            for head in heads_db
                .get()
                .0
                .as_slice()
                .iter()
                .map(ArchivedString::as_str)
            {
                let commit_tree = repository.get().commit_tree(db.clone(), head);
                let name = head.strip_prefix("refs/heads/");

                if let (Some(name), Some(commit)) = (name, commit_tree.fetch_latest_one()?) {
                    heads.insert(name.to_string(), commit);
                }
            }
        }

        let tags = repository.get().tag_tree(db).fetch_all()?;

        Ok(into_response(View {
            repo,
            refs: Refs { heads, tags },
            commit_list: commits,
            branch: None,
            child_path,
        }))
    })
    .await
    .context("Failed to attach to tokio task")?
}

pub fn get_default_branch_commits(
    repository: &YokedRepository,
    database: &Arc<rocksdb::DB>,
) -> Result<Vec<YokedCommit>, Error> {
    for branch in repository
        .get()
        .default_branch
        .as_deref()
        .into_iter()
        .chain(DEFAULT_BRANCHES.into_iter())
    {
        let commit_tree = repository.get().commit_tree(database.clone(), branch);
        let commits = commit_tree.fetch_latest(11, 0)?;

        if !commits.is_empty() {
            return Ok(commits);
        }
    }

    Ok(vec![])
}
