use crate::commands::run_watch;
use crate::processor::LOCALHOST_8080;
use crate::load_relays_or_bootstrap;
use crate::{build_gnostr_query, fetch_relay_texts, parse_relay_metadata, send};
use axum::{
    body::Body,
    extract::{Path as AxumPath, Query},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use log::{debug, error, info, warn};
use nostr_sdk::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs as sync_fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tokio::fs;
use tokio::task::spawn;
use tower_http::trace::{self, TraceLayer};
use ::url::Url;

pub(crate) async fn collect_supported_relays_for_nip(
    nip_lower: i32,
    client: &reqwest::Client,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let relays = load_relays_or_bootstrap();
    info!(
        "collect_supported_relays_for_nip: checking {} relays for NIP {}",
        relays.len(),
        nip_lower
    );

    let bodies = fetch_relay_texts(relays, client, "collect_supported_relays_for_nip").await;

    let mut supported = Vec::new();
    for item in bodies {
        let (url, json_string, _ping_ms) = match item {
            Ok(tuple) => tuple,
            Err(e) => {
                warn!(
                    "Failed to fetch relay metadata for nip {}: {}",
                    nip_lower, e
                );
                continue;
            }
        };

        let data = parse_relay_metadata(&json_string);
        if let Ok(relay_info) = data {
            if relay_info
                .supported_nips
                .unwrap_or_default()
                .iter()
                .any(|n| *n == nip_lower)
            {
                supported.push(url);
            }
        }
    }

    Ok(supported)
}

pub(crate) async fn prime_all_nip_relays_files(
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("prime_all_nip_relays_files: starting pass");
    let relays = load_relays_or_bootstrap();
    info!(
        "prime_all_nip_relays_files: checking {} relays for NIP support",
        relays.len()
    );
    let bodies = fetch_relay_texts(relays, client, "prime_all_nip_relays_files").await;

    let mut nip_relays: HashMap<i32, HashSet<String>> = HashMap::new();
    for item in bodies {
        if let Ok((url, json_string, ping_ms)) = item {
            if json_string.is_empty() {
                info!("prime_all_nip_relays_files: no metadata body for {}", url);
                continue;
            }
            info!(
                "prime_all_nip_relays_files: read metadata for {} ({} bytes)",
                url,
                json_string.len()
            );
            if let Ok(mut relay_info) = parse_relay_metadata(&json_string) {
                relay_info.ping_ms = Some(ping_ms);
                let supported_nips = relay_info.supported_nips.clone().unwrap_or_default();
                if supported_nips.is_empty() {
                    info!(
                        "prime_all_nip_relays_files: {} reported no supported_nips",
                        url
                    );
                }
                info!(
                    "prime_all_nip_relays_files: {} supports {:?}",
                    url, supported_nips
                );
                for nip in &supported_nips {
                    let dir_path = crate::relays::get_config_dir_path().join(format!("{}", nip));
                    if let Err(e) = sync_fs::create_dir_all(&dir_path) {
                        warn!("Failed to create nip dir {}: {}", dir_path.display(), e);
                        continue;
                    }
                    if let Ok(parsed_url) = Url::parse(&url) {
                        let host = parsed_url.host_str().unwrap_or("unknown");
                        let file_path = dir_path.join(format!("{}.json", host));
                        info!(
                            "prime_all_nip_relays_files: writing relay metadata to {}",
                            file_path.display()
                        );
                        let serialized = serde_json::to_string_pretty(&relay_info)
                            .map_err(std::io::Error::other)?;
                        if let Err(e) = sync_fs::write(&file_path, serialized) {
                            warn!(
                                "Failed to write individual relay file {}: {}",
                                file_path.display(),
                                e
                            );
                        }
                    } else {
                        warn!(
                            "prime_all_nip_relays_files: invalid relay URL {}",
                            url
                        );
                    }
                    nip_relays.entry(*nip).or_default().insert(url.clone());
                }
            } else {
                info!(
                    "prime_all_nip_relays_files: failed to parse relay metadata for {}",
                    url
                );
            }
        } else if let Err(e) = item {
            info!(
                "prime_all_nip_relays_files: request failed while fetching relay metadata: {}",
                e
            );
        }
    }

    for (nip, _) in nip_relays {
        crate::relays::record_live_nips(std::iter::once(nip));
        info!(
            "prime_all_nip_relays_files: rebuilding NIP {} aggregate files",
            nip
        );
        if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip) {
            warn!("Failed to prime nip {} relay files: {}", nip, e);
        }
    }

    info!("prime_all_nip_relays_files: completed pass");
    Ok(())
}

pub(crate) async fn run_sniper_service(client: reqwest::Client) {
    info!("starting sniper service");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
    interval.tick().await;

    loop {
        info!("run_sniper_service: triggering prime pass");
        if let Err(e) = prime_all_nip_relays_files(&client).await {
            warn!("Sniper service failed: {}", e);
        }
        interval.tick().await;
    }
}

pub(crate) async fn refresh_nip_relays_files(
    nip_lower: i32,
    client: &reqwest::Client,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let relays = collect_supported_relays_for_nip(nip_lower, client).await?;
    let dir = crate::relays::write_nip_relays_serve_files(nip_lower, &relays)?;
    Ok(dir)
}

pub(crate) async fn get_relays_yaml() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("relays.yaml");
    debug!(
        "Attempting to serve relays.yaml from: {}",
        file_path.display()
    );

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_relays_serve_files() {
            error!("Failed to create relays.yaml: {}", e);
        }
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => {
            let relays: Vec<String> = content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(String::from)
                .collect();

            match serde_yaml::to_string(&relays) {
                Ok(yaml_content) => Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, "application/x-yaml")
                    .body(Body::from(yaml_content))
                    .unwrap_or_else(|e| {
                        error!("Failed to build relays.yaml response: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Body::from("Internal Server Error"),
                        )
                            .into_response()
                    }),
                Err(e) => {
                    error!("Failed to serialize relays.yaml content: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Body::from(format!("Failed to serialize relays.yaml: {}", e)),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to read relays.yaml: {}. Path: {}",
                e,
                file_path.display()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Failed to read relays.yaml: {}", e)),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_relays_json() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("relays.json");
    debug!(
        "Attempting to serve relays.json from: {}",
        file_path.display()
    );

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_relays_json_from_yaml() {
            error!("Failed to create relays.json: {}", e);
        }
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build relays.json response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Body::from("Internal Server Error"),
                )
                    .into_response()
            }),
        Err(e) => {
            error!(
                "Failed to read relays.json: {}. Path: {}",
                e,
                file_path.display()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Failed to read relays.json: {}", e)),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_relays_txt() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("relays.txt");
    debug!(
        "Attempting to serve relays.txt from: {}",
        file_path.display()
    );

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_relays_serve_files() {
            error!("Failed to create relays.txt: {}", e);
        }
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build relays.txt response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Body::from("Internal Server Error"),
                )
                    .into_response()
            }),
        Err(e) => {
            error!(
                "Failed to read relays.txt: {}. Path: {}",
                e,
                file_path.display()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Failed to read relays.txt: {}", e)),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_nip_relays_yaml(AxumPath(nip_lower): AxumPath<i32>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let file_path = config_dir.join("relays.yaml");
    debug!(
        "Attempting to serve nip relays.yaml from: {}",
        file_path.display()
    );

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower) {
            error!("Failed to derive nip relays.yaml from disk: {}", e);
            let client = reqwest::Client::new();
            if let Err(refresh_err) = refresh_nip_relays_files(nip_lower, &client).await {
                error!(
                    "Failed to refresh nip {} relay cache: {}",
                    nip_lower, refresh_err
                );
            }
        }
    }

    info!("get_nip_relays_yaml: reading {}", file_path.display());
    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/x-yaml")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build nip YAML response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Body::from("Internal Server Error"),
                )
                    .into_response()
            }),
        Err(e) => {
            error!(
                "Failed to read nip relays.yaml: {}. Path: {}",
                e,
                file_path.display()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Failed to read nip relays.yaml: {}", e)),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_nip_relays_json(AxumPath(nip_lower): AxumPath<i32>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let file_path = config_dir.join("relays.json");
    debug!(
        "Attempting to serve nip relays.json from: {}",
        file_path.display()
    );

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower) {
            error!("Failed to derive nip relays.json from disk: {}", e);
            let client = reqwest::Client::new();
            if let Err(refresh_err) = refresh_nip_relays_files(nip_lower, &client).await {
                error!(
                    "Failed to refresh nip {} relay cache: {}",
                    nip_lower, refresh_err
                );
            }
        }
    }

    info!("get_nip_relays_json: reading {}", file_path.display());
    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build nip JSON response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Body::from("Internal Server Error"),
                )
                    .into_response()
            }),
        Err(e) => {
            error!(
                "Failed to read nip relays.json: {}. Path: {}",
                e,
                file_path.display()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Failed to read nip relays.json: {}", e)),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_nip_relays_txt(AxumPath(nip_lower): AxumPath<i32>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let file_path = config_dir.join("relays.txt");
    debug!(
        "Attempting to serve nip relays.txt from: {}",
        file_path.display()
    );

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower) {
            error!("Failed to derive nip relays.txt from disk: {}", e);
            let client = reqwest::Client::new();
            if let Err(refresh_err) = refresh_nip_relays_files(nip_lower, &client).await {
                error!(
                    "Failed to refresh nip {} relay cache: {}",
                    nip_lower, refresh_err
                );
            }
        }
    }

    info!("get_nip_relays_txt: reading {}", file_path.display());
    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build nip TXT response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Body::from("Internal Server Error"),
                )
                    .into_response()
            }),
        Err(e) => {
            error!(
                "Failed to read nip relays.txt: {}. Path: {}",
                e,
                file_path.display()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Failed to read nip relays.txt: {}", e)),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_nip_index(AxumPath(nip_lower): AxumPath<i32>) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let default_kinds = nip_lower.to_string();
    fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let mut entries = vec![
        format!(
            "<li><a href=\"/{}/relays.json\">relays.json</a></li>",
            nip_lower
        ),
        format!(
            "<li><a href=\"/{}/relays.yaml\">relays.yaml</a></li>",
            nip_lower
        ),
        format!(
            "<li><a href=\"/{}/relays.txt\">relays.txt</a></li>",
            nip_lower
        ),
    ];

    if let Ok(mut dir) = fs::read_dir(&config_dir).await {
        let mut relay_cards = Vec::new();
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".json") && name != "relays.json" {
                let file_path = entry.path();
                match fs::read_to_string(&file_path).await {
                    Ok(content) => {
                        let pretty = serde_json::from_str::<serde_json::Value>(&content)
                            .ok()
                            .map(|value| {
                                let relay_name = value
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&name);
                                let nip_links = value
                                    .get("supported_nips")
                                    .and_then(|v| v.as_array())
                                    .map(|nips| {
                                        let links = nips
                                            .iter()
                                            .filter_map(|nip| nip.as_i64())
                                            .map(|nip| {
                                                format!(
                                                    "<a href=\"/{0}\" style=\"margin-right:0.35rem;\">NIP {0}</a>",
                                                    nip
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                            .join("");
                                        if links.is_empty() {
                                            String::new()
                                        } else {
                                            format!("<div style=\"margin:0.25rem 0 0.5rem 0;\">{}</div>", links)
                                        }
                                    })
                                    .unwrap_or_default();
                                let extension_links = value
                                    .get("supported_nip_extensions")
                                    .and_then(|v| v.as_array())
                                    .map(|extensions| {
                                        let links = extensions
                                            .iter()
                                            .filter_map(|extension| extension.as_str())
                                            .map(|extension| {
                                                format!(
                                                    "<code style=\"margin-right:0.35rem;\">{}</code>",
                                                    escape_html(extension)
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                            .join("");
                                        if links.is_empty() {
                                            String::new()
                                        } else {
                                            format!(
                                                "<div style=\"margin:0.25rem 0 0.5rem 0;\"><strong>supported_nip_extensions</strong>: {}</div>",
                                                links
                                            )
                                        }
                                    })
                                    .unwrap_or_default();
                                let icon_html = value
                                    .get("icon")
                                    .and_then(|v| v.as_str())
                                    .map(|icon| {
                                        format!(
                                            "<div style=\"margin:0.5rem 0;\"><img src=\"{}\" alt=\"icon\" style=\"width:48px;height:48px;object-fit:contain;border-radius:0.35rem;background:rgba(255,255,255,0.06);padding:0.25rem;\"></div>",
                                            escape_html(icon)
                                        )
                                    })
                                    .unwrap_or_default();
                                let pretty = serde_json::to_string_pretty(&value).ok().unwrap_or_default();
                                format!(
                                    "<div><strong>{}</strong></div>{}{}{}<pre>{}</pre>",
                                    escape_html(relay_name),
                                    nip_links,
                                    extension_links,
                                    icon_html,
                                    escape_html(&pretty)
                                )
                            })
                            .unwrap_or_else(|| format!("<pre>{}</pre>", escape_html(&content)));
                        let relay_url = name
                            .strip_suffix(".json")
                            .map(|host| format!("wss://{}", host))
                            .unwrap_or_else(|| name.clone());
                        relay_cards.push(format!(
                            "<li><details class=\"relay-favorite-card\" tabindex=\"0\" data-relay-url=\"{}\"><summary><span class=\"relay-favorite-heart\" aria-hidden=\"true\"></span><a href=\"/{}/{}\">{}</a></summary>{}</details></li>",
                            escape_html(&relay_url),
                            nip_lower,
                            name,
                            escape_html(&name),
                            pretty
                        ));
                    }
                    Err(e) => {
                        relay_cards.push(format!(
                            "<li><a href=\"/{}/{}\">{}</a> <em>(failed to read metadata: {})</em></li>",
                            nip_lower,
                            name,
                            escape_html(&name),
                            escape_html(&e.to_string())
                        ));
                    }
                }
            }
        }
        relay_cards.sort();
        entries.extend(relay_cards);
    }

    let query_href = format!("/{}/query", nip_lower);
    let nav = vec![
        ("/", "gnostr/crawler"),
        ("/relays.json", "relays.json"),
        ("/relays.yaml", "relays.yaml"),
        ("/relays.txt", "relays.txt"),
    ];
    let query_form = crate::query::forms::nip_query_form(
        nip_lower,
        &query_href,
        "",
        Some(default_kinds.as_str()),
    );
    let body = format!(
        "{}\
          <section><p><a href=\"/\">&larr; back to home</a></p>\
            <h2>NIP {}</h2><ul>{}</ul></section>",
        query_form,
        nip_lower,
        entries.join("")
    );
    let favorite_script = r#"<script>
    (() => {
      const key = "gnostr-crawler-favorite-relays";
      const load = () => {
        try { return new Set(JSON.parse(localStorage.getItem(key) || "[]")); }
        catch (_) { return new Set(); }
      };
      const save = (values) => localStorage.setItem(key, JSON.stringify([...values]));
      const favorites = load();
      const update = (card) => {
        const url = card.dataset.relayUrl;
        const heart = card.querySelector(".relay-favorite-heart");
        const favorite = favorites.has(url);
        card.classList.toggle("is-favorite", favorite);
        if (heart) heart.textContent = favorite ? "❤" : "";
      };
      const cards = document.querySelectorAll("[data-relay-url]");
      cards.forEach((card) => update(card));
      document.addEventListener("keydown", (ev) => {
        if (ev.code !== "Space") return;
        const card = ev.target.closest("[data-relay-url]");
        if (!card) return;
        ev.preventDefault();
        ev.stopPropagation();
        const url = card.dataset.relayUrl;
        if (favorites.has(url)) {
          favorites.delete(url);
        } else {
          favorites.add(url);
        }
        save(favorites);
        update(card);
      });
    })();
    </script>"#;
    let html = crate::relays::render_page_shell(
        &format!("gnostr crawler / NIP {}", nip_lower),
        &nav,
        &format!("{}{}", body, favorite_script),
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(html))
        .unwrap_or_else(|e| {
            error!("Failed to build nip index response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from("Internal Server Error"),
            )
                .into_response()
        })
}

pub(crate) fn load_nip_query_relays(
    nip_lower: i32,
    relay_override: Option<&str>,
) -> Result<Vec<Url>, Box<dyn std::error::Error>> {
    if let Some(relay) = relay_override {
        return Ok(vec![Url::parse(relay)?]);
    }

    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let relays_path = config_dir.join("relays.txt");
    if !relays_path.exists() {
        let _ = crate::relays::write_nip_relays_serve_files_from_dir(nip_lower);
    }

    let content = std::fs::read_to_string(&relays_path)?;
    let relays = content
        .split_whitespace()
        .filter_map(|relay| Url::parse(relay).ok())
        .collect::<Vec<_>>();

    if relays.is_empty() {
        return Err(format!("no relays available for NIP {}", nip_lower).into());
    }

    Ok(relays)
}

fn prepend_local_relay(relays: Vec<Url>) -> Vec<Url> {
    let mut with_local = Vec::with_capacity(relays.len() + 1);
    if let Ok(local_relay) = Url::parse(LOCALHOST_8080) {
        with_local.push(local_relay);
    }
    for relay in relays {
        if !with_local.iter().any(|existing| existing == &relay) {
            with_local.push(relay);
        }
    }
    with_local
}

pub(crate) fn non_empty_param<'a>(params: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    params
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
}

pub(crate) async fn execute_query_page(
    title: &str,
    nav: &[(&str, &str)],
    form_html: &str,
    query_string: String,
    relays: Vec<Url>,
    limit: Option<i32>,
    search_term: Option<&str>,
) -> Response {
    let results = match send(query_string.clone(), relays, limit.or(Some(100))).await {
        Ok(results) => results,
        Err(e) => {
            let html = crate::relays::render_page_shell(
                title,
                nav,
                &format!("{}<p>Query failed: {}</p>", form_html, e),
            );
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build query failure response: {}", build_err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Body::from("Internal Server Error"),
                    )
                        .into_response()
                });
        }
    };

    let results = if let Some(search_term) = search_term {
        filter_query_results(results, search_term)
    } else {
        results
    };

    let results_html = if results.is_empty() {
        "<p>No results.</p>".to_string()
    } else {
        format!("<pre>{}</pre>", results.join("\n"))
    };

    let body = format!(
        "{}<section><h2>Query results</h2><p><code>{}</code></p>{}</section>",
        form_html, query_string, results_html
    );
    let html = crate::relays::render_page_shell(title, nav, &body);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(html))
        .unwrap_or_else(|e| {
            error!("Failed to build query response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from("Internal Server Error"),
            )
                .into_response()
        })
}

pub(crate) async fn get_query(Query(params): Query<HashMap<String, String>>) -> Response {
    fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let relay = non_empty_param(&params, "relay");
    let authors = non_empty_param(&params, "authors");
    let ids = non_empty_param(&params, "ids");
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i32>().ok());
    let kinds = non_empty_param(&params, "kinds");
    let search = non_empty_param(&params, "search");
    let generic_tag = non_empty_param(&params, "generic_tag");
    let generic_value = non_empty_param(&params, "generic_value");
    let hashtag = non_empty_param(&params, "hashtag");
    let mentions = non_empty_param(&params, "mentions");
    let references = non_empty_param(&params, "references");

    let generic = match (generic_tag, generic_value) {
        (Some(tag), Some(value)) => Some((tag, value)),
        _ => None,
    };

    let query_string = match build_gnostr_query(
        authors,
        ids,
        limit,
        generic,
        hashtag,
        mentions,
        references,
        kinds,
        search.map(|s| ("search", s)),
    ) {
        Ok(query) => query,
        Err(e) => {
            let html = crate::relays::render_page_shell(
                "gnostr crawler / query",
                &[("/", "gnostr/crawler")],
                &format!("<p>Failed to build query: {}</p>", e),
            );
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build query error response: {}", build_err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Body::from("Internal Server Error"),
                    )
                        .into_response()
                });
        }
    };

    let relays = if let Some(relay) = relay {
        match Url::parse(relay) {
            Ok(url) => vec![url],
            Err(e) => {
                let html = crate::relays::render_page_shell(
                    "gnostr crawler / query",
                    &[("/", "gnostr/crawler")],
                    &format!("<p>Invalid relay URL: {}</p>", e),
                );
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(CONTENT_TYPE, "text/html")
                    .body(Body::from(html))
                    .unwrap_or_else(|build_err| {
                        error!("Failed to build relay error response: {}", build_err);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Body::from("Internal Server Error"),
                        )
                            .into_response()
                    });
            }
        }
    } else {
        load_relays_or_bootstrap()
            .into_iter()
            .filter_map(|relay| Url::parse(&relay).ok())
            .collect()
    };
    let relays = prepend_local_relay(relays);

    let kinds_value = crate::relays::live_kinds().join(",");
    let query_form = crate::query::forms::generic_query_form("/query", Some(kinds_value.as_str()));
    let nav = [("/", "gnostr/crawler"), ("/query", "query")];
    execute_query_page(
        "gnostr crawler / query",
        &nav,
        &query_form,
        escape_html(&query_string),
        relays,
        limit,
        search,
    )
    .await
}

pub(crate) async fn get_nip_query(
    AxumPath(nip_lower): AxumPath<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let relay = params
        .get("relay")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let authors = params
        .get("authors")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let ids = params
        .get("ids")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i32>().ok());
    let kinds = params
        .get("kinds")
        .map(String::as_str)
        .or(Some("1630,1632,1621,30618,1633,1631,1617,30617"));
    let search = params
        .get("search")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let generic_tag = params
        .get("generic_tag")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let generic_value = params
        .get("generic_value")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let hashtag = params
        .get("hashtag")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let mentions = params
        .get("mentions")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let references = params
        .get("references")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty());
    let default_kinds = nip_lower.to_string();
    let query_href = format!("/{}/query", nip_lower);
    let back_href = format!("/{}/", nip_lower);

    let generic = match (generic_tag, generic_value) {
        (Some(tag), Some(value)) => Some((tag, value)),
        _ => None,
    };

    let query_string = match build_gnostr_query(
        authors,
        ids,
        limit,
        generic,
        hashtag,
        mentions,
        references,
        kinds.or(Some(default_kinds.as_str())),
        search.map(|s| ("search", s)),
    ) {
        Ok(query) => query,
        Err(e) => {
            let nav = vec![("/", "gnostr/crawler"), (back_href.as_str(), "back")];
            let html = crate::relays::render_page_shell(
                &format!("gnostr crawler / NIP {} query", nip_lower),
                &nav,
                &format!(
                    "<p>Failed to build query: {}</p>",
                    escape_html(&e.to_string())
                ),
            );
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build query error response: {}", build_err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Body::from("Internal Server Error"),
                    )
                        .into_response()
                });
        }
    };

    let relays = match load_nip_query_relays(nip_lower, relay) {
        Ok(relays) => relays,
        Err(e) => {
            let nav = vec![("/", "gnostr/crawler"), (back_href.as_str(), "back")];
            let html = crate::relays::render_page_shell(
                &format!("gnostr crawler / NIP {} query", nip_lower),
                &nav,
                &format!(
                    "<p>Failed to load relays: {}</p>",
                    escape_html(&e.to_string())
                ),
            );
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build relay error response: {}", build_err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Body::from("Internal Server Error"),
                    )
                        .into_response()
                });
        }
    };
    let relays = prepend_local_relay(relays);

    let results = match send(query_string.clone(), relays, limit.or(Some(100))).await {
        Ok(results) => results,
        Err(e) => {
            let nav = vec![("/", "gnostr/crawler"), (back_href.as_str(), "back")];
            let html = crate::relays::render_page_shell(
                &format!("gnostr crawler / NIP {} query", nip_lower),
                &nav,
                &format!("<p>Query failed: {}</p>", escape_html(&e.to_string())),
            );
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(html))
                .unwrap_or_else(|build_err| {
                    error!("Failed to build query failure response: {}", build_err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Body::from("Internal Server Error"),
                    )
                        .into_response()
                });
        }
    };

    let query_form = crate::query::forms::nip_query_form(
        nip_lower,
        &query_href,
        relay.unwrap_or(""),
        Some(default_kinds.as_str()),
    );
    let results = if let Some(search_term) = search {
        filter_query_results(results, search_term)
    } else {
        results
    };
    let results_html = if results.is_empty() {
        "<p>No results.</p>".to_string()
    } else {
        format!("<pre>{}</pre>", escape_html(&results.join("\n")))
    };

    let nav = vec![
        ("/", "gnostr/crawler"),
        (query_href.as_str(), "query"),
        (back_href.as_str(), "back"),
    ];
    let body = format!(
        "{}<section><h2>NIP {} query results</h2><p><code>{}</code></p>{}</section>",
        query_form,
        nip_lower,
        escape_html(&query_string),
        results_html
    );
    let html = crate::relays::render_page_shell(
        &format!("gnostr crawler / NIP {} query", nip_lower),
        &nav,
        &body,
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(html))
        .unwrap_or_else(|e| {
            error!("Failed to build nip query response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from("Internal Server Error"),
            )
                .into_response()
        })
}

fn filter_query_results(results: Vec<String>, search_term: &str) -> Vec<String> {
    let needles: Vec<String> = search_term
        .split(|c: char| c.is_whitespace() || c == ',')
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(|term| term.to_ascii_lowercase())
        .collect();

    if needles.is_empty() {
        return results;
    }

    results
        .into_iter()
        .filter(|result| {
            let haystack = result.to_ascii_lowercase();
            needles.iter().all(|needle| haystack.contains(needle))
        })
        .collect()
}

pub(crate) async fn get_nip_relay_json(
    AxumPath((nip_lower, relay_file)): AxumPath<(i32, String)>,
) -> Response {
    let config_dir = crate::relays::get_config_dir_path().join(nip_lower.to_string());
    let file_path = config_dir.join(&relay_file);
    debug!(
        "Attempting to serve nip relay file from: {}",
        file_path.display()
    );

    if !relay_file.ends_with(".json") {
        return (
            StatusCode::BAD_REQUEST,
            Body::from("Expected a .json relay file"),
        )
            .into_response();
    }

    info!("get_nip_relay_json: reading {}", file_path.display());
    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build nip relay JSON response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Body::from("Internal Server Error"),
                )
                    .into_response()
            }),
        Err(e) => {
            error!(
                "Failed to read nip relay json: {}. Path: {}",
                e,
                file_path.display()
            );
            (
                StatusCode::NOT_FOUND,
                Body::from(format!("Failed to read nip relay json: {}", e)),
            )
                .into_response()
        }
    }
}

pub(crate) async fn get_index_html() -> Response {
    let config_dir = crate::relays::get_config_dir_path();
    let file_path = config_dir.join("index.html");
    debug!(
        "Attempting to serve index.html from: {}",
        file_path.display()
    );

    if !file_path.exists() {
        if let Err(e) = crate::relays::write_index_html() {
            error!("Failed to create index.html: {}", e);
        }
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "text/html")
            .body(Body::from(content))
            .unwrap_or_else(|e| {
                error!("Failed to build HTML response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Body::from("Internal Server Error"),
                )
                    .into_response()
            }),
        Err(e) => {
            error!(
                "Failed to read index.html: {}. Path: {}",
                e,
                file_path.display()
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Failed to read index.html: {}", e)),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, routing::get, Router};
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use tokio::net::TcpListener;

    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvGuard {
        prev_home: Option<std::ffi::OsString>,
        prev_xdg: Option<std::ffi::OsString>,
        _lock: std::sync::MutexGuard<'static, ()>,
        root: PathBuf,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match self.prev_home.take() {
                    Some(value) => env::set_var("HOME", value),
                    None => env::remove_var("HOME"),
                }
                match self.prev_xdg.take() {
                    Some(value) => env::set_var("XDG_CONFIG_HOME", value),
                    None => env::remove_var("XDG_CONFIG_HOME"),
                }
            }
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn isolate_config_dir() -> EnvGuard {
        let lock = TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let mut root = std::env::temp_dir();
        root.push(format!(
            "gnostr-crawler-api-tests-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be monotonic")
                .as_nanos()
        ));
        let home = root.join("home");
        let xdg = root.join("xdg");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&xdg).unwrap();

        let prev_home = env::var_os("HOME");
        let prev_xdg = env::var_os("XDG_CONFIG_HOME");

        unsafe {
            env::set_var("HOME", &home);
            env::set_var("XDG_CONFIG_HOME", &xdg);
        }

        EnvGuard {
            prev_home,
            prev_xdg,
            _lock: lock,
            root,
        }
    }

    async fn start_http_server(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = Router::new().route("/", get(move || async move { body }));
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });
        format!("ws://{}", addr)
    }

    async fn response_text(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn relay_files_are_served_with_content_types() {
        let _guard = isolate_config_dir();
        let config_dir = crate::relays::get_config_dir_path();
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("relays.yaml"),
            "wss://relay.example.com\nws://relay.example.org\n",
        )
        .unwrap();
        fs::write(config_dir.join("relays.json"), "[\"wss://relay.example.com/\"]").unwrap();
        fs::write(config_dir.join("relays.txt"), "wss://relay.example.com/ ws://relay.example.org/")
            .unwrap();

        let yaml = get_relays_yaml().await;
        assert_eq!(yaml.status(), StatusCode::OK);
        assert_eq!(
            yaml.headers().get(CONTENT_TYPE).unwrap(),
            "application/x-yaml"
        );
        assert!(response_text(yaml).await.contains("relay.example.com"));

        let json = get_relays_json().await;
        assert_eq!(json.status(), StatusCode::OK);
        assert_eq!(
            json.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
        assert!(response_text(json).await.contains("relay.example.com"));

        let txt = get_relays_txt().await;
        assert_eq!(txt.status(), StatusCode::OK);
        assert_eq!(txt.headers().get(CONTENT_TYPE).unwrap(), "text/plain");
        assert!(response_text(txt).await.contains("relay.example.com"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn nip_files_and_index_are_served_from_disk() {
        let _guard = isolate_config_dir();
        let config_dir = crate::relays::get_config_dir_path();
        let nip_dir = config_dir.join("34");
        fs::create_dir_all(&nip_dir).unwrap();
        fs::write(
            nip_dir.join("relay-one.json"),
            r#"{"name":"Relay One","supported_nips":[34,35],"ping_ms":17}"#,
        )
        .unwrap();
        fs::write(nip_dir.join("relays.yaml"), "wss://relay-one/\n").unwrap();
        fs::write(nip_dir.join("relays.json"), "[\"wss://relay-one/\"]").unwrap();
        fs::write(nip_dir.join("relays.txt"), "wss://relay-one/").unwrap();

        let yaml = get_nip_relays_yaml(AxumPath(34)).await;
        assert_eq!(yaml.status(), StatusCode::OK);
        assert_eq!(
            yaml.headers().get(CONTENT_TYPE).unwrap(),
            "application/x-yaml"
        );
        assert!(response_text(yaml).await.contains("wss://relay-one"));

        let json = get_nip_relays_json(AxumPath(34)).await;
        assert_eq!(json.status(), StatusCode::OK);
        assert_eq!(
            json.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
        assert!(response_text(json).await.contains("wss://relay-one"));

        let txt = get_nip_relays_txt(AxumPath(34)).await;
        assert_eq!(txt.status(), StatusCode::OK);
        assert_eq!(txt.headers().get(CONTENT_TYPE).unwrap(), "text/plain");
        assert!(response_text(txt).await.contains("wss://relay-one"));

        let index = get_nip_index(AxumPath(34)).await;
        assert_eq!(index.status(), StatusCode::OK);
        assert_eq!(index.headers().get(CONTENT_TYPE).unwrap(), "text/html");
        let html = response_text(index).await;
        assert!(html.contains("Relay One"));
        assert!(html.contains("/34/relay-one.json"));
        assert!(html.contains("NIP 35"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn prime_all_nip_relays_files_writes_ping_and_aggregate_files() {
        let _guard = isolate_config_dir();
        let relay_url = start_http_server(
            r#"{"name":"Relay One","supported_nips":[34],"version":"1.0"}"#,
        )
        .await;
        let config_dir = crate::relays::get_config_dir_path();
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("relays.yaml"),
            format!("{}\nws://127.0.0.1:1\n", relay_url),
        )
        .unwrap();

        let client = reqwest::Client::new();
        prime_all_nip_relays_files(&client).await.unwrap();

        let nip_dir = config_dir.join("34");
        let relay_file = nip_dir.join("127.0.0.1.json");
        assert!(relay_file.exists());
        let relay_json = fs::read_to_string(&relay_file).unwrap();
        let relay: crate::relay_metadata::Relay = serde_json::from_str(&relay_json).unwrap();
        assert_eq!(relay.name.as_deref(), Some("Relay One"));
        assert_eq!(relay.supported_nips, Some(vec![34]));
        assert!(relay.ping_ms.is_some());

        assert!(nip_dir.join("relays.yaml").exists());
        assert!(nip_dir.join("relays.json").exists());
        assert!(nip_dir.join("relays.txt").exists());
    }
}

pub async fn run_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    debug!("run_api_server: Starting API server on port {}", port);

    let client = reqwest::Client::new();
    if let Err(e) = crate::relays::write_relays_serve_files() {
        warn!("Failed to prepare relay serve files: {}", e);
    }
    crate::relays::prime_live_kinds_from_disk();
    if let Err(e) = crate::relays::write_kinds_serve_files() {
        warn!("Failed to prepare kinds serve files: {}", e);
    }
    if let Err(e) = crate::relays::write_index_html() {
        warn!("Failed to prepare index.html: {}", e);
    }

    let client_for_watch = client.clone();
    tokio::task::spawn(async move {
        if let Err(e) = run_watch(None, &client_for_watch).await {
            error!("Watch process failed: {}", e);
        }
    });

    let client_for_sniper = client.clone();
    spawn(async move {
        run_sniper_service(client_for_sniper).await;
    });

    let app = Router::new()
        .route("/", get(get_index_html))
        .route("/query", get(get_query))
        .route("/relays.yaml", get(get_relays_yaml))
        .route("/relays.json", get(get_relays_json))
        .route("/relays.txt", get(get_relays_txt))
        .route("/:nip", get(get_nip_index))
        .route("/:nip/query", get(get_nip_query))
        .route("/:nip/relays.yaml", get(get_nip_relays_yaml))
        .route("/:nip/relays.json", get(get_nip_relays_json))
        .route("/:nip/relays.txt", get(get_nip_relays_txt))
        .route("/:nip/:relay.json", get(get_nip_relay_json))
        .route("/nip/:nip", get(get_nip_index))
        .route("/nip/:nip/query", get(get_nip_query))
        .route("/nip/:nip/relays.yaml", get(get_nip_relays_yaml))
        .route("/nip/:nip/relays.json", get(get_nip_relays_json))
        .route("/nip/:nip/relays.txt", get(get_nip_relays_txt))
        .route("/nip/:nip/:relay.json", get(get_nip_relay_json))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_response(trace::DefaultOnResponse::new().include_headers(true)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("run_api_server: listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

pub fn run_api_server_detached(
    command_prefix: &[&str],
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()?;
    let mut command = Command::new(current_exe);
    command.args(command_prefix);
    command.arg("--port");
    command.arg(port.to_string());
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    let child = command.spawn()?;
    println!(
        "run_api_server_detached: started background server (pid: {})",
        child.id()
    );
    Ok(())
}
