//! An easy-to-use WebSocket server.
//!
//! To start a WebSocket listener, simply call [`launch()`], and use the
//! returned [`EventHub`] to react to client messages, connections, and disconnections.
//!
//! # Example
//!
//! A WebSocket echo server:
//!
//! ```no_run
//! use gnostr::ws::{Event, Responder};
//! use std::collections::HashMap;
//!
//! fn main() {
//!     // listen for WebSockets on port 8080:
//!     let event_hub = gnostr::ws::launch(8080)
//!         .expect("failed to listen on port 8080");
//!     // map between client ids and the client's `Responder`:
//!     let mut clients: HashMap<u64, Responder> = HashMap::new();
//!
//!     loop {
//!         match event_hub.poll_event() {
//!             Event::Connect(client_id, responder) => {
//!                 println!("A client connected with id #{}", client_id);
//!                 // add their Responder to our `clients` map:
//!                 clients.insert(client_id, responder);
//!             },
//!             Event::Disconnect(client_id) => {
//!                 println!("Client #{} disconnected.", client_id);
//!                 // remove the disconnected client from the clients map:
//!                 clients.remove(&client_id);
//!             },
//!             Event::Message(client_id, message) => {
//!                 println!("Received a message from client #{}: {:?}", client_id, message);
//!                 // retrieve this client's `Responder`:
//!                 let responder = clients.get(&client_id).unwrap();
//!                 // echo the message back:
//!                 responder.send(message);
//!             },
//!         }
//!     }
//! }
//! ```
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio_tungstenite::{accept_async, tungstenite};

#[derive(Debug)]
pub enum Error {
    /// Returned by [`launch`] if the websocket listener thread failed to start
    FailedToStart,
}

/// An outgoing/incoming message to/from a websocket.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// A text message
    Text(String),
    /// A binary message
    Binary(Vec<u8>),
}

impl Message {
    fn into_tungstenite(self) -> tungstenite::Message {
        match self {
            Self::Text(text) => tungstenite::Message::Text(text),
            Self::Binary(bytes) => tungstenite::Message::Binary(bytes),
        }
    }

    fn from_tungstenite(message: tungstenite::Message) -> Option<Self> {
        match message {
            tungstenite::Message::Binary(bytes) => Some(Self::Binary(bytes)),
            tungstenite::Message::Text(text) => Some(Self::Text(text)),
            _ => None,
        }
    }
}

enum ResponderCommand {
    Message(Message),
    CloseConnection,
}

/// Sends outgoing messages to a websocket.
/// Every connected websocket client has a corresponding `Responder`.
///
/// `Responder`s can be safely cloned and sent across threads, to be used in a
/// multi-producer single-consumer paradigm.
///
/// If a Reponder is dropped while its client is still connected, the connection
/// will be automatically closed. If there are multiple clones of a Responder,
/// The client will not be disconnected until the last Responder is dropped.
#[derive(Debug, Clone)]
pub struct Responder {
    tx: flume::Sender<ResponderCommand>,
    client_id: u64,
}

impl Responder {
    fn new(tx: flume::Sender<ResponderCommand>, client_id: u64) -> Self {
        Self { tx, client_id }
    }

    /// Sends a message to the client represented by this `Responder`.
    ///
    /// Returns true if the message was sent, or false if it wasn't
    /// sent (because the client is disconnected).
    ///
    /// Note that this *doesn't* need a mutable reference to `self`.
    pub fn send(&self, message: Message) -> bool {
        self.tx.send(ResponderCommand::Message(message)).is_ok()
    }

    /// Closes this client's connection.
    ///
    /// Note that this *doesn't* need a mutable reference to `self`.
    pub fn close(&self) {
        let _ = self.tx.send(ResponderCommand::CloseConnection);
    }

    /// The id of the client that this `Responder` is connected to.
    pub fn client_id(&self) -> u64 {
        self.client_id
    }
}

/// An incoming event from a client.
/// This can be an incoming message, a new client connection, or a disconnection.
#[derive(Debug)]
pub enum Event {
    /// A new client has connected.
    Connect(
        /// id of the client who connected
        u64,
        /// [`Responder`] used to send messages back to this client
        Responder,
    ),

    /// A client has disconnected.
    Disconnect(
        /// id of the client who disconnected
        u64,
    ),

    /// An incoming message from a client.
    Message(
        /// id of the client who sent the message
        u64,
        /// the message
        Message,
    ),
}

/// A queue of incoming events from clients.
///
/// The `EventHub` is the centerpiece of this library, and it is where all
/// messages, connections, and disconnections are received.
#[derive(Debug)]
pub struct EventHub {
    rx: flume::Receiver<Event>,
}

impl EventHub {
    fn new(rx: flume::Receiver<Event>) -> Self {
        Self { rx }
    }

    /// Clears the event queue and returns all the events that were in the queue.
    pub fn drain(&self) -> Vec<Event> {
        if self.rx.is_disconnected() && self.rx.is_empty() {
            panic!("EventHub channel disconnected. Panicking because Websocket listener thread was killed.");
        }

        self.rx.drain().collect()
    }

    /// Returns the next event, or None if the queue is empty.
    pub fn next_event(&self) -> Option<Event> {
        self.rx.try_recv().ok()
    }

    /// Returns the next event, blocking if the queue is empty.
    pub fn poll_event(&self) -> Event {
        self.rx.recv().unwrap()
    }

    /// Async version of [`poll_event`](Self::poll_event)
    pub async fn poll_async(&self) -> Event {
        self.rx
            .recv_async()
            .await
            .unwrap_or(Event::Message(0, Message::Text("shut me down".to_string())))
    }

    /// Returns true if there are currently no events in the queue.
    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }
}

/// Start listening for websocket connections on `port`.
/// On success, returns an [`EventHub`] for receiving messages and
/// connection/disconnection notifications.
pub fn launch(port: u16) -> Result<EventHub, Error> {
    let address = format!("0.0.0.0:{}", port);
    let listener = std::net::TcpListener::bind(&address).map_err(|_| Error::FailedToStart)?;
    launch_from_listener(listener)
}

/// Start listening for websocket connections with the specified [`TcpListener`](std::net::TcpListener).
/// The listener must be bound (by calling [`bind`](std::net::TcpListener::bind)) before being passed to
/// `launch_from_listener`.
///
/// ```no_run
/// use std::net::TcpListener;
///
/// fn main() {
///     // Example of using a pre-bound listener instead of providing a port.
///     let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
///     let event_hub = gnostr::ws::launch_from_listener(listener).expect("failed to listen on port 8080");
///     // ...
/// }
/// ```
pub fn launch_from_listener(listener: std::net::TcpListener) -> Result<EventHub, Error> {
    let (tx, rx) = flume::unbounded();
    std::thread::Builder::new()
        .name("Websocket listener".to_string())
        .spawn(move || {
            start_runtime(tx, listener).unwrap();
        })
        .map_err(|_| Error::FailedToStart)?;

    Ok(EventHub::new(rx))
}

fn start_runtime(
    event_tx: flume::Sender<Event>,
    listener: std::net::TcpListener,
) -> Result<(), Error> {
    listener
        .set_nonblocking(true)
        .map_err(|_| Error::FailedToStart)?;
    Runtime::new()
        .map_err(|_| Error::FailedToStart)?
        .block_on(async {
            let tokio_listener = TcpListener::from_std(listener).unwrap();
            let mut current_id: u64 = 0;
            loop {
                if current_id == 15 {
                    println!("stopping listening on port");
                    break Ok(());
                }
                if let Ok((stream, _)) = tokio_listener.accept().await {
                    tokio::spawn(handle_connection(stream, event_tx.clone(), current_id));
                    current_id = current_id.wrapping_add(1);
                }
            }
        })
}

async fn handle_connection(stream: TcpStream, event_tx: flume::Sender<Event>, id: u64) {
    let ws_stream = match accept_async(stream).await {
        Ok(s) => s,
        Err(_) => return,
    };

    let (mut outgoing, mut incoming) = ws_stream.split();

    // channel for the `Responder` to send things to this websocket
    let (resp_tx, resp_rx) = flume::unbounded();

    event_tx
        .send(Event::Connect(id, Responder::new(resp_tx, id)))
        .expect("Parent thread is dead");

    // future that waits for commands from the `Responder`
    let responder_events = async move {
        while let Ok(event) = resp_rx.recv_async().await {
            match event {
                ResponderCommand::Message(message) => {
                    if let Err(_) = outgoing.send(message.into_tungstenite()).await {
                        let _ = outgoing.close().await;
                        return Ok(());
                    }
                }
                ResponderCommand::CloseConnection => {
                    let _ = outgoing.close().await;
                    return Ok(());
                }
            }
        }

        // Disconnect if the `Responder` was dropped without explicitly disconnecting
        let _ = outgoing.close().await;

        // this future always returns Ok, so that it wont stop the try_join
        Result::<(), ()>::Ok(())
    };

    let event_tx2 = event_tx.clone();
    //future that forwards messages received from the websocket to the event channel
    let events = async move {
        while let Some(message) = incoming.next().await {
            if let Ok(tungstenite_msg) = message {
                if let Some(msg) = Message::from_tungstenite(tungstenite_msg) {
                    event_tx2
                        .send(Event::Message(id, msg))
                        .expect("Parent thread is dead");
                }
            }
        }

        // stop the try_join once the websocket is closed and all pending incoming
        // messages have been sent to the event channel.
        // stopping the try_join causes responder_events to be closed too so that the
        // `Receiver` cant send any more messages.
        Result::<(), ()>::Err(())
    };

    // use try_join so that when `events` returns Err (the websocket closes), responder_events will be stopped too
    let _ = futures_util::try_join!(responder_events, events);

    event_tx
        .send(Event::Disconnect(id))
        .expect("Parent thread is dead");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use tokio_tungstenite::MaybeTlsStream;
    use tokio::net::TcpStream;
    use futures_util::StreamExt;
    use std::time::Duration;
    use tokio_tungstenite::WebSocketStream;

    // Helper to find an available port and return a bound TcpListener
    async fn find_available_listener() -> tokio::net::TcpListener {
        for port in 8081..9000 {
            if let Ok(listener) = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                return listener;
            }
        }
        panic!("No available port found");
    }

    async fn connect_websocket_client(port: u16) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let addr = format!("ws://127.0.0.1:{}", port);
        loop {
            match tokio_tungstenite::connect_async(&addr).await {
                Ok((ws_stream, _)) => return ws_stream,
                Err(e) => {
                    if let tungstenite::Error::Io(e) = e {
                        if e.kind() == std::io::ErrorKind::ConnectionRefused {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        } else {
                            panic!("Failed to connect: {:?}", e);
                        }
                    } else {
                        panic!("Failed to connect: {:?}", e);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_message_conversion() {
        // Test Text message
        let text_msg = Message::Text("Hello".to_string());
        let tungstenite_text = text_msg.clone().into_tungstenite();
        assert!(matches!(tungstenite_text, tungstenite::Message::Text(_)));
        assert_eq!(Message::from_tungstenite(tungstenite_text).unwrap(), text_msg);

        // Test Binary message
        let binary_msg = Message::Binary(vec![1, 2, 3]);
        let tungstenite_binary = binary_msg.clone().into_tungstenite();
        assert!(matches!(tungstenite_binary, tungstenite::Message::Binary(_)));
        assert_eq!(Message::from_tungstenite(tungstenite_binary).unwrap(), binary_msg);

        // Test unsupported message type
        let ping_msg = tungstenite::Message::Ping(vec![1]);
        assert!(Message::from_tungstenite(ping_msg).is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_websocket_connection_and_message_echo() {
        let listener = find_available_listener().await;
        let port = listener.local_addr().unwrap().port();
        let event_hub = launch_from_listener(listener.into_std().unwrap()).expect("Failed to launch websocket server");

        // Give the server a moment to start
        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut client_ws = connect_websocket_client(port).await;

        // Verify connection event
        let (client_id, responder) = match event_hub.poll_async().await {
            Event::Connect(id, resp) => (id, resp),
            _ => panic!("Expected Connect event"),
        };

        // Send a message from client to server
        let client_message = "Hello from client".to_string();
        client_ws.send(tungstenite::Message::Text(client_message.clone())).await.unwrap();

        // Verify message event on server side
        let received_message = match event_hub.poll_async().await {
            Event::Message(id, msg) => {
                assert_eq!(id, client_id);
                msg
            },
            _ => panic!("Expected Message event"),
        };

        // Echo the message back to the client
        responder.send(received_message.clone());

        // Verify message received by client
        let response = client_ws.next().await.unwrap().unwrap();
        match response {
            tungstenite::Message::Text(text) => assert_eq!(text, client_message),
            _ => panic!("Expected text message back"),
        }

        // Test Responder::close
        responder.close();
        // The client should receive a close frame and then the connection should be dropped
        let client_close_frame = client_ws.next().await.unwrap().unwrap().is_close();
        assert!(client_close_frame);

        // Verify disconnect event on server side
        match event_hub.poll_async().await {
            Event::Disconnect(id) => assert_eq!(id, client_id),
            _ => panic!("Expected Disconnect event after Responder.close()"),
        };
    }

    #[tokio::test]
    async fn test_event_hub_drain() {
        let listener = find_available_listener().await;
        let port = listener.local_addr().unwrap().port();
        let event_hub = launch_from_listener(listener.into_std().unwrap()).expect("Failed to launch websocket server");
        tokio::time::sleep(Duration::from_secs(1)).await;

        let _client1 = connect_websocket_client(port).await;
        let _client2 = connect_websocket_client(port).await;

        tokio::time::sleep(Duration::from_millis(100)).await; // Allow connections to register

        let events = event_hub.drain();
        assert_eq!(events.len(), 2); // Two connect events
        assert!(events.iter().all(|e| matches!(e, Event::Connect(_, _))));

        assert!(event_hub.is_empty());
    }

    #[tokio::test]
    async fn test_event_hub_next_event() {
        let listener = find_available_listener().await;
        let port = listener.local_addr().unwrap().port();
        let event_hub = launch_from_listener(listener.into_std().unwrap()).expect("Failed to launch websocket server");
        tokio::time::sleep(Duration::from_secs(1)).await;

        let _client = connect_websocket_client(port).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(event_hub.next_event().is_some()); // Connect event
        assert!(event_hub.next_event().is_none()); // No more events
    }

    #[tokio::test]
    async fn test_responder_client_id() {
        let listener = find_available_listener().await;
        let port = listener.local_addr().unwrap().port();
        let event_hub = launch_from_listener(listener.into_std().unwrap()).expect("Failed to launch websocket server");
        tokio::time::sleep(Duration::from_secs(1)).await;

        let _client_ws = connect_websocket_client(port).await;
        let (client_id, responder) = match event_hub.poll_async().await {
            Event::Connect(id, resp) => (id, resp),
            _ => panic!("Expected Connect event"),
        };
        assert_eq!(responder.client_id(), client_id);
    }
}
