#![cfg_attr(not(test), warn(clippy::pedantic))]
#![allow(clippy::large_futures)]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use clap::Parser;
use gnostr_ngit::{cli::Cli, cli_interactor::CliError};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = gnostr_ngit::run_cli(&cli).await;

    if let Err(err) = result {
        if err.downcast_ref::<CliError>().is_some() {
            // Already printed styled output to stderr
            std::process::exit(1);
        }
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
