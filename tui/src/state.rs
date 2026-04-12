//! Persistent TUI state — serialised to `~/.config/blossom-tui/state.json`.

use std::path::PathBuf;

use crate::{App, SortField, TAB_NAMES, NIP_TAB_NAMES};

/// User-facing configuration and UI preferences persisted between sessions.
///
/// All fields are `Option` so missing keys in a saved file are treated as
/// "not set" and fall back gracefully to env-vars or compiled defaults.
///
/// Written atomically on clean exit; loaded on startup.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TuiState {
    /// Blossom server URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    /// Secret key in hex (64 chars).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    /// Last active main tab index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab: Option<usize>,
    /// Last active NIP sub-tab index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nip_tab: Option<usize>,
    /// Blob list sort preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_field: Option<SortField>,
    /// Active blob filter string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_str: Option<String>,
    /// Whether to publish a NIP-94 event after upload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_nip94: Option<bool>,
    /// Relay URL used for NIP-94 publishing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_relay: Option<String>,
    /// NIP-34 relay URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nip34_relay: Option<String>,
    // NIP-65 relay list (kind:10002)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nip65_relays: Vec<(String, String)>, // (url, marker)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nip65_nostr_relay: Option<String>,
    // NIP-B7 server list (kind:10063)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nipb7_servers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nipb7_nostr_relay: Option<String>,
    // Profile (kind:0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_nip05: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_lud16: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_nostr_relay: Option<String>,
}

impl TuiState {
    /// Clamp tab indices to valid ranges after loading.
    pub fn clamp_tabs(&mut self) {
        if let Some(t) = self.tab {
            self.tab = Some(t.min(TAB_NAMES.len().saturating_sub(1)));
        }
        if let Some(n) = self.nip_tab {
            self.nip_tab = Some(n.min(NIP_TAB_NAMES.len().saturating_sub(1)));
        }
    }
}

/// Return the path to the state file, honouring `$XDG_CONFIG_HOME`.
pub fn state_path() -> Option<PathBuf> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok()?;
    Some(config_dir.join("blossom-tui").join("state.json"))
}

/// Load [`TuiState`] from disk. Returns a default (empty) state on any error.
pub fn load_state() -> TuiState {
    let Some(path) = state_path() else {
        return TuiState::default();
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return TuiState::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Persist [`TuiState`] to disk, creating the config directory if needed.
/// Writes atomically via a temp file + rename.
pub fn save_state(state: &TuiState) -> Result<(), Box<dyn std::error::Error>> {
    let path = state_path().ok_or("cannot determine state file path")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json.as_bytes())?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

impl App {
    /// Snapshot the persistent fields into a [`TuiState`].
    pub fn to_state(&self) -> TuiState {
        let opt_str = |s: &str| if s.is_empty() { None } else { Some(s.to_owned()) };
        TuiState {
            server: Some(self.server.clone()),
            secret_key: self.secret_key.clone(),
            tab: Some(self.tab),
            nip_tab: Some(self.nip_tab),
            sort_field: Some(self.sort_field),
            filter_str: opt_str(&self.filter_str),
            publish_nip94: Some(self.publish_nip94),
            publish_relay: opt_str(&self.publish_relay),
            nip34_relay: opt_str(&self.nip34_relay),
            nip65_relays: self.nip65_relays.clone(),
            nip65_nostr_relay: opt_str(&self.nip65_nostr_relay),
            nipb7_servers: self.nipb7_servers.clone(),
            nipb7_nostr_relay: opt_str(&self.nipb7_nostr_relay),
            profile_name: opt_str(&self.profile_name),
            profile_about: opt_str(&self.profile_about),
            profile_picture: opt_str(&self.profile_picture),
            profile_nip05: opt_str(&self.profile_nip05),
            profile_website: opt_str(&self.profile_website),
            profile_lud16: opt_str(&self.profile_lud16),
            profile_nostr_relay: opt_str(&self.profile_nostr_relay),
        }
    }

    /// Apply saved state. Call right after `App::new` before first render.
    pub fn apply_state(&mut self, state: &TuiState) {
        if let Some(t) = state.tab {
            self.tab = t.min(TAB_NAMES.len().saturating_sub(1));
        }
        if let Some(n) = state.nip_tab {
            self.nip_tab = n.min(NIP_TAB_NAMES.len().saturating_sub(1));
        }
        if let Some(sf) = state.sort_field {
            self.sort_field = sf;
        }
        if let Some(f) = &state.filter_str { self.filter_str = f.clone(); }
        if let Some(v) = state.publish_nip94 { self.publish_nip94 = v; }
        if let Some(r) = &state.publish_relay { self.publish_relay = r.clone(); }
        if let Some(r) = &state.nip34_relay { self.nip34_relay = r.clone(); }
        if !state.nip65_relays.is_empty() {
            self.nip65_relays = state.nip65_relays.clone();
        }
        if let Some(r) = &state.nip65_nostr_relay { self.nip65_nostr_relay = r.clone(); }
        if !state.nipb7_servers.is_empty() {
            self.nipb7_servers = state.nipb7_servers.clone();
        }
        if let Some(r) = &state.nipb7_nostr_relay { self.nipb7_nostr_relay = r.clone(); }
        if let Some(v) = &state.profile_name     { self.profile_name     = v.clone(); }
        if let Some(v) = &state.profile_about    { self.profile_about    = v.clone(); }
        if let Some(v) = &state.profile_picture  { self.profile_picture  = v.clone(); }
        if let Some(v) = &state.profile_nip05    { self.profile_nip05    = v.clone(); }
        if let Some(v) = &state.profile_website  { self.profile_website  = v.clone(); }
        if let Some(v) = &state.profile_lud16    { self.profile_lud16    = v.clone(); }
        if let Some(v) = &state.profile_nostr_relay {
            self.profile_nostr_relay = v.clone();
        }
    }
}
