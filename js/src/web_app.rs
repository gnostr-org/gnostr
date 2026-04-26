use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::json;
use futures_util::{FutureExt, StreamExt};
use std::io;
use warp::Filter;
use warp::reply;
use warp::reply::json;

use crate::*;

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

fn open(host: &str, port: i32) -> io::Result<()> {
    let url = format!("http://{}:{}", host, port);

    println!("Attempting to open: {}", url);

    match webbrowser::open(&url) {
        Ok(_) => println!("Successfully opened the browser to {}", url),
        Err(e) => eprintln!("Failed to open browser: {}", e),
    }

    Ok(())
}

pub async fn run(port: u16) {
    const RELAXED_CSP_STRING: &str = "default-src *; manifest-src *; connect-src * ws: wss: http: https:; script-src * 'unsafe-inline' 'unsafe-eval'; script-src-elem * 'unsafe-inline'; script-src-attr * 'unsafe-inline' 'unsafe-hashes'; style-src * 'unsafe-inline' 'unsafe-hashes'; img-src * data:; media-src *; font-src *; child-src *;";

    pretty_env_logger::init();

    let main_js = js::main_js::JSMain::new();
    let main_js_bytes: &[u8] = include_bytes!("js/main.js");
    assert_eq!(main_js_bytes, main_js.main_js);

    let template = template_html::TemplateHtml::new();
    let mut hb = Handlebars::new();
    hb.register_template_string("template.html", template.to_string())
        .unwrap();
    let hb = Arc::new(hb);
    let handlebars = move |with_template| render(with_template, hb.clone());

    let route = warp::get()
        .and(warp::path::end())
        .map(|| WithTemplate {
            name: "template.html",
            value: json!({"user" : "🅖"}),
        })
        .map(handlebars.clone())
        .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING));

    let routes = warp::path("echo")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(|websocket| {
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        });

    let cargo_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_static_files = warp::fs::dir(cargo_manifest_dir.join("."));

    let messages_route = warp::path("messages")
        .and(warp::get())
        .map(|| WithTemplate {
            name: "template.html",
            value: json!({ "user": "🅖" }),
        })
        .map(handlebars.clone())
        .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING));

    let pwa_route = warp::path("pwa").and(warp::fs::dir(
        cargo_manifest_dir.join("../../asyncgit/src/lib/pwa"),
    ));
    let images_route = warp::path("images").and(warp::fs::dir(
        cargo_manifest_dir.join("../../asyncgit/src/lib/images"),
    ));
    let js_files = warp::path("js").and(warp::fs::dir(
        cargo_manifest_dir.join("../../asyncgit/src/lib/js"),
    ));
    let css_files = warp::path("css").and(warp::fs::dir(
        cargo_manifest_dir.join("../../asyncgit/src/lib/css"),
    ));

    let nip34_detail_route = warp::path!("repository-details" / String)
        .and(warp::get())
        .map(|_repo_id: String| WithTemplate {
            name: "template.html",
            value: json!({ "user": "🅖" }),
        })
        .map(handlebars.clone())
        .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING));

    let gnostr = warp::path!("gnostr")
        .and(warp::get())
        .map(|| WithTemplate {
            name: "template.html",
            value: json!({ "user": "🅖" }),
        })
        .map(handlebars.clone())
        .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING));

    let root_routes = root_static_files
        .or(messages_route)
        .or(pwa_route)
        .or(images_route)
        .or(js_files)
        .or(css_files)
        .or(nip34_detail_route)
        .or(gnostr)
        .or(warp::get().and(
            warp::path::end()
                .map(|| WithTemplate {
                    name: "template.html",
                    value: json!({ "user": "🅖" }),
                })
                .map(handlebars.clone())
                .map(|reply| {
                    reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING)
                }),
        ))
        .or(warp::get()
            .and(warp::path::full())
            .map(|_| WithTemplate {
                name: "template.html",
                value: json!({ "user": "🅖" }),
            })
            .map(handlebars)
            .map(|reply| reply::with_header(reply, "Content-Security-Policy", RELAXED_CSP_STRING)));

    let _ = open("127.0.0.1", port.try_into().unwrap());
    println!("http://127.0.0.1:{}", port);
    warp::serve(root_routes.clone())
        .run(([127, 0, 0, 1], port))
        .await;
}
