#![deny(clippy::pedantic)]
use std::collections::HashMap;
use handlebars::Handlebars;
use clap::Args;
use clap::Subcommand;
use clap::Parser;
use rkyv::Serialize as rkyvSerialize;
use std::env::Args as stdEnvArgs;
use serde::Serialize as serdeSerialize;

use warp::body::json as warpBodyJson;

use warp::reply::json as warpReplayJson;



pub mod chat;
//pub mod css;
pub mod database;
pub mod git;
//pub mod js;
pub mod kill_process;
pub mod layers;
pub mod methods;
pub mod syntax_highlight;
pub mod template_html;
pub mod layout_html;
pub mod unified_diff_builder;
pub mod websock_index_html;
pub use crate::web::{
    database::schema::{commit::Commit, repository::Repository, tag::Tag},
    git::Git,
    syntax_highlight::prime_highlighters,
    unified_diff_builder::UnifiedDiffBuilder,
};

// Re-export commonly used items for templates
pub use const_hex;
pub use std::sync::OnceLock;
pub use xxhash_rust;

// Constants that were previously in main.rs
pub const CRATE_VERSION: &str = clap::crate_version!();
pub const GLOBAL_CSS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/src/css/style.css"));
pub const GLOBAL_CSS_HASH: &str = const_hex::Buffer::<16, false>::new()
    .const_format(&xxhash_rust::const_xxh3::xxh3_128(GLOBAL_CSS).to_be_bytes())
    .as_str();
pub const JS_BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/src/js/bundle.js"));

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

pub const HOME_SVG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/images/home.svg"));
pub static HOME_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const ICON_MASKABLE_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/images/icon-maskable.svg"
));
pub static ICON_MASKABLE_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const ICON_ICNS: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/images/icon.icns"));
pub static ICON_ICNS_HASH: OnceLock<&'static str> = OnceLock::new();

pub const ICON_SVG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/images/icon.svg"));
pub static ICON_SVG_HASH: OnceLock<&'static str> = OnceLock::new();

pub const KEY_SVG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/images/key.svg"));
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

pub const LOGO_SVG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/images/logo.svg"));
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

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    net::IpAddr,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use rocksdb::{Options, SliceTransform};
use xxhash_rust::const_xxh3;

use crate::web::database::schema::prefixes::{
    COMMIT_COUNT_FAMILY, COMMIT_FAMILY, REFERENCE_FAMILY, REPOSITORY_FAMILY, TAG_FAMILY,
};

/// Configuration for the gnostr-gnit library
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to a directory where the RocksDB database should be stored
    pub db_store: PathBuf,
    /// The path in which your bare Git repositories reside
    pub scan_path: PathBuf,
    /// Configures the metadata refresh interval for Git repositories
    pub refresh_interval: RefreshInterval,
    /// The IP address to bind to
    pub bind_address: IpAddr,
    /// The socket port to bind to
    pub bind_port: u16,
    /// Configures the request timeout for incoming HTTP requests
    pub request_timeout: Duration,
    /// Enable debug logging
    pub debug: bool,
    /// Enable info logging
    pub info: bool,
    /// Run the process in the background (daemonize)
    pub detach: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_store: PathBuf::from(".gnostr/web"),
            scan_path: PathBuf::from("."),
            refresh_interval: RefreshInterval::Duration(Duration::from_secs(30)),
            bind_address: IpAddr::from_str("127.0.0.1").unwrap(),
            bind_port: 3333,
            request_timeout: Duration::from_secs(10),
            debug: false,
            info: false,
            detach: false,
        }
    }
}

/// Configures the metadata refresh interval for Git repositories
#[derive(Debug, Clone, Copy)]
pub enum RefreshInterval {
    /// Never refresh
    Never,
    /// Refresh after a duration
    Duration(Duration),
}

impl Display for RefreshInterval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Never => write!(f, "never"),
            Self::Duration(s) => write!(f, "{}", humantime::format_duration(*s)),
        }
    }
}

impl FromStr for RefreshInterval {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "never" {
            Ok(Self::Never)
        } else if let Ok(v) = humantime::parse_duration(s) {
            Ok(Self::Duration(v))
        } else {
            Err("must be seconds, a human readable duration (eg. '10m') or 'never'")
        }
    }
}

/// Opens and configures the RocksDB database
pub fn open_database(config: &Config) -> Result<Arc<rocksdb::DB>, anyhow::Error> {
    loop {
        let mut db_options = Options::default();
        db_options.create_missing_column_families(true);
        db_options.create_if_missing(true);

        let mut commit_family_options = Options::default();
        commit_family_options.set_prefix_extractor(SliceTransform::create(
            "commit_prefix",
            |input| input.split(|&c| c == b'\0').next().unwrap_or(input),
            None,
        ));

        let mut tag_family_options = Options::default();
        tag_family_options.set_prefix_extractor(SliceTransform::create_fixed_prefix(
            std::mem::size_of::<u64>(),
        ));

        let db = rocksdb::DB::open_cf_with_opts(
            &db_options,
            &config.db_store,
            vec![
                (COMMIT_FAMILY, commit_family_options),
                (REPOSITORY_FAMILY, Options::default()),
                (TAG_FAMILY, tag_family_options),
                (REFERENCE_FAMILY, Options::default()),
                (COMMIT_COUNT_FAMILY, Options::default()),
            ],
        )?;

        let needs_schema_regen = match db.get("schema_version")? {
            Some(v) if v.as_slice() != database::schema::SCHEMA_VERSION.as_bytes() => Some(Some(v)),
            Some(_) => None,
            None => {
                db.put("schema_version", database::schema::SCHEMA_VERSION)?;
                None
            }
        };

        if let Some(version) = needs_schema_regen {
            let old_version = version
                .as_deref()
                .map_or(Cow::Borrowed("unknown"), String::from_utf8_lossy);

            tracing::warn!(
                "Clearing outdated database ({old_version} != {})",
                database::schema::SCHEMA_VERSION
            );

            drop(db);
            rocksdb::DB::destroy(&Options::default(), &config.db_store)?;
        } else {
            break Ok(Arc::new(db));
        }
    }
}

/// Runs the repository indexer
pub async fn run_indexer(
    db: Arc<rocksdb::DB>,
    scan_path: PathBuf,
    refresh_interval: RefreshInterval,
) -> Result<(), tokio::task::JoinError> {
    let (indexer_wakeup_send, mut indexer_wakeup_recv) = tokio::sync::mpsc::channel(10);

    std::thread::spawn(move || loop {
        tracing::info!("Running periodic index");
        crate::web::database::indexer::run(&scan_path, &db);
        tracing::info!("Finished periodic index");

        if indexer_wakeup_recv.blocking_recv().is_none() {
            break;
        }
    });

    tokio::spawn({
        let mut sighup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())
            .expect("could not subscribe to sighup");
        let build_sleeper = move || async move {
            match refresh_interval {
                RefreshInterval::Never => futures_util::future::pending().await,
                RefreshInterval::Duration(v) => tokio::time::sleep(v).await,
            };
        };

        async move {
            loop {
                tokio::select! {
                    _ = sighup.recv() => {},
                    () = build_sleeper() => {},
                }

                if indexer_wakeup_send.send(()).await.is_err() {
                    tracing::error!(
                        "Indexing thread has died and is no longer accepting wakeup messages"
                    );
                }
            }
        }
    })
    .await
}

#[must_use]
pub fn build_asset_hash(v: &'static [u8]) -> &'static str {
    let hasher = const_xxh3::xxh3_128(v);
    let out = const_hex::encode(hasher.to_be_bytes());
    Box::leak(out.into_boxed_str())
}

/// Initializes all static asset hashes.
pub fn init_static_asset_hashes() {
    // These values are computed at runtime, not compile-time like GLOBAL_CSS_HASH
    // for now we set the css for dark and light here.
    let css = {
        let theme =
            toml::from_str::<crate::theme::Theme>(include_str!("../../../themes/solarized_light.toml"))
                .unwrap()
                .build_css();
        Box::leak(
            format!(r#"@media (prefers-color-scheme: light){{{theme}}}"#)
                .into_boxed_str()
                .into_boxed_bytes(),
        )
    };
    HIGHLIGHT_CSS_BYTES.set(css).unwrap();
    HIGHLIGHT_CSS_HASH
        .set(crate::web::build_asset_hash(css))
        .unwrap();

    let dark_css = {
        let theme =
            toml::from_str::<crate::theme::Theme>(include_str!("../../../themes/solarized_dark.toml"))
                .unwrap()
                .build_css();
        Box::leak(
            format!(r#"@media (prefers-color-scheme: dark){{{theme}}}"#)
                .into_boxed_str()
                .into_boxed_bytes(),
        )
    };
    DARK_HIGHLIGHT_CSS_BYTES.set(dark_css).unwrap();
    DARK_HIGHLIGHT_CSS_HASH
        .set(crate::web::build_asset_hash(dark_css))
        .unwrap();

    // Image hashes
    ADD_RELAY_SVG_HASH
        .set(crate::web::build_asset_hash(ADD_RELAY_SVG))
        .unwrap();
    CLOSE_MODAL_SVG_HASH
        .set(crate::web::build_asset_hash(CLOSE_MODAL_SVG))
        .unwrap();
    CONTENT_WARNING_SVG_HASH
        .set(crate::web::build_asset_hash(CONTENT_WARNING_SVG))
        .unwrap();
    EDIT_PROFILE_SVG_HASH
        .set(crate::web::build_asset_hash(EDIT_PROFILE_SVG))
        .unwrap();
    EVENT_DELETE_SVG_HASH
        .set(crate::web::build_asset_hash(EVENT_DELETE_SVG))
        .unwrap();
    EVENT_DETAILS_SVG_HASH
        .set(crate::web::build_asset_hash(EVENT_DETAILS_SVG))
        .unwrap();
    EVENT_LIKE_SVG_HASH
        .set(crate::web::build_asset_hash(EVENT_LIKE_SVG))
        .unwrap();
    EVENT_LIKED_SVG_HASH
        .set(crate::web::build_asset_hash(EVENT_LIKED_SVG))
        .unwrap();
    EVENT_OPTIONS_SVG_HASH
        .set(crate::web::build_asset_hash(EVENT_OPTIONS_SVG))
        .unwrap();
    EVENT_REPLY_ALL_SVG_HASH
        .set(crate::web::build_asset_hash(EVENT_REPLY_ALL_SVG))
        .unwrap();
    EVENT_REPLY_SVG_HASH
        .set(crate::web::build_asset_hash(EVENT_REPLY_SVG))
        .unwrap();
    EVENT_SHARE_SVG_HASH
        .set(crate::web::build_asset_hash(EVENT_SHARE_SVG))
        .unwrap();
    EXPLORE_ACTIVE_SVG_HASH
        .set(crate::web::build_asset_hash(EXPLORE_ACTIVE_SVG))
        .unwrap();
    EXPLORE_SVG_HASH
        .set(crate::web::build_asset_hash(EXPLORE_SVG))
        .unwrap();
    FAVICON_NOTIF_ICO_HASH
        .set(crate::web::build_asset_hash(FAVICON_NOTIF_ICO))
        .unwrap();
    FAVICON_ICO_HASH
        .set(crate::web::build_asset_hash(FAVICON_ICO))
        .unwrap();
    GNOSTR_NOTIF_SVG_HASH
        .set(crate::web::build_asset_hash(GNOSTR_NOTIF_SVG))
        .unwrap();
    GNOSTR_NOBG_SVG_HASH
        .set(crate::web::build_asset_hash(GNOSTR_NOBG_SVG))
        .unwrap();
    GNOSTR_SVG_HASH
        .set(crate::web::build_asset_hash(GNOSTR_SVG))
        .unwrap();
    HOME_ACTIVE_SVG_HASH
        .set(crate::web::build_asset_hash(HOME_ACTIVE_SVG))
        .unwrap();
    HOME_SVG_HASH
        .set(crate::web::build_asset_hash(HOME_SVG))
        .unwrap();
    ICON_MASKABLE_SVG_HASH
        .set(crate::web::build_asset_hash(ICON_MASKABLE_SVG))
        .unwrap();
    ICON_ICNS_HASH
        .set(crate::web::build_asset_hash(ICON_ICNS))
        .unwrap();
    ICON_SVG_HASH
        .set(crate::web::build_asset_hash(ICON_SVG))
        .unwrap();
    KEY_SVG_HASH.set(crate::web::build_asset_hash(KEY_SVG)).unwrap();
    LOADER_FRAGMENT_SVG_HASH
        .set(crate::web::build_asset_hash(LOADER_FRAGMENT_SVG))
        .unwrap();
    LOGO_INVERTED_SVG_HASH
        .set(crate::web::build_asset_hash(LOGO_INVERTED_SVG))
        .unwrap();
    LOGO_SVG_HASH
        .set(crate::web::build_asset_hash(LOGO_SVG))
        .unwrap();
    MESSAGE_USER_SVG_HASH
        .set(crate::web::build_asset_hash(MESSAGE_USER_SVG))
        .unwrap();
    MESSAGES_ACTIVE_SVG_HASH
        .set(crate::web::build_asset_hash(MESSAGES_ACTIVE_SVG))
        .unwrap();
    MESSAGES_SVG_HASH
        .set(crate::web::build_asset_hash(MESSAGES_SVG))
        .unwrap();
    NEW_NOTE_SVG_HASH
        .set(crate::web::build_asset_hash(NEW_NOTE_SVG))
        .unwrap();
    NO_USER_SVG_HASH
        .set(crate::web::build_asset_hash(NO_USER_SVG))
        .unwrap();
    NOTIFICATIONS_ACTIVE_SVG_HASH
        .set(crate::web::build_asset_hash(NOTIFICATIONS_ACTIVE_SVG))
        .unwrap();
    NOTIFICATIONS_SVG_HASH
        .set(crate::web::build_asset_hash(NOTIFICATIONS_SVG))
        .unwrap();
    OPEN_THREAD_HERE_SVG_HASH
        .set(crate::web::build_asset_hash(OPEN_THREAD_HERE_SVG))
        .unwrap();
    OPEN_THREAD_SVG_HASH
        .set(crate::web::build_asset_hash(OPEN_THREAD_SVG))
        .unwrap();
    PROFILE_WEBSITE_SVG_HASH
        .set(crate::web::build_asset_hash(PROFILE_WEBSITE_SVG))
        .unwrap();
    PROFILE_ZAP_SVG_HASH
        .set(crate::web::build_asset_hash(PROFILE_ZAP_SVG))
        .unwrap();
    PUBKEY_SVG_HASH
        .set(crate::web::build_asset_hash(PUBKEY_SVG))
        .unwrap();
    READ_MORE_SVG_HASH
        .set(crate::web::build_asset_hash(READ_MORE_SVG))
        .unwrap();
    SETTINGS_ACTIVE_SVG_HASH
        .set(crate::web::build_asset_hash(SETTINGS_ACTIVE_SVG))
        .unwrap();
    SETTINGS_SVG_HASH
        .set(crate::web::build_asset_hash(SETTINGS_SVG))
        .unwrap();
    SIGN_OUT_SVG_HASH
        .set(crate::web::build_asset_hash(SIGN_OUT_SVG))
        .unwrap();

    // JS bundle hash
    JS_BUNDLE_HASH
        .set(crate::web::build_asset_hash(JS_BUNDLE))
        .unwrap();
}
pub struct TemplateResponse<T> {
    template: T,
}

impl<T: askama::Template> axum::response::IntoResponse for TemplateResponse<T> {
    fn into_response(self) -> axum::response::Response {
        match self.template.render() {
            Ok(body) => {
                let headers = [(
                    axum::http::header::CONTENT_TYPE,
                    axum::http::HeaderValue::from_static(T::MIME_TYPE),
                )];

                (headers, body).into_response()
            }
            Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

/// Convert a template into a response
pub fn into_response<T: askama::Template>(template: T) -> impl axum::response::IntoResponse {
    TemplateResponse { template }
}

/// Either left or right response for conditional handling
pub enum ResponseEither<A, B> {
    Left(A),
    Right(B),
}

impl<A: axum::response::IntoResponse, B: axum::response::IntoResponse> axum::response::IntoResponse
    for ResponseEither<A, B>
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Left(a) => a.into_response(),
            Self::Right(b) => b.into_response(),
        }
    }
}

// Helper function to find available port
pub async fn find_available_port() -> u16 {
    use std::sync::atomic::{AtomicU16, Ordering};
    use tokio::net::TcpListener;

    static PORT_COUNTER: AtomicU16 = AtomicU16::new(3333);

    for port_offset in 0..100 {
        let test_port = PORT_COUNTER.fetch_add(1, Ordering::Relaxed) + port_offset;
        if let Ok(_) = TcpListener::bind(&format!("127.0.0.1:{}", test_port)).await {
            return test_port;
        }
    }

    3333 // fallback if no ports available
}

use webbrowser;

pub fn open(host: &str, port: i32) -> Result<(), tokio::io::Error> {

let url = format!("http://{}:{}", host, port); // Correctly format with the protocol

println!("Attempting to open: {}", url);

match webbrowser::open(&url) {
    Ok(_) => println!("Successfully opened the browser to {}", url),
    Err(e) => eprintln!("Failed to open browser: {}", e),
}

Ok(())
}

#[derive(Args)]
#[command(author, version, about, long_about = None)]
struct AppArgs {
    /// Port to listen on for the main server
    #[arg(short, long, default_value_t = 3030)]
    port: u16,
}

struct WithTemplate<T: serdeSerialize> {
    name: &'static str,
    value: T,
}

fn render<T>(template: WithTemplate<T>, hbs: Arc<Handlebars<'_>>) -> impl warp::Reply
where
    T: serdeSerialize,
{
    let render = hbs
        .render(template.name, &template.value)
        .unwrap_or_else(|err| err.to_string());
    warp::reply::html(render)
}


// Define a simple structure for our response
#[derive(serdeSerialize)]
struct Message {
    text: String,
}

// 1. Define the handler function for the new path
fn get_messages() -> impl warp::Reply {
    let response = HashMap::from([
        ("status", "ok"),
        ("data", "This is the messages endpoint!")
    ]);
    warpReplayJson(&response)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct W2UiArgs {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3030)]
    pub port: u16,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct GnitArgs {
    /// Path to a directory where the RocksDB database should be stored.
    ///
    /// This directory will be created if it doesn't exist. The RocksDB database is
    /// quick to generate, so it can be pointed to temporary storage.
    #[clap(short, long, value_parser, default_value = ".gnostr/web")]
    pub db_store: PathBuf,
    /// The IP address to bind to (e.g., 127.0.0.1, 0.0.0.0).
    #[clap(long, value_parser, default_value = "127.0.0.1")]
    pub bind_address: std::net::IpAddr,
    /// The socket port to bind to (e.g., 3333).
    #[arg(
        short,
        long,
        value_parser,
        default_value = "3333",
        env = "GNOSTR_GNIT_BIND_PORT"
    )]
    pub bind_port: u16,
    /// The path in which your bare Git repositories reside.
    ///
    /// This directory will be scanned recursively for Git repositories.
    #[clap(short, long, value_parser, default_value = ".")]
    pub scan_path: PathBuf,
    /// Configures the metadata refresh interval for Git repositories (e.g., "never" or "60s").
    #[arg(long, default_value_t = RefreshInterval::Duration(std::time::Duration::from_secs(30)), env = "GNOSTR_GNIT_REFRESH_INTERVAL")]
    pub refresh_interval: RefreshInterval,
    /// Configures the request timeout for incoming HTTP requests (e.g., "10s").
    #[arg(long, default_value_t = humantime::Duration::from(std::time::Duration::from_secs(10)), env = "GNOSTR_GNIT_REQUEST_TIMEOUT")]
    pub request_timeout: humantime::Duration,
    /// debug logging
    #[clap(long, value_parser, default_value = "false")]
    pub debug: bool,
    /// info logging
    #[clap(long, value_parser, default_value = "false")]
    pub info: bool,
    /// Run the process in the background (daemonize)
    #[clap(long, value_parser, default_value = "false")]
    pub detach: bool,
}
