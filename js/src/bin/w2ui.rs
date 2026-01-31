use std::net::SocketAddr;
use warp::Filter;
use std::collections::HashMap;
use std::sync::Arc;
use clap::Parser;
use gnostr_js::images::images_bundle::get_images_assets;
use gnostr_js::js::js_bundle::get_js_assets;
use gnostr_js::css::css_bundle::get_css_assets;
use gnostr_js::template_w2_layout_rs::TemplateW2Layout;

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

        // Core w2ui dependencies (modules)
        tags.push_str("<script type=\"module\" src=\"/js/query.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2base.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2compat.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2field.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2form.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2grid.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2layout.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2locale.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2popup.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2sidebar.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2tabs.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2toolbar.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2tooltip.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/w2utils.js\"></script>\n");

        // Other application-specific JS files (non-modules first, then modules)
        tags.push_str("<script src=\"/js/core.js\"></script>\n");
        tags.push_str("<script src=\"/js/db.js\"></script>\n");
        tags.push_str("<script src=\"/js/model.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/state.js\"></script>\n");
        tags.push_str("<script src=\"/js/util.js\"></script>\n"); // Assuming util.js is not a module
        tags.push_str("<script src=\"/js/contacts.js\"></script>\n");
        tags.push_str("<script src=\"/js/event.js\"></script>\n");
        tags.push_str("<script src=\"/js/lib.js\"></script>\n");
        tags.push_str("<script src=\"/js/nostr.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/safe-html.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/util.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/render.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/fmt.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/profile.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/settings.js\"></script>\n");
        tags.push_str("<script src=\"/js/ui/dm.js\"></script>\n");

        tags.push_str("<script type=\"module\" src=\"/js/nostr_git_forge.js\"></script>\n");
        tags.push_str("<script type=\"module\" src=\"/js/main.js\"></script>\n");
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



    let base_html = TemplateW2Layout::new().to_string();

    let first_placeholder_idx = base_html.find("{}").expect("First placeholder not found in template_w2_layout.html");
    // Search for the second "{}" *after* the first one
    let remaining_html = &base_html[first_placeholder_idx + 1..]; // Start search after the first character of the first "{}"
    let second_placeholder_relative_idx = remaining_html.find("{}").expect("Second placeholder not found in template_w2_layout.html");
    let second_placeholder_idx = first_placeholder_idx + 1 + second_placeholder_relative_idx; // Adjust to absolute index

    let part1 = &base_html[..first_placeholder_idx];
    let part2 = &base_html[first_placeholder_idx + 2..second_placeholder_idx]; // +2 to skip "{}"
    let part3 = &base_html[second_placeholder_idx + 2..]; // +2 to skip "{}"

    let index_html_content = format!(
        "{}{}{}{}{}", // 5 placeholders for (part1, link_tags, part2, script_tags, part3)
        part1,
        link_tags,
        part2,
        script_tags,
        part3,
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
