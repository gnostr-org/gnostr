pub use crate::js::js_bundle::get_js_assets;

/// Return the MIME type used for a static asset filename.
pub fn asset_content_type(filename: &str) -> &'static str {
    if filename.ends_with(".css") {
        "text/css"
    } else if filename.ends_with(".svg") {
        "image/svg+xml"
    } else if filename.ends_with(".png") {
        "image/png"
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename.ends_with(".ico") {
        "image/x-icon"
    } else if filename.ends_with(".js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    }
}

#[cfg(test)]
mod tests {
    use super::{asset_content_type, get_js_assets};

    #[test]
    fn asset_content_types_match_extensions() {
        assert_eq!(asset_content_type("main.js"), "application/javascript");
        assert_eq!(asset_content_type("style.css"), "text/css");
        assert_eq!(asset_content_type("logo.svg"), "image/svg+xml");
    }

    #[test]
    fn js_assets_are_available() {
        assert!(get_js_assets().contains_key("core.js"));
        assert!(get_js_assets().contains_key("ui/state.js"));
    }
}
