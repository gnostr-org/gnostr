use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    net::IpAddr,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use axum::response::{IntoResponse, Response};
use clap::Args;
use clap::Parser;
use tokio::net::TcpListener;

use crate::app::database::schema::prefixes::{
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
        let mut db_options = rocksdb::Options::default();
        db_options.create_missing_column_families(true);
        db_options.create_if_missing(true);

        let mut commit_family_options = rocksdb::Options::default();
        commit_family_options.set_prefix_extractor(rocksdb::SliceTransform::create(
            "commit_prefix",
            |input| input.split(|&c| c == b'\0').next().unwrap_or(input),
            None,
        ));

        let mut tag_family_options = rocksdb::Options::default();
        tag_family_options.set_prefix_extractor(rocksdb::SliceTransform::create_fixed_prefix(
            std::mem::size_of::<u64>(),
        ));

        let db = rocksdb::DB::open_cf_with_opts(
            &db_options,
            &config.db_store,
            vec![
                (COMMIT_FAMILY, commit_family_options),
                (REPOSITORY_FAMILY, rocksdb::Options::default()),
                (TAG_FAMILY, tag_family_options),
                (REFERENCE_FAMILY, rocksdb::Options::default()),
                (COMMIT_COUNT_FAMILY, rocksdb::Options::default()),
            ],
        )?;

        let needs_schema_regen = match db.get("schema_version")? {
            Some(v) if v.as_slice() != crate::app::database::schema::SCHEMA_VERSION.as_bytes() => {
                Some(Some(v))
            }
            Some(_) => None,
            None => {
                db.put("schema_version", crate::app::database::schema::SCHEMA_VERSION)?;
                None
            }
        };

        if let Some(version) = needs_schema_regen {
            let old_version = version
                .as_deref()
                .map_or(Cow::Borrowed("unknown"), String::from_utf8_lossy);

            tracing::warn!(
                "Clearing outdated database ({old_version} != {})",
                crate::app::database::schema::SCHEMA_VERSION
            );

            drop(db);
            rocksdb::DB::destroy(&rocksdb::Options::default(), &config.db_store)?;
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
        crate::app::database::indexer::run(&scan_path, &db);
        tracing::info!("Finished periodic index");

        if indexer_wakeup_recv.blocking_recv().is_none() {
            break;
        }
    });

    tokio::spawn({
        #[cfg(unix)]
        let mut sighup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())
            .expect("could not subscribe to sighup");
        let build_sleeper = move || async move {
            match refresh_interval {
                RefreshInterval::Never => futures_util::future::pending().await,
                RefreshInterval::Duration(v) => tokio::time::sleep(v).await,
            };
        };

        async move {
            let bootstrap_runs = 2;
            let bootstrap_delay = Duration::from_secs(1);

            for remaining in (0..bootstrap_runs).rev() {
                #[cfg(unix)]
                {
                    tokio::select! {
                        _ = sighup.recv() => {},
                        _ = tokio::time::sleep(bootstrap_delay) => {},
                    }
                }
                #[cfg(not(unix))]
                {
                    tokio::time::sleep(bootstrap_delay).await;
                }

                if indexer_wakeup_send.send(()).await.is_err() {
                    tracing::error!(
                        "Indexing thread has died and is no longer accepting wakeup messages"
                    );
                    return;
                }

                tracing::info!("Bootstrap index pass scheduled; {remaining} remaining");
            }

            tracing::info!("Switching to steady-state index cadence");
            loop {
                #[cfg(unix)]
                {
                    tokio::select! {
                        _ = sighup.recv() => {},
                        () = build_sleeper() => {},
                    }
                }
                #[cfg(not(unix))]
                {
                    build_sleeper().await;
                }

                if indexer_wakeup_send.send(()).await.is_err() {
                    tracing::error!(
                        "Indexing thread has died and is no longer accepting wakeup messages"
                    );
                    return;
                }
            }
        }
    })
    .await
}

#[must_use]
pub fn build_asset_hash(v: &'static [u8]) -> &'static str {
    let hasher = xxhash_rust::const_xxh3::xxh3_128(v);
    let out = const_hex::encode(hasher.to_be_bytes());
    Box::leak(out.into_boxed_str())
}

/// Initializes all static asset hashes.
pub fn init_static_asset_hashes() {
    let css = {
        let theme =
            toml::from_str::<crate::theme::Theme>(include_str!("../themes/solarized_light.toml"))
                .unwrap()
                .build_css();
        Box::leak(
            format!(r#"@media (prefers-color-scheme: light){{{theme}}}"#)
                .into_boxed_str()
                .into_boxed_bytes(),
        )
    };
    super::HIGHLIGHT_CSS_BYTES.set(css).unwrap();
    super::HIGHLIGHT_CSS_HASH
        .set(build_asset_hash(css))
        .unwrap();

    let dark_css = {
        let theme =
            toml::from_str::<crate::theme::Theme>(include_str!("../themes/solarized_dark.toml"))
                .unwrap()
                .build_css();
        Box::leak(
            format!(r#"@media (prefers-color-scheme: dark){{{theme}}}"#)
                .into_boxed_str()
                .into_boxed_bytes(),
        )
    };
    super::DARK_HIGHLIGHT_CSS_BYTES.set(dark_css).unwrap();
    super::DARK_HIGHLIGHT_CSS_HASH
        .set(build_asset_hash(dark_css))
        .unwrap();

    super::ADD_RELAY_SVG_HASH.set(build_asset_hash(super::ADD_RELAY_SVG)).unwrap();
    super::CLOSE_MODAL_SVG_HASH
        .set(build_asset_hash(super::CLOSE_MODAL_SVG))
        .unwrap();
    super::CONTENT_WARNING_SVG_HASH
        .set(build_asset_hash(super::CONTENT_WARNING_SVG))
        .unwrap();
    super::EDIT_PROFILE_SVG_HASH
        .set(build_asset_hash(super::EDIT_PROFILE_SVG))
        .unwrap();
    super::EVENT_DELETE_SVG_HASH
        .set(build_asset_hash(super::EVENT_DELETE_SVG))
        .unwrap();
    super::EVENT_DETAILS_SVG_HASH
        .set(build_asset_hash(super::EVENT_DETAILS_SVG))
        .unwrap();
    super::EVENT_LIKE_SVG_HASH
        .set(build_asset_hash(super::EVENT_LIKE_SVG))
        .unwrap();
    super::EVENT_LIKED_SVG_HASH
        .set(build_asset_hash(super::EVENT_LIKED_SVG))
        .unwrap();
    super::EVENT_OPTIONS_SVG_HASH
        .set(build_asset_hash(super::EVENT_OPTIONS_SVG))
        .unwrap();
    super::EVENT_REPLY_ALL_SVG_HASH
        .set(build_asset_hash(super::EVENT_REPLY_ALL_SVG))
        .unwrap();
    super::EVENT_REPLY_SVG_HASH
        .set(build_asset_hash(super::EVENT_REPLY_SVG))
        .unwrap();
    super::EVENT_SHARE_SVG_HASH
        .set(build_asset_hash(super::EVENT_SHARE_SVG))
        .unwrap();
    super::EXPLORE_ACTIVE_SVG_HASH
        .set(build_asset_hash(super::EXPLORE_ACTIVE_SVG))
        .unwrap();
    super::EXPLORE_SVG_HASH
        .set(build_asset_hash(super::EXPLORE_SVG))
        .unwrap();
    super::FAVICON_NOTIF_ICO_HASH
        .set(build_asset_hash(super::FAVICON_NOTIF_ICO))
        .unwrap();
    super::FAVICON_ICO_HASH
        .set(build_asset_hash(super::FAVICON_ICO))
        .unwrap();
    super::GNOSTR_NOTIF_SVG_HASH
        .set(build_asset_hash(super::GNOSTR_NOTIF_SVG))
        .unwrap();
    super::GNOSTR_NOBG_SVG_HASH
        .set(build_asset_hash(super::GNOSTR_NOBG_SVG))
        .unwrap();
    super::GNOSTR_SVG_HASH
        .set(build_asset_hash(super::GNOSTR_SVG))
        .unwrap();
    super::HOME_ACTIVE_SVG_HASH
        .set(build_asset_hash(super::HOME_ACTIVE_SVG))
        .unwrap();
    super::HOME_SVG_HASH
        .set(build_asset_hash(super::HOME_SVG))
        .unwrap();
    super::ICON_MASKABLE_SVG_HASH
        .set(build_asset_hash(super::ICON_MASKABLE_SVG))
        .unwrap();
    super::ICON_ICNS_HASH
        .set(build_asset_hash(super::ICON_ICNS))
        .unwrap();
    super::ICON_SVG_HASH
        .set(build_asset_hash(super::ICON_SVG))
        .unwrap();
    super::KEY_SVG_HASH
        .set(build_asset_hash(super::KEY_SVG))
        .unwrap();
    super::LOADER_FRAGMENT_SVG_HASH
        .set(build_asset_hash(super::LOADER_FRAGMENT_SVG))
        .unwrap();
    super::LOGO_INVERTED_SVG_HASH
        .set(build_asset_hash(super::LOGO_INVERTED_SVG))
        .unwrap();
    super::LOGO_SVG_HASH
        .set(build_asset_hash(super::LOGO_SVG))
        .unwrap();
    super::MESSAGE_USER_SVG_HASH
        .set(build_asset_hash(super::MESSAGE_USER_SVG))
        .unwrap();
    super::MESSAGES_ACTIVE_SVG_HASH
        .set(build_asset_hash(super::MESSAGES_ACTIVE_SVG))
        .unwrap();
    super::MESSAGES_SVG_HASH
        .set(build_asset_hash(super::MESSAGES_SVG))
        .unwrap();
    super::NEW_NOTE_SVG_HASH
        .set(build_asset_hash(super::NEW_NOTE_SVG))
        .unwrap();
    super::NO_USER_SVG_HASH
        .set(build_asset_hash(super::NO_USER_SVG))
        .unwrap();
    super::NOTIFICATIONS_ACTIVE_SVG_HASH
        .set(build_asset_hash(super::NOTIFICATIONS_ACTIVE_SVG))
        .unwrap();
    super::NOTIFICATIONS_SVG_HASH
        .set(build_asset_hash(super::NOTIFICATIONS_SVG))
        .unwrap();
    super::OPEN_THREAD_HERE_SVG_HASH
        .set(build_asset_hash(super::OPEN_THREAD_HERE_SVG))
        .unwrap();
    super::OPEN_THREAD_SVG_HASH
        .set(build_asset_hash(super::OPEN_THREAD_SVG))
        .unwrap();
    super::PROFILE_WEBSITE_SVG_HASH
        .set(build_asset_hash(super::PROFILE_WEBSITE_SVG))
        .unwrap();
    super::PROFILE_ZAP_SVG_HASH
        .set(build_asset_hash(super::PROFILE_ZAP_SVG))
        .unwrap();
    super::PUBKEY_SVG_HASH
        .set(build_asset_hash(super::PUBKEY_SVG))
        .unwrap();
    super::READ_MORE_SVG_HASH
        .set(build_asset_hash(super::READ_MORE_SVG))
        .unwrap();
    super::SETTINGS_ACTIVE_SVG_HASH
        .set(build_asset_hash(super::SETTINGS_ACTIVE_SVG))
        .unwrap();
    super::SETTINGS_SVG_HASH
        .set(build_asset_hash(super::SETTINGS_SVG))
        .unwrap();
    super::SIGN_OUT_SVG_HASH
        .set(build_asset_hash(super::SIGN_OUT_SVG))
        .unwrap();
    super::JS_BUNDLE_HASH
        .set(build_asset_hash(super::JS_BUNDLE))
        .unwrap();
}

pub struct TemplateResponse<T> {
    template: T,
}

impl<T: askama::Template> IntoResponse for TemplateResponse<T> {
    fn into_response(self) -> Response {
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
pub fn into_response<T: askama::Template>(template: T) -> impl IntoResponse {
    TemplateResponse { template }
}

/// Either left or right response for conditional handling
pub enum ResponseEither<A, B> {
    Left(A),
    Right(B),
}

impl<A: IntoResponse, B: IntoResponse> IntoResponse for ResponseEither<A, B> {
    fn into_response(self) -> Response {
        match self {
            Self::Left(a) => a.into_response(),
            Self::Right(b) => b.into_response(),
        }
    }
}

/// Helper function to find available port
pub async fn find_available_port() -> u16 {
    use std::sync::atomic::{AtomicU16, Ordering};

    static PORT_COUNTER: AtomicU16 = AtomicU16::new(3333);

    for port_offset in 0..100 {
        let test_port = PORT_COUNTER.fetch_add(1, Ordering::Relaxed) + port_offset;
        if let Ok(_) = TcpListener::bind(&format!("127.0.0.1:{}", test_port)).await {
            return test_port;
        }
    }

    3333
}

pub fn open(host: &str, port: i32) -> Result<(), tokio::io::Error> {
    let url = format!("http://{}:{}", host, port);

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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct W2UiArgs {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3030)]
    pub port: u16,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct GnitArgs {
    /// Path to a directory where the RocksDB database should be stored.
    ///
    /// This directory will be created if it doesn't exist. The RocksDB database is
    /// quick to generate, so it can be pointed to temporary storage.
    #[arg(short, long, default_value = ".gnostr/web")]
    pub db_store: PathBuf,
    /// The IP address to bind to (e.g., 127.0.0.1, 0.0.0.0).
    #[arg(long, default_value = "127.0.0.1")]
    pub bind_address: std::net::IpAddr,
    /// The socket port to bind to (e.g., 3333).
    #[arg(short, long, default_value = "3333", env = "GNOSTR_GNIT_BIND_PORT")]
    pub bind_port: u16,
    /// The path in which your bare Git repositories reside.
    ///
    /// This directory will be scanned recursively for Git repositories.
    #[arg(short, long, default_value = ".")]
    pub scan_path: PathBuf,
    /// Configures the metadata refresh interval for Git repositories (e.g., "never" or "60s").
    #[arg(long, default_value_t = RefreshInterval::Duration(std::time::Duration::from_secs(30)), env = "GNOSTR_GNIT_REFRESH_INTERVAL")]
    pub refresh_interval: RefreshInterval,
    /// Configures the request timeout for incoming HTTP requests (e.g., "10s").
    #[arg(long, default_value_t = humantime::Duration::from(std::time::Duration::from_secs(10)), env = "GNOSTR_GNIT_REQUEST_TIMEOUT")]
    pub request_timeout: humantime::Duration,
    /// debug logging
    #[arg(long, default_value_t = false)]
    pub debug: bool,
    /// info logging
    #[arg(long, default_value_t = false)]
    pub info: bool,
    /// Run the process in the background (daemonize)
    #[arg(long, default_value_t = false)]
    pub detach: bool,
}
