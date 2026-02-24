use anyhow::Result;
use clap::Parser;
use gnostr_bitcoin::send_raw_tx;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Hex-encoded raw transaction you’d like to blast
    #[arg(long)]
    tx: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    let args = Args::parse();

match send_raw_tx::send_raw_transaction_to_peers(args.tx).await {
        Ok(_) => log::info!("Raw transaction sent successfully."),
        Err(e) => log::error!("Failed to send raw transaction: {}", e),
    }
 Ok(())
}
