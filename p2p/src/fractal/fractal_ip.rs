use std::{fs, path::PathBuf};

use crate::keypair_from_seed;
use futures::StreamExt;
use log::{debug, info};
// pub use crate::perfect_ip_core::*;
use crate::fractal::fractal_ip_core::{
    build_fractal_swarm, generate_manifest, packetize, reconstruct_directory, reconstruct_payload,
    run_directory_walk, sha256_hex, summarize_packets, FractalBehaviourEvent, IntegrityManager,
    ProtocolSlice,
};
use crate::message::{EventBuilder, EventKind, PrivateKey, Tag};
use libp2p::{request_response, swarm::SwarmEvent};

#[derive(Debug)]
struct DemoArgs {
    file: Option<PathBuf>,
    out: Option<PathBuf>,
    recursive: Option<PathBuf>,
    depth: usize,
    depth_set: bool,
    logging: Option<String>,
    help: bool,
}

fn usage() {
    println!(
        "Usage: fractal_swarm_perfect_ip [--file PATH] [--out PATH] [--recursive PATH] [--depth N] [--logging] [--help]\n\
         \n\
         Options:\n\
           --file PATH       Read a single file from PATH\n\
          --out PATH        Write reconstructed bytes to PATH [default: example.reconstructed.bin]\n\
          --recursive PATH   Walk PATH as a directory tree and preserve relative paths\n\
          --depth N         Limit recursive directory walking to N levels [default: 3]\n\
          --logging         Enable verbose logging\n\
           --help            Show this help message\n"
    );
}

fn parse_args() -> DemoArgs {
    let mut args = std::env::args().skip(1).peekable();
    let mut file = None;
    let mut out = None;
    let mut recursive = None;
    let mut depth = 3usize;
    let mut depth_set = false;
    let mut logging: Option<String> = None;
    let mut help = false;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            help = true;
        } else if arg == "--logging" {
            logging = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next(),
                _ => Some("info".to_string()),
            };
        } else if arg == "--recursive" {
            recursive = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next().map(PathBuf::from),
                _ => {
                    help = true;
                    None
                }
            };
        } else if let Some(path) = arg.strip_prefix("--recursive=") {
            recursive = Some(PathBuf::from(path));
        } else if let Some(path) = arg.strip_prefix("--file=") {
            file = Some(PathBuf::from(path));
        } else if let Some(path) = arg.strip_prefix("--out=") {
            out = Some(PathBuf::from(path));
        } else if arg == "--out" {
            out = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next().map(PathBuf::from),
                _ => {
                    help = true;
                    None
                }
            };
        } else if arg == "--file" {
            file = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next().map(PathBuf::from),
                _ => {
                    help = true;
                    None
                }
            };
        } else if let Some(value) = arg.strip_prefix("--depth=") {
            depth = value.parse().unwrap_or(3);
            depth_set = true;
        } else if arg == "--depth" {
            depth = match args.peek() {
                Some(next) if !next.starts_with('-') => args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(3),
                _ => {
                    help = true;
                    3
                }
            };
            depth_set = true;
        } else {
            eprintln!("unrecognized argument: {arg}");
            help = true;
        }
    }

    if recursive.is_none() && depth_set {
        help = true;
    }
    if recursive.is_some() && file.is_some() {
        help = true;
    }

    DemoArgs {
        file,
        out,
        recursive,
        depth,
        depth_set,
        logging,
        help,
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let DemoArgs {
        file,
        out,
        recursive,
        depth,
        depth_set,
        logging,
        help,
    } = parse_args();

    if let Some(ref level) = logging {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();
    }

    if help {
        usage();
        return Ok(());
    }

    if let Some(ref root) = recursive {
        if file.is_some() {
            return Err("--file and --recursive are mutually exclusive".into());
        }

        if !root.is_dir() {
            usage();
            return Ok(());
        }

        info!("recursive walk root: {}", root.display());
        info!("recursive depth: {}", depth);
        info!("logging flag is: {:?}", logging);
        let walk = run_directory_walk(root, depth, logging.is_some())?;
        if let Some(ref out_path) = out {
            reconstruct_directory(walk, out_path)?;
            println!(
                "recursively reconstructed directory to {}",
                out_path.display()
            );
            return Ok(());
        }
        return Ok(());
    }

    // Commented out the old check to allow --depth when --recursive is used:
    // if depth_set {
    //     return Err("--depth is only valid with --recursive".into());
    // }
    if recursive.is_none() && depth_set {
        return Err("--depth is only valid with --recursive".into());
    }

    let example_path = if let Some(ref root) = recursive {
        root.clone()
    } else {
        file.unwrap_or_else(|| PathBuf::from("example.bin"))
    };

    let original_bytes = if example_path.exists() {
        if example_path.is_dir() {
            // If it's a directory, we need to decide how to packetize it.
            // For now, treat it as empty or handle appropriately.
            vec![]
        } else {
            fs::read(&example_path)?
        }
    } else {
        let bytes = b"perfect_ip demo payload\n".repeat(128);
        fs::write(&example_path, &bytes)?;
        bytes
    };

    let original_sha256 = sha256_hex(&original_bytes);
    let batch = packetize("ROOT".to_string(), original_bytes.clone());
    let manifest = generate_manifest("ROOT".to_string(), original_bytes.len());

    println!("sender wrote {}", example_path.display());
    println!("sender sha256: {original_sha256}");
    if logging.is_some() {
        info!("perfect_ip packet batch: {} packets", batch.total_packets);
        for line in summarize_packets(&batch.packets) {
            debug!("{line}");
        }
    }

    let mut integrity = IntegrityManager::new(manifest.clone());
    for slice in batch.packets.clone() {
        integrity.record_slice(slice);
    }
    println!("missing nodes: {:?}", integrity.get_missing_nodes());
    println!("integrity verified: {}", integrity.verify_integrity());

    let reconstructed = reconstruct_payload(&batch.packets);
    let reconstructed_sha256 = sha256_hex(&reconstructed);
    let output_path = out.unwrap_or_else(|| PathBuf::from("example.reconstructed.bin"));
    fs::write(&output_path, &reconstructed)?;

    println!("peer reconstructed {}", output_path.display());
    println!("peer sha256: {reconstructed_sha256}");
    println!("sha256 match: {}", reconstructed_sha256 == original_sha256);

    let local_key = keypair_from_seed(None);
    let mut swarm = build_fractal_swarm(local_key).await?;
    let listen_address = "/ip4/127.0.0.1/udp/0/quic-v1".parse()?;
    let listener_id = swarm.listen_on(listen_address)?;
    println!("repair swarm listener: {listener_id:?}");
    println!("press Ctrl-C to stop the demo");

    // Construct and broadcast PIP events
    let private_key = PrivateKey::generate();
    let config_dir = crate::p2p::relay_paths::get_config_dir_path();

    // 1. Broadcast Manifest Event (Kind 39078)
    let manifest_content = serde_json::to_string(&manifest).expect("serialize manifest");
    let manifest_tags = vec![
        Tag::new_identifier("ROOT".to_string()),
        Tag::new_tag("sha256", &original_sha256),
    ];
    let manifest_event =
        EventBuilder::new(EventKind::Other(39078), manifest_content, manifest_tags)
            .to_event(&private_key)
            .expect("build manifest event");

    info!(
        "Broadcasting PIP Manifest event: {}",
        manifest_event.id.as_hex_string()
    );
    if let Ok(count) = crate::p2p::crawler_broadcast::broadcast_event_to_crawler_relays(
        &config_dir,
        &manifest_event,
    )
    .await
    {
        info!(
            "Broadcasted PIP Manifest event: {:?}. Published to {} relays.",
            manifest_event.id, count
        );
    }

    // 2. Broadcast Slice Events (Kind 39079)
    for slice in batch.packets {
        let slice_content = serde_json::to_string(&slice).expect("serialize slice");
        let slice_tags = vec![
            Tag::new_identifier("ROOT".to_string()),
            Tag::new_event(manifest_event.id, None, Some("root".to_string())),
            Tag::new_tag("seq", &slice.header.seq_num.to_string()),
            Tag::new_tag("path", &slice.id),
        ];
        let slice_event = EventBuilder::new(EventKind::Other(39079), slice_content, slice_tags)
            .to_event(&private_key)
            .expect("build slice event");

        debug!(
            "Broadcasting PIP Slice event: {}",
            serde_json::to_string_pretty(&slice_event).expect("serialize event")
        );
        if let Ok(count) = crate::p2p::crawler_broadcast::broadcast_event_to_crawler_relays(
            &config_dir,
            &slice_event,
        )
        .await
        {
            debug!(
                "Broadcasted PIP Slice event: {:?}. Published to {} relays.",
                slice_event.id, count
            );
        }
    }

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("stopping demo");
                break;
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) = event {
                    if let request_response::Event::Message { message, .. } = event {
                        if let request_response::Message::Request { request, channel, .. } = message {
                            let response = integrity
                                .received_slices
                                .get(&request.id)
                                .cloned()
                                .unwrap_or_else(|| ProtocolSlice {
                                    id: request.id.clone(),
                                    header: request.header.clone(),
                                    data: Vec::new(),
                                    is_parity: false,
                                });
                            let _ = swarm.behaviour_mut().repair_rpc.send_response(channel, response);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
