#![deny(clippy::pedantic)]

pub mod chat;
//pub mod css;
pub mod database;
pub mod git;
//pub mod js;
pub mod kill_process;
pub mod layers;
pub mod layout_html;
pub mod methods;
pub mod syntax_highlight;
pub mod template_html;
pub mod unified_diff_builder;
pub mod websock_index_html;
pub use crate::web::{
    database::schema::{commit::Commit, repository::Repository, tag::Tag},
    git::Git,
    syntax_highlight::prime_highlighters,
    unified_diff_builder::UnifiedDiffBuilder,
};

use std::sync::OnceLock;

pub mod startup;
pub use startup::*;

// Constants that were previously in main.rs
pub const CRATE_VERSION: &str = clap::crate_version!();
pub const GLOBAL_CSS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/src/lib/css/style.css"));
pub const GLOBAL_CSS_HASH: &str = const_hex::Buffer::<16, false>::new()
    .const_format(&xxhash_rust::const_xxh3::xxh3_128(GLOBAL_CSS).to_be_bytes())
    .as_str();
pub const JS_BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/src/lib/js/bundle.js"));

pub static HIGHLIGHT_CSS_HASH: OnceLock<&'static str> = OnceLock::new();
pub static HIGHLIGHT_CSS_BYTES: OnceLock<&'static [u8]> = OnceLock::new();
pub static DARK_HIGHLIGHT_CSS_HASH: OnceLock<&'static str> = OnceLock::new();
pub static DARK_HIGHLIGHT_CSS_BYTES: OnceLock<&'static [u8]> = OnceLock::new();
pub static JS_BUNDLE_HASH: OnceLock<&'static str> = OnceLock::new();

pub const ADD_RELAY_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/add-relay.svg"
));
pub static ADD_RELAY_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const CLOSE_MODAL_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/close-modal.svg"
));
pub static CLOSE_MODAL_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const CONTENT_WARNING_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/content-warning.svg"
));
pub static CONTENT_WARNING_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EDIT_PROFILE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/edit-profile.svg"
));
pub static EDIT_PROFILE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EVENT_DELETE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/event-delete.svg"
));
pub static EVENT_DELETE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EVENT_DETAILS_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/event-details.svg"
));
pub static EVENT_DETAILS_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EVENT_LIKE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/event-like.svg"
));
pub static EVENT_LIKE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EVENT_LIKED_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/event-liked.svg"
));
pub static EVENT_LIKED_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EVENT_OPTIONS_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/event-options.svg"
));
pub static EVENT_OPTIONS_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EVENT_REPLY_ALL_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/event-reply-all.svg"
));
pub static EVENT_REPLY_ALL_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EVENT_REPLY_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/event-reply.svg"
));
pub static EVENT_REPLY_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EVENT_SHARE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/event-share.svg"
));
pub static EVENT_SHARE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EXPLORE_ACTIVE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/explore-active.svg"
));
pub static EXPLORE_ACTIVE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const EXPLORE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/explore.svg"
));
pub static EXPLORE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const FAVICON_NOTIF_ICO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/favicon-notif.ico"
));
pub static FAVICON_NOTIF_ICO_HASH: OnceLock<&'static str> = OnceLock::new();

pub const FAVICON_ICO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/favicon.ico"
));
pub static FAVICON_ICO_HASH: OnceLock<&'static str> = OnceLock::new();

pub const GNOSTR_NOTIF_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/gnostr_notif.svg"
));
pub static GNOSTR_NOTIF_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const GNOSTR_NOBG_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/gnostr-nobg.svg"
));
pub static GNOSTR_NOBG_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const GNOSTR_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/gnostr.svg"
));
pub static GNOSTR_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const HOME_ACTIVE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/home-active.svg"
));
pub static HOME_ACTIVE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const HOME_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/home.svg"
));
pub static HOME_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const ICON_MASKABLE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/icon-maskable.svg"
));
pub static ICON_MASKABLE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const ICON_ICNS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/icon.icns"
));
pub static ICON_ICNS_HASH: OnceLock<&'static str> = OnceLock::new();

pub const ICON_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/icon.svg"
));
pub static ICON_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const KEY_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/key.svg"
));
pub static KEY_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const LOADER_FRAGMENT_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/loader-fragment.svg"
));
pub static LOADER_FRAGMENT_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const LOGO_INVERTED_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/logo-inverted.svg"
));
pub static LOGO_INVERTED_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const LOGO_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/logo.svg"
));
pub static LOGO_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const MESSAGE_USER_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/message-user.svg"
));
pub static MESSAGE_USER_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const MESSAGES_ACTIVE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/messages-active.svg"
));
pub static MESSAGES_ACTIVE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const MESSAGES_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/messages.svg"
));
pub static MESSAGES_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const NEW_NOTE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/new-note.svg"
));
pub static NEW_NOTE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const NO_USER_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/no-user.svg"
));
pub static NO_USER_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const NOTIFICATIONS_ACTIVE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/notifications-active.svg"
));
pub static NOTIFICATIONS_ACTIVE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const NOTIFICATIONS_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/notifications.svg"
));
pub static NOTIFICATIONS_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const OPEN_THREAD_HERE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/open-thread-here.svg"
));
pub static OPEN_THREAD_HERE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const OPEN_THREAD_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/open-thread.svg"
));
pub static OPEN_THREAD_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const PROFILE_WEBSITE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/profile-website.svg"
));
pub static PROFILE_WEBSITE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const PROFILE_ZAP_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/profile-zap.svg"
));
pub static PROFILE_ZAP_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const PUBKEY_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/pubkey.svg"
));
pub static PUBKEY_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const READ_MORE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/read-more.svg"
));
pub static READ_MORE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const SETTINGS_ACTIVE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/settings-active.svg"
));
pub static SETTINGS_ACTIVE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const SETTINGS_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/settings.svg"
));
pub static SETTINGS_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const SIGN_OUT_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/sign-out.svg"
));
pub static SIGN_OUT_SVG_HASH: OnceLock<&'static str> = OnceLock::new();
