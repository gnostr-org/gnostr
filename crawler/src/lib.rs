pub mod processor;
pub mod pubkeys;
pub mod relay_manager;
pub mod relays;
pub mod stats;

use clap::Parser;
use git2::{Commit, DiffOptions, ObjectType, Repository, Signature, Time};
use git2::{DiffFormat, Error, Pathspec};
use std::str;

use ::time::at;
use ::time::Timespec;
use nostr_sdk::prelude::*;

use crate::processor::Processor;
use crate::processor::APP_SECRET_KEY;
use crate::relay_manager::RelayManager;

use crate::processor::BOOTSTRAP_RELAY1;
use crate::processor::BOOTSTRAP_RELAY2;
use crate::processor::BOOTSTRAP_RELAY3;

#[allow(clippy::manual_strip)]
#[derive(Parser)]
pub struct CliArgs {
    #[structopt(name = "topo-order", long)]
    /// sort commits in topological order
    flag_topo_order: bool,
    #[structopt(name = "date-order", long)]
    /// sort commits in date order
    flag_date_order: bool,
    #[structopt(name = "reverse", long)]
    /// sort commits in reverse
    flag_reverse: bool,
    #[structopt(name = "author", long)]
    /// author to sort by
    flag_author: Option<String>,
    #[structopt(name = "committer", long)]
    /// committer to sort by
    flag_committer: Option<String>,
    #[structopt(name = "pat", long = "grep")]
    /// pattern to filter commit messages by
    flag_grep: Option<String>,
    #[structopt(name = "dir", long = "git-dir")]
    /// alternative git directory to use
    flag_git_dir: Option<String>,
    #[structopt(name = "skip", long)]
    /// number of commits to skip
    flag_skip: Option<usize>,
    #[structopt(name = "max-count", short = 'n', long)]
    /// maximum number of commits to show
    flag_max_count: Option<usize>,
    #[structopt(name = "merges", long)]
    /// only show merge commits
    flag_merges: bool,
    #[structopt(name = "no-merges", long)]
    /// don't show merge commits
    flag_no_merges: bool,
    #[structopt(name = "no-min-parents", long)]
    /// don't require a minimum number of parents
    flag_no_min_parents: bool,
    #[structopt(name = "no-max-parents", long)]
    /// don't require a maximum number of parents
    flag_no_max_parents: bool,
    #[structopt(name = "max-parents")]
    /// specify a maximum number of parents for a commit
    flag_max_parents: Option<usize>,
    #[structopt(name = "min-parents")]
    /// specify a minimum number of parents for a commit
    flag_min_parents: Option<usize>,
    #[structopt(name = "patch", long, short)]
    /// show commit diff
    flag_patch: bool,
    #[structopt(name = "commit")]
    arg_commit: Vec<String>,
    #[structopt(name = "spec", last = true)]
    arg_spec: Vec<String>,
}

pub fn run(args: &CliArgs) -> Result<()> {
    let path = args.flag_git_dir.as_ref().map(|s| &s[..]).unwrap_or(".");
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;

    let _run_async = async {
        let opts = Options::new(); //.wait_for_send(true);
        let app_keys = Keys::from_sk_str(APP_SECRET_KEY).unwrap();
        let relay_client = Client::new_with_opts(&app_keys, opts);
        let _ = relay_client
            .publish_text_note("run:async:11<--------------------------<<<<<", &[])
            .await;
        let _ = relay_client
            .publish_text_note("run:async:22<--------------------------<<<<<", &[])
            .await;
        let _ = relay_client
            .publish_text_note("run:async:33<--------------------------<<<<<", &[])
            .await;
        let _ = relay_client
            .publish_text_note("run:async:44<--------------------------<<<<<", &[])
            .await;
        let _ = relay_client.publish_text_note("#gnostr", &[]).await;
    };

    // Prepare the revwalk based on CLI parameters
    let base = if args.flag_reverse {
        git2::Sort::REVERSE
    } else {
        git2::Sort::NONE
    };
    revwalk.set_sorting(
        base | if args.flag_topo_order {
            git2::Sort::TOPOLOGICAL
        } else if args.flag_date_order {
            git2::Sort::TIME
        } else {
            git2::Sort::NONE
        },
    )?;
    for commit in &args.arg_commit {
        #[allow(clippy::manual_strip)]
        if commit.starts_with('^') {
            let obj = repo.revparse_single(&commit[1..])?;
            revwalk.hide(obj.id())?;
            continue;
        }
        let revspec = repo.revparse(commit)?;
        if revspec.mode().contains(git2::RevparseMode::SINGLE) {
            revwalk.push(revspec.from().unwrap().id())?;
        } else {
            let from = revspec.from().unwrap().id();
            let to = revspec.to().unwrap().id();
            revwalk.push(to)?;
            if revspec.mode().contains(git2::RevparseMode::MERGE_BASE) {
                let base = repo.merge_base(from, to)?;
                let o = repo.find_object(base, Some(ObjectType::Commit))?;
                revwalk.push(o.id())?;
            }
            revwalk.hide(from)?;
        }
    }
    if args.arg_commit.is_empty() {
        revwalk.push_head()?;
    }

    // Prepare our diff options and pathspec matcher
    let (mut diffopts, mut diffopts2) = (DiffOptions::new(), DiffOptions::new());
    for spec in &args.arg_spec {
        diffopts.pathspec(spec);
        diffopts2.pathspec(spec);
    }
    let ps = Pathspec::new(args.arg_spec.iter())?;

    // Filter our revwalk based on the CLI parameters
    macro_rules! filter_try {
        ($e:expr) => {
            match $e {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            }
        };
    }
    let revwalk = revwalk
        .filter_map(|id| {
            let id = filter_try!(id);
            let commit = filter_try!(repo.find_commit(id));
            let parents = commit.parents().len();
            if parents < args.min_parents() {
                return None;
            }
            if let Some(n) = args.max_parents() {
                if parents >= n {
                    return None;
                }
            }
            if !args.arg_spec.is_empty() {
                match commit.parents().len() {
                    0 => {
                        let tree = filter_try!(commit.tree());
                        let flags = git2::PathspecFlags::NO_MATCH_ERROR;
                        if ps.match_tree(&tree, flags).is_err() {
                            return None;
                        }
                    }
                    _ => {
                        let m = commit.parents().all(|parent| {
                            match_with_parent(&repo, &commit, &parent, &mut diffopts)
                                .unwrap_or(false)
                        });
                        if !m {
                            return None;
                        }
                    }
                }
            }
            if !sig_matches(&commit.author(), &args.flag_author) {
                return None;
            }
            if !sig_matches(&commit.committer(), &args.flag_committer) {
                return None;
            }
            if !log_message_matches(commit.message(), &args.flag_grep) {
                return None;
            }
            Some(Ok(commit))
        })
        .skip(args.flag_skip.unwrap_or(0))
        .take(args.flag_max_count.unwrap_or(!0));

    // print!
    for commit in revwalk {
        let commit = commit?;
        //print_commit(&commit);
        if !args.flag_patch || commit.parents().len() > 1 {
            continue;
        }
        let a = if commit.parents().len() == 1 {
            let parent = commit.parent(0)?;
            Some(parent.tree()?)
        } else {
            None
        };
        let b = commit.tree()?;
        let diff = repo.diff_tree_to_tree(a.as_ref(), Some(&b), Some(&mut diffopts2))?;
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                ' ' | '+' | '-' => print!("{}", line.origin()),
                _ => {}
            }
            print!("{}", str::from_utf8(line.content()).unwrap());
            true
        })?;
    }

    let app_secret_key = SecretKey::from_bech32(APP_SECRET_KEY);
    let app_keys = Keys::new(app_secret_key.expect("REASON"));
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor);
    let _run_async = relay_manager.run(vec![BOOTSTRAP_RELAY1, BOOTSTRAP_RELAY2, BOOTSTRAP_RELAY3]);
    //.await;
    //relay_manager.processor.dump();

    Ok(())
}

pub fn sig_matches(sig: &Signature, arg: &Option<String>) -> bool {
    match *arg {
        Some(ref s) => {
            sig.name().map(|n| n.contains(s)).unwrap_or(false)
                || sig.email().map(|n| n.contains(s)).unwrap_or(false)
        }
        None => true,
    }
}

pub fn log_message_matches(msg: Option<&str>, grep: &Option<String>) -> bool {
    match (grep, msg) {
        (&None, _) => true,
        (&Some(_), None) => false,
        (Some(s), Some(msg)) => msg.contains(s),
    }
}

pub fn print_commit(commit: &Commit) {
    //println!("commit {}", commit.id());

    if commit.parents().len() > 1 {
        print!("Merge:");
        for id in commit.parent_ids() {
            print!(" {:.8}", id);
        }
        println!();
    }

    let author = commit.author();
    println!("Author: {}", author);
    print_time(&author.when(), "Date:   ");
    println!();

    for line in String::from_utf8_lossy(commit.message_bytes()).lines() {
        println!("    {}", line);
    }
    println!();
}

pub fn print_time(time: &Time, prefix: &str) {
    let (offset, sign) = match time.offset_minutes() {
        n if n < 0 => (-n, '-'),
        n => (n, '+'),
    };
    let (hours, minutes) = (offset / 60, offset % 60);
    let ts = Timespec::new(time.seconds() + (time.offset_minutes() as i64) * 60, 0);
    let time = at(ts);

    println!(
        "{}{} {}{:02}{:02}",
        prefix,
        time.strftime("%a %b %e %T %Y").unwrap(),
        sign,
        hours,
        minutes
    );
}

pub fn match_with_parent(
    repo: &Repository,
    commit: &Commit,
    parent: &Commit,
    opts: &mut DiffOptions,
) -> Result<bool, Error> {
    let a = parent.tree()?;
    let b = commit.tree()?;
    let diff = repo.diff_tree_to_tree(Some(&a), Some(&b), Some(opts))?;
    Ok(diff.deltas().len() > 0)
}

impl CliArgs {
    fn min_parents(&self) -> usize {
        if self.flag_no_min_parents {
            return 0;
        }
        self.flag_min_parents
            .unwrap_or(if self.flag_merges { 2 } else { 0 })
    }

    fn max_parents(&self) -> Option<usize> {
        if self.flag_no_max_parents {
            return None;
        }
        self.flag_max_parents
            .or(if self.flag_no_merges { Some(1) } else { None })
    }
}
