use std::{env, path::PathBuf};

pub fn get_config_dir_path() -> PathBuf {
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("gnostr").join("p2p");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("gnostr").join("p2p");
    }

    PathBuf::from(".")
}

fn normalize_relay_entry(relay: &str) -> Option<String> {
    let relay = relay
        .trim()
        .trim_start_matches("- ")
        .trim_start_matches('-')
        .trim_matches('\'')
        .trim_matches('"')
        .trim();

    if relay.is_empty() {
        None
    } else {
        Some(relay.to_string())
    }
}

pub fn load_relays_or_bootstrap() -> Vec<String> {
    let config_dir = get_config_dir_path();
    let yaml_path = config_dir.join("relays.yaml");

    match std::fs::read_to_string(&yaml_path) {
        Ok(content) => content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(normalize_relay_entry)
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub fn websocket_http_url(url: &str) -> String {
    url.replace("wss://", "https://")
        .replace("ws://", "http://")
}
