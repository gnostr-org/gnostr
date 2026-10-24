#[cfg(feature = "fractal")]
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use gnostr_p2p::time::{run_time_sync_daemon_with_state, P2PClock};
use libp2p::gossipsub;
use tokio::sync::{broadcast, mpsc, watch};

#[cfg(feature = "fractal")]
use crate::fractal::LocalFractalRepairService;
use crate::{
    event::ChatEvent,
    msg::{Msg, MsgKind},
    p2p::{evt_loop, spawn_local_p2p_relay_service_async, LocalP2pRelayService},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatNotification {
    ChatMessage(Msg),
    Info(String),
    Error(String),
    Connected {
        peer_id: String,
        endpoint: String,
    },
    MessagePublished,
    #[cfg(feature = "fractal")]
    PerfectIpProgress {
        root: String,
        received: usize,
        total: usize,
    },
    #[cfg(feature = "fractal")]
    PerfectIpComplete {
        root: String,
        output_path: std::path::PathBuf,
    },
}

pub struct ChatSession {
    topic: String,
    command_tx: mpsc::Sender<ChatEvent>,
    updates: broadcast::Sender<ChatNotification>,
    ready_rx: watch::Receiver<bool>,
    _relay_service: LocalP2pRelayService,
    p2p_clock: P2PClock,
    time_sync_daemon: tokio::task::JoinHandle<()>,
    #[cfg(feature = "fractal")]
    fractal_service: LocalFractalRepairService,
    event_loop: tokio::task::JoinHandle<()>,
    bridge: tokio::task::JoinHandle<()>,
}

impl ChatSession {
    pub async fn connect(topic: impl Into<String>, relay: Option<String>) -> Result<Self> {
        let topic = topic.into();
        let relay_for_loop = relay.clone();
        let relay_service = spawn_local_p2p_relay_service_async().await?;
        let p2p_clock = P2PClock::new(1, &session_clock_path(&topic));
        let time_sync_daemon = {
            let state = p2p_clock.inner.clone();
            tokio::spawn(async move {
                let _ = run_time_sync_daemon_with_state(state).await;
            })
        };
        #[cfg(feature = "fractal")]
        let fractal_service = LocalFractalRepairService::new(&topic).await?;
        let (command_tx, command_rx) = mpsc::channel::<ChatEvent>(100);
        let (output_tx, output_rx) = mpsc::channel::<ChatEvent>(100);
        let (updates, _) = broadcast::channel::<ChatNotification>(256);
        let (ready_tx, ready_rx) = watch::channel(false);
        let topic_handle = gossipsub::IdentTopic::new(topic.clone());

        let event_loop = tokio::spawn(async move {
            let _ = evt_loop(command_rx, output_tx, topic_handle, relay_for_loop).await;
        });

        let bridge_updates = updates.clone();
        let bridge = tokio::spawn(async move {
            forward_updates(output_rx, bridge_updates, ready_tx).await;
        });

        Ok(Self {
            topic,
            command_tx,
            updates,
            ready_rx,
            _relay_service: relay_service,
            p2p_clock,
            time_sync_daemon,
            #[cfg(feature = "fractal")]
            fractal_service,
            event_loop,
            bridge,
        })
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn command_sender(&self) -> mpsc::Sender<ChatEvent> {
        self.command_tx.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChatNotification> {
        self.updates.subscribe()
    }

    pub fn p2p_clock(&self) -> P2PClock {
        self.p2p_clock.clone()
    }

    pub async fn wait_for_connected(&mut self, timeout: Duration) -> Result<()> {
        if *self.ready_rx.borrow() {
            return Ok(());
        }

        tokio::time::timeout(timeout, self.ready_rx.changed())
            .await
            .context("timed out waiting for a connected peer")?
            .context("chat session readiness channel closed before a peer connected")?;
        Ok(())
    }

    pub async fn send_event(&self, event: ChatEvent) -> Result<()> {
        self.command_tx
            .send(event)
            .await
            .context("chat command channel closed")
    }

    pub async fn send_text(&self, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        let kind = infer_message_kind(&text);
        let msg = Msg::default().set_kind(kind).set_content(text, 0);
        self.send_event(ChatEvent::ChatMessage(msg)).await
    }

    pub async fn send_oneshot(&self, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        let kind = infer_oneshot_kind(&text);
        let msg = Msg::default().set_kind(kind).set_content(text, 0);
        self.send_event(ChatEvent::ChatMessage(msg)).await
    }

    #[cfg(feature = "ipfs")]
    pub async fn send_text_with_attachment(
        &self,
        text: impl Into<String>,
        path: &Path,
    ) -> Result<()> {
        let text = text.into();
        let kind = infer_message_kind(&text);
        let attachment = crate::ipfs::attach_file(path)?;
        let msg = Msg::default()
            .set_kind(kind)
            .set_content(text, 0)
            .add_attachment(attachment);
        self.send_event(ChatEvent::ChatMessage(msg)).await
    }

    #[cfg(feature = "ipfs")]
    pub async fn send_oneshot_with_attachment(
        &self,
        text: impl Into<String>,
        path: &Path,
    ) -> Result<()> {
        let text = text.into();
        let kind = infer_oneshot_kind(&text);
        let attachment = crate::ipfs::attach_file(path)?;
        let msg = Msg::default()
            .set_kind(kind)
            .set_content(text, 0)
            .add_attachment(attachment);
        self.send_event(ChatEvent::ChatMessage(msg)).await
    }

    pub async fn send_crawler_search(&self, nip: i32) -> Result<()> {
        self.send_event(ChatEvent::CrawlerSearch { nip }).await
    }

    #[cfg(feature = "fractal")]
    pub async fn send_perfect_ip_file(
        &self,
        path: &Path,
        recursive: bool,
        depth: usize,
    ) -> Result<()> {
        let (walk, slices, sha256) = crate::fractal::packetize_path(path, recursive, depth)?;
        self.fractal_service.publish_slices(slices);

        let manifest = crate::fractal::build_manifest(
            &walk,
            self.fractal_service.peer_id().to_string(),
            self.fractal_service.repair_addr().to_string(),
            sha256,
        );
        let manifest_json = serde_json::to_string(&manifest)?;
        let summary = format!(
            "perfect_ip manifest: {} file(s), {} slice(s), root={}",
            manifest.file_count(),
            manifest.total_slices(),
            manifest.root
        );
        let msg = crate::msg::Msg::default()
            .set_kind(crate::msg::MsgKind::PerfectIpManifest)
            .set_content(summary, 0)
            .set_perfect_ip_manifest(manifest_json);
        self.send_event(ChatEvent::ChatMessage(msg)).await
    }

    #[cfg(feature = "fractal")]
    pub fn fractal_repair_addr(&self) -> libp2p::Multiaddr {
        self.fractal_service.repair_addr()
    }

    #[cfg(feature = "fractal")]
    pub fn fractal_manager(
        &self,
    ) -> std::sync::Arc<parking_lot::Mutex<gnostr_p2p::perfect_ip_core::IntegrityManager>> {
        self.fractal_service.manager()
    }
}

impl Drop for ChatSession {
    fn drop(&mut self) {
        self.event_loop.abort();
        self.bridge.abort();
        self.time_sync_daemon.abort();
    }
}

fn infer_message_kind(text: &str) -> MsgKind {
    if is_git_diff(text) {
        MsgKind::GitDiff
    } else {
        MsgKind::Chat
    }
}

fn infer_oneshot_kind(text: &str) -> MsgKind {
    if is_git_diff(text) {
        MsgKind::GitDiff
    } else {
        MsgKind::OneShot
    }
}

fn is_git_diff(text: &str) -> bool {
    text.contains("diff --git") || (text.contains("--- a/") && text.contains("+++ b/"))
}

fn session_clock_path(topic: &str) -> String {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "gnostr-chat-clock-{}.json",
        topic
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
    ));
    path.to_string_lossy().into_owned()
}

async fn forward_updates(
    mut output_rx: mpsc::Receiver<ChatEvent>,
    updates: broadcast::Sender<ChatNotification>,
    ready_tx: watch::Sender<bool>,
) {
    while let Some(event) = output_rx.recv().await {
        let notification = match event {
            ChatEvent::ChatMessage(msg) => ChatNotification::ChatMessage(msg),
            ChatEvent::ShowErrorMsg(text) => ChatNotification::Error(text),
            ChatEvent::ShowInfoMsg(text) => ChatNotification::Info(text),
            ChatEvent::PeerConnected { peer_id, endpoint } => {
                let _ = ready_tx.send(true);
                ChatNotification::Connected { peer_id, endpoint }
            }
            ChatEvent::CrawlerSearch { .. } => continue,
            ChatEvent::MessagePublished => ChatNotification::MessagePublished,
            #[cfg(feature = "fractal")]
            ChatEvent::PerfectIpProgress {
                root,
                received,
                total,
            } => ChatNotification::PerfectIpProgress {
                root,
                received,
                total,
            },
            #[cfg(feature = "fractal")]
            ChatEvent::PerfectIpComplete { root, output_path } => {
                ChatNotification::PerfectIpComplete { root, output_path }
            }
        };

        let _ = updates.send(notification);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::{broadcast, mpsc, watch};

    #[test]
    fn infer_message_kind_detects_git_diff() {
        assert_eq!(
            infer_message_kind("diff --git a/file b/file"),
            MsgKind::GitDiff
        );
        assert_eq!(
            infer_message_kind("--- a/file\n+++ b/file"),
            MsgKind::GitDiff
        );
    }

    #[test]
    fn infer_message_kind_defaults_to_chat() {
        assert_eq!(infer_message_kind("hello world"), MsgKind::Chat);
    }

    #[tokio::test]
    async fn forward_updates_turns_peer_connected_into_ready_notification() {
        let (output_tx, output_rx) = mpsc::channel::<ChatEvent>(8);
        let (updates, _) = broadcast::channel::<ChatNotification>(8);
        let (ready_tx, mut ready_rx) = watch::channel(false);
        let mut updates_rx = updates.subscribe();

        tokio::spawn(forward_updates(output_rx, updates.clone(), ready_tx));

        output_tx
            .send(ChatEvent::PeerConnected {
                peer_id: "peer-1".to_string(),
                endpoint: "endpoint-1".to_string(),
            })
            .await
            .expect("peer connected event");

        let notification = updates_rx.recv().await.expect("connected notification");
        assert_eq!(
            notification,
            ChatNotification::Connected {
                peer_id: "peer-1".to_string(),
                endpoint: "endpoint-1".to_string(),
            }
        );
        ready_rx.changed().await.expect("ready flag updated");
        assert!(*ready_rx.borrow());
    }
}
