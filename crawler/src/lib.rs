pub mod processor;
pub mod pubkeys;
pub mod relay_manager;
pub mod relays;
pub mod stats;

use clap::{Parser, Subcommand};
use futures::{stream, StreamExt};
use git2::Error;
use git2::{Commit, DiffOptions, Repository, Signature, Time};
use reqwest::header::ACCEPT;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::str;
use tracing::{debug, error};

use serde::{Deserialize, Serialize};

use ::time::at;
use ::time::Timespec;
use nostr_sdk::prelude::*;

use crate::processor::Processor;
use crate::processor::APP_SECRET_KEY;
use crate::relay_manager::RelayManager;

use crate::processor::LOCALHOST_8080;
use crate::processor::BOOTSTRAP_RELAYS;
use crate::processor::BOOTSTRAP_RELAY0;
use crate::processor::BOOTSTRAP_RELAY1;
use crate::processor::BOOTSTRAP_RELAY2;

const CONCURRENT_REQUESTS: usize = 16;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Runs the sniper mode to find relays supporting a specific NIP
    Sniper {
        /// The NIP number to search for (e.g., 1)
        nip: i32,
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
    /// Runs the watch mode to monitor relays and print their metadata
    Watch {
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
    /// Lists relays that are likely to support NIP-34 (Git collaboration)
    Nip34 {
        /// Optional: Path to a shitlist file to exclude relays
        #[clap(long, short)]
        shitlist: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Relay {
    pub contact: String,
    pub description: String,
    pub name: String,
    pub software: String,
    pub supported_nips: Vec<i32>,
    pub version: String,
}

pub fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(fs::File::open(filename)?).lines().collect()
}

pub fn load_shitlist(filename: impl AsRef<Path>) -> io::Result<HashSet<String>> {
    BufReader::new(fs::File::open(filename)?).lines().collect()
}

#[allow(clippy::manual_strip)]
#[derive(Parser)]
pub struct CliArgs {
    //#[clap(name = "topo-order", long)]
    ///// sort commits in topological order
    //flag_topo_order: bool,
    //#[clap(name = "date-order", long)]
    ///// sort commits in date order
    //flag_date_order: bool,
    //#[clap(name = "reverse", long)]
    ///// sort commits in reverse
    //flag_reverse: bool,
    //#[clap(name = "author", long)]
    ///// author to sort by
    //flag_author: Option<String>,
    //#[clap(name = "committer", long)]
    ///// committer to sort by
    //flag_committer: Option<String>,
    //#[clap(name = "pat", long = "grep")]
    ///// pattern to filter commit messages by
    //flag_grep: Option<String>,
    #[clap(name = "dir", long = "git-dir")]
    /// alternative git directory to use
    flag_git_dir: Option<String>,
    //#[clap(name = "skip", long)]
    ///// number of commits to skip
    //flag_skip: Option<usize>,
    //#[clap(name = "max-count", short = 'n', long)]
    ///// maximum number of commits to show
    //flag_max_count: Option<usize>,
    //#[clap(name = "merges", long)]
    ///// only show merge commits
    //flag_merges: bool,
    //#[clap(name = "no-merges", long)]
    ///// don't show merge commits
    //flag_no_merges: bool,
    //#[clap(name = "no-min-parents", long)]
    ///// don't require a minimum number of parents
    //flag_no_min_parents: bool,
    //#[clap(name = "no-max-parents", long)]
    ///// don't require a maximum number of parents
    //flag_no_max_parents: bool,
    //#[clap(name = "max-parents")]
    ///// specify a maximum number of parents for a commit
    //flag_max_parents: Option<usize>,
    //#[clap(name = "min-parents")]
    ///// specify a minimum number of parents for a commit
    //flag_min_parents: Option<usize>,
    #[clap(name = "patch", long, short)]
    /// show commit diff
    flag_patch: bool,
    #[clap(
        name = "nsec",
        default_value = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    )]
    arg_nsec: Option<String>,
    #[clap(name = "commit")]
    arg_commit: Vec<String>,
    #[clap(name = "spec", last = true)]
    arg_spec: Vec<String>,
}

pub fn run(args: &CliArgs) -> Result<()> {
    let path = args.flag_git_dir.as_ref().map(|s| &s[..]).unwrap_or(".");
    let repo = Repository::discover(path)?;
    let _revwalk = repo.revwalk()?;

    //println!("{:?}", args.arg_nsec.clone());
    let _run_async = async {
        let opts = Options::new(); //.wait_for_send(true);
        let app_keys = Keys::from_sk_str(args.arg_nsec.clone().as_ref().expect("REASON")).unwrap();
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
    //let base = if args.flag_reverse {
    //    git2::Sort::REVERSE
    //} else {
    //    git2::Sort::NONE
    //};
    //revwalk.set_sorting(
    //    base | if args.flag_topo_order {
    //        git2::Sort::TOPOLOGICAL
    //    } else if args.flag_date_order {
    //        git2::Sort::TIME
    //    } else {
    //        git2::Sort::NONE
    //    },
    //)?;
    //for commit in &args.arg_commit {
    //    #[allow(clippy::manual_strip)]
    //    if commit.starts_with('^') {
    //        let obj = repo.revparse_single(&commit[1..])?;
    //        revwalk.hide(obj.id())?;
    //        continue;
    //    }
    //    let revspec = repo.revparse(commit)?;
    //    if revspec.mode().contains(git2::RevparseMode::SINGLE) {
    //        revwalk.push(revspec.from().unwrap().id())?;
    //    } else {
    //        let from = revspec.from().unwrap().id();
    //        let to = revspec.to().unwrap().id();
    //        revwalk.push(to)?;
    //        if revspec.mode().contains(git2::RevparseMode::MERGE_BASE) {
    //            let base = repo.merge_base(from, to)?;
    //            let o = repo.find_object(base, Some(ObjectType::Commit))?;
    //            revwalk.push(o.id())?;
    //        }
    //        revwalk.hide(from)?;
    //    }
    //}
    //if args.arg_commit.is_empty() {
    //    revwalk.push_head()?;
    //}

    //// Prepare our diff options and pathspec matcher
    //let (mut diffopts, mut diffopts2) = (DiffOptions::new(), DiffOptions::new());
    //for spec in &args.arg_spec {
    //    diffopts.pathspec(spec);
    //    diffopts2.pathspec(spec);
    //}
    //let ps = Pathspec::new(args.arg_spec.iter())?;

    //// Filter our revwalk based on the CLI parameters
    //macro_rules! filter_try {
    //    ($e:expr) => {
    //        match $e {
    //            Ok(t) => t,
    //            Err(e) => return Some(Err(e)),
    //        }
    //    };
    //}
    //let revwalk = revwalk
    //    .filter_map(|id| {
    //        let id = filter_try!(id);
    //        let commit = filter_try!(repo.find_commit(id));
    //        let parents = commit.parents().len();
    //        if parents < args.min_parents() {
    //            return None;
    //        }
    //        if let Some(n) = args.max_parents() {
    //            if parents >= n {
    //                return None;
    //            }
    //        }
    //        if !args.arg_spec.is_empty() {
    //            match commit.parents().len() {
    //                0 => {
    //                    let tree = filter_try!(commit.tree());
    //                    let flags = git2::PathspecFlags::NO_MATCH_ERROR;
    //                    if ps.match_tree(&tree, flags).is_err() {
    //                        return None;
    //                    }
    //                }
    //                _ => {
    //                    let m = commit.parents().all(|parent| {
    //                        match_with_parent(&repo, &commit, &parent, &mut diffopts)
    //                            .unwrap_or(false)
    //                    });
    //                    if !m {
    //                        return None;
    //                    }
    //                }
    //            }
    //        }
    //        if !sig_matches(&commit.author(), &args.flag_author) {
    //            return None;
    //        }
    //        if !sig_matches(&commit.committer(), &args.flag_committer) {
    //            return None;
    //        }
    //        if !log_message_matches(commit.message(), &args.flag_grep) {
    //            return None;
    //        }
    //        Some(Ok(commit))
    //    })
    //    .skip(args.flag_skip.unwrap_or(0))
    //    .take(args.flag_max_count.unwrap_or(!0));

    //// print!
    //for commit in revwalk {
    //    let commit = commit?;
    //    //print_commit(&commit);
    //    if !args.flag_patch || commit.parents().len() > 1 {
    //        continue;
    //    }
    //    let a = if commit.parents().len() == 1 {
    //        let parent = commit.parent(0)?;
    //        Some(parent.tree()?)
    //    } else {
    //        None
    //    };
    //    let b = commit.tree()?;
    //    let diff = repo.diff_tree_to_tree(a.as_ref(), Some(&b), Some(&mut diffopts2))?;
    //    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
    //        match line.origin() {
    //            ' ' | '+' | '-' => print!("{}", line.origin()),
    //            _ => {}
    //        }
    //        print!("230:{}", str::from_utf8(line.content()).unwrap());
    //        true
    //    })?;
    //}

    //println!("{:?}", args.arg_nsec.clone());
    let app_keys = Keys::from_sk_str(args.arg_nsec.clone().as_ref().expect("REASON")).unwrap();
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor);
    let _run_async = relay_manager.run(vec![
        BOOTSTRAP_RELAY0,
        BOOTSTRAP_RELAY1,
        BOOTSTRAP_RELAY2,
    ]);
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

pub async fn run_sniper(
    nip_lower: i32,
    shitlist_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let relays = load_file("relays.yaml").unwrap();
    let client = reqwest::Client::new();

    let shitlist = if let Some(path) = shitlist_path {
        match load_shitlist(&path) {
            Ok(sl) => sl,
            Err(e) => {
                eprintln!("Failed to load shitlist from {}: {}", path, e);
                return Err(e.into());
            }
        }
    } else {
        std::collections::HashSet::new()
    };

    let filtered_relays: Vec<String> = relays
        .into_iter()
        .filter(|url| {
            if shitlist.is_empty() {
                true
            } else {
                !shitlist
                    .iter()
                    .any(|shitlisted_url| url.contains(shitlisted_url))
            }
        })
        .collect();

    let bodies = stream::iter(filtered_relays)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client
                    .get(
                        url.replace("wss://", "https://")
                            .replace("ws://", "http://"),
                    )
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                let r: Result<(String, String), reqwest::Error> = Ok((url.clone(), text.clone()));
                r
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                let data: Result<Relay, _> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    for n in &relay_info.supported_nips {
                        if n == &nip_lower {
                            debug!("contact:{:?}", &relay_info.contact);
                            debug!("description:{:?}", &relay_info.description);
                            debug!("name:{:?}", &relay_info.name);
                            debug!("software:{:?}", &relay_info.software);
                            debug!("version:{:?}", &relay_info.version);

                            let dir_name = format!("{}", nip_lower);
                            let path = Path::new(&dir_name);

                            if !path.exists() {
                                match fs::create_dir(path) {
                                    Ok(_) => debug!("created {}", nip_lower),
                                    Err(e) => eprintln!("Error creating directory: {}", e),
                                }
                            } else {
                                debug!("{} already exists...", dir_name);
                            }

                            let file_name = url
                                .replace("https://", "")
                                .replace("http://", "")
                                .replace("ws://", "")
                                .replace("wss://", "")
                                + ".json";
                            let file_path = path.join(&file_name);
                            let file_path_str = file_path.display().to_string();
                            debug!(
                                "\n\n{}\n\n",
                                file_path_str
                            );

                            match File::create(&file_path) {
                                Ok(mut file) => {
                                    debug!("{}", &file_path_str);
                                    match file.write_all(json_string.as_bytes()) {
                                        Ok(_) => debug!("wrote relay metadata:{}", &file_path_str),
                                        Err(e) => {
                                            error!("Failed to write to {}: {}", &file_path_str, e)
                                        }
                                    }
                                }
                                Err(e) => error!("Failed to create file {}: {}", &file_path_str, e),
                            }

                            println!(
                                "{}/{}",
                                nip_lower,
                                url.replace("https://", "")
                                    .replace("wss://", "")
                                    .replace("ws://", "")
                            );
                        }
                    }
                }
            }
        })
        .await;

    Ok(())
}

pub async fn run_watch(shitlist_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let app_secret_key = SecretKey::from_bech32(APP_SECRET_KEY)?;
    let app_keys = Keys::new(app_secret_key);
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor);

    let bootstrap_relays = vec![
        BOOTSTRAP_RELAY0,
        BOOTSTRAP_RELAY1,
        BOOTSTRAP_RELAY2,
        BOOTSTRAP_RELAYS
            .get(3)
            .expect("BOOTSTRAP_RELAYS should have at least 4 elements")
            .as_str(),
    ];
    relay_manager.run(bootstrap_relays).await?;
    let relays: Vec<String> = relay_manager.relays.get_all();

    let shitlist = if let Some(path) = shitlist_path {
        match load_shitlist(&path) {
            Ok(sl) => sl,
            Err(e) => {
                eprintln!("Failed to load shitlist from {}: {}", path, e);
                return Err(e.into());
            }
        }
    } else {
        std::collections::HashSet::new()
    };

    let relays_iterator = relays.into_iter().filter(|url| {
        if shitlist.is_empty() {
            true
        } else {
            !shitlist
                .iter()
                .any(|shitlisted_url| url.contains(shitlisted_url))
        }
    });

    let client = reqwest::Client::new();
    let bodies = stream::iter(relays_iterator)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client
                    .get(
                        url.replace("wss://", "https://")
                            .replace("ws://", "http://"),
                    )
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;
                Ok((url, text))
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                println!("{{\"relay\":\"{}\", \"data\":{}}}", url, json_string);
                let data: Result<Relay, serde_json::Error> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    print!("{{\"nips\":\"");
                    let mut nip_count = relay_info.supported_nips.len();
                    for n in &relay_info.supported_nips {
                        debug!("nip_count:{}", nip_count);
                        if nip_count > 1 {
                            print!("{:0>2} ", n);
                        } else {
                            print!("{:0>2}", n);
                        }
                        nip_count -= 1;
                    }
                    print!("}}");
                    println!();
                }
            }
        })
        .await;

    // Add the processor.dump() call here
    relay_manager.processor.dump();

    Ok(())
}

pub async fn run_nip34(shitlist_path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let relays = load_file("relays.yaml").unwrap();
    let client = reqwest::Client::new();

    let shitlist = if let Some(path) = shitlist_path {
        match load_shitlist(&path) {
            Ok(sl) => sl,
            Err(e) => {
                eprintln!("Failed to load shitlist from {}: {}", path, e);
                return Err(e.into());
            }
        }
    } else {
        std::collections::HashSet::new()
    };

    let filtered_relays: Vec<String> = relays
        .into_iter()
        .filter(|url| {
            if shitlist.is_empty() {
                true
            } else {
                !shitlist
                    .iter()
                    .any(|shitlisted_url| url.contains(shitlisted_url))
            }
        })
        .collect();

    let bodies = stream::iter(filtered_relays)
        .map(|url| {
            let client = &client;
            async move {
                let resp = client
                    .get(
                        url.replace("wss://", "https://")
                            .replace("ws://", "http://"),
                    )
                    .header(ACCEPT, "application/nostr+json")
                    .send()
                    .await?;
                let text = resp.text().await?;

                let r: Result<(String, String), reqwest::Error> = Ok((url.clone(), text.clone()));
                r
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    bodies
        .for_each(|b: Result<(String, String), reqwest::Error>| async {
            if let Ok((url, json_string)) = b {
                let data: Result<Relay, _> = serde_json::from_str(&json_string);
                if let Ok(relay_info) = data {
                    let supports_nip01 = relay_info.supported_nips.contains(&1);
                    let supports_nip11 = relay_info.supported_nips.contains(&11);

                    if supports_nip01 && supports_nip11 {
                        println!("{}", url);
                    }
                }
            }
        })
        .await;

    Ok(())
}
