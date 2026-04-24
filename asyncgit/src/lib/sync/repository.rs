use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use git2::{Repository, RepositoryOpenFlags};
use path_clean::PathClean;

use crate::error::Result;

///
pub type RepoPathRef = RefCell<RepoPath>;

///
#[derive(Clone, Debug)]
pub enum RepoPath {
    ///
    Path(PathBuf),
    ///
    Workdir {
        ///
        gitdir: PathBuf,
        ///
        workdir: PathBuf,
    },
}

impl RepoPath {
    ///
    pub fn gitpath(&self) -> &Path {
        match self {
            Self::Path(p) => p.as_path(),
            Self::Workdir { gitdir, .. } => gitdir.as_path(),
        }
    }

    ///
    pub fn workdir(&self) -> Option<&Path> {
        match self {
            Self::Path(_) => None,
            Self::Workdir { workdir, .. } => Some(workdir.as_path()),
        }
    }

    ///
    pub fn as_path(&self) -> &Path {
        self.gitpath()
    }
}

impl From<&str> for RepoPath {
    fn from(p: &str) -> Self {
        Self::Path(PathBuf::from(p))
    }
}

fn resolve_clean_path(path: &Path) -> Result<PathBuf> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(resolved.clean())
}

pub fn resolve_repo_path(repo_path: &RepoPath) -> Result<RepoPath> {
    Ok(match repo_path {
        RepoPath::Path(path) => RepoPath::Path(resolve_clean_path(path.as_path())?),
        RepoPath::Workdir { gitdir, workdir } => RepoPath::Workdir {
            gitdir: resolve_clean_path(gitdir.as_path())?,
            workdir: resolve_clean_path(workdir.as_path())?,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::{resolve_repo_path, RepoPath};
    use path_clean::PathClean;

    #[test]
    fn resolves_relative_repo_paths_against_current_dir() {
        let repo_path = RepoPath::from("../");
        let resolved = resolve_repo_path(&repo_path).expect("path resolution should succeed");

        assert!(resolved.as_path().is_absolute());
        assert_eq!(
            resolved.as_path(),
            std::env::current_dir().expect("cwd").join("../").clean()
        );
    }
}

pub fn repo(repo_path: &RepoPath) -> Result<Repository> {
    let repo = Repository::open_ext(
        repo_path.gitpath(),
        RepositoryOpenFlags::empty(),
        Vec::<&Path>::new(),
    )?;

    if let Some(workdir) = repo_path.workdir() {
        repo.set_workdir(workdir, false)?;
    }

    Ok(repo)
}
