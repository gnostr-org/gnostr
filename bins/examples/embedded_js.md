### `src/bin/embedded_js.rs` Explanation

The `src/bin/embedded_js.rs` program sets up a web server using the `warp` framework to serve an HTML page with embedded JavaScript files.

Here's a breakdown of how it works:

1.  **Initialization**:
    - `#[tokio::main] async fn main()`: The `main` function is asynchronous and uses `tokio` as its runtime, which is standard for `warp` applications.
    - `pretty_env_logger::init()`: Initializes a logger for better debugging output.

2.  **JavaScript Asset Loading**:
    - `let js_assets_map = Arc::new(get_js_assets());`: It calls `get_js_assets()` (from `gnostr_js::js::js_bundle`) to retrieve a `HashMap` where keys are JavaScript filenames (e.g., "main.js", "util.js") and values are their static byte content. This `HashMap` is then wrapped in an `Arc` (Atomic Reference Count) to allow it to be safely shared across multiple asynchronous tasks (routes) that `warp` creates.

3.  **Dynamic HTML Generation**:
    - `let script_tags = {...};`: This block dynamically generates `<script>` HTML tags for each JavaScript file found in `js_assets_map`. It iterates through the filenames, sorts them for consistent order, and creates a `<script src="/js/filename.js"></script>` tag for each.
    - `let index_html_content = format!(...);`: This string then embeds the generated `script_tags` into a basic HTML template. This ensures that the browser will request and load all the embedded JavaScript files when it loads the root page.

4.  **Warp Routes Definition**:
    - `let index_html_route = warp::path::end().map(move || { ... });`: This defines the route for the root path (`/`). When a request comes to `/`, it responds with the `index_html_content` as an HTML reply. The `move` keyword ensures that `index_html_content` is captured by the closure.
    - `let js_route = warp::path!("js" / String).and(...).and_then(...);`: This is a dynamic route designed to serve the JavaScript files.
      - `warp::path!("js" / String)`: It matches any URL that starts with `/js/` followed by a string (which will be the JavaScript filename, e.g., "main.js", "util.js").
      - `.and(warp::any().map(move || Arc::clone(&js_assets_map)))`: This part captures a clone of the `Arc` containing the `js_assets_map` for use in the next step.
      - `.and_then(|filename: String, assets: Arc<HashMap<String, &'static [u8]>>| async move { ... })`: This is the core logic for serving JS files. It receives the extracted `filename` and the `assets` map.
        - If found, it returns `warp::reply::with_header(content, "Content-Type", "application/javascript")`, serving the JS file with the correct MIME type.
        - If not found, it returns `Err(warp::reject::not_found())`, resulting in a 404 Not Found error.

5.  **Route Combination and Server Startup**:
    - `let routes = index_html_route.or(js_route);`: This combines the two defined routes. `warp` will try to match requests against `index_html_route` first, and if that doesn't match, it will try `js_route`.
    - `let addr: SocketAddr = ([127, 0, 0, 1], 3030).into();`: Sets the server to listen on `127.0.0.1:3030`.
    - `println!("Serving on http://{}", addr);`: Prints the server address to the console.
    - `warp::serve(routes).run(addr).await;`: Starts the `warp` server, listening for incoming HTTP requests on the specified address and handling them with the defined `routes`.
