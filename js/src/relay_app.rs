use clap::Parser;
use gnostr_relay::cli::RelayCli;
use gnostr_relay::launcher;

use crate::utils::detach::{
    capture_detached_pid, existing_detached_pid, relay_port_is_listening,
    spawn_detached_current_exe_named_with_env,
};

const DETACHED_ENV: &str = "GNOSTR_JS_RELAY_DETACHED";

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(flatten)]
    relay: RelayCli,

    /// Detach and run in the background
    #[arg(long)]
    detach: bool,
}

pub async fn run() {
    let args = Args::parse();
    let is_detached_child = std::env::var_os(DETACHED_ENV).is_some();

    if !is_detached_child && relay_port_is_listening(8080) {
        println!("gnostr-js relay is already listening on 127.0.0.1:8080");
        return;
    }

    if args.detach {
        if let Some(pid) = existing_detached_pid("gnostr-js-relay").expect("check detached relay pid") {
            println!("gnostr-js relay already running with pid {pid}");
            return;
        }
        let relay = args.relay.clone();
        let pid = spawn_detached_current_exe_named_with_env(
            Some("gnostr-js-relay"),
            [
                "--logging",
                relay.logging.as_str(),
                "--config-file-path",
                relay.config_file_path.as_str(),
            ],
            [(DETACHED_ENV, "1")],
        )
            .expect("spawn detached relay");
        let pid_path = capture_detached_pid("gnostr-js-relay", pid).expect("write detached relay pid");
        println!("spawned detached relay pid {pid} at {}", pid_path.display());
        return;
    }

    if !is_detached_child {
        if let Some(pid) = existing_detached_pid("gnostr-js-relay").expect("check detached relay pid") {
            println!("gnostr-js relay already running with pid {pid}");
            return;
        }
    }
    launcher::run(args.relay.clone(), args.relay.config_path_if_exists(), "NOSTR")
        .await
        .expect("run relay server");
}
