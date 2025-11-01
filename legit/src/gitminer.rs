use std::process;
use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;
// use git2::*;
use super::worker::Worker;
use time_0_3::OffsetDateTime;

#[derive(Clone, Debug)]
pub struct Options {
    pub threads:   u32,
    pub target:    String,
    pub message:   String,
    pub repo:      String,
    pub timestamp: OffsetDateTime,
}

pub struct Gitminer {
    opts:   Options,
    repo:   git2::Repository,
    author: String
}

impl std::fmt::Debug for Gitminer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gitminer")
            .field("opts", &self.opts)
            .field("repo_path", &self.repo.path())
            .field("author", &self.author)
            .finish()
    }
}

impl Clone for Gitminer {
    fn clone(&self) -> Self {
        let repo = git2::Repository::open(&self.opts.repo)
            .expect("Failed to open repository during clone");
        Self {
            opts: self.opts.clone(),
            repo,
            author: self.author.clone(),
        }
    }
}


impl Gitminer {
    pub fn new(opts: Options) -> Result<Gitminer, &'static str> {

        let repo = match git2::Repository::open(&opts.repo) {
            Ok(r)  => r,
            Err(e) => {
                error!("Failed to open repository: {}", e);
                return Err("Failed to open repository");
            }
        };

        let author = Gitminer::load_author(&repo)?;
        debug!("Gitminer initialized with author: {}", author);

        Ok(Gitminer {
            opts:   opts,
            repo:   repo,
            author: author
        })
    }

    pub fn mine(&mut self) -> Result<String, &'static str> {
        debug!("Starting mining process with options: {:?}", self.opts);
        let (tree, parent) = match Gitminer::prepare_tree(&mut self.repo) {
            Ok((t, p)) => (t, p),
            Err(e)   => {
                error!("Failed to prepare tree: {}", e);
                return Err(e);
            }
        };
        debug!("Tree: {}, Parent: {}", tree, parent);

        Gitminer::ensure_gnostr_dirs_exist(Path::new(&self.opts.repo))?;
        debug!(".gnostr directories ensured to exist.");


        let (tx, rx) = channel();

        for i in 0..self.opts.threads {
            let target = self.opts.target.clone();
            let author = self.author.clone();
            let msg    = self.opts.message.clone();
            let wtx    = tx.clone();
            let ts     = self.opts.timestamp.clone();
            let (wtree, wparent) = (tree.clone(), parent.clone());

            debug!("Spawning worker {}", i);
            thread::spawn(move || {
                Worker::new(i, target, wtree, wparent, author, msg, ts, wtx).work();
            });
        }

        let (_, blob, hash) = rx.recv().unwrap();
        info!("Received hash {} from a worker.", hash);

        match self.write_commit(&hash, &blob) {
            Ok(_)  => Ok(hash),
            Err(e) => {
                error!("Failed to write commit: {}", e);
                Err(e)
            }
        }
    }

    fn write_commit(&self, hash: &String, blob: &String) -> Result<(), &'static str> {
        debug!("Writing commit for hash: {}", hash);
        /* repo.blob() generates a blob, not a commit.
         * don't know if there's a way to do this with libgit2. */
        let tmpfile  = format!("/tmp/{}.tmp", hash);
        debug!("Creating temporary file: {}", tmpfile);
        let mut file = File::create(&Path::new(&tmpfile))
            .ok()
            .expect(&format!("Failed to create temporary file {}", &tmpfile));

        file.write_all(blob.as_bytes())
            .ok()
            .expect(&format!("Failed to write temporary file {}", &tmpfile));
        debug!("Temporary file {} written.", tmpfile);

        // Write the blob to .gnostr/blobs/<commit_hash>
        let gnostr_blob_path = Path::new(&self.opts.repo).join(".gnostr/blobs").join(hash);
        debug!("Creating .gnostr blob file: {}", gnostr_blob_path.display());
        let mut gnostr_blob_file = File::create(&gnostr_blob_path)
            .ok()
            .expect(&format!("Failed to create .gnostr blob file {}", &gnostr_blob_path.display()));
        gnostr_blob_file.write_all(blob.as_bytes())
            .ok()
            .expect(&format!("Failed to write .gnostr blob file {}", &gnostr_blob_path.display()));
        debug!(".gnostr blob file {} written.", gnostr_blob_path.display());

        // Write the blob to .gnostr/reflog/<commit_hash>
        let gnostr_reflog_path = Path::new(&self.opts.repo).join(".gnostr/reflog").join(hash);
        debug!("Creating .gnostr reflog file: {}", gnostr_reflog_path.display());
        let mut gnostr_reflog_file = File::create(&gnostr_reflog_path)
            .ok()
            .expect(&format!("Failed to create .gnostr reflog file {}", &gnostr_reflog_path.display()));
        gnostr_reflog_file.write_all(blob.as_bytes())
            .ok()
            .expect(&format!("Failed to write .gnostr reflog file {}", &gnostr_reflog_path.display()));
        debug!(".gnostr reflog file {} written.", gnostr_reflog_path.display());

        let command_str = format!("cd {} && git hash-object -t commit -w --stdin < {} && git reset --hard {}", self.opts.repo, tmpfile, hash);
        debug!("Executing git command: {}", command_str);
        Command::new("sh")
            .arg("-c")
            .arg(command_str)
            .output()
            .ok()
            .expect("Failed to generate commit");
        info!("Commit {} generated and reset.", hash);

        Ok(())
    }


    fn load_author(repo: &git2::Repository) -> Result<String, &'static str> {
        debug!("Loading author from git config.");
        let cfg = match repo.config() {
            Ok(c)  => c,
            Err(e) => {
                error!("Failed to load git config: {}", e);
                return Err("Failed to load git config");
            }
        };

        let name  = match cfg.get_string("user.name") {
            Ok(s)  => s,
            Err(e) => {
                error!("Failed to find git user name: {}", e);
                return Err("Failed to find git user name");
            }
        };
        debug!("Found git user name: {}", name);

        let email = match cfg.get_string("user.email") {
            Ok(s)  => s,
            Err(e) => {
                error!("Failed to find git email address: {}", e);
                return Err("Failed to find git email address");
            }
        };
        debug!("Found git email address: {}", email);

        Ok(format!("{} <{}>", name, email))
    }

    fn ensure_gnostr_dirs_exist(repo_root_path: &Path) -> Result<(), &'static str> {
        debug!("Ensuring .gnostr directories exist in: {}", repo_path.display());
        let gnostr_path = repo_path.join(".gnostr");
        let blobs_path = gnostr_path.join("blobs");
        let reflog_path = gnostr_path.join("reflog");

        if !gnostr_path.exists() {
            debug!("Creating .gnostr directory: {}", gnostr_path.display());
            std::fs::create_dir_all(&gnostr_path)
                .map_err(|e| {
                    error!("Failed to create .gnostr directory {}: {}", gnostr_path.display(), e);
                    "Failed to create .gnostr directory"
                })?;
        }
        if !blobs_path.exists() {
            debug!("Creating .gnostr/blobs directory: {}", blobs_path.display());
            std::fs::create_dir_all(&blobs_path)
                .map_err(|e| {
                    error!("Failed to create .gnostr/blobs directory {}: {}", blobs_path.display(), e);
                    "Failed to create .gnostr/blobs directory"
                })?;
        }
        if !reflog_path.exists() {
            debug!("Creating .gnostr/reflog directory: {}", reflog_path.display());
            std::fs::create_dir_all(&reflog_path)
                .map_err(|e| {
                    error!("Failed to create .gnostr/reflog directory {}: {}", reflog_path.display(), e);
                    "Failed to create .gnostr/reflog directory"
                })?;
        }
        Ok(())
    }

    fn prepare_tree(repo: &mut git2::Repository) -> Result<(String, String), &'static str> {
        debug!("Preparing tree.");
        Gitminer::ensure_no_unstaged_changes(repo)?;

        let head      = repo.revparse_single("HEAD").unwrap();
        let mut index = repo.index().unwrap();
        let tree      = index.write_tree().unwrap();

        let head_s = format!("{}", head.id());
        let tree_s = format!("{}", tree);
        debug!("Head: {}, Tree: {}", head_s, tree_s);

        Ok((tree_s, head_s))
    }

    fn ensure_no_unstaged_changes(repo: &mut git2::Repository) -> Result<(), &'static str> {
        debug!("Ensuring no unstaged changes.");
        let mut opts = git2::StatusOptions::new();
        let mut m    = git2::Status::empty();
        let statuses = repo.statuses(Some(&mut opts)).unwrap();

        m.insert(git2::Status::WT_NEW);
        m.insert(git2::Status::WT_MODIFIED);
        m.insert(git2::Status::WT_DELETED);
        m.insert(git2::Status::WT_RENAMED);
        m.insert(git2::Status::WT_TYPECHANGE);

        for i in 0..statuses.len() {
            let status_entry = statuses.get(i).unwrap();
            if status_entry.status().intersects(m) {
                warn!("Please stash all unstaged changes before running.");
                //return Err("Please stash all unstaged changes before running.");
                process::exit(1)
            }
        }

        debug!("No unstaged changes found.");
        Ok(())
    }

}
