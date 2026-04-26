use gnostr_relay::App;

#[actix_web::main]
async fn main() {
    let local_set = tokio::task::LocalSet::new();

    local_set
        .run_until(async {
            let app_data = App::create(
                Some("config/gnostr.toml"),
                true,
                Some("NOSTR".to_owned()),
                None,
            )
            .expect("create relay app");
            app_data
                .web_server()
                .expect("build relay server")
                .await
                .expect("run relay server");
        })
        .await;
}
