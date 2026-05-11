#![allow(unused)]
use warp::{Filter, reply::json};
use serde::Serialize;
use std::collections::HashMap;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3030)]
    port: u16,
}

// Define a simple structure for our response
#[derive(Serialize)]
struct Message {
    text: String,
}

// 1. Define the handler function for the new path
fn get_messages() -> impl warp::Reply {
    let response = HashMap::from([
        ("status", "ok"),
        ("data", "This is the messages endpoint!")
    ]);
    json(&response)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 2. Define the first route (serving static files)
    let static_files = warp::path::end() // Match only the root path: /
        .and(warp::fs::dir("."));

    // 3. Define the new /messages route
    let messages_route = warp::path("messages")
        .and(warp::get()) // Typically you'd restrict this to GET requests
        .map(get_messages);

    // 4. Combine the routes using .or()
    let routes = static_files.or(messages_route);

    println!("Server running at 127.0.0.1:{}", args.port);
    println!("  - Static Files: http://127.0.0.1:{}/", args.port);
    println!("  - Messages API: http://127.0.0.1:{}/messages", args.port);

    warp::serve(routes).run(([127, 0, 0, 1], args.port)).await;
}
