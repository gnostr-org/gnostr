use std::{
    fs, io,
    path::{Path, PathBuf},
};

use futures::StreamExt;
use gnostr_p2p::keypair_from_seed;
use gnostr_p2p::message::{EventBuilder, EventKind, PrivateKey, Tag};
use gnostr_p2p::perfect_ip::{
    build_fractal_swarm, generate_manifest, packetize, summarize_packets, FractalBehaviourEvent,
    IntegrityManager, ProtocolSlice,
};
use libp2p::{request_response, swarm::SwarmEvent};
use log::{debug, info};
use sha2::{Digest, Sha256};

#[derive(Debug)]
struct DemoArgs {
    file: Option<PathBuf>,
    out: Option<PathBuf>,
    recursive: Option<PathBuf>,
    depth: usize,
    depth_set: bool,
    logging: Option<String>,
    help: bool,
    serve_network: bool,
}

#[derive(Debug)]
struct DirectoryWalkResult {
    lines: Vec<String>,
    packets: Vec<ProtocolSlice>,
}

struct FileModeResult {
    root_id: String,
    original_sha256: String,
    original_len: usize,
    packets: Vec<ProtocolSlice>,
    integrity: IntegrityManager,
}

fn usage() {
    println!(
        "Usage: fractal_swarm_perfect_ip [--file PATH] [--out PATH] [--recursive PATH] [--depth N] [--logging] [--serve-network] [--help]\n\
         \n\
         Options:\n\
           --file PATH         Read a single file from PATH\n\
           --out PATH          Write reconstructed bytes to PATH [default: example.reconstructed.bin]\n\
           --recursive PATH    Walk PATH as a directory tree and preserve relative paths\n\
           --depth N           Limit recursive directory walking to N levels [default: 3]\n\
           --logging [LEVEL]   Enable verbose logging (default level: info)\n\
           --serve-network     Broadcast PIP events and run libp2p repair loop\n\
           --help              Show this help message\n"
    );
}

fn parse_usize(value: &str, flag: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}

fn parse_required_path(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<PathBuf, String> {
    match args.peek() {
        Some(next) if !next.starts_with('-') => Ok(PathBuf::from(args.next().unwrap_or_default())),
        _ => Err(format!("missing value for {flag}")),
    }
}

fn parse_args() -> Result<DemoArgs, String> {
    let mut args = std::env::args().skip(1).peekable();
    let mut file = None;
    let mut out = None;
    let mut recursive = None;
    let mut depth = 3usize;
    let mut depth_set = false;
    let mut logging: Option<String> = None;
    let mut help = false;
    let mut serve_network = false;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            help = true;
        } else if arg == "--serve-network" {
            serve_network = true;
        } else if arg == "--logging" {
            logging = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next(),
                _ => Some("info".to_string()),
            };
        } else if arg == "--recursive" {
            recursive = Some(parse_required_path(&mut args, "--recursive")?);
        } else if let Some(path) = arg.strip_prefix("--recursive=") {
            if path.is_empty() {
                return Err("missing value for --recursive".to_string());
            }
            recursive = Some(PathBuf::from(path));
        } else if arg == "--file" {
            file = Some(parse_required_path(&mut args, "--file")?);
        } else if let Some(path) = arg.strip_prefix("--file=") {
            if path.is_empty() {
                return Err("missing value for --file".to_string());
            }
            file = Some(PathBuf::from(path));
        } else if arg == "--out" {
            out = Some(parse_required_path(&mut args, "--out")?);
        } else if let Some(path) = arg.strip_prefix("--out=") {
            if path.is_empty() {
                return Err("missing value for --out".to_string());
            }
            out = Some(PathBuf::from(path));
        } else if arg == "--depth" {
            let raw = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next().unwrap_or_default(),
                _ => return Err("missing value for --depth".to_string()),
            };
            depth = parse_usize(&raw, "--depth")?;
            depth_set = true;
        } else if let Some(value) = arg.strip_prefix("--depth=") {
            if value.is_empty() {
                return Err("missing value for --depth".to_string());
            }
            depth = parse_usize(value, "--depth")?;
            depth_set = true;
        } else {
            return Err(format!("unrecognized argument: {arg}"));
        }
    }

    Ok(DemoArgs {
        file,
        out,
        recursive,
        depth,
        depth_set,
        logging,
        help,
        serve_network,
    })
}

fn validate_args(args: &DemoArgs) -> Result<(), String> {
    if args.file.is_some() && args.recursive.is_some() {
        return Err("--file and --recursive are mutually exclusive".to_string());
    }
    if args.depth_set && args.recursive.is_none() {
        return Err("--depth is only valid with --recursive".to_string());
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

fn reconstruct_payload(packets: &[ProtocolSlice]) -> Vec<u8> {
    let mut leaves: Vec<_> = packets.iter().filter(|packet| !packet.is_parity).collect();
    leaves.sort_by_key(|packet| packet.header.seq_num);
    leaves
        .into_iter()
        .flat_map(|packet| packet.data.clone())
        .collect()
}

fn collect_directory_walk(
    root: &Path,
    current: &Path,
    depth: usize,
    level: usize,
    logging: bool,
    lines: &mut Vec<String>,
    all_packets: &mut Vec<ProtocolSlice>,
) -> io::Result<()> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(current)? {
        entries.push(entry?);
    }
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let indent = "  ".repeat(level + 1);
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            lines.push(format!("{indent}{}/", relative.display()));
            if level + 1 < depth {
                collect_directory_walk(root, &path, depth, level + 1, logging, lines, all_packets)?;
            }
        } else if metadata.is_file() {
            let bytes = fs::read(&path)?;
            lines.push(format!(
                "{indent}{} ({}B sha256:{})",
                relative.display(),
                bytes.len(),
                sha256_hex(&bytes)
            ));
            let batch = packetize(relative.to_string_lossy().into_owned(), bytes);
            all_packets.extend(batch.packets.clone());
            if logging {
                info!("File: {}", relative.display());
                info!("Batch size: {} packets", batch.total_packets);
                lines.push(format!(
                    "{indent}  perfect_ip packet batch: {} packets",
                    batch.total_packets
                ));
                for line in summarize_packets(&batch.packets) {
                    debug!("{line}");
                    lines.push(format!("{indent}  {line}"));
                }
            }
        } else {
            lines.push(format!("{indent}{} [special]", relative.display()));
        }
    }

    Ok(())
}

fn run_directory_walk(path: &Path, depth: usize, logging: bool) -> io::Result<DirectoryWalkResult> {
    let mut lines = vec![".".to_string()];
    let mut all_packets = Vec::new();

    if depth > 0 {
        collect_directory_walk(path, path, depth, 0, logging, &mut lines, &mut all_packets)?;
    }

    Ok(DirectoryWalkResult {
        lines,
        packets: all_packets,
    })
}

fn run_file_mode(
    file: Option<PathBuf>,
    out: Option<PathBuf>,
    logging: bool,
) -> io::Result<FileModeResult> {
    let example_path = file.unwrap_or_else(|| PathBuf::from("example.bin"));

    let original_bytes = if example_path.exists() {
        if example_path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("--file path is a directory: {}", example_path.display()),
            ));
        }
        fs::read(&example_path)?
    } else {
        let bytes = b"perfect_ip demo payload\n".repeat(128);
        fs::write(&example_path, &bytes)?;
        bytes
    };

    let original_sha256 = sha256_hex(&original_bytes);
    let root_id = "ROOT".to_string();
    let batch = packetize(root_id.clone(), original_bytes.clone());
    let manifest = generate_manifest(root_id.clone(), original_bytes.len());

    println!("sender wrote {}", example_path.display());
    println!("sender sha256: {original_sha256}");
    if logging {
        info!("perfect_ip packet batch: {} packets", batch.total_packets);
        for line in summarize_packets(&batch.packets) {
            debug!("{line}");
        }
    }

    let mut integrity = IntegrityManager::new(manifest);
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

    Ok(FileModeResult {
        root_id,
        original_sha256,
        original_len: original_bytes.len(),
        packets: batch.packets,
        integrity,
    })
}

fn run_recursive_mode(
    root: &Path,
    depth: usize,
    logging: bool,
    out: Option<&Path>,
) -> io::Result<()> {
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("--recursive path is not a directory: {}", root.display()),
        ));
    }

    info!("recursive walk root: {}", root.display());
    info!("recursive depth: {}", depth);

    let walk = run_directory_walk(root, depth, logging)?;
    for line in walk.lines {
        println!("{line}");
    }

    if let Some(out_path) = out {
        let reconstructed = reconstruct_payload(&walk.packets);
        fs::write(out_path, &reconstructed)?;
        println!(
            "recursively reconstructed {} bytes to {}",
            reconstructed.len(),
            out_path.display()
        );
    }

    Ok(())
}

async fn broadcast_pip_events(
    root_id: &str,
    original_sha256: &str,
    original_len: usize,
    packets: &[ProtocolSlice],
) {
    let private_key = PrivateKey::generate();
    let config_dir = gnostr_p2p::p2p::relay_paths::get_config_dir_path();
    let manifest = generate_manifest(root_id.to_string(), original_len);

    let manifest_content = serde_json::to_string(&manifest).expect("serialize manifest");
    let manifest_tags = vec![
        Tag::new_identifier(root_id.to_string()),
        Tag::new_tag("sha256", original_sha256),
    ];
    let manifest_event =
        EventBuilder::new(EventKind::Other(39078), manifest_content, manifest_tags)
            .to_event(&private_key)
            .expect("build manifest event");

    info!(
        "Broadcasting PIP Manifest event: {}",
        manifest_event.id.as_hex_string()
    );
    if let Ok(count) = gnostr_p2p::p2p::crawler_broadcast::broadcast_event_to_crawler_relays(
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

    for slice in packets {
        let slice_content = serde_json::to_string(slice).expect("serialize slice");
        let slice_tags = vec![
            Tag::new_identifier(root_id.to_string()),
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
        if let Ok(count) = gnostr_p2p::p2p::crawler_broadcast::broadcast_event_to_crawler_relays(
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
}

async fn serve_repair_loop(
    integrity: IntegrityManager,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_key = keypair_from_seed(None);
    let mut swarm = build_fractal_swarm(local_key).await?;
    let listen_address = "/ip4/127.0.0.1/udp/0/quic-v1".parse()?;
    let listener_id = swarm.listen_on(listen_address)?;
    println!("repair swarm listener: {listener_id:?}");
    println!("press Ctrl-C to stop the demo");

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = parse_args().map_err(io::Error::other)?;

    if args.help {
        usage();
        return Ok(());
    }

    validate_args(&args).map_err(io::Error::other)?;

    if let Some(ref level) = args.logging {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();
    }

    if let Some(ref root) = args.recursive {
        run_recursive_mode(
            root,
            args.depth,
            args.logging.is_some(),
            args.out.as_deref(),
        )?;
        return Ok(());
    }

    let file_result = run_file_mode(args.file, args.out, args.logging.is_some())?;

    if args.serve_network {
        broadcast_pip_events(
            &file_result.root_id,
            &file_result.original_sha256,
            file_result.original_len,
            &file_result.packets,
        )
        .await;
        serve_repair_loop(file_result.integrity).await?;
    }

    Ok(())
}
