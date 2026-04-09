/// Async nostr client for gnostr-tui.
///
/// Follows the same `std::thread::spawn` + crossbeam-channel pattern used by
/// `AsyncPush` / `AsyncPull` in asyncgit.  A dedicated OS thread hosts a
/// single-threaded tokio runtime; relay connections use tokio-tungstenite;
/// events are forwarded to the TUI via a `Sender<AsyncNostrNotification>`.
use std::{
	sync::{Arc, Mutex},
	thread,
	time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossbeam_channel::{unbounded, Sender};
use futures_util::{
	future::FutureExt as _,
	stream::{SplitSink, SplitStream},
	SinkExt, StreamExt,
};
use secp256k1::{Keypair, XOnlyPublicKey, SECP256K1};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::net::TcpStream;
use tokio_tungstenite::{
	connect_async,
	tungstenite::Message as WsMessage,
	MaybeTlsStream, WebSocketStream,
};

type WsSink =
	SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

use crate::error::{Error, Result};

use super::keys::NostrIdentity;

// ── notification enum ─────────────────────────────────────────────────────────

/// Notification sent from the nostr background thread to the TUI event loop.
#[derive(Clone, Debug)]
pub enum AsyncNostrNotification {
	/// Connected to at least one relay.
	Connected,
	/// Disconnected / shut down.
	Disconnected,
	/// A kind-1 text note arrived.
	TextNote(Box<NostrEvent>),
	/// Profile metadata (kind 0) arrived.
	Profile { pubkey: String, display: String },
	/// A note was published; carries the hex event id.
	NotePublished(String),
	/// An error in the background thread.
	Error(String),
}

// ── minimal nostr event ───────────────────────────────────────────────────────

/// A nostr event (NIP-01).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NostrEvent {
	pub id: String,
	pub pubkey: String,
	pub created_at: u64,
	pub kind: u64,
	pub tags: Vec<Vec<String>>,
	pub content: String,
	pub sig: String,
}

impl NostrEvent {
	/// Compute the NIP-01 event id (SHA-256 of canonical JSON).
	fn compute_id(
		pubkey: &str,
		created_at: u64,
		kind: u64,
		tags: &[Vec<String>],
		content: &str,
	) -> String {
		let serialised = json!([
			0, pubkey, created_at, kind, tags, content
		])
		.to_string();
		let mut h = Sha256::new();
		h.update(serialised.as_bytes());
		hex::encode(h.finalize())
	}

	/// Build and schnorr-sign a new text note (kind 1).
	pub fn new_text_note(content: &str, kp: &Keypair) -> Self {
		let (xonly, _) = XOnlyPublicKey::from_keypair(kp);
		let pubkey = hex::encode(xonly.serialize());
		let created_at =
			SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
		let kind = 1u64;
		let tags: Vec<Vec<String>> = vec![];
		let id = Self::compute_id(&pubkey, created_at, kind, &tags, content);

		let msg_bytes: [u8; 32] = {
			let mut a = [0u8; 32];
			let decoded = hex::decode(&id).unwrap_or_default();
			let len = decoded.len().min(32);
			a[..len].copy_from_slice(&decoded[..len]);
			a
		};
		let msg = secp256k1::Message::from_digest(msg_bytes);
		let sig = SECP256K1.sign_schnorr_no_aux_rand(&msg, kp);

		Self {
			id,
			pubkey,
			created_at,
			kind,
			tags,
			content: content.to_owned(),
			sig: hex::encode(sig.serialize()),
		}
	}
}

// ── commands ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(super) enum NostrCmd {
	/// Publish a kind-1 text note.
	PublishNote { content: String },
	/// Subscribe to a basic global feed.
	SubscribeGlobal,
	/// Gracefully shut down.
	Shutdown,
}

// ── AsyncNostr ────────────────────────────────────────────────────────────────

/// Async nostr client handle.
pub struct AsyncNostr {
	cmd_tx: Sender<NostrCmd>,
	pending: Arc<Mutex<bool>>,
	notification_tx: Sender<AsyncNostrNotification>,
}

impl AsyncNostr {
	pub fn new(sender: Sender<AsyncNostrNotification>) -> Self {
		let (cmd_tx, _) = unbounded();
		Self {
			cmd_tx,
			pending: Arc::new(Mutex::new(false)),
			notification_tx: sender,
		}
	}

	/// Connect to the given relay URLs and start the background event loop.
	pub fn connect(
		&mut self,
		identity: NostrIdentity,
		relay_urls: Vec<String>,
	) -> Result<()> {
		if self.is_pending() {
			return Ok(());
		}

		let (cmd_tx, cmd_rx) = unbounded::<NostrCmd>();
		self.cmd_tx = cmd_tx;

		let notification_tx = self.notification_tx.clone();
		let pending = Arc::clone(&self.pending);
		*pending.lock().map_err(|_| Error::Generic("mutex poisoned".to_owned()))? =
			true;

		thread::Builder::new()
			.name("asyncnostr".into())
			.spawn(move || {
				let rt = tokio::runtime::Builder::new_current_thread()
					.enable_all()
					.build()
					.expect("tokio runtime");
				rt.block_on(run(identity, relay_urls, cmd_rx, notification_tx));
				if let Ok(mut p) = pending.lock() {
					*p = false;
				}
			})
			.map_err(|e| Error::Generic(format!("thread spawn: {e}")))?;

		Ok(())
	}

	pub fn is_pending(&self) -> bool {
		self.pending.lock().map_or(false, |g| *g)
	}

	pub fn publish_note(&self, content: String) -> Result<()> {
		self.send_cmd(NostrCmd::PublishNote { content })
	}

	pub fn subscribe_global(&self) -> Result<()> {
		self.send_cmd(NostrCmd::SubscribeGlobal)
	}

	pub fn shutdown(&self) -> Result<()> {
		self.send_cmd(NostrCmd::Shutdown)
	}

	fn send_cmd(&self, cmd: NostrCmd) -> Result<()> {
		self.cmd_tx
			.send(cmd)
			.map_err(|e| Error::Generic(format!("nostr cmd send: {e}")))
	}
}

// ── background async loop ─────────────────────────────────────────────────────

const TIMELINE_LIMIT: usize = 50;
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

async fn run(
	identity: NostrIdentity,
	relay_urls: Vec<String>,
	cmd_rx: crossbeam_channel::Receiver<NostrCmd>,
	notif_tx: Sender<AsyncNostrNotification>,
) {
	// Connect to all relays concurrently.
	let mut sinks: Vec<(String, WsSink)> = Vec::new();
	let mut streams: Vec<WsStream> = Vec::new();

	for url in &relay_urls {
		match connect_async(url.as_str()).await {
			Ok((ws, _)) => {
				let (sink, stream) = ws.split();
				sinks.push((url.clone(), sink));
				streams.push(stream);
				log::info!("nostr: connected to {url}");
			}
			Err(e) => {
				log::warn!("nostr: connect {url}: {e}");
				send(
					&notif_tx,
					AsyncNostrNotification::Error(format!("connect {url}: {e}")),
				);
			}
		}
	}

	if sinks.is_empty() {
		send(
			&notif_tx,
			AsyncNostrNotification::Error("no relays connected".to_owned()),
		);
		return;
	}

	send(&notif_tx, AsyncNostrNotification::Connected);

	// Build a subscription id and initial filter.
	let sub_id = "gnostr-tui-global";
	let sub_msg = json!(["REQ", sub_id, {
		"kinds": [1, 0],
		"limit": TIMELINE_LIMIT
	}])
	.to_string();

	// Send REQ to all sinks.
	for (url, sink) in &mut sinks {
		if let Err(e) = sink.send(WsMessage::Text(sub_msg.clone().into())).await {
			log::warn!("nostr: REQ to {url}: {e}");
		}
	}

	// Fuse all relay streams into one via tokio::select polling.
	loop {
		// Drain commands first (non-blocking).
		loop {
			match cmd_rx.try_recv() {
				Ok(NostrCmd::Shutdown) => {
					// Send CLOSE to all relays.
					let close_msg = json!(["CLOSE", sub_id]).to_string();
					for (_, sink) in &mut sinks {
						let _ =
							sink.send(WsMessage::Text(close_msg.clone().into())).await;
						let _ = sink.close().await;
					}
					send(&notif_tx, AsyncNostrNotification::Disconnected);
					return;
				}
				Ok(NostrCmd::PublishNote { content }) => {
					if let NostrIdentity::Keypair(kp) = &identity {
						let event = NostrEvent::new_text_note(&content, kp);
						let id = event.id.clone();
						let msg =
							json!(["EVENT", event]).to_string();
						for (url, sink) in &mut sinks {
							if let Err(e) =
								sink.send(WsMessage::Text(msg.clone().into())).await
							{
								log::warn!("nostr: EVENT to {url}: {e}");
							}
						}
						send(
							&notif_tx,
							AsyncNostrNotification::NotePublished(id),
						);
					} else {
						send(
							&notif_tx,
							AsyncNostrNotification::Error(
								"read-only mode: cannot publish".to_owned(),
							),
						);
					}
				}
				Ok(NostrCmd::SubscribeGlobal) => {
					let msg = json!(["REQ", sub_id, {
						"kinds": [1, 0],
						"limit": TIMELINE_LIMIT
					}])
					.to_string();
					for (_, sink) in &mut sinks {
						let _ = sink.send(WsMessage::Text(msg.clone().into())).await;
					}
				}
				Err(crossbeam_channel::TryRecvError::Empty) => break,
				Err(crossbeam_channel::TryRecvError::Disconnected) => {
					send(&notif_tx, AsyncNostrNotification::Disconnected);
					return;
				}
			}
		}

		// Poll all streams for one message, then sleep briefly.
		let mut got_message = false;
		for stream in &mut streams {
			match stream.next().now_or_never() {
				Some(Some(Ok(WsMessage::Text(text)))) => {
					got_message = true;
					handle_relay_message(text.as_str(), &notif_tx);
				}
				Some(Some(Err(e))) => {
					log::debug!("nostr ws error: {e}");
				}
				_ => {}
			}
		}

		if !got_message {
			tokio::time::sleep(Duration::from_millis(50)).await;
		}
	}
}

fn handle_relay_message(
	text: &str,
	notif_tx: &Sender<AsyncNostrNotification>,
) {
	let v: Value = match serde_json::from_str(text) {
		Ok(v) => v,
		Err(_) => return,
	};

	let arr = match v.as_array() {
		Some(a) if !a.is_empty() => a,
		_ => return,
	};

	let msg_type = arr[0].as_str().unwrap_or("");

	if msg_type == "EVENT" && arr.len() >= 3 {
		let event: NostrEvent = match serde_json::from_value(arr[2].clone()) {
			Ok(e) => e,
			Err(_) => return,
		};

		match event.kind {
			1 => {
				send(notif_tx, AsyncNostrNotification::TextNote(Box::new(event)));
			}
			0 => {
				// Parse metadata content (nested JSON string).
				if let Ok(meta) =
					serde_json::from_str::<Value>(&event.content)
				{
					let display = meta
						.get("display_name")
						.and_then(Value::as_str)
						.or_else(|| meta.get("name").and_then(Value::as_str))
						.unwrap_or("<unknown>")
						.to_owned();
					send(
						notif_tx,
						AsyncNostrNotification::Profile {
							pubkey: event.pubkey,
							display,
						},
					);
				}
			}
			_ => {}
		}
	}
}

fn send(
	tx: &Sender<AsyncNostrNotification>,
	notif: AsyncNostrNotification,
) {
	if let Err(e) = tx.send(notif) {
		log::error!("nostr notification send: {e}");
	}
}
