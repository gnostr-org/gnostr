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
use gnostr_asyncgit::{
    git::Git, web::layers::logger::LoggingMiddleware, web::methods,
    web::syntax_highlight::prime_highlighters,
};
use gnostr_asyncgit::{
    web::init_static_asset_hashes, /* build_asset_hash, */ web::open_database,
    web::run_indexer, web::Config, web::RefreshInterval, web::ADD_RELAY_SVG,
    web::ADD_RELAY_SVG_HASH,
    web::CLOSE_MODAL_SVG, web::CLOSE_MODAL_SVG_HASH, web::CONTENT_WARNING_SVG,
    web::CONTENT_WARNING_SVG_HASH, web::DARK_HIGHLIGHT_CSS_BYTES, web::DARK_HIGHLIGHT_CSS_HASH,
    web::EDIT_PROFILE_SVG, web::EDIT_PROFILE_SVG_HASH, web::EVENT_DELETE_SVG,
    web::EVENT_DELETE_SVG_HASH, web::EVENT_DETAILS_SVG, web::EVENT_DETAILS_SVG_HASH,
    web::EVENT_LIKED_SVG, web::EVENT_LIKED_SVG_HASH, web::EVENT_LIKE_SVG,
    web::EVENT_LIKE_SVG_HASH, web::EVENT_OPTIONS_SVG, web::EVENT_OPTIONS_SVG_HASH,
    web::EVENT_REPLY_ALL_SVG, web::EVENT_REPLY_ALL_SVG_HASH, web::EVENT_REPLY_SVG,
    web::EVENT_REPLY_SVG_HASH, web::EVENT_SHARE_SVG, web::EVENT_SHARE_SVG_HASH,
    web::EXPLORE_ACTIVE_SVG, web::EXPLORE_ACTIVE_SVG_HASH, web::EXPLORE_SVG,
    web::EXPLORE_SVG_HASH, web::FAVICON_ICO, web::FAVICON_ICO_HASH, web::FAVICON_NOTIF_ICO,
    web::FAVICON_NOTIF_ICO_HASH, web::GLOBAL_CSS, web::GLOBAL_CSS_HASH, web::GNOSTR_NOBG_SVG,
    web::GNOSTR_NOBG_SVG_HASH, web::GNOSTR_NOTIF_SVG, web::GNOSTR_NOTIF_SVG_HASH,
    web::GNOSTR_SVG, web::GNOSTR_SVG_HASH, web::HIGHLIGHT_CSS_BYTES, web::HIGHLIGHT_CSS_HASH,
    web::HOME_ACTIVE_SVG, web::HOME_ACTIVE_SVG_HASH, web::HOME_SVG, web::HOME_SVG_HASH,
    web::ICON_ICNS, web::ICON_ICNS_HASH, web::ICON_MASKABLE_SVG, web::ICON_MASKABLE_SVG_HASH,
    web::ICON_SVG, web::ICON_SVG_HASH, web::JS_BUNDLE, web::JS_BUNDLE_HASH, web::KEY_SVG,
    web::KEY_SVG_HASH, web::LOADER_FRAGMENT_SVG, web::LOADER_FRAGMENT_SVG_HASH,
    web::LOGO_INVERTED_SVG, web::LOGO_INVERTED_SVG_HASH, web::LOGO_SVG, web::LOGO_SVG_HASH,
    web::MESSAGES_ACTIVE_SVG, web::MESSAGES_ACTIVE_SVG_HASH, web::MESSAGES_SVG,
    web::MESSAGES_SVG_HASH, web::MESSAGE_USER_SVG, web::MESSAGE_USER_SVG_HASH, web::NEW_NOTE_SVG,
    web::NEW_NOTE_SVG_HASH, web::NOTIFICATIONS_ACTIVE_SVG, web::NOTIFICATIONS_ACTIVE_SVG_HASH,
    web::NOTIFICATIONS_SVG, web::NOTIFICATIONS_SVG_HASH, web::NO_USER_SVG, web::NO_USER_SVG_HASH,
    web::OPEN_THREAD_HERE_SVG, web::OPEN_THREAD_HERE_SVG_HASH, web::OPEN_THREAD_SVG,
    web::OPEN_THREAD_SVG_HASH, web::PROFILE_WEBSITE_SVG, web::PROFILE_WEBSITE_SVG_HASH,
    web::PROFILE_ZAP_SVG, web::PROFILE_ZAP_SVG_HASH, web::PUBKEY_SVG, web::PUBKEY_SVG_HASH,
    web::READ_MORE_SVG, web::READ_MORE_SVG_HASH, web::SETTINGS_ACTIVE_SVG,
    web::SETTINGS_ACTIVE_SVG_HASH, web::SETTINGS_SVG, web::SETTINGS_SVG_HASH,
    web::SIGN_OUT_SVG, web::SIGN_OUT_SVG_HASH, web::GnitArgs
};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};
use tower_layer::layer_fn;
use tracing::info;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

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
    let args: GnitArgs = GnitArgs::parse();

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
            unsafe { std::env::set_var("RUST_LOG", "debug") };
        } else if args.info {
            unsafe { std::env::set_var("RUST_LOG", "info") };
        } else {
            unsafe { std::env::set_var("RUST_LOG", "warn") };
        }
    }

    let logger_layer = tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE);
    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(logger_layer)
        .init();

    let canonical_scan_path = args
        .scan_path
        .canonicalize()
        .context("Could not canonicalize scan path")?;

    println!("canonical_scan_path={}", &canonical_scan_path.display());
    let config = Config {
        db_store: args.db_store.clone(),
        scan_path: canonical_scan_path.clone(),
        refresh_interval: args.refresh_interval,
        bind_address: args.bind_address,
        bind_port: args.bind_port,
        request_timeout: args.request_timeout.into(),
        debug: args.debug,
        info: args.info,
        detach: args.detach,
    };

    let db = open_database(&config)?;

    let indexer_wakeup_task = run_indexer(db.clone(), canonical_scan_path.clone(), args.refresh_interval);

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
        .route("/gnostr", get(methods::gnostr::handle))
        .route("/gnostr/", get(methods::gnostr::handle)) // Handle trailing slash
        .route("/thread/:thread_id", get(methods::index::handle_spa)) // Thread view
        .route("/messages", get(methods::index::handle_spa)) // Messages view
        .route("/dm", get(methods::index::handle_spa)) // Messages view
        .route("/notifications", get(methods::index::handle_spa)) // Notifications view
        .route("/settings", get(methods::index::handle_spa)) // Settings view
        .route("/profile/:pubkey", get(methods::index::handle_spa)) // Profile view
        .route("/dm/:pubkey", get(methods::index::handle_spa)) // DM view with profile context
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
        .layer(Extension(Arc::new(canonical_scan_path)))
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
