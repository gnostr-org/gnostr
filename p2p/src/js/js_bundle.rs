use std::collections::HashMap;

macro_rules! js_bytes {
    ($path:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../js/src/js/", $path))
            as &'static [u8]
    };
}

pub fn get_js_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert("db.js".to_string(), js_bytes!("db.js"));
    assets.insert("mempool.js".to_string(), js_bytes!("mempool.js"));
    assets.insert(
        "browserify-cipher@1.0.1.js".to_string(),
        js_bytes!("browserify-cipher@1.0.1.js"),
    );
    assets.insert(
        "noble-secp256k1@1.2.14.js".to_string(),
        js_bytes!("noble-secp256k1@1.2.14.js"),
    );
    assets.insert("buffer@6.0.3.js".to_string(), js_bytes!("buffer@6.0.3.js"));
    assets.insert("bip32@2.0.6.js".to_string(), js_bytes!("bip32@2.0.6.js"));
    assets.insert("bip39@3.0.4.js".to_string(), js_bytes!("bip39@3.0.4.js"));
    assets.insert("bitcoinjs-lib.js".to_string(), js_bytes!("bitcoinjs-lib.js"));
    assets.insert("contacts.js".to_string(), js_bytes!("contacts.js"));
    assets.insert("core.js".to_string(), js_bytes!("core.js"));
    assets.insert("postbox.js".to_string(), js_bytes!("postbox.js"));
    assets.insert("event.js".to_string(), js_bytes!("event.js"));
    assets.insert("lib.js".to_string(), js_bytes!("lib.js"));
    assets.insert("main.js".to_string(), js_bytes!("main.js"));
    assets.insert("nip/34.js".to_string(), js_bytes!("nip/34.js"));
    assets.insert("startup.js".to_string(), js_bytes!("startup.js"));
    assets.insert("relay.js".to_string(), js_bytes!("relay.js"));
    assets.insert("timers.js".to_string(), js_bytes!("timers.js"));
    assets.insert("bootstrap.js".to_string(), js_bytes!("bootstrap.js"));
    assets.insert("model.js".to_string(), js_bytes!("model.js"));
    assets.insert("nostr.js".to_string(), js_bytes!("nostr.js"));
    assets.insert("util.js".to_string(), js_bytes!("util.js"));
    assets.insert("ui/dm.js".to_string(), js_bytes!("ui/dm.js"));
    assets.insert("ui/fmt.js".to_string(), js_bytes!("ui/fmt.js"));
    assets.insert("ui/profile.js".to_string(), js_bytes!("ui/profile.js"));
    assets.insert("ui/render.js".to_string(), js_bytes!("ui/render.js"));
    assets.insert("ui/safe-html.js".to_string(), js_bytes!("ui/safe-html.js"));
    assets.insert("ui/settings.js".to_string(), js_bytes!("ui/settings.js"));
    assets.insert("ui/state.js".to_string(), js_bytes!("ui/state.js"));
    assets.insert("ui/util.js".to_string(), js_bytes!("ui/util.js"));
    assets
}
