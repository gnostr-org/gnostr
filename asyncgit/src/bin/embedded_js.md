### `src/lib/bin/embedded_js.rs` Explanation

The `src/lib/bin/embedded_js.rs` program sets up a `warp` web server to serve the shared HTML template and embedded JavaScript assets.

Here's a breakdown of how it works:

1.  **Initialization**:
    - `#[tokio::main] async fn main()`: The `main` function is asynchronous and uses `tokio` as its runtime, which is standard for `warp` applications.
    - `pretty_env_logger::init()`: Initializes a logger for better debugging output.

2.  **Shared asset loading**:
    - `get_js_assets()` returns the embedded JS map used by the server.
    - The route layer reuses the shared content-type helper from `types::bridge` instead of hardcoding MIME logic in the handler.

3.  **Template serving**:
    - The root route serves `TemplateHtml::new().to_string()` directly.
    - This avoids brittle string slicing and keeps the HTML shell source in one place.

4.  **Warp routes**:
    - `/` serves the shared template.
    - `/js/<file>` serves the embedded assets from the shared map.
    - Unknown files return `404`.
