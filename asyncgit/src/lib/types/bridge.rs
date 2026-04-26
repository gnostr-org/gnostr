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

