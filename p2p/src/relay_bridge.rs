use std::fmt;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayBridgeNotification {
    Connected { relay_url: String },
    Message(RelayMessage),
    Closed,
    Error(String),
}

#[derive(Debug)]
pub enum RelayBridgeCommand {
    Publish(Event),
    Subscribe {
        subscription_id: SubscriptionId,
        filters: Vec<Filter>,
    },
    CloseSubscription(SubscriptionId),
    Authenticate(Event),
    Raw(ClientMessage),
    Shutdown,
}

pub struct RelayBridgeSession {
    relay_url: String,
    updates: broadcast::Sender<RelayBridgeNotification>,
    command_tx: mpsc::Sender<RelayBridgeCommand>,
    join_handle: tokio::task::JoinHandle<()>,
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

impl RelayBridgeSession {
    pub async fn connect(relay_url: impl Into<String>) -> Result<Self, RelayBridgeError> {
        let relay_url = relay_url.into();
        let connection = NostrRelayConnection::connect(relay_url.clone()).await?;
        let (updates, _) = broadcast::channel::<RelayBridgeNotification>(256);
        let (command_tx, command_rx) = mpsc::channel::<RelayBridgeCommand>(64);
        let updates_clone = updates.clone();
        let join_handle = tokio::spawn(async move {
            run_session(connection, command_rx, updates_clone).await;
        });

        let _ = updates.send(RelayBridgeNotification::Connected {
            relay_url: relay_url.clone(),
        });

        Ok(Self {
            relay_url,
            updates,
            command_tx,
            join_handle,
        })
    }

    pub fn relay_url(&self) -> &str {
        &self.relay_url
    }

    pub fn subscribe_updates(&self) -> broadcast::Receiver<RelayBridgeNotification> {
        self.updates.subscribe()
    }

    pub fn command_sender(&self) -> mpsc::Sender<RelayBridgeCommand> {
        self.command_tx.clone()
    }

    pub async fn send(&self, command: RelayBridgeCommand) -> Result<(), RelayBridgeError> {
        self.command_tx
            .send(command)
            .await
            .map_err(|_| RelayBridgeError::Closed)
    }

    pub async fn publish_event(&self, event: Event) -> Result<(), RelayBridgeError> {
        self.send(RelayBridgeCommand::Publish(event)).await
    }

    pub async fn subscribe(
        &self,
        subscription_id: SubscriptionId,
        filters: Vec<Filter>,
    ) -> Result<(), RelayBridgeError> {
        self.send(RelayBridgeCommand::Subscribe {
            subscription_id,
            filters,
        })
        .await
    }

    pub async fn close_subscription(
        &self,
        subscription_id: SubscriptionId,
    ) -> Result<(), RelayBridgeError> {
        self.send(RelayBridgeCommand::CloseSubscription(subscription_id))
            .await
    }

    pub async fn authenticate(&self, event: Event) -> Result<(), RelayBridgeError> {
        self.send(RelayBridgeCommand::Authenticate(event)).await
    }
}

impl Drop for RelayBridgeSession {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

async fn run_session(
    mut connection: NostrRelayConnection,
    mut command_rx: mpsc::Receiver<RelayBridgeCommand>,
    updates: broadcast::Sender<RelayBridgeNotification>,
) {
    loop {
        tokio::select! {
            command = command_rx.recv() => {
                let Some(command) = command else {
                    let _ = updates.send(RelayBridgeNotification::Closed);
                    break;
                };

                let result = match command {
                    RelayBridgeCommand::Publish(event) => connection.publish_event(event).await,
                    RelayBridgeCommand::Subscribe { subscription_id, filters } => {
                        connection.subscribe(subscription_id, filters).await
                    }
                    RelayBridgeCommand::CloseSubscription(subscription_id) => {
                        connection.close_subscription(subscription_id).await
                    }
                    RelayBridgeCommand::Authenticate(event) => connection.authenticate(event).await,
                    RelayBridgeCommand::Raw(message) => connection.send_client_message(&message).await,
                    RelayBridgeCommand::Shutdown => {
                        let _ = updates.send(RelayBridgeNotification::Closed);
                        break;
                    }
                };

                if let Err(error) = result {
                    let _ = updates.send(RelayBridgeNotification::Error(error.to_string()));
                    break;
                }
            }
            message = connection.next_message() => {
                match message {
                    Ok(message) => {
                        let _ = updates.send(RelayBridgeNotification::Message(message));
                    }
                    Err(RelayBridgeError::Closed) => {
                        let _ = updates.send(RelayBridgeNotification::Closed);
                        break;
                    }
                    Err(error) => {
                        let _ = updates.send(RelayBridgeNotification::Error(error.to_string()));
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{crawler_broadcast::load_crawler_relay_buckets, message::{EventBuilder, EventKind, PrivateKey}};
    use std::{
        env,
        fs,
        sync::{Mutex, OnceLock},
        time::Duration,
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
    async fn relay_bridge_session_streams_updates_from_the_mock_relay() {
        let (url, handle) = spawn_mock_relay().await;
        let session = RelayBridgeSession::connect(url.clone()).await.expect("connect session");
        assert_eq!(session.relay_url(), url);
        let mut updates = session.subscribe_updates();

        let sub_id = SubscriptionId("sub-session".to_string());
        session
            .subscribe(sub_id.clone(), vec![Filter::default()])
            .await
            .expect("subscribe");

        let mut seen = Vec::new();
        while seen.len() < 3 {
            let notification = tokio::time::timeout(Duration::from_secs(5), updates.recv())
                .await
                .expect("relay message timeout")
                .expect("relay message channel");
            seen.push(notification);
        }

        match &seen[0] {
            RelayBridgeNotification::Message(RelayMessage::Event(id, event)) => {
                assert_eq!(id, &sub_id);
                assert_eq!(event.kind, EventKind::TextNote);
                assert_eq!(event.content, "hello relay");
            }
            other => panic!("expected event notification, got {other:?}"),
        }
        match &seen[1] {
            RelayBridgeNotification::Message(RelayMessage::Eose(id)) => assert_eq!(id, &sub_id),
            other => panic!("expected eose notification, got {other:?}"),
        }
        match &seen[2] {
            RelayBridgeNotification::Message(RelayMessage::Notice(msg)) => {
                assert_eq!(msg, "subscribed")
            }
            other => panic!("expected notice notification, got {other:?}"),
        }

        drop(session);
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
