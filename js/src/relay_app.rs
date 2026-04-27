use gnostr_relay::cli::RelayCli;
use gnostr_relay::launcher;

pub async fn run() {
    let config = RelayCli::default();
    launcher::run(config.clone(), config.config_path_if_exists(), "NOSTR")
        .await
        .expect("run relay server");
}
