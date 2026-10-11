//! `gnostr-chat` consumes the shared Nostr and git-note types through
//! `gnostr-p2p` so the protocol chain stays `asyncgit -> p2p -> chat`.
//!
//! If chat needs a new wire type, expose it from `p2p` instead of importing
//! `asyncgit` directly.

pub mod command;
pub mod event;
pub mod message;
pub mod msg;
pub mod p2p;
pub mod session;
pub mod tui;

#[cfg(test)]
pub mod tests;

pub use command::{chat, run, ChatSubCommands};
pub use event::ChatEvent;
pub use message::*;
pub use p2p::{evt_loop, global_rt};
pub use session::{ChatNotification, ChatSession};
pub use tui::run_chat_tui;
