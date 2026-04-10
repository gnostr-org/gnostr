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
	connect_async, tungstenite::Message as WsMessage, MaybeTlsStream,
	WebSocketStream,
};

type WsSink =
	SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
type WsStream =
	SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

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
	Profile {
		/// Hex-encoded public key of the profile owner.
		pubkey: String,
		/// Display name or name from the metadata.
		display: String,
	},
	/// A note was published; carries the hex event id.
	NotePublished(String),
	/// NIP-34 patch (kind 1617) received.
	RepoPatch(Box<super::nip34::GitPatch>),
	/// NIP-34 issue (kind 1621) received.
	RepoIssue(Box<super::nip34::GitIssue>),
	/// NIP-34 status update (kind 1630–1633) received.
	RepoStatus {
		/// Target event id (patch or issue).
		target_id: String,
		/// New status.
		status: super::nip34::PatchStatus,
	},
	/// A NIP-34 repository announcement (kind 30617) received from a relay.
	RepoAnnouncement(Box<super::nip34::GitRepoAnnouncement>),
	/// Repository was announced successfully; carries the event id.
	RepoAnnounced(String),
	/// Patch was submitted; carries the event id.
	PatchSubmitted(String),
	/// Issue was submitted; carries the event id.
	IssueSubmitted(String),
	/// An error in the background thread.
	Error(String),
}

// ── minimal nostr event ───────────────────────────────────────────────────────

/// A nostr event (NIP-01).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NostrEvent {
	/// Hex-encoded SHA-256 event id.
	pub id: String,
	/// Hex-encoded x-only public key of the author.
	pub pubkey: String,
	/// Unix timestamp (seconds).
	pub created_at: u64,
	/// Event kind number.
	pub kind: u64,
	/// Tags array.
	pub tags: Vec<Vec<String>>,
	/// Event content.
	pub content: String,
	/// Hex-encoded Schnorr signature.
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
		let serialised =
			json!([0, pubkey, created_at, kind, tags, content])
				.to_string();
		let mut h = Sha256::new();
		h.update(serialised.as_bytes());
		hex::encode(h.finalize())
	}

	/// Build and schnorr-sign a new text note (kind 1).
	pub fn new_text_note(content: &str, kp: &Keypair) -> Self {
		let (xonly, _) = XOnlyPublicKey::from_keypair(kp);
		let pubkey = hex::encode(xonly.serialize());
		let created_at = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();
		let kind = 1u64;
		let tags: Vec<Vec<String>> = vec![];
		let id = Self::compute_id(
			&pubkey, created_at, kind, &tags, content,
		);

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
	/// Subscribe to NIP-34 patches and issues for a repository.
	FetchRepoItems {
		/// Full `a`-tag string: `30617:<maintainer-pubkey>:<repo-id>`.
		repo_a_tag: String,
	},
	/// Subscribe to all NIP-34 event kinds (broad query, no repo filter).
	FetchAllNip34,
	/// Announce the current repository (kind 30617).
	AnnounceRepo {
		/// Short repo id (kebab-case).
		repo_id: String,
		/// Human-readable name.
		name: String,
		/// Brief description.
		description: String,
		/// Git clone URLs.
		clone_urls: Vec<String>,
		/// Web URLs.
		web_urls: Vec<String>,
		/// Relay URLs this repo will monitor.
		relay_urls: Vec<String>,
		/// Earliest unique commit id.
		earliest_commit: Option<String>,
	},
	/// Submit a NIP-34 patch (kind 1617).
	SubmitPatch {
		/// Raw `git format-patch` output.
		patch_content: String,
		/// Full `a`-tag for the target repo.
		repo_a_tag: String,
		/// Maintainer pubkey (hex) for the `p` tag.
		maintainer_pubkey: String,
		/// Commit id being proposed.
		commit_id: Option<String>,
		/// Parent commit id.
		parent_commit_id: Option<String>,
	},
	/// Submit a NIP-34 issue (kind 1621).
	SubmitIssue {
		/// Issue subject / title.
		subject: String,
		/// Markdown body.
		body: String,
		/// Full `a`-tag for the target repo.
		repo_a_tag: String,
		/// Maintainer pubkey (hex).
		maintainer_pubkey: String,
		/// Optional labels.
		labels: Vec<String>,
	},
	/// Set status on a patch or issue (kind 1630–1633).
	SetStatus {
		/// Target event id (hex).
		target_event_id: String,
		/// Target author pubkey (hex).
		target_author_pubkey: String,
		/// Full `a`-tag for the repo.
		repo_a_tag: String,
		/// New status.
		status: super::nip34::PatchStatus,
		/// Optional comment.
		comment: String,
	},
	/// Gracefully shut down.
	Shutdown,
}

// ── AsyncNostr ────────────────────────────────────────────────────────────────

/// Async nostr client handle.
pub struct AsyncNostr {
	cmd_tx: Sender<NostrCmd>,
	pending: Arc<Mutex<bool>>,
	notification_tx: Sender<AsyncNostrNotification>,
	/// Hex-encoded x-only public key of the connected identity.
	own_pubkey: Option<String>,
}

impl AsyncNostr {
	/// Create a new `AsyncNostr` that forwards events on `sender`.
	pub fn new(sender: Sender<AsyncNostrNotification>) -> Self {
		let (cmd_tx, _) = unbounded();
		Self {
			cmd_tx,
			pending: Arc::new(Mutex::new(false)),
			notification_tx: sender,
			own_pubkey: None,
		}
	}

	/// Return the hex-encoded public key of the connected identity, if known.
	pub fn own_pubkey_hex(&self) -> Option<String> {
		self.own_pubkey.clone()
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

		// Cache the public key for use from the main thread.
		self.own_pubkey = {
			let xonly = identity.public_key();
			Some(hex::encode(xonly.serialize()))
		};

		let (cmd_tx, cmd_rx) = unbounded::<NostrCmd>();
		self.cmd_tx = cmd_tx;

		let notification_tx = self.notification_tx.clone();
		let pending = Arc::clone(&self.pending);
		*pending.lock().map_err(|_| {
			Error::Generic("mutex poisoned".to_owned())
		})? = true;

		thread::Builder::new()
			.name("asyncnostr".into())
			.spawn(move || {
				// Catch any panics so the global hook (which would eprintln!
				// and corrupt the TUI) is never reached from this thread.
				let result = std::panic::catch_unwind(
					std::panic::AssertUnwindSafe(|| {
						let rt = match tokio::runtime::Builder::new_current_thread()
								.enable_all()
								.build()
							{
								Ok(rt) => rt,
								Err(e) => {
									log::error!(
										"nostr: tokio runtime build failed: {e}"
									);
									let _ = notification_tx.send(
										AsyncNostrNotification::Error(
											format!("runtime: {e}"),
										),
									);
									return;
								}
							};
						rt.block_on(run(
							identity,
							relay_urls,
							cmd_rx,
							notification_tx,
						));
					}),
				);
				if let Err(e) = result {
					log::error!(
						"nostr: asyncnostr thread panicked: {:?}",
						e
					);
				}
				if let Ok(mut p) = pending.lock() {
					*p = false;
				}
			})
			.map_err(|e| {
				Error::Generic(format!("thread spawn: {e}"))
			})?;

		Ok(())
	}

	/// Returns `true` while the background thread is running.
	pub fn is_pending(&self) -> bool {
		self.pending.lock().map_or(false, |g| *g)
	}

	/// Publish a kind-1 text note to all connected relays.
	pub fn publish_note(&self, content: String) -> Result<()> {
		self.send_cmd(NostrCmd::PublishNote { content })
	}

	/// Subscribe to the global feed (last N kind-1 notes from any author).
	pub fn subscribe_global(&self) -> Result<()> {
		self.send_cmd(NostrCmd::SubscribeGlobal)
	}

	/// Subscribe to NIP-34 patches and issues for a repository.
	///
	/// `repo_a_tag` is `30617:<maintainer-pubkey>:<repo-id>`.
	pub fn fetch_repo_items(&self, repo_a_tag: String) -> Result<()> {
		self.send_cmd(NostrCmd::FetchRepoItems { repo_a_tag })
	}

	/// Subscribe to all NIP-34 event kinds (broad query).
	pub fn fetch_all_nip34(&self) -> Result<()> {
		self.send_cmd(NostrCmd::FetchAllNip34)
	}

	/// Announce the current repository as a NIP-34 kind 30617 event.
	pub fn announce_repo(
		&self,
		repo_id: String,
		name: String,
		description: String,
		clone_urls: Vec<String>,
		web_urls: Vec<String>,
		relay_urls: Vec<String>,
		earliest_commit: Option<String>,
	) -> Result<()> {
		self.send_cmd(NostrCmd::AnnounceRepo {
			repo_id,
			name,
			description,
			clone_urls,
			web_urls,
			relay_urls,
			earliest_commit,
		})
	}

	/// Submit a NIP-34 patch (kind 1617).
	pub fn submit_patch(
		&self,
		patch_content: String,
		repo_a_tag: String,
		maintainer_pubkey: String,
		commit_id: Option<String>,
		parent_commit_id: Option<String>,
	) -> Result<()> {
		self.send_cmd(NostrCmd::SubmitPatch {
			patch_content,
			repo_a_tag,
			maintainer_pubkey,
			commit_id,
			parent_commit_id,
		})
	}

	/// Submit a NIP-34 issue (kind 1621).
	pub fn submit_issue(
		&self,
		subject: String,
		body: String,
		repo_a_tag: String,
		maintainer_pubkey: String,
		labels: Vec<String>,
	) -> Result<()> {
		self.send_cmd(NostrCmd::SubmitIssue {
			subject,
			body,
			repo_a_tag,
			maintainer_pubkey,
			labels,
		})
	}

	/// Set the status of a patch or issue (kind 1630–1633).
	pub fn set_status(
		&self,
		target_event_id: String,
		target_author_pubkey: String,
		repo_a_tag: String,
		status: super::nip34::PatchStatus,
		comment: String,
	) -> Result<()> {
		self.send_cmd(NostrCmd::SetStatus {
			target_event_id,
			target_author_pubkey,
			repo_a_tag,
			status,
			comment,
		})
	}

	/// Gracefully shut down relay connections and the background thread.
	pub fn shutdown(&self) -> Result<()> {
		self.send_cmd(NostrCmd::Shutdown)
	}

	fn send_cmd(&self, cmd: NostrCmd) -> Result<()> {
		self.cmd_tx.send(cmd).map_err(|e| {
			Error::Generic(format!("nostr cmd send: {e}"))
		})
	}
}

// ── background async loop ─────────────────────────────────────────────────────

const TIMELINE_LIMIT: usize = 50;

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
					AsyncNostrNotification::Error(format!(
						"connect {url}: {e}"
					)),
				);
			}
		}
	}

	if sinks.is_empty() {
		send(
			&notif_tx,
			AsyncNostrNotification::Error(
				"no relays connected".to_owned(),
			),
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
		if let Err(e) =
			sink.send(WsMessage::Text(sub_msg.clone().into())).await
		{
			log::warn!("nostr: REQ to {url}: {e}");
		}
	}

	// Immediately subscribe to all NIP-34 event kinds so the Nostr tab
	// populates on connect without any manual trigger.
	let nip34_msg = json!([
		"REQ",
		"gnostr-nip34-all",
		super::nip34::all_nip34_filter()
	])
	.to_string();
	for (url, sink) in &mut sinks {
		if let Err(e) =
			sink.send(WsMessage::Text(nip34_msg.clone().into())).await
		{
			log::warn!("nostr: NIP-34 REQ to {url}: {e}");
		}
	}

	// Fuse all relay streams into one via tokio::select polling.
	loop {
		// Drain commands first (non-blocking).
		loop {
			match cmd_rx.try_recv() {
				Ok(NostrCmd::Shutdown) => {
					// Send CLOSE to all relays.
					let close_msg =
						json!(["CLOSE", sub_id]).to_string();
					for (_, sink) in &mut sinks {
						let _ = sink
							.send(WsMessage::Text(
								close_msg.clone().into(),
							))
							.await;
						let _ = sink.close().await;
					}
					send(
						&notif_tx,
						AsyncNostrNotification::Disconnected,
					);
					return;
				}
				Ok(NostrCmd::PublishNote { content }) => {
					if let NostrIdentity::Keypair(kp) = &identity {
						let event =
							NostrEvent::new_text_note(&content, kp);
						let id = event.id.clone();
						let msg = json!(["EVENT", event]).to_string();
						for (url, sink) in &mut sinks {
							if let Err(e) = sink
								.send(WsMessage::Text(
									msg.clone().into(),
								))
								.await
							{
								log::warn!(
									"nostr: EVENT to {url}: {e}"
								);
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
								"read-only mode: cannot publish"
									.to_owned(),
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
						let _ = sink
							.send(WsMessage::Text(msg.clone().into()))
							.await;
					}
				}
				Ok(NostrCmd::FetchRepoItems { repo_a_tag }) => {
					let filter =
						super::nip34::repo_filter(&repo_a_tag);
					let msg = json!(["REQ", "gnostr-nip34", filter])
						.to_string();
					for (url, sink) in &mut sinks {
						if let Err(e) = sink
							.send(WsMessage::Text(msg.clone().into()))
							.await
						{
							log::warn!(
								"nostr: NIP-34 REQ to {url}: {e}"
							);
						}
					}
				}
				Ok(NostrCmd::FetchAllNip34) => {
					let filter = super::nip34::all_nip34_filter();
					let msg =
						json!(["REQ", "gnostr-nip34-all", filter])
							.to_string();
					for (url, sink) in &mut sinks {
						if let Err(e) = sink
							.send(WsMessage::Text(msg.clone().into()))
							.await
						{
							log::warn!(
								"nostr: NIP-34 all REQ to {url}: {e}"
							);
						}
					}
				}
				Ok(NostrCmd::AnnounceRepo {
					repo_id,
					name,
					description,
					clone_urls,
					web_urls,
					relay_urls,
					earliest_commit,
				}) => {
					if let NostrIdentity::Keypair(kp) = &identity {
						match super::nip34::build_repo_announcement(
							&repo_id,
							&name,
							&description,
							&clone_urls,
							&web_urls,
							&relay_urls,
							earliest_commit.as_deref(),
							kp,
						) {
							Ok(event) => {
								let id = event.id.clone();
								let msg = json!(["EVENT", event])
									.to_string();
								for (url, sink) in &mut sinks {
									if let Err(e) = sink
										.send(WsMessage::Text(
											msg.clone().into(),
										))
										.await
									{
										log::warn!("nostr: announce to {url}: {e}");
									}
								}
								send(
									&notif_tx,
									AsyncNostrNotification::RepoAnnounced(id),
								);
							}
							Err(e) => send(
								&notif_tx,
								AsyncNostrNotification::Error(
									format!("announce build: {e}"),
								),
							),
						}
					} else {
						send(
							&notif_tx,
							AsyncNostrNotification::Error(
								"read-only mode: cannot announce"
									.to_owned(),
							),
						);
					}
				}
				Ok(NostrCmd::SubmitPatch {
					patch_content,
					repo_a_tag,
					maintainer_pubkey,
					commit_id,
					parent_commit_id,
				}) => {
					if let NostrIdentity::Keypair(kp) = &identity {
						match super::nip34::build_patch(
							&patch_content,
							&repo_a_tag,
							&maintainer_pubkey,
							commit_id.as_deref(),
							parent_commit_id.as_deref(),
							true,
							kp,
						) {
							Ok(event) => {
								let id = event.id.clone();
								let msg = json!(["EVENT", event])
									.to_string();
								for (url, sink) in &mut sinks {
									if let Err(e) = sink
										.send(WsMessage::Text(
											msg.clone().into(),
										))
										.await
									{
										log::warn!("nostr: patch to {url}: {e}");
									}
								}
								send(
									&notif_tx,
									AsyncNostrNotification::PatchSubmitted(id),
								);
							}
							Err(e) => send(
								&notif_tx,
								AsyncNostrNotification::Error(
									format!("patch build: {e}"),
								),
							),
						}
					} else {
						send(
							&notif_tx,
							AsyncNostrNotification::Error(
								"read-only mode: cannot submit patch"
									.to_owned(),
							),
						);
					}
				}
				Ok(NostrCmd::SubmitIssue {
					subject,
					body,
					repo_a_tag,
					maintainer_pubkey,
					labels,
				}) => {
					if let NostrIdentity::Keypair(kp) = &identity {
						match super::nip34::build_issue(
							&subject,
							&body,
							&repo_a_tag,
							&maintainer_pubkey,
							&labels,
							kp,
						) {
							Ok(event) => {
								let id = event.id.clone();
								let msg = json!(["EVENT", event])
									.to_string();
								for (url, sink) in &mut sinks {
									if let Err(e) = sink
										.send(WsMessage::Text(
											msg.clone().into(),
										))
										.await
									{
										log::warn!("nostr: issue to {url}: {e}");
									}
								}
								send(
									&notif_tx,
									AsyncNostrNotification::IssueSubmitted(id),
								);
							}
							Err(e) => send(
								&notif_tx,
								AsyncNostrNotification::Error(
									format!("issue build: {e}"),
								),
							),
						}
					} else {
						send(
							&notif_tx,
							AsyncNostrNotification::Error(
								"read-only mode: cannot submit issue"
									.to_owned(),
							),
						);
					}
				}
				Ok(NostrCmd::SetStatus {
					target_event_id,
					target_author_pubkey,
					repo_a_tag,
					status,
					comment,
				}) => {
					if let NostrIdentity::Keypair(kp) = &identity {
						match super::nip34::build_status(
							status,
							&target_event_id,
							&target_author_pubkey,
							&repo_a_tag,
							&comment,
							kp,
						) {
							Ok(event) => {
								let msg = json!(["EVENT", event])
									.to_string();
								for (url, sink) in &mut sinks {
									if let Err(e) = sink
										.send(WsMessage::Text(
											msg.clone().into(),
										))
										.await
									{
										log::warn!("nostr: status to {url}: {e}");
									}
								}
							}
							Err(e) => send(
								&notif_tx,
								AsyncNostrNotification::Error(
									format!("status build: {e}"),
								),
							),
						}
					}
				}
				Err(crossbeam_channel::TryRecvError::Empty) => break,
				Err(
					crossbeam_channel::TryRecvError::Disconnected,
				) => {
					send(
						&notif_tx,
						AsyncNostrNotification::Disconnected,
					);
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
		let event: NostrEvent =
			match serde_json::from_value(arr[2].clone()) {
				Ok(e) => e,
				Err(_) => return,
			};

		match event.kind {
			1 => {
				send(
					notif_tx,
					AsyncNostrNotification::TextNote(Box::new(event)),
				);
			}
			0 => {
				// Parse metadata content (nested JSON string).
				if let Ok(meta) =
					serde_json::from_str::<Value>(&event.content)
				{
					let display = meta
						.get("display_name")
						.and_then(Value::as_str)
						.or_else(|| {
							meta.get("name").and_then(Value::as_str)
						})
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
			// NIP-34 patch
			super::nip34::KIND_PATCH => {
				if let Some(patch) = super::nip34::parse_patch(&event)
				{
					send(
						notif_tx,
						AsyncNostrNotification::RepoPatch(Box::new(
							patch,
						)),
					);
				}
			}
			// NIP-34 issue
			super::nip34::KIND_ISSUE => {
				if let Some(issue) = super::nip34::parse_issue(&event)
				{
					send(
						notif_tx,
						AsyncNostrNotification::RepoIssue(Box::new(
							issue,
						)),
					);
				}
			}
			// NIP-34 repository announcement
			super::nip34::KIND_REPO_ANNOUNCEMENT => {
				if let Some(ann) =
					super::nip34::parse_repo_announcement(&event)
				{
					send(
						notif_tx,
						AsyncNostrNotification::RepoAnnouncement(
							Box::new(ann),
						),
					);
				}
			}
			// NIP-34 status events
			k @ (super::nip34::KIND_STATUS_OPEN
			| super::nip34::KIND_STATUS_APPLIED
			| super::nip34::KIND_STATUS_CLOSED
			| super::nip34::KIND_STATUS_DRAFT) => {
				if let Some(status) =
					super::nip34::PatchStatus::from_kind(k)
				{
					// Target event id is in the first `e` tag.
					let target_id = event
						.tags
						.iter()
						.find(|t| {
							t.first()
								.map(|s| s == "e")
								.unwrap_or(false)
						})
						.and_then(|t| t.get(1).cloned())
						.unwrap_or_default();
					if !target_id.is_empty() {
						send(
							notif_tx,
							AsyncNostrNotification::RepoStatus {
								target_id,
								status,
							},
						);
					}
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
