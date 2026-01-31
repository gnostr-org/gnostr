use std::collections::HashMap;

pub fn get_js_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert("db.js".to_string(), include_bytes!("db.js") as &'static [u8]);
    assets.insert("mempool.js".to_string(), include_bytes!("mempool.js") as &'static [u8]);
    assets.insert("browserify-cipher@1.0.1.js".to_string(), include_bytes!("browserify-cipher@1.0.1.js") as &'static [u8]);
    assets.insert("noble-secp256k1@1.2.14.js".to_string(), include_bytes!("noble-secp256k1@1.2.14.js") as &'static [u8]);
    assets.insert("buffer@6.0.3.js".to_string(), include_bytes!("buffer@6.0.3.js") as &'static [u8]);
    assets.insert("bip32@2.0.6.js".to_string(), include_bytes!("bip32@2.0.6.js") as &'static [u8]);
    assets.insert("bip39@3.0.4.js".to_string(), include_bytes!("bip39@3.0.4.js") as &'static [u8]);
    assets.insert("bitcoinjs-lib.js".to_string(), include_bytes!("bitcoinjs-lib.js") as &'static [u8]);
    assets.insert("contacts.js".to_string(), include_bytes!("contacts.js") as &'static [u8]);
    assets.insert("core.js".to_string(), include_bytes!("core.js") as &'static [u8]);
    assets.insert("event.js".to_string(), include_bytes!("event.js") as &'static [u8]);
    assets.insert("lib.js".to_string(), include_bytes!("lib.js") as &'static [u8]);
    assets.insert("main.js".to_string(), include_bytes!("main.js") as &'static [u8]);
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

    //assets.insert("w2ui-1.5.js".to_string(), include_bytes!("w2ui-1.5.js") as &'static [u8]); //manually insert

    //2.0

    assets.insert("w2base.js".to_string(), include_bytes!("w2base.js") as &'static [u8]);
    assets.insert("w2compat.js".to_string(), include_bytes!("w2compat.js") as &'static [u8]);
    assets.insert("w2field.js".to_string(), include_bytes!("w2field.js") as &'static [u8]);
    assets.insert("w2form.js".to_string(), include_bytes!("w2form.js") as &'static [u8]);
    assets.insert("w2grid.js".to_string(), include_bytes!("w2grid.js") as &'static [u8]);
    assets.insert("w2locale.js".to_string(), include_bytes!("w2locale.js") as &'static [u8]);
    assets.insert("w2popup.js".to_string(), include_bytes!("w2popup.js") as &'static [u8]);
    assets.insert("w2sidebar.js".to_string(), include_bytes!("w2sidebar.js") as &'static [u8]);
    assets.insert("w2tabs.js".to_string(), include_bytes!("w2tabs.js") as &'static [u8]);
    assets.insert("w2toolbar.js".to_string(), include_bytes!("w2toolbar.js") as &'static [u8]);
    assets.insert("w2tooltip.js".to_string(), include_bytes!("w2tooltip.js") as &'static [u8]);
    assets.insert("w2utils.js".to_string(), include_bytes!("w2utils.js") as &'static [u8]);
    assets.insert("query.js".to_string(), include_bytes!("query.js") as &'static [u8]); //manually insert
    //assets.insert("jquery.js".to_string(), include_bytes!("jquery-4.0.0.js") as &'static [u8]); //manually insert

    //defer? 
    assets.insert("w2layout.js".to_string(), include_bytes!("w2layout.js") as &'static [u8]);
    assets.insert("nostr_git_forge.js".to_string(), include_bytes!("nostr_git_forge.js") as &'static [u8]);
    assets
}
