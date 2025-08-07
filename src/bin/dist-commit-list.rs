#![doc = include_str!("../../README.md")]

use clap::Parser;
use git2::{Commit, Diff, DiffOptions, ObjectType, Oid, Repository, Signature, Time};
use git2::{DiffFormat, Error as GitError, Pathspec};
use std::str;
use std::{error::Error, time::Duration};

use futures::stream::StreamExt;
use libp2p::kad::store::Error as KadError;
use libp2p::PeerId;
use libp2p::StreamProtocol;
use libp2p::{
    identify, kad,
    kad::{
        store::MemoryStore, store::MemoryStoreConfig, store::RecordStore, Mode, PutRecordError,
        Quorum, Record, RecordKey,
    },
    mdns, noise, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::{
    io::{self, AsyncBufReadExt},
    select,
};
use tracing::Level;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};
//use tracing_log::LogTracer;
use tracing_log::log;
fn init_subscriber(_level: Level) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info")) //default
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tracing_subscriber::fmt()
        // Setting a filter based on the value of the RUST_LOG environment variable
        // Examples:
        //
        // RUST_LOG="off,libp2p_mdns::behaviour=off"
        // RUST_LOG="warn,libp2p_mdns::behaviour=off"
        // RUST_LOG="debug,libp2p_mdns::behaviour=off"
        //
        .with_env_filter(EnvFilter::from_default_env())
        //.with_max_level(level)
        // Configure the subscriber to emit logs in JSON format.
        .json()
        // Configure the subscriber to flatten event fields in the output JSON objects.
        //.flatten_event(true)
        // Set the subscriber as the default, returning an error if this fails.
        .try_init()?;

    Ok(())
}

async fn get_blockheight() -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::builder()
        .build()
        .expect("should be able to build reqwest client");
    let blockheight = client
        .get("https://mempool.space/api/blocks/tip/height")
        .send()
        .await?;
    log::debug!("mempool.space status: {}", blockheight.status());
    if blockheight.status() != reqwest::StatusCode::OK {
        log::debug!("didn't get OK status: {}", blockheight.status());
        Ok(String::from(">>>>>"))
    } else {
        let blockheight = blockheight.text().await?;
        log::debug!("{}", blockheight);
        Ok(blockheight)
    }
}

const IPFS_BOOTNODES: [&str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];
const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");

fn get_commit_message_bytes(repo: &Repository, commit_id: &str) -> Result<Vec<u8>, git2::Error> {
    let oid = Oid::from_str(commit_id)?;
    let commit = repo.find_commit(oid)?;
    Ok(commit.message_bytes().to_vec())
}

fn get_commit_diff(repo: &Repository, commit_id: Oid) -> Result<Diff, git2::Error> {
    let commit = repo.find_commit(commit_id)?;

    // Get the tree of the current commit
    let tree = commit.tree()?;

    // Get the tree of the first parent commit (if it exists)
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    // Create a diff between the parent tree and the current tree
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    Ok(diff)
}

fn get_commit_diff_as_string(repo: &Repository, commit_id: Oid) -> Result<String, git2::Error> {
    let commit = repo.find_commit(commit_id)?;

    // Get the tree of the current commit
    let tree = commit.tree()?;

    // Get the tree of the first parent commit (if it exists)
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    // Create a diff between the parent tree and the current tree
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    // Create a buffer to write the diff to
    let mut buf = Vec::new();

    // Use Diff::print to write the diff content to the buffer
    diff.print(DiffFormat::Patch, |_, _, line| {
        buf.extend_from_slice(line.content());
        true
    })?;

    let diff_string = String::from_utf8(buf);

    // Return the successful result.
    Ok(diff_string.expect(""))
}

fn get_commit_diff_as_bytes(repo: &Repository, commit_id: Oid) -> Result<Vec<u8>, git2::Error> {
    let commit = repo.find_commit(commit_id)?;

    // Get the tree of the current commit
    let tree = commit.tree()?;

    // Get the tree of the first parent commit (if it exists)
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    // Create a diff between the parent tree and the current tree
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    // Create a buffer to write the diff to
    let mut buf = Vec::new();

    // Use Diff::print to write the diff content to the buffer
    diff.print(DiffFormat::Patch, |_, _, line| {
        buf.extend_from_slice(line.content());
        true
    })?;

    // Return the buffer containing the diff bytes
    Ok(buf)
}

// A simple utility function to print a Kademlia record.
fn print_record(record: Record) {
    println!("--- Kademlia Record ---");
    println!("Key: {:?}", record.key);
    if let Ok(value) = String::from_utf8(record.value.clone()) {
        println!("Value (as string): {}", value);
    } else {
        println!("Value (as bytes): {:?}", record.value);
    }
    println!("Publisher: {:?}", record.publisher);
    println!("--- End Record ---");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = init_subscriber(Level::INFO);

    //let blockheight = get_blockheight().await.unwrap();
    //log::info!("blockheight = {blockheight:?}");

    //TODO create key from arg
    let args = Args::parse();

    //for arg in args.into() {
    tracing::debug!("args={:?}", args);
    //}
    // Results in PeerID 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN which is
    // used as the rendezvous point by the other peer examples.
    // TODO --key arg
    let keypair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32]).unwrap();
    let local_peer_id = PeerId::from(keypair.public());

    // We create a custom network behaviour that combines
    // Kademlia and mDNS identify rendezvous ping
    #[derive(NetworkBehaviour)]
    struct Behaviour {
        ipfs: kad::Behaviour<MemoryStore>,
        commit_message: kad::Behaviour<MemoryStore>,
        commit_diff: kad::Behaviour<MemoryStore>,
        kademlia: kad::Behaviour<MemoryStore>,
        mdns: mdns::tokio::Behaviour,
        identify: identify::Behaviour,
        rendezvous: rendezvous::server::Behaviour,
        ping: ping::Behaviour,
    }

    // let mut swarm = libp2p::SwarmBuilder::with_new_identity()
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|key| {
            let mut ipfs_cfg = kad::Config::new(IPFS_PROTO_NAME);
            ipfs_cfg.set_query_timeout(Duration::from_secs(5 * 60));

            tracing::debug!("The maximum value for usize is: {}", std::usize::MAX);
            //handle large git commit diffs
            let store_config = MemoryStoreConfig {
                max_records: usize::MAX,
                max_value_bytes: usize::MAX,
                max_providers_per_key: usize::MAX,
                max_provided_keys: usize::MAX,
            };

            let ipfs_store = kad::store::MemoryStore::with_config(
                key.public().to_peer_id(),
                store_config.clone(),
            );
            let kademlia_store = kad::store::MemoryStore::with_config(
                key.public().to_peer_id(),
                store_config.clone(),
            );
            let commit_message = kad::store::MemoryStore::with_config(
                key.public().to_peer_id(),
                store_config.clone(),
            );
            let commit_diff =
                kad::store::MemoryStore::with_config(key.public().to_peer_id(), store_config);

            Ok(Behaviour {
                ipfs: kad::Behaviour::with_config(
                    key.public().to_peer_id(),
                    ipfs_store,
                    ipfs_cfg.clone(),
                ),
                commit_message: kad::Behaviour::with_config(
                    key.public().to_peer_id(),
                    commit_message,
                    ipfs_cfg.clone(),
                ),
                commit_diff: kad::Behaviour::with_config(
                    key.public().to_peer_id(),
                    commit_diff,
                    ipfs_cfg.clone(),
                ),
                identify: identify::Behaviour::new(identify::Config::new(
                    "/yamux/1.0.0".to_string(),
                    key.public(),
                )),
                rendezvous: rendezvous::server::Behaviour::new(
                    rendezvous::server::Config::default(),
                ),
                ping: ping::Behaviour::new(
                    ping::Config::new().with_interval(Duration::from_secs(1)),
                ),
                kademlia: kad::Behaviour::with_config(
                    key.public().to_peer_id(),
                    kademlia_store,
                    ipfs_cfg.clone(),
                ),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?,
            })
        })?
        .build();

    // Add the bootnodes to the local routing table. `libp2p-dns` built
    // into the `transport` resolves the `dnsaddr` when Kademlia tries
    // to dial these nodes.
    for peer in &IPFS_BOOTNODES {
        swarm
            .behaviour_mut()
            .ipfs
            .add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
        swarm
            .behaviour_mut()
            .commit_message
            .add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
        swarm
            .behaviour_mut()
            .commit_diff
            .add_address(&peer.parse()?, "/dnsaddr/bootstrap.libp2p.io".parse()?);
    }

    // TODO get weeble/blockheight/wobble
    let listen_on = swarm.listen_on("/ip4/0.0.0.0/tcp/62649".parse().unwrap());
    log::debug!("listen_on={}", listen_on.unwrap());
    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
    log::info!("swarm.local_peer_id()={:?}", swarm.local_peer_id());
    //net work is primed

    //run
    //let result = run(&args, &mut swarm.behaviour_mut().kademlia).await;
    //log::trace!("result={:?}", result);

    //push commit hashes and commit diffs

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Listen on all interfaces and whatever port the OS assigns.
    // TODO get weeble/blockheight/wobble
    let listen_on = swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    log::debug!("listen_on={}", listen_on);

    // Kick it off.
    loop {
        //run
        let result = run(&args, &mut swarm.behaviour_mut().kademlia).await;
        log::info!("result={:?}", result);
        //let result = run(&args, &mut swarm.behaviour_mut().ipfs).await;
        //log::trace!("result={:?}", result);
        //let result = run(&args, &mut swarm.behaviour_mut().commit_message).await;
        //log::trace!("result={:?}", result);
        //let result = run(&args, &mut swarm.behaviour_mut().commit_diff).await;
        //log::trace!("result={:?}", result);

        select! {
                Ok(Some(line)) = stdin.next_line() => {
                    log::trace!("line.len()={}", line.len());
                    if line.len() <= 3 {
                    log::debug!("{:?}", swarm.local_peer_id());
                    for address in swarm.external_addresses() {
                        log::trace!("{:?}", address);
                    }
                    for peer in swarm.connected_peers() {
                        log::trace!("{:?}", peer);
                    }
                    }
                    handle_input_line(&mut swarm.behaviour_mut().kademlia, line).await;
                }

                event = swarm.select_next_some() => match event {


                //match event

                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        tracing::trace!("Connected to {}", peer_id);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        tracing::trace!("Disconnected from {}", peer_id);
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
                        rendezvous::server::Event::PeerRegistered { peer, registration },
                    )) => {
                        tracing::info!(
                            "Peer {} registered for namespace '{}'",
                            peer,
                            registration.namespace
                        );
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(
                        rendezvous::server::Event::DiscoverServed {
                            enquirer,
                            registrations,
                        },
                    )) => {
                        tracing::info!(
                            "Served peer {} with {} registrations",
                            enquirer,
                            registrations.len()
                        );
                    }
                //    other => {
                //        tracing::debug!("Unhandled {:?}", other);
                //    }

                SwarmEvent::NewListenAddr { address, .. } => {
                    log::debug!("Listening in {address:?}");
                }


                SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr.clone());
                        tracing::info!("{}", peer_id.clone());
                        tracing::info!("{}", multiaddr.clone());
                    }
                }



                SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed { result, ..})) => {
                match result {
                    kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders { key, providers, .. })) => {
                        for peer in providers {
                            println!(
                                "Peer {peer:?} provides key {:?}",
                                std::str::from_utf8(key.as_ref()).unwrap()
                            );
                        }
                    }
                    kad::QueryResult::GetProviders(Err(err)) => {
                        //eprintln!("Failed to get providers: {err:?}");
                        log::info!("Failed to get providers: {err:?}");
                    }
                    kad::QueryResult::GetRecord(
                        Ok(
                        kad::GetRecordOk::FoundRecord(
                            kad::PeerRecord {
                            record: kad::Record { key, value, .. },
                            ..
                        }
                        )
                        )
                        ) => {
                        //tracing::info!("{}",record);

                        println!(
                            "{{\"key\":{:?},\"value\":{:?}}}",
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            std::str::from_utf8(&value).unwrap(),
                        );


                    }
                    //kad::QueryResult::GetRecord(Ok(_)) => {}
                    kad::QueryResult::GetRecord(Err(err)) => {
                        //eprintln!("Failed to get record: {err:?}");
                        log::info!("Failed to get record: {err:?}");
                    }
                    kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                        log::info!(
                            "PUT {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                    kad::QueryResult::PutRecord(Err(err)) => {
                        //eprintln!("Failed to put record: {err:?}");
                        log::debug!("Failed to put record: {err:?}");
                    }
                    kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
                        log::debug!(
                            "PUT_PROVIDER {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                    kad::QueryResult::StartProviding(Err(err)) => {
                        //eprintln!("Failed to put provider record: {err:?}");
                        log::trace!("Failed to put provider record: {err:?}");
                    }
                    _ => {}
                }
            }
            other => {
                tracing::debug!("Unhandled {:?}", other);
            }
                }
        }
    }
}

//fn commit_list(kademlia: &mut kad::Behaviour<MemoryStore>) {
//    let key = {
//            let Some(key) => kad::RecordKey::new(&key)
//    };
//
//    kademlia
//        .start_providing(key)
//        .expect("Failed to start providing key");
//}
async fn handle_input_line(kademlia: &mut kad::Behaviour<MemoryStore>, line: String) {
    let mut args = line.split(' ');

    match args.next() {
        Some("GET") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("gnostr> GET <commit_hash>");
                        eprint!("gnostr> ");
                        return;
                    }
                }
            };
            let query_id = kademlia.get_record(key.clone());
            //print_record(record);
            tracing::debug!(
                "kademlia.get_record({})\n{}",
                kademlia.get_record(key),
                query_id
            );
        }
        Some("GET_PROVIDERS") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprint!("gnostr> GET_PROVIDERS <commit_hash>");
                        eprint!("gnostr> ");
                        return;
                    }
                }
            };
            kademlia.get_providers(key);
        }
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("gnostr> PUT <key> <value>");
                        eprint!("gnostr> ");
                        return;
                    }
                }
            };
            let value = {
                match args.next() {
                    Some(value) => value.as_bytes().to_vec(),
                    None => {
                        eprintln!("gnostr> PUT {:?} <value>", key);
                        eprint!("gnostr> ");
                        return;
                    }
                }
            };
            let record = kad::Record {
                key,
                value,
                publisher: None,
                expires: None,
            };

            match kademlia.put_record(record, Quorum::One) {
                Ok(query_id) => {
                    // Record was successfully put locally, and a query was started.
                    tracing::debug!(
                        "Successfully started put_record query with ID: {:?}",
                        query_id
                    );
                }
                Err(e) => {
                    // The put_record call failed. Handle the error here.
                    eprintln!("565:Failed to put record: {:?}", e);
                    // You could also specifically check for the MaxRecords error
                    if let libp2p::kad::store::Error::MaxRecords = e {
                        eprintln!("The record could not be stored due to the MaxRecords limit.");
                        // Maybe you want to evict an old record here, or just log and continue.
                    }
                }
            }

            //kademlia
            //  .put_record(record, kad::Quorum::One)
            //.expect("Failed to store record locally.");
        }
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprint!("gnostr> ");
                        return;
                    }
                }
            };
            kademlia
                .start_providing(key)
                .expect("Failed to start providing key");
        }
        Some("QUIT") => {
            std::process::exit(0);
        }
        Some("Q") => {
            std::process::exit(0);
        }
        Some("EXIT") => {
            std::process::exit(0);
        }
        _ => {
            tracing::info!("\nGET, GET_PROVIDERS, PUT, PUT_PROVIDER <commit_hash>");
            eprint!("gnostr> ");
        }
    }
}

fn sig_matches(sig: &Signature, arg: &Option<String>) -> bool {
    match *arg {
        Some(ref s) => {
            sig.name().map(|n| n.contains(s)).unwrap_or(false)
                || sig.email().map(|n| n.contains(s)).unwrap_or(false)
        }
        None => true,
    }
}

fn log_message_matches(msg: Option<&str>, grep: &Option<String>) -> bool {
    match (grep, msg) {
        (&None, _) => true,
        (&Some(_), None) => false,
        (&Some(ref s), Some(msg)) => msg.contains(s),
    }
}

//this formats and prints the commit header/message
fn print_commit_header(commit: &Commit) {
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

//called from above
//part of formatting the output
fn print_time(time: &Time, prefix: &str) {
    let (offset, sign) = match time.offset_minutes() {
        n if n < 0 => (-n, '-'),
        n => (n, '+'),
    };
    let (hours, minutes) = (offset / 60, offset % 60);
    let ts = time::Timespec::new(time.seconds() + (time.offset_minutes() as i64) * 60, 0);
    let time = time::at(ts);

    println!(
        "{}{} {}{:02}{:02}",
        prefix,
        time.strftime("%a %b %e %T %Y").unwrap(),
        sign,
        hours,
        minutes
    );
}

fn match_with_parent(
    repo: &Repository,
    commit: &Commit,
    parent: &Commit,
    opts: &mut DiffOptions,
) -> Result<bool, GitError> {
    let a = parent.tree()?;
    let b = commit.tree()?;
    let diff = repo.diff_tree_to_tree(Some(&a), Some(&b), Some(opts))?;
    Ok(diff.deltas().len() > 0)
}

async fn run(args: &Args, kademlia: &mut kad::Behaviour<MemoryStore>) -> Result<(), GitError> {
    // Assuming you have a Kademlia instance named `kademlia`
    let keystore: &MemoryStore = kademlia.store_mut();

    // Now you can get the length of the keystore
    let num_records = keystore.records().len();

    tracing::info!("The number of records in the keystore is: {}", num_records);

    if num_records >= 1024 {
        tracing::info!("The number of records in the keystore is: {}", num_records);
        //std::process::exit(0);
    }

    let path = args.flag_git_dir.as_ref().map(|s| &s[..]).unwrap_or(".");
    let repo = Repository::discover(path)?;

    let tag_names = &repo.tag_names(Some("")).expect("REASON");
    for tag in tag_names {
        //println!("println!={}", tag.unwrap());
        log::trace!("tag.unwrap()={}", tag.unwrap());
    }

    let mut revwalk = repo.revwalk()?;

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

    let tag_names = &repo.tag_names(Some("")).expect("REASON");
    log::debug!("tag_names.len()={}", tag_names.len());
    for tag in tag_names {
        log::trace!("{}", tag.unwrap());
        let key = kad::RecordKey::new(&format!("{}", &tag.unwrap()));

        ////push commit key and commit content as value
        ////let value = Vec::from(commit.message_bytes().clone());
        //let value = Vec::from(commit.message_bytes());
        //let record = kad::Record {
        //    key,
        //    value,
        //    publisher: None,
        //    expires: None,
        //};
        //kademlia
        //    .put_record(record, kad::Quorum::One)
        //    .expect("Failed to store record locally.");
        //let key = kad::RecordKey::new(&format!("{}", &commit.id()));
        //kademlia
        //    .start_providing(key)
        //    .expect("Failed to start providing key");
    }

    // print!
    for commit in revwalk {
        let commit = commit?;

        //TODO construct nostr event
        //commit_privkey
        let commit_privkey: String = String::from(format!("{:0>64}", &commit.id().clone()));
        log::debug!("commit_privkey=\n{}", commit_privkey);

        //commit.id
        //we want to broadcast as provider for the actual commit.id()
        log::debug!("&commit.id=\n{}", &commit.id());

        //store git commit message
        let key = kad::RecordKey::new(&format!("{}", &commit.id()));

        //push commit key and commit content as value
        //let value = Vec::from(commit.message_bytes().clone());
        let value = Vec::from(commit.message_bytes());
        tracing::debug!("value={:?}", value.clone());

        let quorum = Quorum::from(Quorum::Majority);
        let record = kad::Record {
            key,
            value,
            publisher: None,
            expires: None,
        };

        match kademlia.put_record(record, Quorum::One) {
            Ok(query_id) => {
                // Record was successfully put locally, and a query was started.
                tracing::debug!(
                    "Successfully started put_record query with ID: {:?}",
                    query_id
                );
            }
            Err(e) => {
                // The put_record call failed. Handle the error here.
                eprintln!("874:Failed to put record: {:?}", e);
                // You could also specifically check for the MaxRecords error
                if let libp2p::kad::store::Error::MaxRecords = e {
                    eprintln!("The record could not be stored due to the MaxRecords limit.");
                    // Maybe you want to evict an old record here, or just log and continue.
                }
            }
        }

        //        kademlia
        //            //commit_message
        //            .put_record(record, quorum)
        //            .expect("Failed to store record locally.");
        let key = kad::RecordKey::new(&format!("{}", &commit.id()));
        ////kademlia
        //commit_message
        //    .start_providing(key)
        //    .expect("Failed to start providing key");

        let repo_path = "."; // Path to your Git repository
        let repo = Repository::discover(repo_path).expect("Failed to open repository");

        //let commit_id = "your_commit_hash_here"; // Replace with a valid commit hash
        let message_bytes = get_commit_message_bytes(&repo, &commit.id().to_string())
            .expect("Failed to get commit message bytes");

        match String::from_utf8(message_bytes) {
            Ok(message) => {
                tracing::debug!("message:\n{}", message);
            }
            Err(e) => {
                eprintln!("Failed to decode commit message: {}", e);
                // You can inspect the bytes that caused the error
                eprintln!("Invalid bytes: {:?}", e.as_bytes());
            }
        }

        //store git commit diff <commit_hash>-diff
        let key = kad::RecordKey::new(&format!("{}-diff", &commit.id()));
        let diff = get_commit_diff_as_string(&repo, commit.id());
        tracing::debug!("diff={:?}", diff?);
        let value = get_commit_diff_as_bytes(&repo, commit.id())?;

        let record = kad::Record {
            key,
            value,
            publisher: None,
            expires: None,
        };

        //match kademlia.put_record(record, quorum) {
        match kademlia.put_record(record, Quorum::One) {
            Ok(query_id) => {
                // Record was successfully put locally, and a query was started.
                tracing::debug!(
                    "Successfully started put_record query with ID: {:?}",
                    query_id
                );
            }
            Err(e) => {
                // The put_record call failed. Handle the error here.
                eprintln!("934:Failed to put record: {:?}", e);
                // You could also specifically check for the MaxRecords error
                if let libp2p::kad::store::Error::MaxRecords = e {
                    eprintln!("The record could not be stored due to the MaxRecords limit.");
                    // Maybe you want to evict an old record here, or just log and continue.
                }
            }
        }

        //        kademlia
        //            .put_record(record, kad::Quorum::One)
        //            .expect("Failed to store record locally.");
        let key = kad::RecordKey::new(&format!("{}", &commit.id()));
        kademlia
            .start_providing(key)
            .expect("Failed to start providing key");

        ////println!("commit.tree_id={}", &commit.tree_id());
        //let key = kad::RecordKey::new(&format!("{}", &commit.tree_id()));
        ////println!("commit.tree={:?}", &commit.tree());
        //let value = Vec::from(format!("{:?}", commit.tree()));
        //let record = kad::Record {
        //    key,
        //    value,
        //    publisher: None,
        //    expires: None,
        //};
        //kademlia
        //    .put_record(record, kad::Quorum::One)
        //    .expect("Failed to store record locally.");
        //let key = kad::RecordKey::new(&format!("{}", &commit.id()));
        //kademlia
        //    .start_providing(key)
        //    .expect("Failed to start providing key");

        //println!("commit.tree={:?}", &commit.tree());
        //println!("commit.raw={:?}", &commit.raw()); //pointer?

        //println!("commit.message={:?}", &commit.message()); //commit diff body
        let mut part_index = 0;
        let commit_parts = commit.message().clone().unwrap().split("\n");
        //let parts = commit.message().clone().unwrap().split("gpgsig");
        for part in commit_parts {
            log::debug!(
                "commit.message part={}:{}",
                part_index,
                part.replace("", "")
            );
            part_index += 1;
        }
        part_index = 0;

        ////println!("commit.message_bytes{:?}", &commit.message_bytes());
        //println!("commit.message_encoding={:?}", &commit.message_encoding());
        //println!("commit.message_raw={:?}", &commit.message_raw());
        ////println!("commit.message_raw_bytes={:?}", &commit.message_raw_bytes());

        //raw_header
        //println!("commit.raw_header={:?}", commit.raw_header());
        let raw_header_parts = commit.raw_header().clone().unwrap().split("\n");
        for part in raw_header_parts {
            log::debug!("raw_header part={}:{}", part_index, part.replace("", ""));
            part_index += 1;
        }
        //parts = commit.raw_header().clone().unwrap().split("gpgsig");
        //for part in parts {
        //    println!("raw_header gpgsig part={}", part.replace("", ""))
        //};
        ////println!("commit.header_field_bytes={:?}", &commit.header_field_bytes());
        ////println!("commit.raw_header_bytes={:?}", &commit.raw_header_bytes());
        //println!("commit.summary={:?}", &commit.summary());
        ////println!("commit.summary_bytes={:?}", &commit.summary_bytes());
        //println!("commit.body={:?}", &commit.body());
        ////println!("commit.body_bytes={:?}", &commit.body_bytes());
        //println!("commit.time={:?}", &commit.time());
        //println!("commit.author={:?}", &commit.author().name());
        //print_commit_header(&commit);

        if !args.flag_patch || commit.parents().len() > 1 {
            continue;
        }

        let a = if commit.parents().len() == 1 {
            //we have arrived at the initial commit
            let parent = commit.parent(0)?;
            Some(parent.tree()?)
        } else {
            None
        };

        //print the diff content
        //push diff to commit_key
        let b = commit.tree()?;
        let diff = repo.diff_tree_to_tree(a.as_ref(), Some(&b), Some(&mut diffopts2))?;
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                ' ' | '+' | '-' => print!("{}", line.origin()),
                _ => {}
            }
            print!(
                "769:==================>{}",
                str::from_utf8(line.content()).unwrap()
            );
            true
        })?;
    }

    Ok(())
}

//TODO Server Mode or ??
#[derive(Debug, Parser)]
struct Args {
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

impl Args {
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
