//! `gnostr-chat` consumes the shared Nostr and git-note types through
//! `gnostr-p2p` so the protocol chain stays `asyncgit -> p2p -> chat`.
//!
//! If chat needs a new wire type, expose it from `p2p` instead of importing
//! `asyncgit` directly.
//!
//! Shared primitives are available through [`types`], while [`fractal`],
//! [`perfect_ip`], and [`perfect_ip_core`] expose the P2P repair and demo APIs.

pub mod command;
pub mod event;
#[cfg(feature = "fractal")]
pub mod fractal;
#[cfg(feature = "ipfs")]
pub mod ipfs;
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
pub use msg::{Msg, MsgKind, USER_NAME};
pub use p2p::{evt_loop, global_rt};
pub use session::{ChatNotification, ChatSession};
pub use tui::run_chat_tui;

pub use gnostr_p2p::fractal as p2p_fractal;
pub use gnostr_p2p::{perfect_ip, perfect_ip_core};
pub use gnostr_types as types;
