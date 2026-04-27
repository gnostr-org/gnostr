use clap::{Parser, Subcommand};
use gnostr_relay::cli::RelayCli;
use gnostr_relay::launcher;

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Run the embedded web app
    Web {
        /// Port to listen on
        #[arg(short, long, default_value_t = 3030)]
        port: u16,
    },
    /// Run the relay server using the shared relay config
    Relay {
        #[command(flatten)]
        relay: RelayCli,
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
    match args.command.unwrap_or(Commands::Web { port: 3030 }) {
        Commands::Web { port } => gnostr_js::web_app::run(port).await,
        Commands::Relay { relay } => {
            launcher::run(relay.clone(), relay.config_path_if_exists(), "NOSTR")
                .await
                .expect("run relay server");
        }
    }
}
