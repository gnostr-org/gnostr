use std::net::SocketAddr;
use warp::Filter;
use std::collections::HashMap;
use std::sync::Arc;
use clap::Parser;

// CONSTRUCTIONS
use gnostr_asyncgit::images::images_bundle::get_images_assets;
use gnostr_asyncgit::js::js_bundle::get_js_assets;
use gnostr_asyncgit::css::css_bundle::get_css_assets;
use gnostr_asyncgit::web::template_html::get_template_assets;
use gnostr_asyncgit::web::layout_html::get_layout_assets;

use gnostr_asyncgit::web::W2UiArgs;

#[tokio::main]
async fn main() {
    let args = W2UiArgs::parse();

    pretty_env_logger::init();

    // CONSTRUCTIONS BEGIN
    let js_assets_map = Arc::new(get_js_assets());
    let css_assets_map = Arc::new(get_css_assets());
    let images_assets_map = Arc::new(get_images_assets());
    let template_assets_map = Arc::new(get_template_assets());
    let layout_assets_map = Arc::new(get_layout_assets());

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

    // CONSTRUCTIONS END

    //1. construct the layout_html
    //2. insert other elements

    let index_html_content = format!(
        r#"<!DOCTYPE html>
    <html>
    <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>W2UI Demo: combo/1</title>
    {}
    {}
     </head>
     <body>

     <div id="main" style="width: 100%; height: 400px;"></div>

    <link rel="icon" href="/images/favicon.ico" type="image/x-icon">
    <h1>Hello from Rust with embedded w2ui JS and CSS!</h1>
    {}
    <!-- Main application wrapper -->
    <div id="app-main" class="hide">
      <!-- Global Header (e.g., for app title or global actions) -->
      <header id="global-header">
        <button
          id="global-header-gnostr-btn"
          action="open-view"
          data-view="gnostr"
          class="nav icon active"
          title="gnostr.org"
        >
          <!-- <img class="icon svg inactive" src="/images/logo-inverted.svg"/> -->
          <img class="icon svg active" src="/images/logo.svg" />
          <span id="app-title-btn-label">gnostr</span>
          <label id="app-title"></label>
        </button>
        <!-- Potentially add global actions here -->
      </header>

      <!-- List-Detail Container -->
      <div id="list-detail-container">
        <!-- Master Pane (e.g., main navigation, list of items) -->
        <div id="master-pane">
          <nav id="nav" class="nav full flex-noshrink">
            <div>
              <button action="open-view" data-view="friends" class="nav icon" title="__73Home__">
                <img class="icon svg inactive" src="/images/home.svg" />
                <img class="icon svg active" src="/images/home-active.svg" />
              </button>
              <!-- add data-view="nip34-view-friends" -->
              <button action="open-view" data-view="nip34-view-friends" class="nav icon" title="Nip34">
                <img class="icon svg inactive" src="/images/logo-inverted.svg" />
                <img class="icon svg active" src="/images/logo.svg" />
              </button>

              <button action="open-view" data-view="dm" class="nav icon" title="Direct Messages">
                <img class="icon svg inactive" src="/images/messages.svg" />
                <img class="icon svg active" src="/images/messages-active.svg" />
                <div class="new-notifications hide" role="dm"></div>
              </button>
              <button action="open-view" data-view="notifications" class="nav icon" title="Notifications">
                <img class="icon svg inactive" src="/images/notifications.svg" />
                <img class="icon svg active" src="/images/notifications-active.svg" />
                <div class="new-notifications hide" role="activity"></div>
              </button>
              <button action="open-view" data-view="settings" title="Settings" class="nav icon">
                <img class="icon svg inactive" src="/images/settings.svg" />
                <img class="icon svg active" src="/images/settings-active.svg" />
              </button>
              <button action="new-note" title="New Note" class="nav icon new-note">
                <img class="icon svg invert" src="/images/new-note.svg" />
              </button>
            </div>
          </nav>
        </div>

        <!-- Detail Pane (e.g., content for selected item/view) -->
        <div id="detail-pane">
          <div id="view">
            <header>
              <label></label>
              <div id="header-tools">
                <input type="search" id="main-search" placeholder="Search..." />
                <button class="action small hide" disabled action="mark-all-read">Mark All Read</button>
                <img class="pfp hide" role="their-pfp" data-pubkey="" src="/images/no-user.svg" />
                <img class="pfp hide" role="my-pfp" data-pubkey="" src="/images/no-user.svg" />
              </div>
            </header>
            <div id="gnostr-nip34-feed" class="events hide"></div>
            <div id="profile-info" role="profile-info" class="bottom-border hide">
              <div class="profile-banner" name="profile-banner"></div>
              <div class="flex">
                <img name="profile-image" class="pfp jumbo hide" />
                <label name="profile-nip05"></label>
                <div class="profile-tools">
                  <button class="icon link hide" name="profile-website" action="open-link">
                    <img class="icon svg" src="/images/profile-website.svg" />
                  </button>
                  <button
                    class="icon link hide"
                    title="Copy Lightning Address"
                    name="profile-lud06"
                    action="open-lud06"
                  >
                    <img class="icon svg" src="/images/profile-zap.svg" />
                  </button>
                  <button class="icon" name="message-user" title="Directly Message">
                    <img class="icon svg" src="/images/message-user.svg" />
                  </button>
                  <button class="icon" name="copy-pk" data-pk="" title="Copy Public Key">
                    <img class="icon svg" src="/images/pubkey.svg" />
                  </button>

                  <button class="action" name="follow-user" data-pk="">Follow</button>
                  <button class="action" name="edit-profile" title="Update Profile">Update</button>
                </div>
              </div>
              <div>
                <p name="profile-about"></p>
              </div>
            </div>
            <div id="dms" class="hide"></div>
            <div id="show-new" class="show-new hide" action="show-timeline-new">
              <button>Show <span>0</span> new notes</button>
            </div>
            <div id="timeline" class="events"></div>
            <div id="show-more" class="show-more">
              <button action="show-timeline-more">Show More</button>
              <button action="show-nip34-more" class="hide">Show More NIP-34</button>
            </div>
            <div class="loading-events">
              <div class="loader" title="Loading...">
                <img class="dark-invert" src="/images/loader-fragment.svg" />
              </div>
            </div>
            <div id="settings" class="hide">
              <section>
                <header>
                  <label>Relays</label>
                  <button id="add-relay" class="btn-text">
                    <img class="svg icon small" src="/images/add-relay.svg" />
                  </button>
                </header>
                <table id="relay-list" class="row">
                  <thead>
                    <tr>
                      <td class="column-address">Address</td>
                      <td class="column-status">Status</td>
                      <td class="column-ping">Ping</td>
                      <td class="column-action">Remove</td>
                    </tr>
                  </thead>
                  <tbody>
                    <!-- Dynamically generated rows should have <td> with classes: column-address, column-action -->
                  </tbody>
                </table>
              </section>
              <section>
                <header>
                  <label>NIP-65 Relays</label>
                </header>
                <table id="nip65-relay-list" class="row">
                  <thead>
                    <tr>
                      <td class="column-address">Address</td>
                      <td class="column-action">Policy</td>
                      <td class="column-action">Add</td>
                    </tr>
                  </thead>
                  <tbody>
                    <!-- Dynamically generated rows should have <td> with classes: column-address, column-action -->
                  </tbody>
                </table>
              </section>
              <section>
                <header><label>Info</label></header>
                <p>
                  <a href="https://github.com/gnostr-org/gnostr">Source Code</a>
                  <a href="https://github.com/gnostr-org/gnostr/issues">Bug Tracker</a>
                  <a href="mailto:admin@gnostr.org">Email Me</a>
                </p>
              </section>
            </div>
            <footer>
              <div id="dm-post" class="hide">
                <textarea class="post-input dm" name="message"></textarea>
                <div class="post-tools">
                  <button name="send-dm" class="action">Send</button>
                </div>
              </div>
              <nav class="nav mobile">
                <button action="open-view" data-view="friends" class="icon" title="__228Home__">
                  <img class="icon svg inactive" src="/images/home.svg" />
                  <img class="icon svg active" src="/images/home-active.svg" />
                </button>
                <button action="open-view" data-view="dm" class="icon" title="Direct Messages">
                  <img class="icon svg inactive" src="/images/messages.svg" />
                  <img class="icon svg active" src="/images/messages-active.svg" />
                  <div class="new-notifications hide" role="dm"></div>
                </button>
                <button action="open-view" data-view="notifications" class="icon" title="Notifications">
                  <img class="icon svg inactive" src="/images/notifications.svg" />
                  <img class="icon svg active" src="/images/notifications-active.svg" />
                  <div class="new-notifications hide" role="activity"></div>
                </button>
                <button action="open-view" data-view="settings" title="Settings" class="icon">
                  <img class="icon svg inactive" src="/images/settings.svg" />
                  <img class="icon svg active" src="/images/settings-active.svg" />
                </button>
                <button id="new-note-mobile" action="new-note" title="New Note" class="nav icon new-note">
                  <img class="icon svg invert" src="/images/new-note.svg" />
                </button>
              </nav>
            </footer>
          </div>
        </div>
      </div>
    </div>

    <dialog id="media-preview" action="close-media">
      <img action="close-media" src="" />
      <!-- TODO add loader to media preview -->
    </dialog>
    <dialog id="reply-modal">
      <div class="container">
        <header>
          <label>Reply To</label>
          <button class="icon" action="close-modal">
            <img class="icon svg" src="/images/close-modal.svg" />
          </button>
        </header>
        <div id="replying-to"></div>
        <div id="replybox">
          <textarea id="reply-content" class="post-input" placeholder="Reply..."></textarea>
          <div class="post-tools new">
            <button class="action" name="send">Send</button>
          </div>
          <div class="post-tools reply">
            <button class="action" name="reply-all" data-all="1">Reply All</button>
            <button class="action" name="reply">Reply</button>
          </div>
        </div>
      </div>
    </dialog>
    <dialog id="profile-editor">
      <div class="container">
        <header>
          <label>Update Profile</label>
          <button class="icon" action="close-modal">
            <img class="icon svg" src="/images/close-modal.svg" />
          </button>
        </header>
        <div>
          <input type="text" class="block w100" name="name" placeholder="Name" />
          <input type="text" class="block w100" name="display_name" placeholder="Display Name" />
          <input type="text" class="block w100" name="picture" placeholder="Picture URL" />
          <input type="text" class="block w100" name="banner" placeholder="Banner URL" />
          <input type="text" class="block w100" name="website" placeholder="Website" />
          <input type="text" class="block w100" name="lud06" placeholder="lud06" />
          <input type="text" class="block w100" name="nip05" placeholder="nip05" />
          <textarea name="about" class="block w100" placeholder="A bit about you."></textarea>
          <button class="action float-right" action="open-profile-editor">Update</button>
        </div>
      </div>
    </dialog>
    <dialog id="event-details">
      <div class="container">
        <header>
          <label>Event Details</label>
          <button class="icon modal-floating-close-btn" action="close-modal">
            <img class="icon svg" src="/images/close-modal.svg" />
          </button>
        </header>
        <div class="max-content">
          <pre><code></code></pre>
        </div>
      </div>
    </dialog>



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
