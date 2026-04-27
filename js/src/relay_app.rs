use clap::Parser;
use gnostr_relay::cli::RelayCli;
use gnostr_relay::launcher;

use crate::utils::detach::{capture_detached_pid, spawn_detached_current_exe_named};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Detach and run in the background
    #[arg(long)]
    detach: bool,
}

pub async fn run() {
    let args = Args::parse();

    if args.detach {
        let pid = spawn_detached_current_exe_named(Some("gnostr-js-relay"), std::iter::empty::<&str>())
            .expect("spawn detached relay");
        let pid_path = capture_detached_pid("gnostr-js-relay", pid).expect("write detached relay pid");
        println!("spawned detached relay pid {pid} at {}", pid_path.display());
        return;
    }

    let config = RelayCli::default();
    launcher::run(config.clone(), config.config_path_if_exists(), "NOSTR")
        .await
        .expect("run relay server");
}
