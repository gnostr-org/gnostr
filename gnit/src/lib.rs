#![deny(clippy::pedantic)]

pub mod database;
pub mod git;
pub mod layers;
pub mod methods;
pub mod syntax_highlight;
pub mod theme;
pub mod unified_diff_builder;
pub use crate::{
    database::schema::{commit::Commit, repository::Repository, tag::Tag},
    git::Git,
    syntax_highlight::prime_highlighters,
    theme::Theme,
    unified_diff_builder::UnifiedDiffBuilder,
};

// Re-export commonly used items for templates
pub use const_hex;
pub use std::sync::OnceLock;
pub use xxhash_rust;

// Constants that were previously in main.rs
pub const CRATE_VERSION: &str = clap::crate_version!();
pub const GLOBAL_CSS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/statics/css/style.css"));
pub const GLOBAL_CSS_HASH: &str = const_hex::Buffer::<16, false>::new()
    .const_format(&xxhash_rust::const_xxh3::xxh3_128(GLOBAL_CSS).to_be_bytes())
    .as_str();
pub const JS_BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/statics/js/bundle.js"));

pub static HIGHLIGHT_CSS_HASH: OnceLock<&'static str> = OnceLock::new();
pub static DARK_HIGHLIGHT_CSS_HASH: OnceLock<&'static str> = OnceLock::new();
pub static JS_BUNDLE_HASH: OnceLock<&'static str> = OnceLock::new();

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

use crate::database::schema::prefixes::{
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
        crate::database::indexer::run(&scan_path, &db);
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

/// Response wrapper for Askama templates
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
