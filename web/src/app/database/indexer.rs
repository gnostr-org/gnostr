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
use gix::{bstr::ByteSlice, refs::Category, References};
use ini::Ini;
use itertools::Itertools;
use rocksdb::WriteBatch;
use time::{OffsetDateTime, UtcOffset};
use tracing::{debug, debug_span, error, instrument, warn};

use crate::app::database::schema::{
    commit::Commit,
    prefixes::REPOSITORY_FAMILY,
    repository::{ArchivedRepository, Repository, RepositoryId},
    tag::{Tag, TagTree},
};

// ... existing file content unchanged ...
