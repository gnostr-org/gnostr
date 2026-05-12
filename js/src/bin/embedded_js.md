# `src/bin/embedded_js.rs`

This program serves an HTML page with embedded JavaScript assets using `warp`.

## Flow

1. **Initialization**
   - `#[tokio::main] async fn main()` starts the async runtime.
   - `pretty_env_logger::init()` enables logging.

2. **JavaScript asset loading**
   - `let js_assets_map = Arc::new(get_js_assets());` loads the JS bundle from `gnostr_js::js::js_bundle`.
   - The `Arc` lets the asset map be shared safely across async routes.

3. **Dynamic HTML generation**
   - `script_tags` is built from the JS filenames in sorted order.
   - `index_html_content` embeds those tags into the page template.

4. **Route definition**
   - `index_html_route` serves the root page at `/`.
   - `js_route` serves files from `/js/<filename>`.
   - If a file is missing, the route returns `warp::reject::not_found()`.

5. **Server startup**
   - `routes` combines the index and JS routes.
   - The server listens on `127.0.0.1:3030`.
   - `warp::serve(routes).run(addr).await;` starts the server.

## Key identifiers

- `get_js_assets()` returns the in-memory asset map.
- `Arc<HashMap<String, &'static [u8]>>` keeps assets shareable across route handlers.
- `warp::path::end()` handles the index route.
- `warp::path!("js" / String)` handles static JS asset lookup.
