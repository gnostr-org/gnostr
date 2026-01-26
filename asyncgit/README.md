# asyncgit

_allow using git2 in an asynchronous context_

This crate is designed as part of the [gitui](http://gitui.org) project.

`asyncgit` provides the primary interface to interact with _git_ repositories. It is split into the main module and a `sync` part. The latter provides convenience wrapper for typical usage patterns against git repositories.

The primary goal however is to allow putting certain (potentially) long running [git2](https://github.com/rust-lang/git2-rs) calls onto a thread pool.[crossbeam-channel](https://github.com/crossbeam-rs/crossbeam) is then used to wait for a notification confirming the result.

In `gitui` this allows the main-thread and therefore the _ui_ to stay responsive.
