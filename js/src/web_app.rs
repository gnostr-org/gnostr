use std::io;
use std::sync::Arc;

use clap::Parser;
use warp::Filter;

use crate::bridge;
use crate::css::css_bundle::get_css_assets;
use crate::images::images_bundle::get_images_assets;
use crate::js::js_bundle::get_js_assets;
use crate::pwa::pwa_bundle::get_pwa_assets;

fn open(host: &str, port: i32) -> io::Result<()> {
    let url = format!("http://{}:{}", host, port);

    println!("Attempting to open: {}", url);

    match webbrowser::open(&url) {
        Ok(_) => println!("Successfully opened the browser to {}", url),
        Err(e) => eprintln!("Failed to open browser: {}", e),
    }

    Ok(())
}

/// Run the embedded Nostr web app on the given port.
pub async fn run(port: u16) {
    const RELAXED_CSP_STRING: &str = "default-src *; manifest-src *; connect-src * ws: wss: http: https:; script-src * 'unsafe-inline' 'unsafe-eval'; script-src-elem * 'unsafe-inline'; script-src-attr * 'unsafe-inline' 'unsafe-hashes'; style-src * 'unsafe-inline' 'unsafe-hashes'; img-src * data:; media-src *; font-src *; child-src *;";

    pretty_env_logger::init();

    let shell_html = Arc::new(bridge::shell_html());
    let js_assets_map = Arc::new(get_js_assets());
    let css_assets_map = Arc::new(get_css_assets());
    let images_assets_map = Arc::new(get_images_assets());
    let pwa_assets_map = Arc::new(get_pwa_assets());

    let shell = {
        let shell_html = Arc::clone(&shell_html);
        warp::get().and(warp::path::end()).map(move || {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let messages_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path("messages").and(warp::get()).map(move || {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let gnostr_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path("gnostr").and(warp::get()).map(move || {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let repository_detail_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path!("repository-details" / String)
            .and(warp::get())
            .map(move |_repo_id: String| {
                warp::reply::with_header(
                    warp::reply::html((*shell_html).clone()),
                    "Content-Security-Policy",
                    RELAXED_CSP_STRING,
                )
            })
    };

    let js_route = warp::path("js")
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| tail.as_str().to_string())
        .and(warp::any().map(move || Arc::clone(&js_assets_map)))
        .map(bridge::asset_response);

    let css_route = warp::path("css")
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| tail.as_str().to_string())
        .and(warp::any().map(move || Arc::clone(&css_assets_map)))
        .map(bridge::asset_response);

    let images_route = warp::path("images")
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| tail.as_str().to_string())
        .and(warp::any().map(move || Arc::clone(&images_assets_map)))
        .map(bridge::asset_response);

    let pwa_route = warp::path("pwa")
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| tail.as_str().to_string())
        .and(warp::any().map(move || Arc::clone(&pwa_assets_map)))
        .map(bridge::asset_response);

    let routes = shell
        .or(messages_route)
        .or(gnostr_route)
        .or(repository_detail_route)
        .or(js_route)
        .or(css_route)
        .or(images_route)
        .or(pwa_route);

    let _ = open("127.0.0.1", port.try_into().unwrap());
    println!("http://127.0.0.1:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
