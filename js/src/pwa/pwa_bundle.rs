use std::collections::HashMap;

pub fn get_pwa_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert(
        "manifest.json".to_string(),
        include_bytes!("manifest.json") as &'static [u8],
    );
    assets.insert(
        "icon-256.png".to_string(),
        include_bytes!("icon-256.png") as &'static [u8],
    );
    assets.insert(
        "splash-512.png".to_string(),
        include_bytes!("splash-512.png") as &'static [u8],
    );
    assets
}

