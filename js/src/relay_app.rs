use clap::Parser;
use gnostr_relay::cli::RelayCli;
use gnostr_relay::launcher;

use crate::utils::detach::{
    capture_detached_pid, existing_detached_pid, relay_port_is_listening,
    spawn_detached_current_exe_named,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Detach and run in the background
    #[arg(long)]
    detach: bool,
}

pub async fn run() {
    let args = Args::parse();

    if relay_port_is_listening(8080) {
        println!("gnostr-js relay is already listening on 127.0.0.1:8080");
        return;
    }

    if args.detach {
        if let Some(pid) = existing_detached_pid("gnostr-js-relay").expect("check detached relay pid") {
            println!("gnostr-js relay already running with pid {pid}");
            return;
        }
        let pid = spawn_detached_current_exe_named(Some("gnostr-js-relay"), std::iter::empty::<&str>())
            .expect("spawn detached relay");
        let pid_path = capture_detached_pid("gnostr-js-relay", pid).expect("write detached relay pid");
        println!("spawned detached relay pid {pid} at {}", pid_path.display());
        return;
    }

    let config = RelayCli::default();
    if let Some(pid) = existing_detached_pid("gnostr-js-relay").expect("check detached relay pid") {
        println!("gnostr-js relay already running with pid {pid}");
        return;
    }
    launcher::run(config.clone(), config.config_path_if_exists(), "NOSTR")
        .await
        .expect("run relay server");
}
