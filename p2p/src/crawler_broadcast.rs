use std::{
    error::Error,
    fs,
    path::Path,
};

use libp2p::gossipsub::IdentTopic;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

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

    Ok(RelayBucket { nip, relays })
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

pub async fn broadcast_event_to_crawler_relays(
    config_dir: &Path,
    event: &Event,
) -> anyhow::Result<usize> {
    let buckets = load_relay_buckets(config_dir).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let mut published = 0usize;

    for bucket in buckets {
        for relay_url in bucket.relays {
            println!(
                "pretty_print_attestations relays_sent_to nip={} relay_url={}",
                bucket.nip, relay_url
            );
            let mut connection = NostrRelayConnection::connect(relay_url.clone()).await?;
            connection.publish_event(event.clone()).await?;
            published += 1;
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
}
