use clap::Parser;
use gnostr_relay::cli::RelayCli;
use gnostr_relay::launcher;

#[actix_web::main]
async fn main() {
    let args = RelayCli::parse();
    launcher::run(args.clone(), args.config_path_always(), "NOSTR")
        .await
        .expect("run relay server");
}
