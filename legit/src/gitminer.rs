use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;
// use git2::*;
use super::worker::Worker;
use gnostr_asyncgit::types::Keys;
use gnostr_types::nostr::patch::{
    build_patch_announcement_with_tags, commit_nonce_tag, get_relay_urls,
    publish_patch_event_with_tags, ParentCommitLink,
};
use gnostr_types::nostr::{
    client::Options as ClientOptions, patch::install_rustls_crypto_provider, Client, Event,
    EventBuilder, EventKind, Filter, Id, Metadata, PublicKey, Tag,
};
use std::time::Duration;
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
    depth: usize,
}

impl std::fmt::Debug for Gitminer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gitminer")
            .field("opts", &self.opts)
            .field("repo_path", &self.repo.path())
            .field("author", &self.author)
            .field("depth", &self.depth)
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
            depth: self.depth,
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
            depth: 0,
        })
    }

    /// Use explicit keys instead of the commit-derived key for NIP-34 publication.
    pub fn with_signing_keys(mut self, keys: Keys) -> Self {
        self.signing_keys = Some(keys);
        self
    }

    /// Append up to `depth` first-parent commits, listing every parent of merges.
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    fn mining_message(&self, parent: &str) -> Result<String, gnostr_asyncgit::git2::Error> {
        let mut message = match self.opts.message.as_slice() {
            [] => String::new(),
            [one] => one.clone(),
            [first, rest @ ..] => format!("{first}\n\n{}", rest.join("\n")),
        };
        if self.depth == 0 {
            return Ok(message);
        }

        let mut history = self.repo.revwalk()?;
        history.push(gnostr_asyncgit::git2::Oid::from_str(parent)?)?;
        history.simplify_first_parent()?;
        let mut entries = Vec::new();
        let mut commits = 0;
        for id in history.take(self.depth) {
            let commit = self.repo.find_commit(id?)?;
            if commit.parent_count() > 1 {
                for parent in commit.parent_ids() {
                    entries.push(format!("{} {}", parent, commit.id()));
                }
            } else {
                entries.push(commit.id().to_string());
            }
            commits += 1;
        }
        for entry in &entries {
            debug!("Including history entry {} in mined message", entry);
        }
        info!(
            "Including {} first-parent commit(s) in {} history line(s), requested depth={}",
            commits,
            entries.len(),
            self.depth
        );
        if !entries.is_empty() {
            if !message.is_empty() {
                message.push_str("\n\n");
            }
            message.push_str(&entries.join("\n"));
        }
        Ok(message)
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
        let message = self.mining_message(&parent).map_err(|err| {
            error!(
                "Failed to prepare commit history for parent {}: {}",
                parent, err
            );
            "Failed to prepare commit history"
        })?;

        let (tx, rx) = channel();

        for i in 0..self.opts.threads {
            let target = self.opts.target.clone();
            let author = self.author.clone();
            let msg = message.clone();
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

        let nonce = Self::mined_commit_nonce(&blob)?;
        match self.write_commit(&hash, &blob) {
            Ok(_) => {
                self.send_nip34_patch_event(&hash, &parent, nonce)?;
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

    fn commit_profile_metadata(&self, commit: &str) -> Option<Metadata> {
        self.signing_keys.is_none().then(|| {
            let name = format!("gnostr@{commit:.7}");
            let mut metadata = Metadata {
                name: Some(name.clone()),
                ..Metadata::default()
            };
            let _ = metadata
                .other
                .insert("display_name".to_string(), name.into());
            metadata
        })
    }

    fn commit_profile_follow_event(
        keys: &Keys,
        parent: &PublicKey,
        nonce_tag: Tag,
    ) -> anyhow::Result<Event> {
        let followed = PublicKey::try_from_bech32_string(
            "npub1gulj0vlre2dcpxltznw790wcds382wpkxg6kt444hvmefpdmm9wqtqv4vx",
            true,
        )?;
        let mut tags = vec![Tag::new_tag("p", &followed.as_hex_string())];
        if parent != &followed {
            tags.push(Tag::new_tag("p", &parent.as_hex_string()));
        }
        tags.extend([Tag::new_tag("t", "gnostr"), nonce_tag]);
        EventBuilder::new(EventKind::ContactList, String::new(), tags).to_event(&keys.secret_key()?)
    }

    fn mined_commit_nonce(blob: &str) -> Result<&str, &'static str> {
        let nonce = blob.rsplit_once("\n\n").map(|(_, nonce)| nonce);
        match nonce {
            Some(nonce) if commit_nonce_tag(nonce).is_ok() => Ok(nonce),
            _ => {
                error!("Mined commit does not contain a valid worker/counter nonce");
                Err("Invalid mined commit nonce")
            }
        }
    }

    fn cache_patch_event(cache_dir: &Path, commit: &str, event_id: Id) -> Result<(), &'static str> {
        std::fs::create_dir_all(cache_dir)
            .and_then(|_| {
                std::fs::write(
                    cache_dir.join(commit),
                    format!("{}\n", event_id.as_hex_string()),
                )
            })
            .map_err(|err| {
                error!(
                    "Failed to cache patch event {} for commit {}: {}",
                    event_id, commit, err
                );
                "Failed to cache patch event"
            })
    }

    fn select_parent_patch(events: Vec<Event>, commit: &str, pubkey: PublicKey) -> Option<Id> {
        events
            .into_iter()
            .filter(|event| {
                event.pubkey == pubkey
                    && event.kind == EventKind::Patches
                    && event
                        .tags
                        .iter()
                        .any(|tag| tag.tagname() == "commit" && tag.value() == commit)
            })
            .filter(|event| match event.verify(None) {
                Ok(()) => true,
                Err(err) => {
                    warn!("Ignoring invalid parent patch event {}: {}", event.id, err);
                    false
                }
            })
            .min_by_key(|event| (event.created_at, event.id))
            .map(|event| event.id)
    }

    async fn resolve_parent(
        client: &Client,
        cache_dir: &Path,
        commit: &str,
    ) -> Result<ParentCommitLink, &'static str> {
        let keys = Keys::parse(format!("{:0>64}", commit)).ok_or_else(|| {
            error!("Failed to derive public key for parent commit {}", commit);
            "Failed to derive parent commit identity"
        })?;
        let pubkey = keys.public_key();
        info!(
            "Parent commit identity: commit={}, pubkey={}",
            commit, pubkey
        );
        let event_id = match std::fs::read_to_string(cache_dir.join(commit)) {
            Ok(value) => Some(Id::try_from_hex_string(value.trim()).map_err(|err| {
                error!(
                    "Invalid cached patch event ID for parent commit {}: {}",
                    commit, err
                );
                "Invalid cached parent patch event ID"
            })?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                let mut filter = Filter::new();
                filter
                    .add_author(&pubkey.into())
                    .add_event_kind(EventKind::Patches);
                filter.limit = Some(32);
                let events = tokio::time::timeout(
                    Duration::from_secs(8),
                    client.get_events_of(vec![filter], Some(Duration::from_secs(2))),
                )
                .await;
                let event_id = match events {
                    Ok(Ok(events)) => Self::select_parent_patch(events, commit, pubkey),
                    Ok(Err(err)) => {
                        warn!("Parent patch lookup failed for commit {}: {}", commit, err);
                        None
                    }
                    Err(err) => {
                        warn!(
                            "Parent patch lookup timed out for commit {}: {}",
                            commit, err
                        );
                        None
                    }
                };
                if let Some(event_id) = event_id {
                    Self::cache_patch_event(cache_dir, commit, event_id)?;
                }
                event_id
            }
            Err(err) => {
                error!(
                    "Failed to read cached patch event for parent commit {}: {}",
                    commit, err
                );
                return Err("Failed to read cached parent patch event");
            }
        };
        if let Some(event_id) = event_id {
            info!(
                "Linked parent patch: commit={}, pubkey={}, event_id={}",
                commit, pubkey, event_id
            );
        } else {
            warn!(
                "No parent patch found for commit {}; publishing as a chain root with parent pubkey {} but no parent e tag",
                commit, pubkey
            );
        }
        Ok(ParentCommitLink { pubkey, event_id })
    }

    fn send_nip34_patch_event(
        &self,
        head: &str,
        parent: &str,
        nonce: &str,
    ) -> Result<(), &'static str> {
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
        let metadata = self.commit_profile_metadata(head);
        let parent_hash = parent.to_string();
        let cache_dir = self.repo.path().join("gnostr/patch-events");
        let nonce_tag = commit_nonce_tag(nonce).map_err(|err| {
            error!("Invalid mining nonce for commit {}: {}", head, err);
            "Invalid mined commit nonce"
        })?;
        info!("Mined commit proof: commit={}, nonce={}", head, nonce);

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
                install_rustls_crypto_provider();
                let mut client = Client::new(
                    &keys,
                    ClientOptions::new().send_timeout(Some(Duration::from_secs(1))),
                );
                client.add_relays(relay_urls.clone()).await.map_err(|err| {
                    error!("Failed to configure commit event relays: {}", err);
                    "Failed to configure commit event relays"
                })?;
                client.connect().await;
                let parent = Self::resolve_parent(&client, &cache_dir, &parent_hash).await?;
                let mut commit_tags = parent.tags();
                commit_tags.push(nonce_tag.clone());
                if let Some(metadata) = metadata {
                    debug!("Commit profile metadata: {:?}", metadata);
                    info!(
                        "Publishing kind-0 metadata for commit {} with public key {}",
                        head_hash,
                        keys.public_key()
                    );
                    let metadata_id = client
                        .set_metadata_with_tags(
                            &metadata,
                            vec![Tag::new_tag("t", "gnostr"), nonce_tag.clone()],
                        )
                        .await
                        .map_err(|err| {
                            error!(
                                "Failed to publish metadata for commit {}: {}",
                                head_hash, err
                            );
                            "Failed to publish commit profile metadata"
                        })?;
                    info!(
                        "Published commit profile metadata: id={}, commit={}, pubkey={}",
                        metadata_id,
                        head_hash,
                        keys.public_key()
                    );
                    let follows = Self::commit_profile_follow_event(&keys, &parent.pubkey, nonce_tag).map_err(
                        |err| {
                            error!(
                                "Failed to build follow list for commit {}: {}",
                                head_hash, err
                            );
                            "Failed to build commit profile follow list"
                        },
                    )?;
                    let follows_id = client.send_event(follows).await.map_err(|err| {
                        error!(
                            "Failed to publish follow list for commit {}: {}",
                            head_hash, err
                        );
                        "Failed to publish commit profile follow list"
                    })?;
                    info!(
                        "Published commit profile follow list: id={}, commit={}, pubkey={}, parent_pubkey={}",
                        follows_id,
                        head_hash,
                        keys.public_key(),
                        parent.pubkey
                    );
                }
                info!(
                    "Publishing NIP-34 patch event for {} to {} relay(s)",
                    repo_name,
                    relay_urls.len()
                );
                let event_id = publish_patch_event_with_tags(
                    &keys,
                    &relay_urls,
                    &repo_name,
                    &head_hash,
                    &patch_content,
                    None,
                    &commit_tags,
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
                Self::cache_patch_event(&cache_dir, &head_hash, event_id)?;
                let announcement =
                    build_patch_announcement_with_tags(&keys, event_id, &commit_tags).map_err(
                        |err| {
                            error!("Failed to build announcement for patch {}: {}", event_id, err);
                            "Failed to build patch announcement"
                        },
                    )?;
                debug!(
                    "Publishing kind-1 patch announcement: id={}, pubkey={}, patch_event={}, content={}",
                    announcement.id,
                    announcement.pubkey,
                    event_id,
                    announcement.content
                );
                let announcement_id = client.send_event(announcement).await.map_err(|err| {
                    error!(
                        "Failed to publish announcement for patch {}: {}",
                        event_id, err
                    );
                    "Failed to publish patch announcement"
                })?;
                info!(
                    "Published kind-1 patch announcement: id={}, patch_event={}, commit={}, pubkey={}",
                    announcement_id,
                    event_id,
                    head_hash,
                    keys.public_key()
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

#[cfg(test)]
mod tests {
    use super::*;
    use gnostr_types::nostr::{Unixtime, DEFAULT_AVATAR, DEFAULT_BANNER};

    fn miner() -> (tempfile::TempDir, Gitminer) {
        let directory = tempfile::tempdir().unwrap();
        let repo = gnostr_asyncgit::git2::Repository::init(directory.path()).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        let miner = Gitminer::new(Options {
            threads: 1,
            target: "000".to_string(),
            message: vec!["test".to_string()],
            repo: directory.path().to_str().unwrap().to_string(),
            timestamp: OffsetDateTime::now_utc(),
            kind: None,
        })
        .unwrap();
        (directory, miner)
    }

    #[test]
    fn commit_derived_identity_publishes_gnostr_profile_name_and_images() {
        let (_directory, miner) = miner();
        let client = Client::new(&Keys::generate(), ClientOptions::new());
        for (commit, name) in [
            ("0000ed4f76930afd9e13bb471a7734d29ece6e4c", "gnostr@0000ed4"),
            ("000dba5dac8fb736d0a7e811b289cca088064ea4", "gnostr@000dba5"),
        ] {
            let metadata = miner.commit_profile_metadata(commit).unwrap();
            let event = client
                .build_metadata_event(&metadata, vec![Tag::new_tag("t", "gnostr")])
                .unwrap();
            event.verify(None).unwrap();
            let content: serde_json::Value = serde_json::from_str(&event.content).unwrap();
            assert_eq!(content["name"], name);
            assert_eq!(content["display_name"], name);
            assert_eq!(content["picture"], DEFAULT_AVATAR);
            assert_eq!(content["banner"], DEFAULT_BANNER);
        }
    }

    #[test]
    fn explicit_signer_and_its_clone_do_not_get_profile_metadata() {
        let (_directory, miner) = miner();
        let miner = miner.with_signing_keys(Keys::generate());
        let commit = "000dba5dac8fb736d0a7e811b289cca088064ea4";
        assert!(miner.commit_profile_metadata(commit).is_none());
        assert!(miner.clone().commit_profile_metadata(commit).is_none());
    }

    #[test]
    fn commit_profile_follows_parent_and_empty_sha256_identity_with_signed_kind_3() {
        let keys = Keys::generate();
        let parent = Keys::parse(format!(
            "{:0>64}",
            "000b3840ed5d6b9a6795d687fcb4cd948969b7a8"
        ))
        .unwrap()
        .public_key();
        let nonce = commit_nonce_tag("01-00000122").unwrap();
        let event = Gitminer::commit_profile_follow_event(&keys, &parent, nonce.clone()).unwrap();
        event.verify(None).unwrap();
        assert_eq!(event.kind, EventKind::ContactList);
        assert_eq!(event.pubkey, keys.public_key());
        assert!(event.content.is_empty());
        let followed = PublicKey::try_from_bech32_string(
            "npub1gulj0vlre2dcpxltznw790wcds382wpkxg6kt444hvmefpdmm9wqtqv4vx",
            true,
        )
        .unwrap();
        assert_eq!(
            event.tags,
            vec![
                Tag::new_tag("p", &followed.as_hex_string()),
                Tag::new_tag("p", &parent.as_hex_string()),
                Tag::new_tag("t", "gnostr"),
                nonce.clone(),
            ]
        );
        let event = Gitminer::commit_profile_follow_event(&keys, &followed, nonce).unwrap();
        event.verify(None).unwrap();
        assert_eq!(
            event.tags.iter().filter(|tag| tag.tagname() == "p").count(),
            1
        );
    }

    #[test]
    fn depth_appends_first_parent_hashes_before_nonce_and_stops_at_root() {
        use gnostr_asyncgit::git2::{Oid, Signature};

        let (_directory, mut miner) = miner();
        miner.opts.message = vec![
            "subject".to_string(),
            "body one".to_string(),
            "body two".to_string(),
        ];
        let (root, main, side, tip, tree_id) = {
            let repo = &miner.repo;
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            let signature = Signature::now("Test", "test@example.com").unwrap();
            let commit = |message: &str, ids: &[Oid]| {
                let parents = ids
                    .iter()
                    .map(|id| repo.find_commit(*id).unwrap())
                    .collect::<Vec<_>>();
                let parents = parents.iter().collect::<Vec<_>>();
                repo.commit(None, &signature, &signature, message, &tree, &parents)
                    .unwrap()
            };
            let root = commit("root", &[]);
            let main = commit("main", &[root]);
            let side = commit("side branch", &[root]);
            let tip = commit("merge", &[main, side]);
            (root, main, side, tip, tree_id)
        };
        miner
            .repo
            .reference("refs/heads/main", tip, true, "test history")
            .unwrap();
        miner.repo.set_head("refs/heads/main").unwrap();
        let parent = miner.repo.head().unwrap().target().unwrap().to_string();
        let original = "subject\n\nbody one\nbody two";
        assert_eq!(miner.mining_message(&parent).unwrap(), original);

        let merge_entries = format!("{} {}\n{} {}", main, tip, side, tip);
        assert_eq!(
            miner.clone().with_depth(1).mining_message(&parent).unwrap(),
            format!("{}\n\n{}", original, merge_entries)
        );
        let miner = miner.with_depth(2);
        let message = format!("{}\n\n{}\n{}", original, merge_entries, main);
        assert_eq!(miner.mining_message(&parent).unwrap(), message);
        assert_eq!(miner.clone().mining_message(&parent).unwrap(), message);

        let (tx, rx) = channel();
        Worker::new(
            1,
            String::new(),
            tree_id.to_string(),
            parent.clone(),
            miner.author.clone(),
            message.clone(),
            miner.opts.timestamp,
            tx,
        )
        .work();
        let (_, blob, _) = rx.recv().unwrap();
        let body = blob.split_once("\n\n").unwrap().1;
        assert_eq!(body, format!("{}\n\n01-00000000", message));
        assert_eq!(Gitminer::mined_commit_nonce(&blob).unwrap(), "01-00000000");

        let miner = miner.with_depth(usize::MAX);
        assert_eq!(
            miner.mining_message(&parent).unwrap(),
            format!("{}\n\n{}\n{}\n{}", original, merge_entries, main, root)
        );
        assert_eq!(
            miner.with_depth(0).mining_message(&parent).unwrap(),
            original
        );
    }

    #[test]
    fn mined_nonce_is_the_exact_final_worker_counter_not_an_earlier_body_line() {
        let blob = "tree example\n\nsubject\n\nEarlier proof 99-00000001\n\n01-00000122";
        assert_eq!(Gitminer::mined_commit_nonce(blob).unwrap(), "01-00000122");
        assert!(Gitminer::mined_commit_nonce("tree example\n\nsubject").is_err());
        assert!(Gitminer::mined_commit_nonce("tree example\n\n01-invalid").is_err());
    }

    #[test]
    fn parent_cache_preserves_actual_event_id_and_missing_parent_becomes_root() {
        let directory = tempfile::tempdir().unwrap();
        let commit = "000d93aaec00000000000000000000000000000000";
        let keys = Keys::parse(format!("{:0>64}", commit)).unwrap();
        let client = Client::new(&keys, ClientOptions::new());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let root = runtime
            .block_on(Gitminer::resolve_parent(&client, directory.path(), commit))
            .unwrap();
        assert_eq!(root.pubkey, keys.public_key());
        assert_eq!(root.event_id, None);

        let event_id = Id::mock();
        Gitminer::cache_patch_event(directory.path(), commit, event_id).unwrap();
        let linked = runtime
            .block_on(Gitminer::resolve_parent(&client, directory.path(), commit))
            .unwrap();
        assert_eq!(linked.pubkey, keys.public_key());
        assert_eq!(linked.event_id, Some(event_id));

        std::fs::write(directory.path().join(commit), "corrupted event ID").unwrap();
        assert!(runtime
            .block_on(Gitminer::resolve_parent(&client, directory.path(), commit))
            .is_err());
    }

    #[test]
    fn parent_discovery_checks_signature_author_kind_and_commit_before_selecting_oldest() {
        let commit = "000d93aaec00000000000000000000000000000000";
        let keys = Keys::parse(format!("{:0>64}", commit)).unwrap();
        let event = |keys: &Keys, kind, commit: &str, timestamp| {
            EventBuilder::new(
                kind,
                "patch".to_string(),
                vec![Tag::new_tag("commit", commit)],
            )
            .created_at(Unixtime(timestamp))
            .to_event(&keys.secret_key().unwrap())
            .unwrap()
        };
        let oldest = event(&keys, EventKind::Patches, commit, 2);
        let another = EventBuilder::new(
            EventKind::Patches,
            "another signed patch".to_string(),
            vec![Tag::new_tag("commit", commit)],
        )
        .created_at(Unixtime(2))
        .to_event(&keys.secret_key().unwrap())
        .unwrap();
        let expected = oldest.id.min(another.id);
        let mut invalid = event(&keys, EventKind::Patches, commit, 1);
        invalid.content.push_str("tampered");
        let events = vec![
            event(&Keys::generate(), EventKind::Patches, commit, 1),
            event(&keys, EventKind::TextNote, commit, 1),
            event(&keys, EventKind::Patches, "other commit", 1),
            invalid,
            event(&keys, EventKind::Patches, commit, 3),
            oldest,
            another,
        ];
        assert_eq!(
            Gitminer::select_parent_patch(events.clone(), commit, keys.public_key()),
            Some(expected)
        );
        assert_eq!(
            Gitminer::select_parent_patch(events[..4].to_vec(), commit, keys.public_key()),
            None
        );
    }
}
