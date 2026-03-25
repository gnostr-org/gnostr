use std::{
    fs,
    io::Write,
    net::{TcpStream, TcpListener},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::Parser;
use directories::ProjectDirs;
/// Initializes the logger for the application.
/// Sets up logging to file and console output.
use gnostr_bitcoin::ui::{App, init_tui, restore_tui};
use gnostr_bitcoin::{
    ActivePeerState, DEFAULT_PORT, DNS_SEEDS, build_mempool_message, build_ping_message,
    build_pong_message, connect_and_handshake, init_logger, read_message, send_raw_tx,
    build_getheaders_message, VarIntReader, build_getdata_message, accept_and_handshake, build_feefilter_message, build_sendcmpct_message,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use sha2::{Digest, Sha256};

/// Maximum number of concurrent peer connections allowed.
pub const MAX_PEERS: usize = 8;
/// Filename for storing peer information persistently.
const PEERS_FILE_NAME: &str = "peers.json";

const GENESIS_HASH: [u8; 32] = [
    0x6f, 0xe2, 0x8c, 0x0a, 0xb6, 0xf1, 0xb3, 0x72,
    0xc1, 0xa6, 0xa2, 0x46, 0xae, 0x63, 0xf7, 0x4f,
    0x93, 0x1e, 0x83, 0x65, 0xe1, 0x5a, 0x08, 0x9c,
    0x68, 0xd6, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional: Connect to a specific Bitcoin peer address (e.g., "127.0.0.1:8333")
    #[arg(short, long)]
    target_peer_addr: Option<String>,

    /// Optional: Maximum number of concurrent peer connections to maintain
    #[arg(short, long, default_value_t = MAX_PEERS)]
    max_peers: usize,

    /// Optional: Enable sending raw transactions.
    #[arg(long)]
    sendrawtx: bool,

    /// Optional: Hex-encoded raw transaction to blast.
    #[arg(long)]
    tx: Option<String>,

    /// Optional: Specify the data directory
    #[arg(short, long)]
    datadir: Option<PathBuf>,

    /// Optional: Listen for incoming connections
    #[arg(short, long)]
    listen: bool,
}

/// Represents information about a connected peer.
/// Stores address, current session traffic, and total accumulated traffic.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PeerInfo {
    /// The network address of the peer.
    addr: String,
    /// Inbound traffic for the current session (in bytes).
    inbound_traffic: u64,
    /// Outbound traffic for the current session (in bytes).
    outbound_traffic: u64,
    /// Total accumulated inbound traffic since the application started (in
    /// bytes).
    total_inbound_traffic: u64,
    /// Total accumulated outbound traffic since the application started (in
    /// bytes).
    total_outbound_traffic: u64,
}

/// Determines and returns the application data directory path.
/// Ensures that the directory exists, creating it if necessary.
fn get_app_data_dir(custom_path: Option<PathBuf>) -> Result<PathBuf> {
    let path = if let Some(p) = custom_path {
        p
    } else if let Some(proj_dirs) = ProjectDirs::from("org", "gnostr", "gnostr") {
        let mut p = proj_dirs.data_dir().to_path_buf();
        p.push("bitcoin");
        p
    } else {
        let mut p = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine application data directory"))?;
        p.push("gnostr");
        p.push("bitcoin");
        p
    };
    fs::create_dir_all(&path)?;
    Ok(path)
}

/// Saves the current list of known peers (with their total traffic) to a JSON
/// file. This allows for persistent storage of peer data across application
/// runs.
fn save_peers(peers: &std::collections::HashMap<String, (u64, u64)>, data_dir: &std::path::Path) -> Result<()> {
    let peers_file_path = data_dir.join(PEERS_FILE_NAME);

    // Map the HashMap to a Vec<PeerInfo> for serialization.
    let serializable_peers: Vec<PeerInfo> = peers
        .iter()
        .map(|(addr, (total_inbound, total_outbound))| {
            PeerInfo {
                addr: addr.clone(),
                inbound_traffic: 0, // Session traffic is not saved, only total traffic.
                outbound_traffic: 0,
                total_inbound_traffic: *total_inbound,
                total_outbound_traffic: *total_outbound,
            }
        })
        .collect();

    let json = serde_json::to_string_pretty(&serializable_peers)?;
    fs::write(peers_file_path, json)?;
    info!("Saved {} peers.", peers.len());
    Ok(())
}

/// Loads the list of known peers from a JSON file.
/// If the file does not exist, it returns an empty HashMap.
fn load_peers(data_dir: &std::path::Path) -> Result<std::collections::HashMap<String, (u64, u64)>> {
    let peers_file_path = data_dir.join(PEERS_FILE_NAME);

    if !peers_file_path.exists() {
        info!(
            "No peers file found at {:?}. Starting with empty peer list.",
            peers_file_path
        );
        return Ok(std::collections::HashMap::new());
    }

    let json = fs::read_to_string(peers_file_path)?;
    let serializable_peers: Vec<PeerInfo> = serde_json::from_str(&json)?;
    // Convert the Vec<PeerInfo> back into a HashMap for efficient lookup.
    let peers_map: std::collections::HashMap<String, (u64, u64)> = serializable_peers
        .into_iter()
        .map(|p| (p.addr, (p.total_inbound_traffic, p.total_outbound_traffic)))
        .collect();
    info!("Loaded {} peers.", peers_map.len());
    Ok(peers_map)
}

fn spawn_peer_handler(
    mut stream: TcpStream,
    peer_addr: String,
    messages: Arc<Mutex<Vec<(String, SystemTime)>>>,
    running: Arc<AtomicBool>,
    active_peers: Arc<Mutex<std::collections::HashMap<String, ActivePeerState>>>,
    known_peers: Arc<Mutex<std::collections::HashMap<String, (u64, u64)>>>,
    _block_hash: Arc<Mutex<String>>,
    _local_height: Arc<Mutex<i32>>,
    _data_dir: PathBuf,
) {
    std::thread::spawn(move || {
        let add_message_for_peer = |msg: String| {
            messages.lock().unwrap().push((
                format!("[{}] {}", peer_addr, msg),
                SystemTime::now(),
            ));
        };

        let (mut session_inbound_traffic, mut session_outbound_traffic) = (0, 0);
        let mut last_traffic_update = Instant::now();

        add_message_for_peer("Entering listener message processing loop...".to_string());

        // Send initial feefilter message
        match build_feefilter_message(10) { // Default feerate 10 sat/kB
            Ok(msg) => {
                if stream.write_all(&msg).is_ok() {
                    session_outbound_traffic += msg.len() as u64;
                    add_message_for_peer("Sent 'feefilter' message (10 sat/kB).".to_string());
                }
            }
            Err(e) => add_message_for_peer(format!("[ERROR] Error building feefilter msg: {}", e)),
        }

        // Send initial sendcmpct message
        match build_sendcmpct_message(1, 2) { // High-bandwidth mode, version 2 (BIP152)
            Ok(msg) => {
                if stream.write_all(&msg).is_ok() {
                    session_outbound_traffic += msg.len() as u64;
                    add_message_for_peer("Sent 'sendcmpct' message (high-bandwidth, version 2).".to_string());
                }
            }
            Err(e) => add_message_for_peer(format!("[ERROR] Error building sendcmpct msg: {}", e)),
        }

        loop {
            if !running.load(Ordering::SeqCst) { break; }

            if last_traffic_update.elapsed() >= Duration::from_secs(1) {
                if let Some(state) = active_peers.lock().unwrap().get_mut(&peer_addr) {
                    state.inbound_traffic = session_inbound_traffic;
                    state.outbound_traffic = session_outbound_traffic;
                }
                last_traffic_update = Instant::now();
            }

            stream.set_read_timeout(Some(Duration::from_secs(60))).ok();

            match read_message(&mut stream) {
                Ok((header, payload)) => {
                    session_inbound_traffic += (header.len() + payload.len()) as u64;
                    if let Ok(command) = std::str::from_utf8(&header[4..16]) {
                        let command = command.trim_matches(|c: char| c == '\0' || c == ' ');
                        match command {
                            "ping" => {
                                if let Ok(nonce) = payload.try_into() {
                                    if let Ok(pong) = build_pong_message(nonce) {
                                        if stream.write_all(&pong).is_ok() {
                                            session_outbound_traffic += pong.len() as u64;
                                        }
                                    }
                                }
                            }
                            "feefilter" => {
                                if payload.len() >= 8 {
                                    let feerate_bytes: [u8; 8] = payload[0..8].try_into().unwrap();
                                    let feerate = u64::from_le_bytes(feerate_bytes);
                                    add_message_for_peer(format!("[INFO] Received 'feefilter' message: {} sat/kB.", feerate));
                                    // Update peer's fee filter in active_peers
                                    if let Some(state) = active_peers.lock().unwrap().get_mut(&peer_addr) {
                                        state.fee_filter = feerate;
                                        add_message_for_peer(format!("[DEBUG] Updated {} fee filter to {} sat/kB.", peer_addr, feerate));
                                    }
                                } else {
                                    add_message_for_peer("[ERROR] Received malformed 'feefilter' message.".to_string());
                                }
                            }
                            "sendcmpct" => {
                                if payload.len() >= 9 {
                                    let high_bandwidth_mode = payload[0];
                                    let version_bytes: [u8; 8] = payload[1..9].try_into().unwrap();
                                    let version = u64::from_le_bytes(version_bytes);
                                    add_message_for_peer(format!("[INFO] Received 'sendcmpct' message: high_bandwidth_mode={}, version={}.", high_bandwidth_mode, version));
                                } else {
                                    add_message_for_peer("[ERROR] Received malformed 'sendcmpct' message.".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(_) => break,
            }
        }
        
        active_peers.lock().unwrap().remove(&peer_addr);
        let mut kp = known_peers.lock().unwrap();
        kp.entry(peer_addr).and_modify(|(i, o)| { *i += session_inbound_traffic; *o += session_outbound_traffic; }).or_insert((session_inbound_traffic, session_outbound_traffic));
    });
}

/// The main function that orchestrates the Bitcoin P2P client.
/// Initializes logging, TUI, and starts network and UI threads.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let max_peers = cli.max_peers;
    let target_peer_addr = cli.target_peer_addr;
    let send_raw_tx_enabled = cli.sendrawtx;
    let tx_hex_string = cli.tx;
    let custom_datadir = cli.datadir;
    let listen_enabled = cli.listen;

    let data_dir = get_app_data_dir(custom_datadir)?;
    
    init_logger(Some(data_dir.clone()))?;

    debug!("Send raw transaction enabled: {}", send_raw_tx_enabled);
    debug!("Listen enabled: {}", listen_enabled);
    debug!("Data directory: {:?}", data_dir);

    if send_raw_tx_enabled {
        if let Some(tx_hex) = tx_hex_string {
            info!("Attempting to send raw transaction: {}", tx_hex);
            match send_raw_tx::send_raw_transaction_to_peers(tx_hex).await {
                Ok(_) => println!("Raw transaction sent successfully."),
                Err(e) => error!("Failed to send raw transaction: {}", e),
            }
            return Ok(()); // Exit after sending transaction
        } else {
            error!("The --sendrawtx flag was provided, but no --tx was specified.");
            return Err(anyhow::anyhow!("Missing --tx argument for --sendrawtx"));
        }
    }

    // Shared state for managing application lifecycle and data across threads.
    let running = Arc::new(AtomicBool::new(true)); // Flag to signal shutdown.
    let messages = Arc::new(Mutex::new(Vec::<(String, SystemTime)>::new())); // Log messages buffer.
    let block_height = Arc::new(Mutex::new(0)); // Current block height.
    let block_hash = Arc::new(Mutex::new("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f".to_string())); // Current block hash.
    let local_height = Arc::new(Mutex::new(0)); // Our synced block height.
    let known_peers = Arc::new(Mutex::new(load_peers(&data_dir).unwrap_or_else(|e| {
        error!("Failed to load known peers on startup: {}", e);
        std::collections::HashMap::new()
    }))); // Cache of known peers and their traffic.
    let active_peers = Arc::new(Mutex::new(std::collections::HashMap::<
        String,
        ActivePeerState,
    >::new())); // Currently active connections.
    let discovered_peers_queue = Arc::new(Mutex::new(Vec::<String>::new())); // Queue for discovered peers to connect to.

    // Populate discovery queue with initially known peers.
    let mut discovered_peers_queue_lock = discovered_peers_queue.lock().unwrap();
    let mut known_peers_lock = known_peers.lock().unwrap();

    // Load Gnostr relays from file and add to queue
    let relays_file_path = data_dir.join("relays.json");
    if relays_file_path.exists() {
        match fs::read_to_string(&relays_file_path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<String>>(&content) {
                    Ok(relays) => {
                        for peer_addr in relays {
                            if !discovered_peers_queue_lock.contains(&peer_addr) {
                                discovered_peers_queue_lock.push(peer_addr.clone());
                                info!("[INFO] Added Gnostr relay from file to discovery queue: {}", peer_addr);
                            }
                            known_peers_lock.entry(peer_addr).or_insert((0, 0)); // Also add to known peers
                        }
                    }
                    Err(e) => error!("[ERROR] Failed to parse relays.json: {}", e),
                }
            }
            Err(e) => error!("[ERROR] Failed to read relays.json: {}", e),
        }
    }

    for (addr, _) in known_peers_lock.iter() {
        if !discovered_peers_queue_lock.contains(addr) {
            discovered_peers_queue_lock.push(addr.clone());
        }
    }
    info!(
        "Added {} known peers to discovery queue.",
        discovered_peers_queue_lock.len()
    );
    drop(discovered_peers_queue_lock); // Release the lock.
    drop(known_peers_lock); // Release lock.

    // --- Initial DNS Seed Discovery ---
    // Spawn threads to connect to DNS seeds and gather initial peer information.
    let (tx_initial_peers, rx_initial_peers) = std::sync::mpsc::channel();
    let mut handles = Vec::new();

    // If a target_peer_addr is provided, only try to connect to that peer initially.
    let initial_addresses_to_try = if let Some(target) = target_peer_addr.clone() {
        vec![target]
    } else {
        DNS_SEEDS.iter().map(|&s| s.to_string()).collect()
    };

    for seed_addr in initial_addresses_to_try.iter() {
        let seed_addr = seed_addr.to_string();
        let tx_clone = tx_initial_peers.clone();
        let block_height_clone = Arc::clone(&block_height);
        let running_clone = Arc::clone(&running);
        let messages_clone = Arc::clone(&messages);
        let local_height_val = *local_height.lock().unwrap();

        let handle = std::thread::spawn(move || {
            let add_message = |msg: String| {
                messages_clone
                    .lock()
                    .unwrap()
                    .push((msg, SystemTime::now()));
            };
            add_message(format!(
                "Attempting initial connection to DNS seed: {}",
                seed_addr
            ));
            // Attempt to connect and handshake with the DNS seed.
            let conn_result: Result<(TcpStream, String, Vec<String>, i32, String, u64), anyhow::Error> =
                connect_and_handshake(
                    DNS_SEEDS, // Pass DNS_SEEDS for potential peer discovery during handshake
                    DEFAULT_PORT,
                    block_height_clone,
                    running_clone,
                    Some(seed_addr.clone()), // Target this specific seed
                    local_height_val,
                );
            if let Ok((_, _, new_peers, _, _, _)) = conn_result {
                add_message(format!(
                    "Discovered {} new peers from {}.",
                    new_peers.len(),
                    seed_addr
                ));
                let _ = tx_clone.send(new_peers); // Send discovered peers back to main thread.
            } else if let Err(e) = conn_result {
                add_message(format!(
                    "[ERROR] Initial connection to {} failed: {}",
                    seed_addr, e
                ));
            }
        });
        handles.push(handle);
    }

    // Drop the original sender to signal the receiver that no more messages will be
    // sent.
    drop(tx_initial_peers);

    // Collect results from all initial peer discovery threads.
    let mut initial_discovered_peers: Vec<String> = Vec::new();
    for new_peers_from_seed in rx_initial_peers.iter() {
        initial_discovered_peers.extend(new_peers_from_seed);
    }
    info!(
        "Collected {} initial peers from DNS seeds.",
        initial_discovered_peers.len()
    );

    // Add newly discovered peers to the main discovery queue and known peers list.
    let mut discovered_peers_queue_lock = discovered_peers_queue.lock().unwrap();
    let mut known_peers_lock = known_peers.lock().unwrap();
    for peer in initial_discovered_peers {
        if !discovered_peers_queue_lock.contains(&peer) {
            discovered_peers_queue_lock.push(peer.clone());
        }
        known_peers_lock.entry(peer).or_insert((0, 0));
    }
    info!(
        "Total peers in discovery queue after initial DNS scan: {}.",
        discovered_peers_queue_lock.len()
    );
    drop(discovered_peers_queue_lock); // Release lock.
    drop(known_peers_lock); // Release lock.
    // --- End of initial DNS seed discovery ---

    // Setup Ctrl-C handler for graceful shutdown.
    let r_ctrlc = running.clone();
    let known_peers_for_shutdown_ctrlc = Arc::clone(&known_peers);
    let data_dir_for_shutdown = data_dir.clone();

    ctrlc::set_handler(move || {
        info!("Ctrl-C received. Initiating shutdown...");
        r_ctrlc.store(false, Ordering::SeqCst); // Signal running flag to false.
        // Attempt to save peer data before exiting.
        if let Err(e) = save_peers(&known_peers_for_shutdown_ctrlc.lock().unwrap(), &data_dir_for_shutdown) {
            error!("Failed to save peers on shutdown: {}", e);
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Clone shared state for the network thread.
    let messages_network = Arc::clone(&messages);
    let running_network = Arc::clone(&running);
    let block_height_network = Arc::clone(&block_height);
    let block_hash_network = Arc::clone(&block_hash);
    let local_height_network = Arc::clone(&local_height);
    let active_peers_network = Arc::clone(&active_peers);
    let known_peers_network = Arc::clone(&known_peers);
    let discovered_peers_queue_network = Arc::clone(&discovered_peers_queue);
    let data_dir_network = data_dir.clone();

    if listen_enabled {
        let active_peers_listener = Arc::clone(&active_peers);
        let known_peers_listener = Arc::clone(&known_peers);
        let messages_listener = Arc::clone(&messages);
        let running_listener = Arc::clone(&running);
        let block_hash_listener = Arc::clone(&block_hash);
        let local_height_listener = Arc::clone(&local_height);
        let data_dir_listener = data_dir.clone();
        
        std::thread::spawn(move || {
            let listener = match TcpListener::bind("0.0.0.0:8333") {
                Ok(l) => l,
                Err(e) => {
                    messages_listener.lock().unwrap().push((format!("[ERROR] Failed to bind to port 8333: {}", e), SystemTime::now()));
                    return;
                }
            };
            listener.set_nonblocking(true).ok();
            messages_listener.lock().unwrap().push(("[INFO] Listening on 0.0.0.0:8333".to_string(), SystemTime::now()));
            
            loop {
                if !running_listener.load(Ordering::SeqCst) { break; }
                
                match listener.accept() {
                    Ok((stream, _addr)) => {
                        let local_height_val = *local_height_listener.lock().unwrap();
                        match accept_and_handshake(stream, local_height_val) {
                            Ok((stream, peer_addr, _ver, ua, _fee_filter)) => {
                                messages_listener.lock().unwrap().push((format!("[DEBUG] Peer connected: {} UA: '{}'", peer_addr, ua), SystemTime::now()));
                                active_peers_listener.lock().unwrap().insert(peer_addr.clone(), ActivePeerState {
                                    inbound_traffic: 0,
                                    outbound_traffic: 0,
                                    connection_time: SystemTime::now(),
                                    protocol_version: _ver,
                                    user_agent: ua.clone(),
                                    fee_filter: _fee_filter,
                                });
                                
                                if ua.contains("Gnostr") {
                                    messages_listener.lock().unwrap().push((format!("[INFO] Gnostr peer detected! UA: {}", ua), SystemTime::now()));
                                    let relays_file_path = data_dir_listener.join("relays.json");
                                    messages_listener.lock().unwrap().push((format!("[DEBUG] Saving peer address to: {:?}", relays_file_path), SystemTime::now()));
                                    let mut relays: Vec<String> = if relays_file_path.exists() {
                                        match fs::read_to_string(&relays_file_path) {
                                            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                                            Err(_) => Vec::new(),
                                        }
                                    } else { Vec::new() };

                                    if !relays.contains(&peer_addr) {
                                        relays.push(peer_addr.clone());
                                        if let Ok(json) = serde_json::to_string_pretty(&relays) {
                                            if let Err(e) = fs::write(&relays_file_path, json) {
                                                messages_listener.lock().unwrap().push((format!("[ERROR] Failed to write relays.json: {}", e), SystemTime::now()));
                                            } else {
                                                messages_listener.lock().unwrap().push((format!("[INFO] Saved Gnostr peer {} to relays.json", peer_addr), SystemTime::now()));
                                            }
                                        }
                                    }
                                }
                                
                                spawn_peer_handler(
                                    stream,
                                    peer_addr,
                                    Arc::clone(&messages_listener),
                                    Arc::clone(&running_listener),
                                    Arc::clone(&active_peers_listener),
                                    Arc::clone(&known_peers_listener),
                                    Arc::clone(&block_hash_listener),
                                    Arc::clone(&local_height_listener),
                                    data_dir_listener.clone(),
                                );
                            }
                            Err(e) => {
                                messages_listener.lock().unwrap().push((format!("[ERROR] Handshake failed with incoming peer: {}", e), SystemTime::now()));
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(e) => {
                         messages_listener.lock().unwrap().push((format!("[ERROR] Listener accept failed: {}", e), SystemTime::now()));
                    }
                }
            }
        });
    }

    // Spawn the network thread.
    // This thread manages all P2P connections, message handling, and peer
    // discovery.
    let _network_thread_handle = std::thread::spawn(move || {
        let add_message = |msg: String| {
            messages_network
                .lock()
                .unwrap()
                .push((msg, SystemTime::now()));
        };

        add_message("Starting Bitcoin P2P client...".to_string());

        // Main loop for managing network connections and activity.
        loop {
            // Check if shutdown has been initiated.
            if !running_network.load(Ordering::SeqCst) {
                add_message("Network thread received shutdown signal.".to_string());
                // Update known_peers with traffic from active connections before exiting.
                let mut known_peers_lock = known_peers_network.lock().unwrap();
                let active_peers_lock = active_peers_network.lock().unwrap();
                for (addr, state) in active_peers_lock.iter()
                {
                    known_peers_lock
                        .entry(addr.clone())
                        .and_modify(|(total_in, total_out)| {
                            *total_in += state.inbound_traffic;
                            *total_out += state.outbound_traffic;
                        })
                        .or_insert((state.inbound_traffic, state.outbound_traffic));
                }
                break; // Exit the network loop.
            }

            // Limit the number of active peer connections.
            let num_connected_peers = active_peers_network.lock().unwrap().len();
            if num_connected_peers >= max_peers {
                std::thread::sleep(Duration::from_secs(5)); // Wait before checking again if max peers reached.
                continue;
            }

            // Get a peer address from the discovery queue to attempt connection.
            let mut target_peer_addr_for_conn_attempt: Option<String> = None;

            #[cfg(debug_assertions)]
            {
                let active_peers_lock = active_peers_network.lock().unwrap();
                // If we aren't connected to localhost:8333, force it as the next target.
                // We avoid connecting to ourselves by checking if we are Node 1 (using the default port).
                // Note: This is a simple check; in a production node, we'd check our own advertised address/port.
                if !active_peers_lock.contains_key("127.0.0.1:8333") {
                    target_peer_addr_for_conn_attempt = Some("127.0.0.1:8333".to_string());
                }
                drop(active_peers_lock);
            }

            if target_peer_addr_for_conn_attempt.is_none() {
                if let Some(peer) = discovered_peers_queue_network.lock().unwrap().pop() {
                    target_peer_addr_for_conn_attempt = Some(peer);
                }
            }

            if let Some(target_addr) = &target_peer_addr_for_conn_attempt {
                add_message(format!(
                    "Attempting to connect and handshake ({} / {} peers) to {}...",
                    num_connected_peers, max_peers, target_addr
                ));
            } else {
                continue; // No peers to connect to
            }
            // Channel for receiving results from the connection attempt thread.
            let (tx_conn, rx_conn) = std::sync::mpsc::channel();
            let block_height_clone_for_conn = Arc::clone(&block_height_network);
            let running_network_clone_for_conn = Arc::clone(&running_network);
            let active_peers_clone_for_conn = Arc::clone(&active_peers_network);
            let known_peers_clone_for_conn = Arc::clone(&known_peers_network);
            let messages_clone_for_logging = Arc::clone(&messages_network);
            let discovered_peers_queue_for_conn = Arc::clone(&discovered_peers_queue_network);
            let target_peer_addr_for_thread = target_peer_addr_for_conn_attempt.clone();
            let local_height_val = *local_height_network.lock().unwrap();

            // Spawn a thread for each connection attempt.
            std::thread::spawn(move || {
                // Attempt to connect and handshake with a peer.
                let conn_result: Result<(TcpStream, String, Vec<String>, i32, String, u64), anyhow::Error> =
                    connect_and_handshake(
                        DNS_SEEDS, /* DNS seeds used for initial discovery and potentially
                                    * during handshake. */
                        DEFAULT_PORT,
                        block_height_clone_for_conn,
                        running_network_clone_for_conn,
                        target_peer_addr_for_thread,
                        local_height_val,
                    );
                // Process the connection result.
                if let Ok((_, peer_addr, new_peers, version, ua, _fee_filter)) = &conn_result {
                    let mut active_peers_lock = active_peers_clone_for_conn.lock().unwrap();
                    // Add the successfully connected peer to the active peers list.
                    active_peers_lock.insert(peer_addr.clone(), ActivePeerState {
                        inbound_traffic: 0,
                        outbound_traffic: 0,
                        connection_time: SystemTime::now(),
                        protocol_version: *version,
                        user_agent: ua.clone(),
                        fee_filter: *_fee_filter,
                    }); // Initialize session traffic to 0 and set connection time.

                    // Ensure the peer is also in the known_peers cache.
                    let mut known_peers_lock = known_peers_clone_for_conn.lock().unwrap();
                    if !known_peers_lock.contains_key(peer_addr) {
                        known_peers_lock.insert(peer_addr.clone(), (0, 0)); // Add new peer with zero traffic.
                    }

                    messages_clone_for_logging
                        .lock()
                        .unwrap()
                        .push((format!("Connected to: {}", peer_addr), SystemTime::now()));
                    // Add newly discovered peers from the connected peer to the discovery queue.
                    let mut discovered_peers_queue_lock =
                        discovered_peers_queue_for_conn.lock().unwrap();
                    for new_peer in new_peers.iter() {
                        if !discovered_peers_queue_lock.contains(new_peer) {
                            discovered_peers_queue_lock.push(new_peer.clone());
                        }
                        if !known_peers_lock.contains_key(new_peer) {
                            known_peers_lock.insert(new_peer.clone(), (0, 0)); // Add new peer to known list.
                        }
                    }
                }
                let _ = tx_conn.send(conn_result); // Send the result back to the main network loop.
            });

            // Receive the connection result with a timeout.
            let stream_result: Result<(TcpStream, String, Vec<String>, i32, String, u64), anyhow::Error> =
                match rx_conn.recv_timeout(Duration::from_secs(10)) {
                    Ok(Ok((stream, peer_addr, new_peers, version, ua, fee_filter))) => Ok((stream, peer_addr, new_peers, version, ua, fee_filter)),
                    Ok(Err(e)) => {
                        add_message(format!(
                            "[ERROR] Failed to connect and handshake: {}. Trying next peer...",
                            e
                        ));
                        // If connection failed, re-add the peer to the queue for a potential retry
                        // later.
                        if let Some(failed_peer) = target_peer_addr_for_conn_attempt {
                            let mut discovered_peers_queue_lock =
                                discovered_peers_queue_network.lock().unwrap();
                            if !discovered_peers_queue_lock.contains(&failed_peer) {
                                discovered_peers_queue_lock.push(failed_peer.clone());
                                add_message(format!(
                                    "[INFO] Re-added {} to discovery queue for retry.",
                                    failed_peer
                                ));
                            }
                        }
                        Err(e)
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        add_message("[WARN] Connection and handshake timed out after 10 seconds. Trying next peer...".to_string());
                        // If timed out, re-add the peer to the queue for retry.
                        if let Some(timed_out_peer) = target_peer_addr_for_conn_attempt {
                            let mut discovered_peers_queue_lock =
                                discovered_peers_queue_network.lock().unwrap();
                            if !discovered_peers_queue_lock.contains(&timed_out_peer) {
                                discovered_peers_queue_lock.push(timed_out_peer.clone());
                                add_message(format!(
                                    "[INFO] Re-added {} to discovery queue for retry (timeout).",
                                    timed_out_peer
                                ));
                            }
                        }
                        Err(anyhow::anyhow!("Connection timeout"))
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        add_message("[ERROR] Connection thread disconnected before sending result. Trying next peer...".to_string());
                        Err(anyhow::anyhow!("Connection thread disconnected"))
                    }
                };

            // If a connection was successfully established and handshake completed:
            if let Ok((mut stream, connected_peer_addr, _, _, ua, _fee_filter)) = stream_result {
                add_message(format!("[DEBUG] Peer connected: {} UA: '{}'", connected_peer_addr, ua));
                // Check if UA contains "Gnostr" and save to relays.json
                if ua.contains("Gnostr") {
                    add_message(format!("[INFO] Gnostr peer detected! UA: {}", ua));
                    let relays_file_path = data_dir_network.join("relays.json");
                    add_message(format!("[DEBUG] Saving peer address to: {:?}", relays_file_path));
                    
                    let mut relays: Vec<String> = if relays_file_path.exists() {
                        match fs::read_to_string(&relays_file_path) {
                            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                            Err(_) => Vec::new(),
                        }
                    } else {
                        Vec::new()
                    };

                    if !relays.contains(&connected_peer_addr) {
                        relays.push(connected_peer_addr.clone());
                        if let Ok(json) = serde_json::to_string_pretty(&relays) {
                            if let Err(e) = fs::write(&relays_file_path, json) {
                                add_message(format!("[ERROR] Failed to write to relays.json: {}", e));
                            } else {
                                add_message(format!("[INFO] Saved Gnostr peer {} to relays.json", connected_peer_addr));
                            }
                        }
                    }
                }

                // Clone shared state for the peer-specific thread.
                let active_peers_clone_for_peer_thread = Arc::clone(&active_peers_network);
                let known_peers_clone_for_peer_thread = Arc::clone(&known_peers_network);
                let messages_clone_for_peer_thread = Arc::clone(&messages_network);
                let running_network_clone_for_peer_thread = Arc::clone(&running_network);
                let block_hash_clone_for_peer_thread = Arc::clone(&block_hash_network);
                let local_height_clone_for_peer_thread = Arc::clone(&local_height_network);
                let data_dir_peer = data_dir_network.clone();
                let peer_addr_for_peer_thread = connected_peer_addr.clone();
                let discovered_peers_queue_peer = Arc::clone(&discovered_peers_queue_network);

                // Spawn a thread to handle communication with this specific peer.
                std::thread::spawn(move || {
                    let add_message_for_peer = |msg: String| {
                        messages_clone_for_peer_thread.lock().unwrap().push((
                            format!("[{}] {}", peer_addr_for_peer_thread, msg),
                            SystemTime::now(),
                        ));
                    };

                    let (mut session_inbound_traffic, mut session_outbound_traffic) = (0, 0);
                    let mut last_traffic_update = Instant::now();

                    add_message_for_peer("Entering message processing loop...".to_string());

                    // Send an initial 'mempool' request to the peer.
                    add_message_for_peer("Requesting mempool information...".to_string());
                    match build_mempool_message() {
                        Ok(mempool_message) => {
                            if let Err(e) = stream.write_all(&mempool_message) {
                                add_message_for_peer(format!(
                                    "[ERROR] Failed to send mempool request: {}",
                                    e
                                ));
                            } else {
                                session_outbound_traffic += mempool_message.len() as u64;
                                add_message_for_peer("Sent 'mempool' request.".to_string());
                            }
                        }
                        Err(e) => {
                            add_message_for_peer(format!(
                                "[ERROR] Failed to build mempool message: {}",
                                e
                            ));
                        }
                    }

                    // Send 'getheaders' to request headers starting from genesis.
                    add_message_for_peer("Requesting block headers...".to_string());
                    match build_getheaders_message(vec![GENESIS_HASH], [0u8; 32]) {
                        Ok(getheaders_msg) => {
                             if let Err(e) = stream.write_all(&getheaders_msg) {
                                add_message_for_peer(format!(
                                    "[ERROR] Failed to send getheaders request: {}",
                                    e
                                ));
                            } else {
                                session_outbound_traffic += getheaders_msg.len() as u64;
                                add_message_for_peer("Sent 'getheaders' request.".to_string());
                            }
                        }
                        Err(e) => {
                            add_message_for_peer(format!(
                                "[ERROR] Failed to build getheaders message: {}",
                                e
                            ));
                        }
                    }

                    // Loop to continuously read messages from the peer.
                    loop {
                        // Check for shutdown signal.
                        if !running_network_clone_for_peer_thread.load(Ordering::SeqCst) {
                            add_message_for_peer(
                                "Peer thread received shutdown signal.".to_string(),
                            );
                            break; // Exit the peer communication loop.
                        }

                        // Periodically update session traffic statistics in the shared state.
                        if last_traffic_update.elapsed() >= Duration::from_secs(1) {
                            let mut active_peers_lock =
                                active_peers_clone_for_peer_thread.lock().unwrap();
                            if let Some(state) =
                                active_peers_lock.get_mut(&peer_addr_for_peer_thread)
                            {
                                state.inbound_traffic = session_inbound_traffic; // Update session inbound traffic.
                                state.outbound_traffic = session_outbound_traffic; // Update session outbound traffic.
                                // Peer connection time (peer_entry.2) remains
                                // unchanged.
                            }
                            last_traffic_update = Instant::now();
                        }
                        // Set a read timeout to detect idle connections and send pings.
                        if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(60))) {
                            add_message_for_peer(format!(
                                "[ERROR] Failed to set read timeout: {}",
                                e
                            ));
                            break; // Exit loop on read timeout error.
                        }

                        // Read a message from the peer.
                        match read_message(&mut stream) {
                            Ok((header, payload)) => {
                                // Accumulate traffic statistics.
                                session_inbound_traffic += (header.len() + payload.len()) as u64;
                                // Parse the command from the header.
                                let command_result = std::str::from_utf8(&header[4..16]);
                                match command_result {
                                    Ok(command) => {
                                        let command = command.trim_matches(|c: char| c == '\0' || c == ' ');
                                        let log_msg = format!(
                                            "[RECEIVED] Command: '{}', Payload Size: {} bytes",
                                            command,
                                            payload.len()
                                        );
                                        add_message_for_peer(log_msg);

                                        // Handle different P2P commands.
                                        match command {
                                            "version" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'version' message again."
                                                        .to_string(),
                                                );
                                            }
                                            "verack" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'verack' message.".to_string(),
                                                );
                                            }
                                            "ping" => {
                                                add_message_for_peer("[INFO] Received 'ping' message. Sending 'pong'.".to_string());
                                                // A ping message contains an 8-byte nonce.
                                                if let Ok(nonce) = payload
                                                    .try_into()
                                                    .map_err(|_| "Invalid ping nonce size")
                                                {
                                                    match build_pong_message(nonce) {
                                                        Ok(pong_message) => {
                                                            // Send the pong message back to the
                                                            // peer.
                                                            if let Err(e) =
                                                                stream.write_all(&pong_message)
                                                            {
                                                                add_message_for_peer(format!(
                                                                    "[ERROR] Failed to send pong: {}",
                                                                    e
                                                                ));
                                                            } else {
                                                                session_outbound_traffic +=
                                                                    pong_message.len() as u64;
                                                                add_message_for_peer(
                                                                    "[SENT] 'pong' message."
                                                                        .to_string(),
                                                                );
                                                            }
                                                        }
                                                        Err(e) => add_message_for_peer(format!(
                                                            "[ERROR] Failed to build pong message: {}",
                                                            e
                                                        )),
                                                    }
                                                } else {
                                                    add_message_for_peer(
                                                        "[ERROR] Invalid ping nonce size."
                                                            .to_string(),
                                                    );
                                                }
                                            }
                                            "pong" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'pong' message.".to_string(),
                                                );
                                            }
                                            "mempool" => {
                                                add_message_for_peer("[INFO] Received 'mempool' response (or another mempool request).".to_string());
                                            }
                                            "inv" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'inv' message (inventory)."
                                                        .to_string(),
                                                );
                                            }
                                            "tx" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'tx' message (transaction)."
                                                        .to_string(),
                                                );
                                            }
                                            "block" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'block' message.".to_string(),
                                                );
                                                // Save block to disk
                                                // Calculate hash to name the file
                                                // Block header is first 80 bytes.
                                                if payload.len() >= 80 {
                                                    let header = &payload[0..80];
                                                    let hash1 = Sha256::digest(header);
                                                    let hash2 = Sha256::digest(hash1);
                                                    let mut hash_bytes = hash2.to_vec();
                                                    hash_bytes.reverse();
                                                    let hash_str = hex::encode(hash_bytes);
                                                    
                                                    let blocks_dir = data_dir_peer.join("blocks");
                                                    if let Err(e) = fs::create_dir_all(&blocks_dir) {
                                                        add_message_for_peer(format!("[ERROR] Failed to create blocks dir: {}", e));
                                                    } else {
                                                        let file_path = blocks_dir.join(format!("block_{}.dat", hash_str));
                                                        if let Err(e) = fs::write(&file_path, &payload) {
                                                            add_message_for_peer(format!("[ERROR] Failed to write block to disk: {}", e));
                                                        } else {
                                                            add_message_for_peer(format!("[SUCCESS] Saved block to {:?}", file_path));
                                                        }
                                                    }
                                                }
                                            }
                                            "headers" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'headers' message.".to_string(),
                                                );
                                                // Parse headers to get the latest block hash
                                                let mut offset = 0;
                                                match payload.read_varint_and_advance(offset) {
                                                    Ok((count, bytes_read)) => {
                                                        offset += bytes_read;
                                                        add_message_for_peer(format!("[INFO] 'headers' message contains {} headers.", count));
                                                        
                                                        let mut last_header_hash_bytes: Option<[u8; 32]> = None;
                                                        let mut batch_hashes = Vec::new();
                                                        
                                                        for _ in 0..count {
                                                            if payload.len() < offset + 80 {
                                                                break;
                                                            }
                                                            let header_bytes = &payload[offset..offset+80];
                                                            
                                                            // Calculate Double-SHA256 hash of the header
                                                            let hash1 = Sha256::digest(header_bytes);
                                                            let hash2 = Sha256::digest(hash1);
                                                            
                                                            let mut hash_array = [0u8; 32];
                                                            hash_array.copy_from_slice(&hash2);
                                                            last_header_hash_bytes = Some(hash_array);
                                                            batch_hashes.push(hash_array);
                                                            
                                                            offset += 80;
                                                            
                                                            // Read tx count (should be 0 for headers)
                                                            match payload.read_varint_and_advance(offset) {
                                                                Ok((_tx_count, bytes_read_tx)) => {
                                                                    offset += bytes_read_tx;
                                                                }
                                                                Err(_) => break,
                                                            }
                                                        }
                                                        
                                                        if let Some(hash_bytes) = last_header_hash_bytes {
                                                            let mut display_bytes = hash_bytes.to_vec();
                                                            display_bytes.reverse();
                                                            let hash_str = hex::encode(display_bytes);
                                                            
                                                            add_message_for_peer(format!("[INFO] Updating block hash to: {}", hash_str));
                                                            *block_hash_clone_for_peer_thread.lock().unwrap() = hash_str;

                                                            // Update local height
                                                            let mut height_lock = local_height_clone_for_peer_thread.lock().unwrap();
                                                            *height_lock += count as i32;
                                                            add_message_for_peer(format!("[INFO] Synced to height: {}", *height_lock));
                                                            drop(height_lock);

                                                            // If we got max headers (2000), request more
                                                            if count == 2000 {
                                                                add_message_for_peer("Received 2000 headers, requesting more...".to_string());
                                                                match build_getheaders_message(vec![hash_bytes], [0u8; 32]) {
                                                                    Ok(getheaders_msg) => {
                                                                        if let Err(e) = stream.write_all(&getheaders_msg) {
                                                                             add_message_for_peer(format!("[ERROR] Failed to send getheaders request: {}", e));
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        add_message_for_peer(format!("[ERROR] Failed to build getheaders message: {}", e));
                                                                    }
                                                                }
                                                            } else if count > 0 {
                                                                // Assume tip. Request last 10 blocks.
                                                                add_message_for_peer("Synced near tip. Requesting last 10 blocks...".to_string());
                                                                let start_idx = if batch_hashes.len() > 10 { batch_hashes.len() - 10 } else { 0 };
                                                                let hashes_to_request = &batch_hashes[start_idx..];
                                                                
                                                                let inventory: Vec<([u8; 32], u32)> = hashes_to_request.iter().map(|h| (*h, 0x40000002)).collect(); // MSG_WITNESS_BLOCK
                                                                
                                                                match build_getdata_message(inventory) {
                                                                    Ok(getdata_msg) => {
                                                                        if let Err(e) = stream.write_all(&getdata_msg) {
                                                                             add_message_for_peer(format!("[ERROR] Failed to send getdata request: {}", e));
                                                                        } else {
                                                                            session_outbound_traffic += getdata_msg.len() as u64;
                                                                            add_message_for_peer("Sent 'getdata' request for blocks.".to_string());
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        add_message_for_peer(format!("[ERROR] Failed to build getdata message: {}", e));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        add_message_for_peer(format!("[ERROR] Failed to parse headers count: {}", e));
                                                    }
                                                }
                                            }
                                            "getheaders" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'getheaders' message."
                                                        .to_string(),
                                                );
                                            }
                                            "getdata" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'getdata' message."
                                                        .to_string(),
                                                );
                                            }
                                            "addr" => {
                                                add_message_for_peer(
                                                    "[INFO] Received 'addr' message.".to_string(),
                                                );
                                            }
                                            _ => {
                                                add_message_for_peer(format!(
                                                    "[INFO] Received unhandled command: '{}'",
                                                    command
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        add_message_for_peer(format!(
                                            "[ERROR] Failed to parse command from header: {}",
                                            e
                                        ));
                                        break; // Exit loop on command parsing error.
                                    }
                                }
                            }
                            Err(e) => {
                                // Handle specific IO errors, like timeouts.
                                if let Some(io_error) = e.downcast_ref::<std::io::Error>()
                                    && io_error.kind() == std::io::ErrorKind::TimedOut {
                                        // If read times out, send a 'ping' message to keep the
                                        // connection alive.
                                        add_message_for_peer("[INFO] Read timeout. No data received for 60 seconds. Sending ping...".to_string());
                                        let nonce_u64 = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs();
                                        let mut nonce_bytes = [0u8; 8];
                                        nonce_bytes.copy_from_slice(&nonce_u64.to_le_bytes());
                                        match build_ping_message(nonce_bytes) {
                                            Ok(ping_message) => {
                                                if let Err(e) = stream.write_all(&ping_message) {
                                                    add_message_for_peer(format!(
                                                        "[ERROR] Failed to send ping: {}",
                                                        e
                                                    ));
                                                } else {
                                                    session_outbound_traffic +=
                                                        ping_message.len() as u64;
                                                    add_message_for_peer(
                                                        "[SENT] 'ping' message with nonce."
                                                            .to_string(),
                                                    );
                                                }
                                            }
                                            Err(e) => add_message_for_peer(format!(
                                                "[ERROR] Failed to build ping message: {}",
                                                e
                                            )),
                                        }
                                        continue; // Continue the loop to wait for a response (pong).
                                    }
                                // For any other read errors, log the error and break.
                                add_message_for_peer(format!(
                                    "[ERROR] Failed to read message: {}",
                                    e
                                ));
                                break; // Exit loop on other read errors.
                            }
                        }
                    }
                    // When the peer loop breaks (e.g., due to error or shutdown),
                    // update known_peers with the total traffic and remove from active_peers.
                    add_message_for_peer(format!(
                        "Disconnected from {}. Session In: {} B, Session Out: {} B",
                        peer_addr_for_peer_thread,
                        session_inbound_traffic,
                        session_outbound_traffic
                    ));
                    let mut known_peers_lock = known_peers_clone_for_peer_thread.lock().unwrap();
                    known_peers_lock
                        .entry(peer_addr_for_peer_thread.clone())
                        .and_modify(|(total_in, total_out)| {
                            *total_in += session_inbound_traffic;
                            *total_out += session_outbound_traffic;
                        })
                        .or_insert((session_inbound_traffic, session_outbound_traffic));

                    let mut active_peers_lock = active_peers_clone_for_peer_thread.lock().unwrap();
                    if let Some(state) = active_peers_lock.remove(&peer_addr_for_peer_thread) {
                        if state.user_agent.contains("Gnostr") {
                            add_message_for_peer("[INFO] Gnostr peer disconnected. Scheduling immediate reconnect.".to_string());
                            discovered_peers_queue_peer.lock().unwrap().push(peer_addr_for_peer_thread.clone());
                        }
                    }
                });
            } else {
                // If connection attempt failed, wait a bit before the next attempt.
                std::thread::sleep(Duration::from_secs(2));
            }
        }
        add_message("Network thread finished. Press 'q' to exit TUI.".to_string());
    });

    // Initialize the Terminal User Interface (TUI).
    let mut terminal = init_tui()?;

    // Create the application state instance.
    let mut app = App::new(
        Arc::clone(&messages),
        Arc::clone(&running),
        Arc::clone(&block_height),
        Arc::clone(&block_hash),
        Arc::clone(&active_peers),
    );

    // Run the TUI application loop.
    // This loop handles drawing the UI and processing user input events.
    // It will return an error if the network thread exits unexpectedly, triggering
    // `restore_tui`.
    app.run(&mut terminal)?;

    // Restore the terminal to its original state upon application exit.
    restore_tui(&mut terminal)?;

    Ok(())
}
