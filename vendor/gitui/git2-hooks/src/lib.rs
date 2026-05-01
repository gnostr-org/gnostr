//! git2-rs addon supporting git hooks
//!
//! we look for hooks in the following locations:
//!  * whatever `config.hooksPath` points to
//!  * `.git/hooks/`
//!  * whatever list of paths provided as `other_paths` (in order)
//!
//! most basic hook is: [`hooks_pre_commit`]. see also other `hooks_*` functions.
//!
//! [`create_hook`] is useful to create git hooks from code (unittest make heavy usage of it)

#![forbid(unsafe_code)]
#![deny(
	mismatched_lifetime_syntaxes,
	unused_imports,
	unused_must_use,
	dead_code,
	unstable_name_collisions,
	unused_assignments
)]
#![deny(clippy::all, clippy::perf, clippy::pedantic, clippy::nursery)]
#![allow(
	clippy::missing_errors_doc,
	clippy::must_use_candidate,
	clippy::module_name_repetitions
)]

mod error;
mod hookspath;

use std::{
	fs::File,
	io::{Read, Write},
	path::{Path, PathBuf},
};

pub use error::HooksError;
use error::Result;
use hookspath::HookPaths;

use git2::{Oid, Repository};

pub const HOOK_POST_COMMIT: &str = "post-commit";
pub const HOOK_PRE_COMMIT: &str = "pre-commit";
pub const HOOK_COMMIT_MSG: &str = "commit-msg";
pub const HOOK_PREPARE_COMMIT_MSG: &str = "prepare-commit-msg";
pub const HOOK_PRE_PUSH: &str = "pre-push";
pub const HOOK_APPLYPATCH_MSG: &str = "applypatch-msg";
pub const HOOK_PRE_APPLYPATCH: &str = "pre-applypatch";
pub const HOOK_POST_APPLYPATCH: &str = "post-applypatch";
pub const HOOK_PRE_MERGE_COMMIT: &str = "pre-merge-commit";
pub const HOOK_POST_MERGE: &str = "post-merge";
pub const HOOK_PRE_REBASE: &str = "pre-rebase";
pub const HOOK_POST_REWRITE: &str = "post-rewrite";
pub const HOOK_POST_CHECKOUT: &str = "post-checkout";
pub const HOOK_PRE_AUTO_GC: &str = "pre-auto-gc";
pub const HOOK_SENDEMAIL_VALIDATE: &str = "sendemail-validate";
pub const HOOK_UPDATE: &str = "update";
pub const HOOK_POST_UPDATE: &str = "post-update";
pub const HOOK_PRE_RECEIVE: &str = "pre-receive";
pub const HOOK_POST_RECEIVE: &str = "post-receive";
pub const HOOK_PROC_RECEIVE: &str = "proc-receive";
pub const HOOK_PUSH_TO_CHECKOUT: &str = "push-to-checkout";
pub const HOOK_REFERENCE_TRANSACTION: &str = "reference-transaction";

/// Return the current target operating system name.
///
/// This is a lightweight wrapper around [`std::env::consts::OS`] so callers
/// can branch on platform without hard-coding `cfg` checks everywhere.
pub fn current_os() -> &'static str {
	std::env::consts::OS
}

/// Return `true` when the current target is Windows.
///
/// Use this when a hook path, shell, or interpreter differs on Windows.
pub fn is_windows() -> bool {
	current_os() == "windows"
}

/// Return `true` when the current target is Unix-like.
///
/// This includes Linux, macOS, BSDs, and other Rust-supported Unix targets.
pub fn is_unix() -> bool {
	cfg!(unix)
}

/// Return `true` when the current target is macOS.
///
/// This is useful for tests or hook wrappers that need Apple-specific paths or
/// command-line behavior.
pub fn is_macos() -> bool {
	current_os() == "macos"
}

/// Return `true` when the current target is Linux.
///
/// This is useful for tests or hook wrappers that need Linux-specific
/// assumptions without spelling out `cfg!(target_os = "linux")`.
pub fn is_linux() -> bool {
	current_os() == "linux"
}

const HOOK_COMMIT_MSG_TEMP_FILE: &str = "COMMIT_EDITMSG";

/// Check if a given hook is present considering config/paths and optional extra paths.
pub fn hook_available(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	hook: &str,
) -> Result<bool> {
	let hook = HookPaths::new(repo, other_paths, hook)?;
	Ok(hook.found())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrePushRef {
	pub local_ref: String,
	pub local_oid: Option<Oid>,
	pub remote_ref: String,
	pub remote_oid: Option<Oid>,
}

impl PrePushRef {
	pub fn new(
		local_ref: impl Into<String>,
		local_oid: Option<Oid>,
		remote_ref: impl Into<String>,
		remote_oid: Option<Oid>,
	) -> Self {
		Self {
			local_ref: local_ref.into(),
			local_oid,
			remote_ref: remote_ref.into(),
			remote_oid,
		}
	}

	fn format_oid(oid: Option<Oid>) -> String {
		// "If the foreign ref does not yet exist the <remote-object-name> will be the all-zeroes object name"
		// see https://git-scm.com/docs/githooks#_pre_push
		oid.map_or_else(|| "0".repeat(40), |id| id.to_string())
	}

	pub fn to_line(&self) -> String {
		format!(
			"{} {} {} {}",
			self.local_ref,
			Self::format_oid(self.local_oid),
			self.remote_ref,
			Self::format_oid(self.remote_oid)
		)
	}

	/// Build stdin content from a slice of updates (for pre-push hook)
	pub fn to_stdin(updates: &[Self]) -> String {
		let mut stdin = String::new();
		for update in updates {
			stdin.push_str(&update.to_line());
			stdin.push('\n');
		}
		stdin
	}
}

/// Response from running a hook
#[derive(Debug, PartialEq, Eq)]
pub struct HookRunResponse {
	/// path of the hook that was run
	pub hook: PathBuf,
	/// stdout output emitted by hook
	pub stdout: String,
	/// stderr output emitted by hook
	pub stderr: String,
	/// exit code as reported back from process calling the hook (0 = success)
	pub code: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HookResult {
	/// No hook found
	NoHookFound,
	/// Hook executed (check `HookRunResponse.code` for success/failure)
	Run(HookRunResponse),
}

impl HookResult {
	/// helper to check if hook ran successfully (found and exit code 0)
	pub const fn is_successful(&self) -> bool {
		matches!(self, Self::Run(response) if response.is_successful())
	}
}

impl HookRunResponse {
	/// Check if the hook succeeded (exit code 0)
	pub const fn is_successful(&self) -> bool {
		self.code == 0
	}
}

/// helper method to create git hooks programmatically (heavy used in unittests)
///
/// # Panics
/// Panics if hook could not be created
pub fn create_hook(
	r: &Repository,
	hook: &str,
	hook_script: &[u8],
) -> PathBuf {
	let hook = HookPaths::new(r, None, hook).unwrap();

	let path = hook.hook.clone();

	create_hook_in_path(&hook.hook, hook_script);

	path
}

fn create_hook_in_path(path: &Path, hook_script: &[u8]) {
	File::create(path).unwrap().write_all(hook_script).unwrap();

	#[cfg(unix)]
	{
		std::process::Command::new("chmod")
			.arg("+x")
			.arg(path)
			// .current_dir(path)
			.output()
			.unwrap();
	}
}

/// Git hook: `commit_msg`
///
/// This hook is documented here <https://git-scm.com/docs/githooks#_commit_msg>.
/// We use the same convention as other git clients to create a temp file containing
/// the commit message at `<.git|hooksPath>/COMMIT_EDITMSG` and pass it's relative path as the only
/// parameter to the hook script.
pub fn hooks_commit_msg(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	msg: &mut String,
) -> Result<HookResult> {
	let hook = HookPaths::new(repo, other_paths, HOOK_COMMIT_MSG)?;

	if !hook.found() {
		return Ok(HookResult::NoHookFound);
	}

	let temp_file = hook.git.join(HOOK_COMMIT_MSG_TEMP_FILE);
	File::create(&temp_file)?.write_all(msg.as_bytes())?;

	let res = hook.run_hook_os_str([&temp_file])?;

	// load possibly altered msg
	msg.clear();
	File::open(temp_file)?.read_to_string(msg)?;

	Ok(res)
}

/// this hook is documented here <https://git-scm.com/docs/githooks#_pre_commit>
pub fn hooks_pre_commit(
	repo: &Repository,
	other_paths: Option<&[&str]>,
) -> Result<HookResult> {
	let hook = HookPaths::new(repo, other_paths, HOOK_PRE_COMMIT)?;

	if !hook.found() {
		return Ok(HookResult::NoHookFound);
	}

	hook.run_hook(&[])
}

/// this hook is documented here <https://git-scm.com/docs/githooks#_post_commit>
pub fn hooks_post_commit(
	repo: &Repository,
	other_paths: Option<&[&str]>,
) -> Result<HookResult> {
	let hook = HookPaths::new(repo, other_paths, HOOK_POST_COMMIT)?;

	if !hook.found() {
		return Ok(HookResult::NoHookFound);
	}

	hook.run_hook(&[])
}

/// this hook is documented here <https://git-scm.com/docs/githooks#_pre_push>
///
/// According to git documentation, pre-push hook receives:
/// - remote name as first argument (or URL if remote is not named)
/// - remote URL as second argument
/// - information about refs being pushed via stdin in format:
///   `<local-ref> SP <local-object-name> SP <remote-ref> SP <remote-object-name> LF`
///
/// If `remote` is `None` or empty, the `url` is used for both arguments as per Git spec.
///
/// Note: The hook is called even when `updates` is empty (matching Git's behavior).
/// This can occur when pushing tags that already exist on the remote.
pub fn hooks_pre_push(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	remote: Option<&str>,
	url: &str,
	updates: &[PrePushRef],
) -> Result<HookResult> {
	let hook = HookPaths::new(repo, other_paths, HOOK_PRE_PUSH)?;

	if !hook.found() {
		return Ok(HookResult::NoHookFound);
	}

	// If a remote is not named (None or empty), the URL is passed for both arguments
	let remote_name = match remote {
		Some(r) if !r.is_empty() => r,
		_ => url,
	};

	let stdin_data = PrePushRef::to_stdin(updates);

	hook.run_hook_os_str_with_stdin(
		[remote_name, url],
		Some(stdin_data.as_bytes()),
	)
}

fn run_hook_stub(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	hook_name: &str,
	args: &[&str],
	stdin: Option<&[u8]>,
) -> Result<HookResult> {
	let hook = HookPaths::new(repo, other_paths, hook_name)?;

	if !hook.found() {
		return Ok(HookResult::NoHookFound);
	}

	hook.run_hook_os_str_with_stdin(args, stdin)
}

/// Git invokes `applypatch-msg` when `git am` prepares to apply a patch.
///
/// The hook receives the path to the temporary commit-message file as its
/// first argument, allowing scripts to validate or rewrite the message before
/// the patch is applied.
pub fn hooks_applypatch_msg(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	msg_file: &str,
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_APPLYPATCH_MSG, &[msg_file], None)
}

/// Git invokes `pre-applypatch` after a patch has been applied but before the
/// resulting commit is finalized.
///
/// This helper simply discovers and runs the hook if it exists; the hook
/// itself decides whether the patch should proceed.
pub fn hooks_pre_applypatch(
	repo: &Repository,
	other_paths: Option<&[&str]>,
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_PRE_APPLYPATCH, &[], None)
}

/// Git invokes `post-applypatch` after `git am` has successfully applied a
/// patch.
///
/// Use this for notifications, cleanup, or lightweight bookkeeping that should
/// happen only after the patch is committed.
pub fn hooks_post_applypatch(
	repo: &Repository,
	other_paths: Option<&[&str]>,
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_POST_APPLYPATCH, &[], None)
}

/// Git invokes `pre-merge-commit` before it creates a merge commit.
///
/// The helper forwards any supplied arguments unchanged. In practice Git uses
/// this hook as a last chance to veto or adjust a merge before the commit is
/// written.
pub fn hooks_pre_merge_commit(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	args: &[&str],
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_PRE_MERGE_COMMIT, args, None)
}

/// Git invokes `post-merge` after a merge has completed.
///
/// This hook is a good place for cache refreshes, notifications, or any
/// follow-up work that should only run once the merge is already in place.
pub fn hooks_post_merge(
	repo: &Repository,
	other_paths: Option<&[&str]>,
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_POST_MERGE, &[], None)
}

/// Git invokes `pre-rebase` before a rebase begins.
///
/// Git may pass the upstream and branch names as arguments. The helper accepts
/// a caller-supplied argument slice so you can forward exactly what your
/// workflow needs.
pub fn hooks_pre_rebase(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	args: &[&str],
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_PRE_REBASE, args, None)
}

/// Git invokes `post-rewrite` after a history-rewriting command finishes.
///
/// The first argument identifies the rewrite command and the rewritten commit
/// mapping is typically streamed on stdin. The helper forwards both pieces
/// unchanged.
pub fn hooks_post_rewrite(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	args: &[&str],
	stdin: Option<&[u8]>,
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_POST_REWRITE, args, stdin)
}

/// Git invokes `post-checkout` after switching branches or checking out paths.
///
/// Git passes the old head, the new head, and a `0`/`1` flag that indicates
/// whether the checkout was a branch switch. This helper formats those values
/// and runs the hook if it exists.
pub fn hooks_post_checkout(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	old_head: &str,
	new_head: &str,
	branch_switch: bool,
) -> Result<HookResult> {
	run_hook_stub(
		repo,
		other_paths,
		HOOK_POST_CHECKOUT,
		[
			old_head,
			new_head,
			if branch_switch { "1" } else { "0" },
		]
		.as_slice(),
		None,
	)
}

/// Git invokes `pre-auto-gc` before auto-garbage-collection runs.
///
/// This is a lightweight maintenance hook with no required arguments.
pub fn hooks_pre_auto_gc(
	repo: &Repository,
	other_paths: Option<&[&str]>,
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_PRE_AUTO_GC, &[], None)
}

/// Git invokes `sendemail-validate` while validating mail generated by
/// `git send-email`.
///
/// The helper accepts a caller-supplied argument slice so scripts can mirror
/// the exact command-line context used by Git.
pub fn hooks_sendemail_validate(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	args: &[&str],
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_SENDEMAIL_VALIDATE, args, None)
}

/// Git invokes `update` on the server side once per ref that is about to be
/// updated.
///
/// Git passes the ref name, the old object id, and the new object id as
/// positional arguments.
pub fn hooks_update(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	ref_name: &str,
	old_oid: &str,
	new_oid: &str,
) -> Result<HookResult> {
	run_hook_stub(
		repo,
		other_paths,
		HOOK_UPDATE,
		[ref_name, old_oid, new_oid].as_slice(),
		None,
	)
}

/// Git invokes `post-update` after refs have been updated.
///
/// The hook receives the updated ref names as positional arguments.
pub fn hooks_post_update(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	refs: &[&str],
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_POST_UPDATE, refs, None)
}

/// Git invokes `pre-receive` before refs are updated on the server side.
///
/// The update list is streamed on stdin in the same format Git uses for
/// receive-pack hooks, so callers can supply that byte stream directly.
pub fn hooks_pre_receive(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	stdin: &[u8],
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_PRE_RECEIVE, &[], Some(stdin))
}

/// Git invokes `post-receive` after refs have been updated on the server side.
///
/// Like `pre-receive`, this hook receives the reference update list on stdin,
/// but it is intended for follow-up work that should only happen after the
/// push has already been accepted.
pub fn hooks_post_receive(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	stdin: &[u8],
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_POST_RECEIVE, &[], Some(stdin))
}

/// Git invokes `proc-receive` for custom server-side ref processing.
///
/// This helper forwards both the positional arguments and the stdin payload
/// unchanged so it can be used as a thin runner for experimental or scripted
/// ref processors.
pub fn hooks_proc_receive(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	args: &[&str],
	stdin: &[u8],
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_PROC_RECEIVE, args, Some(stdin))
}

/// Git invokes `push-to-checkout` when a push needs to update the checked-out
/// worktree directly.
///
/// The hook sees positional arguments describing the update and can inspect or
/// reject the operation by exiting non-zero.
pub fn hooks_push_to_checkout(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	args: &[&str],
) -> Result<HookResult> {
	run_hook_stub(repo, other_paths, HOOK_PUSH_TO_CHECKOUT, args, None)
}

/// Git invokes `reference-transaction` around batches of ref updates.
///
/// The first positional argument usually identifies the transaction stage and
/// the update list is streamed on stdin. This helper forwards both pieces
/// unchanged for consumers that want to inspect or gate ref transitions.
pub fn hooks_reference_transaction(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	args: &[&str],
	stdin: &[u8],
) -> Result<HookResult> {
	run_hook_stub(
		repo,
		other_paths,
		HOOK_REFERENCE_TRANSACTION,
		args,
		Some(stdin),
	)
}

pub enum PrepareCommitMsgSource {
	Message,
	Template,
	Merge,
	Squash,
	Commit(git2::Oid),
}

/// this hook is documented here <https://git-scm.com/docs/githooks#_prepare_commit_msg>
#[allow(clippy::needless_pass_by_value)]
pub fn hooks_prepare_commit_msg(
	repo: &Repository,
	other_paths: Option<&[&str]>,
	source: PrepareCommitMsgSource,
	msg: &mut String,
) -> Result<HookResult> {
	let hook =
		HookPaths::new(repo, other_paths, HOOK_PREPARE_COMMIT_MSG)?;

	if !hook.found() {
		return Ok(HookResult::NoHookFound);
	}

	let temp_file = hook.git.join(HOOK_COMMIT_MSG_TEMP_FILE);
	File::create(&temp_file)?.write_all(msg.as_bytes())?;

	let temp_file_path = temp_file.as_os_str().to_string_lossy();

	let vec = vec![
		temp_file_path.as_ref(),
		match source {
			PrepareCommitMsgSource::Message => "message",
			PrepareCommitMsgSource::Template => "template",
			PrepareCommitMsgSource::Merge => "merge",
			PrepareCommitMsgSource::Squash => "squash",
			PrepareCommitMsgSource::Commit(_) => "commit",
		},
	];
	let mut args = vec;

	let id = if let PrepareCommitMsgSource::Commit(id) = &source {
		Some(id.to_string())
	} else {
		None
	};

	if let Some(id) = &id {
		args.push(id);
	}

	let res = hook.run_hook(args.as_slice())?;

	// load possibly altered msg
	msg.clear();
	File::open(temp_file)?.read_to_string(msg)?;

	Ok(res)
}

#[cfg(test)]
mod tests {
	use std::process::Command;

	use super::*;
	use git2_testing::{repo_init, repo_init_bare};
	use pretty_assertions::assert_eq;
	use tempfile::TempDir;

	#[cfg(not(windows))]
	fn python_interpreter() -> Option<&'static str> {
		["python3", "python"]
			.into_iter()
			.find(|candidate| {
				Command::new(candidate)
					.arg("--version")
					.output()
					.is_ok()
			})
	}

	fn branch_update(
		repo: &Repository,
		remote: Option<&str>,
		branch: &str,
		remote_branch: Option<&str>,
		delete: bool,
	) -> PrePushRef {
		let local_ref = format!("refs/heads/{branch}");
		let local_oid = (!delete).then(|| {
			repo.find_branch(branch, git2::BranchType::Local)
				.unwrap()
				.get()
				.peel_to_commit()
				.unwrap()
				.id()
		});

		let remote_branch = remote_branch.unwrap_or(branch);
		let remote_ref = format!("refs/heads/{remote_branch}");
		let remote_oid = remote.and_then(|remote_name| {
			repo.find_reference(&format!(
				"refs/remotes/{remote_name}/{remote_branch}"
			))
			.ok()
			.and_then(|r| r.peel_to_commit().ok())
			.map(|c| c.id())
		});

		PrePushRef::new(local_ref, local_oid, remote_ref, remote_oid)
	}

	fn head_branch(repo: &Repository) -> String {
		repo.head().unwrap().shorthand().unwrap().to_string()
	}

	#[test]
	fn test_pre_push_ref_format() {
		let zero_oid = "0".repeat(40);
		let oid_a = "a".repeat(40);
		let oid_b = "b".repeat(40);

		// Both oids present
		let update = PrePushRef::new(
			"refs/heads/main",
			Some(git2::Oid::from_str(&oid_a).unwrap()),
			"refs/heads/main",
			Some(git2::Oid::from_str(&oid_b).unwrap()),
		);
		assert_eq!(
			update.to_line(),
			format!(
				"refs/heads/main {oid_a} refs/heads/main {oid_b}"
			)
		);

		// No remote oid (new branch)
		let update = PrePushRef::new(
			"refs/heads/feature",
			Some(git2::Oid::from_str(&oid_a).unwrap()),
			"refs/heads/feature",
			None,
		);
		assert_eq!(
			update.to_line(),
			format!("refs/heads/feature {oid_a} refs/heads/feature {zero_oid}")
		);

		// No local oid (delete)
		let update = PrePushRef::new(
			"refs/heads/old",
			None,
			"refs/heads/old",
			Some(git2::Oid::from_str(&oid_b).unwrap()),
		);
		assert_eq!(
			update.to_line(),
			format!(
				"refs/heads/old {zero_oid} refs/heads/old {oid_b}"
			)
		);

		// to_stdin adds newlines
		let updates = [
			PrePushRef::new(
				"refs/heads/a",
				Some(git2::Oid::from_str(&oid_a).unwrap()),
				"refs/heads/a",
				None,
			),
			PrePushRef::new(
				"refs/heads/b",
				Some(git2::Oid::from_str(&oid_b).unwrap()),
				"refs/heads/b",
				None,
			),
		];
		assert_eq!(
			PrePushRef::to_stdin(&updates),
			format!(
				"refs/heads/a {oid_a} refs/heads/a {zero_oid}\nrefs/heads/b {oid_b} refs/heads/b {zero_oid}\n"
			)
		);
	}

	#[test]
	fn test_smoke() {
		let (_td, repo) = repo_init();

		let mut msg = String::from("test");
		let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

		assert_eq!(res, HookResult::NoHookFound);

		let hook = b"#!/bin/sh
exit 0
        ";

		create_hook(&repo, HOOK_POST_COMMIT, hook);

		let res = hooks_post_commit(&repo, None).unwrap();

		assert!(res.is_successful());
	}

	#[test]
	fn test_hooks_commit_msg_ok() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
exit 0
        ";

		create_hook(&repo, HOOK_COMMIT_MSG, hook);

		let mut msg = String::from("test");
		let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

		assert!(res.is_successful());

		assert_eq!(msg, String::from("test"));
	}

	#[test]
	fn test_hooks_commit_msg_with_shell_command_ok() {
		let (_td, repo) = repo_init();

		let hook = br#"#!/bin/sh
COMMIT_MSG="$(cat "$1")"
printf "$COMMIT_MSG" | sed 's/sth/shell_command/g' > "$1"
exit 0
        "#;

		create_hook(&repo, HOOK_COMMIT_MSG, hook);

		let mut msg = String::from("test_sth");
		let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

		assert!(res.is_successful());

		assert_eq!(msg, String::from("test_shell_command"));
	}

	#[test]
	fn test_pre_commit_sh() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
exit 0
        ";

		create_hook(&repo, HOOK_PRE_COMMIT, hook);
		let res = hooks_pre_commit(&repo, None).unwrap();
		assert!(res.is_successful());
	}

	#[test]
	fn test_hook_with_missing_shebang() {
		const TEXT: &str = "Hello, world!";

		let (_td, repo) = repo_init();

		let hook = b"echo \"$@\"\nexit 42";

		create_hook(&repo, HOOK_PRE_COMMIT, hook);

		let hook =
			HookPaths::new(&repo, None, HOOK_PRE_COMMIT).unwrap();

		assert!(hook.found());

		let result = hook.run_hook(&[TEXT]).unwrap();

		let HookResult::Run(response) = result else {
			unreachable!("run_hook should've run");
		};

		let stdout = response.stdout.as_str().trim_ascii_end();

		assert_eq!(response.code, 42);
		assert_eq!(response.hook, hook.hook);
		assert_eq!(stdout, TEXT, "{:?} != {TEXT:?}", stdout);
		assert!(response.stderr.is_empty());
	}

	#[test]
	fn test_no_hook_found() {
		let (_td, repo) = repo_init();

		let res = hooks_pre_commit(&repo, None).unwrap();
		assert_eq!(res, HookResult::NoHookFound);
	}

	#[test]
	fn test_other_path() {
		let (td, repo) = repo_init();

		let hook = b"#!/bin/sh
exit 0
        ";

		let custom_hooks_path = td.path().join(".myhooks");

		std::fs::create_dir(dbg!(&custom_hooks_path)).unwrap();
		create_hook_in_path(
			dbg!(custom_hooks_path.join(HOOK_PRE_COMMIT).as_path()),
			hook,
		);

		let res =
			hooks_pre_commit(&repo, Some(&["../.myhooks"])).unwrap();

		assert!(res.is_successful());
	}

	#[test]
	fn test_other_path_precedence() {
		let (td, repo) = repo_init();

		{
			let hook = b"#!/bin/sh
exit 0
        ";

			create_hook(&repo, HOOK_PRE_COMMIT, hook);
		}

		{
			let reject_hook = b"#!/bin/sh
exit 1
        ";

			let custom_hooks_path = td.path().join(".myhooks");
			std::fs::create_dir(dbg!(&custom_hooks_path)).unwrap();
			create_hook_in_path(
				dbg!(custom_hooks_path
					.join(HOOK_PRE_COMMIT)
					.as_path()),
				reject_hook,
			);
		}

		let res =
			hooks_pre_commit(&repo, Some(&["../.myhooks"])).unwrap();

		assert!(res.is_successful());
	}

	#[test]
	fn test_pre_commit_fail_sh() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
echo 'rejected'
exit 1
        ";

		create_hook(&repo, HOOK_PRE_COMMIT, hook);
		let res = hooks_pre_commit(&repo, None).unwrap();
		assert!(!res.is_successful());
	}

	#[test]
	fn test_env_containing_path() {
		const PATH_EXPORT: &str = "export PATH";

		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
export
exit 1
        ";

		create_hook(&repo, HOOK_PRE_COMMIT, hook);
		let res = hooks_pre_commit(&repo, None).unwrap();

		let HookResult::Run(response) = res else {
			unreachable!()
		};

		assert!(
			response
				.stdout
				.lines()
				.any(|line| line.starts_with(PATH_EXPORT)),
			"Could not find line starting with {PATH_EXPORT:?} in: {:?}",
			response.stdout
		);
	}

	#[test]
	fn test_pre_commit_fail_hookspath() {
		let (_td, repo) = repo_init();
		let hooks = TempDir::new().unwrap();

		let hook = b"#!/bin/sh
echo 'rejected'
exit 1
        ";

		create_hook_in_path(&hooks.path().join("pre-commit"), hook);

		repo.config()
			.unwrap()
			.set_str(
				"core.hooksPath",
				hooks.path().as_os_str().to_str().unwrap(),
			)
			.unwrap();

		let res = hooks_pre_commit(&repo, None).unwrap();

		let HookResult::Run(response) = res else {
			unreachable!()
		};

		assert_eq!(response.code, 1);
		assert_eq!(&response.stdout, "rejected\n");
	}

	#[test]
	fn test_pre_commit_fail_bare() {
		let (_td, repo) = repo_init_bare();

		let hook = b"#!/bin/sh
echo 'rejected'
exit 1
        ";

		create_hook(&repo, HOOK_PRE_COMMIT, hook);
		let res = hooks_pre_commit(&repo, None).unwrap();
		assert!(!res.is_successful());
	}

	#[test]
	fn test_pre_commit_py() {
		let (_td, repo) = repo_init();

		// mirror how python pre-commit sets itself up
		#[cfg(not(windows))]
		let Some(python) = python_interpreter() else {
			return;
		};
		#[cfg(not(windows))]
		let hook = format!(
			"#!/usr/bin/env {python}\n\
import sys
sys.exit(0)
        "
		);
		#[cfg(windows)]
		let hook = b"#!/bin/env python.exe
import sys
sys.exit(0)
        ";

		#[cfg(not(windows))]
		create_hook(&repo, HOOK_PRE_COMMIT, hook.as_bytes());
		#[cfg(windows)]
		create_hook(&repo, HOOK_PRE_COMMIT, hook);
		let res = hooks_pre_commit(&repo, None).unwrap();
		assert!(res.is_successful(), "{res:?}");
	}

	#[test]
	fn test_pre_commit_fail_py() {
		let (_td, repo) = repo_init();

		// mirror how python pre-commit sets itself up
		#[cfg(not(windows))]
		let Some(python) = python_interpreter() else {
			return;
		};
		#[cfg(not(windows))]
		let hook = format!(
			"#!/usr/bin/env {python}\n\
import sys
sys.exit(1)
        "
		);
		#[cfg(windows)]
		let hook = b"#!/bin/env python.exe
import sys
sys.exit(1)
        ";

		#[cfg(not(windows))]
		create_hook(&repo, HOOK_PRE_COMMIT, hook.as_bytes());
		#[cfg(windows)]
		create_hook(&repo, HOOK_PRE_COMMIT, hook);
		let res = hooks_pre_commit(&repo, None).unwrap();
		assert!(!res.is_successful());
	}

	#[test]
	fn test_hooks_commit_msg_reject() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
	echo 'msg' > \"$1\"
	echo 'rejected'
	exit 1
        ";

		create_hook(&repo, HOOK_COMMIT_MSG, hook);

		let mut msg = String::from("test");
		let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

		let HookResult::Run(response) = res else {
			unreachable!()
		};

		assert_eq!(response.code, 1);
		assert_eq!(&response.stdout, "rejected\n");

		assert_eq!(msg, String::from("msg\n"));
	}

	#[test]
	fn test_commit_msg_no_block_but_alter() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
echo 'msg' > \"$1\"
exit 0
        ";

		create_hook(&repo, HOOK_COMMIT_MSG, hook);

		let mut msg = String::from("test");
		let res = hooks_commit_msg(&repo, None, &mut msg).unwrap();

		assert!(res.is_successful());
		assert_eq!(msg, String::from("msg\n"));
	}

	#[test]
	fn test_hook_pwd_in_bare_without_workdir() {
		let (_td, repo) = repo_init_bare();
		let git_root = repo.path().to_path_buf();

		let hook =
			HookPaths::new(&repo, None, HOOK_POST_COMMIT).unwrap();

		assert_eq!(hook.pwd, git_root);
	}

	#[test]
	fn test_hook_pwd() {
		let (_td, repo) = repo_init();
		let git_root = repo.path().to_path_buf();

		let hook =
			HookPaths::new(&repo, None, HOOK_POST_COMMIT).unwrap();

		assert_eq!(hook.pwd, git_root.parent().unwrap());
	}

	#[test]
	fn test_hooks_prep_commit_msg_success() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
echo \"msg:$2\" > \"$1\"
exit 0
        ";

		create_hook(&repo, HOOK_PREPARE_COMMIT_MSG, hook);

		let mut msg = String::from("test");
		let res = hooks_prepare_commit_msg(
			&repo,
			None,
			PrepareCommitMsgSource::Message,
			&mut msg,
		)
		.unwrap();

		assert!(res.is_successful());
		assert_eq!(msg, String::from("msg:message\n"));
	}

	#[test]
	fn test_hooks_prep_commit_msg_reject() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
echo \"$2,$3\" > \"$1\"
echo 'rejected'
exit 2
        ";

		create_hook(&repo, HOOK_PREPARE_COMMIT_MSG, hook);

		let mut msg = String::from("test");
		let res = hooks_prepare_commit_msg(
			&repo,
			None,
			PrepareCommitMsgSource::Commit(git2::Oid::zero()),
			&mut msg,
		)
		.unwrap();

		let HookResult::Run(response) = res else {
			unreachable!()
		};

		assert_eq!(response.code, 2);
		assert_eq!(&response.stdout, "rejected\n");

		assert_eq!(
			msg,
			String::from(
				"commit,0000000000000000000000000000000000000000\n"
			)
		);
	}

	#[test]
	fn test_pre_push_sh() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
exit 0
	";

		create_hook(&repo, HOOK_PRE_PUSH, hook);

		let branch = head_branch(&repo);
		let updates = [branch_update(
			&repo,
			Some("origin"),
			&branch,
			None,
			false,
		)];

		let res = hooks_pre_push(
			&repo,
			None,
			Some("origin"),
			"https://example.com/repo.git",
			&updates,
		)
		.unwrap();

		assert!(res.is_successful());
	}

	#[test]
	fn test_pre_push_fail_sh() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
echo 'failed'
exit 3
	";
		create_hook(&repo, HOOK_PRE_PUSH, hook);

		let branch = head_branch(&repo);
		let updates = [branch_update(
			&repo,
			Some("origin"),
			&branch,
			None,
			false,
		)];

		let res = hooks_pre_push(
			&repo,
			None,
			Some("origin"),
			"https://example.com/repo.git",
			&updates,
		)
		.unwrap();
		let HookResult::Run(response) = res else {
			unreachable!()
		};
		assert_eq!(response.code, 3);
		assert_eq!(&response.stdout, "failed\n");
	}

	#[test]
	fn test_pre_push_no_remote_name() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
# Verify that when remote is None, URL is passed for both arguments
echo \"arg1=$1 arg2=$2\"
exit 0
	";

		create_hook(&repo, HOOK_PRE_PUSH, hook);

		let branch = head_branch(&repo);
		let updates =
			[branch_update(&repo, None, &branch, None, false)];

		let res = hooks_pre_push(
			&repo,
			None,
			None,
			"https://example.com/repo.git",
			&updates,
		)
		.unwrap();

		let HookResult::Run(response) = res else {
			panic!("Expected Run result, got: {res:?}");
		};

		assert!(response.is_successful());
		// When remote is None, URL should be passed for both arguments
		assert_eq!(
			response.stdout,
			"arg1=https://example.com/repo.git arg2=https://example.com/repo.git\n"
		);
	}

	#[test]
	fn test_pre_push_with_arguments() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
echo \"remote_name=$1\"
echo \"remote_url=$2\"
exit 0
	";

		create_hook(&repo, HOOK_PRE_PUSH, hook);

		let branch = head_branch(&repo);
		let updates = [branch_update(
			&repo,
			Some("origin"),
			&branch,
			None,
			false,
		)];

		let res = hooks_pre_push(
			&repo,
			None,
			Some("origin"),
			"https://example.com/repo.git",
			&updates,
		)
		.unwrap();

		let HookResult::Run(response) = res else {
			unreachable!("Expected Run result, got: {res:?}")
		};

		assert!(response.is_successful());
		assert_eq!(
			response.stdout,
			"remote_name=origin\nremote_url=https://example.com/repo.git\n"
		);
	}

	#[test]
	fn test_pre_push_multiple_updates() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
cat
exit 0
	";

		create_hook(&repo, HOOK_PRE_PUSH, hook);

		let branch = head_branch(&repo);
		let branch_update = branch_update(
			&repo,
			Some("origin"),
			&branch,
			None,
			false,
		);

		// create a tag to add a second refspec
		let head_commit =
			repo.head().unwrap().peel_to_commit().unwrap();
		repo.tag_lightweight("v1", head_commit.as_object(), false)
			.unwrap();
		let tag_ref = repo.find_reference("refs/tags/v1").unwrap();
		let tag_oid = tag_ref.target().unwrap();
		let tag_update = PrePushRef::new(
			"refs/tags/v1",
			Some(tag_oid),
			"refs/tags/v1",
			None,
		);

		let updates = [branch_update, tag_update];
		let expected_stdin = PrePushRef::to_stdin(&updates);

		let res = hooks_pre_push(
			&repo,
			None,
			Some("origin"),
			"https://example.com/repo.git",
			&updates,
		)
		.unwrap();

		let HookResult::Run(response) = res else {
			unreachable!("Expected Run result, got: {res:?}")
		};

		assert!(
			response.is_successful(),
			"Hook should succeed: stdout {} stderr {}",
			response.stdout,
			response.stderr
		);
		assert_eq!(
			response.stdout, expected_stdin,
			"stdin should include all refspec lines"
		);
	}

	#[test]
	fn test_pre_push_delete_ref_uses_zero_oid() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
cat
exit 0
	";

		create_hook(&repo, HOOK_PRE_PUSH, hook);

		let branch = head_branch(&repo);
		let updates = [branch_update(
			&repo,
			Some("origin"),
			&branch,
			None,
			true,
		)];
		let expected_stdin = PrePushRef::to_stdin(&updates);

		let res = hooks_pre_push(
			&repo,
			None,
			Some("origin"),
			"https://example.com/repo.git",
			&updates,
		)
		.unwrap();

		let HookResult::Run(response) = res else {
			unreachable!("Expected Run result, got: {res:?}")
		};

		assert!(response.is_successful());
		assert_eq!(response.stdout, expected_stdin);
	}

	#[test]
	fn test_pre_push_stdin() {
		let (_td, repo) = repo_init();

		let hook = b"#!/bin/sh
cat
exit 0
		";

		create_hook(&repo, HOOK_PRE_PUSH, hook);

		let branch = head_branch(&repo);
		let updates = [branch_update(
			&repo,
			Some("origin"),
			&branch,
			None,
			false,
		)];
		let expected_stdin = PrePushRef::to_stdin(&updates);

		let res = hooks_pre_push(
			&repo,
			None,
			Some("origin"),
			"https://github.com/user/repo.git",
			&updates,
		)
		.unwrap();

		let HookResult::Run(response) = res else {
			unreachable!("Expected Run result, got: {res:?}")
		};

		assert!(response.is_successful());
		assert_eq!(response.stdout, expected_stdin);
	}

	#[test]
	fn test_pre_push_uses_push_target_remote_not_upstream() {
		let (_td, repo) = repo_init();

		// repo_init() already creates an initial commit on master
		let head = repo.head().unwrap();
		let local_commit = head.target().unwrap();

		// Set up scenario:
		// - Local master is at local_commit (latest)
		// - origin/master exists at local_commit (fully synced - upstream)
		// - backup/master exists at old_commit (behind/different)
		// - Branch tracks origin/master as upstream
		// - We push to "backup" remote
		// - Expected: remote SHA should be old_commit (not origin/master)

		// Create origin/master tracking branch (at same commit as local)
		repo.reference(
			"refs/remotes/origin/master",
			local_commit,
			true,
			"create origin/master",
		)
		.unwrap();

		// Create backup/master at a different commit
		let sig = repo.signature().unwrap();
		let tree_id = {
			let mut index = repo.index().unwrap();
			index.write_tree().unwrap()
		};
		let tree = repo.find_tree(tree_id).unwrap();
		let old_commit = repo
			.commit(None, &sig, &sig, "old backup commit", &tree, &[])
			.unwrap();

		repo.reference(
			"refs/remotes/backup/master",
			old_commit,
			true,
			"create backup/master at old commit",
		)
		.unwrap();

		// Configure upstream to origin
		{
			let mut config = repo.config().unwrap();
			config.set_str("branch.master.remote", "origin").unwrap();
			config
				.set_str("branch.master.merge", "refs/heads/master")
				.unwrap();
		}

		let hook = b"#!/bin/sh
cat
exit 0
";

		create_hook(&repo, HOOK_PRE_PUSH, hook);

		let branch = head_branch(&repo);
		let updates = [branch_update(
			&repo,
			Some("backup"),
			&branch,
			None,
			false,
		)];
		let expected_stdin = PrePushRef::to_stdin(&updates);

		let res = hooks_pre_push(
			&repo,
			None,
			Some("backup"),
			"https://github.com/user/backup-repo.git",
			&updates,
		)
		.unwrap();

		let HookResult::Run(response) = res else {
			panic!("Expected Run result, got: {res:?}")
		};

		assert!(response.is_successful());
		assert_eq!(response.stdout, expected_stdin);
	}
}
