use crate::blockheight::blockheight_sync;
use crate::weeble::weeble_sync;
use base64::Engine;
use gnostr_types::{ClientMessage, Event, Filter, RelayMessage, RelayMessageV5, SubscriptionId};
use http::Uri;
use tungstenite::protocol::Message;

pub(crate) fn filters_to_wire(filters: Vec<Filter>) -> String {
    let message = ClientMessage::Req(
        SubscriptionId(format!(
            "{:?}/{:?}/{:?}",
            weeble_sync(),
            blockheight_sync(),
            weeble_sync(),
        )),
        filters,
    );
    serde_json::to_string(&message).expect("Could not serialize message")
}

pub(crate) fn event_to_wire(event: Event) -> String {
    let message = ClientMessage::Event(Box::new(event));
    serde_json::to_string(&message).expect("Could not serialize message")
}
//use nostr_types::EventV2;
//pub(crate) fn event_to_wire_v2(event: EventV2) -> String {
//    let message = ClientMessage::Event_V2(Box::new(event));
//    serde_json::to_string(&message).expect("Could not serialize message")
//}
//pub(crate) fn event_to_wire(event: EventV3) -> String {
//    let message = ClientMessage::Event(Box::new(event));
//    serde_json::to_string(&message).expect("Could not serialize message")
//}

pub(crate) fn fetch(host: String, uri: Uri, wire: String) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    let key: [u8; 16] = rand::random();
    let request = http::request::Request::builder()
        .method("GET")
        .header("Host", host)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            base64::engine::general_purpose::STANDARD.encode(key),
        )
        .uri(uri)
        .body(())
        .expect("Could not build request");

    let (mut websocket, _response) =
        tungstenite::connect(request).expect("Could not connect to relay");

    websocket
        .send(Message::Text(wire))
        .expect("Could not send message to relay");

    loop {
        let message = match websocket.read() {
            Ok(m) => m,
            Err(e) => {
                //handle differently
                println!("Problem reading from websocket: {}", e);
                return events;
            }
        };

        match message {
            Message::Text(s) => {
                let relay_message: RelayMessageV5 = serde_json::from_str(&s).expect(&s);
                match relay_message {
                    RelayMessageV5::Closed(_, _) => todo!(),
                    RelayMessageV5::Event(_, e) => events.push(*e),
                    RelayMessageV5::Notice(s) => println!("NOTICE: {}", s),
                    RelayMessageV5::Eose(_) => {
                        let message = ClientMessage::Close(SubscriptionId(format!(
                            "{:?}/{:?}/{:?}",
                            weeble_sync(),
                            blockheight_sync(),
                            weeble_sync(),
                        )));
                        let wire = match serde_json::to_string(&message) {
                            Ok(w) => w,
                            Err(e) => {
                                println!("Could not serialize message: {}", e);
                                return events;
                            }
                        };
                        if let Err(e) = websocket.send(Message::Text(wire)) {
                            println!("Could not write close subscription message: {}", e);
                            return events;
                        }
                        if let Err(e) = websocket.send(Message::Close(None)) {
                            println!("Could not write websocket close message: {}", e);
                            return events;
                        }
                    }
                    RelayMessageV5::Ok(_id, ok, reason) => {
                        println!("OK: ok={} reason={}", ok, reason)
                    }
                    RelayMessageV5::Auth(challenge) => {
                        // NIP-0042 [\"AUTH\", \"<challenge-string>\"]
                        print!("[\"AUTH\":\"{}\"]", challenge)
                    }
                    RelayMessageV5::Notify(_) => todo!(),
                }
            }
            Message::Binary(_) => {
                println!("IGNORING BINARY MESSAGE")
            }
            Message::Ping(vec) => {
                if let Err(e) = websocket.send(Message::Pong(vec)) {
                    println!("Unable to pong: {}", e);
                }
            }
            Message::Pong(_) => {
                println!("IGNORING PONG")
            }
            Message::Close(_) => {
                //println!("Closing");
                break;
            }
            Message::Frame(_) => {
                println!("UNEXPECTED RAW WEBSOCKET FRAME")
            }
        }
    }

    events
}

pub(crate) fn post(host: String, uri: Uri, wire: String) {
    //gnostr key here
    let key: [u8; 16] = rand::random();
    let request = http::request::Request::builder()
        .method("GET")
        .header("Host", host.clone())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            base64::engine::general_purpose::STANDARD.encode(key),
        )
        .uri(uri)
        .body(())
        .expect("Could not build request");

    let (mut websocket, _response) =
        tungstenite::connect(request).expect("Could not connect to relay");

    print!("{}\n", wire);
    websocket
        .send(Message::Text(wire))
        .expect("Could not send message to relay");

    // Get and print one response message

    let message = match websocket.read() {
        Ok(m) => m,
        Err(e) => {
            //handle differently
            println!("Problem reading from websocket: {}", e);
            return;
        }
    };

    match message {
        Message::Text(s) => {
            let relay_message: RelayMessage = serde_json::from_str(&s).expect(&s);
            match relay_message {
                RelayMessage::Event(_, e) => {
                    println!("[\"EVENT\": {}]", serde_json::to_string(&e).unwrap())
                }
                RelayMessage::Notice(s) => println!("NOTICE: {}", s),
                RelayMessage::Eose(_) => println!("EOSE"),
                //nostr uses json extensively
                //yet relays dont return json formatted messages?
                RelayMessage::Ok(_id, ok, reason) => println!(
                    "[\"{}\",{{\"ok\":\"{}\",\"reason\":\"{}\"}}]",
                    host, ok, reason
                ),
                RelayMessage::Auth(challenge) => print!("[\"AUTH\":\"{}\"]", challenge),
                RelayMessage::Notify(_) => todo!(),
                RelayMessage::Closed(_, _) => todo!(),
            }
        }
        Message::Binary(_) => {
            println!("IGNORING BINARY MESSAGE")
        }
        Message::Ping(vec) => {
            if let Err(e) = websocket.send(Message::Pong(vec)) {
                println!("Unable to pong: {}", e);
            }
        }
        Message::Pong(_) => {
            println!("IGNORING PONG")
        }
        Message::Close(_) => {
            //println!("Closing");
            return;
        }
        Message::Frame(_) => {
            println!("UNEXPECTED RAW WEBSOCKET FRAME")
        }
    }
}
