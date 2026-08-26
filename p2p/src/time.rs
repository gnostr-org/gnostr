//! P2P Byzantine Fault Tolerant time synchronization.
//!
//! This module provides a small distributed clock abstraction backed by
//! libp2p gossip and request/response sync messages.

use chrono::{DateTime, Duration, Utc};
use futures::StreamExt;
use libp2p::{
    gossipsub, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(test)]
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration as StdDuration, Instant};

/// Trait for clock implementations.
pub trait Clock: Send + Sync {
    /// Get the current UTC time according to this clock.
    fn now_utc(&self) -> DateTime<Utc>;
    /// Get the current synchronization status.
    fn status(&self) -> ClockStatus;
    /// Get detailed metrics about clock state.
    fn get_metrics(&self) -> ClockMetrics;
}

/// Represents the synchronization status of the clock.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClockStatus {
    /// Initial state, not yet synchronized.
    Init,
    /// Successfully synchronized with peers.
    Synced,
    /// Adjusting time gradually.
    Slewing,
    /// Clock is unreliable due to an error condition.
    Unreliable(String),
}

/// Metrics about the clock's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockMetrics {
    /// Current slew rate (1.0 = no adjustment).
    pub slew_rate: f64,
    /// Current offset from system time in milliseconds.
    pub offset_ms: i64,
    /// Current synchronization status.
    pub status: ClockStatus,
    /// Seconds since the last successful sync.
    pub last_sync_secs_ago: u64,
}

/// Health alert broadcast to peers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthAlert {
    /// Peer ID that generated the alert.
    pub peer_id: String,
    /// Reason for the alert.
    pub reason: String,
    /// Timestamp when the alert was generated.
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClockCheckpoint {
    last_adjustment_ns: i64,
    last_slew_rate: f64,
}

/// Time estimation from a peer.
#[derive(Debug, Clone, Copy)]
pub struct Estimation {
    /// Estimated clock difference.
    pub d: f64,
    /// Accuracy bound.
    pub a: f64,
}

/// Core synchronization state machine.
pub struct SyncState {
    base_utc: DateTime<Utc>,
    base_instant: Instant,
    slew_rate: f64,
    persistence_path: PathBuf,
    last_emitted_utc: DateTime<Utc>,
    last_sync_success: Instant,
    /// Byzantine fault tolerance parameter.
    pub f: usize,
    /// Current clock status.
    pub status: ClockStatus,
    /// Pending alert to broadcast.
    pub pending_alert: Option<String>,
}

impl SyncState {
    /// Create a new `SyncState` with a fault tolerance budget and storage file.
    pub fn new(f: usize, storage_file: &str) -> Self {
        let path = PathBuf::from(storage_file);
        let mut base_utc = Utc::now();
        let mut slew_rate = 1.0;
        let mut status = ClockStatus::Init;

        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(checkpoint) = serde_json::from_str::<ClockCheckpoint>(&data) {
                base_utc = Utc::now() + Duration::nanoseconds(checkpoint.last_adjustment_ns);
                slew_rate = checkpoint.last_slew_rate;
                status = ClockStatus::Synced;
            }
        }

        Self {
            base_utc,
            base_instant: Instant::now(),
            slew_rate,
            persistence_path: path,
            last_emitted_utc: Utc::now(),
            last_sync_success: Instant::now(),
            f,
            status,
            pending_alert: None,
        }
    }

    /// Get the current logical UTC time with monotonicity guarantees.
    pub fn get_logical_utc(&mut self) -> DateTime<Utc> {
        let elapsed = self.base_instant.elapsed().as_nanos() as f64;
        let slewed = elapsed * self.slew_rate;
        let mut current = self.base_utc + Duration::nanoseconds(slewed as i64);

        if current <= self.last_emitted_utc {
            current = self.last_emitted_utc + Duration::nanoseconds(1);
        }
        self.last_emitted_utc = current;

        if self.last_sync_success.elapsed() > StdDuration::from_secs(300) {
            if !matches!(self.status, ClockStatus::Unreliable(_)) {
                self.pending_alert = Some("Consensus Lost".into());
            }
            self.status = ClockStatus::Unreliable("Timeout".into());
        }

        current
    }

    /// Get current clock metrics without mutating the state.
    pub fn get_metrics(&self) -> ClockMetrics {
        ClockMetrics {
            slew_rate: self.slew_rate,
            offset_ms: (self.last_emitted_utc - Utc::now()).num_milliseconds(),
            status: self.status.clone(),
            last_sync_secs_ago: self.last_sync_success.elapsed().as_secs(),
        }
    }

    /// Apply Byzantine Fault Tolerant synchronization from peer estimates.
    pub fn apply_bft_sync(&mut self, estimates: Vec<Estimation>) {
        let count = estimates.len();
        if count < (2 * self.f + 1) {
            return;
        }

        let mut d_overs: Vec<f64> = estimates.iter().map(|e| e.d + e.a).collect();
        let mut d_unders: Vec<f64> = estimates.iter().map(|e| e.d - e.a).collect();
        d_overs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        d_unders.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let m_min = d_overs[self.f];
        let m_max = d_unders[count - 1 - self.f];

        if m_min <= m_max {
            let offset = (m_min + m_max) / 2.0;
            self.base_utc = self.get_logical_utc();
            self.base_instant = Instant::now();
            self.slew_rate = 1.0 + (offset / 30.0).clamp(-0.005, 0.005);
            self.last_sync_success = Instant::now();
            self.status = if offset.abs() < 0.01 {
                ClockStatus::Synced
            } else {
                ClockStatus::Slewing
            };

            let total_offset_ns = (self.get_logical_utc() - Utc::now())
                .num_nanoseconds()
                .unwrap_or(0);
            let cp = ClockCheckpoint {
                last_adjustment_ns: total_offset_ns,
                last_slew_rate: self.slew_rate,
            };
            let _ = fs::write(&self.persistence_path, serde_json::to_string(&cp).unwrap());
        } else {
            self.status = ClockStatus::Unreliable("Byzantine Error".into());
            self.pending_alert = Some("Byzantine Partition".into());
        }
    }
}

/// Request for time synchronization.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncRequest {
    /// Timestamp when the request was sent.
    pub t1: i64,
}

/// Response to a time synchronization request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncResponse {
    /// Original request timestamp.
    pub t1: i64,
    /// Timestamp when the response was generated.
    pub t2: i64,
}

/// Combined network behaviour for time synchronization.
#[derive(Debug)]
pub enum TimeSyncBehaviourEvent {
    /// Request-response events.
    RequestResponse(request_response::Event<SyncRequest, SyncResponse>),
    /// Gossipsub events.
    Gossipsub(gossipsub::Event),
}

impl From<request_response::Event<SyncRequest, SyncResponse>> for TimeSyncBehaviourEvent {
    fn from(event: request_response::Event<SyncRequest, SyncResponse>) -> Self {
        Self::RequestResponse(event)
    }
}

impl From<gossipsub::Event> for TimeSyncBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        Self::Gossipsub(event)
    }
}

/// Combined network behaviour for time synchronization.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "TimeSyncBehaviourEvent")]
pub struct TimeSyncBehaviour {
    /// Request-response protocol for time sync.
    pub request_response: request_response::cbor::Behaviour<SyncRequest, SyncResponse>,
    /// Gossipsub for broadcasting alerts.
    pub gossipsub: gossipsub::Behaviour,
}

/// P2P clock implementation using libp2p.
pub struct P2PClock {
    /// Shared synchronization state.
    pub inner: Arc<RwLock<SyncState>>,
}

impl P2PClock {
    /// Create a new P2PClock with a given fault tolerance and storage file.
    pub fn new(f: usize, storage_file: &str) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SyncState::new(f, storage_file))),
        }
    }
}

impl Clock for P2PClock {
    fn now_utc(&self) -> DateTime<Utc> {
        self.inner.write().unwrap().get_logical_utc()
    }

    fn status(&self) -> ClockStatus {
        self.inner.read().unwrap().status.clone()
    }

    fn get_metrics(&self) -> ClockMetrics {
        let s = self.inner.read().unwrap();
        ClockMetrics {
            slew_rate: s.slew_rate,
            offset_ms: (s.last_emitted_utc - Utc::now()).num_milliseconds(),
            status: s.status.clone(),
            last_sync_secs_ago: s.last_sync_success.elapsed().as_secs(),
        }
    }
}

/// Run the P2P time synchronization daemon.
pub async fn run_time_sync_daemon() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let (p2p_clock, shared_state) = {
        let state = Arc::new(RwLock::new(SyncState::new(1, "clock_final.json")));
        (P2PClock { inner: state.clone() }, state)
    };

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(TimeSyncBehaviour {
                request_response: request_response::cbor::Behaviour::new(
                    [(StreamProtocol::new("/time/5.0"), ProtocolSupport::Full)],
                    request_response::Config::default(),
                ),
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    Default::default(),
                )
                .expect("valid gossipsub config"),
            })
        })?
        .build();

    let alert_topic = gossipsub::IdentTopic::new("clock-alerts");
    swarm.behaviour_mut().gossipsub.subscribe(&alert_topic)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut peer_estimates: HashMap<PeerId, Estimation> = HashMap::new();
    let mut sync_interval = tokio::time::interval(StdDuration::from_secs(10));
    let mut metrics_interval = tokio::time::interval(StdDuration::from_secs(60));

    loop {
        tokio::select! {
            _ = metrics_interval.tick() => {
                let m = p2p_clock.get_metrics();
                println!("[METRICS] Status: {:?} | Slew: {:.6} | Offset: {}ms", m.status, m.slew_rate, m.offset_ms);
            }

            _ = sync_interval.tick() => {
                let alert = {
                    let mut s = shared_state.write().unwrap();
                    s.pending_alert.take().map(|reason| HealthAlert {
                        peer_id: swarm.local_peer_id().to_string(),
                        reason,
                        timestamp: s.get_logical_utc(),
                    })
                };

                if let Some(a) = alert {
                    if let Ok(data) = serde_json::to_vec(&a) {
                        let _ = swarm.behaviour_mut().gossipsub.publish(alert_topic.clone(), data);
                    }
                }

                for peer in swarm.connected_peers().cloned().collect::<Vec<_>>() {
                    let t1 = p2p_clock.now_utc().timestamp_millis();
                    swarm.behaviour_mut().request_response.send_request(&peer, SyncRequest { t1 });
                }
            }

            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(TimeSyncBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                    if let Ok(alert) = serde_json::from_slice::<HealthAlert>(&message.data) {
                        eprintln!(">>> EXTERNAL CLOCK ALERT: Peer {} is {}", alert.peer_id, alert.reason);
                    }
                }
                SwarmEvent::Behaviour(TimeSyncBehaviourEvent::RequestResponse(request_response::Event::Message { peer, message })) => {
                    match message {
                        request_response::Message::Request { request, channel, .. } => {
                            let t2 = p2p_clock.now_utc().timestamp_millis();
                            let _ = swarm.behaviour_mut().request_response.send_response(channel, SyncResponse { t1: request.t1, t2 });
                        }
                        request_response::Message::Response { response, .. } => {
                            let t3 = p2p_clock.now_utc().timestamp_millis();
                            peer_estimates.insert(peer, Estimation {
                                d: (response.t2 - ((t3 + response.t1) / 2)) as f64 / 1000.0,
                                a: (t3 - response.t1) as f64 / 2000.0,
                            });
                            if peer_estimates.len() >= 3 {
                                shared_state.write().unwrap().apply_bft_sync(peer_estimates.values().cloned().collect());
                                peer_estimates.clear();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair_from_seed;
    use gnostr_asyncgit::{blockheight::blockheight_sync, weeble::weeble_sync, wobble::wobble_sync};
    use tempfile::NamedTempFile;

    fn padded_metric_identity(metric: &str) -> String {
        format!("{:0>64}", metric.trim())
    }

    fn relay_node_id(label: &str, seed: String) -> String {
        let peer_id = keypair_from_seed(Some(seed)).public().to_peer_id();
        format!("{label} node_id={peer_id}")
    }

    #[test]
    fn test_sync_state_new() {
        let state = SyncState::new(1, "/tmp/test_clock.json");
        assert_eq!(state.f, 1);
        assert!((state.slew_rate - 1.0).abs() < 0.01 || state.status == ClockStatus::Synced);
    }

    #[test]
    fn test_clock_monotonicity() {
        let mut state = SyncState::new(1, "/tmp/test_clock2.json");
        let t1 = state.get_logical_utc();
        let t2 = state.get_logical_utc();
        let t3 = state.get_logical_utc();
        assert!(t2 > t1);
        assert!(t3 > t2);
    }

    #[test]
    fn test_bft_sync_insufficient_estimates() {
        let mut state = SyncState::new(1, "/tmp/test_clock3.json");
        let initial_slew = state.slew_rate;
        state.apply_bft_sync(vec![
            Estimation { d: 0.01, a: 0.001 },
            Estimation { d: 0.02, a: 0.001 },
        ]);
        assert_eq!(state.slew_rate, initial_slew);
    }

    #[test]
    fn test_bft_sync_valid() {
        let mut state = SyncState::new(1, "/tmp/test_clock4.json");
        state.apply_bft_sync(vec![
            Estimation { d: 0.005, a: 0.001 },
            Estimation { d: 0.005, a: 0.001 },
            Estimation { d: 0.007, a: 0.001 },
            Estimation { d: 0.007, a: 0.001 },
        ]);
        assert!(matches!(state.status, ClockStatus::Synced | ClockStatus::Slewing));
    }

    #[test]
    fn test_multi_peer_time_consensus_with_outlier() {
        let checkpoint = NamedTempFile::new().expect("temp checkpoint");
        let checkpoint_path = checkpoint.path().to_string_lossy().to_string();
        let mut state = SyncState::new(1, &checkpoint_path);

        let peer_estimates = vec![
            ("peer-alpha", Estimation { d: 0.005, a: 0.001 }),
            ("peer-beta", Estimation { d: 0.005, a: 0.001 }),
            ("peer-gamma", Estimation { d: 0.007, a: 0.001 }),
            ("peer-delta", Estimation { d: 0.007, a: 0.001 }),
            ("peer-byzantine", Estimation { d: 0.250, a: 0.001 }),
        ];

        println!(
            "before consensus: status={:?}, slew_rate={:.6}",
            state.status, state.slew_rate
        );
        for (peer, estimate) in &peer_estimates {
            println!(
                "peer sample: {peer} -> d={:.6}s a={:.6}s",
                estimate.d, estimate.a
            );
        }

        let estimates: Vec<Estimation> = peer_estimates.iter().map(|(_, estimate)| *estimate).collect();
        let mut d_overs: Vec<f64> = estimates.iter().map(|e| e.d + e.a).collect();
        let mut d_unders: Vec<f64> = estimates.iter().map(|e| e.d - e.a).collect();
        d_overs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        d_unders.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let m_min = d_overs[state.f];
        let m_max = d_unders[estimates.len() - 1 - state.f];
        println!(
            "consensus window: m_min={:.6}s m_max={:.6}s",
            m_min, m_max
        );

        state.apply_bft_sync(estimates);

        println!(
            "after consensus: status={:?}, slew_rate={:.6}, pending_alert={:?}",
            state.status, state.slew_rate, state.pending_alert
        );

        assert!(matches!(state.status, ClockStatus::Synced | ClockStatus::Slewing));
        assert!(state.pending_alert.is_none());
        assert!((state.slew_rate - 1.0).abs() <= 0.005);
    }

    #[test]
    fn test_quorum_churn_replaces_original_nodes() {
        let checkpoint = NamedTempFile::new().expect("temp checkpoint");
        let checkpoint_path = checkpoint.path().to_string_lossy().to_string();
        let mut state = SyncState::new(1, &checkpoint_path);
        let mut last_time = state.get_logical_utc();
        let mut last_round_peer_names: HashSet<&str> = HashSet::new();
        println!(
            "initial logical utc: {} status={:?} slew_rate={:.6}",
            last_time.to_rfc3339(),
            state.status,
            state.slew_rate
        );

        let rounds: Vec<(&str, Vec<(&str, Estimation)>)> = vec![
            (
                "bootstrap",
                vec![
                    ("peer-alpha", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-beta", Estimation { d: 0.005, a: 0.001 }),
                ],
            ),
            (
                "quorum-forms",
                vec![
                    ("peer-alpha", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-beta", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-gamma", Estimation { d: 0.007, a: 0.001 }),
                    ("peer-delta", Estimation { d: 0.007, a: 0.001 }),
                ],
            ),
            (
                "churn-one",
                vec![
                    ("peer-beta", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-gamma", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-delta", Estimation { d: 0.007, a: 0.001 }),
                    ("peer-epsilon", Estimation { d: 0.007, a: 0.001 }),
                ],
            ),
            (
                "churn-two",
                vec![
                    ("peer-gamma", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-delta", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-epsilon", Estimation { d: 0.007, a: 0.001 }),
                    ("peer-zeta", Estimation { d: 0.007, a: 0.001 }),
                ],
            ),
            (
                "replacement-complete",
                vec![
                    ("peer-epsilon", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-zeta", Estimation { d: 0.005, a: 0.001 }),
                    ("peer-eta", Estimation { d: 0.007, a: 0.001 }),
                    ("peer-theta", Estimation { d: 0.007, a: 0.001 }),
                ],
            ),
        ];

        for (label, peers) in rounds {
            let current_peer_names: HashSet<&str> = peers.iter().map(|(peer, _)| *peer).collect();
            let entered: Vec<&str> = current_peer_names
                .difference(&last_round_peer_names)
                .copied()
                .collect();
            let left: Vec<&str> = last_round_peer_names
                .difference(&current_peer_names)
                .copied()
                .collect();

            println!(
                "round {label}: peers={} quorum_needed={} entered={entered:?} left={left:?}",
                peers.len(),
                2 * state.f + 1
            );
            last_round_peer_names = current_peer_names;
            for (peer, estimate) in &peers {
                println!(
                    "peer sample: {peer} -> d={:.6}s a={:.6}s",
                    estimate.d, estimate.a
                );
            }

            let estimates: Vec<Estimation> = peers.iter().map(|(_, estimate)| *estimate).collect();
            let mut d_overs: Vec<f64> = estimates.iter().map(|e| e.d + e.a).collect();
            let mut d_unders: Vec<f64> = estimates.iter().map(|e| e.d - e.a).collect();
            d_overs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            d_unders.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let m_min = d_overs[state.f];
            let m_max = d_unders[estimates.len() - 1 - state.f];
            println!(
                "consensus window: m_min={:.6}s m_max={:.6}s",
                m_min, m_max
            );

            state.apply_bft_sync(estimates);

            let now: DateTime<Utc> = state.get_logical_utc();
            let logical_delta = now - last_time;
            println!(
                "after round {label}: utc={} delta={}ms status={:?} slew_rate={:.6} pending_alert={:?}",
                now.to_rfc3339(),
                logical_delta.num_milliseconds(),
                state.status,
                state.slew_rate,
                state.pending_alert
            );

            if peers.len() < 2 * state.f + 1 {
                assert_eq!(state.status, ClockStatus::Init);
                assert!(state.pending_alert.is_none());
            } else {
                assert!(matches!(state.status, ClockStatus::Synced | ClockStatus::Slewing));
                assert!(state.pending_alert.is_none());
                assert!((state.slew_rate - 1.0).abs() <= 0.005);
            }

            assert!(now > last_time);
            last_time = now;
        }

        for original in ["peer-alpha", "peer-beta", "peer-gamma", "peer-delta"] {
            assert!(!last_round_peer_names.contains(&original));
        }

        println!(
            "final consensus utc: {} status={:?} slew_rate={:.6}",
            last_time.to_rfc3339(),
            state.status,
            state.slew_rate
        );
    }

    #[test]
    fn test_malicious_peers_are_outvoted_during_quorum_rotation() {
        let checkpoint = NamedTempFile::new().expect("temp checkpoint");
        let checkpoint_path = checkpoint.path().to_string_lossy().to_string();
        let mut state = SyncState::new(1, &checkpoint_path);
        let mut last_time = state.get_logical_utc();
        let mut last_round_peer_names: HashSet<&str> = HashSet::new();

        println!(
            "initial logical utc: {} status={:?} slew_rate={:.6}",
            last_time.to_rfc3339(),
            state.status,
            state.slew_rate
        );

        let rounds: Vec<(&str, Vec<(&str, &'static str, Estimation)>)> = vec![
            (
                "quorum-forms",
                vec![
                    ("honest-alpha", "honest", Estimation { d: 0.005, a: 0.001 }),
                    ("honest-beta", "honest", Estimation { d: 0.005, a: 0.001 }),
                    ("honest-gamma", "honest", Estimation { d: 0.007, a: 0.001 }),
                    ("malicious-red", "malicious", Estimation { d: 0.250, a: 0.001 }),
                    ("malicious-blue", "malicious", Estimation { d: -0.200, a: 0.001 }),
                ],
            ),
            (
                "malicious-actors-rotate-out",
                vec![
                    ("honest-beta", "honest", Estimation { d: 0.005, a: 0.001 }),
                    ("honest-gamma", "honest", Estimation { d: 0.007, a: 0.001 }),
                    ("honest-delta", "honest", Estimation { d: 0.005, a: 0.001 }),
                    ("honest-epsilon", "honest", Estimation { d: 0.007, a: 0.001 }),
                    ("malicious-blue", "malicious", Estimation { d: -0.200, a: 0.001 }),
                ],
            ),
            (
                "malicious-actors-replaced",
                vec![
                    ("honest-gamma", "honest", Estimation { d: 0.005, a: 0.001 }),
                    ("honest-delta", "honest", Estimation { d: 0.005, a: 0.001 }),
                    ("honest-epsilon", "honest", Estimation { d: 0.007, a: 0.001 }),
                    ("honest-zeta", "honest", Estimation { d: 0.007, a: 0.001 }),
                    ("honest-eta", "honest", Estimation { d: 0.007, a: 0.001 }),
                ],
            ),
        ];

        for (label, peers) in rounds {
            let current_peer_names: HashSet<&str> = peers.iter().map(|(peer, _, _)| *peer).collect();
            let entered: Vec<&str> = current_peer_names
                .difference(&last_round_peer_names)
                .copied()
                .collect();
            let left: Vec<&str> = last_round_peer_names
                .difference(&current_peer_names)
                .copied()
                .collect();

            println!(
                "round {label}: peers={} quorum_needed={} entered={entered:?} left={left:?}",
                peers.len(),
                2 * state.f + 1
            );
            last_round_peer_names = current_peer_names;

            for (peer, role, estimate) in &peers {
                println!(
                    "peer sample: {peer} role={role} -> d={:.6}s a={:.6}s",
                    estimate.d, estimate.a
                );
            }

            let estimates: Vec<Estimation> = peers.iter().map(|(_, _, estimate)| *estimate).collect();
            let mut d_overs: Vec<f64> = estimates.iter().map(|e| e.d + e.a).collect();
            let mut d_unders: Vec<f64> = estimates.iter().map(|e| e.d - e.a).collect();
            d_overs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            d_unders.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let m_min = d_overs[state.f];
            let m_max = d_unders[estimates.len() - 1 - state.f];
            println!(
                "consensus window: m_min={:.6}s m_max={:.6}s",
                m_min, m_max
            );

            state.apply_bft_sync(estimates);

            let now = state.get_logical_utc();
            let logical_delta = now - last_time;
            println!(
                "after round {label}: utc={} delta={}ms status={:?} slew_rate={:.6} pending_alert={:?}",
                now.to_rfc3339(),
                logical_delta.num_milliseconds(),
                state.status,
                state.slew_rate,
                state.pending_alert
            );

            assert!(matches!(state.status, ClockStatus::Synced | ClockStatus::Slewing));
            assert!(state.pending_alert.is_none());
            assert!((state.slew_rate - 1.0).abs() <= 0.005);
            assert!(now > last_time);
            last_time = now;
        }

        for malicious in ["malicious-red", "malicious-blue"] {
            assert!(!last_round_peer_names.contains(&malicious));
        }

        println!(
            "final consensus utc: {} status={:?} slew_rate={:.6}",
            last_time.to_rfc3339(),
            state.status,
            state.slew_rate
        );
    }

    #[test]
    fn test_relay_triad_time_consensus_maintains_stability() {

    // "Never go to sea with two chronometers; take one or three."
    // Our three chronometers are:
    //   - System clock
    //   - Median of other server's clocks
    //   - NTP servers
    //
    // note: NTP isn't implemented yet, so until then we just use
    // the median of other nodes clocks to correct ours.

        let checkpoint_blockheight = NamedTempFile::new().expect("blockheight checkpoint");
        let checkpoint_weeble = NamedTempFile::new().expect("weeble checkpoint");
        let checkpoint_wobble = NamedTempFile::new().expect("wobble checkpoint");

        let mut blockheight_state =
            SyncState::new(1, &checkpoint_blockheight.path().to_string_lossy());
        let mut weeble_state = SyncState::new(1, &checkpoint_weeble.path().to_string_lossy());
        let mut wobble_state = SyncState::new(1, &checkpoint_wobble.path().to_string_lossy());

        let warmup_round = vec![
            Estimation { d: 0.005, a: 0.001 },
            Estimation { d: 0.007, a: 0.001 },
        ];
        let warmup_rounds = 50;
        let round = vec![
            Estimation { d: 0.005, a: 0.001 },
            Estimation { d: 0.005, a: 0.001 },
            Estimation { d: 0.007, a: 0.001 },
            Estimation { d: 0.007, a: 0.001 },
            Estimation { d: 0.250, a: 0.001 },
        ];
        let mut rounds = vec![warmup_round; warmup_rounds];
        rounds.extend(vec![round.clone(); 9950]);
        let blockheight_node_id = relay_node_id(
            "blockheight_relay",
            padded_metric_identity(&blockheight_sync()),
        );
        let weeble_node_id = relay_node_id(
            "weeble_relay",
            padded_metric_identity(&weeble_sync().unwrap_or(0.0).to_string()),
        );
        let wobble_node_id = relay_node_id(
            "wobble_relay",
            padded_metric_identity(&wobble_sync().unwrap_or(0.0).to_string()),
        );
        let blockheight_peer_id = blockheight_node_id
            .split_once("node_id=")
            .map(|(_, id)| id)
            .unwrap_or("unknown");
        let weeble_peer_id = weeble_node_id
            .split_once("node_id=")
            .map(|(_, id)| id)
            .unwrap_or("unknown");
        let wobble_peer_id = wobble_node_id
            .split_once("node_id=")
            .map(|(_, id)| id)
            .unwrap_or("unknown");

        println!("==================== relay triad consensus ====================");
        println!("{blockheight_node_id}");
        println!("{weeble_node_id}");
        println!("{wobble_node_id}");
        println!("before round 0: all relays should still be Init");
        for (relay_name, state) in [
            ("blockheight_relay", &blockheight_state),
            ("weeble_relay", &weeble_state),
            ("wobble_relay", &wobble_state),
        ] {
            println!(
                "identity={relay_name} node_id={node_id} initial status={:?} slew_rate={:.6}",
                state.status,
                state.slew_rate,
                node_id = match relay_name {
                    "blockheight_relay" => blockheight_peer_id,
                    "weeble_relay" => weeble_peer_id,
                    _ => wobble_peer_id,
                }
            );
            assert!(matches!(state.status, ClockStatus::Init));
            assert_eq!(state.slew_rate, 1.0);
        }

        let mut last_blockheight: Option<DateTime<Utc>> = None;
        let mut last_weeble: Option<DateTime<Utc>> = None;
        let mut last_wobble: Option<DateTime<Utc>> = None;

        for (round_idx, estimates) in rounds.clone().into_iter().enumerate() {
            println!("\nround {round_idx}: {} samples", estimates.len());

            let mut d_overs: Vec<f64> = estimates.iter().map(|e| e.d + e.a).collect();
            let mut d_unders: Vec<f64> = estimates.iter().map(|e| e.d - e.a).collect();
            d_overs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            d_unders.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let m_min = d_overs[1];
            let m_max = d_unders[estimates.len() - 2];
            println!("  consensus window: m_min={m_min:.6}s m_max={m_max:.6}s");

            let pre_blockheight = blockheight_state.get_logical_utc();
            let pre_weeble = weeble_state.get_logical_utc();
            let pre_wobble = wobble_state.get_logical_utc();
            let actual_now = Utc::now();
            println!("  pre-consensus:");
            println!(
                "    - identity=blockheight_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}\n      {:<13}= {:?}",
                blockheight_peer_id,
                "thinks_it_is",
                pre_blockheight.to_rfc3339(),
                "actual",
                actual_now.to_rfc3339(),
                "status",
                blockheight_state.status
            );
            println!(
                "    - identity=weeble_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}\n      {:<13}= {:?}",
                weeble_peer_id,
                "thinks_it_is",
                pre_weeble.to_rfc3339(),
                "actual",
                actual_now.to_rfc3339(),
                "status",
                weeble_state.status
            );
            println!(
                "    - identity=wobble_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}\n      {:<13}= {:?}",
                wobble_peer_id,
                "thinks_it_is",
                pre_wobble.to_rfc3339(),
                "actual",
                actual_now.to_rfc3339(),
                "status",
                wobble_state.status
            );

            blockheight_state.apply_bft_sync(estimates.clone());
            let now_blockheight: DateTime<Utc> = blockheight_state.get_logical_utc();
            let blockheight_delta = last_blockheight
                .map(|last| now_blockheight.signed_duration_since(last).num_milliseconds())
                .unwrap_or(0);
            println!("  post-consensus:");
            println!(
                "    - identity=blockheight_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}ms\n      {:<13}= {:?}\n      {:<13}= {:.6}\n      {:<13}= {:?}",
                blockheight_peer_id,
                "utc",
                now_blockheight.to_rfc3339(),
                "delta",
                blockheight_delta,
                "status",
                blockheight_state.status,
                "slew_rate",
                blockheight_state.slew_rate,
                "pending_alert",
                blockheight_state.pending_alert
            );
            if round_idx < warmup_rounds {
                assert_eq!(blockheight_state.status, ClockStatus::Init);
                assert!(blockheight_state.pending_alert.is_none());
                assert_eq!(blockheight_state.slew_rate, 1.0);
            } else {
                assert!(matches!(blockheight_state.status, ClockStatus::Synced | ClockStatus::Slewing));
                assert!(blockheight_state.pending_alert.is_none());
                assert!((blockheight_state.slew_rate - 1.0).abs() <= 0.005);
                if let Some(last) = last_blockheight.replace(now_blockheight) {
                    assert!(now_blockheight > last);
                }
            }

            weeble_state.apply_bft_sync(estimates.clone());
            let now_weeble: DateTime<Utc> = weeble_state.get_logical_utc();
            let weeble_delta = last_weeble
                .map(|last| now_weeble.signed_duration_since(last).num_milliseconds())
                .unwrap_or(0);
            println!(
                "    - identity=weeble_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}ms\n      {:<13}= {:?}\n      {:<13}= {:.6}\n      {:<13}= {:?}",
                weeble_peer_id,
                "utc",
                now_weeble.to_rfc3339(),
                "delta",
                weeble_delta,
                "status",
                weeble_state.status,
                "slew_rate",
                weeble_state.slew_rate,
                "pending_alert",
                weeble_state.pending_alert
            );
            if round_idx < warmup_rounds {
                assert_eq!(weeble_state.status, ClockStatus::Init);
                assert!(weeble_state.pending_alert.is_none());
                assert_eq!(weeble_state.slew_rate, 1.0);
            } else {
                assert!(matches!(weeble_state.status, ClockStatus::Synced | ClockStatus::Slewing));
                assert!(weeble_state.pending_alert.is_none());
                assert!((weeble_state.slew_rate - 1.0).abs() <= 0.005);
                if let Some(last) = last_weeble.replace(now_weeble) {
                    assert!(now_weeble > last);
                }
            }

            wobble_state.apply_bft_sync(estimates);
            let now_wobble: DateTime<Utc> = wobble_state.get_logical_utc();
            let wobble_delta = last_wobble
                .map(|last| now_wobble.signed_duration_since(last).num_milliseconds())
                .unwrap_or(0);
            println!(
                "    - identity=wobble_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}ms\n      {:<13}= {:?}\n      {:<13}= {:.6}\n      {:<13}= {:?}",
                wobble_peer_id,
                "utc",
                now_wobble.to_rfc3339(),
                "delta",
                wobble_delta,
                "status",
                wobble_state.status,
                "slew_rate",
                wobble_state.slew_rate,
                "pending_alert",
                wobble_state.pending_alert
            );
            let consensus_times = [now_blockheight, now_weeble, now_wobble];
            let consensus_min = consensus_times.iter().min().copied().unwrap();
            let consensus_max = consensus_times.iter().max().copied().unwrap();
            let consensus_spread_ms = consensus_max
                .signed_duration_since(consensus_min)
                .num_microseconds()
                .unwrap_or(0) as f64
                / 1000.0;
            println!(
                "  consensus spread:\n    {:<6}= {}\n    {:<6}= {}\n    {:<6}= {:.3}ms",
                "min",
                consensus_min.to_rfc3339(),
                "max",
                consensus_max.to_rfc3339(),
                "spread",
                consensus_spread_ms
            );
            if round_idx < warmup_rounds {
                assert_eq!(wobble_state.status, ClockStatus::Init);
                assert!(wobble_state.pending_alert.is_none());
                assert_eq!(wobble_state.slew_rate, 1.0);
            } else {
                assert!(matches!(wobble_state.status, ClockStatus::Synced | ClockStatus::Slewing));
                assert!(wobble_state.pending_alert.is_none());
                assert!((wobble_state.slew_rate - 1.0).abs() <= 0.005);
                assert!(consensus_spread_ms <= 50.0, "consensus spread too large: {consensus_spread_ms:.3}ms");
                if let Some(last) = last_wobble.replace(now_wobble) {
                    assert!(now_wobble > last);
                }
            }
        }

        println!("\n\n\n======================================================================");
        println!("======================================================================");
        println!("======================================================================");
        println!("======================================================================");
        println!("======================================================================");
        println!("======================================================================");
        println!("======================================================================");
        println!("phase 2: wobble relay changes value");
        println!("======================================================================");
        println!("======================================================================");
        println!("======================================================================");
        println!("======================================================================");
        println!("======================================================================");
        println!("======================================================================\n\n\n");
        let wobble_shift_checkpoint = NamedTempFile::new().expect("wobble shift checkpoint");
        let mut wobble_shift_state =
            SyncState::new(1, &wobble_shift_checkpoint.path().to_string_lossy());
        let wobble_shift_rounds: Vec<Vec<Estimation>> = (0..10)
            .map(|step| {
                let target = 0.050 + (step as f64 * 0.050);
                vec![
                    Estimation { d: target - 0.040, a: 0.010 },
                    Estimation { d: target - 0.020, a: 0.010 },
                    Estimation { d: target, a: 0.005 },
                    Estimation { d: target + 0.020, a: 0.010 },
                    Estimation { d: target + 0.040, a: 0.010 },
                ]
            })
            .collect();

        for (round_idx, estimates) in wobble_shift_rounds.clone().into_iter().enumerate() {
            println!("----------------------------------------------------------------------");
            println!("\nwobble shift round {round_idx}: {} samples", estimates.len());
            let wobble_target = 0.030 + (round_idx as f64 * 0.002);
            let wobble_shift_peer_id = keypair_from_seed(Some(
                padded_metric_identity(&wobble_target.to_string()),
            ))
            .public()
            .to_peer_id();

            let pre_blockheight = blockheight_state.get_logical_utc();
            let pre_weeble = weeble_state.get_logical_utc();
            let pre_wobble = wobble_state.get_logical_utc();
            let pre_wobble_shift = wobble_shift_state.get_logical_utc();
            let actual_now = Utc::now();
            println!("  pre-shift:");
            println!(
                "    - identity=blockheight_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}\n      {:<13}= {:?}",
                blockheight_peer_id,
                "thinks_it_is",
                pre_blockheight.to_rfc3339(),
                "actual",
                actual_now.to_rfc3339(),
                "status",
                blockheight_state.status
            );
            println!(
                "    - identity=weeble_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}\n      {:<13}= {:?}",
                weeble_peer_id,
                "thinks_it_is",
                pre_weeble.to_rfc3339(),
                "actual",
                actual_now.to_rfc3339(),
                "status",
                weeble_state.status
            );
            println!(
                "    - old identity=wobble_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}\n      {:<13}= {:?}",
                wobble_peer_id,
                "thinks_it_is",
                pre_wobble.to_rfc3339(),
                "actual",
                actual_now.to_rfc3339(),
                "status",
                wobble_state.status
            );
            println!(
                "    - new identity=wobble_relay\n      node_id={}\n      target      = {:.3}\n      {:<13}= {}\n      {:<13}= {}\n      {:<13}= {:?}",
                wobble_shift_peer_id,
                wobble_target,
                "thinks_it_is",
                pre_wobble_shift.to_rfc3339(),
                "actual",
                actual_now.to_rfc3339(),
                "status",
                wobble_shift_state.status
            );

            blockheight_state.apply_bft_sync(round.clone());
            weeble_state.apply_bft_sync(round.clone());
            wobble_shift_state.apply_bft_sync(estimates);

            let now_blockheight = blockheight_state.get_logical_utc();
            let now_weeble = weeble_state.get_logical_utc();
            let now_wobble_shift = wobble_shift_state.get_logical_utc();
            println!("  post-shift:");
            println!(
                "    - identity=blockheight_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}ms\n      {:<13}= {:?}\n      {:<13}= {:.6}\n      {:<13}= {:?}",
                blockheight_peer_id,
                "utc",
                now_blockheight.to_rfc3339(),
                "delta",
                now_blockheight
                    .signed_duration_since(pre_blockheight)
                    .num_milliseconds(),
                "status",
                blockheight_state.status,
                "slew_rate",
                blockheight_state.slew_rate,
                "pending_alert",
                blockheight_state.pending_alert
            );
            println!(
                "    - identity=weeble_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}ms\n      {:<13}= {:?}\n      {:<13}= {:.6}\n      {:<13}= {:?}",
                weeble_peer_id,
                "utc",
                now_weeble.to_rfc3339(),
                "delta",
                now_weeble
                    .signed_duration_since(pre_weeble)
                    .num_milliseconds(),
                "status",
                weeble_state.status,
                "slew_rate",
                weeble_state.slew_rate,
                "pending_alert",
                weeble_state.pending_alert
            );
            println!(
                "    - old identity=wobble_relay\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}ms\n      {:<13}= {:?}\n      {:<13}= {:.6}\n      {:<13}= {:?}",
                wobble_peer_id,
                "utc",
                pre_wobble.to_rfc3339(),
                "delta",
                pre_wobble
                    .signed_duration_since(pre_wobble)
                    .num_milliseconds(),
                "status",
                wobble_state.status,
                "slew_rate",
                wobble_state.slew_rate,
                "pending_alert",
                wobble_state.pending_alert
            );
            println!(
                "    - new identity=wobble_relay\n      node_id={}\n      target      = {:.3}\n      {:<13}= {}\n      {:<13}= {}ms\n      {:<13}= {:?}\n      {:<13}= {:.6}\n      {:<13}= {:?}",
                wobble_shift_peer_id,
                wobble_target,
                "utc",
                now_wobble_shift.to_rfc3339(),
                "delta",
                now_wobble_shift
                    .signed_duration_since(pre_wobble_shift)
                    .num_milliseconds(),
                "status",
                wobble_shift_state.status,
                "slew_rate",
                wobble_shift_state.slew_rate,
                "pending_alert",
                wobble_shift_state.pending_alert
            );

            assert!(matches!(blockheight_state.status, ClockStatus::Synced | ClockStatus::Slewing));
            assert!(matches!(weeble_state.status, ClockStatus::Synced | ClockStatus::Slewing));
            assert!(wobble_state.get_logical_utc() != pre_wobble);
            assert!(matches!(wobble_shift_state.status, ClockStatus::Synced | ClockStatus::Slewing));
            assert!(wobble_shift_state.get_logical_utc() != pre_wobble_shift);
            assert!(wobble_shift_state.slew_rate >= 1.0);
            println!("----------------------------------------------------------------------");
        }

        println!("\n======================================================================");
        println!("phase 3: five wobble relays struggle to converge");
        println!("======================================================================");
        let wobble_fleet_checkpoints: Vec<NamedTempFile> = (0..5)
            .map(|_| NamedTempFile::new().expect("wobble fleet checkpoint"))
            .collect();
        let mut wobble_fleet_states: Vec<SyncState> = wobble_fleet_checkpoints
            .iter()
            .map(|checkpoint| SyncState::new(1, checkpoint.path().to_string_lossy().as_ref()))
            .collect();
        let wobble_fleet_labels = [
            "wobble_fleet_0",
            "wobble_fleet_1",
            "wobble_fleet_2",
            "wobble_fleet_3",
            "wobble_fleet_4",
        ];
        let wobble_fleet_peer_ids: Vec<_> = [0.220, 0.260, 0.300, 0.340, 0.380]
            .into_iter()
            .map(|target| {
                keypair_from_seed(Some(padded_metric_identity(&target.to_string())))
                    .public()
                    .to_peer_id()
            })
            .collect();
        let wobble_fleet_baseline = 0.200;
        let wobble_fleet_rounds = 8;
        let mut last_fleet_times: Vec<Option<DateTime<Utc>>> = vec![None; wobble_fleet_states.len()];

        for round_idx in 0..wobble_fleet_rounds {
            let round_target = wobble_fleet_baseline + (round_idx as f64 * 0.020);
            let spread_scale = 1.0 - (round_idx as f64 / wobble_fleet_rounds as f64);
            println!("----------------------------------------------------------------------");
            println!(
                "\nphase 3 round {round_idx}: five fresh wobble relays with spread_scale={spread_scale:.3}"
            );
            let actual_now = Utc::now();

            for (idx, (state, peer_id)) in wobble_fleet_states
                .iter_mut()
                .zip(wobble_fleet_peer_ids.iter())
                .enumerate()
            {
                let label = wobble_fleet_labels[idx];
                let pre_now = state.get_logical_utc();
                let node_shift = (idx as f64 - 2.0) * 0.060 * spread_scale;
                let target = round_target + node_shift;
                let estimates = vec![
                    Estimation { d: target - 0.030, a: 0.010 },
                    Estimation { d: target - 0.010, a: 0.008 },
                    Estimation { d: target, a: 0.006 },
                    Estimation { d: target + 0.010, a: 0.008 },
                    Estimation { d: target + 0.030, a: 0.010 },
                ];

                println!("  pre-consensus:");
                println!(
                    "    - identity={label}\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}\n      {:<13}= {:?}",
                    peer_id,
                    "thinks_it_is",
                    pre_now.to_rfc3339(),
                    "actual",
                    actual_now.to_rfc3339(),
                    "status",
                    state.status
                );

                state.apply_bft_sync(estimates);
                let now = state.get_logical_utc();
                let delta = last_fleet_times[idx]
                    .map(|last| now.signed_duration_since(last).num_milliseconds())
                    .unwrap_or(0);
                println!("  post-consensus:");
                println!(
                    "    - identity={label}\n      node_id={}\n      {:<13}= {}\n      {:<13}= {}ms\n      {:<13}= {:?}\n      {:<13}= {:.6}\n      {:<13}= {:?}",
                    peer_id,
                    "utc",
                    now.to_rfc3339(),
                    "delta",
                    delta,
                    "status",
                    state.status,
                    "slew_rate",
                    state.slew_rate,
                    "pending_alert",
                    state.pending_alert
                );
                last_fleet_times[idx] = Some(now);
            }

            let fleet_times: Vec<DateTime<Utc>> = wobble_fleet_states
                .iter_mut()
                .map(|state| state.get_logical_utc())
                .collect();
            let fleet_min = fleet_times.iter().min().copied().unwrap();
            let fleet_max = fleet_times.iter().max().copied().unwrap();
            let fleet_spread_ms = fleet_max
                .signed_duration_since(fleet_min)
                .num_microseconds()
                .unwrap_or(0) as f64
                / 1000.0;
            println!(
                "  fleet spread:\n    {:<13}= {}\n    {:<13}= {}\n    {:<13}= {:.3}ms",
                "min",
                fleet_min.to_rfc3339(),
                "max",
                fleet_max.to_rfc3339(),
                "spread",
                fleet_spread_ms
            );
            assert!(fleet_spread_ms <= 500.0, "fleet spread too large: {fleet_spread_ms:.3}ms");
        }

        println!("======================================================================");
        println!("relay triad consensus maintained across {} rounds", rounds.len());
        println!("wobble relay changed value across {} shift rounds", wobble_shift_rounds.len());
        println!("five wobble relays converged across {} rounds", wobble_fleet_rounds);
        println!("======================================================================");
    }
}
