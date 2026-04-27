#[actix_web::main]
async fn main() {
    gnostr_js::relay_app::run().await;
}
