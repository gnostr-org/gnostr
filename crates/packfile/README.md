# Packfile [![Crate](https://img.shields.io/crates/v/packfile.svg)](https://crates.io/crates/packfile) [![API](https://docs.rs/packfile/badge.svg)](https://docs.rs/packfile) [![codecov](https://codecov.io/github/w4/packfile/branch/master/graph/badge.svg?token=4Z9E4UJ0Y6)](https://codecov.io/github/w4/packfile)

`packfile` is a simple library providing utilities to generate [Git Packfiles] in memory.

Usage:
```rust
use packfile::{high_level::GitRepository, low_level::PackFile};

let mut repo = GitRepository::default();
repo.insert(&["path", "to"], "file.txt", "hello world!".into()).unwrap();
let (_commit_hash, entries) =
    repo.commit("Linus Torvalds", "torvalds@example.com", "Some commit message").unwrap();

let _packfile = PackFile::new(&entries);
// ... packfile can then be encoded within a SidebandData to send the data to a client
```

[Git Packfiles]: https://git-scm.com/book/en/v2/Git-Internals-Packfiles
