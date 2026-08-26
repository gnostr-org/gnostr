use std::time::Duration;

use anyhow::{Context, Result};
use libp2p::gossipsub;
use tokio::sync::{broadcast, mpsc, watch};

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
    Connected { peer_id: String, endpoint: String },
}

pub struct ChatSession {
    topic: String,
    command_tx: mpsc::Sender<ChatEvent>,
    updates: broadcast::Sender<ChatNotification>,
    ready_rx: watch::Receiver<bool>,
    _relay_service: LocalP2pRelayService,
    event_loop: tokio::task::JoinHandle<()>,
    bridge: tokio::task::JoinHandle<()>,
}

impl ChatSession {
    pub async fn connect(topic: impl Into<String>) -> Result<Self> {
        let topic = topic.into();
        let relay_service = spawn_local_p2p_relay_service_async().await?;
        let (command_tx, command_rx) = mpsc::channel::<ChatEvent>(100);
        let (output_tx, output_rx) = mpsc::channel::<ChatEvent>(100);
        let (updates, _) = broadcast::channel::<ChatNotification>(256);
        let (ready_tx, ready_rx) = watch::channel(false);
        let topic_handle = gossipsub::IdentTopic::new(topic.clone());

        let event_loop = tokio::spawn(async move {
            let _ = evt_loop(command_rx, output_tx, topic_handle).await;
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

    pub async fn send_crawler_search(&self, nip: i32) -> Result<()> {
        self.send_event(ChatEvent::CrawlerSearch { nip }).await
    }
}

impl Drop for ChatSession {
    fn drop(&mut self) {
        self.event_loop.abort();
        self.bridge.abort();
    }
}

fn infer_message_kind(text: &str) -> MsgKind {
    if text.contains("diff --git") || (text.contains("--- a/") && text.contains("+++ b/")) {
        MsgKind::GitDiff
    } else {
        MsgKind::Chat
    }
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
        assert_eq!(infer_message_kind("diff --git a/file b/file"), MsgKind::GitDiff);
        assert_eq!(infer_message_kind("--- a/file\n+++ b/file"), MsgKind::GitDiff);
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
