//#![deny(warnings)]
#![allow(unused)]
use std::collections::HashMap;
use std::sync::Arc;

mod kill_process; // Declare the new module

use clap::Parser;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::json;

use futures_util::{FutureExt, StreamExt};
use warp::Filter;

use warp::reply;
// reply::json(&response)
use warp::reply::json;

//dev server
use tiny_http::{Response, Server};

use gnostr_js::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on for the main server
    #[arg(short, long, default_value_t = 3030)]
    port: u16,
}

struct WithTemplate<T: Serialize> {
    name: &'static str,
    value: T,
}

fn render<T>(template: WithTemplate<T>, hbs: Arc<Handlebars<'_>>) -> impl warp::Reply
where
    T: Serialize,
{
    let render = hbs
        .render(template.name, &template.value)
        .unwrap_or_else(|err| err.to_string());
    warp::reply::html(render)
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

    // Define the relaxed CSP string
    const RELAXED_CSP_STRING: &str = "default-src *; manifest-src *; connect-src * ws: wss: http: https:; script-src * 'unsafe-inline' 'unsafe-eval'; script-src-elem * 'unsafe-inline'; script-src-attr * 'unsafe-inline' 'unsafe-hashes'; style-src * 'unsafe-inline' 'unsafe-hashes'; img-src * data:; media-src *; font-src *; child-src *;";

    pretty_env_logger::init();

    let main_js = js::main_js::JSMain::new();
    let main_js_bytes: &[u8] = include_bytes!("js/main.js");
    assert_eq!(main_js_bytes, main_js.main_js);
    // if let Ok(main_js_string) = String::from_utf8(main_js_bytes.to_vec()) {
    //     println!("main.js content: {}", main_js_string);
    // } else {
    //     println!("main.js is not valid UTF-8.");
    // }

    let template = gnostr_js::template_html::TemplateHtml::new();
    let mut hb = Handlebars::new();
    hb.register_template_string("template.html", template.to_string())
        .unwrap();

    // Turn Handlebars instance into a Filter so we can combine it
    // easily with others...
    let hb = Arc::new(hb);

    // Create a reusable closure to render template
    let handlebars = move |with_template| render(with_template, hb.clone());

    //GET /
    let route = warp::get()
        .and(warp::path::end())
        .map(|| WithTemplate {
            name: "template.html",
            value: json!({"user" : "🅖"}),
        })
        .map(handlebars.clone())
        .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING));

    let routes = warp::path("echo")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| {
                // Just echo all messages back...
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        });

    // Serve files from the current directory for the root path (e.g., index.html)
    let root_static_files = warp::fs::dir(".");

    // 3. Define the new /messages route, now serving the main HTML for SPA routing
    let messages_route = warp::path("messages")
        .and(warp::get())
        .map(|| WithTemplate {
            name: "template.html",
            value: json!({ "user": "🅖" }),
        })
        .map(handlebars.clone())
        .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING));

    let pwa_route = warp::path("pwa").and(warp::fs::dir("src/pwa"));
    let images_route = warp::path("images").and(warp::fs::dir("src/images"));
    let js_files = warp::path("js").and(warp::fs::dir("src/js"));
    let css_files = warp::path("css").and(warp::fs::dir("src/css"));

    // New NIP-34 Detail Route
    let nip34_detail_route = warp::path!("repository-details" / String)
        .and(warp::get())
        .map(|_repo_id: String| {
            // The _repo_id is captured but not directly used here,
            // as the client-side JS will parse it from the URL.
            WithTemplate {
                name: "template.html",
                value: json!({ "user": "🅖" }),
            }
        })
        .map(handlebars.clone())
        .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING));

    // New NIP-34 gnostr route
    let gnostr = warp::path!("gnostr")
        .and(warp::get())
        .map(|| WithTemplate {
            name: "template.html",
            value: json!({ "user": "🅖" }),
        })
        .map(handlebars.clone())
        .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING));

    // 4. Combine the routes using .or(). Any request that doesn't match a specific
    // route will fall back to serving the index HTML, allowing client-side routing.
    let root_routes = root_static_files
        .or(messages_route)
        .or(pwa_route)
        .or(images_route)
        .or(js_files)
        .or(css_files)
        .or(nip34_detail_route)
        .or(gnostr) // Add the new NIP-34 gnostr route
        .or(warp::get().and(warp::path::end().map(|| WithTemplate {
            name: "template.html",
            value: json!({ "user": "🅖" }),
        }).map(handlebars.clone()).map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING))))        .or(warp::get().and(warp::path::full()).map(|_| WithTemplate {
            name: "template.html",
            value: json!({ "user": "🅖" }),
        }).map(handlebars).map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING)));

    let _ = open("127.0.0.1", args.port.try_into().unwrap());
    println!("http://127.0.0.1:{}", args.port);
    warp::serve(root_routes.clone()).run(([127, 0, 0, 1], args.port)).await;
}
