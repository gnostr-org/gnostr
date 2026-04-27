use clap::{Parser, Subcommand};
use gnostr_relay::cli::RelayCli;
use gnostr_relay::launcher;

use gnostr_js::utils::detach::spawn_detached_current_exe_named;

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Run the embedded web app
    Web {
        /// Port to listen on
        #[arg(short, long, default_value_t = 3030)]
        port: u16,
        /// Detach and run in the background
        #[arg(long)]
        detach: bool,
    },
    /// Run the relay server using the shared relay config
    Relay {
        #[command(flatten)]
        relay: RelayCli,
        /// Detach and run in the background
        #[arg(long)]
        detach: bool,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

async fn run_web(port: u16, detach: bool) {
    if detach {
        let port_str = port.to_string();
        spawn_detached_current_exe_named(Some("gnostr-js"), ["web", "--port", port_str.as_str()])
            .expect("spawn detached web app");
        return;
    }

    gnostr_js::web_app::run(port).await;
}

async fn run_relay(relay: RelayCli, detach: bool) {
    if detach {
        let logging = relay.logging.clone();
        let config_file_path = relay.config_file_path.clone();
        spawn_detached_current_exe_named(
            Some("gnostr-js"),
            [
                "relay",
                "--logging",
                logging.as_str(),
                "--config-file-path",
                config_file_path.as_str(),
            ],
        )
        .expect("spawn detached relay");
        return;
    }

    launcher::run(relay.clone(), relay.config_path_if_exists(), "NOSTR")
        .await
        .expect("run relay server");
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command.unwrap_or(Commands::Web {
        port: 3030,
        detach: false,
    }) {
        Commands::Web { port, detach } => run_web(port, detach).await,
        Commands::Relay { relay, detach } => run_relay(relay, detach).await,
    }
}
