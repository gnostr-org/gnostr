use std::{fs, path::Path, path::PathBuf, process::Command};

use crate::keypair_from_seed;
use clap::Parser;
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

#[derive(Debug, Parser)]
#[command(name = "fractal_swarm_perfect_ip")]
struct Cli {
    /// Read a single file from PATH.
    #[arg(long)]
    file: Option<PathBuf>,

    /// Write reconstructed bytes to PATH.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Walk PATH as a directory tree and preserve relative paths.
    #[arg(long, conflicts_with = "file")]
    recursive: Option<PathBuf>,

    /// After recursive reconstruction, git clone the reconstructed directory to PATH.
    ///
    /// If --recursive is omitted, --clone implies --recursive . and
    /// --out example.reconstructed.dir.
    #[arg(long)]
    clone: Option<PathBuf>,

    /// Limit recursive directory walking to N levels.
    #[arg(long)]
    depth: Option<usize>,

    /// Enable verbose logging (optional level, defaults to "info").
    #[arg(long, num_args = 0..=1, default_missing_value = "info")]
    logging: Option<String>,
}

fn clone_and_verify_reconstructed_repo(
    reconstructed: &Path,
    clone_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let status = Command::new("git")
        .arg("clone")
        .arg(reconstructed)
        .arg(clone_path)
        .status()?;
    if !status.success() {
        return Err(format!(
            "git clone from {} to {} failed",
            reconstructed.display(),
            clone_path.display()
        )
        .into());
    }

    let reconstructed_head = git_output(reconstructed, &["rev-parse", "HEAD"]);
    let clone_head = git_output(clone_path, &["rev-parse", "HEAD"]);
    if reconstructed_head != clone_head {
        return Err(format!(
            "clone HEAD mismatch: reconstructed={reconstructed_head}, clone={clone_head}"
        )
        .into());
    }

    let status_output = git_output(clone_path, &["status", "--porcelain"]);
    if !status_output.is_empty() {
        return Err(format!("cloned working tree is not clean:\n{status_output}").into());
    }

    println!(
        "verified git clone: {} -> {} (HEAD {})",
        reconstructed.display(),
        clone_path.display(),
        clone_head
    );
    Ok(())
}

fn git_output(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        output.status.success(),
        "git -C {} {} failed",
        repo.display(),
        args.join(" ")
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    if let Some(ref level) = cli.logging {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();
    }

    let depth = cli.depth.unwrap_or(3);
    let depth_set = cli.depth.is_some();

    let mut recursive = cli.recursive;
    let mut out = cli.out;
    if cli.clone.is_some() && recursive.is_none() {
        recursive = Some(PathBuf::from("."));
        if out.is_none() {
            out = Some(PathBuf::from("example.reconstructed.dir"));
        }
    }

    if depth_set && recursive.is_none() {
        return Err("--depth is only valid with --recursive".into());
    }

    if let Some(ref root) = recursive {
        if !root.is_dir() {
            return Err(format!("--recursive path is not a directory: {}", root.display()).into());
        }

        info!("recursive walk root: {}", root.display());
        info!("recursive depth: {}", depth);
        info!("logging flag is: {:?}", cli.logging);
        let walk = run_directory_walk(root, depth, cli.logging.is_some())?;
        if let Some(ref out_path) = out {
            reconstruct_directory(walk, out_path)?;
            println!(
                "recursively reconstructed directory to {}",
                out_path.display()
            );
            if let Some(ref clone_path) = cli.clone {
                clone_and_verify_reconstructed_repo(out_path, clone_path)?;
            }
            return Ok(());
        }
        return Ok(());
    }

    let example_path = cli.file.unwrap_or_else(|| PathBuf::from("example.bin"));

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
    if cli.logging.is_some() {
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
