use std::collections::HashMap;

pub fn get_js_assets() -> HashMap<String, &'static [u8]> {
    gnostr_js::get_js_assets()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_asset_map_exposes_expected_files() {
        let assets = get_js_assets();
        assert!(assets.contains_key("main.js"));
        assert!(assets.contains_key("ui/settings.js"));
        assert!(assets.contains_key("nip/34.js"));
        assert!(!assets.get("main.js").unwrap().is_empty());
    }
}
