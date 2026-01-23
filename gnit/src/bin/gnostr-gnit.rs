use std::{future::IntoFuture, net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    body::Body,
    http::{self, HeaderValue},
    response::Response,
    routing::get,
    Extension, Router,
};
use clap::Parser;
use const_format::formatcp;
use gnostr_gnit::{
    git::Git, layers::logger::LoggingMiddleware, methods, syntax_highlight::prime_highlighters,
};
use gnostr_gnit::{
    init_static_asset_hashes, /* build_asset_hash, */ open_database, run_indexer, Config,
    RefreshInterval, ADD_RELAY_SVG, ADD_RELAY_SVG_HASH, CLOSE_MODAL_SVG, CLOSE_MODAL_SVG_HASH,
    CONTENT_WARNING_SVG, CONTENT_WARNING_SVG_HASH, DARK_HIGHLIGHT_CSS_BYTES,
    DARK_HIGHLIGHT_CSS_HASH, EDIT_PROFILE_SVG, EDIT_PROFILE_SVG_HASH, EVENT_DELETE_SVG,
    EVENT_DELETE_SVG_HASH, EVENT_DETAILS_SVG, EVENT_DETAILS_SVG_HASH, EVENT_LIKED_SVG,
    EVENT_LIKED_SVG_HASH, EVENT_LIKE_SVG, EVENT_LIKE_SVG_HASH, EVENT_OPTIONS_SVG,
    EVENT_OPTIONS_SVG_HASH, EVENT_REPLY_ALL_SVG, EVENT_REPLY_ALL_SVG_HASH, EVENT_REPLY_SVG,
    EVENT_REPLY_SVG_HASH, EVENT_SHARE_SVG, EVENT_SHARE_SVG_HASH, EXPLORE_ACTIVE_SVG,
    EXPLORE_ACTIVE_SVG_HASH, EXPLORE_SVG, EXPLORE_SVG_HASH, FAVICON_ICO, FAVICON_ICO_HASH,
    FAVICON_NOTIF_ICO, FAVICON_NOTIF_ICO_HASH, GLOBAL_CSS, GLOBAL_CSS_HASH, GNOSTR_NOBG_SVG,
    GNOSTR_NOBG_SVG_HASH, GNOSTR_NOTIF_SVG, GNOSTR_NOTIF_SVG_HASH, GNOSTR_SVG, GNOSTR_SVG_HASH,
    HIGHLIGHT_CSS_BYTES, HIGHLIGHT_CSS_HASH, HOME_ACTIVE_SVG, HOME_ACTIVE_SVG_HASH, HOME_SVG,
    HOME_SVG_HASH, ICON_ICNS, ICON_ICNS_HASH, ICON_MASKABLE_SVG, ICON_MASKABLE_SVG_HASH, ICON_SVG,
    ICON_SVG_HASH, JS_BUNDLE, JS_BUNDLE_HASH, KEY_SVG, KEY_SVG_HASH, LOADER_FRAGMENT_SVG,
    LOADER_FRAGMENT_SVG_HASH, LOGO_INVERTED_SVG, LOGO_INVERTED_SVG_HASH, LOGO_SVG, LOGO_SVG_HASH,
    MESSAGES_ACTIVE_SVG, MESSAGES_ACTIVE_SVG_HASH, MESSAGES_SVG, MESSAGES_SVG_HASH,
    MESSAGE_USER_SVG, MESSAGE_USER_SVG_HASH, NEW_NOTE_SVG, NEW_NOTE_SVG_HASH,
    NOTIFICATIONS_ACTIVE_SVG, NOTIFICATIONS_ACTIVE_SVG_HASH, NOTIFICATIONS_SVG,
    NOTIFICATIONS_SVG_HASH, NO_USER_SVG, NO_USER_SVG_HASH, OPEN_THREAD_HERE_SVG,
    OPEN_THREAD_HERE_SVG_HASH, OPEN_THREAD_SVG, OPEN_THREAD_SVG_HASH, PROFILE_WEBSITE_SVG,
    PROFILE_WEBSITE_SVG_HASH, PROFILE_ZAP_SVG, PROFILE_ZAP_SVG_HASH, PUBKEY_SVG, PUBKEY_SVG_HASH,
    READ_MORE_SVG, READ_MORE_SVG_HASH, SETTINGS_ACTIVE_SVG, SETTINGS_ACTIVE_SVG_HASH, SETTINGS_SVG,
    SETTINGS_SVG_HASH, SIGN_OUT_SVG, SIGN_OUT_SVG_HASH,
};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};
use tower_layer::layer_fn;
use tracing::info;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Path to a directory where the RocksDB database should be stored.
    ///
    /// This directory will be created if it doesn't exist. The RocksDB database is
    /// quick to generate, so it can be pointed to temporary storage.
    #[clap(short, long, value_parser, default_value = ".gnostr/web")]
    db_store: PathBuf,
    /// The IP address to bind to (e.g., 127.0.0.1, 0.0.0.0).
    #[clap(long, value_parser, default_value = "127.0.0.1")]
    bind_address: std::net::IpAddr,
    /// The socket port to bind to (e.g., 3333).
    #[arg(
        short,
        long,
        value_parser,
        default_value = "3333",
        env = "GNOSTR_GNIT_BIND_PORT"
    )]
    bind_port: u16,
    /// The path in which your bare Git repositories reside.
    ///
    /// This directory will be scanned recursively for Git repositories.
    #[clap(short, long, value_parser, default_value = ".")]
    scan_path: PathBuf,
    /// Configures the metadata refresh interval for Git repositories (e.g., "never" or "60s").
    #[arg(long, default_value_t = RefreshInterval::Duration(std::time::Duration::from_secs(30)), env = "GNOSTR_GNIT_REFRESH_INTERVAL")]
    refresh_interval: RefreshInterval,
    /// Configures the request timeout for incoming HTTP requests (e.g., "10s").
    #[arg(long, default_value_t = humantime::Duration::from(std::time::Duration::from_secs(10)), env = "GNOSTR_GNIT_REQUEST_TIMEOUT")]
    request_timeout: humantime::Duration,
    /// debug logging
    #[clap(long, value_parser, default_value = "false")]
    debug: bool,
    /// info logging
    #[clap(long, value_parser, default_value = "false")]
    info: bool,
    /// Run the process in the background (daemonize)
    #[clap(long, value_parser, default_value = "false")]
    detach: bool,
}

async fn run_as_service() -> Result<(), anyhow::Error> {
    info!("Starting gnostr-gnit in daemon mode...");

    // Get current executable path
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    // Build daemon command arguments (remove --detach flag)
    let daemon_args = build_daemon_args();

    // Set environment variables for minimal logging
    let mut cmd = tokio::process::Command::new(&current_exe);
    cmd.args(&daemon_args);
    cmd.env("RUST_LOG", "warn");
    cmd.env("GNOSTR_GNIT_MODE", "daemon");

    // Detach from terminal
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
    }

    // Spawn daemon process
    cmd.spawn().context("Failed to spawn daemon process")?;

    println!("gnostr-gnit started in background");
    println!("Process ID: {}", std::process::id());

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn build_daemon_args() -> Vec<String> {
    let mut daemon_args = Vec::new();

    // Pass through essential arguments (remove --detach)
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--detach")
        .collect();

    for arg in args {
        daemon_args.push(arg);
    }

    // Force warn-level logging for daemon mode to reduce noise
    daemon_args.push("--debug".to_string());
    daemon_args.push("false".to_string());
    daemon_args.push("--info".to_string());
    daemon_args.push("--false".to_string());

    daemon_args
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), anyhow::Error> {
    let args: Args = Args::parse();

    // Handle service mode before any other setup
    if args.detach {
        #[cfg(unix)]
        {
            return run_as_service().await;
        }

        #[cfg(not(unix))]
        {
            return Err(anyhow!(
                "Detach functionality is currently only supported on Unix-like systems"
            ));
        }
    }

    // Set logging level based on args, only if RUST_LOG is not already set
    if std::env::var_os("RUST_LOG").is_none() {
        if args.debug {
            std::env::set_var("RUST_LOG", "debug");
        } else if args.info {
            std::env::set_var("RUST_LOG", "info");
        } else {
            std::env::set_var("RUST_LOG", "warn");
        }
    }

    let logger_layer = tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE);
    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(logger_layer)
        .init();

    let config = Config {
        db_store: args.db_store.clone(),
        scan_path: args.scan_path.clone(),
        refresh_interval: args.refresh_interval,
        bind_address: args.bind_address,
        bind_port: args.bind_port,
        request_timeout: args.request_timeout.into(),
        debug: args.debug,
        info: args.info,
        detach: args.detach,
    };

    let db = open_database(&config)?;

    let scan_path = args
        .scan_path
        .canonicalize()
        .context("Could not canonicalize scan path")?;

    let indexer_wakeup_task = run_indexer(db.clone(), scan_path.clone(), args.refresh_interval);

    let static_ico = |content: &'static [u8]| {
        move || async move {
            let mut resp = Response::new(Body::from(content));
            resp.headers_mut().insert(
                http::header::CONTENT_TYPE,
                HeaderValue::from_static("image/x-icon"),
            );
            resp
        }
    };

    let static_css = |content: &'static [u8]| {
        move || async move {
            let mut resp = Response::new(Body::from(content));
            resp.headers_mut().insert(
                http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/css"),
            );
            resp
        }
    };

    let static_svg = |content: &'static [u8]| {
        move || async move {
            let mut resp = Response::new(Body::from(content));
            resp.headers_mut().insert(
                http::header::CONTENT_TYPE,
                HeaderValue::from_static("image/svg+xml"),
            );
            resp
        }
    };

    let static_js = |content: &'static [u8]| {
        move || async move {
            let mut resp = Response::new(Body::from(content));
            resp.headers_mut().insert(
                http::header::CONTENT_TYPE,
                HeaderValue::from_static("application/javascript"),
            );
            resp
        }
    };

    info!("Priming highlighters...");
    prime_highlighters();
    init_static_asset_hashes();
    info!("Server starting up...");

    #[allow(deprecated)]
    let app = Router::new()
        .route("/", get(methods::index::handle))
        .route("/gnostr", get(methods::gnostr::handle)) // GEMINI: Add /gnostr route
        .route("/gnostr/", get(methods::gnostr::handle)) // Handle trailing slash
        .route("/thread/:thread_id", get(methods::index::handle_spa)) // Thread view
        .route("/messages", get(methods::index::handle_spa)) // Messages view
        .route("/notifications", get(methods::index::handle_spa)) // Notifications view
        .route("/nip34", get(methods::index::handle_spa)) // NIP-34 view
        .route("/nip34-global", get(methods::index::handle_spa)) // NIP-34 global view
        .route(
            "/repository-details/:repo_id",
            get(methods::index::handle_spa),
        ) // NIP-34 repository details
        .route(
            formatcp!("/style-{}.css", GLOBAL_CSS_HASH),
            get(static_css(GLOBAL_CSS)),
        )
        .route(
            &format!("/js-{}.js", JS_BUNDLE_HASH.get().unwrap()),
            get(static_js(JS_BUNDLE)),
        )
        .route(
            &format!("/highlight-{}.css", HIGHLIGHT_CSS_HASH.get().unwrap()),
            get(static_css(HIGHLIGHT_CSS_BYTES.get().unwrap())),
        )
        .route(
            &format!(
                "/highlight-dark-{}.css",
                DARK_HIGHLIGHT_CSS_HASH.get().unwrap()
            ),
            get(static_css(DARK_HIGHLIGHT_CSS_BYTES.get().unwrap())),
        )
        .route(
            &format!("/add-relay-{}.svg", ADD_RELAY_SVG_HASH.get().unwrap()),
            get(static_svg(ADD_RELAY_SVG)),
        )
        .route(
            &format!("/close-modal-{}.svg", CLOSE_MODAL_SVG_HASH.get().unwrap()),
            get(static_svg(CLOSE_MODAL_SVG)),
        )
        .route(
            &format!(
                "/content-warning-{}.svg",
                CONTENT_WARNING_SVG_HASH.get().unwrap()
            ),
            get(static_svg(CONTENT_WARNING_SVG)),
        )
        .route(
            &format!("/edit-profile-{}.svg", EDIT_PROFILE_SVG_HASH.get().unwrap()),
            get(static_svg(EDIT_PROFILE_SVG)),
        )
        .route(
            &format!("/event-delete-{}.svg", EVENT_DELETE_SVG_HASH.get().unwrap()),
            get(static_svg(EVENT_DELETE_SVG)),
        )
        .route(
            &format!(
                "/event-details-{}.svg",
                EVENT_DETAILS_SVG_HASH.get().unwrap()
            ),
            get(static_svg(EVENT_DETAILS_SVG)),
        )
        .route(
            &format!("/event-like-{}.svg", EVENT_LIKE_SVG_HASH.get().unwrap()),
            get(static_svg(EVENT_LIKE_SVG)),
        )
        .route(
            &format!("/event-liked-{}.svg", EVENT_LIKED_SVG_HASH.get().unwrap()),
            get(static_svg(EVENT_LIKED_SVG)),
        )
        .route(
            &format!(
                "/event-options-{}.svg",
                EVENT_OPTIONS_SVG_HASH.get().unwrap()
            ),
            get(static_svg(EVENT_OPTIONS_SVG)),
        )
        .route(
            &format!(
                "/event-reply-all-{}.svg",
                EVENT_REPLY_ALL_SVG_HASH.get().unwrap()
            ),
            get(static_svg(EVENT_REPLY_ALL_SVG)),
        )
        .route(
            &format!("/event-reply-{}.svg", EVENT_REPLY_SVG_HASH.get().unwrap()),
            get(static_svg(EVENT_REPLY_SVG)),
        )
        .route(
            &format!("/event-share-{}.svg", EVENT_SHARE_SVG_HASH.get().unwrap()),
            get(static_svg(EVENT_SHARE_SVG)),
        )
        .route(
            &format!(
                "/explore-active-{}.svg",
                EXPLORE_ACTIVE_SVG_HASH.get().unwrap()
            ),
            get(static_svg(EXPLORE_ACTIVE_SVG)),
        )
        .route(
            &format!("/explore-{}.svg", EXPLORE_SVG_HASH.get().unwrap()),
            get(static_svg(EXPLORE_SVG)),
        )
        .route(
            &format!(
                "/favicon-notif-{}.ico",
                FAVICON_NOTIF_ICO_HASH.get().unwrap()
            ),
            get(static_ico(FAVICON_NOTIF_ICO)),
        )
        .route(
            &format!("/favicon-{}.ico", FAVICON_ICO_HASH.get().unwrap()),
            get(static_ico(FAVICON_ICO)),
        )
        .route(
            &format!("/gnostr_notif-{}.svg", GNOSTR_NOTIF_SVG_HASH.get().unwrap()),
            get(static_svg(GNOSTR_NOTIF_SVG)),
        )
        .route(
            &format!("/gnostr-nobg-{}.svg", GNOSTR_NOBG_SVG_HASH.get().unwrap()),
            get(static_svg(GNOSTR_NOBG_SVG)),
        )
        .route(
            &format!("/gnostr-{}.svg", GNOSTR_SVG_HASH.get().unwrap()),
            get(static_svg(GNOSTR_SVG)),
        )
        .route(
            &format!("/home-active-{}.svg", HOME_ACTIVE_SVG_HASH.get().unwrap()),
            get(static_svg(HOME_ACTIVE_SVG)),
        )
        .route(
            &format!("/home-{}.svg", HOME_SVG_HASH.get().unwrap()),
            get(static_svg(HOME_SVG)),
        )
        .route(
            &format!(
                "/icon-maskable-{}.svg",
                ICON_MASKABLE_SVG_HASH.get().unwrap()
            ),
            get(static_svg(ICON_MASKABLE_SVG)),
        )
        .route(
            &format!("/icon-{}.icns", ICON_ICNS_HASH.get().unwrap()),
            get(static_ico(ICON_ICNS)),
        )
        .route(
            &format!("/icon-{}.svg", ICON_SVG_HASH.get().unwrap()),
            get(static_svg(ICON_SVG)),
        )
        .route(
            &format!("/key-{}.svg", KEY_SVG_HASH.get().unwrap()),
            get(static_svg(KEY_SVG)),
        )
        .route(
            &format!(
                "/loader-fragment-{}.svg",
                LOADER_FRAGMENT_SVG_HASH.get().unwrap()
            ),
            get(static_svg(LOADER_FRAGMENT_SVG)),
        )
        .route(
            &format!(
                "/logo-inverted-{}.svg",
                LOGO_INVERTED_SVG_HASH.get().unwrap()
            ),
            get(static_svg(LOGO_INVERTED_SVG)),
        )
        .route(
            &format!("/logo-{}.svg", LOGO_SVG_HASH.get().unwrap()),
            get(static_svg(LOGO_SVG)),
        )
        .route(
            &format!("/message-user-{}.svg", MESSAGE_USER_SVG_HASH.get().unwrap()),
            get(static_svg(MESSAGE_USER_SVG)),
        )
        .route(
            &format!(
                "/messages-active-{}.svg",
                MESSAGES_ACTIVE_SVG_HASH.get().unwrap()
            ),
            get(static_svg(MESSAGES_ACTIVE_SVG)),
        )
        .route(
            &format!("/messages-{}.svg", MESSAGES_SVG_HASH.get().unwrap()),
            get(static_svg(MESSAGES_SVG)),
        )
        .route(
            &format!("/new-note-{}.svg", NEW_NOTE_SVG_HASH.get().unwrap()),
            get(static_svg(NEW_NOTE_SVG)),
        )
        .route(
            &format!("/no-user-{}.svg", NO_USER_SVG_HASH.get().unwrap()),
            get(static_svg(NO_USER_SVG)),
        )
        .route(
            &format!(
                "/notifications-active-{}.svg",
                NOTIFICATIONS_ACTIVE_SVG_HASH.get().unwrap()
            ),
            get(static_svg(NOTIFICATIONS_ACTIVE_SVG)),
        )
        .route(
            &format!(
                "/notifications-{}.svg",
                NOTIFICATIONS_SVG_HASH.get().unwrap()
            ),
            get(static_svg(NOTIFICATIONS_SVG)),
        )
        .route(
            &format!(
                "/open-thread-here-{}.svg",
                OPEN_THREAD_HERE_SVG_HASH.get().unwrap()
            ),
            get(static_svg(OPEN_THREAD_HERE_SVG)),
        )
        .route(
            &format!("/open-thread-{}.svg", OPEN_THREAD_SVG_HASH.get().unwrap()),
            get(static_svg(OPEN_THREAD_SVG)),
        )
        .route(
            &format!(
                "/profile-website-{}.svg",
                PROFILE_WEBSITE_SVG_HASH.get().unwrap()
            ),
            get(static_svg(PROFILE_WEBSITE_SVG)),
        )
        .route(
            &format!("/profile-zap-{}.svg", PROFILE_ZAP_SVG_HASH.get().unwrap()),
            get(static_svg(PROFILE_ZAP_SVG)),
        )
        .route(
            &format!("/pubkey-{}.svg", PUBKEY_SVG_HASH.get().unwrap()),
            get(static_svg(PUBKEY_SVG)),
        )
        .route(
            &format!("/read-more-{}.svg", READ_MORE_SVG_HASH.get().unwrap()),
            get(static_svg(READ_MORE_SVG)),
        )
        .route(
            &format!(
                "/settings-active-{}.svg",
                SETTINGS_ACTIVE_SVG_HASH.get().unwrap()
            ),
            get(static_svg(SETTINGS_ACTIVE_SVG)),
        )
        .route(
            &format!("/settings-{}.svg", SETTINGS_SVG_HASH.get().unwrap()),
            get(static_svg(SETTINGS_SVG)),
        )
        .route(
            &format!("/sign-out-{}.svg", SIGN_OUT_SVG_HASH.get().unwrap()),
            get(static_svg(SIGN_OUT_SVG)),
        )
        .fallback(methods::index::handle_spa)
        .fallback(methods::repo::service)
        .layer(TimeoutLayer::new(args.request_timeout.into()))
        .layer(layer_fn(LoggingMiddleware))
        .layer(Extension(Arc::new(Git::new())))
        .layer(Extension(db))
        .layer(Extension(Arc::new(scan_path)))
        .layer(CorsLayer::new());

    println!("{}", &args.bind_port);
    let socket = SocketAddr::new(args.bind_address, args.bind_port);

    let listener = TcpListener::bind(&socket).await?;
    let app = app.into_make_service_with_connect_info::<SocketAddr>();
    let server = axum::serve(listener, app).into_future();

    tokio::select! {
        res = server => res.context("failed to run server"),
        res = indexer_wakeup_task => res.context("failed to run indexer"),
        _ = tokio::signal::ctrl_c() => {
            info!("Received ctrl-c, shutting down");
            Ok(())
        }
    }
}
