use std::io;
use std::sync::Arc;

use clap::Parser;
use warp::http::StatusCode;
use warp::Filter;

use crate::bridge;
use crate::embedded::{get_css_assets, get_images_assets, get_js_assets, get_pwa_assets};
use crate::relay_control;

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
pub async fn run(port: u16) -> anyhow::Result<()> {
    const RELAXED_CSP_STRING: &str = "default-src *; manifest-src *; connect-src * ws: wss: http: https:; script-src * 'unsafe-inline' 'unsafe-eval'; script-src-elem * 'unsafe-inline'; script-src-attr * 'unsafe-inline' 'unsafe-hashes'; style-src * 'unsafe-inline' 'unsafe-hashes'; img-src * data:; media-src *; font-src *; child-src *;";
    const NIP34_REPO_KINDS: [i32; 10] = [
        30617, 30618, 1617, 1618, 1619, 1620, 1630, 1631, 1632, 1633,
    ];

    pretty_env_logger::init();
    relay_control::start_relay()?;

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

    let nip_index_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path("nip")
            .and(warp::path::end())
            .and(warp::get())
            .map(move || {
                warp::reply::with_header(
                    warp::reply::html((*shell_html).clone()),
                    "Content-Security-Policy",
                    RELAXED_CSP_STRING,
                )
            })
    };

    let nip34_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path!("nip" / "34").and(warp::get()).map(move || {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let nip34_kind_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path!("nip" / "34" / i32)
            .and(warp::get())
            .map(move |kind: i32| {
                let status = if NIP34_REPO_KINDS.contains(&kind) {
                    StatusCode::OK
                } else {
                    StatusCode::NOT_FOUND
                };
                let reply = warp::reply::with_header(
                    warp::reply::html((*shell_html).clone()),
                    "Content-Security-Policy",
                    RELAXED_CSP_STRING,
                );
                warp::reply::with_status(reply, status)
            })
    };

    let nip34_query_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path!("nip" / "34" / "query").and(warp::get()).map(move || {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let nip34_relays_yaml_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path!("nip" / "34" / "relays.yaml").and(warp::get()).map(move || {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let nip34_relays_json_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path!("nip" / "34" / "relays.json").and(warp::get()).map(move || {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let nip34_relays_txt_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path!("nip" / "34" / "relays.txt").and(warp::get()).map(move || {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let nip34_relay_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path!("nip" / "34" / String).and(warp::get()).map(move |_relay_file: String| {
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

    let relay_status_route = warp::path!("api" / "relay" / "status")
        .and(warp::get())
        .map(|| match relay_control::relay_status() {
            Ok(status) => relay_control::response(status, StatusCode::OK),
            Err(err) => relay_control::response(
                relay_control::RelayProcessState {
                    running: false,
                    pid: None,
                    message: err.to_string(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        });

    let relay_start_route = warp::path!("api" / "relay" / "start")
        .and(warp::post())
        .map(|| match relay_control::start_relay() {
            Ok(status) => relay_control::response(status, StatusCode::OK),
            Err(err) => relay_control::response(
                relay_control::RelayProcessState {
                    running: false,
                    pid: None,
                    message: err.to_string(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        });

    let relay_stop_route = warp::path!("api" / "relay" / "stop")
        .and(warp::post())
        .map(|| match relay_control::stop_relay() {
            Ok(status) => relay_control::response(status, StatusCode::OK),
            Err(err) => relay_control::response(
                relay_control::RelayProcessState {
                    running: true,
                    pid: None,
                    message: err.to_string(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        });

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

    let shell_fallback = {
        let shell_html = Arc::clone(&shell_html);
        warp::path::full().and(warp::get()).map(move |_| {
            warp::reply::with_header(
                warp::reply::html((*shell_html).clone()),
                "Content-Security-Policy",
                RELAXED_CSP_STRING,
            )
        })
    };

    let routes = shell
        .or(messages_route)
        .or(gnostr_route)
        .or(nip_index_route)
        .or(nip34_route)
        .or(nip34_kind_route)
        .or(nip34_query_route)
        .or(nip34_relays_yaml_route)
        .or(nip34_relays_json_route)
        .or(nip34_relays_txt_route)
        .or(nip34_relay_route)
        .or(repository_detail_route)
        .or(relay_status_route)
        .or(relay_start_route)
        .or(relay_stop_route)
        .or(js_route)
        .or(css_route)
        .or(images_route)
        .or(pwa_route)
        .or(shell_fallback);

    let _ = open("127.0.0.1", port.try_into().unwrap());
    println!("http://127.0.0.1:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    Ok(())
}
