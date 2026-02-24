use anyhow::Result;
use arti_client::{IsolationToken, StreamPrefs, TorClient, TorClientConfig};
use bitcoin::{
    Transaction,
    consensus::{Decodable, Encodable},
    p2p::{
        Address, Magic, ServiceFlags,
        address::AddrV2,
        message::{NetworkMessage, RawNetworkMessage},
        message_blockdata::Inventory,
        message_network::VersionMessage,
    },
};

use std::io::Cursor;
use rand::seq::SliceRandom;
use sha3::{Digest, Sha3_256};
use tor_rtcompat::PreferredRuntime;

use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::lookup_host,
    sync::Semaphore,
    task::JoinSet,
    time::timeout,
};

use data_encoding::BASE32_NOPAD;
use tracing::{error, info};
const DNS_SEEDS: &[&str] = &[
    "dnsseed.bluematt.me",
    "dnsseed.bitcoin.dashjr-list-of-p2p-nodes.us",
    "seed.bitcoinstats.com",
    "seed.btc.petertodd.net",
    "seed.bitcoin.sprovoost.nl",
    "dnsseed.emzy.de",
    "seed.bitcoin.wiz.biz",
    "seed.bitcoin.sipa.be",
    "seed.bitcoin.jonasschnelli.ch",
    "seed.mainnet.achownodes.xyz",
];

const NODE_LIBRE_RELAY: u64 = 1 << 29;
const NODE_NETWORK: u64 = 1 << 0;
const NODE_WITNESS: u64 = 1 << 3;
const MAX_CONCURRENT_DELIVERIES: usize = 100;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NetworkAddress {
    Ip(SocketAddr),
    Onion(String),
}
pub fn build_version_msg() -> VersionMessage {
    VersionMessage {
        version: 70016,
        services: ServiceFlags::from(NODE_NETWORK | NODE_WITNESS | NODE_LIBRE_RELAY),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        receiver: Address::new(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            ServiceFlags::from(NODE_NETWORK | NODE_LIBRE_RELAY),
        ),
        sender: Address::new(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            ServiceFlags::from(NODE_NETWORK | NODE_WITNESS | NODE_LIBRE_RELAY),
        ),
        nonce: rand::random::<u64>(),
        user_agent: "/Satoshi:27.0.0/".into(),
        start_height: 897157,
        relay: true,
    }
}

async fn deliver_poop_tx(
    addr: NetworkAddress,
    tx: Transaction,
    tor_client: Arc<TorClient<PreferredRuntime>>,
    prefs: StreamPrefs,
) -> Result<bool> {
    let txid = tx.compute_txid();

    let mut stream = match &addr {
        NetworkAddress::Ip(sa) => {
            let target = (sa.ip().to_string(), sa.port());

            timeout(
                CONNECTION_TIMEOUT,
                tor_client.connect_with_prefs(target, &prefs),
            )
            .await
            .map_err(|_| anyhow::anyhow!("timeout connecting to {}", sa))??
        }

        NetworkAddress::Onion(host) => timeout(
            CONNECTION_TIMEOUT,
            tor_client.connect_with_prefs((host.as_str(), 8333), &prefs),
        )
        .await
        .map_err(|_| anyhow::anyhow!("timeout connecting to {}", host))??,
    };

    if let Err(e) = send_msg(&mut stream, NetworkMessage::Version(build_version_msg())).await {
        return Err(e);
    }

    let (mut rd, mut wr) = stream.split();

    let peer_version_message = match timeout(Duration::from_secs(5), async {
        loop {
            match read_msg(&mut rd).await {
                Ok(raw_msg) => {
                    if let NetworkMessage::Version(version) = raw_msg.payload() {
                        break Ok::<VersionMessage, anyhow::Error>(version.clone());
                    }
                }
                Err(e) => {
                    break Err(e);
                }
            }
        }
    })
    .await
    {
        Ok(Ok(vm)) => vm,
        Ok(Err(e)) => {
            return Err(e);
        }
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Timeout waiting for peer Version from {:?}",
                addr
            ));
        }
    };

    let libre_flag_check = ServiceFlags::from(NODE_LIBRE_RELAY);
    if !peer_version_message.services.has(libre_flag_check) {
        return Ok(false);
    }

    if let Err(e) = send_msg(&mut wr, NetworkMessage::Verack).await {
        return Err(e);
    }

    // Dont wait for verack, just send the tx

    if let Err(e) = send_msg(&mut wr, NetworkMessage::Tx(tx.clone())).await {
        return Err(e);
    }

    if let Err(e) = send_msg(
        &mut wr,
        NetworkMessage::GetData(vec![Inventory::Transaction(txid)]),
    )
    .await
    {
        return Err(e);
    }

    let mut tx_confirmed_by_peer = false;
    loop {
        match timeout(CONNECTION_TIMEOUT, read_msg(&mut rd)).await {
            Ok(Ok(m)) => match m.payload() {
                NetworkMessage::Tx(received_tx) => {
                    if received_tx.compute_txid() == txid {
                        println!(
                            "[CONFIRMED HIT] {:?} (UA: '{}') direct hit confirmed on libre node! poop deliverd",
                            addr, peer_version_message.user_agent
                        );
                        tx_confirmed_by_peer = true;
                        break;
                    }
                }
                NetworkMessage::NotFound(not_found_list) => {
                    if not_found_list.iter().any(|inv| {
                        if let Inventory::Transaction(hash) = inv {
                            *hash == txid
                        } else {
                            false
                        }
                    }) {
                        error!(
                            "[NotFound] {:?} (UA: '{}') Possible LIAR detected! This node might be lying and tricking the code!! (If tx {} is already in a block, this is expected)",
                            addr, peer_version_message.user_agent, txid
                        );
                        break;
                    }
                }
                NetworkMessage::Inv(inv_list) => {
                    if inv_list.iter().any(|inv| {
                        if let Inventory::Transaction(hash) = inv {
                            *hash == txid
                        } else {
                            false
                        }
                    }) {
                        println!(
                            "[CONFIRMED HIT] INV returned TX: direct hit confirmed on libre node! poop deliverd to {:?}! (UA: '{}')",
                            addr, peer_version_message.user_agent
                        );
                        tx_confirmed_by_peer = true;
                        break;
                    }
                }

                _ => {}
            },
            Ok(Err(read_err)) => {
                error!(
                    "Read error from {:?} while awaiting Tx confirmation: {}. Possible LIAR detected!",
                    addr, read_err
                );
                break;
            }
            Err(_) => {
                error!(
                    "Timeout waiting for Tx, Inv, or Reject from {:?} (UA: '{}'). Possible LIAR detected!",
                    addr, peer_version_message.user_agent
                );
                break;
            }
        }
    }
    Ok(tx_confirmed_by_peer)
}

async fn crawl_seed_node(seed: &SocketAddr) -> Result<Vec<NetworkAddress>> {
    let mut found_peers = Vec::new();
    //print!("crawling seed:");
    //let mut spinner = Spinner::new(format!("{:?}", seed));
    println!("Crawling seed: {:?}", seed);
    let mut stream = match timeout(
        Duration::from_secs(1),
        tokio::net::TcpStream::connect((seed.ip().to_string(), seed.port())),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            println!("Error connecting to seed {:?}: {:?}", seed, e);
            return Err(e.into());
        }
        Err(_) => {
            println!("Timeout connecting to {:?}", seed);
            return Err(anyhow::anyhow!("Timeout connecting to {}", seed));
        }
    };

    send_msg(&mut stream, NetworkMessage::Version(build_version_msg())).await?;

    let (mut rd, mut wr) = stream.split();

    println!("Waiting for addresses from {:?}...", seed);

    loop {
        let msg_result = timeout(Duration::from_secs(1), read_msg(&mut rd)).await;

        let msg = match msg_result {
            Ok(Ok(m)) => m,
            Ok(Err(e)) => {
                error!("Read error from {:?}: {}", seed, e);
                break; // Break loop on read error
            }
            Err(_) => {
                error!("Timeout waiting for message from {:?}", seed);
                break; // Break loop on timeout
            }
        };

        match msg.payload() {
            NetworkMessage::Version(_) => {
                send_msg(&mut wr, NetworkMessage::SendAddrV2).await?;
                send_msg(&mut wr, NetworkMessage::Verack).await?;
            }
            NetworkMessage::Verack => {
                send_msg(&mut wr, NetworkMessage::GetAddr).await?;
            }
            NetworkMessage::Addr(list) => {
                let flag = ServiceFlags::from(NODE_LIBRE_RELAY);
                found_peers.extend(list.iter().filter_map(|(_, a)| {
                    if !a.services.has(flag) {
                        return None;
                    }

                    if let Ok(addr) = a.socket_addr() {
                        Some(NetworkAddress::Ip(addr))
                    } else {
                        None
                    }
                }));
                break;
            }
            NetworkMessage::AddrV2(list) => {
                let flag = ServiceFlags::from(NODE_LIBRE_RELAY);
                found_peers.extend(list.iter().filter_map(|a| {
                    if !a.services.has(flag) {
                        return None;
                    }
                    match &a.addr {
                        AddrV2::Ipv4(ip) => {
                            Some(NetworkAddress::Ip(SocketAddr::new(IpAddr::V4(*ip), a.port)))
                        }
                        AddrV2::Ipv6(ip) => {
                            Some(NetworkAddress::Ip(SocketAddr::new(IpAddr::V6(*ip), a.port)))
                        }
                        AddrV2::TorV3(key) => {
                            let onion_addr = tor_v3_onion_from_pubkey(key);
                            Some(NetworkAddress::Onion(onion_addr))
                        }
                        _ => None,
                    }
                }));
                break;
            }
                        _ => {
                            // If we receive any other message, we break the loop.
                            // This is to prevent infinite loops if the peer sends unexpected messages.
                            break;
                        }
                    }
                }
            
                found_peers.shuffle(&mut rand::rng());

    Ok(found_peers)
}

pub async fn send_msg<S: AsyncWriteExt + Unpin>(stream: &mut S, msg: NetworkMessage) -> Result<()> {
    let mut buf = Vec::new();
    RawNetworkMessage::new(Magic::BITCOIN, msg).consensus_encode(&mut buf)?;

    stream.write_all(&buf).await?;

    stream.flush().await?;
    Ok(())
}

pub async fn read_msg<R: AsyncReadExt + Unpin>(r: &mut R) -> Result<RawNetworkMessage> {
    let mut hdr = [0u8; 24];
    r.read_exact(&mut hdr).await?;
    let len = u32::from_le_bytes(hdr[16..20].try_into().unwrap()) as usize;
    let mut payload = vec![0; len];
    r.read_exact(&mut payload).await?;
    Ok(RawNetworkMessage::consensus_decode(&mut Cursor::new(
        [hdr.to_vec(), payload].concat(),
    ))?)
}

pub fn tor_v3_onion_from_pubkey(pubkey: &[u8; 32]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(b".onion checksum");
    hasher.update(pubkey);
    hasher.update([0x03]);
    let checksum = &hasher.finalize()[..2];

    let mut addr_raw = Vec::with_capacity(35);
    addr_raw.extend_from_slice(pubkey);
    addr_raw.extend_from_slice(checksum);
    addr_raw.push(0x03);

    BASE32_NOPAD.encode(&addr_raw).to_lowercase() + ".onion"
}

pub async fn send_raw_transaction_to_peers(tx_hex_string: String) -> Result<()> {
    let tx = bitcoin::consensus::deserialize::<Transaction>(&hex::decode(tx_hex_string)?)?;
    let txid = tx.compute_txid();

    let mut seed_addrs = Vec::new();
    let mut seed_tasks = JoinSet::new();

    for seed_host in DNS_SEEDS {
        info!("fetching addrs from {:?}", seed_host);

        let host = seed_host.to_owned();

        seed_tasks.spawn(async move {
            let lookup = lookup_host(format!("{}:8333", seed_host));

            match timeout(Duration::from_secs(2), lookup).await {
                Ok(Ok(addrs)) => {
                    let addrs: Vec<_> = addrs.collect();
                    Ok((host, addrs))
                }
                Ok(Err(e)) => Err(anyhow::Error::new(e)),
                Err(_) => {
                    error!("Timeout while looking up {}", seed_host);
                    Err(anyhow::anyhow!("Timeout"))
                }
            }
        });
    }

    while let Some(res) = seed_tasks.join_next().await {
        match res {
            Ok(Ok((host, addresses))) => {
                info!("{} returned {} IPs", host, addresses.len());
                seed_addrs.extend(addresses);
            }
            Ok(Err(crawl_error)) => {
                error!("dns seed node error: {crawl_error},");
            }
            Err(join_error) => {
                error!("join error during dns seed: {join_error}");
            }
        }
    }

    info!("found {} seed node addresses", seed_addrs.len());
    seed_addrs.shuffle(&mut rand::rng());

    info!("time to blast some nodes with pigeon poop! 🐦💩");

    info!("blasting tx {:?} to libre relay nodes...", txid);

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_DELIVERIES));

    let mut libre_peers = HashSet::<NetworkAddress>::new();
    let mut crawl_tasks = JoinSet::new();

    for addr in seed_addrs.clone() {
        crawl_tasks.spawn({
            let sem = semaphore.clone();
            async move {
                let _permit = sem.acquire_owned().await?;
                crawl_seed_node(&addr).await
            }
        });
    }

    while let Some(res) = crawl_tasks.join_next().await {
        match res {
            Ok(Ok(addresses)) => {
                libre_peers.extend(addresses);
            }
            Ok(Err(crawl_error)) => {
                error!("crawl seed node error: {}", crawl_error);
            }
            Err(join_error) => {
                error!("join error during crawl: {}", join_error);
            }
        }
    }

    info!(
        "found {} addresses advertising the libre relay service flag",
        libre_peers.len()
    );

    //connect to tor
    info!("Bootstrapping Tor client...");
    let config = TorClientConfig::builder().build()?;
    let tor_client = Arc::new(TorClient::create_bootstrapped(config).await?);

    let common_token = IsolationToken::no_isolation();
    let mut prefs = StreamPrefs::new();
    prefs.set_isolation(common_token);

    let mut poop_delivery_tasks = JoinSet::new();
    for peer_addr in libre_peers.clone() {
        let tx_clone = tx.clone();
        let permit = semaphore.clone().acquire_owned().await?;
        let tor_client = tor_client.clone();
        let peer_addr_cloned = peer_addr.clone();
        let prefs = prefs.clone();
        poop_delivery_tasks.spawn(async move {
            let _permit_guard = permit;
            match deliver_poop_tx(peer_addr_cloned.clone(), tx_clone, tor_client, prefs).await {
                Ok(true) => Ok(peer_addr_cloned.clone()),
                Ok(false) => Err((
                    peer_addr_cloned.clone(),
                    "No Tx confirmation, rejected, or skipped by peer.".to_string(),
                )),
                Err(e) => Err((peer_addr_cloned.clone(), e.to_string())),
            }
        });
    }

    let mut success_count = 0;

    while let Some(res) = poop_delivery_tasks.join_next().await {
        match res {
            Ok(Ok(_)) => {
                success_count += 1;
            }
            Ok(Err((_, _))) => (),
            Err(join_error) => {
                error!("join error during poop delivery: {}", join_error);
                if join_error.is_panic() {
                    error!("a poop delivery task panicked!");
                }
            }
        }
    }

    if success_count == 0 {
        error!(
            "No libre relay nodes accepted the transaction. TX {} may already be in a block or its invalid.",
            txid
        );
        return Ok(());
    }

    info!(
        "TX: {:?} blasted to {} libre relay nodes. GLHF",
        txid, success_count,
    );

    Ok(())
}
