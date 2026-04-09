/// Async nostr client for gnostr-tui.
///
/// Follows the same `std::thread::spawn` + crossbeam-channel pattern used by
/// `AsyncPush` / `AsyncPull` in asyncgit.  A dedicated OS thread hosts a
/// single-threaded tokio runtime that drives nostr-sdk; events are forwarded
/// back to the TUI via a `Sender<AsyncNostrNotification>`.
///
/// Ported from nostr/src/infrastructure/subscription/nostr.rs.
use std::{
	sync::{Arc, Mutex},
	thread,
};

use crossbeam_channel::{unbounded, Sender};
use nostr_sdk::prelude::*;

use crate::error::{Error, Result};

use super::keys::NostrIdentity;

// ── public notification enum (mirrors AsyncGitNotification) ─────────────────

/// Notification type sent from the nostr background thread to the TUI event loop.
#[derive(Clone, Debug)]
pub enum AsyncNostrNotification {
	/// Client connected to relays and is ready.
	Connected,
	/// Client disconnected / shut down.
	Disconnected,
	/// A kind-1 text note arrived from a relay.
	TextNote(Box<Event>),
	/// Profile metadata (kind 0) arrived.
	Profile { pubkey: PublicKey, display: String },
	/// A note was successfully published; carries the resulting event id.
	NotePublished(EventId),
	/// An error occurred in the background thread.
	Error(String),
}

// ── commands sent TO the background thread ───────────────────────────────────

#[derive(Debug)]
pub(super) enum NostrCmd {
	/// Publish a kind-1 text note.
	PublishNote { content: String },
	/// Subscribe to a basic global feed (kind 1, limit 50).
	SubscribeGlobal,
	/// Subscribe to the user's contact-list (following) feed.
	SubscribeFollowing,
	/// Gracefully shut down.
	Shutdown,
}

// ── AsyncNostr ───────────────────────────────────────────────────────────────

/// Async nostr client handle.
///
/// Create it with [`AsyncNostr::new`], then call [`AsyncNostr::connect`] to
/// start the background thread.  Commands are sent via the typed helpers;
/// results arrive as [`AsyncNostrNotification`] on the channel you supplied.
pub struct AsyncNostr {
	cmd_tx: Sender<NostrCmd>,
	pending: Arc<Mutex<bool>>,
	notification_tx: Sender<AsyncNostrNotification>,
}

impl AsyncNostr {
	/// Create a new `AsyncNostr` that will forward events on `sender`.
	pub fn new(sender: Sender<AsyncNostrNotification>) -> Self {
		let (cmd_tx, _) = unbounded(); // placeholder; real one created in connect()
		Self {
			cmd_tx,
			pending: Arc::new(Mutex::new(false)),
			notification_tx: sender,
		}
	}

	/// Connect to the given relays and start the background event loop.
	/// Ported from nostr/src/main.rs client initialisation.
	pub fn connect(
		&mut self,
		identity: NostrIdentity,
		relay_urls: Vec<String>,
	) -> Result<()> {
		if self.is_pending() {
			return Ok(()); // already running
		}

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
				let rt = tokio::runtime::Builder::new_current_thread()
					.enable_all()
					.build()
					.expect("tokio runtime");
				rt.block_on(run(
					identity,
					relay_urls,
					cmd_rx,
					notification_tx,
				));
				if let Ok(mut p) = pending.lock() {
					*p = false;
				}
			})
			.map_err(|e| {
				Error::Generic(format!("thread spawn failed: {e}"))
			})?;

		Ok(())
	}

	/// Returns `true` while the background thread is running.
	pub fn is_pending(&self) -> bool {
		self.pending.lock().map_or(false, |g| *g)
	}

	// ── commands ─────────────────────────────────────────────────────────────

	/// Publish a kind-1 text note to all connected relays.
	pub fn publish_note(&self, content: String) -> Result<()> {
		self.send_cmd(NostrCmd::PublishNote { content })
	}

	/// Subscribe to the global feed (last 50 kind-1 notes from any author).
	pub fn subscribe_global(&self) -> Result<()> {
		self.send_cmd(NostrCmd::SubscribeGlobal)
	}

	/// Subscribe to the user's contact-list (following) feed.
	pub fn subscribe_following(&self) -> Result<()> {
		self.send_cmd(NostrCmd::SubscribeFollowing)
	}

	/// Gracefully shut down the relay connections and background thread.
	pub fn shutdown(&self) -> Result<()> {
		self.send_cmd(NostrCmd::Shutdown)
	}

	fn send_cmd(&self, cmd: NostrCmd) -> Result<()> {
		self.cmd_tx.send(cmd).map_err(|e| {
			Error::Generic(format!("nostr cmd send failed: {e}"))
		})
	}
}

// ── background async loop ────────────────────────────────────────────────────

/// Number of kind-1 events to pull on the initial subscription.
const TIMELINE_LIMIT: usize = 50;
/// Timeout when fetching the contact list.
const CONTACT_LIST_TIMEOUT_SECS: u64 = 10;

async fn run(
	identity: NostrIdentity,
	relay_urls: Vec<String>,
	cmd_rx: crossbeam_channel::Receiver<NostrCmd>,
	notif_tx: Sender<AsyncNostrNotification>,
) {
	// Build the nostr-sdk client — ported from nostr/src/main.rs
	let client = match &identity {
		NostrIdentity::Keypair(keys) => Client::new(keys.clone()),
		NostrIdentity::ReadOnly(pk) => {
			use crate::nostr::signer::PublicKeySigner;
			Client::new(PublicKeySigner::new(*pk))
		}
	};

	for url in &relay_urls {
		if let Err(e) = client.add_relay(url.as_str()).await {
			log::warn!("nostr: add relay {url}: {e}");
		}
	}
	client.connect().await;
	send(&notif_tx, AsyncNostrNotification::Connected);

	// Subscribe to notifications from the relay pool
	let mut notifications = client.notifications();

	loop {
		// Drain pending commands (non-blocking).
		loop {
			match cmd_rx.try_recv() {
				Ok(NostrCmd::Shutdown) => {
					client.disconnect().await;
					send(&notif_tx, AsyncNostrNotification::Disconnected);
					return;
				}
				Ok(NostrCmd::PublishNote { content }) => {
					if identity.can_sign() {
						let builder = EventBuilder::text_note(content);
						match client.send_event_builder(builder).await {
							Ok(out) => {
								let id = out
									.id()
									.unwrap_or(EventId::all_zeros());
								send(
									&notif_tx,
									AsyncNostrNotification::NotePublished(
										id,
									),
								);
							}
							Err(e) => {
								send(
									&notif_tx,
									AsyncNostrNotification::Error(
										format!("publish failed: {e}"),
									),
								);
							}
						}
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
					// Ported from nostr/src/infrastructure/subscription/nostr.rs
					let filter = Filter::new()
						.kinds([Kind::TextNote, Kind::Metadata])
						.limit(TIMELINE_LIMIT);
					if let Err(e) =
						client.subscribe(vec![filter], None).await
					{
						send(
							&notif_tx,
							AsyncNostrNotification::Error(format!(
								"subscribe global: {e}"
							)),
						);
					}
				}
				Ok(NostrCmd::SubscribeFollowing) => {
					// Fetch contact list then subscribe — mirrors
					// nostr/src/infrastructure/subscription/nostr.rs
					// initialize_timeline()
					match client
						.get_contact_list_public_keys(
							std::time::Duration::from_secs(
								CONTACT_LIST_TIMEOUT_SECS,
							),
						)
						.await
					{
						Ok(following) => {
							let fwd = Filter::new()
								.authors(following.clone())
								.kinds([Kind::TextNote])
								.since(Timestamp::now());
							let bwd = Filter::new()
								.authors(following.clone())
								.kinds([Kind::TextNote])
								.until(Timestamp::now())
								.limit(TIMELINE_LIMIT);
							let meta = Filter::new()
								.authors(following)
								.kinds([Kind::Metadata]);
							if let Err(e) = client
								.subscribe(vec![fwd, bwd, meta], None)
								.await
							{
								send(
									&notif_tx,
									AsyncNostrNotification::Error(
										format!("subscribe following: {e}"),
									),
								);
							}
						}
						Err(e) => {
							send(
								&notif_tx,
								AsyncNostrNotification::Error(format!(
									"contact list fetch failed: {e}"
								)),
							);
						}
					}
				}
				Err(crossbeam_channel::TryRecvError::Empty) => break,
				Err(crossbeam_channel::TryRecvError::Disconnected) => {
					client.disconnect().await;
					send(&notif_tx, AsyncNostrNotification::Disconnected);
					return;
				}
			}
		}

		// Process one relay notification per iteration.
		match notifications.try_recv() {
			Ok(RelayPoolNotification::Event { event, .. }) => {
				match event.kind {
					Kind::TextNote => {
						send(
							&notif_tx,
							AsyncNostrNotification::TextNote(
								Box::new(*event),
							),
						);
					}
					Kind::Metadata => {
						if let Ok(meta) =
							Metadata::from_json(event.content.clone())
						{
							let display = meta
								.display_name
								.as_deref()
								.or(meta.name.as_deref())
								.unwrap_or("<unknown>")
								.to_owned();
							send(
								&notif_tx,
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
			Ok(RelayPoolNotification::RelayStatus {
				relay_url,
				status,
			}) => {
				log::debug!("nostr relay {relay_url}: {status:?}");
			}
			Ok(_) | Err(_) => {}
		}

		tokio::time::sleep(std::time::Duration::from_millis(50)).await;
	}
}

fn send(
	tx: &Sender<AsyncNostrNotification>,
	notif: AsyncNostrNotification,
) {
	if let Err(e) = tx.send(notif) {
		log::error!("nostr notification send failed: {e}");
	}
}
