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
    use crate::message::{EventBuilder, EventKind, PrivateKey};
    use tokio::net::TcpListener;

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
}
