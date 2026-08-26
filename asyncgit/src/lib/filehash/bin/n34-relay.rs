// n34-relay - A nostr GRASP relay implementation
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://gnu.org/licenses/agpl-3.0>.

use std::{net::SocketAddr, process::ExitCode, sync::Arc};

use axum::Extension;
use hyper::{Method, header};
use tokio::signal;
use tower_http::{
    cors,
    decompression::RequestDecompressionLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

/// Relay endpoints
use n34_relay::endpoints;
/// Relay errors.
use n34_relay::errors;
// /// Extension traits
// use n34_relay::ext_traits;
/// GRASP git server
use n34_relay::git_server;
// /// Relay pathes.
// use n34_relay::pathes;
// /// Raw axum websocket
// use n34_relay::raw_websocket;
/// Our relay.
use n34_relay::relay;
/// Relay configuration.
use n34_relay::relay_config;
/// Router state
use n34_relay::router_state;
/// Some useful utils.
use n34_relay::utils;

use self::{errors::RelayResult, relay_config::RelayConfig, router_state::RouterState};

/// Sets up default logging with two outputs, stderr and a log file.
///
/// Log level for stderr is controlled by `RUST_LOG` environment variable,
/// defaults to `ERROR`. The log file always uses `TRACE` level.
fn setup_logs() -> errors::RelayResult<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(true)
                    .with_writer(std::io::stderr)
                    .without_time()
                    .with_filter(EnvFilter::from_default_env()),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_writer(utils::logs_file()?)
                    .with_file(false)
                    .with_line_number(false)
                    .with_filter(LevelFilter::TRACE),
            ),
    )
    .ok();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::SignalKind;

        signal::unix::signal(SignalKind::terminate())
            .expect("Failed to create SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn try_main() -> RelayResult<()> {
    setup_logs()?;
    let config = Arc::new(RelayConfig::reload()?);
    let relay_db = config.get_relay_db().await?;
    let n34_relay = Arc::new(relay::build_relay(Arc::clone(&config), Arc::clone(&relay_db)).await);
    let addr = SocketAddr::new(config.net.ip, config.net.port);

    tracing::debug!("Running relay with configuration: {config:#?}");
    tracing::info!("Relay is running at `{}`", n34_relay.url().await);

    let mut app = axum::Router::new()
        // main handler. GET and POST
        .route("/", axum::routing::get(endpoints::main_handler).post(endpoints::main_handler));

    if config.grasp.enable {
        tracing::info!("Git server is running");
        app = app.merge(git_server::router(&config));
    }

    app = app
        // enable cross-origin access
        .route(
            "/",
            axum::routing::options(|| async { hyper::StatusCode::NO_CONTENT }),
        )
        .layer(
            cors::CorsLayer::new()
                .allow_origin(cors::Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(RequestDecompressionLayer::new())
        .layer(Extension(Arc::new(RouterState::new(
            config, n34_relay, relay_db,
        ))));

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(err) = try_main().await {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }

    tracing::info!("Exited gracefully without any errors");
    ExitCode::SUCCESS
}
