use std::collections::{BTreeMap, HashMap};

use crate::git2::types::{RepoState as Nip34RepoState, Unixtime};

/// Deterministic repo-state map used by the BQS layer.
pub type RepoStateRefs = BTreeMap<String, String>;

/// Canonical p2p view of a repo-state event and its refs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoStateSnapshot {
    pub identifier: String,
    pub refs: RepoStateRefs,
    pub event: crate::git2::types::EventV3,
}

impl RepoStateSnapshot {
    pub fn new(
        identifier: impl Into<String>,
        refs: RepoStateRefs,
        event: crate::git2::types::EventV3,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            refs,
            event,
        }
    }

    pub fn ref_value(&self, name: &str) -> Option<&str> {
        self.refs.get(name).map(String::as_str)
    }

    pub fn head(&self) -> Option<&str> {
        self.ref_value("HEAD")
    }

    pub fn branch_ref(&self, branch: &str) -> Option<&str> {
        self.ref_value(&format!("refs/heads/{branch}"))
    }

    pub fn tag_ref(&self, tag: &str) -> Option<&str> {
        self.ref_value(&format!("refs/tags/{tag}"))
    }
}

impl From<Nip34RepoState> for RepoStateSnapshot {
    fn from(repo_state: Nip34RepoState) -> Self {
        let refs = repo_state
            .state
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        Self::new(repo_state.identifier, refs, repo_state.event)
    }
}

impl From<RepoStateSnapshot> for Nip34RepoState {
    fn from(snapshot: RepoStateSnapshot) -> Self {
        let state = snapshot
            .refs
            .into_iter()
            .collect::<HashMap<String, String>>();
        Self {
            identifier: snapshot.identifier,
            state,
            event: snapshot.event,
        }
    }
}

/// Quorum-backed repo-state envelope used by the BQS layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoStateQuorum {
    pub identifier: String,
    pub snapshots: Vec<RepoStateSnapshot>,
    pub quorum_size: usize,
    pub consensus_utc: Option<Unixtime>,
}

impl RepoStateQuorum {
    pub fn new(identifier: impl Into<String>, quorum_size: usize) -> Self {
        Self {
            identifier: identifier.into(),
            snapshots: Vec::new(),
            quorum_size,
            consensus_utc: None,
        }
    }

    pub fn push(&mut self, snapshot: RepoStateSnapshot) -> Result<(), RepoStateSnapshot> {
        if snapshot.identifier != self.identifier {
            return Err(snapshot);
        }

        self.snapshots.push(snapshot);
        Ok(())
    }

    pub fn is_quorum_met(&self) -> bool {
        self.quorum_size > 0 && self.snapshots.len() >= self.quorum_size
    }

    pub fn consensus_snapshot(&self) -> Option<&RepoStateSnapshot> {
        self.snapshots.iter().max_by(|left, right| {
            left.event
                .created_at
                .cmp(&right.event.created_at)
                .then_with(|| left.event.id.cmp(&right.event.id))
        })
    }

    pub fn consensus_created_at(&self) -> Option<Unixtime> {
        self.consensus_snapshot().map(|snapshot| snapshot.event.created_at)
    }

    pub fn consensus_head(&self) -> Option<&str> {
        self.consensus_snapshot().and_then(RepoStateSnapshot::head)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::git2::types::PrivateKey;

    fn build_snapshot() -> RepoStateSnapshot {
        let private_key = PrivateKey::generate();
        let state = BTreeMap::from([
            ("refs/heads/main".to_string(), "ref: refs/heads/main".to_string()),
            ("refs/tags/v0.1.0".to_string(), "abcdef123456".to_string()),
        ]);
        let repo_state = Nip34RepoState::build("gnostr".to_string(), state.into_iter().collect(), &private_key)
            .expect("repo state");

        RepoStateSnapshot::from(repo_state)
    }

    #[test]
    fn snapshot_uses_deterministic_ref_ordering() {
        let snapshot = build_snapshot();
        let keys = snapshot.refs.keys().cloned().collect::<Vec<_>>();
        assert_eq!(keys, vec!["HEAD", "refs/heads/main", "refs/tags/v0.1.0"]);
        assert_eq!(snapshot.head(), Some("ref: refs/heads/main"));
        assert_eq!(snapshot.branch_ref("main"), Some("ref: refs/heads/main"));
        assert_eq!(snapshot.tag_ref("v0.1.0"), Some("abcdef123456"));
    }

    #[test]
    fn quorum_selects_latest_snapshot() {
        let mut older = build_snapshot();
        let mut newer = older.clone();

        older.event.created_at = Unixtime(1_000);
        older.event.id = crate::git2::types::Id::try_from_hex_string(
            "1111111111111111111111111111111111111111111111111111111111111111",
        )
        .expect("older id");
        newer.event.created_at = Unixtime(2_000);
        newer.event.id = crate::git2::types::Id::try_from_hex_string(
            "2222222222222222222222222222222222222222222222222222222222222222",
        )
        .expect("newer id");

        let mut quorum = RepoStateQuorum::new("gnostr", 2);
        quorum.push(older).expect("older snapshot");
        quorum.push(newer).expect("newer snapshot");

        assert!(quorum.is_quorum_met());
        assert_eq!(quorum.consensus_created_at(), Some(Unixtime(2_000)));
        assert_eq!(quorum.consensus_head(), Some("ref: refs/heads/main"));
    }

    #[test]
    fn snapshot_round_trips_to_nip34_repo_state() {
        let snapshot = build_snapshot();
        let repo_state: Nip34RepoState = snapshot.clone().into();
        let round_trip = RepoStateSnapshot::from(repo_state);

        assert_eq!(round_trip.identifier, snapshot.identifier);
        assert_eq!(round_trip.refs, snapshot.refs);
        assert_eq!(round_trip.event.kind, snapshot.event.kind);
    }

    #[test]
    fn quorum_rejects_mismatched_identifier() {
        let snapshot = build_snapshot();
        let mut quorum = RepoStateQuorum::new("other", 1);

        assert!(quorum.push(snapshot).is_err());
    }
}
