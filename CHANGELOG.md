### Refactoring

*   `GitTestRepo`: Allow `&mut self` for methods modifying the repository. (690f7e4)

This change updates several `GitTestRepo` methods to take `&mut self` instead of `&self`, enabling mutable operations on the repository within tests. This is a crucial refactoring that was necessary for `ngit` push/pull tests to function correctly when the underlying repository state needed to be modified.

The methods affected are:
*   `create_proposals_and_repo_with_proposal_pulled_and_checkedout`
*   `remove_latest_commit_so_proposal_branch_is_behind_and_checkout_main`
*   `amend_last_commit`
*   `create_and_populate_branch`
*   `populate`
*   `new` (in `send.rs` tests)

Additionally:
*   The `test_capture_tmux` test in `tests/test_screenshot.rs` has been added.
*   The `types/Cargo.toml`, `xq/Cargo.toml`, `xq/crates/lang/Cargo.toml`, and `xq/fuzz/Cargo.toml` files have their versions updated to `1906.925289.572444`.
*   The `update_versions.sh` script has been added to automate version updates.
*   The `vendor.sh` script has been removed.
*   The `extend` method calls in `types/src/types/naddr.rs`, `types/src/types/nevent.rs`, `types/src/types/nostr_url.rs`, and `types/src/types/profile.rs` have been updated to use `&self.d.as_bytes()[..len as usize]` instead of `self.d[..len as usize].as_bytes()` for consistency and correctness.