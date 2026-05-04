use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    path::Path,
};

use libp2p::{
    kad::{Quorum, Record, RecordKey},
    swarm::Swarm,
};
use serde::{Deserialize, Serialize};

use crate::behaviour::Behaviour;

const BUCKET_KEY_PREFIX: &str = "gnostr/relay-buckets";
const ROOT_BUCKET_NAME: &str = "relays";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayBucketSnapshot {
    pub bucket: String,
    pub relays: Vec<String>,
}

pub fn collect_local_snapshots() -> std::io::Result<Vec<RelayBucketSnapshot>> {
    let config_dir = gnostr_crawler::relays::get_config_dir_path();
    let mut snapshots = Vec::new();

    let root_relays = gnostr_crawler::relay_io::load_relays_or_bootstrap();
    snapshots.push(RelayBucketSnapshot {
        bucket: ROOT_BUCKET_NAME.to_string(),
        relays: dedup_sorted(root_relays),
    });

    for entry in fs::read_dir(&config_dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }

        let bucket = entry.file_name().to_string_lossy().to_string();
        let relays = collect_bucket_relays(entry.path())?;
        if relays.is_empty() {
            continue;
        }

        snapshots.push(RelayBucketSnapshot {
            bucket,
            relays: dedup_sorted(relays),
        });
    }

    snapshots.sort_by(|a, b| a.bucket.cmp(&b.bucket));
    snapshots.dedup_by(|a, b| a.bucket == b.bucket);
    Ok(snapshots)
}

pub fn bucket_record_key(bucket: &str) -> RecordKey {
    RecordKey::new(&format!("{BUCKET_KEY_PREFIX}/{bucket}"))
}

pub async fn publish_local_snapshots(
    swarm: &mut Swarm<Behaviour>,
) -> Result<Vec<RelayBucketSnapshot>, Box<dyn Error>> {
    let snapshots = collect_local_snapshots()?;
    let peer_id = *swarm.local_peer_id();

    for snapshot in &snapshots {
        let key = bucket_record_key(&snapshot.bucket);
        let record = Record {
            key: key.clone(),
            value: serde_json::to_vec(snapshot)?,
            publisher: Some(peer_id),
            expires: None,
        };

        swarm
            .behaviour_mut()
            .kademlia
            .put_record(record, Quorum::Majority)?;
        swarm.behaviour_mut().kademlia.start_providing(key)?;
    }

    Ok(snapshots)
}

pub fn collect_bucket_relays(bucket_dir: impl AsRef<Path>) -> std::io::Result<Vec<String>> {
    let mut relays = BTreeSet::new();

    for entry in fs::read_dir(bucket_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name == "relays.json" || !name.ends_with(".json") {
            continue;
        }

        if let Some(host) = name.strip_suffix(".json") {
            relays.insert(gnostr_crawler::relay_fetch::websocket_http_url(host));
        }
    }

    Ok(relays.into_iter().collect())
}

fn dedup_sorted(relays: Vec<String>) -> Vec<String> {
    let mut relays = relays;
    relays.sort();
    relays.dedup();
    relays
}
