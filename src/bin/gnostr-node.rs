use axum::{
    body::Body, extract::{Path, RawQuery, State, ws::{Message, WebSocket, WebSocketUpgrade}, Query},
    http::{header, Request, StatusCode}, response::{IntoResponse, Json},
    routing::{get, post}, Form, Router,
};
use axum_template::engine::Engine;
use axum_template::RenderHtml;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use libp2p::{gossipsub, identity, kad, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux, Multiaddr};
use nostr::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Stdio, sync::Arc};
use tokio::{process::Command, sync::mpsc};
use walkdir::WalkDir;
use gnostr::node::templates::{INDEX_HTML, FEED_HTML};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlogPost { pub title: String, pub content: String, pub author: String }

#[derive(Serialize, Clone)]
struct GitRepo { name: String, path: String, is_bare: bool }

struct AppState {
    db: sled::Db,
    tx: mpsc::UnboundedSender<BlogPost>,
    peer_id: String,
    keys: Keys,
}

#[derive(NetworkBehaviour)]
struct BlogBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    kad: kad::Behaviour<kad::store::MemoryStore>,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = 3000)]
    web_port: u16,
    #[arg(short, long)]
    bootstrap: Option<Multiaddr>,
}

// --- Git Discovery Logic ---
fn discover_repos() -> Vec<GitRepo> {
    let mut repos = Vec::new();
    for entry in WalkDir::new(".").max_depth(3).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() && path.extension().map_or(false, |ext| ext == "git") && path.join("HEAD").exists() {
            repos.push(GitRepo { name: path.file_name().unwrap().to_string_lossy().into(), path: path.to_string_lossy().into(), is_bare: true });
        } else if path.is_dir() && path.file_name().map_or(false, |n| n == ".git") {
            let p = path.parent().unwrap_or(path);
            repos.push(GitRepo { name: p.file_name().unwrap().to_string_lossy().into(), path: p.to_string_lossy().into(), is_bare: false });
        }
    }
    repos
}

// --- Handlers ---
async fn nip05_handler(Query(params): Query<HashMap<String, String>>, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let username = params.get("name").map(|s| s.as_str()).unwrap_or("_");
    let mut names = HashMap::new();
    names.insert(username.to_string(), state.keys.public_key().to_string());
    (StatusCode::OK, [(header::CONTENT_TYPE, "application/json"), (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")], Json(serde_json::json!({ "names": names })))
}

async fn git_handler(Path(path): Path<String>, RawQuery(query): RawQuery, req: Request<Body>) -> impl IntoResponse {
    let mut child = Command::new("git").arg("http-backend").env("GIT_PROJECT_ROOT", ".").env("GIT_HTTP_EXPORT_ALL", "1")
        .env("PATH_INFO", format!("/{}", path)).env("QUERY_STRING", query.unwrap_or_default()).env("REQUEST_METHOD", req.method().as_str())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let body_bytes = axum::body::to_bytes(req.into_body(), 100 * 1024 * 1024).await.unwrap();
    let _ = tokio::io::AsyncWriteExt::write_all(&mut stdin, &body_bytes).await;
    let output = child.wait_with_output().await.unwrap();
    (StatusCode::OK, output.stdout).into_response()
}

// Render functions
async fn render_index(
    State(state): State<Arc<AppState>>,
    Engine(engine): Engine<tera::Tera>,
) -> impl IntoResponse {
    let repos = discover_repos();
    let mut context = tera::Context::new();
    context.insert("peer_id", &state.peer_id);
    context.insert("repos", &repos);
    RenderHtml(INDEX_HTML, engine, context)
}

async fn render_feed(
    State(state): State<Arc<AppState>>,
    Engine(engine): Engine<tera::Tera>,
) -> impl IntoResponse {
    let mut posts = Vec::new();
    for (_key, value) in state.db.scan_prefix("blog_post_").filter_map(|r| r.ok()) {
        if let Ok(post) = serde_json::from_slice::<BlogPost>(&value) {
            posts.push(post);
        }
    }
    let mut context = tera::Context::new();
    context.insert("posts", &posts);
    RenderHtml(FEED_HTML, engine, context)
}

async fn publish_post(
    State(state): State<Arc<AppState>>,
    Form(post): Form<BlogPost>,
) -> impl IntoResponse {
    let key = format!("blog_post_{}", uuid::Uuid::new_v4());
    state.db.insert(key.as_bytes(), serde_json::to_vec(&post).unwrap()).unwrap();
    let _ = state.tx.send(post);
    StatusCode::OK
}

async fn relay_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx_clone = state.tx.subscribe();

    // Task for receiving messages from WebSocket
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Placeholder: Process incoming Nostr messages if needed
            println!("Received Nostr message: {}", text);
        }
    });

    // Task for sending messages to WebSocket (e.g., new blog posts)
    let mut send_task = tokio::spawn(async move {
        while let Ok(post) = rx_clone.recv().await {
            let event = EventBuilder::text_note(&post.content, &[]).to_event(&state.keys).unwrap();
            let json_event = serde_json::to_string(&vec!["EVENT", serde_json::to_value(event).unwrap()]).unwrap();
            if let Err(e) = sender.send(Message::Text(json_event)).await {
                eprintln!("Error sending message: {}", e);
                break;
            }
        }
    });

    // If either task completes, abort the other
    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    }
}


#[tokio::main]
async fn main() {
    let args = Args::parse();
    let db = sled::open("gnostr_db").unwrap();
      
    let id_keys = match db.get("id_key").unwrap() {
        Some(b) => identity::Keypair::from_protobuf_encoding(&b).unwrap(),
        None => {
            let k = identity::Keypair::generate_ed25519();
            db.insert("id_key", k.to_protobuf_encoding().unwrap()).unwrap();
            k
        }
    };

    let peer_id = id_keys.public().to_peer_id();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let shared_state = Arc::new(AppState { db, tx: tx.clone(), peer_id: peer_id.to_string(), keys: Keys::generate() });

    let mut tera = tera::Tera::default();
    tera.add_raw_template("index.html", INDEX_HTML).unwrap();
    tera.add_raw_template("feed.html", FEED_HTML).unwrap();

    let app = Router::new()
        .route("/", get(render_index))
        .route("/feed", get(render_feed))
        .route("/publish", post(publish_post))
        .route("/git/*path", get(git_handler).post(git_handler))
        .route("/.well-known/nostr.json", get(nip05_handler))
        .route("/relay", get(relay_handler))
        .with_state(shared_state)
        .layer(axum::Extension(Engine::from(tera)));

    println!("Gnostr Node active on http://localhost:{}", args.web_port);
    axum::serve(tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.web_port)).await.unwrap(), app).await.unwrap();
}
