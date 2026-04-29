use std::io;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;
use std::fs as sync_fs;

use clap::Parser;
use serde::Serialize;
use warp::http::StatusCode;
use warp::Reply;
use warp::Filter;

use crate::bridge;
use crate::crawler;
use crate::crawler_control;
use crate::flat;
use crate::embedded::{get_css_assets, get_images_assets, get_js_assets, get_pwa_assets};
use crate::relay_control;

fn open(host: &str, port: i32) -> io::Result<()> {
    let url = format!("http://{}:{}/nip", host, port);

    println!("Attempting to open: {}", url);

    match webbrowser::open(&url) {
        Ok(_) => println!("Successfully opened the browser to {}", url),
        Err(e) => eprintln!("Failed to open browser: {}", e),
    }

    Ok(())
}

#[derive(Serialize)]
struct RelayDiscoveryEntry {
    url: String,
    contact: Option<String>,
    description: Option<String>,
    name: Option<String>,
    ping_ms: Option<u64>,
    software: Option<String>,
    version: Option<String>,
    supported_nips: Vec<i32>,
    supported_nip_extensions: Vec<String>,
    source_nips: Vec<i32>,
}

#[derive(Default)]
struct RelayDiscoveryState {
    contact: Option<String>,
    description: Option<String>,
    name: Option<String>,
    ping_ms: Option<u64>,
    software: Option<String>,
    version: Option<String>,
    supported_nips: BTreeSet<i32>,
    supported_nip_extensions: BTreeSet<String>,
    source_nips: BTreeSet<i32>,
}

fn collect_relay_discovery() -> Vec<RelayDiscoveryEntry> {
    let config_dir = crawler::relays::get_config_dir_path();
    let mut discovered: BTreeMap<String, RelayDiscoveryState> = BTreeMap::new();

    let Ok(nip_dirs) = sync_fs::read_dir(&config_dir) else {
        return Vec::new();
    };

    for nip_entry in nip_dirs.flatten() {
        let Ok(file_type) = nip_entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let nip = match nip_entry.file_name().to_string_lossy().parse::<i32>() {
            Ok(nip) => nip,
            Err(_) => continue,
        };

        let Ok(relay_files) = sync_fs::read_dir(nip_entry.path()) else {
            continue;
        };

        for relay_entry in relay_files.flatten() {
            let file_name = relay_entry.file_name().to_string_lossy().to_string();
            if !file_name.ends_with(".json") || file_name == "relays.json" {
                continue;
            }

            let Ok(content) = sync_fs::read_to_string(relay_entry.path()) else {
                continue;
            };
            let Ok(relay_meta) = serde_json::from_str::<crawler::Relay>(&content) else {
                continue;
            };

            let Some(host) = file_name.strip_suffix(".json") else {
                continue;
            };
            let url = format!("wss://{}", host);
            let state = discovered.entry(url).or_default();

            if state.contact.is_none() {
                state.contact = relay_meta.contact;
            }
            if state.description.is_none() {
                state.description = relay_meta.description;
            }
            if state.name.is_none() {
                state.name = relay_meta.name;
            }
            if state.ping_ms.is_none() {
                state.ping_ms = relay_meta.ping_ms;
            }
            if state.software.is_none() {
                state.software = relay_meta.software;
            }
            if state.version.is_none() {
                state.version = relay_meta.version;
            }
            state.source_nips.insert(nip);
            if let Some(supported_nips) = relay_meta.supported_nips {
                state.supported_nips.extend(supported_nips);
            }
            if let Some(supported_extensions) = relay_meta.supported_nip_extensions {
                state.supported_nip_extensions.extend(supported_extensions);
            }
        }
    }

    let mut entries: Vec<RelayDiscoveryEntry> = discovered
        .into_iter()
        .map(|(url, state)| RelayDiscoveryEntry {
            url,
            contact: state.contact,
            description: state.description,
            name: state.name,
            ping_ms: state.ping_ms,
            software: state.software,
            version: state.version,
            supported_nips: state.supported_nips.into_iter().collect(),
            supported_nip_extensions: state.supported_nip_extensions.into_iter().collect(),
            source_nips: state.source_nips.into_iter().collect(),
        })
        .collect();

    entries.sort_by(|a, b| {
        b.supported_nips
            .len()
            .cmp(&a.supported_nips.len())
            .then_with(|| b.source_nips.len().cmp(&a.source_nips.len()))
            .then_with(|| a.url.cmp(&b.url))
    });

    entries
}

/// Run the embedded Nostr web app on the given port.
pub async fn run(port: u16) -> anyhow::Result<()> {
    const RELAXED_CSP_STRING: &str = "default-src *; manifest-src *; connect-src * ws: wss: http: https:; script-src * 'unsafe-inline' 'unsafe-eval'; script-src-elem * 'unsafe-inline'; script-src-attr * 'unsafe-inline' 'unsafe-hashes'; style-src * 'unsafe-inline' 'unsafe-hashes'; img-src * data:; media-src *; font-src *; child-src *;";
    const NIP34_REPO_KINDS: [i32; 10] = [
        30617, 30618, 1617, 1618, 1619, 1620, 1630, 1631, 1632, 1633,
    ];

    pretty_env_logger::init();
    relay_control::start_relay()?;
    crawler_control::start_crawler(3000)?;

    let shell_html = Arc::new(bridge::shell_html());
    let js_assets_map = Arc::new(get_js_assets());
    let css_assets_map = Arc::new(get_css_assets());
    let images_assets_map = Arc::new(get_images_assets());
    let pwa_assets_map = Arc::new(get_pwa_assets());

    let root_route = warp::path::end()
        .and(warp::get())
        .map(|| warp::redirect::temporary(warp::http::Uri::from_static("/nip")));

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
        warp::path("gnostr")
            .and(warp::path::end())
            .and(warp::get())
            .map(|| warp::redirect::temporary(warp::http::Uri::from_static("/nip")))
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

    let relays_route = {
        let shell_html = Arc::clone(&shell_html);
        warp::path("relays")
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

    let flat_route = warp::path("flat")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .map(|query: HashMap<String, String>| {
            match query.get("repo") {
                Some(repo_url) if !repo_url.is_empty() => {
                    let git_ref = query.get("ref").map(String::as_str);
                    let is_precheck = query
                        .get("check")
                        .or_else(|| query.get("precheck"))
                        .map(|value| !matches!(value.as_str(), "" | "0" | "false"))
                        .unwrap_or(false);

                    if is_precheck {
                        match flat::scan::probe_repo(repo_url, git_ref) {
                            Ok(_) => warp::reply::with_status(
                                warp::reply::json(&serde_json::json!({
                                    "available": true,
                                    "repo": repo_url,
                                    "ref": git_ref,
                                })),
                                StatusCode::OK,
                            )
                            .into_response(),
                            Err(err) => warp::reply::with_status(
                                warp::reply::json(&serde_json::json!({
                                    "available": false,
                                    "repo": repo_url,
                                    "ref": git_ref,
                                    "error": err.to_string(),
                                })),
                                StatusCode::NOT_FOUND,
                            )
                            .into_response(),
                        }
                    } else {
                        match flat::build_html(repo_url, git_ref, 51_200) {
                            Ok(html) => warp::reply::with_header(
                                warp::reply::html(html),
                                "Content-Security-Policy",
                                RELAXED_CSP_STRING,
                            )
                            .into_response(),
                            Err(err) => warp::reply::with_status(
                                warp::reply::html(format!(
                                    "<h1>Flat view unavailable</h1><p>{}</p>",
                                    err
                                )),
                                StatusCode::INTERNAL_SERVER_ERROR,
                            )
                            .into_response(),
                        }
                    }
                }
                _ => warp::reply::with_status(
                    warp::reply::html(
                        "<h1>Flat view unavailable</h1><p>Missing repo query parameter.</p>",
                    ),
                    StatusCode::BAD_REQUEST,
                )
                .into_response(),
            }
        });

    let relay_status_route = warp::path!("api" / "relay" / "status")
        .and(warp::get())
        .map(|| match relay_control::relay_status() {
            Ok(status) => relay_control::response(status, StatusCode::OK),
            Err(err) => relay_control::response(
                relay_control::RelayProcessState {
                    running: false,
                    pid: None,
                    message: err.to_string(),
                    disk_usage_bytes: None,
                },
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        });

    let relay_discovery_route = warp::path!("api" / "relay" / "discovery")
        .and(warp::get())
        .map(|| warp::reply::json(&collect_relay_discovery()));

    let relay_start_route = warp::path!("api" / "relay" / "start")
        .and(warp::post())
        .map(|| match relay_control::start_relay() {
            Ok(status) => relay_control::response(status, StatusCode::OK),
            Err(err) => relay_control::response(
                relay_control::RelayProcessState {
                    running: false,
                    pid: None,
                    message: err.to_string(),
                    disk_usage_bytes: None,
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
                    disk_usage_bytes: None,
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
        .or(root_route)
        .or(messages_route)
        .or(gnostr_route)
        .or(nip_index_route)
        .or(relays_route)
        .or(nip34_route)
        .or(nip34_kind_route)
        .or(nip34_query_route)
        .or(nip34_relays_yaml_route)
        .or(nip34_relays_json_route)
        .or(nip34_relays_txt_route)
        .or(nip34_relay_route)
        .or(repository_detail_route)
        .or(flat_route)
        .or(relay_status_route)
        .or(relay_discovery_route)
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
