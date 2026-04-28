use std::collections::HashMap;

pub fn get_css_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert(
        "responsive.css".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/css/responsive.css"))
            as &'static [u8],
    );
    assets.insert(
        "styles.css".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/css/styles.css"))
            as &'static [u8],
    );
    assets.insert(
        "utils.css".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/css/utils.css"))
            as &'static [u8],
    );
    assets.insert(
        "vars.css".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/css/vars.css"))
            as &'static [u8],
    );
    assets.insert(
        "w2ui-1.5.css".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/src/css/w2ui-1.5.css"))
            as &'static [u8],
    );
    assets
}
