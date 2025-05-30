use env_logger::Env;
use gnostr::ssh::start;
use log::error;
#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    if let Err(e) = start().await {
        error!("{:#}", e);
    }
}
