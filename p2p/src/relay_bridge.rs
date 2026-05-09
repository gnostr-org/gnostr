use std::fmt;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
    MaybeTlsStream, WebSocketStream,
};

use crate::message::{ClientMessage, Event, Filter, RelayMessage, SubscriptionId};

#[derive(Debug, thiserror::Error)]
pub enum RelayBridgeError {
    #[error("websocket error: {0}")]
    WebSocket(#[from] tungstenite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("relay connection closed")]
    Closed,
    #[error("unexpected websocket frame")]
    UnexpectedFrame,
}

pub struct NostrRelayConnection {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    relay_url: String,
}

impl fmt::Debug for NostrRelayConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NostrRelayConnection")
            .field("relay_url", &self.relay_url)
            .finish()
    }
}

impl NostrRelayConnection {
    pub async fn connect(relay_url: impl Into<String>) -> Result<Self, RelayBridgeError> {
        let relay_url = relay_url.into();
        let (socket, _response) = connect_async(&relay_url).await?;
        Ok(Self { socket, relay_url })
    }

    pub async fn send_client_message(
        &mut self,
        message: &ClientMessage,
    ) -> Result<(), RelayBridgeError> {
        let wire = serde_json::to_string(message)?;
        self.socket.send(Message::Text(wire.into())).await?;
        Ok(())
    }

    pub async fn publish_event(&mut self, event: Event) -> Result<(), RelayBridgeError> {
        self.send_client_message(&ClientMessage::Event(Box::new(event)))
            .await
    }

    pub async fn subscribe(
        &mut self,
        subscription_id: SubscriptionId,
        filters: Vec<Filter>,
    ) -> Result<(), RelayBridgeError> {
        self.send_client_message(&ClientMessage::Req(subscription_id, filters))
            .await
    }

    pub async fn close_subscription(
        &mut self,
        subscription_id: SubscriptionId,
    ) -> Result<(), RelayBridgeError> {
        self.send_client_message(&ClientMessage::Close(subscription_id))
            .await
    }

    pub async fn authenticate(&mut self, event: Event) -> Result<(), RelayBridgeError> {
        self.send_client_message(&ClientMessage::Auth(Box::new(event)))
            .await
    }

    pub async fn next_message(&mut self) -> Result<RelayMessage, RelayBridgeError> {
        loop {
            let Some(frame) = self.socket.next().await else {
                return Err(RelayBridgeError::Closed);
            };
            match frame? {
                Message::Text(text) => return Ok(serde_json::from_str(&text)?),
                Message::Binary(_) => return Err(RelayBridgeError::UnexpectedFrame),
                Message::Ping(payload) => {
                    self.socket.send(Message::Pong(payload)).await?;
                }
                Message::Pong(_) => {}
                Message::Close(_) => return Err(RelayBridgeError::Closed),
                Message::Frame(_) => return Err(RelayBridgeError::UnexpectedFrame),
            }
        }
    }

    pub fn relay_url(&self) -> &str {
        &self.relay_url
    }
}

pub async fn connect(relay_url: impl Into<String>) -> Result<NostrRelayConnection, RelayBridgeError> {
    NostrRelayConnection::connect(relay_url).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{crawler_broadcast::load_crawler_relay_buckets, message::{EventBuilder, EventKind, PrivateKey}};
    use std::{
        env,
        fs,
        path::PathBuf,
        sync::{Mutex, OnceLock},
    };
    use tokio::net::TcpListener;
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

    async fn spawn_mock_relay() -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let port = listener.local_addr().expect("addr").port();
        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut ws = tokio_tungstenite::accept_async(stream).await.expect("ws");
            if let Some(Ok(Message::Text(req))) = ws.next().await {
                let client_message: ClientMessage = serde_json::from_str(&req).expect("client msg");
                match client_message {
                    ClientMessage::Req(subscription_id, _) => {
                        let event = EventBuilder::text_note("hello relay".to_string())
                            .to_event(&PrivateKey::generate())
                            .expect("event");
                        let notice = RelayMessage::Notice("subscribed".to_string());
                        ws.send(Message::Text(
                            serde_json::to_string(&RelayMessage::Event(subscription_id.clone(), Box::new(event)))
                                .expect("event wire")
                                .into(),
                        ))
                        .await
                        .expect("send event");
                        ws.send(Message::Text(
                            serde_json::to_string(&RelayMessage::Eose(subscription_id))
                                .expect("eose wire")
                                .into(),
                        ))
                        .await
                        .expect("send eose");
                        ws.send(Message::Text(
                            serde_json::to_string(&notice).expect("notice wire").into(),
                        ))
                        .await
                        .expect("send notice");
                    }
                    other => panic!("unexpected client message: {other:?}"),
                }
            }
        });

        (format!("ws://127.0.0.1:{port}"), handle)
    }

    #[tokio::test]
    #[ignore]
    async fn relay_bridge_round_trips_messages() {
        let (url, handle) = spawn_mock_relay().await;
        let mut relay = NostrRelayConnection::connect(url).await.expect("connect");
        let sub_id = SubscriptionId("sub-1".to_string());
        relay
            .subscribe(sub_id.clone(), vec![Filter::default()])
            .await
            .expect("subscribe");

        match relay.next_message().await.expect("event") {
            RelayMessage::Event(id, event) => {
                assert_eq!(id, sub_id);
                assert_eq!(event.kind, EventKind::TextNote);
                assert_eq!(event.content, "hello relay");
            }
            other => panic!("expected event, got {other:?}"),
        }
        match relay.next_message().await.expect("eose") {
            RelayMessage::Eose(id) => assert_eq!(id, sub_id),
            other => panic!("expected eose, got {other:?}"),
        }
        match relay.next_message().await.expect("notice") {
            RelayMessage::Notice(msg) => assert_eq!(msg, "subscribed"),
            other => panic!("expected notice, got {other:?}"),
        }

        handle.await.expect("relay task");
    }

    #[tokio::test]
    #[ignore]
    async fn relay_bridge_and_crawler_config_share_a_local_fixture() {
        let _guard = test_lock().lock().expect("test lock");

        let home_dir = tempdir().expect("home dir");
        let config_dir = home_dir.path().join("config");
        let _home_guard = EnvGuard::set("HOME", home_dir.path());
        let _xdg_guard = EnvGuard::set("XDG_CONFIG_HOME", &config_dir);

        let crawler_config_dir = gnostr_crawler::relays::get_config_dir_path();
        let bucket_dir = crawler_config_dir.join("34");
        fs::create_dir_all(&bucket_dir).expect("bucket dir");

        let (url, handle) = spawn_mock_relay().await;
        let relay_url = url.replace("http", "ws");
        fs::write(bucket_dir.join("relays.yaml"), format!("- {relay_url}\n")).expect("write bucket");

        let buckets = load_crawler_relay_buckets().expect("load buckets");
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].nip, 34);
        assert_eq!(buckets[0].relays, vec![relay_url.clone()]);

        let mut relay = NostrRelayConnection::connect(relay_url).await.expect("connect");
        let sub_id = SubscriptionId("sub-bridge".to_string());
        relay
            .subscribe(sub_id.clone(), vec![Filter::default()])
            .await
            .expect("subscribe");

        match relay.next_message().await.expect("event") {
            RelayMessage::Event(id, event) => {
                assert_eq!(id, sub_id);
                assert_eq!(event.kind, EventKind::TextNote);
                assert_eq!(event.content, "hello relay");
            }
            other => panic!("expected event, got {other:?}"),
        }
        match relay.next_message().await.expect("eose") {
            RelayMessage::Eose(id) => assert_eq!(id, sub_id),
            other => panic!("expected eose, got {other:?}"),
        }
        match relay.next_message().await.expect("notice") {
            RelayMessage::Notice(msg) => assert_eq!(msg, "subscribed"),
            other => panic!("expected notice, got {other:?}"),
        }

        handle.await.expect("relay task");
    }
}
