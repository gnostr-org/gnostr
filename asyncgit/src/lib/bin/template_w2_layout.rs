use std::net::SocketAddr;
use warp::Filter;
use std::collections::HashMap;
use std::sync::Arc;
use clap::Parser;
use gnostr_asyncgit::images::images_bundle::get_images_assets;
use gnostr_asyncgit::js::js_bundle::get_js_assets;
use gnostr_asyncgit::css::css_bundle::get_css_assets;

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
-        <meta charset="utf-8">
-        <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>W2UI Demo: combo/1</title>
            {}
            {}
</head>
<body>

<div id="main" style="width: 100%; height: 400px;"></div>

        <link rel="icon" href="/images/favicon.ico" type="image/x-icon">
        </head>
        <body>
            <h1>Hello from Rust with embedded w2ui JS and CSS!</h1>
            {}
    <div id="container-busy">
      <div class="loader" title="Loading...">
        <img class="dark-invert" src="/images/loader-fragment.svg" />
        HELLO! from div
      </div>
    </div>
    <div id="container-welcome" class="hide">
      <div class="hero-box">
        <div class="padded">
          <h1>
            gnostr
            <img class="icon svg" src="/images/logo-inverted.svg" />
          </h1>
          <p>Please access with a nos2x compatible browser.</p>
        </div>
      </div>
    </div>
   <!-- Main application wrapper -->
    <div id="app-main" class="hide"></div>
<script>
let config = {{
    layout: {{
        name: 'layout',
        padding: 0,
        panels: [
            {{ type: 'left', size: 200, resizable: true, minSize: 120 }},
            {{ type: 'main', minSize: 550, overflow: 'hidden' }}
        ]
    }},
    sidebar: {{
        name: 'sidebar',
        nodes: [
            {{ id: 'general', text: 'General', group: true, expanded: true, nodes: [
                {{ id: 'grid1', text: 'Grid 1', icon: 'fa fa-pencil-square-o', selected: true }},
                {{ id: 'grid2', text: 'Grid 2', icon: 'fa fa-pencil-square-o' }},
                {{ id: 'html', text: 'Some HTML', icon: 'fa fa-list-alt' }}
            ]}}
        ],
        onClick(event) {{
            switch (event.target) {{
                case 'grid1':
                    layout.html('main', grid1)
                    break
                case 'grid2':
                    layout.html('main', grid2)
                    break
                case 'html':
                    layout.html('main', '<div style="padding: 10px">Some HTML</div>')
                    query(layout.el('main'))
                        .removeClass('w2ui-grid')
                        .css({{
                            'border-left': '1px solid #efefef'
                        }})
                    break
            }}
        }}
    }},
    grid1: {{
        name: 'grid1',
        columns: [
            {{ field: 'fname', text: 'First Name', size: '180px' }},
            {{ field: 'lname', text: 'Last Name', size: '180px' }},
            {{ field: 'email', text: 'Email', size: '40%' }},
            {{ field: 'sdate', text: 'Start Date', size: '120px' }}
        ],
        records: [
            {{ recid: 1, fname: 'John', lname: 'Doe', email: 'jdoe@gmail.com', sdate: '4/3/2012' }},
            {{ recid: 2, fname: 'Stuart', lname: 'Motzart', email: 'jdoe@gmail.com', sdate: '4/3/2012' }},
            {{ recid: 3, fname: 'Jin', lname: 'Franson', email: 'jdoe@gmail.com', sdate: '4/3/2012' }},
            {{ recid: 4, fname: 'Susan', lname: 'Ottie', email: 'jdoe@gmail.com', sdate: '4/3/2012' }},
            {{ recid: 5, fname: 'Kelly', lname: 'Silver', email: 'jdoe@gmail.com', sdate: '4/3/2012' }},
            {{ recid: 6, fname: 'Francis', lname: 'Gatos', email: 'jdoe@gmail.com', sdate: '4/3/2012' }},
            {{ recid: 7, fname: 'Mark', lname: 'Welldo', email: 'jdoe@gmail.com', sdate: '4/3/2012' }},
            {{ recid: 8, fname: 'Thomas', lname: 'Bahh', email: 'jdoe@gmail.com', sdate: '4/3/2012' }},
            {{ recid: 9, fname: 'Sergei', lname: 'Rachmaninov', email: 'jdoe@gmail.com', sdate: '4/3/2012' }}
        ]
    }},
    grid2: {{
        name: 'grid2',
        columns: [
            {{ field: 'state', text: 'State', size: '80px' }},
            {{ field: 'title', text: 'Title', size: '100%' }},
            {{ field: 'priority', text: 'Priority', size: '80px', attr: 'align="center"' }}
        ],
        records: [
            {{ recid: 1, state: 'Open', title: 'Short title for the record', priority: 2 }},
            {{ recid: 2, state: 'Open', title: 'Short title for the record', priority: 3 }},
            {{ recid: 3, state: 'Closed', title: 'Short title for the record', priority: 1 }}
        ]
    }}
}}

let layout = new w2layout(config.layout)
let sidebar = new w2sidebar(config.sidebar)
let grid1 = new w2grid(config.grid1)
let grid2 = new w2grid(config.grid2)
// initialization
layout.render('#main')
layout.html('left', sidebar)
layout.html('main', grid1)
</script>

</body>

        </html>"#,
        link_tags,
        script_tags,
        images_tags
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
