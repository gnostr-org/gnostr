use std::net::SocketAddr;
use warp::Filter;
use std::collections::HashMap;
use std::sync::Arc;
use clap::Parser;
use gnostr_asyncgit::images::images_bundle::get_images_assets;
use gnostr_asyncgit::js::js_bundle::get_js_assets;
use gnostr_asyncgit::css::css_bundle::get_css_assets;
use gnostr_asyncgit::web::template_html::TemplateHtml;

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
        tags.push_str("<script type=\"modules\" src=\"/js/query.js\"></script>\n");
        //tags.push_str("<script src=\"/js/w2ui-2.0.js\"></script>\n");
        tags.push_str("<script src=\"/js/core.js\"></script>\n");
        tags.push_str("<script src=\"/js/db.js\"></script>\n");
        tags.push_str("<script src=\"/js/model.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/state.js\"></script>\n");

        let module_filenames: std::collections::HashSet<&str> = [
            "query.js",
            "w2base.js",
            "w2compat.js",
            "w2field.js",
            "w2form.js",
            "w2grid.js",
            "w2layout.js",
            "w2locale.js",
            "w2popup.js",
            "w2sidebar.js",
            "w2tabs.js",
            "w2toolbar.js",
            "w2tooltip.js",
            "w2utils.js",
        ].iter().cloned().collect();

        let mut filenames: Vec<_> = js_assets_map.keys().cloned().collect();
        filenames.sort();

        for filename in filenames {
            // Skip db.js, model.js and ui/state.js as they're already added
            if filename == "core.js" ||
                filename == "db.js" ||
                filename == "model.js" ||
                filename == "ui/state.js" { //||
                //filename == "jquery-4.0.0.js" ||
                //filename == "w2ui-1.5.js" {
                continue;
            }
            let script_tag = if module_filenames.contains(filename.as_str()) {
                format!("")//format!("<script type=\"module\" src=\"/js/{}\"></script>\n", filename)
            } else {
                format!("<script src=\"/js/{}\"></script>\n", filename)
            };
            tags.push_str(&script_tag);
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



    let base_html = TemplateHtml::new().to_string();
    let index_html_content = format!(
        "{}\n{}\n{}\n{}\n{}",
        &base_html[..base_html.find("    <link rel=\"stylesheet\" href=\"/css/vars.css?v=1\" />").unwrap_or(base_html.len())],
        link_tags,
        &base_html[base_html.find("    <link rel=\"stylesheet\" href=\"/css/vars.css?v=1\" />").unwrap_or(base_html.len())..base_html.find("    <script defer src=\"/js/util.js?v=5\"></script>").unwrap_or(base_html.len())],
        script_tags,
        &base_html[base_html.find("    <script defer src=\"/js/util.js?v=5\"></script>").unwrap_or(base_html.len())..],
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
