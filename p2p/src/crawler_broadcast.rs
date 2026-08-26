use std::{
    error::Error,
    fs,
    process::Stdio,
    path::Path,
    time::Duration,
};

use libp2p::gossipsub::IdentTopic;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use gnostr_crawler::processor::SHITLIST_RELAYS;
use crate::relay_paths::get_config_dir_path;
use crate::{message::Event, relay_bridge::NostrRelayConnection};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RelayBucket {
    pub nip: i32,
    pub relays: Vec<String>,
}

pub const RELAY_BUCKET_TOPIC_PREFIX: &str = "crawler/relay-buckets";

pub fn bucket_topic(nip: i32) -> IdentTopic {
    IdentTopic::new(format!("{RELAY_BUCKET_TOPIC_PREFIX}/{nip}"))
}

fn is_shitlisted(url: &str) -> bool {
    SHITLIST_RELAYS.iter().any(|relay| url.contains(relay))
}

fn relay_host(relay: &str) -> Option<&str> {
    let relay = relay.trim();
    let relay = relay
        .strip_prefix("wss://")
        .or_else(|| relay.strip_prefix("ws://"))?;
    let authority = relay.split('/').next().unwrap_or("");
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    if authority.starts_with('[') {
        Some(authority)
    } else {
        Some(authority.split(':').next().unwrap_or(authority))
    }
}

fn is_private_ipv4_relay(relay: &str) -> bool {
    let host = match relay_host(relay) {
        Some(host) => host,
        None => return false,
    };

    let mut octets = host.split('.');
    let first = match octets.next().and_then(|part| part.parse::<u8>().ok()) {
        Some(first) => first,
        None => return false,
    };
    let second = match octets.next().and_then(|part| part.parse::<u8>().ok()) {
        Some(second) => second,
        None => return false,
    };

    (first == 10)
        || (first == 172 && (16..=31).contains(&second))
        || (first == 192 && second == 168)
        || (first == 100 && (64..=127).contains(&second))
}

fn is_valid_relay_url(relay: &str) -> bool {
    relay_host(relay)
        .map(|host| {
            !host.is_empty()
                && !host.starts_with('-')
                && !is_private_ipv4_relay(relay)
                && !(host == "localhost" || host == "127.0.0.1")
        })
        .unwrap_or(false)
}

fn is_loopback_relay(relay: &str) -> bool {
    relay_host(relay)
        .map(|host| host == "localhost" || host == "127.0.0.1")
        .unwrap_or(false)
}

const RELAY_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const RELAY_PUBLISH_TIMEOUT: Duration = Duration::from_secs(10);

fn normalize_relay_entry(relay: &str) -> Option<String> {
    let relay = relay
        .trim()
        .trim_start_matches("- ")
        .trim_start_matches('-')
        .trim_matches('\'')
        .trim_matches('"')
        .trim();

    if relay.is_empty() {
        None
    } else {
        Some(relay.to_string())
    }
}

pub fn load_relay_bucket_from_dir(dir: &Path) -> Result<RelayBucket, Box<dyn Error>> {
    let nip = dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("invalid relay bucket directory name")?
        .parse::<i32>()?;

    let relays_json = dir.join("relays.json");
    let relays_yaml = dir.join("relays.yaml");

    let relays = if relays_json.exists() {
        serde_json::from_str::<Vec<String>>(&fs::read_to_string(&relays_json)?)?
    } else if relays_yaml.exists() {
        serde_yaml::from_str::<Vec<String>>(&fs::read_to_string(&relays_yaml)?)?
    } else {
        Vec::new()
    };

    Ok(RelayBucket {
        nip,
        relays: relays
            .into_iter()
            .filter_map(|relay| normalize_relay_entry(&relay))
            .collect(),
    })
}

pub fn load_relay_buckets(config_dir: &Path) -> Result<Vec<RelayBucket>, Box<dyn Error>> {
    let mut buckets = Vec::new();
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        if let Ok(bucket) = load_relay_bucket_from_dir(&entry.path()) {
            if !bucket.relays.is_empty() {
                buckets.push(bucket);
            }
        }
    }
    buckets.sort_by_key(|bucket| bucket.nip);
    Ok(buckets)
}

pub fn load_crawler_relay_buckets() -> Result<Vec<RelayBucket>, Box<dyn Error>> {
    let config_dir = get_config_dir_path();
    load_relay_buckets(&config_dir)
}

pub async fn broadcast_crawler_relay_buckets(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
) -> Result<usize, Box<dyn Error>> {
    let buckets = load_crawler_relay_buckets()?;
    if swarm.connected_peers().next().is_none() {
        info!("skipping crawler relay bucket broadcast: no connected peers");
        return Ok(0);
    }

    let mut published = 0usize;

    for bucket in buckets {
        let topic = bucket_topic(bucket.nip);
        let payload = serde_json::to_vec(&bucket)?;
        debug!(
            "broadcasting crawler relay bucket nip={} relays={}",
            bucket.nip,
            bucket.relays.len()
        );
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        let _ = swarm.behaviour_mut().gossipsub.publish(topic, payload)?;
        published += 1;
    }

    info!("broadcasted {published} crawler relay bucket(s)");
    Ok(published)
}

pub async fn publish_local_snapshots(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
) -> Result<usize, Box<dyn Error>> {
    broadcast_crawler_relay_buckets(swarm).await
}

struct CrawlerServerGuard {
    child: tokio::process::Child,
}

impl Drop for CrawlerServerGuard {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

async fn fetch_live_crawler_relays() -> anyhow::Result<Option<Vec<String>>> {
    let response = match reqwest::get("http://127.0.0.1:8080/relays.yaml").await {
        Ok(response) => response,
        Err(_) => return Ok(None),
    };

    if !response.status().is_success() {
        return Ok(None);
    }

    let relays = response.text().await?;
    let relays = serde_yaml::from_str::<Vec<String>>(&relays)
        .or_else(|_| {
            Ok::<Vec<String>, serde_yaml::Error>(
                relays
                    .lines()
                    .map(str::trim)
                    .filter_map(normalize_relay_entry)
                    .collect(),
            )
        })?;
    let relays = relays
        .into_iter()
        .filter_map(|relay| normalize_relay_entry(&relay))
        .collect::<Vec<_>>();

    if relays.is_empty() {
        Ok(None)
    } else {
        Ok(Some(relays))
    }
}

async fn spawn_crawler_server() -> anyhow::Result<CrawlerServerGuard> {
    let mut command = tokio::process::Command::new("gnostr");
    command
        .args(["crawler", "serve", "--port", "8080"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let child = command.spawn()?;
    Ok(CrawlerServerGuard { child })
}

pub async fn bootstrap_crawler_relay_buckets(
    config_dir: &Path,
    nip: i32,
) -> anyhow::Result<Vec<String>> {
    let relays = if let Some(relays) = fetch_live_crawler_relays().await? {
        relays
    } else {
        let _guard = spawn_crawler_server().await?;
        tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                if let Some(relays) = fetch_live_crawler_relays().await? {
                    break Ok::<Vec<String>, anyhow::Error>(relays);
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await??
    };
    let relays: Vec<String> = relays
        .into_iter()
        .filter_map(|relay| normalize_relay_entry(&relay))
        .filter_map(|relay| {
            if is_valid_relay_url(&relay) || is_loopback_relay(&relay) {
                Some(relay)
            } else {
                warn!(
                    "bootstrap_crawler_relay_buckets: rejecting invalid relay_url={}",
                    relay
                );
                println!(
                    "pretty_print_attestations relay_rejected relay_url={} reason=invalid or private host",
                    relay
                );
                None
            }
        })
        .filter(|relay| {
            if is_shitlisted(relay) {
                warn!("bootstrap_crawler_relay_buckets: skipping shitlisted relay {}", relay);
                false
            } else {
                true
            }
        })
        .collect();

    let bucket_dir = config_dir.join(nip.to_string());
    fs::create_dir_all(&bucket_dir)?;
    fs::write(
        bucket_dir.join("relays.yaml"),
        serde_yaml::to_string(&relays)?,
    )?;
    fs::write(
        bucket_dir.join("relays.json"),
        serde_json::to_string_pretty(&relays)?,
    )?;
    fs::write(bucket_dir.join("relays.txt"), relays.join(" "))?;

    Ok(relays)
}

pub async fn broadcast_event_to_crawler_relays(
    config_dir: &Path,
    event: &Event,
) -> anyhow::Result<usize> {
    let buckets = load_relay_buckets(config_dir).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let mut published = 0usize;

    for bucket in buckets {
        for relay_url in bucket.relays {
            if !is_valid_relay_url(&relay_url) && !is_loopback_relay(&relay_url) {
                warn!(
                    "broadcast_event_to_crawler_relays: rejecting invalid relay_url={}",
                    relay_url
                );
                println!(
                    "pretty_print_attestations relay_rejected nip={} relay_url={} reason=invalid or private host",
                    bucket.nip, relay_url
                );
                continue;
            }
            println!(
                "pretty_print_attestations relays_sent_to nip={} relay_url={}",
                bucket.nip, relay_url
            );
            let relay_url_for_task = relay_url.clone();
            match tokio::time::timeout(
                RELAY_CONNECT_TIMEOUT + RELAY_PUBLISH_TIMEOUT,
                async move {
                    let mut connection = NostrRelayConnection::connect(relay_url_for_task.clone())
                        .await
                        .map_err(|err| {
                            anyhow::anyhow!(
                                "broadcast_event_to_crawler_relays: connect error for {}: {}",
                                relay_url_for_task,
                                err
                            )
                        })?;

                    connection
                        .publish_event(event.clone())
                        .await
                        .map_err(|err| {
                            anyhow::anyhow!(
                                "broadcast_event_to_crawler_relays: publish error for {}: {}",
                                relay_url_for_task,
                                err
                            )
                        })?;

                    Ok::<(), anyhow::Error>(())
                },
            )
            .await
            {
                Ok(Ok(())) => {
                    published += 1;
                }
                Ok(Err(err)) => {
                    warn!("{err}");
                    continue;
                }
                Err(_) => {
                    warn!(
                        "broadcast_event_to_crawler_relays: timeout after {:?} for {}",
                        RELAY_CONNECT_TIMEOUT + RELAY_PUBLISH_TIMEOUT,
                        relay_url
                    );
                    println!(
                        "pretty_print_attestations relay_timeout nip={} relay_url={} timeout_secs={}",
                        bucket.nip,
                        relay_url,
                        (RELAY_CONNECT_TIMEOUT + RELAY_PUBLISH_TIMEOUT).as_secs()
                    );
                    continue;
                }
            }
        }
    }

    Ok(published)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        fs::{create_dir_all, write},
        sync::{Mutex, OnceLock},
    };
    use tempfile::tempdir;

    struct EnvGuard {
        key: &'static str,
        value: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
            let previous = env::var_os(key);
            env::set_var(key, value);
            Self { key, value: previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.value {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn loads_bucket_from_json_dir() {
        let root = tempfile::tempdir().expect("tempdir");
        let nip_dir = root.path().join("42");
        create_dir_all(&nip_dir).expect("dir");
        write(
            nip_dir.join("relays.json"),
            serde_json::to_string(&vec!["wss://relay.example".to_string()]).expect("json"),
        )
        .expect("write");

        let buckets = load_relay_buckets(root.path()).expect("buckets");
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].nip, 42);
        assert_eq!(buckets[0].relays, vec!["wss://relay.example"]);
    }

    #[test]
    fn bucket_topic_uses_nip_suffix() {
        assert_eq!(bucket_topic(7).to_string(), "crawler/relay-buckets/7");
    }

    #[test]
    #[ignore]
    fn loads_crawler_relay_buckets_from_temp_config() {
        let _guard = test_lock().lock().expect("test lock");

        let home_dir = tempdir().expect("home dir");
        let config_dir = home_dir.path().join("config");
        let _home_guard = EnvGuard::set("HOME", home_dir.path());
        let _xdg_guard = EnvGuard::set("XDG_CONFIG_HOME", &config_dir);

        let crawler_config_dir = get_config_dir_path();
        let bucket_dir = crawler_config_dir.join("23");
        create_dir_all(&bucket_dir).expect("bucket dir");
        write(
            bucket_dir.join("relays.yaml"),
            "- wss://relay.example\n",
        )
        .expect("bucket relays");

        let buckets = load_crawler_relay_buckets().expect("buckets");
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].nip, 23);
        assert_eq!(buckets[0].relays, vec!["wss://relay.example"]);
    }

    #[test]
    fn rejects_invalid_relay_hostnames() {
        assert!(!is_valid_relay_url("wss://-auth.nostr1.com/"));
        assert!(!is_valid_relay_url("wss://-pub.wellorder.net/"));
        assert!(!is_valid_relay_url("wss://192.168.1.133:4848/"));
        assert!(!is_valid_relay_url("wss://192.168.100.190:7777/"));
        assert!(!is_valid_relay_url("wss://10.0.10.21:4848/"));
        assert!(!is_valid_relay_url("wss://172.16.0.1:4848/"));
        assert!(!is_valid_relay_url("wss://172.31.255.255:4848/"));
        assert!(!is_valid_relay_url("wss://100.71.217.147:4848/"));
        assert!(!is_valid_relay_url("wss://100.73.251.113/"));
        assert!(is_loopback_relay("wss://localhost:4848/"));
        assert!(is_loopback_relay("ws://127.0.0.1:4848/"));
        assert!(is_valid_relay_url("wss://relay.example/"));
    }
}
