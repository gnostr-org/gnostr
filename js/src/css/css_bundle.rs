use std::collections::HashMap;

pub fn get_css_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert(
        "responsive.css".to_string(),
        include_bytes!("responsive.css") as &'static [u8],
    );
    assets.insert(
        "styles.css".to_string(),
        include_bytes!("styles.css") as &'static [u8],
    );
    assets.insert(
        "utils.css".to_string(),
        include_bytes!("utils.css") as &'static [u8],
    );
    assets.insert(
        "vars.css".to_string(),
        include_bytes!("vars.css") as &'static [u8],
    );
    assets
}
