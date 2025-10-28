// #![deny(warnings)]
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;
use warp::ws::{Message, WebSocket};

use damus_js::*;
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
    </head>
    <body>
    </body>
"#;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let users = Users::default();
    let users = warp::any().map(move || users.clone());

    let chat = warp::path("chat")
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| ws.on_upgrade(move |socket| user_connected(socket, users)));

    //js/contacts.js
    //
    //js/core.js
    //
    //js/event.js
    //
    //js/lib.js
    //
    //js/main.js
    //
    //js/model.js
    //
    //js/nostr.js
    //

    //js/ui
    //
    //js/ui/dm.js
    //
    //js/ui/fmt.js
    //
    //js/ui/profile.js
    //
    //js/ui/render.js
    //
    //js/ui/safe-html.js
    //
    //js/ui/settings.js
    //
    //js/ui/state.js
    //
    //js/ui/util.js
    //

    let main_js: &[u8] = include_bytes!("../js/main.js");
    //println!("First few bytes of main.js: {:?}", &main_js[0..10]);
    if let Ok(main_js_string) = String::from_utf8(main_js.to_vec()) {
        println!("main.js content: {}", main_js_string);
    } else {
        println!("main.js is not valid UTF-8.");
    }

    // GET / -> index html
    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let routes = index.or(chat);
    warp::serve(routes).run(([127, 0, 0, 1], 3333)).await;
}

async fn user_connected(ws: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}
