#![doc = include_str!("../../README.md")]
use gnostr::p2p::handle_input::handle_input_line;
use gnostr::p2p::opt;

use clap::Parser;
use futures::{prelude::*, StreamExt};
use gnostr::p2p::kvs::{new, Event};
use libp2p::multiaddr::Protocol;
use std::{error::Error, io::Write};
use tokio::task::spawn;
use tracing_subscriber::EnvFilter;

use gnostr::p2p::opt::CliArgument;
use gnostr::p2p::opt::Opt;
use base64::{engine::general_purpose, Engine as _};
use git2::Repository;
//use libp2p::identity::Keypair;
//use libp2p::request_response::Behaviour;
use libp2p::{
    kad,
    kad::{store::MemoryStore, Mode},
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::path::Path;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    select,
};

fn hash_folder_name(path: &Path) -> Option<String> {
    if let Some(folder_name) = path.file_name().and_then(|name| name.to_str()) {
        let mut hasher = Sha256::new();
        hasher.update(folder_name.as_bytes());
        let result = hasher.finalize();
        Some(format!("{result:x}"))
    } else {
        None
    }
}

fn create_keypair_from_hex_string(
    secret_key_hex: &str,
) -> Result<libp2p::identity::Keypair, hex::FromHexError> {
    // The secret key for Ed25519 is 32 bytes.
    let mut secret_key_bytes = [0u8; 32];
    hex::decode_to_slice(secret_key_hex, &mut secret_key_bytes)?;
    Ok(libp2p::identity::Keypair::ed25519_from_bytes(secret_key_bytes).unwrap())
}

fn get_repo_name<P: AsRef<Path>>(repo_path: P) -> Result<Option<String>, git2::Error> {
    //discover repo root
    let repo = Repository::discover(repo_path)?;

    // Get the path of the repository
    let path = repo.path();

    //println!("repo_name:{}", path.display());
    // The repo path typically ends in `.git`.
    // We want the parent directory's name.
    let parent_path = path.parent().unwrap_or(path);

    // Use `file_name()` to get the last component of the path.
    // `to_str()` is used to convert OsStr to a string slice.
    let repo_name = parent_path.file_name().and_then(|name| name.to_str());

    Ok(repo_name.map(String::from))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let opt = Opt::parse();

    let (mut network_client, mut network_events, network_event_loop) =
        new(opt.secret_key_seed).await?;

    // Spawn the network task for it to run in the background.
    spawn(network_event_loop.run());

    // In case a listen address was provided use it, otherwise listen on any
    // address.
    match opt.listen_address {
        Some(addr) => network_client
            .start_listening(addr)
            .await
            .expect("Listening not to fail."),
        None => network_client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail."),
    };

    // In case the user provided an address of a peer on the CLI, dial it.
    if let Some(addr) = opt.peer {
        let Some(Protocol::P2p(peer_id)) = addr.iter().last() else {
            return Err("Expect peer multiaddr to contain peer ID.".into());
        };
        network_client
            .dial(peer_id, addr)
            .await
            .expect("Dial to succeed");
    }

    match opt.argument {
        // Providing a file.
        CliArgument::Provide { path, name } => {
            // Advertise oneself as a provider of the file on the DHT.
            network_client.start_providing(name.clone()).await;

            loop {
                match network_events.next().await {
                    // Reply with the content of the file on incoming requests.
                    Some(Event::InboundRequest { request, channel }) => {
                        if request == name {
                            network_client
                                .respond_file(std::fs::read(&path)?, channel)
                                .await;
                        }
                    }
                    e => todo!("{:?}", e),
                }
            }
        }
        // Locating and getting a file.
        CliArgument::Get { name } => {
            // Locate all nodes providing the file.
            let providers = network_client.get_providers(name.clone()).await;
            if providers.is_empty() {
                return Err(format!("Could not find provider for file {name}.").into());
            }

            // Request the content of the file from each node.
            let requests = providers.into_iter().map(|p| {
                let mut network_client = network_client.clone();
                let name = name.clone();
                async move { network_client.request_file(p, name).await }.boxed()
            });

            // Await the requests, ignore the remaining once a single one succeeds.
            let file_content = futures::future::select_ok(requests)
                .await
                .map_err(|_| "None of the providers returned file.")?
                .0;

            std::io::stdout().write_all(&file_content)?;
        }
        CliArgument::Kv { get } => {
            println!("get={}", get.clone().unwrap_or("gnostr".to_string()));
            if let Ok(_result) =
                key_value(&format!("GET {}", get.unwrap_or("gnostr".to_string()))).await
            {
            };
        }
    }

    Ok(())
}

async fn key_value(get: &str) -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    println!("get={get}");
    //let file_transfer = file_transfer().await;
    let my_path = Path::new(".");

    let repo_name = get_repo_name(my_path);

    let repo_name_clone = get_repo_name(my_path);
    let repo_name_clone2 = get_repo_name(my_path);

    println!("repo_name={}", repo_name_clone.unwrap().ok_or("")?);

    let keypair: libp2p::identity::Keypair;

    // We create a custom network behaviour that combines Kademlia and mDNS.
    #[derive(NetworkBehaviour)]
    struct Behaviour {
        kademlia: kad::Behaviour<MemoryStore>,
        mdns: mdns::tokio::Behaviour,
    }

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            Ok(Behaviour {
                kademlia: kad::Behaviour::new(
                    key.public().to_peer_id(),
                    MemoryStore::new(key.public().to_peer_id()),
                ),
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?,
            })
        })?
        .build();

    if let Some(hash) = hash_folder_name(Path::new(&repo_name?.expect("").to_string())) {
        println!("hash_folder_name={hash}");

        keypair = create_keypair_from_hex_string(&hash).expect("");

        println!("{:?}", keypair.public());

        let protobuf_bytes = keypair
            .to_protobuf_encoding()
            .expect("should be able to encode keypair");

        let base64_string = general_purpose::STANDARD.encode(&protobuf_bytes);

        print!("Keypair as Base64 string (Protobuf encoded):\n{base64_string}");

        let decoded_bytes = general_purpose::STANDARD
            .decode(&base64_string)
            .expect("should be able to decode base64");

        let rehydrated_keypair = libp2p::identity::Keypair::from_protobuf_encoding(&decoded_bytes)
            .expect("should be able to decode protobuf bytes");

        let rehydrated_public_key = rehydrated_keypair.public();
        println!("\nRehydrated Public Key:\n{rehydrated_public_key:?}");

        assert_eq!(keypair.public(), rehydrated_public_key);
        println!("Successfully rehydrated the keypair! They are identical.");

        let line = format!("PUT {:?} {:?}", repo_name_clone2.unwrap().unwrap(), hash);
        println!("{line}");
        put_repo_key_value(&mut swarm.behaviour_mut().kademlia, line.replace("\"", ""));
    } else {
        println!("Could not get folder name.");
    }

    //prime network
    //prime network
    //prime network
    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
    // Listen on all interfaces and whatever port the OS assigns.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    //
    //
    //

    //handle initial function key_value(get: &str) {...}
    let byte_slice: &[u8] = get.as_bytes();
    let buf_reader = BufReader::new(byte_slice);
    let mut lines_stream = buf_reader.lines();
    println!("Reading from the mocked async stream:");
    while let Ok(Some(line_result)) = lines_stream.next_line().await {
        let line = line_result;
        {
            println!("Read an async line: {line}");
            handle_input_line(&mut swarm.behaviour_mut().kademlia, line);
        }
    }

    //handle input from user
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Kick it off.
    loop {
        select! {
        Ok(Some(line)) = stdin.next_line() => {
            handle_input_line(&mut swarm.behaviour_mut().kademlia, line);
        }
        event = swarm.select_next_some() => match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening in {address:?}");
            },
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
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
                        eprintln!("Failed to get providers: {err:?}");
                    }
                    kad::QueryResult::GetRecord(Ok(
                        kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                            record: kad::Record { key, value, .. },
                            ..
                        })
                    )) => {
                        println!(
                            "Got record {:?} {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            std::str::from_utf8(&value).unwrap(),
                        );
                    }
                    kad::QueryResult::GetRecord(Ok(_)) => {}
                    kad::QueryResult::GetRecord(Err(err)) => {
                        eprintln!("Failed to get record: {err:?}");
                    }
                    kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                        println!(
                            "Successfully put record {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                    kad::QueryResult::PutRecord(Err(_err)) => {
                        //eprintln!("Quorum may have failed to put record: {_err:?}");
                    }
                    kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
                        println!(
                            "Successfully put provider record {:?}",
                            std::str::from_utf8(key.as_ref()).unwrap()
                        );
                    }
                    kad::QueryResult::StartProviding(Err(err)) => {
                        eprintln!("Failed to put provider record: {err:?}");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        }
    }
}

fn put_repo_key_value(kademlia: &mut kad::Behaviour<MemoryStore>, line: String) {
    println!("line={}", line.replace("\"", ""));
    let line = line.to_string();
    println!("line={}", line.replace("\"", ""));
    let mut args = line.split(' ');
    match args.next() {
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };
            let value = {
                match args.next() {
                    Some(value) => value.as_bytes().to_vec(),
                    None => {
                        eprintln!("Expected value");
                        return;
                    }
                }
            };
            let key_clone = key.clone();
            let record = kad::Record {
                key,
                value,
                publisher: None,
                expires: None,
            };
            kademlia
                .put_record(
                    record,
                    kad::Quorum::N(NonZeroUsize::new(1).expect("REASON")),
                )
                .expect("Failed to store record locally.");
            kademlia
                .start_providing(key_clone)
                .expect("Failed to start providing key");
        }
        _ => {
            eprintln!("put_repo_key_value failed!");
        }
    }
}
