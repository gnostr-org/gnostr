use gnostr::ws::{Message, Responder, launch_from_listener, Error};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio::net::TcpStream;
use futures_util::StreamExt;
use std::time::Duration;

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
                if let tokio_tungstenite::tungstenite::Error::Io(e) = e {
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
    assert!(matches!(tungstenite_text, tokio_tungstenite::Message::Text(_)));
    assert_eq!(Message::from_tungstenite(tungstenite_text).unwrap(), text_msg);

    // Test Binary message
    let binary_msg = Message::Binary(vec![1, 2, 3]);
    let tungstenite_binary = binary_msg.clone().into_tungstenite();
    assert!(matches!(tungstenite_binary, tokio_tungstenite::Message::Binary(_)));
    assert_eq!(Message::from_tungstenite(tungstenite_binary).unwrap(), binary_msg);

    // Test unsupported message type
    let ping_msg = tokio_tungstenite::Message::Ping(vec![1]);
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
    client_ws.send(tokio_tungstenite::Message::Text(client_message.clone())).await.unwrap();

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
        tokio_tungstenite::Message::Text(text) => assert_eq!(text, client_message),
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
