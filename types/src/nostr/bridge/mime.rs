/// MIME helpers used by the static asset bridge.
///
/// This keeps the filename-to-content-type mapping in `types` so browser-facing
/// crates can reuse the same logic later without duplicating it.

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
    use super::asset_content_type;

    #[test]
    fn asset_content_types_match_extensions() {
        assert_eq!(asset_content_type("main.js"), "application/javascript");
        assert_eq!(asset_content_type("style.css"), "text/css");
        assert_eq!(asset_content_type("logo.svg"), "image/svg+xml");
    }
}
