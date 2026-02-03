#[allow(unused)]
use std::net::SocketAddr;
use warp::Filter;
use std::collections::HashMap;
use std::sync::Arc;
use clap::Parser;
use gnostr_asyncgit::images::images_bundle::get_images_assets;
use gnostr_asyncgit::js::js_bundle::get_js_assets;
use gnostr_asyncgit::css::css_bundle::get_css_assets;
use gnostr_asyncgit::web::template_html::*;

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

    let template_assets = TemplateHtml::new();
    let js_assets_map = Arc::new(get_js_assets());
    let css_assets_map = Arc::new(get_css_assets());
    let images_assets_map = Arc::new(get_images_assets());
    let template_assets_map = Arc::new(template_assets.to_string());

    let script_tags = {
        let mut tags = String::new();
        let mut filenames: Vec<_> = js_assets_map.keys().cloned().collect();
        filenames.sort();
        for filename in filenames {
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
        <html>
        <head>
            <title>Embedded JS and CSS</title>
            {}
            {}
            {}
        </head>
        <body>
            <h1>Hello from Rust with embedded JS and CSS!</h1>
            {}
        <!-- we want to display all image files links in image_tags here -->
        </body>
        </html>"#,
        link_tags,
        script_tags,
        images_tags,
        template_assets_map.to_string()
    );

    let index_html_route = warp::path::end()
        .map(move || {
            warp::reply::html(index_html_content.clone())
        });

    let js_route = warp::path!("js" / String)
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
