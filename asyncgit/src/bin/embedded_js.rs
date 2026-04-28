use std::net::SocketAddr;
use warp::Filter;
use std::collections::HashMap;
use std::sync::Arc;
use clap::Parser;
use gnostr_asyncgit::images::images_bundle::get_images_assets;
use gnostr_asyncgit::css::css_bundle::get_css_assets;
use gnostr_asyncgit::types::bridge::{asset_content_type, get_js_assets};
use gnostr_web::web::template_html::TemplateHtml;

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
    let index_html_content = TemplateHtml::new().to_string();

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
                Ok(warp::reply::with_header(content, "Content-Type", asset_content_type(&filename)))
            } else {
                Err(warp::reject::not_found())
            }
        });

    let css_route = warp::path!("css" / String)
        .and(warp::any().map(move || Arc::clone(&css_assets_map)))
        .and_then(|filename: String, assets: Arc<HashMap<String, &'static [u8]>>| async move {
            if let Some(&content) = assets.get(&filename) {
                Ok(warp::reply::with_header(content, "Content-Type", asset_content_type(&filename)))
            } else {
                Err(warp::reject::not_found())
            }
        });

    let images_route = warp::path!("images" / String)
        .and(warp::any().map(move || Arc::clone(&images_assets_map)))
        .and_then(|filename: String, assets: Arc<HashMap<String, &'static [u8]>>| async move {
            if let Some(&content) = assets.get(&filename) {
                Ok(warp::reply::with_header(content, "Content-Type", asset_content_type(&filename)))
            } else {
                Err(warp::reject::not_found())
            }
        });

    let routes = index_html_route.or(js_route).or(css_route).or(images_route);

    let addr: SocketAddr = ([127, 0, 0, 1], args.port).into();
    println!("Serving on http://{}", addr);

    warp::serve(routes).run(addr).await;
}
