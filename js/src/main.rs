use clap::{Parser, Subcommand};
use gnostr_relay::cli::RelayCli;

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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command.unwrap_or(Commands::Web {
        port: 3030,
        detach: false,
    }) {
        Commands::Web { port, detach } => {
            if detach {
                spawn_detached_current_exe_named(Some("gnostr-js"), ["web", "--port", &port.to_string()])
                    .expect("spawn detached web app");
                return;
            }
            gnostr_js::web_app::run(port).await;
        }
        Commands::Relay { relay, detach } => {
            if detach {
                spawn_detached_current_exe_named(
                    Some("gnostr-js"),
                    [
                        "relay",
                        "--logging",
                        relay.logging.as_str(),
                        "--config-file-path",
                        relay.config_file_path.as_str(),
                    ],
                )
                .expect("spawn detached relay");
                return;
            }
            gnostr_js::relay_app::run().await;
        }
    }
}
