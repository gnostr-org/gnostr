use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;
// use git2::*;
use super::worker::Worker;
use gnostr_asyncgit::types::Keys;
use gnostr_types::nostr::patch::{get_relay_urls, publish_patch_event};
use time_0_3::OffsetDateTime;
//TODO use gnostr_asyncgit::types for event creation and injection during the mining loop
#[derive(Clone, Debug)]
pub struct Options {
    pub threads: u32,
    pub target: String,
    pub message: Vec<String>,
    pub repo: String,
    pub timestamp: OffsetDateTime,
    pub kind: Option<u16>,
}

pub struct Gitminer {
    opts: Options,
    repo: gnostr_asyncgit::git2::Repository,
    author: String,
    signing_keys: Option<Keys>,
}

impl std::fmt::Debug for Gitminer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gitminer")
            .field("opts", &self.opts)
            .field("repo_path", &self.repo.path())
            .field("author", &self.author)
            .field(
                "signing_pubkey",
                &self
                    .signing_keys
                    .as_ref()
                    .map(|keys| keys.public_key().as_hex_string()),
            )
            .finish()
    }
}

impl Clone for Gitminer {
    fn clone(&self) -> Self {
        debug!("Reopening repository {} for miner clone", self.opts.repo);
        let repo = gnostr_asyncgit::git2::Repository::open(&self.opts.repo)
            .expect("Failed to open repository during clone");
        Self {
            opts: self.opts.clone(),
            repo,
            author: self.author.clone(),
            signing_keys: self.signing_keys.clone(),
        }
    }
}

impl Gitminer {
    pub fn new(opts: Options) -> Result<Gitminer, &'static str> {
        debug!("Opening mining repository {}", opts.repo);
        let repo = match gnostr_asyncgit::git2::Repository::open(&opts.repo) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to open repository: {}", e);
                return Err("Failed to open repository");
            }
        };
        debug!("Opened mining repository at {}", repo.path().display());

        let author = Gitminer::load_author(&repo)?;
        debug!("Gitminer initialized with author: {}", author);

        Ok(Gitminer {
            opts,
            repo,
            author,
            signing_keys: None,
        })
    }

    /// Use explicit keys instead of the commit-derived key for NIP-34 publication.
    pub fn with_signing_keys(mut self, keys: Keys) -> Self {
        self.signing_keys = Some(keys);
        self
    }

    pub fn mine(&mut self) -> Result<String, &'static str> {
        debug!("Starting mining process with options: {:?}", self.opts);
        info!(
            "Mining commit in {} with target {} using {} thread(s)",
            self.opts.repo, self.opts.target, self.opts.threads
        );
        let (tree, parent) = match Gitminer::prepare_tree(&mut self.repo) {
            Ok((t, p)) => (t, p),
            Err(e) => {
                error!("Failed to prepare tree: {}", e);
                return Err(e);
            }
        };
        debug!("Tree: {}, Parent: {}", tree, parent);
        info!("Prepared tree {} and parent {}", tree, parent);

        let (tx, rx) = channel();

        for i in 0..self.opts.threads {
            let target = self.opts.target.clone();
            let author = self.author.clone();
            let msg = match self.opts.message.as_slice() {
                [] => String::new(),
                [one] => one.clone(),
                [first, rest @ ..] => format!("{first}\n\n{}", rest.join("\n")),
            };
            let wtx = tx.clone();
            let ts = self.opts.timestamp.clone();
            let (wtree, wparent) = (tree.clone(), parent.clone());

            debug!("Spawning worker {}", i);
            info!("Spawning worker {} for target {}", i, target);
            thread::spawn(move || {
                Worker::new(i, target, wtree, wparent, author, msg, ts, wtx).work();
            });
        }

        info!("Waiting for a worker to find a valid commit hash...");
        let (worker_id, blob, hash) = rx.recv().unwrap();
        debug!(
            "Received mined commit from worker {}: hash={}, bytes={}",
            worker_id,
            hash,
            blob.len()
        );
        info!("Received hash {} from a worker.", hash);

        match self.write_commit(&hash, &blob) {
            Ok(_) => {
                self.send_nip34_patch_event(&hash)?;
                Ok(hash)
            }
            Err(e) => {
                error!("Failed to write commit: {}", e);
                Err(e)
            }
        }
    }

    fn write_commit(&self, hash: &String, blob: &String) -> Result<(), &'static str> {
        debug!(
            "Persisting mined commit {} ({} bytes) in {}",
            hash,
            blob.len(),
            self.opts.repo
        );
        Gitminer::ensure_gnostr_dirs_exist(Path::new(&self.opts.repo))?;
        debug!("Writing commit for hash: {}", hash);
        info!("Writing mined commit {}", hash);
        /* repo.blob() generates a blob, not a commit.
         * don't know if there's a way to do this with libgit2. */
        let tmpfile = format!("/tmp/{}.tmp", hash);
        debug!("Creating temporary file: {}", tmpfile);
        let mut file = match File::create(&Path::new(&tmpfile)) {
            Ok(f) => f,
            Err(_) => return Err("Failed to create temporary file"),
        };

        if let Err(_) = file.write_all(blob.as_bytes()) {
            return Err("Failed to write to temporary file");
        }
        debug!("Temporary file {} written.", tmpfile);

        // Write the blob to .gnostr/blobs/<commit_hash>
        let gnostr_blob_path = Path::new(&self.opts.repo).join(".gnostr/blobs").join(hash);
        debug!("Creating .gnostr blob file: {}", gnostr_blob_path.display());
        let mut gnostr_blob_file = match File::create(&gnostr_blob_path) {
            Ok(f) => f,
            Err(_) => return Err("Failed to create .gnostr blob file"),
        };
        if let Err(_) = gnostr_blob_file.write_all(blob.as_bytes()) {
            return Err("Failed to write to .gnostr blob file");
        }
        debug!(".gnostr blob file {} written.", gnostr_blob_path.display());

        // Write the blob to .gnostr/reflog/<commit_hash>
        let gnostr_reflog_path = Path::new(&self.opts.repo).join(".gnostr/reflog").join(hash);
        debug!(
            "Creating .gnostr reflog file: {}",
            gnostr_reflog_path.display()
        );
        let mut gnostr_reflog_file = match File::create(&gnostr_reflog_path) {
            Ok(f) => f,
            Err(_) => return Err("Failed to create .gnostr reflog file"),
        };
        if let Err(_) = gnostr_reflog_file.write_all(blob.as_bytes()) {
            return Err("Failed to write to .gnostr reflog file");
        }
        debug!(
            ".gnostr reflog file {} written.",
            gnostr_reflog_path.display()
        );

        let command_str = format!(
            "cd {} && git hash-object -t commit -w --stdin < {} && git reset --hard {}",
            self.opts.repo, tmpfile, hash
        );
        debug!("Executing git command: {}", command_str);
        let output = Command::new("sh")
            .arg("-c")
            .arg(&command_str)
            .output()
            .map_err(|_| "Failed to execute git command")?;
        debug!(
            "Git write/reset command exited with status {}; stdout={}",
            output.status,
            String::from_utf8_lossy(&output.stdout)
        );

        if !output.status.success() {
            eprintln!("Failed git command: {}", command_str);
            error!("Git command failed: {:?}", output);
            return Err("Git command failed");
        }
        info!("Git command executed successfully.");
        Ok(())
    }

    fn send_nip34_patch_event(&self, head: &str) -> Result<(), &'static str> {
        info!("Preparing NIP-34 patch event for mined commit {}", head);
        let keys = match &self.signing_keys {
            Some(keys) => keys.clone(),
            None => Keys::parse(format!("{:0>64}", head))
                .ok_or("Failed to derive Nostr keys from mined commit")?,
        };
        info!(
            "NIP-34 signing public key: {} (source={})",
            keys.public_key(),
            if self.signing_keys.is_some() {
                "--nsec"
            } else {
                "mined commit"
            }
        );
        let relay_urls = get_relay_urls();
        debug!(
            "Selected {} relay(s) for NIP-34 publication of commit {}",
            relay_urls.len(),
            head
        );
        for (index, relay_url) in relay_urls.iter().enumerate() {
            debug!(
                "Selected NIP-34 relay {}/{} for commit {}: {}",
                index + 1,
                relay_urls.len(),
                head,
                relay_url
            );
        }
        let patch_content = self.patch_content_for_commit(head)?;
        info!(
            "Fetched patch content for {} ({} bytes)",
            head,
            patch_content.len()
        );
        let repo_name = Path::new(&self.opts.repo)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(self.opts.repo.as_str())
            .to_string();
        let head_hash = head.to_string();
        let repo_path = self.opts.repo.clone();

        // Spawn a dedicated OS thread with its own Tokio runtime so that
        // this blocking call is safe whether or not the caller already runs
        // inside a Tokio runtime (e.g. during #[tokio::test]).
        let event_id = std::thread::spawn(move || {
            debug!(
                "Creating dedicated NIP-34 publishing runtime for {}",
                head_hash
            );
            let runtime = tokio::runtime::Runtime::new().expect("Failed to create async runtime");
            runtime.block_on(async move {
                info!(
                    "Publishing NIP-34 patch event for {} to {} relay(s)",
                    repo_name,
                    relay_urls.len()
                );
                let event_id = publish_patch_event(
                    &keys,
                    &relay_urls,
                    &repo_name,
                    &head_hash,
                    &patch_content,
                    None,
                )
                .await
                .map_err(|err| {
                    error!(
                        "Failed to publish NIP-34 patch event for commit {}: {}",
                        head_hash, err
                    );
                    "Failed to publish NIP-34 patch event"
                })?;
                info!(
                    "Published NIP-34 patch event: id={}, commit={}, repository={}",
                    event_id, head_hash, repo_name
                );
                Ok::<_, &'static str>(event_id)
            })
        })
        .join()
        .map_err(|_| "NIP-34 patch event thread panicked")??;

        debug!("NIP-34 publishing thread joined for {}", head);
        info!(
            "Submitted NIP-34 patch event {} for {} (commit {})",
            event_id, repo_path, head
        );
        Ok(())
    }

    fn patch_content_for_commit(&self, head: &str) -> Result<String, &'static str> {
        debug!("Generating patch for commit {} in {}", head, self.opts.repo);
        let output = Command::new("git")
            .args([
                "show",
                "--format=medium",
                "--patch",
                "--no-ext-diff",
                "--no-color",
                head,
            ])
            .current_dir(&self.opts.repo)
            .output()
            .map_err(|_| "Failed to generate patch content")?;

        if !output.status.success() {
            error!(
                "git show failed for {}: status={}, stderr={}",
                head,
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            return Err("Failed to generate patch content");
        }

        debug!("Generated {} patch bytes for {}", output.stdout.len(), head);
        String::from_utf8(output.stdout).map_err(|_| "Patch content was not valid UTF-8")
    }

    fn load_author(repo: &gnostr_asyncgit::git2::Repository) -> Result<String, &'static str> {
        debug!("Loading author from git config.");
        let cfg = match repo.config() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load git config: {}", e);
                return Err("Failed to load git config");
            }
        };

        let name = match cfg.get_string("user.name") {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to find git user name: {}", e);
                return Err("Failed to find git user name");
            }
        };
        debug!("Found git user name: {}", name);

        let email = match cfg.get_string("user.email") {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to find git email address: {}", e);
                return Err("Failed to find git email address");
            }
        };
        debug!("Found git email address: {}", email);

        Ok(format!("{} <{}>", name, email))
    }

    fn ensure_gnostr_dirs_exist(repo_root_path: &Path) -> Result<(), &'static str> {
        debug!(
            "Ensuring .gnostr directories exist in: {}",
            repo_root_path.display()
        );
        let gnostr_path = repo_root_path.join(".gnostr");
        let blobs_path = gnostr_path.join("blobs");
        let reflog_path = gnostr_path.join("reflog");

        std::fs::create_dir_all(&gnostr_path).map_err(|_| "Failed to create .gnostr directory")?;
        std::fs::create_dir_all(&blobs_path)
            .map_err(|_| "Failed to create .gnostr/blobs directory")?;
        std::fs::create_dir_all(&reflog_path)
            .map_err(|_| "Failed to create .gnostr/reflog directory")?;

        debug!("Storage directories ready in {}", gnostr_path.display());
        Ok(())
    }

    fn prepare_tree(
        repo: &mut gnostr_asyncgit::git2::Repository,
    ) -> Result<(String, String), &'static str> {
        debug!("Preparing tree.");
        Gitminer::ensure_no_unstaged_changes(repo)?;

        let head = match repo.revparse_single("HEAD") {
            Ok(h) => h,
            Err(_) => return Err("Failed to get HEAD"),
        };
        let mut index = match repo.index() {
            Ok(i) => i,
            Err(_) => return Err("Failed to get index"),
        };
        let tree = match index.write_tree() {
            Ok(t) => t,
            Err(_) => return Err("Failed to write tree"),
        };

        let head_s = format!("{}", head.id());
        let tree_s = format!("{}", tree);
        debug!("Head: {}, Tree: {}", head_s, tree_s);

        Ok((tree_s, head_s))
    }

    fn ensure_no_unstaged_changes(
        repo: &mut gnostr_asyncgit::git2::Repository,
    ) -> Result<(), &'static str> {
        debug!("Checking for unstaged changes in {}", repo.path().display());
        let mut opts = gnostr_asyncgit::git2::StatusOptions::new();
        let mut m = gnostr_asyncgit::git2::Status::empty();
        let statuses = match repo.statuses(Some(&mut opts)) {
            Ok(s) => s,
            Err(_) => return Err("Failed to get statuses"),
        };
        debug!("Inspecting {} repository status entries", statuses.len());

        m.insert(gnostr_asyncgit::git2::Status::WT_NEW);
        m.insert(gnostr_asyncgit::git2::Status::WT_MODIFIED);
        m.insert(gnostr_asyncgit::git2::Status::WT_DELETED);
        m.insert(gnostr_asyncgit::git2::Status::WT_RENAMED);
        m.insert(gnostr_asyncgit::git2::Status::WT_TYPECHANGE);

        for i in 0..statuses.len() {
            let status_entry = match statuses.get(i) {
                Some(s) => s,
                None => return Err("Failed to get status entry"),
            };
            if status_entry.status().intersects(m) {
                debug!(
                    "Unstaged change blocks mining: path={:?}, status={:?}",
                    status_entry.path(),
                    status_entry.status()
                );
                return Err("Please stash all unstaged changes before running.");
            }
        }

        debug!("No unstaged changes found.");
        Ok(())
    }
}
