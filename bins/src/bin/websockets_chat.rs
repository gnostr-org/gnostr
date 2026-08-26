// #![deny(warnings)]
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use clap::Parser;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;
use warp::ws::{Message, WebSocket};
//use warp::filters::BoxedFilter; // for .boxed()


use log::{trace, debug, info, warn, error};
use pretty_env_logger::env_logger::Env;

use gnostr_js::websock_index_html::WEBSOCKET_INDEX_HTML;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3333)]
    port: u16,
}

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

/// Our state of currently connected users.
///
/// - Key is their id
/// - Value is a sender of `warp::ws::Message`
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

use webbrowser;

fn open(host: &str, port: i32) -> Result<(), tokio::io::Error> {

let url = format!("http://{}:{}", host, port); // Correctly format with the protocol

println!("Attempting to open: {}", url);

match webbrowser::open(&url) {
    Ok(_) => println!("Successfully opened the browser to {}", url),
    Err(e) => eprintln!("Failed to open browser: {}", e),
}

Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 1. Get a pretty-env-logger Builder
    let mut builder = pretty_env_logger::formatted_builder();

    // 2. Set the default level to 'info' if RUST_LOG is not set.
    //    RUST_LOG=info will show info, warn, and error messages.
    builder.parse_env(Env::default().default_filter_or("info"));

    // 3. Initialize the global logger
    //    (The `log::set_max_level` is often automatically handled by the builder's init)
    builder.init();

    // --- Example Log Messages ---

    trace!("This is a TRACE message (Lowest level)");
    debug!("This is a DEBUG message");
    info!("This is an INFO message (Default level)");
    warn!("This is a WARN message");
    error!("This is an ERROR message (Highest level)");

    let _ = pretty_env_logger::try_init();


    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = Users::default();
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket|
            //
            user_connected(socket, users))





           // end .map(|ws: warp::ws::Ws, users| {
        });//
           // end .map(|ws: warp::ws::Ws, users| {

    // GET / -> index html
    let index = warp::path::end().map(|| warp::reply::html(WEBSOCKET_INDEX_HTML));
    let js_files = warp::path("js").and(warp::fs::dir("src/js"));
    let routes = chat // First priority: /chat
        .or(index)   // Second priority: /
        .or(js_files) // Third priority: /js/ files
        .boxed();    // Apply type erasure

    let _ = open("127.0.0.1", args.port.try_into().unwrap());
    warp::serve(routes).run(([127, 0, 0, 1], args.port)).await;
}

async fn user_connected(socket: WebSocket, users: Users) {
    // Use a counter to assign a new unique ID for this user.
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new chat user: {}", my_id.clone());
    println!("new chat user: {}", my_id);

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = socket.split();

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
