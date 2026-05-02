use std::collections::HashMap;

#[cfg(gnostr_workspace_assets)]
macro_rules! asset {
    ($assets:ident, $name:literal, $path:literal) => {
        $assets.insert(
            $name.to_string(),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../js/src/js/", $path))
                as &'static [u8],
        );
    };
}

#[cfg(gnostr_workspace_assets)]
pub fn get_js_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    asset!(assets, "db.js", "db.js");
    asset!(assets, "mempool.js", "mempool.js");
    asset!(assets, "browserify-cipher@1.0.1.js", "browserify-cipher@1.0.1.js");
    asset!(assets, "noble-secp256k1@1.2.14.js", "noble-secp256k1@1.2.14.js");
    asset!(assets, "buffer@6.0.3.js", "buffer@6.0.3.js");
    asset!(assets, "bip32@2.0.6.js", "bip32@2.0.6.js");
    asset!(assets, "bip39@3.0.4.js", "bip39@3.0.4.js");
    asset!(assets, "bitcoinjs-lib.js", "bitcoinjs-lib.js");
    asset!(assets, "contacts.js", "contacts.js");
    asset!(assets, "core.js", "core.js");
    asset!(assets, "postbox.js", "postbox.js");
    asset!(assets, "event.js", "event.js");
    asset!(assets, "lib.js", "lib.js");
    asset!(assets, "main.js", "main.js");
    asset!(assets, "nip/34.js", "nip/34.js");
    asset!(assets, "startup.js", "startup.js");
    asset!(assets, "relay.js", "relay.js");
    asset!(assets, "timers.js", "timers.js");
    asset!(assets, "bootstrap.js", "bootstrap.js");
    asset!(assets, "model.js", "model.js");
    asset!(assets, "nostr.js", "nostr.js");
    asset!(assets, "util.js", "util.js");
    asset!(assets, "ui/dm.js", "ui/dm.js");
    asset!(assets, "ui/fmt.js", "ui/fmt.js");
    asset!(assets, "ui/profile.js", "ui/profile.js");
    asset!(assets, "ui/render.js", "ui/render.js");
    asset!(assets, "ui/safe-html.js", "ui/safe-html.js");
    asset!(assets, "ui/settings.js", "ui/settings.js");
    asset!(assets, "ui/state.js", "ui/state.js");
    asset!(assets, "ui/util.js", "ui/util.js");
    assets
}

#[cfg(not(gnostr_workspace_assets))]
pub fn get_js_assets() -> HashMap<String, &'static [u8]> {
    HashMap::new()
}
