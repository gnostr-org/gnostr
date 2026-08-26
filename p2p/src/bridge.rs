use std::collections::HashMap;
use std::sync::Arc;

use warp::http::StatusCode;
use warp::Reply;

use crate::TemplateHtml;

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
    use super::*;

    #[test]
    fn maps_common_extensions_to_content_types() {
        assert_eq!(asset_content_type("app.js"), "application/javascript");
        assert_eq!(asset_content_type("style.css"), "text/css");
        assert_eq!(asset_content_type("logo.svg"), "image/svg+xml");
        assert_eq!(asset_content_type("icon.ico"), "image/x-icon");
        assert_eq!(asset_content_type("data.json"), "application/json");
        assert_eq!(asset_content_type("binary.bin"), "application/octet-stream");
    }

    #[test]
    fn returns_asset_response_with_expected_headers() {
        let mut assets = HashMap::new();
        assets.insert("app.js".to_string(), b"console.log('ok');" as &'static [u8]);

        let response = asset_response("app.js".to_string(), Arc::new(assets));
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/javascript"
        );
    }

    #[test]
    fn returns_not_found_for_missing_asset() {
        let response = asset_response("missing.js".to_string(), Arc::new(HashMap::new()));
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn shell_html_is_template_html() {
        let html = shell_html();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains(r#"<script defer src="/js/main.js?v=1"></script>"#));
    }
}
