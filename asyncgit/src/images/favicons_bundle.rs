use std::collections::HashMap;

pub fn get_images_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert("favicon-notif.ico".to_string(), include_bytes!("favicon-notif.ico") as &'static [u8]);
    assets.insert("favicon.ico".to_string(), include_bytes!("favicon.ico") as &'static [u8]);
    assets
}
