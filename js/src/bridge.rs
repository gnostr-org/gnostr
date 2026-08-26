use std::collections::HashMap;
use std::sync::Arc;

use warp::http::StatusCode;
use warp::Reply;

use crate::template_html::TemplateHtml;

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
    } else if filename.ends_with(".json") {
        "application/json"
    } else if filename.ends_with(".js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    }
}

pub fn shell_html() -> String {
    TemplateHtml::new().to_string()
}

pub fn asset_response(
    filename: String,
    assets: Arc<HashMap<String, &'static [u8]>>,
) -> warp::reply::Response {
    match assets.get(&filename) {
        Some(content) => warp::reply::with_header(
            *content,
            "Content-Type",
            asset_content_type(&filename),
        )
        .into_response(),
        None => warp::reply::with_status("Not Found", StatusCode::NOT_FOUND).into_response(),
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
        assert_eq!(asset_content_type("manifest.json"), "application/json");
    }
}
