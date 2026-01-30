use std::net::SocketAddr;
use warp::Filter;
use std::collections::HashMap;
use std::sync::Arc;
use clap::Parser;
use gnostr_js::images::images_bundle::get_images_assets;
use gnostr_js::js::js_bundle::get_js_assets;
use gnostr_js::css::css_bundle::get_css_assets;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3030)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    pretty_env_logger::init();

    let js_assets_map = Arc::new(get_js_assets());
    let css_assets_map = Arc::new(get_css_assets());
    let images_assets_map = Arc::new(get_images_assets());

    let script_tags = {
        let mut tags = String::new();
        // Explicitly load db.js, model.js, and ui/state.js first due to dependencies
        tags.push_str("<script src=\"/js/jquery-4.0.0.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/w2ui-1.5.min.js\"></script>\n");
        tags.push_str("<script src=\"/js/core.js\"></script>\n");
        tags.push_str("<script src=\"/js/db.js\"></script>\n");
        tags.push_str("<script src=\"/js/model.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/state.js\"></script>\n");

        let mut filenames: Vec<_> = js_assets_map.keys().cloned().collect();
        filenames.sort();
        for filename in filenames {
            // Skip db.js, model.js and ui/state.js as they're already added
            if filename == "core.js" ||
                filename == "db.js" ||
                filename == "model.js" ||
                filename == "ui/state.js" ||
                filename == "ui/jquery-4.0.0.js" ||
                filename == "ui/w2ui-1.5.min.js" {
                continue;
            }
            tags.push_str(&format!("<script src=\"/js/{}\"></script>\n", filename));
        }
        tags
    };

    let link_tags = {
        let mut tags = String::new();
        let mut filenames: Vec<_> = css_assets_map.keys().cloned().collect();
        filenames.sort();
        for filename in filenames {
            tags.push_str(&format!("<link rel=\"stylesheet\" href=\"/css/{}\">\n", filename));
        }
        tags
    };

    //add let images_tags
    let images_tags = {
        let mut tags = String::new();
        let mut filenames: Vec<_> = images_assets_map.keys().cloned().collect();
        filenames.sort();
        for filename in filenames {
            // Assuming images are SVG for simplicity, adjust Content-Type if other image types are needed
            // For SVG, we might embed them directly or link to them, linking is generally better.
            // Here, we'll link them as <img> tags
            tags.push_str(&format!("<img src=\"/images/{}\" alt=\"{}\">\n", filename, filename));
        }
        tags
    };

    let index_html_content = format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <title>Embedded JS and CSS</title>
        <link rel="icon" href="/images/favicon.ico" type="image/x-icon">
            {}
        </head>
        <body>
            <h1>Hello from Rust with embedded w2ui JS and CSS!</h1>
            {}
            {}
        </body>
        </html>"#,
        link_tags, // Insert CSS link tags here
        script_tags,
        images_tags // Insert image tags here
    );

    let index_html_route = warp::path::end()
        .map(move || {
            warp::reply::html(index_html_content.clone())
        });

    let js_route = warp::path("js")
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| tail.as_str().to_string())
        .and(warp::any().map(move || Arc::clone(&js_assets_map)))
        .and_then(|filename: String, assets: Arc<HashMap<String, &'static [u8]>>| async move {
            if let Some(&content) = assets.get(&filename) {
                Ok(warp::reply::with_header(content, "Content-Type", "application/javascript"))
            } else {
                Err(warp::reject::not_found())
            }
        });

    let css_route = warp::path!("css" / String)
        .and(warp::any().map(move || Arc::clone(&css_assets_map)))
        .and_then(|filename: String, assets: Arc<HashMap<String, &'static [u8]>>| async move {
            if let Some(&content) = assets.get(&filename) {
                Ok(warp::reply::with_header(content, "Content-Type", "text/css"))
            } else {
                Err(warp::reject::not_found())
            }
        });

    let images_route = warp::path!("images" / String)
        .and(warp::any().map(move || Arc::clone(&images_assets_map)))
        .and_then(|filename: String, assets: Arc<HashMap<String, &'static [u8]>>| async move {
            if let Some(&content) = assets.get(&filename) {
                let content_type = if filename.ends_with(".svg") {
                    "image/svg+xml"
                } else if filename.ends_with(".png") {
                    "image/png"
                } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                    "image/jpeg"
                } else if filename.ends_with(".ico") {
                    "image/x-icon"
                } else {
                    "application/octet-stream"
                };
                Ok(warp::reply::with_header(content, "Content-Type", content_type))
            } else {
                Err(warp::reject::not_found())
            }
        });

    let routes = index_html_route.or(js_route).or(css_route).or(images_route);

    let addr: SocketAddr = ([127, 0, 0, 1], args.port).into();
    println!("Serving on http://{}", addr);

    warp::serve(routes).run(addr).await;
}
