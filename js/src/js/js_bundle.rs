use std::collections::HashMap;

use crate::get_file_hash;

pub fn get_js_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert(
        "nip89-app.json".to_string(),
        include_bytes!(concat!(env!("OUT_DIR"), "/nip89-app.json")) as &'static [u8],
    );
    assets.insert("db.js".to_string(), include_bytes!("db.js") as &'static [u8]);
    assets.insert("mempool.js".to_string(), include_bytes!("mempool.js") as &'static [u8]);
    assets.insert("browserify-cipher@1.0.1.js".to_string(), include_bytes!("browserify-cipher@1.0.1.js") as &'static [u8]);
    assets.insert("noble-secp256k1@1.2.14.js".to_string(), include_bytes!("noble-secp256k1@1.2.14.js") as &'static [u8]);
    assets.insert("buffer@6.0.3.js".to_string(), include_bytes!("buffer@6.0.3.js") as &'static [u8]);
    assets.insert("bip32@2.0.6.js".to_string(), include_bytes!("bip32@2.0.6.js") as &'static [u8]);
    assets.insert("bip39@3.0.4.js".to_string(), include_bytes!("bip39@3.0.4.js") as &'static [u8]);
    assets.insert("bitcoinjs-lib.js".to_string(), include_bytes!("bitcoinjs-lib.js") as &'static [u8]);
    assets.insert("browser-nostr.js".to_string(), include_bytes!("browser-nostr.js") as &'static [u8]);
    assets.insert("helia-json.min.js".to_string(), include_bytes!("helia-json.min.js") as &'static [u8]);
    assets.insert("helia-unixfs.min.js".to_string(), include_bytes!("helia-unixfs.min.js") as &'static [u8]);
    assets.insert("helia-verified-fetch.min.js".to_string(), include_bytes!("helia-verified-fetch.min.js") as &'static [u8]);
    assets.insert("helia.min.js".to_string(), include_bytes!("helia.min.js") as &'static [u8]);
    assets.insert("isomorphic-git.js".to_string(), include_bytes!("isomorphic-git.js") as &'static [u8]);
    assets.insert("multiformats.min.js".to_string(), include_bytes!("multiformats.min.js") as &'static [u8]);
    assets.insert("contacts.js".to_string(), include_bytes!("contacts.js") as &'static [u8]);
    assets.insert("core.js".to_string(), include_bytes!("core.js") as &'static [u8]);
    assets.insert(
        "hashes.json".to_string(),
        include_bytes!(concat!(env!("OUT_DIR"), "/hashes.json")) as &'static [u8],
    );
    assets.insert("postbox.js".to_string(), include_bytes!("postbox.js") as &'static [u8]);
    assets.insert("event.js".to_string(), include_bytes!("event.js") as &'static [u8]);
    assets.insert("lib.js".to_string(), include_bytes!("lib.js") as &'static [u8]);
    assets.insert("main.js".to_string(), include_bytes!("main.js") as &'static [u8]);
    assets.insert("nip/34.js".to_string(), include_bytes!("nip/34.js") as &'static [u8]);
    assets.insert("startup.js".to_string(), include_bytes!("startup.js") as &'static [u8]);
    assets.insert("relay.js".to_string(), include_bytes!("relay.js") as &'static [u8]);
    assets.insert("timers.js".to_string(), include_bytes!("timers.js") as &'static [u8]);
    assets.insert("bootstrap.js".to_string(), include_bytes!("bootstrap.js") as &'static [u8]);
    assets.insert("model.js".to_string(), include_bytes!("model.js") as &'static [u8]);
    assets.insert("nostr.js".to_string(), include_bytes!("nostr.js") as &'static [u8]);
    assets.insert("util.js".to_string(), include_bytes!("util.js") as &'static [u8]);
    assets.insert("ui/dm.js".to_string(), include_bytes!("ui/dm.js") as &'static [u8]);
    assets.insert("ui/fmt.js".to_string(), include_bytes!("ui/fmt.js") as &'static [u8]);
    assets.insert("ui/profile.js".to_string(), include_bytes!("ui/profile.js") as &'static [u8]);
    assets.insert("ui/render.js".to_string(), include_bytes!("ui/render.js") as &'static [u8]);
    assets.insert("ui/safe-html.js".to_string(), include_bytes!("ui/safe-html.js") as &'static [u8]);
    assets.insert("ui/settings.js".to_string(), include_bytes!("ui/settings.js") as &'static [u8]);
    assets.insert("ui/state.js".to_string(), include_bytes!("ui/state.js") as &'static [u8]);
    assets.insert("ui/util.js".to_string(), include_bytes!("ui/util.js") as &'static [u8]);
    assets
}

pub fn get_js_hashes() -> HashMap<String, String> {
    let mut hashes = HashMap::new();
    hashes.insert("db.js".to_string(), get_file_hash!("db.js"));
    hashes.insert("mempool.js".to_string(), get_file_hash!("mempool.js"));
    hashes.insert("browserify-cipher@1.0.1.js".to_string(), get_file_hash!("browserify-cipher@1.0.1.js"));
    hashes.insert("noble-secp256k1@1.2.14.js".to_string(), get_file_hash!("noble-secp256k1@1.2.14.js"));
    hashes.insert("buffer@6.0.3.js".to_string(), get_file_hash!("buffer@6.0.3.js"));
    hashes.insert("bip32@2.0.6.js".to_string(), get_file_hash!("bip32@2.0.6.js"));
    hashes.insert("bip39@3.0.4.js".to_string(), get_file_hash!("bip39@3.0.4.js"));
    hashes.insert("bitcoinjs-lib.js".to_string(), get_file_hash!("bitcoinjs-lib.js"));
    hashes.insert("browser-nostr.js".to_string(), get_file_hash!("browser-nostr.js"));
    hashes.insert("helia-json.min.js".to_string(), get_file_hash!("helia-json.min.js"));
    hashes.insert("helia-unixfs.min.js".to_string(), get_file_hash!("helia-unixfs.min.js"));
    hashes.insert("helia-verified-fetch.min.js".to_string(), get_file_hash!("helia-verified-fetch.min.js"));
    hashes.insert("helia.min.js".to_string(), get_file_hash!("helia.min.js"));
    hashes.insert("isomorphic-git.js".to_string(), get_file_hash!("isomorphic-git.js"));
    hashes.insert("multiformats.min.js".to_string(), get_file_hash!("multiformats.min.js"));
    hashes.insert("contacts.js".to_string(), get_file_hash!("contacts.js"));
    hashes.insert("core.js".to_string(), get_file_hash!("core.js"));
    hashes.insert("postbox.js".to_string(), get_file_hash!("postbox.js"));
    hashes.insert("event.js".to_string(), get_file_hash!("event.js"));
    hashes.insert("lib.js".to_string(), get_file_hash!("lib.js"));
    hashes.insert("main.js".to_string(), get_file_hash!("main.js"));
    hashes.insert("nip/34.js".to_string(), get_file_hash!("nip/34.js"));
    hashes.insert("startup.js".to_string(), get_file_hash!("startup.js"));
    hashes.insert("relay.js".to_string(), get_file_hash!("relay.js"));
    hashes.insert("timers.js".to_string(), get_file_hash!("timers.js"));
    hashes.insert("bootstrap.js".to_string(), get_file_hash!("bootstrap.js"));
    hashes.insert("model.js".to_string(), get_file_hash!("model.js"));
    hashes.insert("nostr.js".to_string(), get_file_hash!("nostr.js"));
    hashes.insert("util.js".to_string(), get_file_hash!("util.js"));
    hashes.insert("ui/dm.js".to_string(), get_file_hash!("ui/dm.js"));
    hashes.insert("ui/fmt.js".to_string(), get_file_hash!("ui/fmt.js"));
    hashes.insert("ui/profile.js".to_string(), get_file_hash!("ui/profile.js"));
    hashes.insert("ui/render.js".to_string(), get_file_hash!("ui/render.js"));
    hashes.insert("ui/safe-html.js".to_string(), get_file_hash!("ui/safe-html.js"));
    hashes.insert("ui/settings.js".to_string(), get_file_hash!("ui/settings.js"));
    hashes.insert("ui/state.js".to_string(), get_file_hash!("ui/state.js"));
    hashes.insert("ui/util.js".to_string(), get_file_hash!("ui/util.js"));
    hashes
}
