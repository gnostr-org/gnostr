use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::{process, thread};

//use git2::*;

use super::worker::Worker;

pub struct Options {
    pub threads: u32,
    pub target: String,
    pub message: String,
    pub pwd_hash: String,
    pub repo: String,
    pub timestamp: time::Tm,
    pub weeble: String,
    pub wobble: String,
    pub blockheight: String,
}

pub struct Gitminer {
    opts: Options,
    repo: git2::Repository,
    author: String,
    pwd_hash: String,
    pub relays: String,
}

impl Gitminer {
    pub fn new(opts: Options) -> Result<Gitminer, &'static str> {
        let repo = match git2::Repository::open(&opts.repo) {
            Ok(r) => r,
            Err(_) => {
                return Err("Failed to open repository");
            }
        };

        let author = Gitminer::load_author(&repo)?;
        let relays = Gitminer::load_gnostr_relays(&repo)?;
        let pwd_hash = Default::default();

        Ok(Gitminer {
            opts,
            repo,
            author,
            pwd_hash,
            relays,
        })
    }

    pub fn mine(&mut self) -> Result<String, &'static str> {
        let (tree, parent) = match Gitminer::prepare_tree(&mut self.repo) {
            Ok((t, p)) => (t, p),
            Err(e) => {
                return Err(e);
            }
        };

        let (tx, rx) = channel();
        for i in 0..self.opts.threads {
            let target = self.opts.target.clone();
            let author = self.author.clone();
            let repo = self.author.clone();
            let pwd_hash = self.pwd_hash.clone();
            let msg = self.opts.message.clone();
            let wtx = tx.clone();
            let ts = self.opts.timestamp;
            let weeble = self.opts.weeble.clone();
            let wobble = self.opts.wobble.clone();
            let bh = self.opts.blockheight.clone();
            let (wtree, wparent) = (tree.clone(), parent.clone());

            thread::spawn(move || {
                Worker::new(
                    i, target, wtree, wparent, author, repo, pwd_hash, msg, ts, weeble, wobble, bh,
                    wtx,
                )
                .work();
            });
        }

        let (_, blob, hash) = rx.recv().unwrap();

        match self.write_commit(&hash, &blob) {
            Ok(_) => Ok(hash),
            Err(e) => Err(e),
        }
    }

    fn write_commit(&self, hash: &String, blob: &String) -> Result<(), &'static str> {
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("mkdir -p {}.gnostr/{} && ", self.opts.repo, hash))
            .output();
        //.ok()
        //.expect("Failed to generate commit");

        /* repo.blob() generates a blob, not a commit.
         * we write the commit, then
         * we use the tmpfile to create .gnostr/blobs/<hash>
         * we 'git show' the mined tmpfile
         * and pipe it into the .gnostr/blobs/<hash>
         */

        let tmpfile = format!("/tmp/{}.tmp", hash);
        let mut file = File::create(Path::new(&tmpfile))
            .ok()
            .unwrap_or_else(|| panic!("Failed to create temporary file {}", &tmpfile));

        file.write_all(blob.as_bytes())
            .ok()
            .unwrap_or_else(|| panic!("Failed to write temporary file {}", &tmpfile));

        //write the commit
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "cd {} && gnostr-git hash-object -t commit -w --stdin < {} && gnostr-git reset \
                 --hard {}",
                self.opts.repo, tmpfile, hash
            ))
            .output();
        //.ok()
        //.expect("Failed to generate commit");

        //write the blob
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "cd {} && mkdir -p .gnostr && touch -f .gnostr/blobs/{} && git show {} > \
                 .gnostr/blobs/{}",
                self.opts.repo, hash, hash, hash
            ))
            .output();
        //.ok()
        //.expect("Failed to write .gnostr/blobs/<hash>");

        //REF:
        //gnostr-git reflog
        // --format='wss://{RELAY}/{REPO}/%C(auto)%H/%<|(17)%gd:commit:%s'
        // gnostr-git-reflog -f
        //write the reflog
        //the new reflog is associated with a commit
        //we will use gnostr-git-reflog -f
        //for an integrity check as well
        //to test the 'gnostr' protocol
        //write the reflog

        //gnostr-git update-index --assume-unchanged .gnostr/reflog
        //--[no-]assume-unchanged
        //When this flag is specified, the object names recorded for the paths are not
        // updated. Instead, this option sets/unsets the "assume unchanged" bit for the
        // paths. When the "assume unchanged" bit is on, the user promises not to change
        // the file and allows Git to assume that the working tree file matches what is
        // recorded in the index. If you want to change the working tree file, you need
        // to unset the bit to tell Git. This is sometimes helpful when working with a
        // big project on a filesystem that has very slow lstat(2) system call (e.g.
        // cifs).
        //
        //Git will fail (gracefully) in case it needs to modify this file in the index
        // e.g. when merging in a commit; thus, in case the assumed-untracked file is
        // changed upstream, you will need to handle the situation manually.

        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "cd {} && mkdir -p .gnostr && touch -f .gnostr/reflog && gnostr-git reflog \
                 --format='wss://{}/{}/%C(auto)%H/%<|(17)%gd:commit:%s' > .gnostr/reflog",
                self.opts.repo, "{RELAY}", "{REPO}"
            ))
            .output();
        //.ok()
        //.expect("Failed to write .gnostr/reflog");
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "cd {} && mkdir -p .gnostr && touch -f .gnostr/reflog && gnostr-git update-index \
                 --assume-unchaged .gnostr/reflog",
                self.opts.repo
            ))
            .output();
        //.ok()
        //.expect("Failed to write .gnostr/reflog");
        Ok(())
    }

    fn load_author(repo: &git2::Repository) -> Result<String, &'static str> {
        let cfg = match repo.config() {
            Ok(c) => c,
            Err(_) => {
                return Err("Failed to load git config user.name");
            }
        };

        let name = match cfg.get_string("user.name") {
            Ok(s) => s,
            Err(_) => {
                return Err("Failed to find git config user.name");
            }
        };

        let email = match cfg.get_string("user.email") {
            Ok(s) => s,
            Err(_) => {
                return Err("Failed to find git config user.email");
            }
        };

        Ok(format!("{} <{}>", name, email))
    }

    fn load_gnostr_relays(repo: &git2::Repository) -> Result<String, &'static str> {
        let cfg = match repo.config() {
            Ok(c) => c,
            Err(_) => {
                return Err("Failed to load git config gnostr.relays");
            }
        };

        let relays = match cfg.get_string("gnostr.relays") {
            Ok(s) => s,
            Err(_) => {
                return Err("Failed to find git config gnostr.relays");
            }
        };

        Ok(relays)
    }

    //fn revparse_0(repo: &mut git2::Repository) -> Result<(String), &'static str> {
    //    Gitminer::ensure_no_unstaged_changes(repo)?;

    //    let head = repo.revparse_single("HEAD").unwrap();
    //    let head_2 = format!("{}", head.id());

    //    Ok(head_2)
    //}
    //fn revparse_1(repo: &mut git2::Repository) -> Result<(String), &'static str> {
    //    Gitminer::ensure_no_unstaged_changes(repo)?;

    //    let head = repo.revparse_single("HEAD~1").unwrap();
    //    let head_1 = format!("{}", head.id());

    //    Ok(head_1)
    //}
    fn prepare_tree(repo: &mut git2::Repository) -> Result<(String, String), &'static str> {
        Gitminer::ensure_no_unstaged_changes(repo)?;

        let head = repo.revparse_single("HEAD").unwrap();
        let mut index = repo.index().unwrap();
        let tree = index.write_tree().unwrap();

        let head_s = format!("{}", head.id());
        let tree_s = format!("{}", tree);

        Ok((tree_s, head_s))
    }

    //repo status CLEAN not enough
    fn ensure_no_unstaged_changes(repo: &mut git2::Repository) -> Result<(), &'static str> {
        let mut opts = git2::StatusOptions::new();
        let mut m = git2::Status::empty();
        let statuses = repo.statuses(Some(&mut opts)).unwrap();

        m.insert(git2::Status::WT_NEW);
        m.insert(git2::Status::WT_MODIFIED);
        m.insert(git2::Status::WT_DELETED);
        m.insert(git2::Status::WT_RENAMED);
        m.insert(git2::Status::WT_TYPECHANGE);

        for i in 0..statuses.len() {
            let status_entry = statuses.get(i).unwrap();
            if status_entry.status().intersects(m) {
                println!("Please stash all unstaged changes before running.");
                //return Err("Please stash all unstaged changes before running.");
                process::exit(1)
            }
        }

        Ok(())
    }
}
