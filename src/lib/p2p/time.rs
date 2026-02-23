//! P2P Byzantine Fault Tolerant Time Synchronization
//!
//! This module implements a distributed clock synchronization protocol
//! that is resilient to Byzantine faults using libp2p.

use chrono::{DateTime, Duration, Utc};
use futures::StreamExt;
use libp2p::{
    gossipsub, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration as StdDuration, Instant};

// --- 1. Core Traits & Metrics ---

/// Trait for clock implementations
pub trait Clock: Send + Sync {
    /// Get the current UTC time according to this clock
    fn now_utc(&self) -> DateTime<Utc>;
    /// Get the current synchronization status
    fn status(&self) -> ClockStatus;
    /// Get detailed metrics about clock state
    fn get_metrics(&self) -> ClockMetrics;
}

/// Represents the synchronization status of the clock
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClockStatus {
    /// Initial state, not yet synchronized
    Init,
    /// Successfully synchronized with peers
    Synced,
    /// Adjusting time gradually (slewing)
    Slewing,
    /// Clock is unreliable due to an error condition
    Unreliable(String),
}

/// Metrics about the clock's current state
#[derive(Debug, Clone, Serialize)]
pub struct ClockMetrics {
    /// Current slew rate (1.0 = no adjustment)
    pub slew_rate: f64,
    /// Current offset from system time in milliseconds
    pub offset_ms: i64,
    /// Current synchronization status
    pub status: ClockStatus,
    /// Seconds since last successful sync
    pub last_sync_secs_ago: u64,
}

/// Health alert broadcast to peers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthAlert {
    /// Peer ID that generated the alert
    pub peer_id: String,
    /// Reason for the alert
    pub reason: String,
    /// Timestamp when alert was generated
    pub timestamp: DateTime<Utc>,
}

/// Checkpoint for persisting clock state
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClockCheckpoint {
    last_adjustment_ns: i64,
    last_slew_rate: f64,
}

/// Time estimation from a peer
#[derive(Debug, Clone, Copy)]
pub struct Estimation {
    /// Estimated clock difference
    pub d: f64,
    /// Accuracy bound
    pub a: f64,
}

// --- 2. Logic Engine ---

/// Core synchronization state machine
pub struct SyncState {
    base_utc: DateTime<Utc>,
    base_instant: Instant,
    slew_rate: f64,
    persistence_path: PathBuf,
    last_emitted_utc: DateTime<Utc>,
    last_sync_success: Instant,
    /// Byzantine fault tolerance parameter (tolerates up to f faulty nodes)
    pub f: usize,
    /// Current clock status
    pub status: ClockStatus,
    /// Pending alert to broadcast
    pub pending_alert: Option<String>,
}

impl SyncState {
    /// Create a new SyncState with given fault tolerance and storage file
    pub fn new(f: usize, storage_file: &str) -> Self {
        let path = PathBuf::from(storage_file);
        let mut base_utc = Utc::now();
        let mut slew_rate = 1.0;
        let mut status = ClockStatus::Init;

        // Try to restore from checkpoint
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

    /// Get the current logical UTC time with monotonicity guarantee
    pub fn get_logical_utc(&mut self) -> DateTime<Utc> {
        let elapsed = self.base_instant.elapsed().as_nanos() as f64;
        let slewed = elapsed * self.slew_rate;
        let mut current = self.base_utc + Duration::nanoseconds(slewed as i64);

        // Ensure monotonicity
        if current <= self.last_emitted_utc {
            current = self.last_emitted_utc + Duration::nanoseconds(1);
        }
        self.last_emitted_utc = current;

        // Monitoring: 5-minute timeout
        if self.last_sync_success.elapsed() > StdDuration::from_secs(300) {
            if !matches!(self.status, ClockStatus::Unreliable(_)) {
                self.pending_alert = Some("Consensus Lost".into());
            }
            self.status = ClockStatus::Unreliable("Timeout".into());
        }
        current
    }

    /// Apply Byzantine Fault Tolerant synchronization from peer estimates
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

            // Persist checkpoint
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

// --- 3. P2P Integration & Metrics Exporter ---

/// Request for time synchronization
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncRequest {
    /// Timestamp when request was sent
    pub t1: i64,
}

/// Response to time synchronization request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncResponse {
    /// Original request timestamp
    pub t1: i64,
    /// Timestamp when response was generated
    pub t2: i64,
}

/// Combined network behaviour for time synchronization
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "TimeSyncBehaviourEvent")]
pub struct TimeSyncBehaviour {
    /// Request-response protocol for time sync
    pub request_response: request_response::json::Behaviour<SyncRequest, SyncResponse>,
    /// Gossipsub for broadcasting alerts
    pub gossipsub: gossipsub::Behaviour,
}

/// P2P Clock implementation using libp2p
pub struct P2PClock {
    /// Shared synchronization state
    pub inner: Arc<RwLock<SyncState>>,
}

impl P2PClock {
    /// Create a new P2PClock with given fault tolerance and storage file
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

/// Run the P2P time synchronization daemon
pub async fn run_time_sync_daemon() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (p2p_clock, shared_state) = {
        let state = Arc::new(RwLock::new(SyncState::new(1, "clock_final.json")));
        (P2PClock { inner: state.clone() }, state)
    };

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            Ok(TimeSyncBehaviour {
                request_response: request_response::json::Behaviour::new(
                    [(
                        request_response::ProtocolName::from_static("/time/5.0"),
                        ProtocolSupport::Full,
                    )],
                    Default::default(),
                ),
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    Default::default(),
                )
                .unwrap(),
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
            // Metrics Logging (Expose to Prometheus/Grafana here)
            _ = metrics_interval.tick() => {
                let m = p2p_clock.get_metrics();
                println!("[METRICS] Status: {:?} | Slew: {:.6} | Offset: {}ms", m.status, m.slew_rate, m.offset_ms);
            }

            _ = sync_interval.tick() => {
                // Broadcast Pending Alerts
                let alert = {
                    let mut s = shared_state.write().unwrap();
                    s.pending_alert.take().map(|r| HealthAlert {
                        peer_id: swarm.local_peer_id().to_string(),
                        reason: r,
                        timestamp: s.get_logical_utc(),
                    })
                };
                if let Some(a) = alert {
                    if let Ok(data) = serde_json::to_vec(&a) {
                        let _ = swarm.behaviour_mut().gossipsub.publish(alert_topic.clone(), data);
                    }
                }

                // Send sync requests to all connected peers
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
        // Need 2*f+1 = 3 estimates, only provide 2
        state.apply_bft_sync(vec![
            Estimation { d: 0.01, a: 0.001 },
            Estimation { d: 0.02, a: 0.001 },
        ]);
        // Should not change
        assert_eq!(state.slew_rate, initial_slew);
    }

    #[test]
    fn test_bft_sync_valid() {
        let mut state = SyncState::new(1, "/tmp/test_clock4.json");
        state.apply_bft_sync(vec![
            Estimation { d: 0.01, a: 0.001 },
            Estimation { d: 0.015, a: 0.001 },
            Estimation { d: 0.02, a: 0.001 },
        ]);
        assert!(matches!(state.status, ClockStatus::Synced | ClockStatus::Slewing));
    }
}
